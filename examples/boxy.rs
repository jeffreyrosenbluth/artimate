use artimate::app::{App, AppMode, Config, Error};
use wassily::prelude::*;

struct Rectangle<N>
where
    N: NoiseFn<f64, 2>,
{
    location: Point,
    noise: N,
    noise_opts: NoiseOpts,
    size: f32,
    color: Box<dyn Fn(&Rectangle<N>) -> Color>,
    length: Box<dyn Fn(&Rectangle<N>) -> f32>,
    width: Box<dyn Fn(&Rectangle<N>) -> f32>,
}

impl<N> Rectangle<N>
where
    N: NoiseFn<f64, 2>,
{
    fn new(
        location: Point,
        noise: N,
        noise_opts: NoiseOpts,
        size: f32,
        color: Box<dyn Fn(&Rectangle<N>) -> Color>,
        length: Box<dyn Fn(&Rectangle<N>) -> f32>,
        width: Box<dyn Fn(&Rectangle<N>) -> f32>,
    ) -> Self {
        Self {
            location,
            noise,
            noise_opts,
            size,
            color,
            length,
            width,
        }
    }
    fn draw(&self, canvas: &mut Canvas) {
        let length = (self.length)(self);
        let width = (self.width)(self);
        let color = (self.color)(self);
        Shape::new()
            // .rect_cwh(self.location, pt(length.max(5.0), width.max(5.0)))
            .ellipse(self.location, length.max(0.0), width.max(0.0))
            .fill_color(color)
            .stroke_color(*DEEPPINK)
            .stroke_weight(3.0)
            .draw(canvas);
    }

    fn intersects(&self, other: &Rectangle<N>) -> bool {
        let self_length = (self.length)(self);
        let self_width = (self.width)(self);
        let other_length = (other.length)(other);
        let other_width = (other.width)(other);

        // Check for rectangle intersection using AABB method
        !(self.location.x + self_length < other.location.x
            || other.location.x + other_length < self.location.x
            || self.location.y + self_width < other.location.y
            || other.location.y + other_width < self.location.y)
    }
}

#[derive(Copy, Clone, Debug)]
struct Model;

impl Default for Model {
    fn default() -> Self {
        Self {}
    }
}

fn main() -> Result<(), Error> {
    let config = Config::with_dims(1200, 900);
    let model = Model::default();
    let mut app = App::app(model, config, |_, model| model, draw)
        .set_title("Boxy")
        .no_loop();
    app.run()
}

fn draw(app: &App<AppMode, Model>, _model: &Model) -> Vec<u8> {
    let mut canvas = Canvas::new(app.config.width, app.config.height);
    canvas.fill(*BLACK);

    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    let size = 95.0;
    let mut placed_rectangles = Vec::new();
    let mut attempts = 0;
    let max_attempts = 1000000;

    let (w, h) = app.wh();
    let marble_bg = marble(w, h, (*DIMGRAY).darken(0.5), *DIMGRAY, 15.0, 0);
    canvas.pixmap.draw_pixmap(
        0,
        0,
        marble_bg.pixmap.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );

    while placed_rectangles.len() < 5000 && attempts < max_attempts {
        let rect = Rectangle::new(
            pt(
                app.w_f32() * rng.gen::<f32>(),
                app.h_f32() * rng.gen::<f32>(),
            ),
            Perlin::default().set_seed(123),
            NoiseOpts::with_wh(app.w_f32(), app.h_f32()).scales(2.0),
            size,
            Box::new(|_| (*MAROON).opacity(0.75)),
            Box::new(|rect| {
                rect.size
                    * noise2d_01(
                        &rect.noise,
                        &rect.noise_opts,
                        rect.location.x,
                        rect.location.y,
                    )
            }),
            Box::new(|rect| {
                rect.size
                    * noise2d_01(
                        &rect.noise,
                        &rect.noise_opts,
                        rect.location.x,
                        rect.location.y,
                    )
            }),
        );

        let intersects = placed_rectangles
            .iter()
            .any(|placed| rect.intersects(placed));

        if !intersects {
            rect.draw(&mut canvas);
            placed_rectangles.push(rect);
        }

        attempts += 1;
    }

    canvas.take()
}
