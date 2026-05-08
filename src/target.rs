use nota_codec::{NotaRecord, NotaSum, NotaTransparent};

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NiriWindowId(u64);

impl NiriWindowId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NiriWindow {
    pub id: NiriWindowId,
}

impl NiriWindow {
    pub fn new(id: NiriWindowId) -> Self {
        Self { id }
    }
}

#[derive(NotaSum, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemTarget {
    NiriWindow(NiriWindow),
}

impl SystemTarget {
    pub fn niri_window(id: u64) -> Self {
        Self::NiriWindow(NiriWindow::new(NiriWindowId::new(id)))
    }

    pub fn niri_window_id(self) -> Option<NiriWindowId> {
        match self {
            Self::NiriWindow(window) => Some(window.id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessTarget {
    name: String,
    target: SystemTarget,
}

impl HarnessTarget {
    pub fn new(name: impl Into<String>, target: SystemTarget) -> Self {
        Self {
            name: name.into(),
            target,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owns_target(&self, target: SystemTarget) -> bool {
        self.target == target
    }
}
