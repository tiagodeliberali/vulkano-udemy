use std::error;
use std::fmt;
use vulkano::{
    device::DeviceCreationError,
    instance::InstanceCreationError,
    swapchain::{CapabilitiesError, SwapchainCreationError},
    OomError,
};
use vulkano_win::CreationError;

#[derive(Debug)]
pub enum EngineError {
    VulkanInstanceCreationError(InstanceCreationError),
    VulkanDeviceCreationError(DeviceCreationError),
    VulkanCreationError(CreationError),
    VulkanValidationError(String),
    VulkanCapabilitiesError(CapabilitiesError),
    VulkanSwapchainCreationError(SwapchainCreationError),
    VulkanOomError(OomError),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for EngineError {}

impl From<InstanceCreationError> for EngineError {
    fn from(error: InstanceCreationError) -> Self {
        EngineError::VulkanInstanceCreationError(error)
    }
}

impl From<DeviceCreationError> for EngineError {
    fn from(error: DeviceCreationError) -> Self {
        EngineError::VulkanDeviceCreationError(error)
    }
}

impl From<CreationError> for EngineError {
    fn from(error: CreationError) -> Self {
        EngineError::VulkanCreationError(error)
    }
}

impl From<CapabilitiesError> for EngineError {
    fn from(error: CapabilitiesError) -> Self {
        EngineError::VulkanCapabilitiesError(error)
    }
}

impl From<SwapchainCreationError> for EngineError {
    fn from(error: SwapchainCreationError) -> Self {
        EngineError::VulkanSwapchainCreationError(error)
    }
}

impl From<OomError> for EngineError {
    fn from(error: OomError) -> Self {
        EngineError::VulkanOomError(error)
    }
}
