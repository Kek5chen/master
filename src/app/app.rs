use std::error::Error;
use std::ffi::{CStr, CString};
use ash::extensions::khr::Surface;
use ash::vk;
use ash::vk::{InstanceCreateFlags, SurfaceKHR};
use ash_window::enumerate_required_extensions;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct App {
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    entry: ash::Entry,
    vk_instance: ash::Instance,
    surface: SurfaceKHR,
    surface_loader: Surface,
}

impl App {
    pub fn new(app_name: &str, window_width: u32, window_height: u32) -> Result<Self, Box<dyn Error>> {
        let entry = Self::initialize_vulkan()?;

        let (event_loop, window) = Self::create_window(app_name, window_width, window_height)?;
        let vk_instance = Self::create_vulkan_instance(app_name, &window, &entry)?;
        // Create a debug messenger (optional)
        let (surface, surface_loader) = Self::create_vulkan_surface(&entry, &vk_instance, &window)?;
        // Select a physical device
        // Create a logical device and queues
        // Create a swap chain
        // Create image views
        // Setup framebuffers, command pools, and command buffers
        // Initialize synchronization primitives

        Ok(App {
            event_loop,
            window,
            entry,
            vk_instance,
            surface,
            surface_loader,
        })
    }

    fn initialize_vulkan() -> Result<ash::Entry, Box<dyn Error>> {
        unsafe {
            let entry = ash::Entry::load()?;

            Ok(entry)
        }
    }

    const ENGINE_NAME: &'static[u8] = b"Silly Engine\0";

    fn create_vulkan_instance(app_name: &str, window: &winit::window::Window, entry: &ash::Entry)
        -> Result<ash::Instance, Box<dyn Error>> {
        let app_name: CString = CString::new(app_name)?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(1)
            .engine_name(CStr::from_bytes_with_nul(Self::ENGINE_NAME).unwrap())
            .engine_version(1)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let required_extensions =
            enumerate_required_extensions(window.raw_display_handle())
                .expect("Could not enumerate the required extensions!");

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .flags(InstanceCreateFlags::default())
            .enabled_extension_names(required_extensions)
            .enabled_layer_names(&[]);

        let instance: ash::Instance = unsafe {
            entry.
                create_instance(&create_info, None)
                .expect("Could not create Vulkan instance!")
        };

        println!("[✔] Vulkan Instance successfully created.");

        Ok(instance)
    }

    fn create_window(title: &str, width: u32, height: u32) -> Result<(EventLoop<()>, winit::window::Window), Box<dyn Error>> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                width,
                height,
            ))
            .build(&event_loop)
            .unwrap();

        println!("[✔] Created App Window.");

        Ok((event_loop, window))
    }

    fn create_vulkan_surface(entry: &ash::Entry,
                             vk_instance: &ash::Instance,
                             window: &winit::window::Window)
        -> Result<(SurfaceKHR, Surface), Box<dyn Error>> {
        let surface = unsafe {
            ash_window::create_surface(entry,
                                       vk_instance,
                                       window.raw_display_handle(),
                                       window.raw_window_handle(),
                                       None)?
        };

        let surface_loader = Surface::new(entry, vk_instance);

        println!("[✔] Created Vulkan Surface.");

        Ok((surface, surface_loader))
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
            println!("[✔] Dropped Vulkan Surface.");
            self.window.set_visible(false);
            println!("[✔] Hid window for later drop.")
        }
    }
}