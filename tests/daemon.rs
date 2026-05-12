use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;

use persona_system::{SystemCommandLine, SystemDaemon, SystemFrameCodec};
use signal_core::{FrameBody, Reply, Request};
use signal_persona_system::{
    FocusSubscription, Frame as SystemFrame, SystemBackend, SystemEvent, SystemHealth,
    SystemOperationKind, SystemReadiness, SystemRequest, SystemRequestUnimplemented, SystemStatus,
    SystemStatusQuery, SystemTarget, SystemUnimplementedReason,
};

struct SocketFixture {
    root: PathBuf,
    socket: PathBuf,
}

impl SocketFixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "persona-system-{name}-{}-{}",
            std::process::id(),
            unique_nanos()
        ));
        let socket = root.join("system.sock");
        std::fs::create_dir_all(&root).expect("fixture root created");
        Self { root, socket }
    }

    fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

impl Drop for SocketFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn system_command_line_requires_socket_path() {
    let error = SystemCommandLine::from_arguments(std::iter::empty::<&str>())
        .daemon()
        .expect_err("missing socket is typed");

    assert_eq!(error.to_string(), "system socket path is missing");
}

#[test]
fn system_daemon_answers_status_readiness() {
    let fixture = SocketFixture::new("status");
    let server = SystemDaemon::from_socket(fixture.socket())
        .bind()
        .expect("daemon binds before client connects");
    let socket = server.socket().clone();
    let handle = thread::spawn(move || server.serve_one());

    let mut stream = UnixStream::connect(socket).expect("client connects");
    write_request(
        &mut stream,
        SystemStatusQuery {
            backend: SystemBackend::Niri,
        }
        .into(),
    );
    let event = read_event(&mut stream);
    let server_event = handle
        .join()
        .expect("daemon thread joins")
        .expect("daemon handles one request");

    let expected = SystemEvent::SystemStatus(SystemStatus {
        backend: SystemBackend::Niri,
        health: SystemHealth::Running,
        readiness: SystemReadiness::Ready,
    });
    assert_eq!(event, expected);
    assert_eq!(server_event, expected);
}

#[test]
fn system_daemon_returns_typed_unimplemented() {
    let fixture = SocketFixture::new("unimplemented");
    let server = SystemDaemon::from_socket(fixture.socket())
        .bind()
        .expect("daemon binds before client connects");
    let socket = server.socket().clone();
    let handle = thread::spawn(move || server.serve_one());

    let mut stream = UnixStream::connect(socket).expect("client connects");
    write_request(
        &mut stream,
        FocusSubscription {
            target: SystemTarget::niri_window(223),
        }
        .into(),
    );
    let event = read_event(&mut stream);
    let server_event = handle
        .join()
        .expect("daemon thread joins")
        .expect("daemon handles one request");

    let expected = SystemEvent::SystemRequestUnimplemented(SystemRequestUnimplemented {
        operation: SystemOperationKind::FocusSubscription,
        reason: SystemUnimplementedReason::NotBuiltYet,
    });
    assert_eq!(event, expected);
    assert_eq!(server_event, expected);
}

fn write_request(stream: &mut UnixStream, request: SystemRequest) {
    let frame = SystemFrame::new(FrameBody::Request(Request::assert(request)));
    let bytes = frame.encode_length_prefixed().expect("request encodes");
    stream.write_all(&bytes).expect("request writes");
    stream.flush().expect("request flushes");
}

fn read_event(stream: &mut UnixStream) -> SystemEvent {
    let frame = SystemFrameCodec::default()
        .read_frame(stream)
        .expect("event frame reads");
    match frame.into_body() {
        FrameBody::Reply(Reply::Operation(event)) => event,
        other => panic!("expected system event reply, got {other:?}"),
    }
}

fn unique_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos()
}
