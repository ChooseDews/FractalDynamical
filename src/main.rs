extern crate image;
extern crate rayon;
extern crate crossbeam;
extern crate kdam;

use image::{ImageBuffer, Pixel, Rgb};
use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicUsize, Ordering};
use kdam::{tqdm, BarExt};
use kdam::term::Colorizer;

use crossbeam::channel;
use std::thread;
use std::sync::Arc;


use rand::prelude::*;
use rand::prelude::*;
use rayon::prelude::*;  // for parallelism



#[derive(Clone, Copy)]
struct Attractor {
    x: f64,
    y: f64,
    mass: f64,
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl Attractor {
    fn new(x: f64, y: f64, mass: f64) -> Attractor {
        Attractor { x, y, mass }
    }

    fn calculate_force(&self, point: Point) -> Point {
        let dx = self.x - point.x;
        let dy = self.y - point.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let g = 1.0;
        let force = g * self.mass / (distance * distance);
        Point {
            x: force * dx / distance,
            y: force * dy / distance,
        }
    }

    fn to_string(&self) -> String {
        format!("Attractor {{ x: {}, y: {}, mass: {} }}", self.x, self.y, self.mass)
    }
}

fn perturb(attractor: Attractor, amount: f64) -> Attractor {
    let mut rng = rand::thread_rng();
    let x = attractor.x + (rng.gen::<f64>() - 0.5) * amount;
    let y = attractor.y + (rng.gen::<f64>() - 0.5) * amount;
    let mass: f64 = attractor.mass + (rng.gen::<f64>() - 0.5) * amount;
    Attractor::new(x, y, mass)
}

fn simulate(attractors: &[Attractor], x: f64, y: f64) -> Point {
    let mut vx = 0.0;
    let mut vy = 0.0;
    let steps = 3000;
    let step_size = 0.01;
    let mut x = x;
    let mut y = y;
    let dampening = 1.0;

    //end early if we get close to an attractor
    for attractor in attractors {
        let dist = ((x - attractor.x) * (x - attractor.x) + (y - attractor.y) * (y - attractor.y)).sqrt();
        if dist < 0.1 {
            return Point { x: attractor.x, y: attractor.y };
        }
    }

    for _ in 0..steps {
        for attractor in attractors {
            let force = attractor.calculate_force(Point { x, y });
            vx += force.x * step_size;
            vy += force.y * step_size;
            let threshold = 0.02;
            let dist = ((x - attractor.x) * (x - attractor.x) + (y - attractor.y) * (y - attractor.y)).sqrt();
            if dist < threshold {
                return Point { x: attractor.x, y: attractor.y };
            }
        }
        vx *= dampening;
        vy *= dampening;
        x += vx * step_size;
        y += vy * step_size;
        let dist = (x * x + y * y).sqrt();
        let max_dist = 1500.0;
        if dist > max_dist {
            // println!("dist: {}", dist);
            return Point { x, y };
        }
    }
    Point { x, y }
}


// fn main() {

fn main(){
    // Image generation

let epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
const width: u32 = 10000;
const height: u32 = 10000;


let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
let mut attractors = vec![
    Attractor::new(1.0, 1.0, 1.0),
    Attractor::new(-1.0, -1.0, 1.0),
    //Attractor::new(-1.0, 1.0, 1.0),
    // Attractor::new(1.0, 1.0, 1.0),
];

// //perturb attractors
// for attractor in attractors.iter_mut() {
//     *attractor = perturb(*attractor, 2.0);
// }

let progress_counter = Arc::new(AtomicUsize::new(0));
let mut pb = tqdm!(total = ((width*height) as usize), colour = "gradient(#5A56E0,#EE6FF8)");
let (s, r) = channel::bounded(0);
let progress_counter_clone = Arc::clone(&progress_counter);


let handle = thread::spawn(move || {
    loop {
        pb.set_counter(progress_counter_clone.load(Ordering::Relaxed));
        pb.refresh();
        if r.try_recv().is_ok(){ break };
        thread::sleep(std::time::Duration::from_millis(200)); //update every 200ms
    }
});

let mut raw = img.into_raw();


let zoom = 3.0;
// Write image data
raw.par_chunks_mut(3).enumerate().for_each(|(i, pixel)| {
    let x = (i % width as usize) as u32;
    let y = (i / width as usize) as u32;
    let sim_x = zoom*((x as f64 / width as f64) * 2.0 - 1.0);
    let sim_y = zoom*((y as f64 / height as f64) * 2.0 - 1.0);
    let point = simulate(&attractors, sim_x, sim_y);
    //find closest attractor
    let mut closest_attractor = 0;
    let mut closest_distance = 100000.0;
    for (i, attractor) in attractors.iter().enumerate() {
        let dx = attractor.x - point.x;
        let dy = attractor.y - point.y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance < closest_distance {
            closest_distance = distance;
            closest_attractor = i;
        }
    }
    
    let length_from_origin = (point.x * point.x + point.y * point.y).sqrt();
    let max_length = 1500.0;

    // if length_from_origin > max_length {
    //     closest_attractor = 4;
  
    
    let color = match closest_attractor {
        0 => [255, 255, 255],
        1 => [0, 0, 0],
        2 => [0, 0, 255],
        3 => [255, 255, 0],
        4 => [255, 255, 255],
        _ => [0, 0, 0],
    };

    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];

    progress_counter.fetch_add(1, Ordering::Relaxed);
});

s.send(()).unwrap();
handle.join().unwrap();

let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, raw).unwrap();

let file_path = format!("./figs/attractors_{}.png", epoch);

//print

println!("file_path: {}", file_path);
// Save the image using ImageBuffer's save method
img.save_with_format(file_path, image::ImageFormat::Png).unwrap();
}