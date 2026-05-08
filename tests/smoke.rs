use persona_system::{
    FocusObservation, FocusTracker, HarnessTarget, InputBufferState, NiriEvent, NiriWindowId,
    NiriWindows, SystemTarget,
};

#[test]
fn focus_observation_protects_owned_window() {
    let target = SystemTarget::niri_window(42);
    let harness = HarnessTarget::new("responder", target);
    let observation = FocusObservation::new(target, true, 7);

    assert!(observation.protects(&harness));
}

#[test]
fn empty_input_buffer_accepts_injection() {
    assert!(InputBufferState::Empty.accepts_injection());
    assert!(!InputBufferState::Unknown.accepts_injection());
}

#[test]
fn niri_windows_observe_target_focus_state() {
    let windows = NiriWindows::from_json_slice(
        br#"[
          {
            "id": 10,
            "title": "Responder",
            "app_id": "org.wezfurlong.wezterm",
            "pid": 100,
            "workspace_id": 1,
            "is_focused": false,
            "is_floating": false,
            "is_urgent": false,
            "layout": {},
            "focus_timestamp": {"secs": 1, "nanos": 5}
          },
          {
            "id": 11,
            "title": "Initiator",
            "app_id": "org.wezfurlong.wezterm",
            "pid": 100,
            "workspace_id": 1,
            "is_focused": true,
            "is_floating": false,
            "is_urgent": false,
            "layout": {},
            "focus_timestamp": {"secs": 2, "nanos": 9}
          }
        ]"#,
    )
    .expect("windows decode");

    let observation = windows
        .observe(SystemTarget::niri_window(11), NiriWindowId::new(11))
        .expect("target observed");

    assert!(observation.focused);
    assert_eq!(observation.generation, 2_000_000_009);
}

#[test]
fn focus_tracker_filters_title_chatter_without_focus_change() {
    let target = SystemTarget::niri_window(10);
    let mut tracker = FocusTracker::new(target, NiriWindowId::new(10));
    tracker.accept(FocusObservation::new(target, false, 1_000_000_005));
    let event = NiriEvent::from_json_str(
        r#"{
          "WindowOpenedOrChanged": {
            "window": {
              "id": 10,
              "title": "Responder ⠙",
              "app_id": "org.wezfurlong.wezterm",
              "pid": 100,
              "workspace_id": 1,
              "is_focused": false,
              "is_floating": false,
              "is_urgent": false,
              "layout": {},
              "focus_timestamp": {"secs": 1, "nanos": 5}
            }
          }
        }"#,
    )
    .expect("event decodes");

    assert!(tracker.apply_event(&event).is_empty());
}

#[test]
fn focus_tracker_emits_when_target_focus_changes() {
    let target = SystemTarget::niri_window(10);
    let mut tracker = FocusTracker::new(target, NiriWindowId::new(10));
    tracker.accept(FocusObservation::new(target, false, 1_000_000_005));
    let event = NiriEvent::from_json_str(
        r#"{
          "WindowOpenedOrChanged": {
            "window": {
              "id": 10,
              "title": "Responder",
              "app_id": "org.wezfurlong.wezterm",
              "pid": 100,
              "workspace_id": 1,
              "is_focused": true,
              "is_floating": false,
              "is_urgent": false,
              "layout": {},
              "focus_timestamp": {"secs": 2, "nanos": 7}
            }
          }
        }"#,
    )
    .expect("event decodes");

    let observations = tracker.apply_event(&event);

    assert_eq!(observations.len(), 1);
    assert!(observations[0].focused);
    assert_eq!(observations[0].generation, 2_000_000_007);
}

#[test]
fn focus_tracker_uses_workspace_active_window_events() {
    let target = SystemTarget::niri_window(10);
    let windows = NiriWindows::from_json_slice(
        br#"[
          {
            "id": 10,
            "title": "Responder",
            "app_id": "org.wezfurlong.wezterm",
            "pid": 100,
            "workspace_id": 1,
            "is_focused": false,
            "is_floating": false,
            "is_urgent": false,
            "layout": {},
            "focus_timestamp": {"secs": 1, "nanos": 5}
          }
        ]"#,
    )
    .expect("windows decode");
    let mut tracker = FocusTracker::new(target, NiriWindowId::new(10));
    tracker.accept_window(
        windows
            .window(NiriWindowId::new(10))
            .expect("window exists"),
    );
    let event = NiriEvent::from_json_str(
        r#"{
          "WorkspaceActiveWindowChanged": {
            "workspace_id": 1,
            "active_window_id": 10
          }
        }"#,
    )
    .expect("event decodes");

    let observations = tracker.apply_event(&event);

    assert_eq!(observations.len(), 1);
    assert!(observations[0].focused);
}
