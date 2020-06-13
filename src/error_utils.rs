use std::error;
use std::fmt;
use vulkano::instance::InstanceCreationError;

#[derive(Debug)]
pub enum EngineError {
    VulkanInstanceCreationError(InstanceCreationError),
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
