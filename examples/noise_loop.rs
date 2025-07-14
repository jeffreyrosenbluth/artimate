//! # Noise Loop Example - Seamless Noise Animation
//!
//! This example demonstrates creating seamless looping animations using 3D noise.
//! It generates a grid of values that smoothly transitions from the last frame back
//! to the first frame, creating a perfect loop suitable for GIF creation.
//!
//! ## Features Demonstrated
//! - Seamless looping animation using 3D noise
//! - Frame saving for GIF creation (saves first 50 frames)
//! - 3D noise sampling with circular time parameter
//! - Grayscale image generation from noise values
//! - Integration with `tiny-skia` for rendering
//! - Mathematical mapping functions for value conversion
//!
//! ## Technical Details
//! The seamless loop is achieved by:
//! - Using 3D noise with (x, y, time) parameters
//! - Mapping time to a circle using cos() and sin() functions
//! - This ensures the noise values at frame 0 match frame N
//! - Creating a perfect loop with no visible seam
//!
//! ## Animation
//! - Smooth, organic noise patterns that flow continuously
//! - Each pixel's brightness determined by 3D noise value
//! - Perfect loop that can be played repeatedly
//! - 50 frames saved automatically as PNG files
//!
//! ## File Output
//! The example saves the first 50 frames as PNG files in Downloads/frames/
//! These can be combined into a GIF using external tools.
//!
//! ## Usage
//! ```bash
//! cargo run --example noise_loop
//! ```
//!
//! The application will automatically save frames and can be used to create
//! seamless looping animations.

use artimate::app::{App, AppMode, Config, Error};
use noise::{NoiseFn, Value};
use tiny_skia::*;

const TAU: f32 = std::f32::consts::PI * 2.0;

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = Config::with_dims(700, 700);
    let mut app = App::app(model, config, |_, model| model, draw)
        .set_title("Noise Loop")
        .set_frames_to_save(50);
    app.run()
}

fn map_range(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}

#[derive(Clone)]
struct Model {
    scale: f32,
    factor: f32,
    m: u32,
    num_frames: u32,
    margin: f32,
    noise: Value,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            scale: 0.013,
            factor: 0.01,
            m: 500,
            num_frames: 100,
            margin: 70.0,
            noise: Value::default(),
        }
    }
}

// Create a periodic noise function.
fn periodic_noise(model: &Model, p: f32, seed: f32, x: f32, y: f32) -> f32 {
    let u = seed + (TAU * p).cos();
    let v = (TAU * p).sin();
    model.noise.get([
        u as f64,
        v as f64,
        (model.scale * x) as f64,
        (model.scale * y) as f64,
    ]) as f32
}

// Offset for the the first 2 parameters of the 4d noise function.
fn offset(app: &App<AppMode, Model>, model: &Model, x: f32, y: f32) -> f32 {
    let (w, h) = app.wh_f32();
    let dist2 = (x - w / 2.0) * (x - w / 2.0) + (y - h / 2.0) * (y - h / 2.0);
    model.factor * dist2.sqrt()
}

// Color a single pixel at (x, y) with the given color.
pub fn point(pixmap: &mut Pixmap, x: f32, y: f32, color: Color) {
    let width = pixmap.width();
    let pixel_map = pixmap.pixels_mut();
    let k = y as usize * width as usize + x as usize;
    pixel_map[k] = color.premultiply().to_color_u8();
}

// Draw a single frame.
fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    let mut pixmap = Pixmap::new(app.config.width, app.config.height).unwrap();
    let t = (app.frame_count - 1) as f32 / model.num_frames as f32;
    for i in 0..model.m {
        for j in 0..model.m {
            let x = map_range(
                i as f32,
                0.0,
                model.m as f32 - 1.0,
                model.margin,
                app.config.width as f32 - model.margin,
            );
            let y = map_range(
                j as f32,
                0.0,
                model.m as f32 - 1.0,
                model.margin,
                app.config.height as f32 - model.margin,
            );
            let dx = 40.0 * periodic_noise(model, t - offset(app, model, x, y), 0.0, x, y);
            let dy = 40.0 * periodic_noise(model, t - offset(app, model, x, y), 123.0, x, y);
            point(
                &mut pixmap,
                x + dx,
                y + dy,
                Color::from_rgba8(255, 255, 255, 153),
            );
        }
    }
    pixmap.take()
}
