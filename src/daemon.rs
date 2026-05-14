use std::ffi::OsString;
use std::io::{BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::error::Infallible;
use kameo::message::{Context, Message};
use signal_core::{FrameBody, Reply, Request};
use signal_persona_system::{
    Frame as SystemFrame, SystemBackend, SystemEvent, SystemHealth, SystemOperationKind,
    SystemReadiness, SystemRequest, SystemRequestUnimplemented, SystemStatus, SystemStatusQuery,
    SystemUnimplementedReason,
};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct SystemDaemon {
    socket: PathBuf,
    backend: SystemBackend,
    socket_mode: Option<SocketMode>,
}

impl SystemDaemon {
    pub fn from_socket(socket: impl Into<PathBuf>) -> Self {
        Self {
            socket: socket.into(),
            backend: SystemBackend::Niri,
            socket_mode: SocketMode::from_environment(),
        }
    }

    pub fn with_backend(mut self, backend: SystemBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_socket_mode(mut self, socket_mode: SocketMode) -> Self {
        self.socket_mode = Some(socket_mode);
        self
    }

    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }

    pub fn backend(&self) -> SystemBackend {
        self.backend
    }

    pub fn run(self) -> Result<()> {
        let bound = self.bind()?;
        eprintln!("persona-system-daemon socket={}", bound.socket.display());
        bound.serve_forever()
    }

    pub fn bind(self) -> Result<BoundSystemDaemon> {
        if let Some(parent) = self.socket.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = std::fs::remove_file(&self.socket);
        let listener = UnixListener::bind(&self.socket)?;
        if let Some(socket_mode) = self.socket_mode {
            std::fs::set_permissions(
                &self.socket,
                std::fs::Permissions::from_mode(socket_mode.as_octal()),
            )?;
        }
        let runtime = tokio::runtime::Runtime::new()?;
        let system = runtime.block_on(SystemSupervisor::start(self.backend));
        Ok(BoundSystemDaemon {
            socket: self.socket,
            runtime,
            listener,
            system,
        })
    }

    pub fn serve_one(self) -> Result<SystemEvent> {
        self.bind()?.serve_one()
    }

    fn handle_connection(
        runtime: &tokio::runtime::Runtime,
        system: &ActorRef<SystemSupervisor>,
        stream: UnixStream,
    ) -> Result<SystemEvent> {
        let mut connection = SystemConnection::from_stream(stream);
        let request = connection.read_signal_request()?;
        let event = runtime.block_on(async {
            SystemRequestHandler::new(system.clone())
                .event_for_request(request)
                .await
        })?;
        connection.write_signal_event(event.clone())?;
        Ok(event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketMode(u32);

impl SocketMode {
    pub const fn from_octal(value: u32) -> Self {
        Self(value)
    }

    pub fn from_environment() -> Option<Self> {
        std::env::var("PERSONA_SOCKET_MODE")
            .ok()
            .and_then(|value| u32::from_str_radix(value.as_str(), 8).ok())
            .map(Self::from_octal)
    }

    pub const fn as_octal(self) -> u32 {
        self.0
    }
}

pub struct BoundSystemDaemon {
    socket: PathBuf,
    runtime: tokio::runtime::Runtime,
    listener: UnixListener,
    system: ActorRef<SystemSupervisor>,
}

impl BoundSystemDaemon {
    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }

    pub fn serve_one(self) -> Result<SystemEvent> {
        let (stream, _address) = self.listener.accept()?;
        let event = SystemDaemon::handle_connection(&self.runtime, &self.system, stream)?;
        self.runtime.block_on(SystemSupervisor::stop(self.system))?;
        let _ = std::fs::remove_file(&self.socket);
        Ok(event)
    }

    pub fn serve_forever(self) -> Result<()> {
        for stream in self.listener.incoming() {
            let stream = stream?;
            let _ = SystemDaemon::handle_connection(&self.runtime, &self.system, stream)?;
        }
        Ok(())
    }
}

pub struct SystemConnection {
    stream: BufReader<UnixStream>,
    signal: SystemFrameCodec,
}

impl SystemConnection {
    pub fn from_stream(stream: UnixStream) -> Self {
        Self {
            stream: BufReader::new(stream),
            signal: SystemFrameCodec::default(),
        }
    }

    pub fn read_signal_request(&mut self) -> Result<SystemRequest> {
        self.signal.read_request(&mut self.stream)
    }

    pub fn write_signal_event(&mut self, event: SystemEvent) -> Result<()> {
        let stream = self.stream.get_mut();
        self.signal.write_event(stream, event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemFrameCodec {
    maximum_frame_bytes: usize,
}

impl SystemFrameCodec {
    pub const fn new(maximum_frame_bytes: usize) -> Self {
        Self {
            maximum_frame_bytes,
        }
    }

    pub fn read_frame(&self, reader: &mut impl Read) -> Result<SystemFrame> {
        let mut prefix = [0_u8; 4];
        reader.read_exact(&mut prefix)?;
        let length = u32::from_be_bytes(prefix) as usize;
        if length > self.maximum_frame_bytes {
            return Err(Error::UnexpectedSignalFrame {
                got: format!("frame length {length} exceeds {}", self.maximum_frame_bytes),
            });
        }
        let mut bytes = Vec::with_capacity(4 + length);
        bytes.extend_from_slice(&prefix);
        bytes.resize(4 + length, 0);
        reader.read_exact(&mut bytes[4..])?;
        Ok(SystemFrame::decode_length_prefixed(&bytes)?)
    }

    pub fn read_request(&self, reader: &mut impl Read) -> Result<SystemRequest> {
        match self.read_frame(reader)?.into_body() {
            FrameBody::Request(Request::Operation { payload, .. }) => Ok(payload),
            other => Err(Error::UnexpectedSignalFrame {
                got: format!("{other:?}"),
            }),
        }
    }

    pub fn write_event(&self, writer: &mut impl Write, event: SystemEvent) -> Result<()> {
        let frame = SystemFrame::new(FrameBody::Reply(Reply::operation(event)));
        let bytes = frame.encode_length_prefixed()?;
        writer.write_all(&bytes)?;
        writer.flush()?;
        Ok(())
    }
}

impl Default for SystemFrameCodec {
    fn default() -> Self {
        Self::new(1024 * 1024)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, kameo::Reply)]
pub struct SystemState {
    pub backend: SystemBackend,
    pub served_request_count: u64,
    pub last_operation: Option<SystemOperationKind>,
}

#[derive(Debug)]
pub struct SystemSupervisor {
    backend: SystemBackend,
    served_request_count: u64,
    last_operation: Option<SystemOperationKind>,
}

impl SystemSupervisor {
    pub fn new(backend: SystemBackend) -> Self {
        Self {
            backend,
            served_request_count: 0,
            last_operation: None,
        }
    }

    pub async fn start(backend: SystemBackend) -> ActorRef<Self> {
        let reference = Self::spawn(backend);
        reference.wait_for_startup().await;
        reference
    }

    pub async fn stop(reference: ActorRef<Self>) -> Result<()> {
        reference
            .stop_gracefully()
            .await
            .map_err(|error| Error::ActorCall {
                detail: error.to_string(),
            })?;
        reference.wait_for_shutdown().await;
        Ok(())
    }

    fn state(&self) -> SystemState {
        SystemState {
            backend: self.backend,
            served_request_count: self.served_request_count,
            last_operation: self.last_operation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadSystemState {
    pub minimum_served_request_count: u64,
}

impl ReadSystemState {
    pub fn expecting_at_least(minimum_served_request_count: u64) -> Self {
        Self {
            minimum_served_request_count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordServedSystemRequest {
    pub operation: SystemOperationKind,
}

impl Actor for SystemSupervisor {
    type Args = SystemBackend;
    type Error = Infallible;

    async fn on_start(
        backend: Self::Args,
        _actor_reference: ActorRef<Self>,
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Self::new(backend))
    }
}

impl Message<ReadSystemState> for SystemSupervisor {
    type Reply = SystemState;

    async fn handle(
        &mut self,
        message: ReadSystemState,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let _satisfied = self.served_request_count >= message.minimum_served_request_count;
        self.state()
    }
}

impl Message<RecordServedSystemRequest> for SystemSupervisor {
    type Reply = SystemState;

    async fn handle(
        &mut self,
        message: RecordServedSystemRequest,
        _context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.last_operation = Some(message.operation);
        self.served_request_count = self.served_request_count.saturating_add(1);
        self.state()
    }
}

#[derive(Debug, Clone)]
pub struct SystemRequestHandler {
    system: ActorRef<SystemSupervisor>,
}

impl SystemRequestHandler {
    pub fn new(system: ActorRef<SystemSupervisor>) -> Self {
        Self { system }
    }

    pub async fn event_for_request(&self, request: SystemRequest) -> Result<SystemEvent> {
        let operation = request.operation_kind();
        let _state = self
            .system
            .ask(RecordServedSystemRequest { operation })
            .await
            .map_err(|error| Error::ActorCall {
                detail: error.to_string(),
            })?;
        match request {
            SystemRequest::SystemStatusQuery(query) => self.status_event(query).await,
            other => Ok(SystemRequestUnimplemented {
                operation: other.operation_kind(),
                reason: SystemUnimplementedReason::NotBuiltYet,
            }
            .into()),
        }
    }

    async fn status_event(&self, query: SystemStatusQuery) -> Result<SystemEvent> {
        let state = self
            .system
            .ask(ReadSystemState::expecting_at_least(0))
            .await
            .map_err(|error| Error::ActorCall {
                detail: error.to_string(),
            })?;
        Ok(SystemStatus {
            backend: query.backend,
            health: Self::health(&state),
            readiness: Self::readiness(&state),
        }
        .into())
    }

    fn health(state: &SystemState) -> SystemHealth {
        match state.backend {
            SystemBackend::Niri => SystemHealth::Running,
        }
    }

    fn readiness(state: &SystemState) -> SystemReadiness {
        match state.backend {
            SystemBackend::Niri => SystemReadiness::Ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemCommandLine {
    arguments: Vec<OsString>,
}

impl SystemCommandLine {
    pub fn from_environment() -> Self {
        Self::from_arguments(std::env::args_os().skip(1))
    }

    pub fn from_arguments<I, S>(arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self {
            arguments: arguments.into_iter().map(Into::into).collect(),
        }
    }

    pub fn daemon(&self) -> Result<SystemDaemon> {
        let socket = self.arguments.first().ok_or(Error::MissingSocket)?;
        if let Some(extra) = self.arguments.get(1) {
            return Err(Error::UnexpectedArgument {
                got: extra.to_string_lossy().to_string(),
            });
        }
        Ok(SystemDaemon::from_socket(PathBuf::from(socket)))
    }

    pub fn run(&self) -> Result<()> {
        self.daemon()?.run()
    }
}
