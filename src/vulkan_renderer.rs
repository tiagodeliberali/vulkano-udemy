use std::error::Error;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::{Device, Queue, QueuesIter},
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageUsage, SwapchainImage},
    instance::{
        ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, QueueFamily, Version,
    },
    pipeline::viewport::Viewport,
    pipeline::GraphicsPipeline,
    swapchain::{
        acquire_next_image, AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface,
        SurfaceTransform, Swapchain, SwapchainCreationError,
    },
    sync::{now, FlushError, GpuFuture},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::error_utils::EngineError;

pub struct VulkanRenderer {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub graphics_queue: Arc<Queue>,
}

impl VulkanRenderer {
    pub fn init(
        instance: Arc<Instance>,
        surface: &Arc<Surface<Window>>,
    ) -> Result<VulkanRenderer, EngineError> {
        let (physycal_device, queue_family) =
            VulkanRenderer::get_physical_device(&instance, &surface)?;
        let (device, mut queues) =
            VulkanRenderer::create_logical_device(physycal_device, queue_family)?;

        let graphics_queue = queues.next().unwrap();

        let result = VulkanRenderer {
            instance: instance,
            device,
            graphics_queue,
        };

        Ok(result)
    }

    pub fn create_instance() -> Result<Arc<Instance>, EngineError> {
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

        // This method returns the intersect between the ideal winit requirements and supported_by_core (vkEnumerateInstanceExtensionProperties).
        // There is no error handling, just the intersect result whatever it is
        // So, it doesn't make sense to validate if some requirement returned by it is missing on core
        let extensions = vulkano_win::required_extensions();

        // So, lets pretend we are worried and check if we have all necessary extensions :)
        if !VulkanRenderer::check_instance_extension_support(&extensions) {
            return Err(EngineError::VulkanValidationError(String::from(
                "Expected more instance extensions than available",
            )));
        }

        let instance = Instance::new(Some(&app_info), &extensions, None)?;

        Ok(instance)
    }

    fn check_instance_extension_support(extensions: &InstanceExtensions) -> bool {
        VulkanRenderer::display_supported_by_core();
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
    ) -> Result<(PhysicalDevice<'a>, QueueFamily<'a>), EngineError> {
        let mut physical_device_list = PhysicalDevice::enumerate(&instance);

        while let Some(device) = physical_device_list.next() {
            let valid_queue_family = device
                .queue_families()
                .find(|&q| VulkanRenderer::is_valid_queue_family(q, surface));

            if let Some(family) = valid_queue_family {
                return Ok((device, family));
            }
        }

        Err(EngineError::VulkanValidationError(String::from(
            "No valid physical device available",
        )))
    }

    fn is_valid_queue_family(q: QueueFamily, surface: &Arc<Surface<Window>>) -> bool {
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }

    fn create_logical_device(
        physical: PhysicalDevice,
        queue_family: QueueFamily,
    ) -> Result<(Arc<Device>, QueuesIter), EngineError> {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            ..vulkano::device::DeviceExtensions::none()
        };

        let (device, queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, 0.5)].iter().cloned(),
        )?;

        Ok((device, queues))
    }
}
