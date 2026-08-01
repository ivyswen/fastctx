//! Thin stdio proxy and the per-user, per-build FastCtx control center.

pub(crate) mod activity;
mod local_ipc;
mod protocol;
#[cfg(windows)]
mod windows_process;

use crate::control::paths::ControlPaths;
use crate::file_executor::GrepGlobExecutor;
use crate::server::{FastCtxServer, ServerOptions, SharedRuntime};
use crate::session::{SessionContext, SessionEnvironment};
use fs2::FileExt;
use local_ipc::{BoxedStream, Listener, LocalEndpoint};
use rmcp::ServiceExt;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncWriteExt, split};
use tokio::sync::OnceCell;
use tokio_util::sync::CancellationToken;

const STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_RETRY: Duration = Duration::from_millis(20);
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const DEFAULT_MAINTENANCE_INTERVAL: Duration = Duration::from_secs(60);
const SERVICE_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(3);

/// Captures the thin proxy's native state without loading settings or any heavy executor.
pub(crate) fn capture_proxy_environment() -> Result<SessionEnvironment, String> {
    SessionEnvironment::capture()
}

/// Connects to the matching control center or starts exactly one before MCP stdin is consumed.
pub(crate) async fn connect_or_start(
    options: ServerOptions,
    environment: &SessionEnvironment,
) -> Result<BoxedStream, String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Cannot locate the running fastctx binary: {error}"))?;
    let endpoint = endpoint_for(environment)?;
    crate::edit::private_storage::ensure_private_directory(
        endpoint.runtime_directory(),
        "control-center runtime",
    )?;

    if let Ok(mut stream) = local_ipc::connect(&endpoint).await {
        establish(&mut stream, options, environment.clone()).await?;
        return Ok(stream);
    }

    let startup_lock = crate::edit::private_storage::open_lock_file(
        &endpoint.startup_lock_path(),
        "control-center startup lock",
    )?;
    acquire_startup_lock(&startup_lock, &endpoint.startup_lock_path()).await?;

    if let Ok(mut stream) = local_ipc::connect(&endpoint).await {
        establish(&mut stream, options, environment.clone()).await?;
        return Ok(stream);
    }

    spawn_bootstrap(&executable, environment)?;
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        match local_ipc::connect(&endpoint).await {
            Ok(mut stream) => {
                establish(&mut stream, options, environment.clone()).await?;
                return Ok(stream);
            }
            Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(CONNECT_RETRY).await;
            }
            Err(error) => {
                return Err(format!(
                    "The FastCtx control center did not become ready within 10 seconds: {error}"
                ));
            }
        }
    }
}

async fn establish(
    stream: &mut BoxedStream,
    options: ServerOptions,
    environment: SessionEnvironment,
) -> Result<(), String> {
    tokio::time::timeout(STARTUP_TIMEOUT, async {
        protocol::write_handshake(stream, &protocol::Handshake::new(options, environment)).await?;
        protocol::read_handshake_response(stream).await
    })
    .await
    .map_err(|_| {
        "Timed out waiting for the FastCtx control center to accept the session handshake."
            .to_string()
    })?
}

async fn acquire_startup_lock(file: &File, path: &Path) -> Result<(), String> {
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(()),
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
            {
                tokio::time::sleep(CONNECT_RETRY).await;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(
                    "Timed out waiting for another FastCtx proxy to finish starting the control center."
                        .to_string(),
                );
            }
            Err(error) => {
                return Err(format!(
                    "Cannot lock the control-center startup gate {}: {error}",
                    crate::paths::display_path(path)
                ));
            }
        }
    }
}

