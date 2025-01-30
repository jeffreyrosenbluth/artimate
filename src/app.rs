use dirs;
pub use pixels::Error;
use pixels::{Pixels, SurfaceTexture};
use png::Encoder;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::Key,
    window::{CursorIcon, Window, WindowId},
};

const DEFAULT_WIDTH: u32 = 1080;
const DEFAULT_HEIGHT: u32 = 700;
const DEFAULT_TITLE: &str = "Artimate";

/// Configuration for the application window and rendering behavior
#[derive(Debug)]
pub struct Config {
    /// Width of the window in pixels
    pub width: u32,
    /// Height of the window in pixels
    pub height: u32,
    /// If true, the application will only render one frame
    pub no_loop: bool,
    /// Optional limit on the number of frames to render
    pub frames: Option<u32>,
    /// Controls whether the cursor is visible in the window
    pub cursor_visible: bool,
    /// Number of frames to save as PNG files
    pub frames_to_save: u32,
}

impl Config {
    /// Creates a new configuration with the specified parameters
    ///
    /// # Arguments
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    /// * `no_loop` - If true, renders only one frame
    /// * `cursor_visible` - Controls cursor visibility
    /// * `frames_to_save` - Number of frames to save as PNG files
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

    /// Creates a new configuration with just width and height
    /// Other parameters are set to their defaults
    pub fn with_dims(width: u32, height: u32) -> Self {
        Self::new(width, height, false, true, 0)
    }

    /// Returns the width and height as a tuple of u32
    pub fn wh(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the width and height as a tuple of f32
    pub fn wh_f32(&self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }

    /// Returns the width as a f32
    pub fn w_f32(&self) -> f32 {
        self.width as f32
    }

    /// Returns the height as a f32
    pub fn h_f32(&self) -> f32 {
        self.height as f32
    }

    /// Sets the number of frames to save and returns updated config
    pub fn set_frames_to_save(self, frames_to_save: u32) -> Self {
        Self {
            frames_to_save,
            ..self
        }
    }

    /// Sets cursor visibility and returns updated config
    pub fn set_cursor_visibility(self, cursor_visible: bool) -> Self {
        Self {
            cursor_visible,
            ..self
        }
    }

    /// Sets no_loop to true and returns updated config
    pub fn no_loop(self) -> Self {
        Self {
            no_loop: true,
            ..self
        }
    }

    /// Sets the frame limit and returns updated config
    pub fn set_frames(self, frames: u32) -> Self {
        Self {
            frames: Some(frames),
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, false, true, 0)
    }
}

/// Marker type for simple sketches that only need drawing functionality
pub struct SketchMode;
/// Marker type for stateful sketches that need both model state and update functionality
pub struct AppMode;

/// Main application struct that handles window management and rendering
///
/// # Type Parameters
/// * `M` - The type of the model/state used in the application
pub struct App<Mode = SketchMode, M = ()> {
    /// The application's model/state
    pub model: M,
    /// Configuration settings for the application
    pub config: Config,
    /// Function called each frame to update the model
    pub update: Option<fn(&App<Mode, M>, M) -> M>,
    /// Function called each frame to generate pixel data
    pub draw: fn(&App<Mode, M>, &M) -> Vec<u8>,
    /// Time elapsed since application start in seconds
    pub time: f32,
    /// Instant when the application started
    pub start_time: Instant,
    /// Title of the application window
    pub window_title: String,
    /// Number of frames rendered
    pub frame_count: u32,
    /// Window handle
    window: Option<Window>,
    /// Current mouse position as (x, y) coordinates
    pub mouse_position: (f32, f32),
    /// Channel for sending frame data to be saved
    frame_sender: Option<mpsc::Sender<(Vec<u8>, String, u32, u32)>>,
    /// Map of key handlers for custom key events
    key_handlers: HashMap<Key, Rc<dyn Fn(&mut App<Mode, M>)>>,
    /// Map of mouse button handlers for custom mouse events
    mouse_handlers: HashMap<MouseButton, Rc<dyn Fn(&mut App<Mode, M>)>>,
    key_press_handlers: HashMap<Key, Rc<dyn Fn(&mut App<Mode, M>)>>,
    key_release_handlers: HashMap<Key, Rc<dyn Fn(&mut App<Mode, M>)>>,
    keys_down: HashSet<Key>,
    _mode: PhantomData<Mode>,
}

// Helper function for frame saving setup
fn setup_frame_sender() -> Option<mpsc::Sender<(Vec<u8>, String, u32, u32)>> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        while let Ok((frame_data, filename, width, height)) = rx.recv() {
            if let Err(err) = save_frame(frame_data, filename, width, height) {
                eprintln!("Failed to save frame: {}", err);
            }
        }
    });

    Some(tx)
}

