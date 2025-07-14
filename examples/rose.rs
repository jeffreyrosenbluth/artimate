//! # Rose Example - Mathematical Rose Curve Generator
//!
//! This example generates beautiful mathematical rose curves (rhodonea) using polar equations.
//! It demonstrates complex mathematical visualization with interactive controls for exploring
//! different rose patterns, colors, and styles.
//!
//! ## Features Demonstrated
//! - Complex mathematical curve generation
//! - Interactive parameter control via keyboard
//! - Multiple drawing styles and color schemes
//! - Fourier series approximation
//! - Real-time parameter adjustment
//! - Mathematical rose curve variations (regular, maurer, irrational)
//!
//! ## Mathematical Background
//! Rose curves are defined by the polar equation: r = cos(k*θ) or r = sin(k*θ)
//! - When k is rational (p/q), the rose has p or 2p petals
//! - When k is irrational, the rose never closes and creates complex patterns
//! - Maurer roses add an additional rotation parameter for more complexity
//!
//! ## Controls
//! - **n/d**: Adjust n parameter and degrees
//! - **c/f**: Change color and fourier seeds
//! - **a/m/r/w**: Adjust scale, density, rotation, stroke weight
//! - **i/h**: Toggle irrational and maurer modes
//! - **s**: Cycle through drawing styles
//! - **space**: Switch control modes
//! - **enter**: Regenerate with new random parameters
//! - **p**: Print current parameters
//!
//! ## Usage
//! ```bash
//! cargo run --example rose
//! ```

use artimate::app::{App, AppMode, Config, Error};
use num_complex::Complex;
use std::ops::{Add, Mul};
use wassily::prelude::*;
use winit::keyboard::Key;

const LINES: u32 = 3600;
const COLOR_SEED: u64 = 95;
const FOURIER_SEED: u64 = 0;

fn message(model: &Model) {
    println!();
    println!(" Control Mode  :  {:?}", model.control);
    println!(" Style         :  {:?}", model.style);
    println!("┌──────────────┬───┬────────────┐");
    println!("│ n            │ n │ {:<10} │", format!("{:.2}", model.n));
    println!(
        "│ degrees      │ d │ {:<10} │",
        format!("{:.1}", model.degrees)
    );
    println!("├──────────────┼───┼────────────┤");
    println!("│ color seed   │ c │ {:<10} │", model.color_seed);
    println!("│ fourier seed │ f │ {:<10} │", model.series_seed);
    println!("├──────────────┼───┼────────────┤");
    println!(
        "│ scale        │ a │ {:<10} │",
        format!("{:.2}", model.scale)
    );
    println!("│ density      │ m │ {:<10} │", model.density);
    println!(
        "│ rotate       │ r │ {:<10} │",
        format!("{:.2}", model.rotate)
    );
    println!(
        "│ stroke weight│ w │ {:<10} │",
        format!("{:.2}", model.stroke_weight)
    );
    println!("├──────────────┼───┼────────────┤");
    println!("│ irrational   │ i │ {:<10} │", model.irrational);
    println!("│ maurer       │ h │ {:<10} │", model.maurer);
    println!("└──────────────┴───┴────────────┘");
}

fn control<Mode>(
    app: &mut App<Mode, Model>,
    control: Control,
    n: f32,
    d: f32,
    s: f32,
    r: f32,
    m: i32,
    c: i64,
    f: i64,
    w: f32,
) {
    match control {
        Control::N => app.model.n += n,
        Control::Degrees => app.model.degrees += d,
        Control::Scale => app.model.scale += s,
        Control::Rotate => app.model.rotate += r,
        Control::Density => {
            if app.model.density as i32 + m > 0 {
                app.model.density = (app.model.density as i32 + m) as u32;
            }
        }
        Control::StrokeWeight => app.model.stroke_weight += w,
        Control::Color => {
            app.model.color_seed = (app.model.color_seed as i64 + c) as u64;
            app.model.update_grad(app.model.color_seed);
        }
        Control::Fourier => {
            app.model.series_seed = (app.model.series_seed as i64 + f) as u64;
            app.model.random_series();
        }
    };
    message(&app.model);
}

