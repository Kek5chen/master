use std::error::Error;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct App {
    entry: ash::Entry,
}

impl App {
    pub fn new(app_name: &str, window_width: u32, window_height: u32) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let event_loop = EventLoop::new()?;
            let window = WindowBuilder::new()
                .with_title(app_name)
                .with_inner_size(winit::dpi::LogicalSize::new(
                    f64::from(window_width),
                    f64::from(window_height),
                ))
                .build(&event_loop)
                .unwrap();

            Ok(App{entry})
        }
    }
}