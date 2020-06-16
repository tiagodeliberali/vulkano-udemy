use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::{Device, DeviceExtensions, Queue, QueuesIter},
    format::Format,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageLayout, ImageUsage, SwapchainImage},
    instance::{
        debug::{DebugCallback, MessageSeverity, MessageType},
        layers_list, ApplicationInfo, Instance, InstanceExtensions, PhysicalDevice, QueueFamily,
        Version,
    },
    pipeline::{
        blend::{AttachmentBlend, BlendFactor},
        viewport::Viewport,
        GraphicsPipeline,
    },
    swapchain::{
        acquire_next_image, AcquireError, ColorSpace, FullscreenExclusive, PresentMode,
        SupportedPresentModes, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
    },
    sync::{now, FlushError, GpuFuture, SharingMode},
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
        let (swapchain, images) = Self::create_swapchain(
            physycal_device,
            surface.clone(),
            device.clone(),
            &mut queues,
        )?;

        Self::create_graphic_pipeline(device.clone(), swapchain.clone())?;

        let result = VulkanRenderer {
            instance: instance,
            device,
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

        let msg_severity = MessageSeverity::errors_and_warnings();
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

    fn create_swapchain(
        physical: PhysicalDevice,
        surface: Arc<Surface<Window>>,
        device: Arc<Device>,
        queues: &mut QueuesIter,
    ) -> Result<(Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>), EngineError> {
        let (swapchain, images) = {
            let surface_capabilities = surface.capabilities(physical)?;

            let (surface_format, color_space) =
                Self::choose_best_surface_format(surface_capabilities.supported_formats);

            let presentation_mode =
                Self::choose_best_presentation_mode(surface_capabilities.present_modes);

            let mut image_count: u32 = surface_capabilities.min_image_count + 1;
            if let Some(max_image_count) = surface_capabilities.max_image_count {
                image_count = std::cmp::min(image_count, max_image_count)
            }

            // Opaque (VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR) is the first element, if available, in the iter() implementation
            let alpha = surface_capabilities
                .supported_composite_alpha
                .iter()
                .next()
                .unwrap();

            // VkExtent2D is created inside swapchain creation and uses dimensions values to be built
            let mut dimensions: [u32; 2] = surface.window().inner_size().into();
            dimensions[0] = std::cmp::max(
                surface_capabilities.min_image_extent[0],
                std::cmp::min(surface_capabilities.max_image_extent[0], dimensions[0]),
            );
            dimensions[1] = std::cmp::max(
                surface_capabilities.min_image_extent[1],
                std::cmp::min(surface_capabilities.max_image_extent[1], dimensions[1]),
            );

            let sharing_mode: SharingMode = {
                let mut queue_list: Vec<u32> = Vec::new();
                while let Some(q) = queues.next() {
                    queue_list.push(q.id_within_family());
                }

                if queue_list.len() == 1 {
                    SharingMode::Exclusive
                } else {
                    SharingMode::Concurrent(queue_list)
                }
            };

            Swapchain::new(
                device.clone(),
                surface.clone(),
                image_count,
                surface_format,
                dimensions,
                1,
                ImageUsage::color_attachment(),
                sharing_mode,
                SurfaceTransform::Identity,
                alpha,
                presentation_mode,
                FullscreenExclusive::Default,
                true,
                color_space,
            )?
        };

        Ok((swapchain, images))
    }

    fn choose_best_surface_format(
        avalilable_formats: Vec<(Format, ColorSpace)>,
    ) -> (Format, ColorSpace) {
        let best_format = avalilable_formats.clone().into_iter().find(|f| {
            (f.0 == Format::R8G8B8A8Unorm || f.0 == Format::B8G8R8A8Unorm)
                && f.1 == ColorSpace::SrgbNonLinear
        });

        if let Some(format) = best_format {
            return format;
        }

        return avalilable_formats[0];
    }

    fn choose_best_presentation_mode(supported_modes: SupportedPresentModes) -> PresentMode {
        if supported_modes.mailbox {
            return PresentMode::Mailbox;
        }

        return PresentMode::Fifo;
    }

    fn create_graphic_pipeline(
        device: Arc<Device>,
        swapchain: Arc<Swapchain<Window>>,
    ) -> Result<(), EngineError> {
        #[derive(Default, Debug, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        vulkano::impl_vertex!(Vertex, position);

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex {
                    position: [-0.5, -0.25],
                },
                Vertex {
                    position: [0.0, 0.5],
                },
                Vertex {
                    position: [0.25, -0.1],
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap();

        mod vertex_shader {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
                #version 450
    
                layout(location = 0) in vec2 position;

                layout(location = 0) out vec3 fragColour;

                // Triangle vertex colours
                vec3 colours[3] = vec3[](
                    vec3(1.0, 0.0, 0.0),
                    vec3(0.0, 1.0, 0.0),
                    vec3(0.0, 0.0, 1.0)
                );
    
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                    fragColour = colours[gl_VertexIndex];
                }"
            }
        }

        mod fragment_shader {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
                #version 450

                layout(location = 0) in vec3 fragColour;
    
                layout(location = 0) out vec4 f_color;
    
                void main() {
                    f_color = vec4(fragColour, 1.0);
                }"
            }
        }

        let vertex_shader = vertex_shader::Shader::load(device.clone())?;
        let fragment_shader = fragment_shader::Shader::load(device.clone())?;

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass!(device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                        initial_layout: ImageLayout::Undefined,
                        final_layout: ImageLayout::PresentSrc,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );

        let mut blend_info = AttachmentBlend::alpha_blending();
        blend_info.alpha_source = BlendFactor::One;
        blend_info.alpha_destination = BlendFactor::Zero;

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                // Defines what kind of vertex input is expected.
                .vertex_input_single_buffer::<Vertex>()
                // The vertex shader.
                .vertex_shader(vertex_shader.main_entry_point(), ())
                // VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO
                .triangle_list()
                // Defines the viewport (explanations below).
                .viewports_dynamic_scissors_irrelevant(1)
                // rasterizerCreateInfo.frontFace = VK_FRONT_FACE_CLOCKWISE
                .front_face_clockwise()
                // rasterizerCreateInfo.cullMode = VK_CULL_MODE_BACK_BIT
                .cull_mode_back()
                // POLYGON_MODE_FILL - lets test what other values does to the final render :)
                .polygon_mode_fill()
                // VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO
                .blend_collective(blend_info)
                // The fragment shader.
                .fragment_shader(fragment_shader.main_entry_point(), ())
                // STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO
                // multisamplingCreateInfo.rasterizationSamples is tied to render_pass (VkAttachmentDescription).
                // There is a TODO there. Default is 1 (VK_SAMPLE_COUNT_1_BIT)
                .sample_shading_disabled()
                // This graphics pipeline object concerns the first pass of the render pass.
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                // Now that everything is specified, we call `build`.
                .build(device.clone())
                .unwrap(),
        );

        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        Ok(())
    }
}
