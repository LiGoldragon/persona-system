#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemWindowId {
    value: String,
}

impl SystemWindowId {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessTarget {
    name: String,
    window: SystemWindowId,
}

impl HarnessTarget {
    pub fn new(name: impl Into<String>, window: SystemWindowId) -> Self {
        Self {
            name: name.into(),
            window,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owns_window(&self, window: &SystemWindowId) -> bool {
        self.window == *window
    }
}
