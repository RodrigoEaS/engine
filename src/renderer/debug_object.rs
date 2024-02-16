use ash::vk;

use crate::app::{populate_debug_messenger_create_info, VALIDATION};

pub struct DebugObjects {
    utils_loader: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugObjects {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

        if VALIDATION.is_enable == false {
            Self {
                utils_loader: debug_utils_loader,
                messenger: ash::vk::DebugUtilsMessengerEXT::null()
            }
        } else {
            let messenger_ci = populate_debug_messenger_create_info();

            let utils_messenger = unsafe {
                debug_utils_loader
                    .create_debug_utils_messenger(&messenger_ci, None)
                    .expect("Debug Utils Callback")
            };

            Self {
                utils_loader: debug_utils_loader,
                messenger: utils_messenger,
            }
        } 
    }

    pub(crate) fn destroy(&self) {
        unsafe {
            if VALIDATION.is_enable {
                self.utils_loader
                    .destroy_debug_utils_messenger(self.messenger, None);
            }
        }
    }
}