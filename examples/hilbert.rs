use artimate::core::{App, Config, Error};
use wassily::prelude::*;

#[derive(Clone)]
struct Model {
    order: u32,
}

impl Default for Model {
    fn default() -> Self {
        Self { order: 6 }
    }
}

fn main() -> Result<(), Error> {
    let n = 2u32.pow(6);
    let config = Config::from_dims(1080, 1080).set_frames(n * n);
    let mut app = App::new(Model::default(), config, update, draw).set_title("Hilbert");
    app.run()
}

fn update(_app: &App<Model>, model: Model) -> Model {
    model
}

fn draw(app: &App<Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let n = 2u32.pow(model.order);
    let n2 = n * n;
    let mut path = vec![];

    for i in 0..n2 {
        let j = i as usize;
        path.push(hilbert(i, model.order));
        let (w, h) = app.config.wh_f32();
        let m = w / n as f32;
        let l = h / n as f32;
        path[j] = pt(m * path[j].x, l * path[j].y);
        path[j] = pt(path[j].x + m / 2.0, path[j].y + l / 2.0);
    }

    let path = path[0..app.frame_count as usize].to_vec();

    Shape::new()
        .points(&path)
        .no_fill()
        .stroke_color(*WHITE)
        .stroke_weight(2.0)
        .draw(&mut canvas);

    canvas.take()
}

fn hilbert(k: u32, order: u32) -> Point {
    let points = vec![pt(0.0, 0.0), pt(0.0, 1.0), pt(1.0, 1.0), pt(1.0, 0.0)];
    let idx = k as usize & 3;
    let mut v = points[idx];
    let mut i = k;

    for j in 1..order {
        i >>= 2;
        let index = i & 3;
        let n = 2u32.pow(j) as f32;
        match index {
            0 => {
                let temp = v.x;
                v.x = v.y;
                v.y = temp;
            }
            1 => {
                v.y += n;
            }
            2 => {
                v.x += n;
                v.y += n;
            }
            3 => {
                let temp = n - 1.0 - v.x;
                v.x = n - 1.0 - v.y;
                v.y = temp;
                v.x += n;
            }
            _ => {}
        }
    }
    v
}
