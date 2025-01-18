use dirs;
pub use pixels::Error;
use pixels::{Pixels, SurfaceTexture};
use png::Encoder;
use std::sync::mpsc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{CursorIcon, Window, WindowId},
};

pub struct Config {
    pub width: u32,
    pub height: u32,
    pub no_loop: bool,
    pub frames: Option<u32>,
    pub cursor_visible: bool,
    pub frames_to_save: u32,
}

impl Config {
    pub fn new(
        width: u32,
        height: u32,
        no_loop: bool,
        cursor_visible: bool,
        frames_to_save: u32,
    ) -> Self {
        Self {
            width,
            height,
            no_loop,
            frames: None,
            cursor_visible,
            frames_to_save,
        }
    }

    pub fn from_dims(width: u32, height: u32) -> Self {
        Self::new(width, height, false, true, 0)
    }

    pub fn wh(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn wh_f32(&self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }

    pub fn set_frames_to_save(self, frames_to_save: u32) -> Self {
        Self {
            frames_to_save,
            ..self
        }
    }

    pub fn set_cursor_visibility(self, cursor_visible: bool) -> Self {
        Self {
            cursor_visible,
            ..self
        }
    }

    pub fn no_loop(self) -> Self {
        Self {
            no_loop: true,
            ..self
        }
    }

    pub fn set_frames(self, frames: u32) -> Self {
        Self {
            frames: Some(frames),
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(1080, 700, false, true, 0)
    }
}

pub struct App<M = ()> {
    pub model: M,
    pub config: Config,
    pub update: fn(&App<M>, M) -> M,
    pub draw: fn(&App<M>, &M) -> Vec<u8>,
    pub time: f32,
    pub start_time: Instant,
    pub window_title: String,
    pub frame_count: u32,
    window: Option<Window>,
    pub mouse_position: (f32, f32),
    frame_sender: Option<mpsc::Sender<(Box<[u8]>, String, u32, u32)>>,
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
        let mut maybe_tx = None;
        if config.frames_to_save > 0 {
            let (tx, rx): (
                mpsc::Sender<(Box<[u8]>, String, u32, u32)>,
                mpsc::Receiver<(Box<[u8]>, String, u32, u32)>,
            ) = mpsc::channel();

            // Spawn background thread for saving frames
            std::thread::spawn(move || {
                while let Ok((frame_data, filename, width, height)) = rx.recv() {
                    // Create the PNG encoder
                    let file = std::fs::File::create(&filename).unwrap();
                    let mut encoder = Encoder::new(file, width, height);
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);

                    let mut writer = encoder.write_header().unwrap();
                    writer.write_image_data(&frame_data[..]).unwrap();
                }
            });
            maybe_tx = Some(tx);
        }
        Self {
            model,
            config,
            update,
            draw,
            time: 0.0,
            window_title: "Artimate".to_string(),
            frame_count: 0,
            window: None,
            start_time: Instant::now(),
            mouse_position: (0.0, 0.0),
            frame_sender: maybe_tx,
        }
    }

    pub fn set_title(self, title: &str) -> Self {
        Self {
            window_title: title.to_string(),
            ..self
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let now = Instant::now();
        let res = event_loop.run_app(self);

        println!();
        println!(
            "Average FPS: {}",
            self.frame_count as f32 / now.elapsed().as_secs_f32(),
        );
        println!("Frame count: {}", self.frame_count,);
        println!("Elapsed time: {} seconds", now.elapsed().as_secs_f32(),);

        res.map_err(|e| Error::UserDefined(Box::new(e)))
    }

    pub fn mouse_x(&self) -> f32 {
        self.mouse_position.0
    }

    pub fn mouse_y(&self) -> f32 {
        self.mouse_position.1
    }
}

impl<M> ApplicationHandler for App<M>
where
    M: Clone,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = LogicalSize::new(self.config.width as f64, self.config.height as f64);
        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(self.window_title.clone())
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        let window_size = window.inner_size();
        let mut pixels = {
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);

            Pixels::new(self.config.width, self.config.height, surface_texture).unwrap()
        };

        self.time = self.start_time.elapsed().as_secs_f32();

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Exit on escape key
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    event_loop.exit();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = &self.window {
                    let scale_factor = window.scale_factor();
                    let logical_position = position.to_logical(scale_factor);
                    self.mouse_position = (logical_position.x, logical_position.y);
                }
            }
            WindowEvent::CursorEntered { .. } => {
                if let Some(window) = &self.window {
                    if self.config.cursor_visible {
                        window.set_cursor(CursorIcon::Crosshair);
                    } else {
                        window.set_cursor_visible(false);
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                // Show cursor when it leaves the window
                if let Some(window) = &self.window {
                    window.set_cursor(CursorIcon::Default);
                    window.set_cursor_visible(true);
                }
            }
            WindowEvent::RedrawRequested => {
                pixels
                    .frame_mut()
                    .copy_from_slice((self.draw)(&self, &self.model).as_ref());

                if self.frame_count > 0 && self.frame_count <= self.config.frames_to_save {
                    if let Some(sender) = &self.frame_sender {
                        let frame_data: Box<[u8]> = pixels.frame().to_vec().into();
                        let downloads_dir =
                            dirs::download_dir().expect("Could not find Downloads directory");
                        let output_dir = downloads_dir.join("frames");
                        std::fs::create_dir_all(&output_dir)
                            .expect("Failed to create frames directory");
                        let filename =
                            output_dir.join(format!("frame_{:04}.png", self.frame_count));
                        sender
                            .send((
                                frame_data,
                                filename.to_string_lossy().to_string(),
                                self.config.width,
                                self.config.height,
                            ))
                            .unwrap();
                    }
                }

                if let Err(_err) = pixels.render() {
                    event_loop.exit();
                    return;
                }

                self.model = (self.update)(&self, self.model.clone());
                self.frame_count += 1;

                if !self.config.no_loop {
                    if let Some(frames) = self.config.frames {
                        if self.frame_count <= frames {
                            self.window.as_ref().unwrap().request_redraw();
                        }
                    } else {
                        self.window.as_ref().unwrap().request_redraw();
                    }
                }
            }
            _ => (),
        }
    }
}
