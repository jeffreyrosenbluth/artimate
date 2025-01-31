use artimate::app::{App, AppMode, Config, Error};
use num_complex::Complex;
use std::ops::{Add, Mul};
use wassily::prelude::*;
use winit::keyboard::Key;

const LINES: u32 = 3600;

fn message(model: &Model) {
    println!(
        "n = {}, degrees = {}, scale = {}, rotate = {}",
        model.n, model.degrees, model.scale, model.rotate
    );
}

fn main() -> Result<(), Error> {
    let model = Model::default();

    let config = Config::with_dims(700, 700).no_loop();
    let mut app = App::app(model, config, |_, model| model, draw).set_title("Maurer Rose");

    message(&app.model);

    app.on_key_press(Key::Character("=".into()), |app| {
        app.model.n += 0.5;
        message(&app.model);
    });
    app.on_key_press(Key::Character("+".into()), |app| {
        app.model.n += 0.25;
        message(&app.model);
    });
    app.on_key_press(Key::Character("-".into()), |app| {
        app.model.n -= 0.5;
        message(&app.model);
    });
    app.on_key_press(Key::Character("_".into()), |app| {
        app.model.n -= 0.25;
        message(&app.model);
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowRight), |app| {
        app.model.degrees += 1.0;
        message(&app.model);
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowLeft), |app| {
        app.model.degrees -= 1.0;
        message(&app.model);
    });
    app.on_key_press(Key::Character(">".into()), |app| {
        app.model.degrees += 10.0;
        message(&app.model);
    });
    app.on_key_press(Key::Character("<".into()), |app| {
        app.model.degrees -= 10.0;
        message(&app.model);
    });
    app.on_key_press(Key::Character("r".into()), |app| {
        app.model.rotate -= 1.0;
        message(&app.model);
    });
    app.on_key_press(Key::Character("R".into()), |app| {
        app.model.rotate += 1.0;
        message(&app.model);
    });
    app.on_key_press(Key::Character("1".into()), |app| {
        app.model.degrees = 1.0;
        message(&app.model);
    });
    app.on_key_press(Key::Character("s".into()), |app| {
        app.model.scale -= 0.1;
        message(&app.model);
    });
    app.on_key_press(Key::Character("S".into()), |app| {
        app.model.scale += 0.1;
        message(&app.model);
    });
    app.run()
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
}

impl Default for Model {
    fn default() -> Self {
        Self {
            n: 2.0,
            degrees: 74.0,
            series: FourierSeries::sawtooth() * 0.75 + 1.25 * FourierSeries::s(&[1.0]),
            density: 2,
            stroke_weight: 0.25,
            rotate: 0.0,
            scale: 1.5,
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
    let size = app.config.w_f32() / 2.2;

    for theta in 0..LINES * model.density {
        // The + 0.01 is to prevent periodicity
        let k = theta as f32 * std::f32::consts::PI * (model.degrees + 0.01) / 180.0;
        let r = size * model.series.eval(model.scale, model.n * k);
        vertices.push(pt(r * k.cos(), r * k.sin()));
    }

    let trans = Transform::from_rotate_at(model.rotate, 0.0, 0.0);
    trans.map_points(&mut vertices);

    Shape::new()
        .no_fill()
        .stroke_color(Color::from_rgba8(255, 255, 255, 100))
        .stroke_weight(model.stroke_weight)
        .points(&vertices)
        .cartesian(app.config.width, app.config.height)
        .draw(&mut canvas);

    canvas.take()
}
