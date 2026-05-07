use persona_system::{FocusState, HarnessTarget, InputBufferState, SystemWindowId};

#[test]
fn focus_state_protects_owned_window() {
    let target = HarnessTarget::new("responder", SystemWindowId::new("window-1"));
    let state = FocusState::Focused {
        window: SystemWindowId::new("window-1"),
    };

    assert!(state.protects(&target));
}

#[test]
fn empty_input_buffer_accepts_injection() {
    assert!(InputBufferState::Empty.accepts_injection());
    assert!(!InputBufferState::Unknown.accepts_injection());
}
