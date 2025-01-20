use artimate::core::{App, Config, Error};
use wassily::prelude::*;

fn main() -> Result<(), Error> {
    let config = Config::with_dims(540, 540);
    let model = Model::default();
    let mut app = App::new(model, config, update, draw).set_title("Flag");
    app.run()
}

#[derive(Clone)]
struct Model {
    radius: f32,
    points: u32,
    noise: RidgedMulti<Perlin>,
    scale: f32,
    factor: f32,
    margin: f32,
    speed: f32,
    color1: Color,
    color2: Color,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            radius: 1.0,
            points: 75,
            noise: RidgedMulti::default().set_octaves(1),
            scale: 0.01,
            factor: 50.0,
            margin: 60.0,
            speed: 0.001,
            color1: *WHITE,
            color2: *TURQUOISE,
        }
    }
}

fn update(_app: &App<Model>, model: Model) -> Model {
    model
}

fn periodic_noise(model: &Model, p: f32, seed: f32, x: f32, y: f32) -> f32 {
    let u = seed + (std::f32::consts::PI * 2.0 * p).cos();
    let v = (std::f32::consts::PI * 2.0 * p).sin();
    model.factor
        * model.noise.get([
            u as f64,
            v as f64,
            (model.scale * x) as f64,
            (model.scale * y) as f64,
        ]) as f32
}

fn draw(app: &App<Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);
    let (w, _) = app.config.wh_f32();
    let space = (w - model.margin * 2.0) / model.points as f32;
    let t = model.speed * app.frame_count as f32;
    for i in 0..model.points {
        for j in 0..model.points {
            let x = model.margin + i as f32 * space;
            let y = model.margin + j as f32 * space;
            let dx = periodic_noise(model, t, 0.0, x, y);
            let dy = periodic_noise(model, t, 123.0, x, y);
            Shape::new()
                .circle(pt(x + dx, y + dy), model.radius)
                .no_stroke()
                .fill_color(model.color1)
                .draw(&mut canvas);
            Shape::new()
                .circle(pt(x - 0.75 * dx, y - 0.75 * dy), model.radius)
                .no_stroke()
                .fill_color(model.color2)
                .draw(&mut canvas);
        }
    }
    canvas.take()
}
