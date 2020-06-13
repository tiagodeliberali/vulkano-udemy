use std::process;
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{AutoCommandBufferBuilder, DynamicState},
    device::Device,
    framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass},
    image::{ImageUsage, SwapchainImage},
    instance::{ApplicationInfo, Instance, PhysicalDevice, Version},
    pipeline::viewport::Viewport,
    pipeline::GraphicsPipeline,
    swapchain::{
        acquire_next_image, AcquireError, ColorSpace, FullscreenExclusive, PresentMode,
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

mod error_utils;
mod vulkan_renderer;

use vulkan_renderer::VulkanRenderer;

fn init_window(instance: Arc<Instance>) -> EventLoop<()> {
    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    events_loop
}

fn main() {
    let render = match VulkanRenderer::init() {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to create vulkan: {}", err);
            process::exit(1);
        }
    };

    let events_loop = init_window(render.instance);
    events_loop.run(move |event, _, control_flow| {
        // *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
                println!("The close button was pressed; stopping");
            }
            _ => (),
        }
    });
}
