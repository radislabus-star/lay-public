use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "lay-l1.1-serve", about = "Shadow L1.1 local service host")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run {
        #[arg(long, value_name = "PACKAGE")]
        memory: PathBuf,
        #[arg(long, value_name = "SOCKET", default_value_os_t = lay::nanda_wave::default_l11_socket_path())]
        socket: PathBuf,
    },
    Health {
        #[arg(long, value_name = "SOCKET", default_value_os_t = lay::nanda_wave::default_l11_socket_path())]
        socket: PathBuf,
    },
    Stats {
        #[arg(long, value_name = "SOCKET", default_value_os_t = lay::nanda_wave::default_l11_socket_path())]
        socket: PathBuf,
    },
    Reload {
        #[arg(long, value_name = "PACKAGE")]
        memory: PathBuf,
        #[arg(long, value_name = "SOCKET", default_value_os_t = lay::nanda_wave::default_l11_socket_path())]
        socket: PathBuf,
    },
}

struct ServiceState {
    started_at: Instant,
    socket_path: PathBuf,
    request_count: AtomicU64,
    host: Arc<RwLock<HostedMemory>>,
}

struct RequestJob {
    request: lay::nanda_wave::L1ServiceRequest,
    response: SyncSender<lay::nanda_wave::L1ServiceResponse>,
}

struct ActiveConnectionGuard {
    active: Arc<AtomicUsize>,
}

impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::Relaxed);
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "boxing would change the established hosted-memory state"
)]
enum HostedMemory {
    Loading {
        package_path: PathBuf,
    },
    Ready(lay::nanda_wave::L1RestorationHost),
    Failed {
        package_path: PathBuf,
        message: String,
    },
}

struct SocketGuard {
    path: PathBuf,
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    match args.command {
        Command::Run { memory, socket } => run_server(&memory, &socket)?,
        Command::Health { socket } => print_response(lay::nanda_wave::send_l11_service_request(
            &socket,
            &lay::nanda_wave::L1ServiceRequest::Health,
        )?)?,
        Command::Stats { socket } => print_response(lay::nanda_wave::send_l11_service_request(
            &socket,
            &lay::nanda_wave::L1ServiceRequest::Stats,
        )?)?,
        Command::Reload { memory, socket } => {
            print_response(lay::nanda_wave::send_l11_service_request(
                &socket,
                &lay::nanda_wave::L1ServiceRequest::Reload { memory },
            )?)?
        }
    }
    Ok(())
}

fn run_server(memory: &Path, socket_path: &Path) -> io::Result<()> {
    lay::nanda_wave::admit_l11_service_artifact(memory)?;
    let listener = bind_socket(socket_path)?;
    let _guard = SocketGuard {
        path: socket_path.to_path_buf(),
    };
    let host = Arc::new(RwLock::new(HostedMemory::Loading {
        package_path: memory.to_path_buf(),
    }));
    let state = Arc::new(ServiceState {
        started_at: Instant::now(),
        socket_path: socket_path.to_path_buf(),
        request_count: AtomicU64::new(0),
        host: Arc::clone(&host),
    });
    spawn_background_load(host, memory.to_path_buf());
    let worker_count = service_worker_count();
    let (sender, receiver) = sync_channel::<RequestJob>(worker_count.saturating_mul(4));
    let receiver = Arc::new(Mutex::new(receiver));
    for worker_id in 0..worker_count {
        spawn_request_worker(worker_id, Arc::clone(&receiver), Arc::clone(&state))?;
    }
    let connection_limit = service_connection_limit();
    let active_connections = Arc::new(AtomicUsize::new(0));
    let next_connection_id = AtomicU64::new(0);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if !try_acquire_connection(&active_connections, connection_limit) {
                    reject_connection(stream, "L1.1 connection limit reached");
                    continue;
                }
                let connection_id = next_connection_id.fetch_add(1, Ordering::Relaxed);
                if let Err(error) = spawn_connection_handler(
                    connection_id,
                    stream,
                    sender.clone(),
                    Arc::clone(&active_connections),
                ) {
                    active_connections.fetch_sub(1, Ordering::Relaxed);
                    return Err(error);
                }
            }
            Err(error) => eprintln!("lay-l1.1-serve accept failed: {error}"),
        }
    }
    Ok(())
}

