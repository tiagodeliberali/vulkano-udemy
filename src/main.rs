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

mod error_utils;
mod utilities;
mod vulkan_renderer;

use vulkan_renderer::VulkanRenderer;

fn init_window() -> EventLoop<()> {
    let events_loop = EventLoop::new();
    events_loop
}

fn main() {
    let events_loop = init_window();

    let render = match VulkanRenderer::init(&events_loop) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to create vulkano renderer: {}", err);
            process::exit(1);
        }
    };

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
