//! # Simple Moving Ball Example
//!
//! This example demonstrates the basics of creating a simple animated sketch with Artimate.
//! It shows a blue circle with an orange border moving horizontally across the screen.
//!
//! ## Features Demonstrated
//! - Using `SketchMode` for simple animations
//! - Time-based animation (`app.time`)
//! - Integration with the `wassily` graphics library
//! - Drawing shapes with fills and strokes
//! - Using default configuration settings
//!
//! ## Animation
//! - The ball moves continuously from left to right
//! - When it reaches the right edge, it wraps around to the left
//! - The movement is smooth and time-based
//!
//! ## Usage
//! ```bash
//! cargo run --example wassily_simple
//! ```

use artimate::app::{App, Config, Error};
use wassily::prelude::*;

fn main() -> Result<(), Error> {
    // Default size is 1080 x 700.
    let config = Config::default();
    let mut app = App::sketch(config, draw).set_title("Ball");
    app.run()
}

fn draw(app: &App, _model: &()) -> Vec<u8> {
    let pos = pt(
        100.0 * app.time % app.config.width as f32,
        app.config.height as f32 / 2.0,
    );
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);
    Shape::new()
        .circle(pos, 75.0)
        .fill_color(*CORNFLOWERBLUE)
        .stroke_color(*ORANGERED)
        .stroke_weight(3.0)
        .draw(&mut canvas);
    canvas.take()
}