fn service_connection_limit() -> usize {
    std::env::var("LAY_L11_SERVICE_CONNECTIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(64)
        .clamp(1, 1_024)
}

fn try_acquire_connection(active: &AtomicUsize, limit: usize) -> bool {
    active
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current < limit).then_some(current + 1)
        })
        .is_ok()
}

fn service_worker_count() -> usize {
    std::env::var("LAY_L11_SERVICE_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1)
                .min(4)
        })
        .clamp(1, 32)
}

fn spawn_request_worker(
    worker_id: usize,
    receiver: Arc<Mutex<Receiver<RequestJob>>>,
    state: Arc<ServiceState>,
) -> io::Result<()> {
    thread::Builder::new()
        .name(format!("lay-l11-{worker_id}"))
        .spawn(move || loop {
            let job = {
                let receiver = receiver.lock().expect("L1.1 service receiver lock");
                receiver.recv()
            };
            let Ok(job) = job else {
                break;
            };
            let response = handle_request(job.request, &state);
            let _ = job.response.send(response);
        })
        .map(|_| ())
}

fn spawn_connection_handler(
    connection_id: u64,
    stream: UnixStream,
    requests: SyncSender<RequestJob>,
    active_connections: Arc<AtomicUsize>,
) -> io::Result<()> {
    thread::Builder::new()
        .name(format!("lay-l11-conn-{connection_id}"))
        .spawn(move || {
            let _guard = ActiveConnectionGuard {
                active: active_connections,
            };
            if let Err(error) = handle_client(stream, &requests) {
                eprintln!("lay-l1.1-serve client error: {error}");
            }
        })
        .map(|_| ())
}

fn reject_connection(mut stream: UnixStream, message: &str) {
    let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
    let _ = serde_json::to_writer(
        &mut stream,
        &lay::nanda_wave::L1ServiceResponse::Error {
            message: message.to_string(),
        },
    );
    let _ = stream.write_all(b"\n");
    let _ = stream.flush();
}

fn spawn_background_load(host: Arc<RwLock<HostedMemory>>, package_path: PathBuf) {
    thread::spawn(move || {
        let next = match lay::nanda_wave::L1RestorationHost::load(&package_path) {
            Ok(loaded) => match loaded.warm_first_touch() {
                Ok(report) => {
                    eprintln!("lay-l1.1-serve warmup: {report}");
                    match lay::nanda_wave::admit_l11_service_artifact(&package_path) {
                        Ok(_) => HostedMemory::Ready(loaded),
                        Err(error) => HostedMemory::Failed {
                            package_path,
                            message: format!(
                                "active admission changed before warmup completed: {error}"
                            ),
                        },
                    }
                }
                Err(error) => HostedMemory::Failed {
                    package_path,
                    message: format!("first-touch warmup failed: {error}"),
                },
            },
            Err(error) => HostedMemory::Failed {
                package_path,
                message: error.to_string(),
            },
        };
        *host.write().expect("L1.1 host write lock") = next;
    });
}

fn bind_socket(socket_path: &Path) -> io::Result<UnixListener> {
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if socket_path.exists() {
        match UnixStream::connect(socket_path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    format!("L1.1 service already listens on {}", socket_path.display()),
                ));
            }
            Err(_) => fs::remove_file(socket_path)?,
        }
    }
    let listener = UnixListener::bind(socket_path)?;
    fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

fn handle_client(stream: UnixStream, requests: &SyncSender<RequestJob>) -> io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<lay::nanda_wave::L1ServiceRequest>(&line) {
            Ok(request) => dispatch_request(request, requests),
            Err(error) => lay::nanda_wave::L1ServiceResponse::Error {
                message: format!("invalid request: {error}"),
            },
        };
        serde_json::to_writer(&mut writer, &response).map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }
    Ok(())
}

fn dispatch_request(
    request: lay::nanda_wave::L1ServiceRequest,
    requests: &SyncSender<RequestJob>,
) -> lay::nanda_wave::L1ServiceResponse {
    let mutating = matches!(&request, lay::nanda_wave::L1ServiceRequest::Reload { .. });
    let (response_sender, response_receiver) = sync_channel(1);
    match requests.try_send(RequestJob {
        request,
        response: response_sender,
    }) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            return lay::nanda_wave::L1ServiceResponse::Error {
                message: "L1.1 request capacity is busy".to_string(),
            };
        }
        Err(TrySendError::Disconnected(_)) => {
            return lay::nanda_wave::L1ServiceResponse::Error {
                message: "L1.1 request workers stopped".to_string(),
            };
        }
    }
    let response: Result<_, String> = if mutating {
        response_receiver.recv().map_err(|error| error.to_string())
    } else {
        response_receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|error| error.to_string())
    };
    response.unwrap_or_else(|error| lay::nanda_wave::L1ServiceResponse::Error {
        message: format!("L1.1 request did not complete: {error}"),
    })
}

