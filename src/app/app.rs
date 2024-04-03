use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use ash::extensions::khr::Surface;
use ash::vk;
use ash::vk::{Handle, InstanceCreateFlags, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR};
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
        let (vk_instance, required_extensions) = Self::create_vulkan_instance(app_name, &window, &entry)?;
        // Create a debug messenger (optional)
        let (surface, surface_loader) = Self::create_vulkan_surface(&entry, &vk_instance, &window)?;
        let pdevice = Self::select_gpu(&vk_instance, &required_extensions)?;
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
        -> Result<(ash::Instance, Vec<*const c_char>), Box<dyn Error>> {
        let app_name: CString = CString::new(app_name)?;

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(1)
            .engine_name(CStr::from_bytes_with_nul(Self::ENGINE_NAME).unwrap())
            .engine_version(1)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let required_extensions =
            enumerate_required_extensions(window.raw_display_handle())
                .expect("Could not enumerate the required extensions!").to_vec();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .flags(InstanceCreateFlags::default())
            .enabled_extension_names(&required_extensions)
            .enabled_layer_names(&[]);

        let instance: ash::Instance = unsafe {
            entry.
                create_instance(&create_info, None)
                .expect("Could not create Vulkan instance!")
        };

        println!("[✔] Vulkan Instance successfully created.");

        Ok((instance, required_extensions))
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

    fn is_gpu_suitable(pdevice: &PhysicalDevice, vk_instance: &ash::Instance, required_extensions: &[*const c_char]) -> bool{
        let queue_families = unsafe {
            vk_instance.get_physical_device_queue_family_properties(*pdevice)
        };

        let mut suitable = queue_families.iter().enumerate().any(|(idx, info)| {
            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        });

        // // I accidentally tested Vulkan Extensions against Device Extensions here.. Ignore so far
        // suitable = suitable && unsafe {
        //     let device_extension_props = vk_instance
        //         .enumerate_device_extension_properties(*pdevice)
        //         .expect("Failed to get device extension properties");
        //     let extension_names: Vec<[c_char; 256]> = device_extension_props
        //         .iter()
        //         .map(|prop| prop.extension_name)
        //         .collect();
        //
        //     required_extensions
        //         .iter()
        //         .all(|ext| extension_names
        //             .iter()
        //             .any(|ext2| unsafe {
        //                 CStr::from_ptr(*ext) == CStr::from_ptr(ext2.as_ptr())
        //             }))
        // };

        suitable
    }

    fn select_gpu(vk_instance: &ash::Instance, required_extensions: &Vec<*const c_char>) -> Result<PhysicalDevice, Box<dyn Error>> {
        let pdevices = unsafe {
            vk_instance
                .enumerate_physical_devices()
                .expect("Could not enumerate physical devices!")
        };

        let mut device_props: Option<PhysicalDeviceProperties> = None;
        let pdevice = pdevices.iter().find_map(|device| {
            unsafe {
                device_props = Some(vk_instance.get_physical_device_properties(*device));
                println!("[...] Found GPU: {:?}",
                         CStr::from_ptr(device_props.unwrap().device_name.as_ptr()));
            }
            match Self::is_gpu_suitable(device, vk_instance, &required_extensions) {
                true => Some(device),
                false => None,
            }
        }).ok_or("Could not find a supported device to render onto.")?;

        unsafe {
            println!("[✔] Selected GPU: {:?}", CStr::from_ptr(device_props.unwrap().device_name.as_ptr()));
        }

        Ok(*pdevice)
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