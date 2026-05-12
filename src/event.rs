use nota_codec::NotaRecord;

use crate::{HarnessTarget, SystemTarget};

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusObservation {
    pub target: SystemTarget,
    pub focused: bool,
    pub generation: u64,
}

impl FocusObservation {
    pub fn new(target: SystemTarget, focused: bool, generation: u64) -> Self {
        Self {
            target,
            focused,
            generation,
        }
    }

    pub fn protects(&self, target: &HarnessTarget) -> bool {
        self.focused && target.owns_target(self.target)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemEvent {
    FocusChanged {
        observation: FocusObservation,
    },
}
