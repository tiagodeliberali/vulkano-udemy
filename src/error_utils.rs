use std::error;
use std::fmt;
use vulkano::{device::DeviceCreationError, instance::InstanceCreationError};
use vulkano_win::CreationError;

#[derive(Debug)]
pub enum EngineError {
    VulkanInstanceCreationError(InstanceCreationError),
    VulkanDeviceCreationError(DeviceCreationError),
    VulkanCreationError(CreationError),
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
        EngineError::VulkanDeviceCreationError(error)
    }
}

impl From<CreationError> for EngineError {
    fn from(error: CreationError) -> Self {
        EngineError::VulkanCreationError(error)
    }
}
