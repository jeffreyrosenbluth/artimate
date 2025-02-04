use artimate::app::{App, AppMode, Config, Error};
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
            order: 7,
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
    let config = Config::with_dims(1080, 1080);
    let mut app = App::app(model, config, |_, model| model, draw)
        .set_title("Hilbert")
        .set_frames(n * n * 3 / 2)
        .set_frames_to_save(n * n * 3 / 2);
    app.run()
}

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let n = 2u32.pow(model.order);
    let n2 = n * n;
    let mut path = vec![];

    if app.frame_count < n2 {
        for i in 0..app.frame_count.min(n2 - 1) {
            let j = i as usize;
            path.push(hilbert(i, model.order));
            let (w, h) = app.wh_f32();
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

        let p1 = &path[0..app.frame_count as usize / 2].to_vec();
        Shape::new()
            .points(&p1)
            .cubic()
            .no_fill()
            .stroke_color(*WHITE)
            .stroke_weight(2.5)
            .draw(&mut canvas);

        let p2 = &path[app.frame_count as usize / 2..app.frame_count as usize].to_vec();
        Shape::new()
            .points(&p2)
            .no_fill()
            .stroke_color(*WHITE)
            .stroke_weight(2.5)
            .draw(&mut canvas);
    } else {
        for i in 0..n2 {
            let j = i as usize;
            path.push(hilbert(i, model.order));
            let (w, h) = app.wh_f32();
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

        let p3 = &path[0..(app.frame_count - n2 / 2) as usize].to_vec();
        Shape::new()
            .points(&p3)
            .cubic()
            .no_fill()
            .stroke_color(*WHITE)
            .stroke_weight(2.5)
            .draw(&mut canvas);

        let p4 = &path[(app.frame_count - n2 / 2) as usize..n2 as usize].to_vec();
        Shape::new()
            .points(&p4)
            .no_fill()
            .stroke_color(*WHITE)
            .stroke_weight(2.5)
            .draw(&mut canvas);
    }

    canvas.take()
}

fn hilbert(index: u32, order: u32) -> Point {
    // Base points for the Hilbert curve
    let mut point = match index & 3 {
        0 => pt(0.0, 0.0),
        1 => pt(0.0, 1.0),
        2 => pt(1.0, 1.0),
        _ => pt(1.0, 0.0),
    };

    let mut i = index >> 2; // Start from the next significant bits

    for j in 1..order {
        let n = 2u32.pow(j) as f32;
        match i & 3 {
            0 => (point.x, point.y) = (point.y, point.x), // Rotate 90° clockwise
            1 => point.y += n,                            // Move up
            2 => {
                // Move diagonally
                point.x += n;
                point.y += n;
            }
            _ => {
                // Rotate 90° counter-clockwise and move right
                let temp = n - 1.0 - point.x;
                point.x = n - 1.0 - point.y + n;
                point.y = temp;
            }
        }
        i >>= 2;
    }
    point
}