fn main() -> Result<(), Error> {
    let model = Model::default();

    let config = Config::with_dims(1000, 1000);
    let mut app = App::app(model, config, |_, model| model, draw)
        .set_title("Maurer Rose")
        .no_loop();

    app.on_key_press(Key::Character("n".into()), |app| {
        app.model.control = Control::N;
        println!("Control Mode: n");
    });
    app.on_key_press(Key::Character("d".into()), |app| {
        app.model.control = Control::Degrees;
        println!("Control Mode: Degrees");
    });
    app.on_key_press(Key::Character("r".into()), |app| {
        app.model.control = Control::Rotate;
        println!("Control Mode: Rotate");
    });
    app.on_key_press(Key::Character("a".into()), |app| {
        app.model.control = Control::Scale;
        println!("Control Mode: Scale");
    });
    app.on_key_press(Key::Character("w".into()), |app| {
        app.model.control = Control::StrokeWeight;
        println!("Control Mode: Stroke Weight");
    });
    app.on_key_press(Key::Character("c".into()), |app| {
        app.model.control = Control::Color;
        println!("Control Mode: Color");
    });
    app.on_key_press(Key::Character("f".into()), |app| {
        app.model.control = Control::Fourier;
        println!("Control Mode: Fourier");
    });
    app.on_key_press(Key::Character("m".into()), |app| {
        app.model.control = Control::Density;
        println!("Control Mode: Density");
    });
    app.on_key_press(Key::Character("1".into()), |app| {
        app.model.style = Style::Line;
    });
    app.on_key_press(Key::Character("2".into()), |app| {
        app.model.style = Style::Bezier2;
    });
    app.on_key_press(Key::Character("3".into()), |app| {
        app.model.style = Style::Bezier3;
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowRight), |app| {
        control(
            app,
            app.model.control,
            0.5,  // n
            1.0,  // degrees
            0.1,  // scale
            -1.0, // rotate
            1,    // density
            1,    // color
            1,    // fourier
            0.1,  // stroke weight
        );
        message(&app.model);
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowLeft), |app| {
        control(
            app,
            app.model.control,
            -0.5, // n
            -1.0, // degrees
            -0.1, // scale
            1.0,  // rotate
            -1,   // density
            -1,   // color
            -1,   // fourier
            -0.1, // stroke weight
        );
        message(&app.model);
    });

    app.on_key_press(Key::Character("=".into()), |app| {
        control(
            app,
            app.model.control,
            0.25, // n
            0.1,  // degrees
            0.05, // scale
            -0.5, // rotate
            1,    // density
            1,    // color
            1,    // fourier
            0.05, // stroke weight
        );
        message(&app.model);
    });
    app.on_key_press(Key::Character("-".into()), |app| {
        control(
            app,
            app.model.control,
            -0.25, // n
            -0.1,  // degrees
            0.05,  // scale
            0.5,   // rotate
            -1,    // density
            -1,    // color
            -1,    // fourier
            -0.05, // stroke weight
        );
        message(&app.model);
    });

    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowUp), |app| {
        control(
            app,
            app.model.control,
            1.0,  // n
            10.0, // degrees
            0.5,  // scale
            -5.0, // rotate
            1,    // density
            10,   // color
            10,   // fourier
            0.25, // stroke weight
        );
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowDown), |app| {
        control(
            app,
            app.model.control,
            -1.0,  // n
            -10.0, // degrees
            -0.5,  // scale
            5.0,   // rotate
            -1,    // density
            -10,   // color
            -10,   // fourier
            -0.25, // stroke weight
        );
        message(&app.model);
    });

    app.on_key_press(Key::Character("h".into()), |app| {
        app.model.maurer = !app.model.maurer;
        message(&app.model);
    });
    app.on_key_press(Key::Character("i".into()), |app| {
        app.model.irrational = !app.model.irrational;
        message(&app.model);
    });
    message(&app.model);
    app.run()
}

#[derive(Copy, Clone, Debug)]
enum Style {
    Line,
    Bezier2,
    Bezier3,
}

