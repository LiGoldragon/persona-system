use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonaSystemError {
    UnsupportedBackend { backend: String },
    MissingEventSource { source: String },
}

impl Display for PersonaSystemError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedBackend { backend } => {
                write!(formatter, "unsupported system backend: {backend}")
            }
            Self::MissingEventSource { source } => {
                write!(formatter, "missing system event source: {source}")
            }
        }
    }
}

impl std::error::Error for PersonaSystemError {}
