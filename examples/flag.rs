// Import required libraries for graphics, app management and input handling
use artimate::app::{App, AppMode, Config, Error};
use wassily::prelude::*;
use winit::keyboard::Key;

fn main() -> Result<(), Error> {
    // Initialize app configuration with 540x540 dimensions
    let config = Config::with_dims(540, 540);
    let model = Model::default();
    let mut app = App::app(model, config, |_, model| model, draw).set_title("Flag");

    // Set up keyboard controls for adjusting noise octaves (1-8)
    // Higher octaves create more detailed noise patterns
    for octaves in 1..=8 {
        let key = Key::Character(octaves.to_string().into());
        app.on_key_press(key, move |app| {
            app.model.noise = RidgedMulti::default().set_octaves(octaves);
        });
    }
    app.run()
}

// Model struct containing all the parameters for the animation
#[derive(Clone)]
struct Model {
    radius: f32,                // Size of each circle
    points: u32,                // Grid resolution (number of points per row/column)
    noise: RidgedMulti<Perlin>, // Noise generator for creating organic movement
    scale: f32,                 // Scale factor for noise
    factor: f32,                // Amplitude of the noise effect
    margin: f32,                // Space from edge of canvas
    speed: f32,                 // Animation speed
    color1: Color,              // Color of the circles
}

// Default implementation for Model with initial values
impl Default for Model {
    fn default() -> Self {
        Self {
            radius: 1.0,
            points: 75, // 75x75 grid of points
            noise: RidgedMulti::default().set_octaves(1),
            scale: 0.01,    // Small scale for smooth noise
            factor: 50.0,   // Large factor for visible movement
            margin: 60.0,   // 60px margin
            speed: 0.001,   // Slow animation speed
            color1: *WHITE, // White circles
        }
    }
}

// Generate periodic noise value for a given point
fn periodic_noise(model: &Model, p: f32, seed: f32, x: f32, y: f32) -> f32 {
    // Convert linear time into circular motion using sine and cosine
    let u = seed + (std::f32::consts::PI * 2.0 * p).cos();
    let v = (std::f32::consts::PI * 2.0 * p).sin();
    // Get 4D noise value (u, v, x, y) and scale by factor
    model.factor
        * model.noise.get([
            u as f64,
            v as f64,
            (model.scale * x) as f64,
            (model.scale * y) as f64,
        ]) as f32
}

// Draw function - renders each frame
fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    // Create new canvas and fill with black background
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    // Calculate spacing between points based on canvas width and margins
    let (w, _) = app.config.wh_f32();
    let space = (w - model.margin * 2.0) / model.points as f32;

    // Current time for animation
    let t = model.speed * app.frame_count as f32;

    // Draw grid of circles
    for i in 0..model.points {
        for j in 0..model.points {
            // Calculate base position of each circle
            let x = model.margin + i as f32 * space;
            let y = model.margin + j as f32 * space;

            // Calculate displacement using noise
            let dx = periodic_noise(model, t, 0.0, x, y); // X displacement
            let dy = periodic_noise(model, t, 123.0, x, y); // Y displacement (different seed)

            // Draw circle with calculated displacement
            Shape::new()
                .circle(pt(x + dx, y + dy), model.radius)
                .no_stroke()
                .fill_color(model.color1)
                .draw(&mut canvas);
        }
    }
    canvas.take()
}
