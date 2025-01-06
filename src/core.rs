pub use pixels::Error;
use pixels::{Pixels, SurfaceTexture};
use std::time::Instant;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub struct Config {
    pub width: u32,
    pub height: u32,
}

impl Config {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn wh(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn wh_f32(&self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }
}

pub struct App<M> {
    pub model: M,
    pub config: Config,
    pub update: fn(&App<M>, M) -> M,
    pub draw: fn(&App<M>, &M) -> Vec<u8>,
    pub time: f32,
    pub window_title: String,
    pub frame_count: u32,
}

impl<M> App<M>
where
    M: Clone,
{
    pub fn new(
        model: M,
        config: Config,
        update: fn(&App<M>, M) -> M,
        draw: fn(&App<M>, &M) -> Vec<u8>,
    ) -> Self {
        Self {
            model,
            config,
            update,
            draw,
            time: 0.0,
            window_title: "Artimate".to_string(),
            frame_count: 0,
        }
    }

    pub fn set_title(self, title: &str) -> Self {
        Self {
            window_title: title.to_string(),
            ..self
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let width = self.config.width;
        let height = self.config.height;
        let event_loop = EventLoop::new().unwrap();
        let mut input = WinitInputHelper::new();
        let window = {
            let size = LogicalSize::new(width as f64, height as f64);
            WindowBuilder::new()
                .with_title(&self.window_title)
                .with_inner_size(size)
                .with_min_inner_size(size)
                .build(&event_loop)
                .unwrap()
        };

        let mut pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);

            Pixels::new(width, height, surface_texture)?
        };

        let now = Instant::now();

        let res = event_loop.run(|event, elwt| {
            self.model = (self.update)(&self, self.model.clone());
            if let Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } = event
            {
                self.time = now.elapsed().as_secs_f32();
                pixels
                    .frame_mut()
                    .copy_from_slice((self.draw)(&self, &self.model).as_ref());
                if let Err(_err) = pixels.render() {
                    elwt.exit();
                    return;
                }
            }
            // Handle input events
            if input.update(&event) {
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    elwt.exit();
                    return;
                }
                if let Some(size) = input.window_resized() {
                    if let Err(_err) = pixels.resize_surface(size.width, size.height) {
                        elwt.exit();
                        return;
                    }
                }
                self.frame_count += 1;
                self.model = (self.update)(&self, self.model.clone());
                window.request_redraw();
            }
        });

        println!();
        println!(
            "Average FPS: {}",
            self.frame_count as f32 / now.elapsed().as_secs_f32(),
        );
        println!("Frame count: {}", self.frame_count,);
        println!("Elapsed time: {} seconds", now.elapsed().as_secs_f32(),);

        res.map_err(|e| Error::UserDefined(Box::new(e)))
    }
}
