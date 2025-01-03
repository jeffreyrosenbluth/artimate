use pixels::{Error, Pixels, SurfaceTexture};
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
}

pub struct App<M> {
    pub model: M,
    pub config: Config,
    pub update: fn(M) -> M,
    pub view: fn(&M, f32) -> Vec<u8>,
}

impl<M> App<M>
where
    M: Clone,
{
    pub fn new(model: M, config: Config, update: fn(M) -> M, view: fn(&M, f32) -> Vec<u8>) -> Self {
        Self {
            model,
            config,
            update,
            view,
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
                .with_title("Hello tiny-skia")
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
            // Draw the current frame
            self.model = (self.update)(self.model.clone());
            if let Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } = event
            {
                pixels.frame_mut().copy_from_slice(
                    (self.view)(&self.model, now.elapsed().as_secs_f32()).as_ref(),
                );
                if let Err(_err) = pixels.render() {
                    // log_error("pixels.render", err);
                    elwt.exit();
                    return;
                }
            }

            // Handle input events
            if input.update(&event) {
                // Close events
                if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                    elwt.exit();
                    return;
                }

                // Resize the window
                if let Some(size) = input.window_resized() {
                    if let Err(_err) = pixels.resize_surface(size.width, size.height) {
                        // log_error("pixels.resize_surface", err);
                        elwt.exit();
                        return;
                    }
                }

                // Update internal state and request a redraw
                self.model = (self.update)(self.model.clone());
                window.request_redraw();
            }
        });
        res.map_err(|e| Error::UserDefined(Box::new(e)))
    }
}
