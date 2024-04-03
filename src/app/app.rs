use std::error::Error;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct App {
    event_loop: Option<EventLoop<()>>,
    window: Option<winit::window::Window>,
    entry: ash::Entry,
}

impl App {
    pub fn new(app_name: &str, window_width: u32, window_height: u32) -> Result<Self, Box<dyn Error>> {
        let entry = Self::initialize_vulkan()?;

        let mut app = App {
            event_loop: None,
            window: None,
            entry,
        };

        app.create_window(app_name, window_width, window_height)?;
        // Create a Vulkan instance
        // Create a debug messenger (optional)
        // Create a Vulkan surface
        // Select a physical device
        // Create a logical device and queues
        // Create a swap chain
        // Create image views
        // Setup framebuffers, command pools, and command buffers
        // Initialize synchronization primitives

        Ok(app)

    }

    fn initialize_vulkan() -> Result<ash::Entry, Box<dyn Error>> {
        unsafe {
            let entry = ash::Entry::load()?;

            Ok(entry)
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