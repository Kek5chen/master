use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::mem::swap;
use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk};
use ash::vk::{InstanceCreateFlags, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};
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
        let pdevice = Self::select_gpu(&vk_instance, &surface_loader, &surface)?;
        let (ldevice, graphics_queue) = Self::create_logical_device_and_queues(&vk_instance, &pdevice, &surface_loader, &surface)?;
        let (swapchain_loader, swapchain) = Self::create_swapchain(&vk_instance, &pdevice, &ldevice, &surface_loader, &surface)?;
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
            let entry = ash::Entry::load()
                .map_err(|_| "Vulkan could not be loaded on this device. The library was not found.")?;

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

    fn find_suitable_queue_family(vk_instance: &ash::Instance,
                                  pdevice: &PhysicalDevice,
                                  surface_loader: &Surface,
                                  surface: &SurfaceKHR) -> Option<usize> {
        let queue_families = unsafe {
            vk_instance.get_physical_device_queue_family_properties(*pdevice)
        };

        let mut suitable = queue_families.iter().enumerate().find_map(|(idx, info)| unsafe {
            if info.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
                surface_loader
                    .get_physical_device_surface_support(*pdevice, idx as u32, *surface)
                    .unwrap()
            { Some(idx) } else { None }
        });

        suitable
    }

    fn is_gpu_suitable(pdevice: &PhysicalDevice,
                       vk_instance: &ash::Instance,
                       surface_loader: &Surface,
                       surface: &SurfaceKHR) -> bool{
        let mut suitable = Self::find_suitable_queue_family(vk_instance, pdevice, surface_loader, surface).is_some();

        suitable = suitable && unsafe {
            let device_extension_props = vk_instance
                .enumerate_device_extension_properties(*pdevice)
                .expect("Failed to get device extension properties");
            let extension_names: Vec<[c_char; 256]> = device_extension_props
                .iter()
                .map(|prop| prop.extension_name)
                .collect();

            extension_names.iter().any(|ext|
                CStr::from_ptr(ext.as_ptr()) == ash::extensions::khr::Swapchain::name())
        };

        suitable
    }

    fn select_gpu(vk_instance: &ash::Instance,
                  surface_loader: &Surface,
                  surface: &SurfaceKHR) -> Result<PhysicalDevice, Box<dyn Error>> {
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
            match Self::is_gpu_suitable(device, vk_instance, surface_loader, surface) {
                true => Some(device),
                false => None,
            }
        }).ok_or("Could not find a supported device to render onto.")?;

        unsafe {
            println!("[✔] Selected GPU: {:?}", CStr::from_ptr(device_props.unwrap().device_name.as_ptr()));
        }

        Ok(*pdevice)
    }
    fn create_logical_device_and_queues(vk_instance: &ash::Instance,
                                        pdevice: &PhysicalDevice,
                                        surface_loader: &Surface,
                                        surface: &SurfaceKHR)
            -> Result<(ash::Device, vk::Queue), Box<dyn Error>> {
        let queue_family_index = Self::find_suitable_queue_family(vk_instance, pdevice, surface_loader, surface)
            .expect("No suitable queue family found") as u32;
        let queue_priorities = [1.0_f32];

        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities);

        let physical_device_features = vk::PhysicalDeviceFeatures::builder();

        let swapchain_extension = CString::new("VK_KHR_swapchain").unwrap();
        let device_extensions = [swapchain_extension.as_ptr()];
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&device_extensions);

        let device: ash::Device = unsafe {
            vk_instance
                .create_device(*pdevice, &device_create_info, None)
                .expect("Failed to create logical device")
        };

        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        println!("[✔] Created Logical Device and Graphics Queue.");

        Ok((device, graphics_queue))
    }

    fn create_swapchain(vk_instance: &ash::Instance,
                        physical_device: &PhysicalDevice,
                        device: &ash::Device,
                        surface_loader: &Surface,
                        surface: &SurfaceKHR)
        -> Result<(Swapchain, SwapchainKHR), Box<dyn Error>> {
        let surface_capabilites = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(*physical_device, *surface)
                .expect("Failed to query for surface capabilities")
        };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(*physical_device, *surface)
                .expect("Failed to query for surface formats")
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(*physical_device, *surface)
                .expect("Failed to query for surface present modes")
        };

        let surface_format = formats.iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
            .cloned().unwrap();
        let present_mode = present_modes.iter()
            .find(|p| **p == vk::PresentModeKHR::MAILBOX)
            .cloned().unwrap();
        let extent = surface_capabilites.current_extent;

        let image_count = surface_capabilites.min_image_count + 1;

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilites.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain_loader = Swapchain::new(vk_instance, device);
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        println!("[✔] Swapchain created");

        Ok((swapchain_loader, swapchain))
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