fn handle_request(
    request: lay::nanda_wave::L1ServiceRequest,
    state: &Arc<ServiceState>,
) -> lay::nanda_wave::L1ServiceResponse {
    state.request_count.fetch_add(1, Ordering::Relaxed);
    match request {
        lay::nanda_wave::L1ServiceRequest::Restore { surface, limit } => {
            if let Err(response) = validate_live_lattice_limit(limit) {
                return response;
            }
            let host = state.host.read().expect("L1.1 host read lock");
            match &*host {
                HostedMemory::Ready(host) => restore_response(host.try_restore(&surface, limit)),
                HostedMemory::Loading { package_path } => {
                    lay::nanda_wave::L1ServiceResponse::Error {
                        message: format!(
                            "L1.1 host is still warming for {}",
                            package_path.display()
                        ),
                    }
                }
                HostedMemory::Failed {
                    package_path,
                    message,
                } => lay::nanda_wave::L1ServiceResponse::Error {
                    message: format!(
                        "L1.1 host failed to load {}: {message}",
                        package_path.display()
                    ),
                },
            }
        }
        lay::nanda_wave::L1ServiceRequest::Lattice { surface, limit } => {
            if let Err(response) = validate_live_lattice_limit(limit) {
                return response;
            }
            let host = state.host.read().expect("L1.1 host read lock");
            match &*host {
                HostedMemory::Ready(host) => {
                    lattice_response(host.try_typed_lattice_seed_rows(&surface, limit))
                }
                HostedMemory::Loading { package_path } => {
                    lay::nanda_wave::L1ServiceResponse::Error {
                        message: format!(
                            "L1.1 host is still warming for {}",
                            package_path.display()
                        ),
                    }
                }
                HostedMemory::Failed {
                    package_path,
                    message,
                } => lay::nanda_wave::L1ServiceResponse::Error {
                    message: format!(
                        "L1.1 host failed to load {}: {message}",
                        package_path.display()
                    ),
                },
            }
        }
        lay::nanda_wave::L1ServiceRequest::Decode { terminal_ids } => {
            let host = state.host.read().expect("L1.1 host read lock");
            match &*host {
                HostedMemory::Ready(host) => lay::nanda_wave::L1ServiceResponse::Decode {
                    surfaces: terminal_ids
                        .into_iter()
                        .filter_map(|terminal_id| {
                            host.decode_terminal(terminal_id)
                                .map(|surface| (terminal_id, surface))
                        })
                        .collect(),
                },
                HostedMemory::Loading { package_path } => {
                    lay::nanda_wave::L1ServiceResponse::Error {
                        message: format!(
                            "L1.1 host is still warming for {}",
                            package_path.display()
                        ),
                    }
                }
                HostedMemory::Failed {
                    package_path,
                    message,
                } => lay::nanda_wave::L1ServiceResponse::Error {
                    message: format!(
                        "L1.1 host failed to load {}: {message}",
                        package_path.display()
                    ),
                },
            }
        }
        lay::nanda_wave::L1ServiceRequest::Health => {
            let host = state.host.read().expect("L1.1 host read lock");
            lay::nanda_wave::L1ServiceResponse::Health {
                report: health_report(state, &host),
            }
        }
        lay::nanda_wave::L1ServiceRequest::Stats => {
            let host = state.host.read().expect("L1.1 host read lock");
            lay::nanda_wave::L1ServiceResponse::Stats {
                report: stats_report(state, &host),
            }
        }
        lay::nanda_wave::L1ServiceRequest::Reload { memory } => {
            if matches!(
                &*state.host.read().expect("L1.1 host read lock"),
                HostedMemory::Loading { .. }
            ) {
                return lay::nanda_wave::L1ServiceResponse::Error {
                    message: "L1.1 host is still warming; reload is not available yet".to_string(),
                };
            }
            if let Err(error) = lay::nanda_wave::admit_l11_service_artifact(&memory) {
                return lay::nanda_wave::L1ServiceResponse::Error {
                    message: format!("L1.1 reload admission failed: {error}"),
                };
            }
            // Keep serving the current immutable snapshot while the replacement
            // is loaded and validated. The write lock protects only the flip.
            match load_validate_replace(
                &state.host,
                || {
                    lay::nanda_wave::L1RestorationHost::load(&memory)
                        .and_then(|next| {
                            next.warm_first_touch()?;
                            let report = next.stats();
                            Ok((HostedMemory::Ready(next), report))
                        })
                        .map_err(|error| error.to_string())
                },
                |current, next| {
                    lay::nanda_wave::admit_l11_service_artifact(&memory).map_err(|error| {
                        format!("L1.1 reload admission changed before atomic flip: {error}")
                    })?;
                    let (HostedMemory::Ready(current), HostedMemory::Ready(next)) = (current, next)
                    else {
                        return Ok(());
                    };
                    let current = current.stats();
                    let next = next.stats();
                    if next.package_path != memory {
                        return Err(format!(
                            "L1.1 replacement loaded an unexpected package: {}",
                            next.package_path.display()
                        ));
                    }
                    if is_stale_manifest_reload(
                        &current.package_path,
                        current.manifest_generation,
                        &next.package_path,
                        next.manifest_generation,
                    ) {
                        return Err(format!(
                            "refusing stale L1.1 manifest generation {}",
                            next.manifest_generation
                        ));
                    }
                    Ok(())
                },
            ) {
                Ok(report) => lay::nanda_wave::L1ServiceResponse::Reload { report },
                Err(message) => lay::nanda_wave::L1ServiceResponse::Error { message },
            }
        }
    }
}

