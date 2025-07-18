use delegate::delegate;
use dirs;
pub use pixels::Error;
use pixels::{Pixels, SurfaceTexture};
use png::Encoder;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{Modifiers, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, ModifiersKeyState},
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
    /// Optional limit on the number of frames to render, if None, the application will render indefinitely.
    pub frames: Option<u32>,
    /// Controls whether the cursor is visible in the window
    pub cursor_visible: bool,
    /// Number of frames to save as PNG files
    pub frames_to_save: u32,
    /// Title of the application window
    pub window_title: String,
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
    /// * `window_title` - Title of the application window
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
            window_title: DEFAULT_TITLE.to_string(),
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

    /// Sets the window title and returns updated config
    pub fn set_title(self, title: &str) -> Self {
        Self {
            window_title: title.to_string(),
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
/// 
/// Used with `App::sketch()` to create applications that don't need persistent state.
/// Perfect for static graphics, simple animations, or interactive graphics that only
/// depend on time and mouse position.
pub struct SketchMode;

/// Marker type for stateful sketches that need both model state and update functionality
/// 
/// Used with `App::app()` to create applications that maintain state between frames.
/// The model is updated each frame via an update function, allowing for complex
/// animations and interactive applications.
pub struct AppMode;

/// Main application struct that handles window management and rendering
///
/// Artimate provides a simple framework for creating pixel-based graphics applications.
/// The `App` struct manages the window lifecycle, input handling, and rendering pipeline.
///
/// # Type Parameters
/// * `Mode` - The application mode, either `SketchMode` for simple sketches or `AppMode` for stateful applications
/// * `M` - The type of the model/state used in the application
/// 
/// # Examples
/// 
/// ## Simple Sketch
/// ```rust,no_run
/// use artimate::app::{App, Config, Error};
/// 
/// fn main() -> Result<(), Error> {
///     let config = Config::with_dims(800, 600);
///     let mut app = App::sketch(config, draw);
///     app.run()
/// }
/// 
/// fn draw(app: &App, _model: &()) -> Vec<u8> {
///     // Return RGBA pixel data
///     vec![255; (app.config.width * app.config.height * 4) as usize]
/// }
/// ```
/// 
/// ## Stateful Application
/// ```rust,no_run
/// use artimate::app::{App, AppMode, Config, Error};
/// 
/// #[derive(Default, Clone)]
/// struct Model {
///     counter: i32,
/// }
/// 
/// fn main() -> Result<(), Error> {
///     let config = Config::with_dims(800, 600);
///     let model = Model::default();
///     let mut app = App::app(model, config, update, draw);
///     app.run()
/// }
/// 
/// fn update(app: &App<AppMode, Model>, mut model: Model) -> Model {
///     model.counter += 1;
///     model
/// }
/// 
/// fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
///     // Return RGBA pixel data based on model state
///     vec![255; (app.config.width * app.config.height * 4) as usize]
/// }
/// ```
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
    /// Number of frames rendered
    pub frame_count: u32,
    /// Window handle
    window: Option<Arc<Window>>,
    /// Pixels handle
    pixels: Option<Pixels<'static>>,
    /// Current mouse position as (x, y) coordinates
    pub mouse_position: (f32, f32),
    /// Channel for sending frame data to be saved
    frame_sender: Option<mpsc::Sender<(Vec<u8>, String, u32, u32)>>,
    /// Map of key handlers for custom key events
    key_handlers: HashMap<Key, Rc<dyn Fn(&mut App<Mode, M>)>>,
    /// Map of mouse button handlers for custom mouse events
    mouse_handlers: HashMap<MouseButton, Rc<dyn Fn(&mut App<Mode, M>)>>,
    /// Map of key press handlers for custom key events
    key_press_handlers: HashMap<Key, Rc<dyn Fn(&mut App<Mode, M>)>>,
    /// Map of key release handlers for custom key events
    key_release_handlers: HashMap<Key, Rc<dyn Fn(&mut App<Mode, M>)>>,
    /// Set of keys currently held down
    keys_down: HashSet<Key>,
    /// Modifiers state
    modifiers: Modifiers,
    /// Phantom data for mode type
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
    /// Creates a simple sketch application with just a draw function and configuration
    /// 
    /// This is the simplest way to create an Artimate application. It's perfect for
    /// static graphics, animations that don't need persistent state, or simple
    /// interactive graphics that only depend on time and mouse position.
    ///
    /// # Arguments
    /// * `config` - Configuration settings for the window and rendering
    /// * `draw` - Function called each frame to generate RGBA pixel data
    ///
    /// # Examples
    /// ```rust,no_run
    /// use artimate::app::{App, Config, Error};
    /// 
    /// fn main() -> Result<(), Error> {
    ///     let config = Config::with_dims(400, 400);
    ///     let mut app = App::sketch(config, draw);
    ///     app.run()
    /// }
    /// 
    /// fn draw(app: &App, _model: &()) -> Vec<u8> {
    ///     // Create a simple animated circle
    ///     let mut pixels = vec![0u8; (app.config.width * app.config.height * 4) as usize];
    ///     // Fill with pixel data...
    ///     pixels
    /// }
    /// ```
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
            frame_count: 0,
            window: None,
            pixels: None,
            start_time: Instant::now(),
            mouse_position: (0.0, 0.0),
            frame_sender: maybe_tx,
            key_handlers: HashMap::new(),
            mouse_handlers: HashMap::new(),
            key_press_handlers: HashMap::new(),
            key_release_handlers: HashMap::new(),
            keys_down: HashSet::new(),
            modifiers: Modifiers::default(),
            _mode: PhantomData,
        }
    }
}

