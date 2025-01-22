use artimate::app::{App, AppMode, Config, Error};
use wassily::prelude::*;

#[derive(Clone)]
struct Model {
    order: u32,
    margin: f32,
    path: Vec<Point>,
}

impl Model {
    fn new(order: u32, margin: f32, width: f32, height: f32) -> Self {
        let n = 2u32.pow(order);
        let mut path = vec![];
        let w = width - margin * 2.0;
        let h = height - margin * 2.0;
        let m = w / n as f32;
        let l = h / n as f32;

        for i in 0..n * n {
            let j = i as usize;
            path.push(hilbert(i, order));
            path[j] = pt(m * (path[j].x), l * (path[j].y));
            path[j] = pt(path[j].x + margin, path[j].y + margin);
        }

        Self {
            order,
            margin,
            path,
        }
    }
}

fn main() -> Result<(), Error> {
    const ORDER: u32 = 6;
    let n = 2u32.pow(ORDER);
    let config = Config::with_dims(1080, 1080).set_frames(5 * n * n - 1);
    let model = Model::new(ORDER, 100.0, config.w_f32(), config.h_f32());
    // .set_frames_to_save(n * n);
    let mut app = App::app(model, config, update, draw).set_title("Hilbert");
    app.run()
}

fn update(_app: &App<AppMode, Model>, model: Model) -> Model {
    model
}

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    Shape::new()
        .points(&model.path)
        .cubic()
        .no_fill()
        .stroke_color(*PINK)
        .stroke_weight(2.5)
        .draw(&mut canvas);

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
