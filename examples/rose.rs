use artimate::app::{App, AppMode, Config, Error};
use wassily::prelude::*;

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = if model.animate {
        Config::with_dims(700, 700)
            .set_frames(model.lines)
            .set_frames_to_save(model.lines)
    } else {
        Config::with_dims(700, 700).no_loop()
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
    // The number of lines to draw
    lines: u32,
    stroke_weight: f32,
    animate: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            petals: 3.0,
            degrees: 37.0,
            lines: 6400,
            stroke_weight: 0.25,
            animate: false,
        }
    }
}

fn f(t: f32) -> f32 {
    1.0 * (0.5 * t.cos() + 0.3 * (2.0 * t).cos() + 0.2 * (3.0 * t).cos())
}

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let mut vertices = vec![];
    let size = app.config.w_f32() / 2.2;

    let n = if model.animate {
        app.frame_count
    } else {
        model.lines - 1
    };
    for theta in 0..=n {
        // the + 0.01 is to prevent periodicity
        let k = theta as f32 * std::f32::consts::PI * (model.degrees + 0.01) / 180.0;
        // let r = size * (model.petals * k).sin();
        let r = size * f(model.petals * k);
        vertices.push(pt(r * k.cos(), r * k.sin()));
    }

    // Draw the rose
    let trans = Transform::from_rotate_at(
        27.0 * std::f32::consts::PI * app.frame_count as f32 / 1500.0, // angle in radians
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

    if model.animate && vertices.len() > 2 && app.frame_count < model.lines {
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
    if app.frame_count == model.lines {
        let center = pt(app.config.w_f32() / 2.0, app.config.h_f32() / 2.0);
        Shape::new()
            .circle(center, 2.0)
            .fill_color(*GOLD)
            .no_stroke()
            .draw(&mut canvas);
    }

    canvas.take()
}
