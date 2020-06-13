use std::error;
use std::fmt;
use vulkano::{device::DeviceCreationError, instance::InstanceCreationError};

#[derive(Debug)]
pub enum EngineError {
    VulkanInstanceCreationError(InstanceCreationError),
    VulkanRawDeviceExtensions(DeviceCreationError),
    VulkanValidationError(String),
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
        EngineError::VulkanRawDeviceExtensions(error)
    }
}
