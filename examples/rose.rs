use artimate::app::{App, AppMode, Config, Error};
use num_complex::Complex;
use wassily::prelude::*;

const LINES: u32 = 3600;

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = if model.animate {
        Config::with_dims(700, 700)
            .set_frames(LINES * model.density)
            .set_frames_to_save(LINES * model.density)
    } else {
        Config::with_dims(700, 700).no_loop().set_frames_to_save(1)
    };
    let mut app = App::app(model, config, |_, model| model, draw).set_title("Maurer Rose");
    app.run()
}

#[derive(Clone)]
pub struct Model {
    // The rose has `petals` petals if n is odd, and 2 * `petals` petals if n is even.
    petals: f32,
    // The angle in degrees
    degrees: f32,
    // The number of lines to draw is `LINES * density
    density: u32,
    // Line thickness
    stroke_weight: f32,
    // Whether to animate the rose
    animate: bool,
    // Rotation angle, counterclockwise
    rotate: f32,
    // Scale factor
    scale: f32,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            petals: 3.25,
            degrees: 37.0,
            density: 2,
            stroke_weight: 0.25,
            animate: false,
            rotate: 0.0,
            scale: 1.0,
        }
    }
}

struct F {
    an: Vec<f32>,
    bn: Vec<f32>,
}

#[allow(dead_code)]
impl F {
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

    let n = if model.animate {
        app.frame_count
    } else {
        LINES * model.density - 1
    };

    // let fourier = F::new(
    //     &[],
    //     &[
    //         1.0,
    //         0.0,
    //         1.0 / 3.0,
    //         0.0,
    //         1.0 / 5.0,
    //         // 0.0,
    //         // 1.0 / 7.0,
    //         // 0.0,
    //         // 1.0 / 9.0,
    //     ],
    // );

    let fourier = F::new(&[0.5, 1.0, 0.0, 1.0 / 9.0, 0.0, 1.0 / 25.0], &[]);
    // let fourier = F::new(&[], &[1.0]);

    for theta in 0..=n {
        // the + 0.01 is to prevent periodicity
        let k = theta as f32 * std::f32::consts::PI * (model.degrees + 0.01) / 180.0;
        let r = size * fourier.eval(model.scale, model.petals * k);
        vertices.push(pt(r * k.cos(), r * k.sin()));
    }

    // Draw the rose
    let trans = Transform::from_rotate_at(
        model.rotate + 27.0 * std::f32::consts::PI * app.frame_count as f32 / 1500.0, // angle in radians
        0.0,
        0.0,
    );

    trans.map_points(&mut vertices);

    Shape::new()
        .no_fill()
        .stroke_color(Color::from_rgba8(255, 255, 255, 100))
        .stroke_weight(model.stroke_weight)
        .points(&vertices)
        .cartesian(app.config.width, app.config.height)
        .draw(&mut canvas);

    if model.animate && vertices.len() > 2 && app.frame_count < LINES * model.density {
        Shape::new()
            .line(
                vertices[app.frame_count as usize - 2],
                vertices[app.frame_count as usize - 1],
            )
            .cartesian(app.config.width, app.config.height)
            .stroke_color(*GOLD)
            .stroke_weight(1.5)
            .draw(&mut canvas);
    }

    // Draw a dot in the center when finished
    if app.frame_count == model.density {
        let center = pt(app.config.w_f32() / 2.0, app.config.h_f32() / 2.0);
        Shape::new()
            .circle(center, 2.0)
            .fill_color(*GOLD)
            .no_stroke()
            .draw(&mut canvas);
    }

    canvas.take()
}
