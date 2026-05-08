pub mod command;
pub mod error;
pub mod event;
pub mod niri;
pub mod target;

pub use command::{CommandLine, Input, ObserveFocus, SubscribeFocus};
pub use error::PersonaSystemError;
pub use event::{FocusObservation, FocusState, InputBufferState, PersonaSystemEvent};
pub use niri::{FocusTracker, NiriEvent, NiriFocusSource, NiriWindowSnapshot, NiriWindows};
pub use target::{HarnessTarget, NiriWindow, NiriWindowId, SystemTarget};