fn save_frame(
    frame_data: Vec<u8>,
    filename: String,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(&filename)?;
    let mut encoder = Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&frame_data)?;
    Ok(())
}

/// Simple sketches that only need drawing functionality
impl App<SketchMode> {
    /// Creates a simple sketch with just a draw function and config
    pub fn sketch(config: Config, draw: fn(&App<SketchMode, ()>, &()) -> Vec<u8>) -> Self {
        let maybe_tx = if config.frames_to_save > 0 {
            setup_frame_sender()
        } else {
            None
        };

        Self {
            model: (),
            config,
            update: None,
            draw,
            time: 0.0,
            window_title: DEFAULT_TITLE.to_string(),
            frame_count: 0,
            window: None,
            start_time: Instant::now(),
            mouse_position: (0.0, 0.0),
            frame_sender: maybe_tx,
            key_handlers: HashMap::new(),
            mouse_handlers: HashMap::new(),
            key_press_handlers: HashMap::new(),
            key_release_handlers: HashMap::new(),
            keys_down: HashSet::new(),
            _mode: PhantomData,
        }
    }
}

/// Stateful sketches that need both model state and update functionality
impl<M> App<AppMode, M>
where
    M: Clone,
{
    /// Creates a new application instance
    ///
    /// # Arguments
    /// * `model` - Initial state of the application
    /// * `config` - Configuration settings
    /// * `update` - Function called each frame to update the model
    /// * `draw` - Function called each frame to generate pixel data
    pub fn app(
        model: M,
        config: Config,
        update: fn(&App<AppMode, M>, M) -> M,
        draw: fn(&App<AppMode, M>, &M) -> Vec<u8>,
    ) -> Self {
        let maybe_tx = if config.frames_to_save > 0 {
            setup_frame_sender()
        } else {
            None
        };

        Self {
            model,
            config,
            update: Some(update),
            draw,
            time: 0.0,
            window_title: DEFAULT_TITLE.to_string(),
            frame_count: 0,
            window: None,
            start_time: Instant::now(),
            mouse_position: (0.0, 0.0),
            frame_sender: maybe_tx,
            key_handlers: HashMap::new(),
            mouse_handlers: HashMap::new(),
            key_press_handlers: HashMap::new(),
            key_release_handlers: HashMap::new(),
            keys_down: HashSet::new(),
            _mode: PhantomData,
        }
    }
}

/// Common methods for both sketch and app modes
impl<Mode, M: Clone> App<Mode, M> {
    /// Sets the window title and returns updated app
    pub fn set_title(self, title: &str) -> Self {
        Self {
            window_title: title.to_string(),
            ..self
        }
    }

    /// Starts the application's main loop
    ///
    /// Returns an error if the window creation or rendering fails
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

    /// Returns the current x-coordinate of the mouse
    pub fn mouse_x(&self) -> f32 {
        self.mouse_position.0
    }

    /// Returns the current y-coordinate of the mouse
    pub fn mouse_y(&self) -> f32 {
        self.mouse_position.1
    }
}

