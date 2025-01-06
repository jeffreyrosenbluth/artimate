use artimate::core::{App, Config, Error};
use wassily::prelude::*;

#[derive(Clone)]
struct Model {
    radius: f32,
    offset: f32,
    grad_offset: f32,
    stops_1: Vec<GradientStop>,
    stops_2: Vec<GradientStop>,
    size_factor: f32,
    size_max: f32,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            radius: 125.0,
            offset: 0.3,
            grad_offset: 0.3,
            stops_1: vec![
                GradientStop::new(0.0, *WHITE),
                GradientStop::new(0.35, grays(70)),
                GradientStop::new(0.5, *INDIANRED),
                GradientStop::new(0.75, *DARKSLATEGRAY),
                GradientStop::new(1.0, grays(25)),
            ],
            stops_2: vec![
                GradientStop::new(0.0, *WHITE),
                GradientStop::new(0.30, grays(70)),
                GradientStop::new(0.5, *TEAL),
                GradientStop::new(0.70, *PALEVIOLETRED),
                GradientStop::new(1.0, grays(25)),
            ],
            size_factor: 1.4,
            size_max: 0.6,
        }
    }
}

fn update(_app: &App<Model>, model: Model) -> Model {
    model
}

fn draw_planet(
    app: &App<Model>,
    model: &Model,
    pos: Point,
    stops: Vec<GradientStop>,
    canvas: &mut Canvas,
) {
    let half_time = 0.5 * app.time;
    let quarter_time = 0.25 * app.time;
    let size = model.size_factor * model.radius * quarter_time.cos().abs().max(model.size_max);

    let start = pt(
        pos.x - size * model.grad_offset * half_time.cos(),
        pos.y - size * model.grad_offset * half_time.sin(),
    );
    let end = pt(
        pos.x - size * model.grad_offset * half_time.cos(),
        pos.y - size * model.grad_offset * half_time.sin(),
    );

    let rg = paint_rg(
        start.x,
        start.y,
        end.x,
        end.y,
        (0.5 + quarter_time.cos().abs()) * model.radius,
        stops,
    );

    Shape::new()
        .circle(pos, size)
        .fill_paint(&rg)
        .no_stroke()
        .draw(canvas);
}

fn draw(app: &App<Model>, model: &Model) -> Vec<u8> {
    let (width, height) = app.config.wh();
    let (w_f32, h_f32) = app.config.wh_f32();
    let center = pt(w_f32 / 2.0, h_f32 / 2.0);
    let half_time = 0.5 * app.time;

    let mut canvas = Canvas::new(width, height);
    canvas.fill(*BLACK);

    let mut rng = SmallRng::seed_from_u64(0);
    let mut star_color = *WHITE;
    for _ in 0..100 {
        let x = rng.gen_range(0.0..w_f32);
        let y = rng.gen_range(0.0..h_f32);
        let r = rng.gen_range(0.5..2.0);
        star_color.set_alpha(0.4 + (0.5 + 0.5 * app.time).sin() * rng.gen_range(0.0..0.6));
        Shape::new()
            .star(pt(x, y), r, 3.0 * r, 5)
            .fill_color(star_color)
            .no_stroke()
            .draw(&mut canvas);
    }

    let pos_1 = pt(
        center.x + model.offset * w_f32 * half_time.cos(),
        center.y + model.offset * h_f32 * half_time.sin(),
    );
    draw_planet(app, &model, pos_1, model.stops_1.clone(), &mut canvas);

    let pos_2 = pt(w_f32 - pos_1.x, h_f32 - pos_1.y);
    draw_planet(app, &model, pos_2, model.stops_2.clone(), &mut canvas);

    canvas.take()
}

fn main() -> Result<(), Error> {
    let model = Model::default();
    let config = Config::new(1024, 1024);
    let mut app = App::new(model, config, update, draw).set_title("Sphere");
    app.run()
}
