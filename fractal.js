const { createCanvas } = require('canvas');

// canvas setup
const width = 1500;
const height = 1500;
const canvas = createCanvas(width, height);
const context = canvas.getContext('2d');

// define the positions of the attractors
const attractors = [
    { x: -1, y: -1, mass: 1 },
    { x: 1, y: -1, mass: 1 },
    { x: -1, y: 1, mass: 1 },
    { x: 1, y: 1, mass: 1 },
];

//randomly perturb the attractors
for (const attractor of attractors) {
    attractor.x += Math.random() * 0.2
    attractor.y += Math.random() * 0.2
    attractor.mass += Math.random() * 0.2
}

// define the number of steps and step size for the simulation
const steps = 1000;
const stepSize = 0.01 / 2;

// define the gravitational constant
const G = 1;

// calculate the force between two points
const calculateForce = (point, attractor) => {
    const dx = attractor.x - point.x;
    const dy = attractor.y - point.y;
    const distance = Math.sqrt(dx * dx + dy * dy);
    const force = G * attractor.mass / (distance * distance);
    return { x: force * dx / distance, y: force * dy / distance };
};

// simulate the system for a given initial condition
const simulate = (x, y) => {
    // the point starts at rest

    //check if the point is close to an attractor to begin with
    let threshold = 0.1
    for (const attractor of attractors) {
        dist = Math.sqrt((x - attractor.x) ** 2 + (y - attractor.y) ** 2)
        if (dist < threshold) return { x: attractor.x, y: attractor.y }
    }

    let vx = 0;
    let vy = 0;
    let dampening = 0.9999;
    for (let i = 0; i < steps; i++) {
        for (const attractor of attractors) {
            const force = calculateForce({ x, y }, attractor);
            vx += force.x * stepSize;
            vy += force.y * stepSize;
            //end early if the points is extremely close to an attractor
            threshold = 0.02
            if (Math.abs(x - attractor.x) < threshold && Math.abs(y - attractor.y) < threshold) {
                return { x: attractor.x, y: attractor.y }
            }
        }
        //dampen the velocity
        vx *= dampening;
        vy *= dampening;
        x += vx * stepSize;
        y += vy * stepSize;

        //check if off in infinity
        dist = Math.sqrt(x ** 2 + y ** 2)
        force_mag = Math.sqrt(vx ** 2 + vy ** 2)
        max_dist = 2000
        if (dist > max_dist || force_mag < 0.0001) return { x, y }


    }

    return { x, y, stable: true };
};

//zoom in
const zoom = 1.5;
// plot each pixel
for (let i = 0; i < width; i++) {
    for (let j = 0; j < height; j++) {
        // scale the pixel position to the initial conditions
        const x = ((i / width) * 2 - 1) * zoom;
        const y = ((j / height) * 2 - 1) * zoom;
        // simulate the system
        const { x: xf, y: yf, stable } = simulate(x, y);
        // choose the color based on which attractor it ends nearest to
        let nearestAttractorIndex = 0;
        let nearestAttractorDistance = Infinity;
        for (let k = 0; k < attractors.length; k++) {
            const dx = xf - attractors[k].x;
            const dy = yf - attractors[k].y;
            const distance = dx * dx + dy * dy;
            if (distance < nearestAttractorDistance) {
                nearestAttractorDistance = distance;
                nearestAttractorIndex = k;
            }
        }

        if (stable == true) {
            nearestAttractorIndex = 4; //white maybe stable oribit
        }

        if (nearestAttractorDistance > 2000) {
            nearestAttractorIndex = 5; //black far away
        }

        const colors = ['red', 'green', 'blue', 'yellow', 'white', 'black'];
        const color = colors[nearestAttractorIndex];
        context.fillStyle = color;
        context.fillRect(i, j, 1, 1);
    }
}

// write output image file
const fs = require('fs');
const epoch = new Date().getTime();
const out = fs.createWriteStream(__dirname + `/figs/attractors_${epoch}.png`);
const stream = canvas.createPNGStream();
stream.pipe(out);
out.on('finish', () => console.log('The PNG file was created.'));