#[derive(Copy, Clone, Debug)]
enum Control {
    N,
    Degrees,
    Scale,
    Rotate,
    Density,
    StrokeWeight,
    Color,
    Fourier,
}

#[derive(Clone)]
struct Model {
    // For integral `n`, the rose has `n` petals if n is odd,
    // and 2 * `n` petals if n is even.
    n: f32,
    // The change in the angle in degrees per line
    degrees: f32,
    // The Fourier series coefficients
    series: FourierSeries,
    // The number of lines to draw is `LINES` * density
    density: u32,
    // Line thickness
    stroke_weight: f32,
    // Rotation angle, counterclockwise
    rotate: f32,
    // Scale factor
    scale: f32,
    // Color gradient
    gradient: ColorScale,
    // Irrational ?
    irrational: bool,
    // Seed for the gradient
    color_seed: u64,
    // Seed for the Fourier series
    series_seed: u64,
    // Control
    control: Control,
    // Maurer Rose or Rhodonea
    maurer: bool,
    // Style
    style: Style,
}

impl Model {
    fn update_grad(&mut self, seed: u64) {
        let mut rng = SmallRng::seed_from_u64(seed);
        self.gradient = ColorScale::new(
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
        );
        self.color_seed = seed;
    }

    fn random_series(&mut self) {
        let mut rng = SmallRng::seed_from_u64(self.series_seed);
        self.series = FourierSeries::random(&mut rng, 5);
    }
}

impl Default for Model {
    fn default() -> Self {
        let mut rng = SmallRng::seed_from_u64(COLOR_SEED);
        let gradient = ColorScale::new(
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
            rand_okhsla(&mut rng),
        );
        Self {
            n: 2.0,
            degrees: 45.0,
            series: FourierSeries::sine(),
            density: 2,
            stroke_weight: 0.25,
            rotate: 0.0,
            scale: 1.0,
            gradient,
            irrational: true,
            color_seed: COLOR_SEED,
            series_seed: FOURIER_SEED,
            control: Control::Degrees,
            maurer: true,
            style: Style::Line,
        }
    }
}

/// A Fourier series is a sum of sine and cosine terms.
#[derive(Clone)]
struct FourierSeries {
    an: Vec<f32>,
    bn: Vec<f32>,
}

#[allow(dead_code)]
impl FourierSeries {
    fn new(an: &[f32], bn: &[f32]) -> Self {
        Self {
            an: an.to_vec(),
            bn: bn.to_vec(),
        }
    }

    fn with_complex(cn: &[Complex<f32>]) -> Self {
        let an: Vec<_> = cn.iter().map(|c| c.re).collect();
        let bn: Vec<_> = cn.iter().map(|c| c.im).collect();
        Self::new(&an, &bn)
    }

    fn c(an: &[f32]) -> Self {
        Self::new(an, &[])
    }

    fn s(bn: &[f32]) -> Self {
        Self::new(&[], bn)
    }

    fn sine() -> Self {
        Self::s(&[1.0])
    }

    fn cosine() -> Self {
        Self::c(&[1.0, 7.0])
    }

    fn square() -> Self {
        Self::s(&[1.0, 0.0, 1.0 / 3.0, 0.0, 1.0 / 5.0, 0.0, 1.0 / 7.0])
    }

    fn sawtooth() -> Self {
        Self::s(&[
            1.0,
            -1.0 / 2.0,
            1.0 / 3.0,
            -1.0 / 4.0,
            1.0 / 5.0,
            -1.0 / 6.0,
        ])
    }

    fn triangle() -> Self {
        Self::s(&[1.0, 0.0, 1.0 / 9.0, 0.0, 1.0 / 25.0, 0.0, 1.0 / 49.0])
    }

    fn random(rng: &mut SmallRng, n: usize) -> Self {
        let an: Vec<f32> = (0..n).map(|_| rng.gen_range(-1.0..=1.0)).collect();
        let bn: Vec<f32> = (0..n - 1).map(|_| rng.gen_range(-1.0..=1.0)).collect();
        Self::new(&an, &bn)
    }