fn endpoint_for(environment: &SessionEnvironment) -> Result<LocalEndpoint, String> {
    let home = environment
        .var_os("HOME")
        .filter(|value| !value.is_empty())
        .or_else(|| {
            environment
                .var_os("USERPROFILE")
                .filter(|value| !value.is_empty())
        })
        .ok_or_else(|| {
            "Cannot determine the user home directory for the FastCtx control center. Set HOME or USERPROFILE and retry."
                .to_string()
        })?;
    let home_hash = short_hash(&crate::session::native_bytes(&home), 12);
    let build_id = effective_build_id(environment);
    let id = format!("fastctx-engine-{home_hash}-{build_id}");
    let runtime_directory = crate::edit::private_storage::control_center_directory();
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let socket_length = runtime_directory.as_os_str().as_bytes().len() + id.len() + 7;
        if socket_length > 100 {
            return Err(format!(
                "The private control-center socket path is too long ({socket_length} bytes): {}",
                crate::paths::display_path(&runtime_directory.join(format!("{id}.sock")))
            ));
        }
    }
    Ok(LocalEndpoint::new(runtime_directory, id))
}

fn short_hash(bytes: &[u8], characters: usize) -> String {
    let digest = Sha256::digest(bytes);
    hex::encode(digest)[..characters].to_string()
}

fn effective_build_id(environment: &SessionEnvironment) -> String {
    #[cfg(debug_assertions)]
    if let Ok(override_id) = environment.var("FASTCTX_TEST_BUILD_ID")
        && !override_id.is_empty()
        && override_id.len() <= 32
        && override_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return override_id;
    }
    env!("FASTCTX_BUILD_ID").to_string()
}

fn spawn_bootstrap(executable: &Path, environment: &SessionEnvironment) -> Result<(), String> {
    let mut command = Command::new(executable);
    environment.configure_command(&mut command);
    command
        .arg("runtime-bootstrap")
        .current_dir(environment.cwd())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(crate::process_policy::noninteractive_creation_flags(0));
    }
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot start the FastCtx control-center bootstrap: {error}"))
}

/// Intermediate child that reparents the long-lived control center before the proxy returns.
pub(crate) fn run_bootstrap_entry() -> Result<(), String> {
    let executable = std::env::current_exe()
        .map_err(|error| format!("Cannot locate the control-center binary: {error}"))?;
    let environment = SessionEnvironment::capture()?;
    let mut command = Command::new(&executable);
    environment.configure_command(&mut command);
    command
        .arg("runtime-host")
        .current_dir(environment.cwd())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: pre_exec performs only the async-signal-safe setsid syscall.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(())
                }
            });
        }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{
            CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS,
        };
        let detached = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;
        match windows_process::spawn_without_inherited_handles(
            &executable,
            environment.cwd(),
            detached | CREATE_BREAKAWAY_FROM_JOB,
        ) {
            Ok(_) => Ok(()),
            Err(error) if error.raw_os_error() == Some(5) => {
                windows_process::spawn_without_inherited_handles(
                    &executable,
                    environment.cwd(),
                    detached,
                )
                .map_err(|error| format!("Cannot detach the FastCtx control center: {error}"))
            }
            Err(error) => Err(format!("Cannot detach the FastCtx control center: {error}")),
        }
    }
    #[cfg(not(windows))]
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Cannot detach the FastCtx control center: {error}"))
}

struct HostState {
    runtime: OnceCell<Arc<SharedRuntime>>,
    control_paths: OnceCell<ControlPaths>,
    activity: Arc<activity::RuntimeActivity>,
}

impl HostState {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            runtime: OnceCell::new(),
            control_paths: OnceCell::new(),
            activity: activity::RuntimeActivity::new(),
        })
    }

    async fn runtime_for(
        &self,
        session: &Arc<SessionContext>,
    ) -> Result<Arc<SharedRuntime>, String> {
        let runtime = self
            .runtime
            .get_or_try_init(|| async {
                let parallelism = session.settings.search_parallelism().map_err(|error| {
                    format!(
                        "Cannot start the MCP session with settings from {}: {error}. Repair the value and retry.",
                        crate::paths::display_path(&session.control_paths.fastctx_config)
                    )
                })?;
                let executor = Arc::new(GrepGlobExecutor::with_parallelism(parallelism.effective));
                Ok::<_, String>(SharedRuntime::with_activity(
                    executor,
                    Arc::clone(&self.activity),
                ))
            })
            .await?;
        let _ = self.control_paths.set(session.control_paths.clone());
        Ok(Arc::clone(runtime))
    }
}

