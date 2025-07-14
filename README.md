# Artimate

[![Crates.io](https://img.shields.io/crates/v/artimate.svg)](https://crates.io/crates/artimate)
[![Documentation](https://docs.rs/artimate/badge.svg)](https://docs.rs/artimate)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/jeffreyrosenbluth/artimate#license)

A simple, pixel-based graphics framework for Rust, perfect for creative coding, generative art, and interactive applications.

## Features

- **Simple API**: Just define a draw function that returns RGBA pixel data
- **Two modes**: Sketch mode for simple graphics, App mode for stateful applications
- **Built-in utilities**: Mouse input, time tracking, window management
- **Frame saving**: Automatically save frames as PNG files
- **Input handling**: Keyboard and mouse event handling
- **GPU-accelerated**: Uses `pixels` crate for efficient rendering

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
artimate = "0.1.0"
```

## Quick Start

### Simple Sketch

```rust
use artimate::app::{App, Config, Error};

fn main() -> Result<(), Error> {
    let config = Config::with_dims(800, 600);
    let mut app = App::sketch(config, draw);
    app.run()
}

fn draw(app: &App, _model: &()) -> Vec<u8> {
    // Create a simple gradient
    let mut pixels = vec![0u8; (app.config.width * app.config.height * 4) as usize];
    
    for y in 0..app.config.height {
        for x in 0..app.config.width {
            let i = ((y * app.config.width + x) * 4) as usize;
            pixels[i] = (x * 255 / app.config.width) as u8;     // Red
            pixels[i + 1] = (y * 255 / app.config.height) as u8; // Green
            pixels[i + 2] = 128;                                  // Blue
            pixels[i + 3] = 255;                                  // Alpha
        }
    }
    
    pixels
}
```

### Stateful Application

```rust
use artimate::app::{App, AppMode, Config, Error};

#[derive(Clone)]
struct Model {
    position: f32,
    velocity: f32,
}

fn main() -> Result<(), Error> {
    let config = Config::with_dims(800, 600);
    let model = Model { position: 0.0, velocity: 100.0 };
    let mut app = App::app(model, config, update, draw);
    app.run()
}

fn update(app: &App<AppMode, Model>, mut model: Model) -> Model {
    // Update position based on time
    model.position += model.velocity * (1.0 / 60.0); // Assuming 60 FPS
    
    // Bounce at edges
    if model.position > app.config.width as f32 || model.position < 0.0 {
        model.velocity = -model.velocity;
    }
    
    model
}

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    // Draw based on model state
    let mut pixels = vec![0u8; (app.config.width * app.config.height * 4) as usize];
    
    // Draw a moving circle
    let circle_x = model.position as u32;
    let circle_y = app.config.height / 2;
    let radius = 50;
    
    for y in 0..app.config.height {
        for x in 0..app.config.width {
            let dx = (x as i32 - circle_x as i32).abs();
            let dy = (y as i32 - circle_y as i32).abs();
            
            if dx * dx + dy * dy <= radius * radius {
                let i = ((y * app.config.width + x) * 4) as usize;
                pixels[i] = 255;     // Red
                pixels[i + 1] = 100; // Green
                pixels[i + 2] = 100; // Blue
                pixels[i + 3] = 255; // Alpha
            }
        }
    }
    
    pixels
}
```

## Configuration

Customize your application with the `Config` struct:

```rust
use artimate::app::Config;

let config = Config::with_dims(1200, 800)
    .set_title("My Artimate App")
    .set_frames_to_save(60)          // Save first 60 frames as PNG
    .set_cursor_visibility(false)    // Hide cursor
    .no_loop();                      // Render only one frame
```

## Input Handling

Handle mouse and keyboard input:

```rust
use artimate::app::{App, Config, Error};
use winit::keyboard::Key;

fn main() -> Result<(), Error> {
    let config = Config::with_dims(800, 600);
    let mut app = App::sketch(config, draw);
    
    // Handle keyboard input
    app.on_key_press(Key::Character("r".into()), |app| {
        println!("R key pressed!");
    });
    
    app.run()
}

fn draw(app: &App, _model: &()) -> Vec<u8> {
    // Use mouse position in drawing
    let mouse_x = app.mouse_x();
    let mouse_y = app.mouse_y();
    
    // ... drawing logic using mouse position
    vec![0; (app.config.width * app.config.height * 4) as usize]
}
```

## Integration with Graphics Libraries

Artimate works well with other graphics libraries that can render to pixel buffers:

- **tiny-skia**: For 2D vector graphics
- **wassily**: For creative coding utilities
- **imageproc**: For image processing
- **raqote**: For 2D graphics

## Examples

Check out the `examples/` directory for more complete examples including:

- Simple animations
- Interactive graphics
- Generative art patterns
- Integration with external libraries

## Performance

Artimate is designed for real-time graphics:
- GPU-accelerated rendering via the `pixels` crate
- Minimal overhead pixel buffer management
- Supports high frame rates for smooth animations

Performance statistics are printed when the application exits.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.