#[expect(
    clippy::result_large_err,
    reason = "the service response is the established protocol error"
)]
fn validate_live_lattice_limit(limit: usize) -> Result<(), lay::nanda_wave::L1ServiceResponse> {
    if (1..=lay::nanda_wave::L11_LIVE_LATTICE_LIMIT).contains(&limit) {
        return Ok(());
    }
    Err(lay::nanda_wave::L1ServiceResponse::Error {
        message: format!(
            "L1.1 live lattice limit must be in 1..={}, got {limit}",
            lay::nanda_wave::L11_LIVE_LATTICE_LIMIT
        ),
    })
}

fn restore_response(result: io::Result<serde_json::Value>) -> lay::nanda_wave::L1ServiceResponse {
    match result {
        Ok(report) => lay::nanda_wave::L1ServiceResponse::Restore { report },
        Err(error) => lay::nanda_wave::L1ServiceResponse::Error {
            message: format!("L1.1 restoration query failed: {error}"),
        },
    }
}

fn lattice_response(
    result: io::Result<Vec<(u32, String, bool, u32)>>,
) -> lay::nanda_wave::L1ServiceResponse {
    match result {
        Ok(rows) => lay::nanda_wave::L1ServiceResponse::Lattice {
            seeds: rows
                .into_iter()
                .map(|(terminal_id, surface, authority, score_milli)| {
                    lay::nanda_wave::L11SeedSurface {
                        terminal_id: Some(terminal_id),
                        surface,
                        authority,
                        score_milli,
                    }
                })
                .collect(),
        },
        Err(error) => lay::nanda_wave::L1ServiceResponse::Error {
            message: format!("L1.1 lattice query failed: {error}"),
        },
    }
}

fn load_validate_replace<T, R, E>(
    slot: &RwLock<T>,
    load: impl FnOnce() -> Result<(T, R), E>,
    validate: impl FnOnce(&T, &T) -> Result<(), E>,
) -> Result<R, E> {
    let (next, report) = load()?;
    let mut current = slot.write().expect("L1.1 host write lock");
    validate(&current, &next)?;
    *current = next;
    Ok(report)
}

fn is_stale_manifest_reload(
    current_path: &Path,
    current_generation: u64,
    next_path: &Path,
    next_generation: u64,
) -> bool {
    current_path == next_path && next_generation != 0 && current_generation > next_generation
}

