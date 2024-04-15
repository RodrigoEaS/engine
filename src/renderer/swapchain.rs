use std::{ptr, rc::Rc};

use ash::vk;
use cgmath::Vector2;
use num::clamp;

use crate::core::{device::GraphicDevice, surface::Surface};

pub(crate) struct SwapChainSupportDetail {
    capabilities: vk::SurfaceCapabilitiesKHR,
    pub(crate) formats: Vec<vk::SurfaceFormatKHR>,
    pub(crate) present_modes: Vec<vk::PresentModeKHR>,
}

pub struct SwapChain {
    device: Rc<GraphicDevice>,
    
    pub(crate) loader: ash::extensions::khr::Swapchain,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) format: vk::Format,
    pub(crate) extent: vk::Extent2D,
    pub(crate) imageviews: Vec<vk::ImageView>,
    pub(crate) framebuffers: Vec<vk::Framebuffer>,
}

impl SwapChain {
    pub fn new(
        instance: &ash::Instance,
        device: Rc<GraphicDevice>,
        size: Vector2<u32>,
        surface: &Surface,
    ) -> Self {
        let swapchain_support = Self::query_swapchain_support(device.physical, surface);

        let surface_format = Self::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swapchain_extent(&swapchain_support.capabilities, size);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if device.family_indices.graphics_family != device.family_indices.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        device.family_indices.graphics_family.unwrap(),
                        device.family_indices.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
        };

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(instance, &device.logical);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };

        let swapchain_imageviews = Self::create_image_views(
            &device.logical,
            surface_format.format,
            &swapchain_images,
        );

        Self {
            device,

            loader: swapchain_loader,
            swapchain,
            images: swapchain_images,
            format: surface_format.format,
            extent,
            imageviews: swapchain_imageviews,
            framebuffers: Vec::new(),
        }
    }

    pub(crate) fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> SwapChainSupportDetail {
        unsafe {
            let capabilities = surface
                .loader
                .get_physical_device_surface_capabilities(physical_device, surface.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface
                .loader
                .get_physical_device_surface_formats(physical_device, surface.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface
                .loader
                .get_physical_device_surface_present_modes(physical_device, surface.surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        // check if list contains most widely used R8G8B8A8 format with nonlinear color space
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        // return the first format from the list
        return available_formats.first().unwrap().clone();
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn choose_swapchain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        size: Vector2<u32>,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            println!("Inner Window Size: ({}, {})", size.x, size.y);

            vk::Extent2D {
                width: clamp(
                    size.x,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    size.y,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub(crate) fn create_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in images.iter() {
            let imageview_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::ImageViewCreateFlags::empty(),
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                image,
            };

            let imageview = unsafe {
                device
                    .create_image_view(&imageview_create_info, None)
                    .expect("Failed to create Image View!")
            };
            swapchain_imageviews.push(imageview);
        }

        swapchain_imageviews
    }

    pub(crate) fn create_framebuffer(
        &mut self,
        render_pass: &vk::RenderPass,
        depth_image_view: vk::ImageView,
        color_image_view: vk::ImageView,
    ) {
        let mut framebuffers = vec![];

        for &image_view in self.imageviews.iter() {
            let attachments = [color_image_view, depth_image_view, image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: *render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
            };

            let framebuffer = unsafe {
                self.device.logical
                    .create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create Framebuffer!")
            };

            framebuffers.push(framebuffer);
        }

        self.framebuffers = framebuffers;
    }

    pub(crate) fn destroy_framebuffers(&self) {
        unsafe {
            for &framebuffer in self.framebuffers.iter() {
                self.device.logical
                    .destroy_framebuffer(framebuffer, None);
            }
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            for &image_view in self.imageviews.iter() {
                self.device.logical.destroy_image_view(image_view, None);
            }
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}