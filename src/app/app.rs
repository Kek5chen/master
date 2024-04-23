use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use ash::extensions::khr::{Surface, Swapchain};
use ash::prelude::VkResult;
use ash::{Device, vk};
use ash::vk::{Extent2D, Image, ImageView, ImageViewCreateInfo, InstanceCreateFlags, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};
use ash_window::enumerate_required_extensions;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

pub struct App {
    event_loop: Option<EventLoop<()>>,
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
        let (ldevice, queue_family_index, graphics_queue) = Self::create_logical_device_and_queues(&vk_instance, &pdevice, &surface_loader, &surface)?;
        let (swapchain_loader, swapchain) = Self::create_swapchain(&vk_instance, &pdevice, &ldevice, &surface_loader, &surface)?;
        let image_views = Self::create_image_views(&ldevice, &swapchain_loader, swapchain);
        let render_pass = Self::create_render_pass(&ldevice, vk::Format::R8G8B8A8_SRGB)?;
        let framebuffers = Self::create_framebuffers(&ldevice, &image_views, &render_pass, &window)?;

        // Command pool and command buffers setup
        let command_pool = Self::create_command_pool(&ldevice, queue_family_index)?;
        let command_buffers = Self::create_command_buffers(&ldevice, &command_pool, &framebuffers, &render_pass, &window);

        // Initialize synchronization primitives
        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) = Self::create_sync_objects(&ldevice)?;

        Ok(App {
            event_loop: Some(event_loop),
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

    const ENGINE_NAME: &'static [u8] = b"Silly Engine\0";

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
                       surface: &SurfaceKHR) -> bool {
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
                                        -> Result<(ash::Device, u32, vk::Queue), Box<dyn Error>> {
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

        Ok((device, queue_family_index, graphics_queue))
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
            .cloned().expect("Failed to find Surface Format in Physics Device");
        let present_mode = present_modes.iter()
            .find(|p| **p == vk::PresentModeKHR::FIFO)
            .cloned().expect("Failed to find FIFO Present Mode in Physical Device");
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

    fn create_image_view(device: &ash::Device, img: Image) -> ImageView {
        let create_info = ImageViewCreateInfo::builder()
            .image(img)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_SRGB)
            .components(vk::ComponentMapping::default())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let image_view = unsafe {
            device
                .create_image_view(&create_info, None)
                .expect("Failed to create image view.")
        };

        println!("[✔] Image View created");

        image_view
    }

    fn create_image_views(device: &ash::Device, swapchain: &Swapchain, swapchain_khr: SwapchainKHR)
                          -> Vec<ImageView> {
        let swp_images = unsafe {
            swapchain.get_swapchain_images(swapchain_khr)
                .expect("Couldn't get Swapchain Images.")
        };
        let image_views: Vec<ImageView> = swp_images.iter().map(
            |&img| Self::create_image_view(device, img)
        ).collect();

        assert_eq!(image_views.len(), 3, "[x] 3 Image Views could not be created");

        println!("[✔] {} Image View{} created",
                 image_views.len(),
                 if image_views.len() > 1 { "s" } else { "" });

        image_views
    }

    fn create_framebuffers(
        device: &ash::Device,
        image_views: &[vk::ImageView],
        render_pass: &vk::RenderPass,
        window: &winit::window::Window,
    ) -> VkResult<Vec<vk::Framebuffer>> {
        let mut framebuffers = vec![];

        for &image_view in image_views {
            let framebuffer_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass: *render_pass,
                attachment_count: 1,
                p_attachments: &image_view,
                width: window.inner_size().width,  // Define these according to your swapchain setup
                height: window.inner_size().height,
                layers: 1,
                ..Default::default()
            };

            let framebuffer = unsafe {
                device.create_framebuffer(&framebuffer_info, None)?
            };
            framebuffers.push(framebuffer);
        }

        println!("[✔] Created Framebuffers");
        Ok(framebuffers)
    }

    fn create_command_pool(
        device: &ash::Device,
        queue_family_index: u32,
    ) -> VkResult<vk::CommandPool> {
        let pool_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            queue_family_index,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };

        let command_pool = unsafe {
            device.create_command_pool(&pool_info, None)?
        };

        println!("[✔] Created Command Pool");

        Ok(command_pool)
    }

    fn create_command_buffers(
        device: &ash::Device,
        command_pool: &vk::CommandPool,
        framebuffers: &[vk::Framebuffer],
        render_pass: &vk::RenderPass,
        window: &winit::window::Window,
    ) -> VkResult<Vec<vk::CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: *command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: framebuffers.len() as u32,
            ..Default::default()
        };

        let command_buffers = unsafe {
            device.allocate_command_buffers(&command_buffer_allocate_info)?
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                ..Default::default()
            };

            unsafe {
                device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;
                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                    render_pass: *render_pass,
                    framebuffer: framebuffers[i],
                    render_area: vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: Extent2D { height: window.inner_size().height, width: window.inner_size().width },  // This should match the swapchain image extent
                    },
                    clear_value_count: 1,
                    p_clear_values: &vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: [0.0, 0.0, 0.0, 1.0],
                        },
                    },
                    ..Default::default()
                };
                device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                // Add drawing commands here
                device.cmd_end_render_pass(command_buffer);
                device.end_command_buffer(command_buffer)?;
            }
        }

        println!("[✔] Created Command Buffer");

        Ok(command_buffers)
    }
    fn create_sync_objects(device: &Device) -> VkResult<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>)> {
        const MAX_FRAMES_IN_FLIGHT: usize = 3;
        let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let semaphore_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,  // Start all fences in the signaled state to ensure that the first `wait_for_fences` call doesn't hang
            ..Default::default()
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = unsafe {
                device.create_semaphore(&semaphore_info, None)?
            };
            let render_finished_semaphore = unsafe {
                device.create_semaphore(&semaphore_info, None)?
            };
            let in_flight_fence = unsafe {
                device.create_fence(&fence_info, None)?
            };

            image_available_semaphores.push(image_available_semaphore);
            render_finished_semaphores.push(render_finished_semaphore);
            in_flight_fences.push(in_flight_fence);
        }

        println!("[✔] Created Sync Objects");

        Ok((image_available_semaphores, render_finished_semaphores, in_flight_fences))
    }

    fn create_render_pass(device: &ash::Device, surface_format: vk::Format) -> Result<vk::RenderPass, Box<dyn Error>> {
        let color_attachment = vk::AttachmentDescription {
            format: surface_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            ..Default::default()
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];

        let render_pass_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),
            ..Default::default()
        };

        let render_pass = unsafe {
            device.create_render_pass(&render_pass_info, None)?
        };

        println!("[✔] Created Render Pass");

        Ok(render_pass)
    }

    pub fn run(mut self) {
        let event_loop = self.event_loop.take().expect("Event loop should be present.");

        event_loop.run(|event, window_target| {
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    window_target.exit()
                }
                Event::DeviceEvent { event, device_id } => {
                    
                }
                _ => ()
            }
        }).unwrap()
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
