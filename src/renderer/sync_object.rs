use std::ptr;

use ash::vk;

use crate::device::GraphicDevice;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct SyncObjects {
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,
    pub(crate) in_flight_fences: Vec<vk::Fence>
}

impl SyncObjects {
    pub fn new(device: &GraphicDevice) -> Self {
        let mut sync_objects = Self {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device.logical
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let render_finished_semaphore = device.logical
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object!");
                let inflight_fence = device.logical
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object!");

                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.in_flight_fences.push(inflight_fence);
            }
        }

        sync_objects
    }

    pub(crate) fn destroy(&self, device: &GraphicDevice) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                device.logical
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                device.logical
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                device.logical
                    .destroy_fence(self.in_flight_fences[i], None);
            }
        }
    }
}
