use std::fs;
use std::path::{Path, PathBuf};

use persona_system::{
    ApplyNiriEvent, FocusObservation, FocusTracker, NiriEvent, NiriFocus, NiriWindowId,
    SystemTarget,
};

struct SourceFile {
    path: PathBuf,
    content: String,
}

impl SourceFile {
    fn read(path: PathBuf) -> Self {
        let content = fs::read_to_string(&path).expect("source file is readable");
        Self { path, content }
    }

    fn is_guard_source(&self) -> bool {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "actor_runtime_truth.rs")
    }

    fn contains(&self, fragment: &str) -> bool {
        self.content.contains(fragment)
    }
}

struct SourceTree {
    root: PathBuf,
}

impl SourceTree {
    fn new() -> Self {
        Self {
            root: PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        }
    }

    fn guarded_files(&self) -> Vec<SourceFile> {
        let mut files = vec![self.root.join("Cargo.toml"), self.root.join("Cargo.lock")];
        files.extend(self.source_files());
        files.extend(self.test_files());
        files.into_iter().map(SourceFile::read).collect()
    }

    fn source_files(&self) -> Vec<PathBuf> {
        let src = self.root.join("src");
        fs::read_dir(src)
            .expect("source directory is readable")
            .map(|entry| entry.expect("source entry is readable").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect()
    }

    fn test_files(&self) -> Vec<PathBuf> {
        let tests = self.root.join("tests");
        fs::read_dir(tests)
            .expect("tests directory is readable")
            .map(|entry| entry.expect("test entry is readable").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .collect()
    }
}

#[test]
fn niri_focus_cannot_use_non_kameo_runtime() {
    let forbidden_fragments = [
        "ractor =",
        "name = \"ractor\"",
        "use ractor",
        "ractor::",
        "RpcReplyPort",
        "ActorProcessingErr",
    ];

    let mut violations = Vec::new();
    for file in SourceTree::new().guarded_files() {
        if file.is_guard_source() {
            continue;
        }
        for fragment in forbidden_fragments {
            if file.contains(fragment) {
                violations.push(format!("{} contains {fragment}", file.path.display()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "non-kameo niri actor runtime violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn niri_subscription_cannot_bypass_focus_actor_mailbox() {
    let source = SourceFile::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("niri.rs"),
    );

    assert!(source.contains("NiriFocus::start"));
    assert!(source.contains("focus.ask(ApplyNiriEvent { event }).send()"));
    assert!(!source.contains("tracker.apply_event(&event)"));
}

#[test]
fn niri_focus_cannot_be_empty_marker() {
    let source = SourceFile::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("niri_focus.rs"),
    );

    assert!(source.contains("pub struct NiriFocus {"));
    assert!(source.contains("tracker: FocusTracker,"));
    assert!(source.contains("applied_event_count: u64,"));
    assert!(source.contains("emitted_observation_count: u64,"));
}

#[tokio::test]
async fn niri_focus_cannot_emit_target_chatter_without_focus_change() {
    let target = SystemTarget::niri_window(10);
    let mut tracker = FocusTracker::new(target, NiriWindowId::new(10));
    tracker.accept(FocusObservation::new(target, false, 1_000_000_005));
    let focus = NiriFocus::start(NiriFocus::from_tracker(tracker)).await;

    let observations = focus
        .ask(ApplyNiriEvent {
            event: window_event(false, 1, 5, "Responder spinner"),
        })
        .await
        .expect("actor applies niri event");

    assert!(observations.is_empty());
    NiriFocus::stop(focus).await.expect("actor stops");
}

#[tokio::test]
async fn niri_focus_cannot_forget_previous_window_observation_between_messages() {
    let target = SystemTarget::niri_window(10);
    let focus = NiriFocus::start(NiriFocus::new(target, NiriWindowId::new(10))).await;

    let first = focus
        .ask(ApplyNiriEvent {
            event: window_event(false, 1, 5, "Responder"),
        })
        .await
        .expect("actor applies first event");
    let repeated = focus
        .ask(ApplyNiriEvent {
            event: window_event(false, 1, 5, "Responder renamed"),
        })
        .await
        .expect("actor applies repeated event");

    assert_eq!(
        first,
        vec![FocusObservation::new(target, false, 1_000_000_005)]
    );
    assert!(repeated.is_empty());
    NiriFocus::stop(focus).await.expect("actor stops");
}

fn window_event(focused: bool, seconds: u64, nanos: u32, title: &str) -> NiriEvent {
    NiriEvent::from_json_str(&format!(
        r#"{{
          "WindowOpenedOrChanged": {{
            "window": {{
              "id": 10,
              "title": "{title}",
              "app_id": "org.wezfurlong.wezterm",
              "pid": 100,
              "workspace_id": 1,
              "is_focused": {focused},
              "is_floating": false,
              "is_urgent": false,
              "layout": {{}},
              "focus_timestamp": {{"secs": {seconds}, "nanos": {nanos}}}
            }}
          }}
        }}"#
    ))
    .expect("event decodes")
}