fn health_report(state: &ServiceState, host: &HostedMemory) -> lay::nanda_wave::L1ServiceHealth {
    let requests_served = state.request_count.load(Ordering::Relaxed);
    let uptime_ms = state.started_at.elapsed().as_millis() as u64;
    match host {
        HostedMemory::Ready(host) => {
            let stats = host.stats();
            lay::nanda_wave::L1ServiceHealth {
                status: "ready".to_string(),
                message: None,
                socket: state.socket_path.clone(),
                package_path: stats.package_path,
                package_bytes: Some(stats.package_bytes),
                terminal_count: Some(stats.terminal_count),
                manifest_generation: stats.manifest_generation,
                requests_served,
                uptime_ms,
            }
        }
        HostedMemory::Loading { package_path } => lay::nanda_wave::L1ServiceHealth {
            status: "warming".to_string(),
            message: Some("L1.1 package is still loading".to_string()),
            socket: state.socket_path.clone(),
            package_path: package_path.clone(),
            package_bytes: None,
            terminal_count: None,
            manifest_generation: 0,
            requests_served,
            uptime_ms,
        },
        HostedMemory::Failed {
            package_path,
            message,
        } => lay::nanda_wave::L1ServiceHealth {
            status: "failed".to_string(),
            message: Some(message.clone()),
            socket: state.socket_path.clone(),
            package_path: package_path.clone(),
            package_bytes: None,
            terminal_count: None,
            manifest_generation: 0,
            requests_served,
            uptime_ms,
        },
    }
}

fn stats_report(state: &ServiceState, host: &HostedMemory) -> lay::nanda_wave::L1ServiceStats {
    let requests_served = state.request_count.load(Ordering::Relaxed);
    let uptime_ms = state.started_at.elapsed().as_millis() as u64;
    match host {
        HostedMemory::Ready(host) => lay::nanda_wave::L1ServiceStats {
            status: "ready".to_string(),
            message: None,
            socket: state.socket_path.clone(),
            requests_served,
            uptime_ms,
            host: Some(host.stats()),
        },
        HostedMemory::Loading { package_path } => lay::nanda_wave::L1ServiceStats {
            status: "warming".to_string(),
            message: Some(format!(
                "L1.1 package is still loading: {}",
                package_path.display()
            )),
            socket: state.socket_path.clone(),
            requests_served,
            uptime_ms,
            host: None,
        },
        HostedMemory::Failed {
            package_path,
            message,
        } => lay::nanda_wave::L1ServiceStats {
            status: "failed".to_string(),
            message: Some(format!("{}: {message}", package_path.display())),
            socket: state.socket_path.clone(),
            requests_served,
            uptime_ms,
            host: None,
        },
    }
}

