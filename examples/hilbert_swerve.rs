use artimate::core::{App, Config, Error};
use wassily::prelude::*;

#[derive(Clone)]
struct Model {
    order: u32,
    noise: Perlin,
    scale: f64,
    factor: f32,
    margin: f32,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            order: 6,
            noise: Perlin::default(),
            scale: 0.03,
            factor: 10.0,
            margin: 125.0,
        }
    }
}

fn main() -> Result<(), Error> {
    let model = Model::default();
    let n = 2u32.pow(model.order);
    let config = Config::from_dims(1080, 1080)
        .set_frames(n * n)
        .set_frames_to_save(n * n);
    let mut app = App::new(model, config, update, draw).set_title("Hilbert");
    app.run()
}

fn update(_app: &App<Model>, model: Model) -> Model {
    model
}

fn draw(app: &App<Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let n = 2u32.pow(model.order);
    let mut path = vec![];

    for i in 0..app.frame_count {
        let j = i as usize;
        path.push(hilbert(i, model.order));
        let (w, h) = app.config.wh_f32();
        let w = w - model.margin * 2.0;
        let h = h - model.margin * 2.0;
        let m = w / n as f32;
        let l = h / n as f32;
        let s = model.scale;
        let nx = app.frame_count as f32 / app.config.frames.unwrap() as f32
            * model.factor
            * model
                .noise
                .get([s * path[j].x as f64, s * path[j].y as f64]) as f32;
        let ny = app.frame_count as f32 / app.config.frames.unwrap() as f32
            * model.factor
            * model
                .noise
                .get([123.0 + s * path[j].x as f64, 123.0 + s * path[j].y as f64])
                as f32;
        path[j] = pt(m * (path[j].x + nx), l * (path[j].y + ny));
        path[j] = pt(path[j].x + model.margin, path[j].y + model.margin);
    }

    let t = smoother_step(app.frame_count as f32 / app.config.frames.unwrap() as f32);

    let color = if t < 0.5 {
        (*PINK).lerp(&DEEPPINK, 2.0 * t)
    } else {
        (*DEEPPINK).lerp(&PINK, 2.0 * (t - 0.5))
    };

    Shape::new()
        .points(&path)
        .no_fill()
        .stroke_color(color)
        .stroke_weight(2.0)
        .draw(&mut canvas);

    canvas.take()
}

fn hilbert(k: u32, order: u32) -> Point {
    // Base points for the Hilbert curve
    let mut v = match k & 3 {
        0 => pt(0.0, 0.0),
        1 => pt(0.0, 1.0),
        2 => pt(1.0, 1.0),
        _ => pt(1.0, 0.0),
    };

    let mut i = k >> 2; // Start from the next significant bits

    for j in 1..order {
        let n = 2u32.pow(j) as f32;
        match i & 3 {
            0 => (v.x, v.y) = (v.y, v.x), // Rotate 90° clockwise
            1 => v.y += n,                // Move up
            2 => {
                // Move diagonally
                v.x += n;
                v.y += n;
            }
            _ => {
                // Rotate 90° counter-clockwise and move right
                let temp = n - 1.0 - v.x;
                v.x = n - 1.0 - v.y + n;
                v.y = temp;
            }
        }
        i >>= 2;
    }
    v
}
