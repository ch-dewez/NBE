use vulkanalia::{Version, prelude::v1_4::*};

pub(crate) const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

pub(crate) const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

pub(crate) const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub(crate) const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
