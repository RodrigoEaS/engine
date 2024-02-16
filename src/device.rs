use crate::renderer::{surface::Surface, swapchain::SwapChain, vk_to_string};
use ash::vk;
use std::{collections::HashSet, ptr};

struct DeviceExtension {
    names: [&'static str; 1],
}

const DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
    names: ["VK_KHR_swapchain"],
};

pub(crate) struct QueueFamilyIndices {
    pub(super) graphics_family: Option<u32>,
    pub(super) present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct GraphicDevice {
    pub(crate) physical: vk::PhysicalDevice,
    pub(crate) memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub(crate) logical: ash::Device,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) family_indices: QueueFamilyIndices,
}

impl GraphicDevice {
    pub fn new(instance: &ash::Instance, surface: &Surface) -> Self {
        

        let physical_device = Self::pick_physical_device(instance, &surface);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let (logical_device, family_indices) =
            Self::create_logical_device(&instance, physical_device, surface);
        let graphics_queue =
            unsafe { logical_device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        Self {
            physical: physical_device,
            memory_properties: physical_device_memory_properties,
            logical: logical_device,
            graphics_queue,
            present_queue,
            family_indices,
        }
    }

    fn pick_physical_device(
        instance: &ash::Instance,
        surface: &Surface
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Physical Devices!")
        };

        let result = physical_devices.iter().find(|physical_device| {
            Self::is_physical_device_suitable(
                instance,
                **physical_device,
                surface
            )
        });

        match result {
            Some(p_physical_device) => *p_physical_device,
            None => panic!("Failed to find a suitable GPU!"),
        }
    }

    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_stuff: &Surface
    ) -> bool {
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
        let indices = Self::find_queue_family(instance, physical_device, surface_stuff);

        let is_queue_family_supported = indices.is_complete();
        let is_device_extension_supported = Self::check_device_extension_support(
            instance,
            physical_device
        );
        let is_swapchain_supported = if is_device_extension_supported {
            let swapchain_support =
                SwapChain::query_swapchain_support(physical_device, surface_stuff);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };
        let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;

        return is_queue_family_supported
            && is_device_extension_supported
            && is_swapchain_supported
            && is_support_sampler_anisotropy;
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> (ash::Device, QueueFamilyIndices) {
        let indices = Self::find_queue_family(instance, physical_device, surface);

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for &queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            ..Default::default() // default just enable no feature.
        };

        let enable_extension_names = [
            ash::extensions::khr::Swapchain::name().as_ptr(), // currently just enable the Swapchain extension.
        ];

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            enabled_extension_count: enable_extension_names.len() as u32,
            pp_enabled_extension_names: enable_extension_names.as_ptr(),
            p_enabled_features: &physical_device_features,
            ..Default::default()
        };

        let device: ash::Device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .expect("Failed to create logical device!")
        };

        (device, indices)
    }

    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface.loader.get_physical_device_surface_support(
                    physical_device,
                    index as u32,
                    **surface,
                )
            }
            .unwrap();

            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties.")
        };

        let mut available_extension_names = vec![];

        for extension in available_extensions.iter() {
            let extension_name = vk_to_string(&extension.extension_name);

            available_extension_names.push(extension_name);
        }

        let mut required_extensions = HashSet::new();
        for extension in DEVICE_EXTENSIONS.names.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        return required_extensions.is_empty();
    }

    pub(crate) fn wait_device_idle(&self) {
        unsafe {
            self.logical
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        }
    } 
    
    pub(crate) fn destroy(&self) {
        unsafe {
            self.logical.destroy_device(None);
        }
    }
}
