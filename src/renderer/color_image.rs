use ash::vk;

use crate::{device::GraphicDevice, texture::Texture};

use super::swapchain::SwapChain;

pub struct ColorImage {
    pub(crate) image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub(crate) memory: vk::DeviceMemory,
}

impl ColorImage {
    pub fn new(
        device: &GraphicDevice,
        swapchain: &SwapChain,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let color_format = swapchain.format;

        let (color_image, color_image_memory) = Texture::create_image(
            &device.logical,
            swapchain.extent.width,
            swapchain.extent.height,
            1,
            msaa_samples,
            color_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device.memory_properties,
        );

        let color_image_view = Texture::create_image_view(
            &device.logical,
            color_image,
            color_format,
            vk::ImageAspectFlags::COLOR,
            1,
        );

        Self {
            image: color_image,
            image_view: color_image_view,
            memory: color_image_memory,
        }
    }

    pub(crate) fn destroy(&self, device: &GraphicDevice) {
        unsafe {
            device.logical
                .destroy_image_view(self.image_view, None);
            device.logical
                .destroy_image(self.image, None);
            device.logical
                .free_memory(self.memory, None);
        }
    }
}
