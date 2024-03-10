use std::{ptr, rc::Rc};

use ash::vk;

use crate::core::device::GraphicDevice;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct SyncObjects {
    device: Rc<GraphicDevice>,
    
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,
    pub(crate) in_flight_fences: Vec<vk::Fence>
}

impl SyncObjects {
    pub fn new(device: Rc<GraphicDevice>) -> Self {
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];

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

                image_available_semaphores
                    .push(image_available_semaphore);

                render_finished_semaphores
                    .push(render_finished_semaphore);

                in_flight_fences.push(inflight_fence);
            }
        }

        Self {
            device,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences
        }
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.logical
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.logical
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.logical
                    .destroy_fence(self.in_flight_fences[i], None);
            }
        }
    }
}
