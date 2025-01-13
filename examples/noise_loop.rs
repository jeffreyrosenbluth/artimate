use artimate::core::{App, Config, Error};
use wassily::prelude::*;

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
            num_frames: 50,
            margin: 70.0,
            noise: Value::default(),
        }
    }
}

fn update(_app: &App<Model>, model: Model) -> Model {
    model
}

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

fn offset(app: &App<Model>, model: &Model, x: f32, y: f32) -> f32 {
    let (w, h) = app.config.wh_f32();
    let dist2 = (x - w / 2.0) * (x - w / 2.0) + (y - h / 2.0) * (y - h / 2.0);
    model.factor * dist2.sqrt()
}

fn draw(app: &App<Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);
    let mut color = *WHITE;
    color.set_alpha(0.6);
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
            canvas.dot(x + dx, y + dy, color);
        }
    }
    canvas.take()
}

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = Config::new(700, 700);
    let mut app = App::new(model, config, update, draw).set_frames_to_save(38);
    app.run()
}
