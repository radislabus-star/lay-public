use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

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
    let (sender, receiver) = sync_channel::<UnixStream>(worker_count.saturating_mul(4));
    let receiver = Arc::new(Mutex::new(receiver));
    for worker_id in 0..worker_count {
        spawn_service_worker(worker_id, Arc::clone(&receiver), Arc::clone(&state))?;
    }
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if sender.send(stream).is_err() {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "L1.1 service worker pool stopped",
                    ));
                }
            }
            Err(error) => eprintln!("lay-l1.1-serve accept failed: {error}"),
        }
    }
    Ok(())
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

fn spawn_service_worker(
    worker_id: usize,
    receiver: Arc<Mutex<Receiver<UnixStream>>>,
    state: Arc<ServiceState>,
) -> io::Result<()> {
    thread::Builder::new()
        .name(format!("lay-l11-{worker_id}"))
        .spawn(move || loop {
            let stream = {
                let receiver = receiver.lock().expect("L1.1 service receiver lock");
                receiver.recv()
            };
            let Ok(stream) = stream else {
                break;
            };
            if let Err(error) = handle_client(stream, &state) {
                eprintln!("lay-l1.1-serve client error: {error}");
            }
        })
        .map(|_| ())
}

fn spawn_background_load(host: Arc<RwLock<HostedMemory>>, package_path: PathBuf) {
    thread::spawn(move || {
        let next = match lay::nanda_wave::L1RestorationHost::load(&package_path) {
            Ok(loaded) => HostedMemory::Ready(loaded),
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

fn handle_client(stream: UnixStream, state: &Arc<ServiceState>) -> io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<lay::nanda_wave::L1ServiceRequest>(&line) {
            Ok(request) => handle_request(request, state),
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

fn handle_request(
    request: lay::nanda_wave::L1ServiceRequest,
    state: &Arc<ServiceState>,
) -> lay::nanda_wave::L1ServiceResponse {
    state.request_count.fetch_add(1, Ordering::Relaxed);
    match request {
        lay::nanda_wave::L1ServiceRequest::Restore { surface, limit } => {
            let host = state.host.read().expect("L1.1 host read lock");
            match &*host {
                HostedMemory::Ready(host) => lay::nanda_wave::L1ServiceResponse::Restore {
                    report: host.restore(&surface, limit),
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
        lay::nanda_wave::L1ServiceRequest::Lattice { surface, limit } => {
            let host = state.host.read().expect("L1.1 host read lock");
            match &*host {
                HostedMemory::Ready(host) => lay::nanda_wave::L1ServiceResponse::Lattice {
                    seeds: host
                        .lattice_seed_rows(&surface, limit)
                        .into_iter()
                        .map(|(terminal_id, surface, score_milli)| {
                            lay::nanda_wave::L11SeedSurface {
                                terminal_id: Some(terminal_id),
                                surface,
                                authority: false,
                                score_milli,
                            }
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
            // Keep serving the current immutable snapshot while the replacement
            // is loaded and validated. The write lock protects only the flip.
            match load_validate_replace(
                &state.host,
                || {
                    lay::nanda_wave::L1RestorationHost::load(&memory)
                        .map(|next| {
                            let report = next.stats();
                            (HostedMemory::Ready(next), report)
                        })
                        .map_err(|error| error.to_string())
                },
                |current, next| {
                    let (HostedMemory::Ready(current), HostedMemory::Ready(next)) = (current, next)
                    else {
                        return Ok(());
                    };
                    let current = current.stats();
                    let next = next.stats();
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
}
