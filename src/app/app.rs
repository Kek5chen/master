use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use ash::extensions::khr;
use ash::{Entry, vk};
use ash_window::enumerate_required_extensions;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use raw_window_handle::{HasRawDisplayHandle};

pub struct App {
    event_loop: Option<EventLoop<()>>,
    window: Option<winit::window::Window>,
    entry: ash::Entry,
}

impl App {
    pub fn new(app_name: &str, window_width: u32, window_height: u32) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let mut app = App {
                event_loop: None,
                window: None,
                entry: Entry::linked()
            };

            app.create_window(app_name, window_width, window_height)?;
            Ok(app)
        }
    }

    fn create_window(&mut self, title: &str, width: u32, height: u32) -> Result<(), Box<dyn Error>> {
        let event_loop = EventLoop::new()?;
        self.window = Some(WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                width,
                height,
            ))
            .build(&event_loop)
            .unwrap());
        self.event_loop = Some(event_loop);

        Ok(())
    }
}