/// Final detached control-center entry point.
pub(crate) async fn run_host_entry(
    idle_timeout_ms: Option<u64>,
    maintenance_interval_ms: Option<u64>,
) -> Result<(), String> {
    let environment = SessionEnvironment::capture()?;
    let endpoint = endpoint_for(&environment)?;
    crate::edit::private_storage::ensure_private_directory(
        endpoint.runtime_directory(),
        "control-center runtime",
    )?;
    let instance_lock = crate::edit::private_storage::open_lock_file(
        &endpoint.instance_lock_path(),
        "control-center instance lock",
    )?;
    match instance_lock.try_lock_exclusive() {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
        Err(error) => {
            return Err(format!(
                "Cannot lock the control-center instance gate: {error}"
            ));
        }
    }
    let mut listener = Listener::bind(&endpoint)?;
    #[cfg(debug_assertions)]
    record_test_host_start(&environment);
    let state = HostState::new();
    let shutdown = CancellationToken::new();
    let idle_timeout = idle_timeout_ms
        .or_else(|| debug_duration_override(&environment, "FASTCTX_TEST_RUNTIME_IDLE_MS"))
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_IDLE_TIMEOUT);
    let monitor = tokio::spawn(monitor_idle(
        Arc::clone(&state),
        shutdown.clone(),
        idle_timeout,
    ));
    let maintenance = tokio::spawn(monitor_maintenance(
        Arc::clone(&state),
        shutdown.clone(),
        maintenance_interval_ms
            .or_else(|| {
                debug_duration_override(&environment, "FASTCTX_TEST_RUNTIME_MAINTENANCE_MS")
            })
            .map(Duration::from_millis)
            .unwrap_or(DEFAULT_MAINTENANCE_INTERVAL),
    ));
    let mut connections = tokio::task::JoinSet::new();

    loop {
        tokio::select! {
            () = shutdown.cancelled() => break,
            accepted = listener.accept() => {
                let stream = accepted?;
                state.activity.touch();
                connections.spawn(serve_connection(
                    stream,
                    Arc::clone(&state),
                    shutdown.clone(),
                ));
            }
            result = connections.join_next(), if !connections.is_empty() => {
                if let Some(Err(error)) = result {
                    eprintln!("fastctx control center: connection task failed: {error}");
                }
            }
        }
    }

    shutdown.cancel();
    let deadline = tokio::time::sleep(SERVICE_SHUTDOWN_TIMEOUT);
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => {
                connections.abort_all();
                while connections.join_next().await.is_some() {}
                break;
            }
            result = connections.join_next(), if !connections.is_empty() => {
                if result.is_none() || connections.is_empty() {
                    break;
                }
            }
            else => break,
        }
    }
    monitor.abort();
    let _ = monitor.await;
    maintenance.abort();
    let _ = maintenance.await;
    drop(instance_lock);
    Ok(())
}

#[cfg(debug_assertions)]
fn debug_duration_override(environment: &SessionEnvironment, name: &str) -> Option<u64> {
    environment.var(name).ok()?.parse::<u64>().ok()
}

#[cfg(not(debug_assertions))]
fn debug_duration_override(_environment: &SessionEnvironment, _name: &str) -> Option<u64> {
    None
}

#[cfg(debug_assertions)]
fn record_test_host_start(environment: &SessionEnvironment) {
    use std::io::Write as _;
    let Some(path) = environment.var_os("FASTCTX_TEST_RUNTIME_EVENT_LOG") else {
        return;
    };
    let mut options = std::fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    if let Ok(mut file) = options.open(PathBuf::from(path)) {
        let _ = writeln!(file, "START {}", std::process::id());
    }
}

