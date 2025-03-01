use artimate::app::{App, AppMode, Config, Error};
use wassily::prelude::*;

#[derive(Copy, Clone, Debug)]
struct Model;

impl Default for Model {
    fn default() -> Self {
        Self {}
    }
}

fn main() -> Result<(), Error> {
    let config = Config::with_dims(750, 750);
    let model = Model::default();
    let mut app = App::app(model, config, |_, model| model, draw).set_title("Blues");
    app.run()
}

fn draw(app: &App<AppMode, Model>, _model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);
    let delta_c = 45.0 / 75.0;
    let delta_l = 1.0 / 75.0;
    let s = map_range(app.mouse_x(), 0.0, app.config.width as f32, 0.0, 1.0);
    for i in 0..75 {
        let h = 200.0 + i as f32 * delta_c;
        for j in 0..75 {
            let l = j as f32 * delta_l;
            let x = 10 * i;
            let y = 10 * j;
            let c = Okhsl::new(h, s, l).to_color();
            Shape::new()
                .rect_xywh(pt(x, y), pt(10.0, 10.0))
                .fill_color(c)
                .stroke_color(*WHITE)
                .stroke_weight(0.25)
                .draw(&mut canvas);
        }
    }

    canvas.take()
}
