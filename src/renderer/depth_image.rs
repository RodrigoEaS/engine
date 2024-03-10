use std::rc::Rc;

use ash::vk;

use crate::texture::Texture;

use crate::core::device::GraphicDevice;

pub struct DepthImage {
    device: Rc<GraphicDevice>,
    
    pub(crate) image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub(crate) memory: vk::DeviceMemory,
}

impl DepthImage {
    pub fn new(
        instance: &ash::Instance,
        device: Rc<GraphicDevice>,
        swapchain_extent: &vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
    ) -> Self {
        let depth_format = Self::find_depth_format(instance, device.physical);
        let (depth_image, depth_image_memory) = Texture::create_image(
            &device.logical,
            swapchain_extent.width,
            swapchain_extent.height,
            1,
            msaa_samples,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device.memory_properties,
        );
        let depth_image_view = Texture::create_image_view(
            &device.logical,
            depth_image,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
            1,
        );

        Self {
            device,
            image: depth_image,
            image_view: depth_image_view,
            memory: depth_image_memory
        } 
    }

    pub(crate) fn find_depth_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> vk::Format {
        Self::find_supported_format(
            instance,
            physical_device,
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn find_supported_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        candidate_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for &format in candidate_formats.iter() {
            let format_properties =
                unsafe { instance.get_physical_device_format_properties(physical_device, format) };
            if tiling == vk::ImageTiling::LINEAR
                && format_properties.linear_tiling_features.contains(features)
            {
                return format.clone();
            } else if tiling == vk::ImageTiling::OPTIMAL
                && format_properties.optimal_tiling_features.contains(features)
            {
                return format.clone();
            }
        }

        panic!("Failed to find supported format!")
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