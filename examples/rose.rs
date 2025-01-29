use artimate::app::{App, AppMode, Config, Error};
use num_complex::Complex;
use wassily::prelude::*;
use winit::keyboard::Key;

const LINES: u32 = 3600;

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = Config::with_dims(700, 700).set_frames(LINES * model.density);
    let mut app = App::app(model, config, |_, model| model, draw).set_title("Maurer Rose");

    app.on_key_press(Key::Character("=".into()), |app| {
        app.model.n += 0.5;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Character("+".into()), |app| {
        app.model.n += 0.5;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Character("-".into()), |app| {
        app.model.n -= 0.5;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowRight), |app| {
        app.model.degrees += 1.0;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Named(winit::keyboard::NamedKey::ArrowLeft), |app| {
        app.model.degrees -= 1.0;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Character(">".into()), |app| {
        app.model.degrees += 10.0;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Character("<".into()), |app| {
        app.model.degrees -= 10.0;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Character("r".into()), |app| {
        app.model.rotate -= 1.0;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.on_key_press(Key::Character("l".into()), |app| {
        app.model.rotate += 1.0;
        println!("n = {}, degrees = {}", app.model.n, app.model.degrees);
    });
    app.run()
}

#[derive(Clone)]
pub struct Model {
    // For integral `n`, the rose has `n` petals if n is odd, and 2 * `n` petals if n is even.
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
            series: FourierSeries::s(&[1.0]),
            density: 2,
            stroke_weight: 0.25,
            rotate: 0.0,
            scale: 1.0,
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

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let mut vertices = vec![];
    let size = app.config.w_f32() / 2.2;

    for theta in 0..=app.config.frames.unwrap() {
        // the + 0.01 is to prevent periodicity
        let k = theta as f32 * std::f32::consts::PI * (model.degrees + 0.01) / 180.0;
        let r = size * model.series.eval(model.scale, model.n * k);
        vertices.push(pt(r * k.cos(), r * k.sin()));
    }

    let trans = Transform::from_rotate_at(model.rotate, 0.0, 0.0);
    trans.map_points(&mut vertices);

    // Draw the rose
    Shape::new()
        .no_fill()
        .stroke_color(Color::from_rgba8(255, 255, 255, 100))
        .stroke_weight(model.stroke_weight)
        .points(&vertices)
        .cartesian(app.config.width, app.config.height)
        .draw(&mut canvas);

    canvas.take()
}
