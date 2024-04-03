use std::error::Error;
use std::ffi::{CStr, CString};
use ash::vk;
use ash::vk::InstanceCreateFlags;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct App {
    event_loop: Option<EventLoop<()>>,
    window: Option<winit::window::Window>,
    entry: ash::Entry,
    vk_instance: Option<ash::Instance>,
}

impl App {
    pub fn new(app_name: &str, window_width: u32, window_height: u32) -> Result<Self, Box<dyn Error>> {
        let entry = Self::initialize_vulkan()?;

        let mut app = App {
            event_loop: None,
            window: None,
            entry,
            vk_instance: None,
        };

        app.create_window(app_name, window_width, window_height)?;
        app.create_vulkan_instance(app_name)?;
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

    const ENGINE_NAME: &'static[u8] = b"Silly Engine\0";

    fn create_vulkan_instance(&mut self, app_name: &str) -> Result<(), Box<dyn Error>>{
        let app_name: CString = CString::new(app_name)?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(1)
            .engine_name(CStr::from_bytes_with_nul(Self::ENGINE_NAME).unwrap())
            .engine_version(1)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .flags(InstanceCreateFlags::default())
            .enabled_extension_names(&[])
            .enabled_layer_names(&[]);

        let instance: ash::Instance = unsafe {
            self.entry.
                create_instance(&create_info, None)
                .expect("Could not create Vulkan instance")
        };

        self.vk_instance = Some(instance);

        println!("[✔️] Vulkan Instance successfully created.");

        Ok(())
    }
}