async fn monitor_maintenance(
    state: Arc<HostState>,
    shutdown: CancellationToken,
    interval: Duration,
) {
    loop {
        tokio::select! {
            () = shutdown.cancelled() => return,
            () = tokio::time::sleep(interval) => {}
        }
        let Some(paths) = state.control_paths.get().cloned() else {
            continue;
        };
        match tokio::task::spawn_blocking(move || crate::shell::jobs::reap_history(&paths)).await {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => {
                eprintln!("fastctx control center: periodic background-job cleanup failed: {error}")
            }
            Err(error) => eprintln!(
                "fastctx control center: periodic background-job cleanup task failed: {error}"
            ),
        }
    }
}

async fn serve_connection(
    mut stream: BoxedStream,
    state: Arc<HostState>,
    shutdown: CancellationToken,
) {
    let handshake = match protocol::read_handshake(&mut stream).await {
        Ok(handshake) => handshake,
        Err(error) => {
            let _ = protocol::write_handshake_error(&mut stream, error).await;
            return;
        }
    };
    let session = match SessionContext::from_environment(handshake.environment) {
        Ok(session) => session,
        Err(error) => {
            let _ = protocol::write_handshake_error(&mut stream, error).await;
            return;
        }
    };
    let runtime = match state.runtime_for(&session).await {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = protocol::write_handshake_error(&mut stream, error).await;
            return;
        }
    };
    if protocol::write_handshake_success(&mut stream)
        .await
        .is_err()
    {
        return;
    }
    let (reader, writer) = split(stream);
    let service = match FastCtxServer::with_session_and_runtime(handshake.options, session, runtime)
        .serve((reader, writer))
        .await
    {
        Ok(service) => service,
        Err(error) => {
            eprintln!("fastctx control center: cannot start MCP connection: {error}");
            return;
        }
    };
    let cancellation = service.cancellation_token();
    let mut waiting = tokio::spawn(service.waiting());
    tokio::select! {
        _ = shutdown.cancelled() => {
            cancellation.cancel();
            if tokio::time::timeout(SERVICE_SHUTDOWN_TIMEOUT, &mut waiting).await.is_err() {
                waiting.abort();
                let _ = waiting.await;
            }
        }
        _ = &mut waiting => {}
    }
}

async fn monitor_idle(state: Arc<HostState>, shutdown: CancellationToken, idle_timeout: Duration) {
    let interval = idle_timeout
        .div_f64(4.0)
        .clamp(Duration::from_millis(50), Duration::from_secs(30));
    loop {
        tokio::select! {
            () = shutdown.cancelled() => return,
            () = tokio::time::sleep(interval) => {}
        }
        if !state.activity.is_idle_for(idle_timeout) {
            continue;
        }
        let Some(paths) = state.control_paths.get().cloned() else {
            shutdown.cancel();
            return;
        };
        let running = tokio::task::spawn_blocking(move || {
            crate::shell::jobs::running_summaries(&paths).map(|jobs| !jobs.is_empty())
        })
        .await;
        match running {
            Ok(Ok(false)) => {
                // A request may have arrived while the registry scan ran off-thread.
                if !state.activity.is_idle_for(idle_timeout) {
                    continue;
                }
                shutdown.cancel();
                return;
            }
            Ok(Ok(true)) => {}
            Ok(Err(error)) => eprintln!(
                "fastctx control center: cannot inspect running jobs for idle shutdown: {error}"
            ),
            Err(error) => {
                eprintln!("fastctx control center: running-job inspection task failed: {error}")
            }
        }
    }
}

