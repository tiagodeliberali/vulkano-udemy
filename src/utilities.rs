use vulkano::instance::QueueFamily;

pub struct QueueFamilyIndices<'a> {
    pub graphics_family: Option<QueueFamily<'a>>,
    pub presentation_family: Option<QueueFamily<'a>>,
}

impl<'a> QueueFamilyIndices<'a> {
    pub fn new() -> Self {
        QueueFamilyIndices {
            graphics_family: None,
            presentation_family: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.graphics_family.is_some() && self.presentation_family.is_some()
    }

    pub fn into_vec(self) -> Vec<QueueFamily<'a>> {
        if !self.is_valid() {
            return Vec::new();
        }

        let mut result = Vec::from([self.graphics_family.unwrap()]);

        // could be replaced by a set data structure...
        if result
            .iter()
            .find(|f| f.id() == self.presentation_family.unwrap().id())
            .is_none()
        {
            result.push(self.presentation_family.unwrap());
        }

        result
    }
}
