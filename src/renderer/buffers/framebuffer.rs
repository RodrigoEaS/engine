use std::ptr;

use ash::vk;

use crate::{device::GraphicDevice, renderer::swapchain::SwapChain};

pub fn create_framebuffers(
    device: &GraphicDevice,
    render_pass: &vk::RenderPass,
    depth_image_view: vk::ImageView,
    color_image_view: vk::ImageView,
    swapchain: &SwapChain,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = vec![];

    for &image_view in swapchain.imageviews.iter() {
        let attachments = [color_image_view, depth_image_view, image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FramebufferCreateFlags::empty(),
            render_pass: *render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: swapchain.extent.width,
            height: swapchain.extent.height,
            layers: 1,
        };

        let framebuffer = unsafe {
            device.logical
                .create_framebuffer(&framebuffer_create_info, None)
                .expect("Failed to create Framebuffer!")
        };

        framebuffers.push(framebuffer);
    }

    framebuffers
}

pub(crate) fn destroy_framebuffers(device: &GraphicDevice, framebuffers: &Vec<vk::Framebuffer>) {
    unsafe {
        for &framebuffer in framebuffers.iter() {
            device.logical
                .destroy_framebuffer(framebuffer, None);
        }
    }
}