/// Implementation of ApplicationHandler for App
impl<Mode, M> ApplicationHandler for App<Mode, M>
where
    M: Clone,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = LogicalSize::new(self.config.width as f64, self.config.height as f64);
        self.window.get_or_insert_with(|| {
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(self.window_title.clone())
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .unwrap()
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if event != WindowEvent::RedrawRequested {
            println!("Received event: {:?}", event); // Log all window events
        }

        let window = self.window.as_ref().unwrap();
        let window_size = window.inner_size();

        self.time = self.start_time.elapsed().as_secs_f32();

        match event {
            WindowEvent::CloseRequested => {
                println!("Close Requested");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event, event_loop);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if state == winit::event::ElementState::Pressed {
                    self.handle_mouse_input(button);
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
                let mut pixels = {
                    let surface_texture =
                        SurfaceTexture::new(window_size.width, window_size.height, &window);

                    Pixels::new(self.config.width, self.config.height, surface_texture).unwrap()
                };
                pixels
                    .frame_mut()
                    .copy_from_slice((self.draw)(&self, &self.model).as_ref());

                if self.frame_count < self.config.frames_to_save {
                    if let Some(sender) = &self.frame_sender {
                        let frame_data: Vec<u8> = pixels.frame().to_vec();
                        if let Some(downloads_dir) = dirs::download_dir() {
                            let output_dir = downloads_dir.join("frames");
                            if let Err(err) = std::fs::create_dir_all(&output_dir) {
                                eprintln!("Failed to create frames directory: {}", err);
                            } else {
                                let filename =
                                    output_dir.join(format!("frame_{:04}.png", self.frame_count));
                                if let Err(err) = sender.send((
                                    frame_data,
                                    filename.to_string_lossy().to_string(),
                                    self.config.width,
                                    self.config.height,
                                )) {
                                    eprintln!("Failed to send frame data: {}", err);
                                }
                            }
                        }
                    }
                }

                if let Err(_err) = pixels.render() {
                    event_loop.exit();
                    return;
                }

                if let Some(update) = self.update {
                    self.model = update(&self, self.model.clone());
                }

                if !self.config.no_loop {
                    if let Some(frames) = self.config.frames {
                        if self.frame_count < frames {
                            window.request_redraw();
                        }
                    } else {
                        window.request_redraw();
                    }
                }
                self.frame_count += 1;
            }
            _ => (),
        }
    }
}

/// Handles keyboard input for the application
impl<Mode, M> App<Mode, M>
where
    M: Clone,
{
    fn handle_keyboard_input(
        &mut self,
        event: winit::event::KeyEvent,
        _event_loop: &winit::event_loop::ActiveEventLoop,
    ) {
        match event.state {
            winit::event::ElementState::Pressed => {
                self.keys_down.insert(event.logical_key.clone());
                // Handle one-time press events
                if let Some(handler) = self.key_press_handlers.get(&event.logical_key).cloned() {
                    handler(self);
                }
            }
            winit::event::ElementState::Released => {
                self.keys_down.remove(&event.logical_key);
                // Handle release events
                if let Some(handler) = self.key_release_handlers.get(&event.logical_key).cloned() {
                    handler(self);
                }
            }
        }

        // Handle continuous key holding in the update/draw loop
        if event.state == winit::event::ElementState::Pressed {
            if let Some(handler) = self.key_handlers.get(&event.logical_key).cloned() {
                handler(self);
            }
        }
    }

    fn handle_mouse_input(&mut self, button: MouseButton) {
        let handler = self.mouse_handlers.get(&button).cloned();
        if let Some(handler) = handler {
            handler(self);
        }
    }
}

/// Handles key and mouse events for stateful sketches
impl<M: Clone> App<AppMode, M> {
    pub fn on_key_held<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<AppMode, M>) + 'static,
    {
        self.key_handlers.insert(key, Rc::new(handler));
    }

    pub fn on_key_press<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<AppMode, M>) + 'static,
    {
        self.key_press_handlers.insert(key, Rc::new(handler));
    }

    pub fn on_key_release<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<AppMode, M>) + 'static,
    {
        self.key_release_handlers.insert(key, Rc::new(handler));
    }

    pub fn on_mouse_press<F>(&mut self, button: MouseButton, handler: F)
    where
        F: Fn(&mut App<AppMode, M>) + 'static,
    {
        self.mouse_handlers.insert(button, Rc::new(handler));
    }
}

impl App<SketchMode> {
    pub fn on_key_held<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<SketchMode, ()>) + 'static,
    {
        self.key_handlers.insert(key, Rc::new(handler));
    }

    pub fn on_key_press<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<SketchMode, ()>) + 'static,
    {
        self.key_press_handlers.insert(key, Rc::new(handler));
    }

    pub fn on_key_release<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<SketchMode, ()>) + 'static,
    {
        self.key_release_handlers.insert(key, Rc::new(handler));
    }

    pub fn on_mouse_press<F>(&mut self, button: MouseButton, handler: F)
    where
        F: Fn(&mut App<SketchMode, ()>) + 'static,
    {
        self.mouse_handlers.insert(button, Rc::new(handler));
    }
}