fn print_response(response: lay::nanda_wave::L1ServiceResponse) -> io::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&response).map_err(io::Error::other)?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn manifest_reload_generation_is_monotonic_for_the_same_path() {
        let path = Path::new("/tmp/l11.runtime.json");
        assert!(is_stale_manifest_reload(path, 4, path, 3));
        assert!(!is_stale_manifest_reload(path, 4, path, 4));
        assert!(!is_stale_manifest_reload(path, 4, path, 5));
        assert!(!is_stale_manifest_reload(
            path,
            4,
            Path::new("/tmp/other.runtime.json"),
            3,
        ));
        assert!(!is_stale_manifest_reload(path, 4, path, 0));
    }

    #[test]
    fn replacement_load_does_not_block_current_snapshot_readers() {
        let slot = Arc::new(RwLock::new(1_u8));
        let thread_slot = Arc::clone(&slot);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();

        let replacement = thread::spawn(move || {
            load_validate_replace(
                &thread_slot,
                || {
                    started_tx.send(()).expect("announce replacement load");
                    release_rx.recv().expect("release replacement load");
                    Ok::<_, ()>((2_u8, ()))
                },
                |current, next| {
                    assert_eq!((*current, *next), (1, 2));
                    Ok(())
                },
            )
        });

        started_rx.recv().expect("replacement load started");
        for _ in 0..1_000 {
            assert_eq!(*slot.read().expect("read current snapshot"), 1);
        }
        release_tx.send(()).expect("finish replacement load");
        replacement
            .join()
            .expect("replacement thread")
            .expect("replacement succeeds");
        assert_eq!(*slot.read().expect("read replacement snapshot"), 2);
    }

    #[test]
    fn reload_during_initial_warmup_cannot_race_background_owner() {
        let state = Arc::new(ServiceState {
            started_at: Instant::now(),
            socket_path: PathBuf::from("/tmp/lay-l11-warmup-test.sock"),
            request_count: AtomicU64::new(0),
            host: Arc::new(RwLock::new(HostedMemory::Loading {
                package_path: PathBuf::from("/tmp/initial.v8.bin"),
            })),
        });
        let response = handle_request(
            lay::nanda_wave::L1ServiceRequest::Reload {
                memory: PathBuf::from("/tmp/replacement.v8.bin"),
            },
            &state,
        );
        assert!(matches!(
            response,
            lay::nanda_wave::L1ServiceResponse::Error { message }
                if message.contains("still warming")
        ));
    }

    #[test]
    fn query_failure_is_a_protocol_error_not_an_empty_success() {
        let restore = restore_response(Err(io::Error::other("forced restore failure")));
        let lattice = lattice_response(Err(io::Error::other("forced lattice failure")));

        assert!(matches!(
            restore,
            lay::nanda_wave::L1ServiceResponse::Error { message }
                if message.contains("forced restore failure")
        ));
        assert!(matches!(
            lattice,
            lay::nanda_wave::L1ServiceResponse::Error { message }
                if message.contains("forced lattice failure")
        ));
    }

    #[test]
    fn live_queries_reject_zero_or_oversized_lattice_limits() {
        let state = Arc::new(ServiceState {
            started_at: Instant::now(),
            socket_path: PathBuf::from("/tmp/lay-l11-limit-test.sock"),
            request_count: AtomicU64::new(0),
            host: Arc::new(RwLock::new(HostedMemory::Loading {
                package_path: PathBuf::from("/tmp/loading.v9.bin"),
            })),
        });

        for request in [
            lay::nanda_wave::L1ServiceRequest::Restore {
                surface: "слово".to_string(),
                limit: 0,
            },
            lay::nanda_wave::L1ServiceRequest::Lattice {
                surface: "слово".to_string(),
                limit: lay::nanda_wave::L11_LIVE_LATTICE_LIMIT + 1,
            },
        ] {
            assert!(matches!(
                handle_request(request, &state),
                lay::nanda_wave::L1ServiceResponse::Error { message }
                    if message.contains("live lattice limit")
            ));
        }
    }

    #[test]
    fn idle_persistent_connections_do_not_occupy_request_workers() {
        let state = Arc::new(ServiceState {
            started_at: Instant::now(),
            socket_path: PathBuf::from("/tmp/lay-l11-idle-test.sock"),
            request_count: AtomicU64::new(0),
            host: Arc::new(RwLock::new(HostedMemory::Loading {
                package_path: PathBuf::from("/tmp/loading.v9.bin"),
            })),
        });
        let (requests, receiver) = sync_channel(4);
        spawn_request_worker(31, Arc::new(Mutex::new(receiver)), Arc::clone(&state))
            .expect("spawn request worker");

        let mut idle_clients = Vec::new();
        let mut idle_handlers = Vec::new();
        for _ in 0..8 {
            let (client, server) = UnixStream::pair().expect("idle socket pair");
            let sender = requests.clone();
            idle_handlers.push(thread::spawn(move || handle_client(server, &sender)));
            idle_clients.push(client);
        }

        let (mut client, server) = UnixStream::pair().expect("health socket pair");
        client
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("health timeout");
        let sender = requests.clone();
        let health_handler = thread::spawn(move || handle_client(server, &sender));
        serde_json::to_writer(&mut client, &lay::nanda_wave::L1ServiceRequest::Health)
            .expect("write health request");
        client.write_all(b"\n").expect("health newline");
        client.flush().expect("flush health request");
        let mut reader = BufReader::new(client.try_clone().expect("clone health client"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("read health response");
        let response: lay::nanda_wave::L1ServiceResponse =
            serde_json::from_str(line.trim_end()).expect("decode health response");
        assert!(matches!(
            response,
            lay::nanda_wave::L1ServiceResponse::Health { .. }
        ));

        drop(reader);
        drop(client);
        health_handler
            .join()
            .expect("health handler")
            .expect("health connection");
        drop(idle_clients);
        for handler in idle_handlers {
            handler
                .join()
                .expect("idle handler")
                .expect("idle connection");
        }
        drop(requests);
    }
}
