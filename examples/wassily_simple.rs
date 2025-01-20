use artimate::app::{App, Config, Error};
use wassily::prelude::*;

fn main() -> Result<(), Error> {
    // Default size is 1080 x 700.
    let config = Config::default();
    let mut app = App::sketch(config, draw).set_title("Ball");
    app.run()
}

fn draw(app: &App, _model: &()) -> Vec<u8> {
    let pos = pt(
        100.0 * app.time % app.config.width as f32,
        app.config.height as f32 / 2.0,
    );
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);
    Shape::new()
        .circle(pos, 75.0)
        .fill_color(*CORNFLOWERBLUE)
        .stroke_color(*ORANGERED)
        .stroke_weight(3.0)
        .draw(&mut canvas);
    canvas.take()
}
