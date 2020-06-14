use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::{Device, DeviceExtensions, Queue, QueuesIter},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageUsage, SwapchainImage},
    instance::{
        debug::{DebugCallback, MessageSeverity, MessageType},
        layers_list, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, QueueFamily,
        Version,
    },
    pipeline::viewport::Viewport,
    pipeline::GraphicsPipeline,
    swapchain::{
        acquire_next_image, AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface,
        SurfaceTransform, Swapchain, SwapchainCreationError,
    },
    sync::{now, FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{error_utils::EngineError, utilities::QueueFamilyIndices};

const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_KHRONOS_validation"];

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

#[allow(unused)]
pub struct VulkanRenderer {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,

    // must live to keep working
    surface: Arc<Surface<Window>>,
    debug_callback: Option<DebugCallback>,
}

impl VulkanRenderer {
    pub fn init(event_loop: &EventLoop<()>) -> Result<Self, EngineError> {
        let instance = Self::create_instance()?;
        let debug_callback = Self::setup_debug_callback(&instance);
        let surface = Self::create_surface(instance.clone(), &event_loop)?;
        let physycal_device = Self::get_physical_device(&instance, &surface)?;
        let (device, mut queues) = Self::create_logical_device(physycal_device, &surface)?;

        let graphics_queue = queues.next().unwrap();

        let result = VulkanRenderer {
            instance: instance,
            device,
            graphics_queue,
            surface,
            debug_callback,
        };

        Ok(result)
    }

    fn create_instance() -> Result<Arc<Instance>, EngineError> {
        if ENABLE_VALIDATION_LAYERS {
            if !Self::check_validation_layer_support() {
                println!("Validation layers requested, but not available!\n\n");
            } else {
                println!("Validation layers WORKING!!!\n\n");
            }
        }

        let app_info = ApplicationInfo {
            application_name: Some("Udemy tutorial".into()),
            application_version: Some(Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
            engine_name: Some("No Engine".into()),
            engine_version: Some(Version {
                major: 1,
                minor: 0,
                patch: 0,
            }),
        };

        let extensions = Self::get_required_instance_extensions();

        if !Self::check_instance_extension_support(&extensions) {
            return Err(EngineError::VulkanValidationError(String::from(
                "Expected more instance extensions than available",
            )));
        }

        let instance = if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            Instance::new(
                Some(&app_info),
                &extensions,
                VALIDATION_LAYERS.iter().cloned(),
            )?
        } else {
            Instance::new(Some(&app_info), &extensions, None)?
        };

        Ok(instance)
    }

    fn create_surface(
        instance: Arc<Instance>,
        events_loop: &EventLoop<()>,
    ) -> Result<Arc<Surface<Window>>, EngineError> {
        let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance)?;

        Ok(surface)
    }

    fn get_required_instance_extensions() -> InstanceExtensions {
        // This method returns the intersect between the ideal winit requirements and supported_by_core (vkEnumerateInstanceExtensionProperties).
        // There is no error handling, just the intersect result whatever it is
        // So, it doesn't make sense to validate if some requirement returned by it is missing on core
        let mut extensions = vulkano_win::required_extensions();

        // here is a extension request that will be validated by our check_instance_extension_support
        if ENABLE_VALIDATION_LAYERS {
            extensions.ext_debug_utils = true;
        }

        extensions
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = layers_list()
            .unwrap()
            .map(|l| l.name().to_owned())
            .collect();

        println!("Available validation layers:");
        for l in &layers {
            println!("{}", l);
        }

        VALIDATION_LAYERS
            .iter()
            .all(|layer_name| layers.contains(&layer_name.to_string()))
    }

    fn setup_debug_callback(instance: &Arc<Instance>) -> Option<DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }

        let mut msg_severity = MessageSeverity::errors_and_warnings();
        msg_severity.information = true;
        msg_severity.verbose = true;

        let msg_type = MessageType::all();

        DebugCallback::new(&instance, msg_severity, msg_type, |msg| {
            println!("validation layer: {:?}", msg.description);
        })
        .ok()
    }

    fn check_instance_extension_support(extensions: &InstanceExtensions) -> bool {
        Self::display_supported_by_core();
        println!("Requested extensions: \n {:#?}", &extensions);

        let value = InstanceExtensions::supported_by_core()
            .expect("Could not get core instance extensions from Vulkan");

        value.intersection(&extensions).eq(&extensions)
    }

    fn display_supported_by_core() {
        println!("Vulkan instance extensions supported (vkEnumerateInstanceExtensionProperties):");
        for f in InstanceExtensions::supported_by_core().iter() {
            println!("{:#?}", f);
        }
    }

    fn get_physical_device<'a>(
        instance: &'a Arc<Instance>,
        surface: &Arc<Surface<Window>>,
    ) -> Result<PhysicalDevice<'a>, EngineError> {
        let mut physical_device_list = PhysicalDevice::enumerate(&instance);

        while let Some(device) = physical_device_list.next() {
            if Self::check_device_suitable(&device, surface) {
                return Ok(device);
            }
        }

        Err(EngineError::VulkanValidationError(String::from(
            "No valid physical device available",
        )))
    }

    fn check_device_suitable(
        physical_device: &PhysicalDevice,
        surface: &Arc<Surface<Window>>,
    ) -> bool {
        let queue_families = Self::get_queue_families(physical_device, surface);
        let extensions = Self::get_required_device_extensions();

        queue_families.is_valid()
            && Self::check_device_extension_support(&physical_device, &extensions)
    }

    fn get_queue_families<'a>(
        physical_device: &PhysicalDevice<'a>,
        surface: &Arc<Surface<Window>>,
    ) -> QueueFamilyIndices<'a> {
        let mut queue_family_indices = QueueFamilyIndices::new();

        if let Some(family) = physical_device
            .queue_families()
            .find(|&q| q.supports_graphics())
        {
            queue_family_indices.graphics_family = Some(family);
        }

        if let Some(family) = physical_device
            .queue_families()
            .find(|&q| surface.is_supported(q).unwrap_or(false))
        {
            queue_family_indices.presentation_family = Some(family);
        }

        queue_family_indices
    }

    fn get_required_device_extensions() -> DeviceExtensions {
        DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        }
    }

    fn check_device_extension_support(
        device: &PhysicalDevice,
        extensions: &DeviceExtensions,
    ) -> bool {
        let supported_extensions = DeviceExtensions::supported_by_device(*device);
        println!(
            "Supported Device extensions:\n\n{:#?}",
            supported_extensions
        );

        supported_extensions.intersection(extensions).eq(extensions)
    }

    fn create_logical_device(
        physical: PhysicalDevice,
        surface: &Arc<Surface<Window>>,
    ) -> Result<(Arc<Device>, QueuesIter), EngineError> {
        let device_ext = Self::get_required_device_extensions();

        let families: Vec<(QueueFamily, f32)> = Self::get_queue_families(&physical, surface)
            .into_vec()
            .into_iter()
            .map(|x| (x, 0.5))
            .collect();

        let (device, queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            families,
        )?;

        Ok((device, queues))
    }
}