/// Stateful sketches that need both model state and update functionality
impl<M> App<AppMode, M>
where
    M: Clone,
{
    /// Creates a stateful application with model, update, and draw functions
    ///
    /// This method creates a full-featured application that can maintain state
    /// between frames. The model is updated each frame via the update function,
    /// and the draw function generates pixel data based on the current model state.
    ///
    /// # Arguments
    /// * `model` - Initial state of the application
    /// * `config` - Configuration settings for the window and rendering
    /// * `update` - Function called each frame to update the model based on app state
    /// * `draw` - Function called each frame to generate RGBA pixel data from the model
    ///
    /// # Examples
    /// ```rust,no_run
    /// use artimate::app::{App, AppMode, Config, Error};
    /// 
    /// #[derive(Clone)]
    /// struct Model {
    ///     position: f32,
    ///     direction: f32,
    /// }
    /// 
    /// fn main() -> Result<(), Error> {
    ///     let config = Config::with_dims(800, 600);
    ///     let model = Model { position: 0.0, direction: 1.0 };
    ///     let mut app = App::app(model, config, update, draw);
    ///     app.run()
    /// }
    /// 
    /// fn update(app: &App<AppMode, Model>, mut model: Model) -> Model {
    ///     model.position += model.direction * 100.0 * (1.0 / 60.0); // 60 FPS
    ///     if model.position > app.config.width as f32 {
    ///         model.direction = -1.0;
    ///     } else if model.position < 0.0 {
    ///         model.direction = 1.0;
    ///     }
    ///     model
    /// }
    /// 
    /// fn draw(app: &App<AppMode, Model>, model: &Model) -> Vec<u8> {
    ///     // Generate pixel data based on model
    ///     vec![255; (app.config.width * app.config.height * 4) as usize]
    /// }
    /// ```
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
            frame_count: 0,
            window: None,
            pixels: None,
            start_time: Instant::now(),
            mouse_position: (0.0, 0.0),
            frame_sender: maybe_tx,
            key_handlers: HashMap::new(),
            mouse_handlers: HashMap::new(),
            key_press_handlers: HashMap::new(),
            key_release_handlers: HashMap::new(),
            keys_down: HashSet::new(),
            modifiers: Modifiers::default(),
            _mode: PhantomData,
        }
    }
}

