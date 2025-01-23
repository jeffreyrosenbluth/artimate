use artimate::app::{App, AppMode, Config, Error};
use wassily::prelude::*;
use winit::keyboard::Key;

// The model holds properties that are used to draw the scene.
// These properties can be changed in the update function
#[derive(Clone)]
struct Model {
    // The radius of the planets
    radius: f32,
    // The offset of the planets from the center
    offset: f32,
    // The offset of the gradient from the center
    grad_offset: f32,
    // The gradient stops for the first planet
    stops_1: Vec<GradientStop>,
    // The gradient stops for the second planet
    stops_2: Vec<GradientStop>,
    // The size factor of the planets
    size_factor: f32,
    // The minimum size of the planets
    size_min: f32,
    // The number of stars
    num_stars: usize,
    // Toggle the mouse controls
    mouse_controls: bool,
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
                GradientStop::new(0.35, grays(70)),
                GradientStop::new(0.5, *DARKSLATEGRAY),
                GradientStop::new(0.75, *INDIANRED),
                GradientStop::new(1.0, grays(25)),
            ],
            size_factor: 1.25,
            size_min: 0.6,
            num_stars: 100,
            mouse_controls: true,
        }
    }
}

fn main() -> Result<(), Error> {
    let model = Model::default();
    // Default size is 1080 x 700.
    let config = Config::default().set_frames_to_save(1508);
    let mut app = App::app(model, config, update, draw).set_title("Sphere");
    let key = Key::Character("t".into());
    app.on_key_press(key, move |app| {
        app.model.mouse_controls = !app.model.mouse_controls;
    });
    app.run()
}

// The update function is called on every frame.
fn update(app: &App<AppMode, Model>, model: Model) -> Model {
    if !model.mouse_controls {
        return model;
    };
    let v = map_range(app.mouse_y(), 0.0, app.config.height as f32, 0.35, 0.75);
    let u = map_range(app.mouse_y(), 0.0, app.config.height as f32, 0.35, 0.75);
    let mut stops1 = model.stops_1;
    let mut stops2 = model.stops_2;
    stops1[2] = GradientStop::new(v, *INDIANRED);
    stops2[2] = GradientStop::new(u, *DARKSLATEGRAY);
    let num_stars = if app.mouse_x() < 1.0 {
        100
    } else {
        app.mouse_x() as usize
    };
    Model {
        stops_1: stops1,
        stops_2: stops2,
        num_stars,
        ..model
    }
}

// Draw each planet
fn draw_planet(
    app: &App<AppMode, Model>,
    model: &Model,
    // The position of the planet
    pos: Point,
    stops: Vec<GradientStop>,
    canvas: &mut Canvas,
) {
    let half_time = 0.5 * app.time;
    let quarter_time = 0.25 * app.time;

    // The size of the planet
    let size = model.size_factor * model.radius * quarter_time.cos().abs().max(model.size_min);

    // The gradient start and end points
    let start = pt(
        pos.x - size * model.grad_offset * half_time.cos(),
        pos.y - size * model.grad_offset * half_time.sin(),
    );
    let end = pt(
        pos.x - size * model.grad_offset * half_time.cos(),
        pos.y - size * model.grad_offset * half_time.sin(),
    );

    // The gradient
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

fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    // It's convenient to have both the width and height as u32 and  f32
    let (width, height) = app.config.wh();
    let (w_f32, h_f32) = app.config.wh_f32();

    let center = pt(w_f32 / 2.0, h_f32 / 2.0);
    let half_time = 0.5 * app.time;

    let mut canvas = Canvas::new(width, height);
    canvas.fill(*BLACK);

    // Draw the background stars at random locations.
    let mut rng = SmallRng::seed_from_u64(0);
    let mut star_color = *WHITE;
    for _ in 0..model.num_stars {
        let x = rng.gen_range(0.0..w_f32);
        let y = rng.gen_range(0.0..h_f32);
        let r = rng.gen_range(0.5..2.0);
        star_color.set_alpha(0.4 + (0.5 + half_time).sin() * rng.gen_range(0.0..0.6));
        Shape::new()
            .star(pt(x, y), r, 3.0 * r, 5)
            .fill_color(star_color)
            .no_stroke()
            .draw(&mut canvas);
    }

    // Position of the first planet.
    let pos_1 = pt(
        center.x + model.offset * w_f32 * half_time.cos(),
        center.y + model.offset * h_f32 * half_time.sin(),
    );
    draw_planet(app, &model, pos_1, model.stops_1.clone(), &mut canvas);

    // Position of the second planet, opposite position of the first planet.
    let pos_2 = pt(w_f32 - pos_1.x, h_f32 - pos_1.y);
    draw_planet(app, &model, pos_2, model.stops_2.clone(), &mut canvas);

    // return the canvas data as a Vec<u8>
    canvas.take()
}
