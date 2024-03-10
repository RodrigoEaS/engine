use std::rc::Rc;

use ash::vk;

use crate::{core::device::GraphicDevice, texture::Texture};

pub struct ColorImage {
    device: Rc<GraphicDevice>,
    
    pub(crate) image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub(crate) memory: vk::DeviceMemory,
}

impl ColorImage {
    pub fn new(
        device: Rc<GraphicDevice>,
        format: &vk::Format,
        extent: &vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let color_format = *format;

        let (color_image, color_image_memory) = Texture::create_image(
            &device.logical,
            extent.width,
            extent.height,
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
            device,
            image: color_image,
            image_view: color_image_view,
            memory: color_image_memory,
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.logical
                .destroy_image_view(self.image_view, None);
            self.device.logical
                .destroy_image(self.image, None);
            self.device.logical
                .free_memory(self.memory, None);
        }
    }
}