/// Pumps MCP bytes after the handshake. Any post-establishment failure exits without fallback.
pub(crate) async fn forward_stdio(
    stream: BoxedStream,
    parent: Option<Option<crate::process_identity::ProcessIdentity>>,
) -> Result<ExitCode, String> {
    let stdin = crate::stdio_transport::DetachedStdin::start()?;
    let stdin_eof = stdin.eof_token();
    let stdin_error = stdin.read_error_receiver();
    let (mut reader, mut writer) = split(stream);
    let mut stdout = tokio::io::stdout();
    let mut upload = tokio::spawn(async move {
        let result = tokio::io::copy(&mut { stdin }, &mut writer).await;
        let shutdown = writer.shutdown().await;
        result.and(shutdown)
    });
    let mut download = tokio::spawn(async move {
        tokio::io::copy(&mut reader, &mut stdout).await?;
        stdout.flush().await
    });

    let monitor_stop = Arc::new(AtomicBool::new(false));
    let (parent_exit, monitor) = parent_exit_monitor(parent, Arc::clone(&monitor_stop));
    tokio::pin!(parent_exit);
    let stdin_error_wait = wait_for_stdin_error(stdin_error.clone());
    tokio::pin!(stdin_error_wait);

    let result = tokio::select! {
        result = &mut download => match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(error)) => Err(format!("The FastCtx control-center connection failed: {error}")),
            Err(error) => Err(format!("The FastCtx control-center output task failed: {error}")),
        },
        result = &mut upload => match stdin_error.borrow().clone() {
            Some(error) => Err(error),
            None => match result {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(error)) => Err(format!("Cannot forward MCP stdin to the FastCtx control center: {error}")),
                Err(error) => Err(format!("The FastCtx control-center input task failed: {error}")),
            }
        },
        error = &mut stdin_error_wait => Err(error),
        () = stdin_eof.cancelled() => Ok(()),
        () = &mut parent_exit => Ok(()),
        () = wait_for_termination_signal() => Ok(()),
    };
    upload.abort();
    download.abort();
    monitor_stop.store(true, Ordering::Release);
    if let Some(monitor) = monitor {
        let _ = monitor.await;
    }
    result?;
    Ok(ExitCode::SUCCESS)
}

type ParentExitFuture = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;
type ParentExitMonitor = (ParentExitFuture, Option<tokio::task::JoinHandle<()>>);

fn parent_exit_monitor(
    parent: Option<Option<crate::process_identity::ProcessIdentity>>,
    stop: Arc<AtomicBool>,
) -> ParentExitMonitor {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let monitor = match parent {
        None => None,
        Some(None) => {
            let _ = sender.send(());
            return (Box::pin(async {}), None);
        }
        Some(Some(identity)) => Some(tokio::task::spawn_blocking(move || {
            if crate::process_identity::wait_for_identity_exit_until(&identity, &stop) {
                let _ = sender.send(());
            }
        })),
    };
    let future = async move {
        match receiver.await {
            Ok(()) => {}
            Err(_) => std::future::pending::<()>().await,
        }
    };
    (Box::pin(future), monitor)
}

async fn wait_for_stdin_error(
    mut receiver: tokio::sync::watch::Receiver<Option<String>>,
) -> String {
    loop {
        if let Some(error) = receiver.borrow().clone() {
            return error;
        }
        if receiver.changed().await.is_err() {
            return std::future::pending::<String>().await;
        }
    }
}

#[cfg(unix)]
async fn wait_for_termination_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let Ok(mut terminate) = signal(SignalKind::terminate()) else {
        return std::future::pending::<()>().await;
    };
    let Ok(mut interrupt) = signal(SignalKind::interrupt()) else {
        return std::future::pending::<()>().await;
    };
    tokio::select! {
        _ = terminate.recv() => {}
        _ = interrupt.recv() => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_termination_signal() {
    std::future::pending::<()>().await
}

#[cfg(test)]
mod tests {
    use super::short_hash;

    #[test]
    fn endpoint_hashes_are_stable_and_separate_inputs() {
        assert_eq!(short_hash(b"home-a", 12), short_hash(b"home-a", 12));
        assert_ne!(short_hash(b"home-a", 12), short_hash(b"home-b", 12));
    }
}
