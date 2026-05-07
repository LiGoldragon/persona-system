use crate::{HarnessTarget, SystemWindowId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusState {
    Focused { window: SystemWindowId },
    Unfocused,
    Unknown,
}

impl FocusState {
    pub fn protects(&self, target: &HarnessTarget) -> bool {
        match self {
            Self::Focused { window } => target.owns_window(window),
            Self::Unfocused | Self::Unknown => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputBufferState {
    Empty,
    Occupied { preview: String },
    Unknown,
}

impl InputBufferState {
    pub fn accepts_injection(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonaSystemEvent {
    FocusChanged {
        state: FocusState,
    },
    InputBufferChanged {
        target: HarnessTarget,
        state: InputBufferState,
    },
}