/// Common methods for both sketch and app modes
impl<Mode, M> App<Mode, M>
where
    M: Clone,
{
    /// Starts the application's main loop and runs until the window is closed
    ///
    /// This method creates the window, initializes the rendering context, and begins
    /// the main event loop. It handles window events, updates the model (if in AppMode),
    /// calls the draw function, and renders the result to the screen.
    ///
    /// The method will block until the application is closed and will print performance
    /// statistics (FPS, frame count, elapsed time) when the application exits.
    ///
    /// # Returns
    /// * `Ok(())` - If the application ran successfully and was closed normally
    /// * `Err(Error)` - If there was an error during window creation or rendering
    ///
    /// # Examples
    /// ```rust,no_run
    /// use artimate::app::{App, Config, Error};
    /// 
    /// fn main() -> Result<(), Error> {
    ///     let config = Config::with_dims(800, 600);
    ///     let mut app = App::sketch(config, draw);
    ///     app.run() // Blocks until window is closed
    /// }
    /// 
    /// fn draw(app: &App, _model: &()) -> Vec<u8> {
    ///     vec![255; (app.config.width * app.config.height * 4) as usize]
    /// }
    /// ```
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

    /// Returns the current x-coordinate of the mouse cursor in pixels
    ///
    /// The coordinate is relative to the top-left corner of the window,
    /// with positive values extending to the right.
    pub fn mouse_x(&self) -> f32 {
        self.mouse_position.0
    }

    /// Returns the current y-coordinate of the mouse cursor in pixels
    ///
    /// The coordinate is relative to the top-left corner of the window,
    /// with positive values extending downward.
    pub fn mouse_y(&self) -> f32 {
        self.mouse_position.1
    }

    delegate! {
        to self.config {
            pub fn wh(&self) -> (u32, u32);
            pub fn wh_f32(&self) -> (f32, f32);
            pub fn w_f32(&self) -> f32;
            pub fn h_f32(&self) -> f32;
        }
    }

    /// Sets the number of frames to save as PNG files and returns updated app
    /// 
    /// Frames are saved to the Downloads/frames directory with timestamps.
    /// Set to 0 to disable frame saving.
    pub fn set_frames_to_save(mut self, frames_to_save: u32) -> Self {
        self.config = self.config.set_frames_to_save(frames_to_save);
        self
    }

    /// Sets cursor visibility in the window and returns updated app
    pub fn set_cursor_visibility(mut self, cursor_visible: bool) -> Self {
        self.config = self.config.set_cursor_visibility(cursor_visible);
        self
    }

    /// Configures the app to render only one frame and returns updated app
    /// 
    /// Useful for generating static images or when you want to control
    /// the animation loop manually.
    pub fn no_loop(mut self) -> Self {
        self.config = self.config.no_loop();
        self
    }

    /// Sets the maximum number of frames to render and returns updated app
    /// 
    /// The application will exit after rendering this many frames.
    pub fn set_frames(mut self, frames: u32) -> Self {
        self.config = self.config.set_frames(frames);
        self
    }

    /// Sets the window title and returns updated app
    pub fn set_title(self, title: &str) -> Self {
        Self {
            config: self.config.set_title(title),
            ..self
        }
    }

    /// Registers a handler function for when a key is held down
    ///
    /// # Arguments
    /// * `key` - The key to watch for
    /// * `handler` - The function to call while the key is held
    pub fn on_key_held<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<Mode, M>) + 'static,
    {
        self.key_handlers.insert(key, Rc::new(handler));
    }

    /// Registers a handler function for when a key is initially pressed
    ///
    /// # Arguments
    /// * `key` - The key to watch for
    /// * `handler` - The function to call when the key is pressed
    pub fn on_key_press<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<Mode, M>) + 'static,
    {
        self.key_press_handlers.insert(key, Rc::new(handler));
    }

    /// Registers a handler function for when a key is released
    ///
    /// # Arguments
    /// * `key` - The key to watch for
    /// * `handler` - The function to call when the key is released
    pub fn on_key_release<F>(&mut self, key: Key, handler: F)
    where
        F: Fn(&mut App<Mode, M>) + 'static,
    {
        self.key_release_handlers.insert(key, Rc::new(handler));
    }

    /// Registers a handler function for when a mouse button is pressed
    ///
    /// # Arguments
    /// * `button` - The mouse button to watch for
    /// * `handler` - The function to call when the button is pressed
    pub fn on_mouse_press<F>(&mut self, button: MouseButton, handler: F)
    where
        F: Fn(&mut App<Mode, M>) + 'static,
    {
        self.mouse_handlers.insert(button, Rc::new(handler));
    }

    /// Processes keyboard input events and triggers appropriate handlers
    ///
    /// # Arguments
    /// * `event` - The keyboard event to process
    /// * `_event_loop` - The event loop instance
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
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            winit::event::ElementState::Released => {
                self.keys_down.remove(&event.logical_key);
                // Handle release events
                if let Some(handler) = self.key_release_handlers.get(&event.logical_key).cloned() {
                    handler(self);
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
        }

        // Handle continuous key holding in the update/draw loop
        if event.state == winit::event::ElementState::Pressed {
            if let Some(handler) = self.key_handlers.get(&event.logical_key).cloned() {
                handler(self);
                self.window.as_ref().unwrap().request_redraw();
            }
        }
    }

    /// Processes mouse input events and triggers appropriate handlers
    ///
    /// # Arguments
    /// * `button` - The mouse button that was pressed
    fn handle_mouse_input(&mut self, button: MouseButton) {
        let handler = self.mouse_handlers.get(&button).cloned();
        if let Some(handler) = handler {
            handler(self);
            self.window.as_ref().unwrap().request_redraw();
        }
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
            Arc::new(event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(self.config.window_title.clone())
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                )
                .unwrap())
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_ref().unwrap();
        let window_size = window.inner_size();

        self.time = self.start_time.elapsed().as_secs_f32();

        match event {
            WindowEvent::CloseRequested => {
                println!("Close Requested");
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(new_mods) => {
                self.modifiers = new_mods; // Update stored modifier state
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let Key::Character(ref text) = event.logical_key {
                        if text == "s" {
                            if self.modifiers.lsuper_state() == ModifiersKeyState::Pressed
                                || self.modifiers.rsuper_state() == ModifiersKeyState::Pressed
                            {
                                let draw_result = (self.draw)(&self, &self.model);
                                if let Some(pixels) = self.pixels.as_mut() {
                                    pixels.frame_mut().copy_from_slice(draw_result.as_ref());
                                    let frame_data: Vec<u8> = pixels.frame().to_vec();
                                    if let Some(downloads_dir) = dirs::download_dir() {
                                        let output_dir = downloads_dir.join("artmate");
                                        if let Err(err) = std::fs::create_dir_all(&output_dir) {
                                            eprintln!("Failed to create frames directory: {}", err);
                                        } else {
                                            let timestamp = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap()
                                                .as_secs();
                                            let filename = output_dir
                                                .join(format!("artmate_{}.png", timestamp));
                                            save_frame(
                                                frame_data,
                                                filename.to_string_lossy().to_string(),
                                                self.config.width,
                                                self.config.height,
                                            )
                                            .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
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
                self.pixels.get_or_insert_with(|| {
                    let surface_texture =
                        SurfaceTexture::new(window_size.width, window_size.height, window.clone());
                    Pixels::new(self.config.width, self.config.height, surface_texture).unwrap()
                });

                let draw_result = (self.draw)(&self, &self.model);

                if let Some(pixels) = self.pixels.as_mut() {
                    pixels.frame_mut().copy_from_slice(draw_result.as_ref());

                    if self.frame_count < self.config.frames_to_save {
                        if let Some(sender) = &self.frame_sender {
                            let frame_data: Vec<u8> = pixels.frame().to_vec();
                            if let Some(downloads_dir) = dirs::download_dir() {
                                let output_dir = downloads_dir.join("frames");
                                if let Err(err) = std::fs::create_dir_all(&output_dir) {
                                    eprintln!("Failed to create frames directory: {}", err);
                                } else {
                                    let timestamp = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs();
                                    let filename = output_dir.join(format!(
                                        "frame_{}_{:04}.png",
                                        timestamp, self.frame_count
                                    ));
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
