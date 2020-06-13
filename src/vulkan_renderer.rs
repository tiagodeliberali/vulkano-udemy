use std::error::Error;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::Device,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageUsage, SwapchainImage},
    instance::{ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, Version},
    pipeline::viewport::Viewport,
    pipeline::GraphicsPipeline,
    swapchain::{
        acquire_next_image, AcquireError, ColorSpace, FullscreenExclusive, PresentMode,
        SurfaceTransform, Swapchain, SwapchainCreationError,
    },
    sync::{now, FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;

use crate::error_utils::EngineError;

pub struct VulkanRenderer {
    pub instance: Arc<Instance>,
}

impl VulkanRenderer {
    pub fn init() -> Result<VulkanRenderer, EngineError> {
        let instance = VulkanRenderer::create_instance()?;

        let result = VulkanRenderer { instance };

        Ok(result)
    }

    fn create_instance() -> Result<Arc<Instance>, EngineError> {
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

        // So, lets check just to display whats inside core :)
        if !VulkanRenderer::check_instance_extension_support(&extensions) {
            eprintln!("Expected more instance extensions than available");
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
}