    fn sum(self, other: Self) -> Self {
        let mut an0 = self.an.clone();
        let mut an1 = other.an.clone();
        if an0.len() < an1.len() {
            an0.resize(an1.len(), 0.0);
        } else {
            an1.resize(an0.len(), 0.0);
        };

        let mut bn0 = self.bn.clone();
        let mut bn1 = other.bn.clone();
        if bn0.len() < bn1.len() {
            bn0.resize(bn1.len(), 0.0);
        } else {
            bn1.resize(bn0.len(), 0.0);
        }

        Self::new(
            &an0.iter()
                .zip(an1.iter())
                .map(|(a, b)| a + b)
                .collect::<Vec<f32>>(),
            &bn0.iter()
                .zip(bn1.iter())
                .map(|(a, b)| a + b)
                .collect::<Vec<f32>>(),
        )
    }

    fn scale(&self, scale: f32) -> Self {
        Self::new(
            &self.an.iter().map(|a| a * scale).collect::<Vec<f32>>(),
            &self.bn.iter().map(|b| b * scale).collect::<Vec<f32>>(),
        )
    }

    fn eval(&self, scale: f32, t: f32) -> f32 {
        let mut m = 0.0;
        let mut radius = 0.0;
        for (i, a) in self.an.iter().enumerate() {
            m += a.abs();
            radius += a * ((i as f32) * t).cos();
        }
        for (i, b) in self.bn.iter().enumerate() {
            m += b.abs();
            radius += b * ((1.0 + i as f32) * t).sin();
        }
        scale / m * radius
    }
}

impl Add for FourierSeries {
    type Output = FourierSeries;

    fn add(self, other: Self) -> FourierSeries {
        self.sum(other)
    }
}

impl Mul<f32> for FourierSeries {
    type Output = FourierSeries;

    fn mul(self, rhs: f32) -> FourierSeries {
        self.scale(rhs)
    }
}

impl Mul<FourierSeries> for f32 {
    type Output = FourierSeries;

    fn mul(self, rhs: FourierSeries) -> FourierSeries {
        rhs.scale(self)
    }
}

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let mut vertices = vec![];
    let size = app.w_f32() / 2.2;
    let degrees = if model.maurer { model.degrees } else { 1.0 };

    for theta in 0..LINES * model.density {
        let k = theta as f32
            * std::f32::consts::PI
            * (degrees + if model.irrational { 0.01 } else { 0.0 })
            / 180.0;
        let r = size * model.series.eval(model.scale, model.n * k);
        vertices.push(pt(r * k.cos(), r * k.sin()));
    }

    let trans = Transform::from_rotate_at(model.rotate, 0.0, 0.0);
    trans.map_points(&mut vertices);
    match model.style {
        Style::Line => {
            for v in vertices.windows(2) {
                let t = v[1].mag() / (model.scale * size);
                let color = model.gradient.get_color(t);
                Shape::new()
                    .line(v[0], v[1])
                    .no_fill()
                    .stroke_color(color)
                    .stroke_weight(model.stroke_weight)
                    .cartesian(app.config.width, app.config.height)
                    .draw(&mut canvas);
            }
        }
        Style::Bezier2 => {
            let chunks = vertices.windows(3).step_by(2);
            for chunk in chunks {
                let t = chunk[2].mag() / (model.scale * size);
                let color = model.gradient.get_color(t);
                Shape::new()
                    .points(chunk)
                    .quad()
                    .no_fill()
                    .stroke_color(color)
                    .stroke_weight(model.stroke_weight)
                    .cartesian(app.config.width, app.config.height)
                    .draw(&mut canvas);
            }
        }
        Style::Bezier3 => {
            let chunks = vertices.windows(4).step_by(3);
            for chunk in chunks {
                let t = chunk[3].mag() / (model.scale * size);
                let color = model.gradient.get_color(t);
                Shape::new()
                    .points(chunk)
                    .cubic()
                    .no_fill()
                    .stroke_color(color)
                    .stroke_weight(model.stroke_weight)
                    .cartesian(app.config.width, app.config.height)
                    .draw(&mut canvas);
            }
        }
    }

    canvas.take()
}
