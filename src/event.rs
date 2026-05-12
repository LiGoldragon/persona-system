use crate::{HarnessTarget, SystemTarget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusState {
    Focused { target: SystemTarget },
    Unfocused,
    Unknown,
}

impl FocusState {
    pub fn protects(&self, target: &HarnessTarget) -> bool {
        match self {
            Self::Focused { target: focused } => target.owns_target(*focused),
            Self::Unfocused | Self::Unknown => false,
        }
    }
}
