use artimate::core::{App, Config};
use pixels::Error;
use wassily::prelude::*;

#[derive(Clone)]
struct Model {
    radius: f32,
    size: f32,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            radius: 125.0,
            size: 125.0,
        }
    }
}

fn update(app: &App<Model>, model: Model) -> Model {
    Model {
        size: 1.5 * model.radius * (app.time * 0.25).cos().abs().max(0.2),
        ..model
    }
}

fn draw(app: &App<Model>, model: &Model) -> Vec<u8> {
    let (width, height) = app.config.wh();
    let (w32, h32) = app.config.wh_f32();
    let center = pt(w32 / 2.0, h32 / 2.0);
    let t = app.time * 0.25;
    let mut canvas = Canvas::new(width, height);
    canvas.fill(*BLACK);
    let pos = pt(
        center.x + 0.3 * w32 * (2.0 * t).cos(),
        center.y + 0.3 * h32 * (2.0 * t).sin(),
    );

    let start = pt(
        pos.x - model.size * 0.4 * (4.0 * t).cos(),
        pos.y - model.size * 0.4 * (4.0 * t).sin(),
    );
    let end = pt(
        pos.x - model.size * 0.4 * (4.0 * t).cos(),
        pos.y - model.size * 0.4 * (4.0 * t).sin(),
    );

    let stops = vec![
        GradientStop::new(0.0, *WHITE),
        GradientStop::new(0.35, grays(70)),
        GradientStop::new(0.5, *INDIANRED),
        GradientStop::new(0.75, *DARKSLATEGRAY),
        GradientStop::new(1.0, grays(15)),
    ];
    let rg = paint_rg(
        start.x,
        start.y,
        end.x,
        end.y,
        (0.5 + t.cos().abs()) * model.radius,
        stops,
    );

    Shape::new()
        .circle(pos, model.size)
        .fill_paint(&rg)
        .no_stroke()
        .draw(&mut canvas);
    canvas.take()
}

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = Config::new(1024, 1024);
    let mut app = App::new(model, config, update, draw).set_title("Sphere");
    app.run()
}
