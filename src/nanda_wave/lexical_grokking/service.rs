use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use super::runtime::L1RestorationHostStats;

const DEFAULT_SOCKET_NAME: &str = "lay-l11.sock";
const DEFAULT_MODEL_DIR_SUFFIX: &str = ".local/share/lay/nanda_wave/l1.1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledL11Package {
    pub package_id: String,
    pub receipt_path: PathBuf,
    pub artifact_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct L11SeedSurface {
    pub terminal_id: Option<u32>,
    pub surface: String,
    pub authority: bool,
    pub score_milli: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum L11ServiceEnsureReport {
    Ready {
        socket: PathBuf,
        package_path: PathBuf,
        status: String,
    },
    Reloaded {
        socket: PathBuf,
        package_path: PathBuf,
    },
    Spawned {
        binary: PathBuf,
        socket: PathBuf,
        package_path: PathBuf,
    },
}

#[derive(Clone, Debug, Deserialize)]
struct InstalledL11Receipt {
    package_id: String,
    installed_artifact: PathBuf,
    runtime_authority: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum L1ServiceRequest {
    Restore { surface: String, limit: usize },
    Lattice { surface: String, limit: usize },
    Decode { terminal_ids: Vec<u32> },
    Health,
    Stats,
    Reload { memory: PathBuf },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct L1ServiceHealth {
    pub status: String,
    pub message: Option<String>,
    pub socket: PathBuf,
    pub package_path: PathBuf,
    pub package_bytes: Option<usize>,
    pub terminal_count: Option<u32>,
    #[serde(default)]
    pub manifest_generation: u64,
    pub requests_served: u64,
    pub uptime_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct L1ServiceStats {
    pub status: String,
    pub message: Option<String>,
    pub socket: PathBuf,
    pub requests_served: u64,
    pub uptime_ms: u64,
    pub host: Option<L1RestorationHostStats>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum L1ServiceResponse {
    Restore { report: serde_json::Value },
    Lattice { seeds: Vec<L11SeedSurface> },
    Decode { surfaces: Vec<(u32, String)> },
    Health { report: L1ServiceHealth },
    Stats { report: L1ServiceStats },
    Reload { report: L1RestorationHostStats },
    Error { message: String },
}

struct PersistentL11LatticeClient {
    socket_path: PathBuf,
    reader: BufReader<UnixStream>,
}

impl PersistentL11LatticeClient {
    fn connect(socket_path: &Path, timeout: Duration) -> io::Result<Self> {
        let stream = UnixStream::connect(socket_path)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
        Ok(Self {
            socket_path: socket_path.to_path_buf(),
            reader: BufReader::new(stream),
        })
    }

    fn request_lattice(
        &mut self,
        surface: &str,
        limit: usize,
        timeout: Duration,
    ) -> io::Result<L1ServiceResponse> {
        {
            let stream = self.reader.get_mut();
            stream.set_read_timeout(Some(timeout))?;
            stream.set_write_timeout(Some(timeout))?;
            serde_json::to_writer(
                &mut *stream,
                &L1ServiceRequest::Lattice {
                    surface: surface.to_string(),
                    limit,
                },
            )
            .map_err(io::Error::other)?;
            stream.write_all(b"\n")?;
            stream.flush()?;
        }

        let mut line = String::new();
        if self.reader.read_line(&mut line)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "L1.1 service closed the persistent lattice connection",
            ));
        }
        serde_json::from_str(line.trim_end()).map_err(io::Error::other)
    }
}

static PERSISTENT_L11_LATTICE_CLIENT: OnceLock<Mutex<Option<PersistentL11LatticeClient>>> =
    OnceLock::new();

fn request_l11_lattice_persistent(
    socket_path: &Path,
    surface: &str,
    limit: usize,
    timeout: Duration,
) -> io::Result<L1ServiceResponse> {
    let slot = PERSISTENT_L11_LATTICE_CLIENT.get_or_init(|| Mutex::new(None));
    let mut slot = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    request_l11_lattice_with_slot(&mut slot, socket_path, surface, limit, timeout)
}

fn request_l11_lattice_with_slot(
    slot: &mut Option<PersistentL11LatticeClient>,
    socket_path: &Path,
    surface: &str,
    limit: usize,
    timeout: Duration,
) -> io::Result<L1ServiceResponse> {
    let mut last_error = None;
    for _attempt in 0..2 {
        if slot
            .as_ref()
            .is_none_or(|client| client.socket_path != socket_path)
        {
            *slot = Some(PersistentL11LatticeClient::connect(socket_path, timeout)?);
        }
        let result = slot
            .as_mut()
            .expect("persistent L1.1 lattice client initialized")
            .request_lattice(surface, limit, timeout);
        match result {
            Ok(response) => return Ok(response),
            Err(error) => {
                last_error = Some(error);
                *slot = None;
            }
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("L1.1 lattice request failed")))
}

pub fn default_l11_socket_path() -> PathBuf {
    if let Some(explicit) = env::var_os("LAY_L11_SOCKET") {
        return PathBuf::from(explicit);
    }
    if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join(DEFAULT_SOCKET_NAME);
    }
    let suffix = env::var("UID")
        .ok()
        .or_else(|| env::var("USER").ok())
        .unwrap_or_else(|| "default".to_string());
    env::temp_dir().join(format!("{DEFAULT_SOCKET_NAME}-{suffix}"))
}

pub fn default_l11_model_dir() -> PathBuf {
    if let Some(explicit) = env::var_os("LAY_L11_MODEL_DIR") {
        return PathBuf::from(explicit);
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(DEFAULT_MODEL_DIR_SUFFIX))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL_DIR_SUFFIX))
}

pub fn discover_installed_l11_package() -> io::Result<Option<InstalledL11Package>> {
    if let Some(explicit) = env::var_os("LAY_L11_PACKAGE") {
        let artifact_path = PathBuf::from(explicit);
        if !artifact_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "LAY_L11_PACKAGE points to a missing package: {}",
                    artifact_path.display()
                ),
            ));
        }
        let package_id = artifact_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "l11-explicit".to_string());
        return Ok(Some(InstalledL11Package {
            package_id,
            receipt_path: artifact_path.clone(),
            artifact_path,
        }));
    }

    let model_dir = default_l11_model_dir();
    if !model_dir.exists() {
        return Ok(None);
    }

    let mut installed = Vec::new();
    for entry in fs::read_dir(&model_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name() else {
            continue;
        };
        if !file_name.to_string_lossy().ends_with(".installed.json") {
            continue;
        }
        let receipt = match fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<InstalledL11Receipt>(&bytes).ok())
        {
            Some(receipt) => receipt,
            None => continue,
        };
        if receipt.runtime_authority || !receipt.installed_artifact.is_file() {
            continue;
        }
        let modified = path
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        installed.push((modified, path, receipt));
    }

    installed.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));

    Ok(installed
        .into_iter()
        .next()
        .map(|(_, receipt_path, receipt)| InstalledL11Package {
            package_id: receipt.package_id,
            receipt_path,
            artifact_path: receipt.installed_artifact,
        }))
}

pub fn send_l11_service_request(
    socket_path: &Path,
    request: &L1ServiceRequest,
) -> io::Result<L1ServiceResponse> {
    send_l11_service_request_with_timeout(socket_path, request, None)
}

pub fn send_l11_service_request_with_timeout(
    socket_path: &Path,
    request: &L1ServiceRequest,
    timeout: Option<Duration>,
) -> io::Result<L1ServiceResponse> {
    let mut stream = UnixStream::connect(socket_path)?;
    if let Some(timeout) = timeout {
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;
    }
    serde_json::to_writer(&mut stream, request).map_err(io::Error::other)?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "L1.1 service closed the socket without a response",
        ));
    }
    serde_json::from_str(line.trim_end()).map_err(io::Error::other)
}

pub fn authoritative_restore_surface(report: &serde_json::Value) -> Option<&str> {
    let result = report.get("result")?;
    result
        .get("authority")
        .and_then(serde_json::Value::as_bool)
        .filter(|authority| *authority)?;
    result.get("candidate")?.get("surface")?.as_str()
}

pub fn l11_seed_surfaces(report: &serde_json::Value, limit: usize) -> Vec<L11SeedSurface> {
    if limit == 0 {
        return Vec::new();
    }
    let Some(result) = report.get("result") else {
        return Vec::new();
    };
    let authority = result
        .get("authority")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let mut seeds = Vec::new();
    if let Some(candidate) = result.get("candidate") {
        push_l11_seed_surface(&mut seeds, candidate, authority);
    }
    if let Some(candidates) = result
        .get("candidates")
        .and_then(serde_json::Value::as_array)
    {
        for candidate in candidates {
            push_l11_seed_surface(&mut seeds, candidate, false);
        }
    }
    seeds.sort_by(|left, right| {
        right
            .score_milli
            .cmp(&left.score_milli)
            .then_with(|| right.authority.cmp(&left.authority))
            .then_with(|| left.surface.cmp(&right.surface))
    });
    let mut dedup = BTreeSet::new();
    seeds.retain(|candidate| dedup.insert(candidate.surface.to_lowercase()));
    seeds.truncate(limit);
    seeds
}

pub fn request_l11_authoritative_surface(
    socket_path: &Path,
    surface: &str,
    limit: usize,
    timeout: Duration,
) -> io::Result<Option<String>> {
    match send_l11_service_request_with_timeout(
        socket_path,
        &L1ServiceRequest::Restore {
            surface: surface.to_string(),
            limit,
        },
        Some(timeout),
    )? {
        L1ServiceResponse::Restore { report } => {
            Ok(authoritative_restore_surface(&report).map(str::to_string))
        }
        L1ServiceResponse::Error { .. } => Ok(None),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected L1.1 response for restore request: {other:?}"),
        )),
    }
}

pub fn request_l11_seed_surfaces(
    socket_path: &Path,
    surface: &str,
    limit: usize,
    timeout: Duration,
) -> io::Result<Vec<L11SeedSurface>> {
    match request_l11_lattice_persistent(socket_path, surface, limit, timeout)? {
        L1ServiceResponse::Lattice { mut seeds } => {
            seeds.truncate(limit);
            Ok(seeds)
        }
        L1ServiceResponse::Error { message } => Err(io::Error::other(format!(
            "L1.1 lattice request failed: {message}"
        ))),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected L1.1 response for restore request: {other:?}"),
        )),
    }
}

pub fn request_l11_decoded_surfaces(
    socket_path: &Path,
    terminal_ids: &[u32],
    timeout: Duration,
) -> io::Result<Vec<(u32, String)>> {
    match send_l11_service_request_with_timeout(
        socket_path,
        &L1ServiceRequest::Decode {
            terminal_ids: terminal_ids.to_vec(),
        },
        Some(timeout),
    )? {
        L1ServiceResponse::Decode { surfaces } => Ok(surfaces),
        L1ServiceResponse::Error { message } => Err(io::Error::other(format!(
            "L1.1 decode request failed: {message}"
        ))),
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected L1.1 response for decode request: {other:?}"),
        )),
    }
}

fn push_l11_seed_surface(
    seeds: &mut Vec<L11SeedSurface>,
    candidate: &serde_json::Value,
    authority: bool,
) {
    let Some(surface) = candidate.get("surface").and_then(serde_json::Value::as_str) else {
        return;
    };
    seeds.push(L11SeedSurface {
        terminal_id: candidate
            .get("terminal_id")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok()),
        surface: surface.to_string(),
        authority,
        score_milli: l11_seed_score(candidate),
    });
}

fn l11_seed_score(candidate: &serde_json::Value) -> u32 {
    let evidence = candidate.get("evidence").or(Some(candidate));
    let geometry_distance = evidence
        .and_then(|evidence| evidence.get("geometry_distance"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(8)
        .min(255);
    let positive = evidence_u64(evidence, "positive_milli");
    let backward = evidence_u64(evidence, "backward_milli");
    let anti = evidence_u64(evidence, "anti_milli");
    let hard_negative = evidence_u64(evidence, "hard_negative_milli");
    let ambiguity = evidence_u64(evidence, "ambiguity_milli");
    let crystallization_margin = evidence_u64(evidence, "crystallization_margin_milli");
    let geometry_bonus = 256_u64.saturating_sub(geometry_distance.saturating_mul(24));
    let score = positive
        .saturating_add(backward)
        .saturating_add(crystallization_margin)
        .saturating_add(geometry_bonus)
        .saturating_sub(anti)
        .saturating_sub(hard_negative)
        .saturating_sub(ambiguity / 2);
    score.min(u64::from(u32::MAX)) as u32
}

fn evidence_u64(evidence: Option<&serde_json::Value>, key: &str) -> u64 {
    evidence
        .and_then(|evidence| evidence.get(key))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

pub fn ensure_l11_service_started() -> io::Result<Option<L11ServiceEnsureReport>> {
    let Some(package) = discover_installed_l11_package()? else {
        return Ok(None);
    };
    let socket = default_l11_socket_path();

    if socket.exists() {
        match send_l11_service_request_with_timeout(
            &socket,
            &L1ServiceRequest::Health,
            Some(Duration::from_millis(50)),
        ) {
            Ok(L1ServiceResponse::Health { report }) => {
                let expected_generation =
                    super::composite::manifest_generation(&package.artifact_path)
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                let stale_generation =
                    expected_generation != 0 && report.manifest_generation != expected_generation;
                if report.status == "ready"
                    && (report.package_path != package.artifact_path || stale_generation)
                {
                    if let Ok(L1ServiceResponse::Reload { .. }) =
                        send_l11_service_request_with_timeout(
                            &socket,
                            &L1ServiceRequest::Reload {
                                memory: package.artifact_path.clone(),
                            },
                            Some(Duration::from_secs(5)),
                        )
                    {
                        return Ok(Some(L11ServiceEnsureReport::Reloaded {
                            socket,
                            package_path: package.artifact_path,
                        }));
                    }
                }
                if matches!(report.status.as_str(), "ready" | "warming") {
                    return Ok(Some(L11ServiceEnsureReport::Ready {
                        socket,
                        package_path: report.package_path,
                        status: report.status,
                    }));
                }
            }
            Ok(L1ServiceResponse::Error { .. }) | Err(_) => {}
            Ok(other) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unexpected L1.1 response for health request: {other:?}"),
                ));
            }
        }
    }

    let binary = spawn_l11_service_process(&package.artifact_path, &socket)?;
    Ok(Some(L11ServiceEnsureReport::Spawned {
        binary,
        socket,
        package_path: package.artifact_path,
    }))
}

fn spawn_l11_service_process(package_path: &Path, socket_path: &Path) -> io::Result<PathBuf> {
    let mut last_not_found = None;
    for candidate in l11_service_binary_candidates() {
        let mut command = Command::new(&candidate);
        command
            .arg("run")
            .arg("--memory")
            .arg(package_path)
            .arg("--socket")
            .arg(socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        match command.spawn() {
            Ok(_child) => return Ok(candidate),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                last_not_found = Some(error);
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_not_found.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "could not locate lay-l1.1-serve or lay-l11-serve",
        )
    }))
}

fn l11_service_binary_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    let mut push = |path: PathBuf| {
        if seen.insert(path.clone()) {
            candidates.push(path);
        }
    };

    if let Some(explicit) = env::var_os("LAY_L11_SERVICE_BIN") {
        push(PathBuf::from(explicit));
    }
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            push(parent.join("lay-l1.1-serve"));
            push(parent.join("lay-l11-serve"));
            if let Some(grandparent) = parent.parent() {
                push(grandparent.join("lay-l1.1-serve"));
                push(grandparent.join("lay-l11-serve"));
            }
        }
    }
    push(PathBuf::from("lay-l1.1-serve"));
    push(PathBuf::from("lay-l11-serve"));
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    #[test]
    fn default_socket_uses_runtime_dir_when_present() {
        let _lock = env_lock();
        let previous_runtime = env::var_os("XDG_RUNTIME_DIR");
        let previous_socket = env::var_os("LAY_L11_SOCKET");
        env::remove_var("LAY_L11_SOCKET");
        env::set_var("XDG_RUNTIME_DIR", "/tmp/lay-runtime");
        assert_eq!(
            default_l11_socket_path(),
            PathBuf::from("/tmp/lay-runtime").join(DEFAULT_SOCKET_NAME)
        );
        match previous_runtime {
            Some(value) => env::set_var("XDG_RUNTIME_DIR", value),
            None => env::remove_var("XDG_RUNTIME_DIR"),
        }
        match previous_socket {
            Some(value) => env::set_var("LAY_L11_SOCKET", value),
            None => env::remove_var("LAY_L11_SOCKET"),
        }
    }

    #[test]
    fn default_socket_uses_explicit_override_when_present() {
        let _lock = env_lock();
        let previous = env::var_os("LAY_L11_SOCKET");
        env::set_var("LAY_L11_SOCKET", "/tmp/lay-l11-explicit.sock");
        assert_eq!(
            default_l11_socket_path(),
            PathBuf::from("/tmp/lay-l11-explicit.sock")
        );
        match previous {
            Some(value) => env::set_var("LAY_L11_SOCKET", value),
            None => env::remove_var("LAY_L11_SOCKET"),
        }
    }

    #[test]
    fn request_roundtrip_preserves_reload_path() {
        let request = L1ServiceRequest::Reload {
            memory: PathBuf::from("/tmp/example.bin"),
        };
        let encoded = serde_json::to_string(&request).expect("serialize request");
        let decoded: L1ServiceRequest =
            serde_json::from_str(&encoded).expect("deserialize request");
        assert_eq!(decoded, request);
    }

    #[test]
    fn authoritative_surface_requires_authority_winner() {
        let winner = serde_json::json!({
            "result": {
                "verdict": "winner",
                "authority": true,
                "candidate": {
                    "surface": "время"
                }
            }
        });
        let tied = serde_json::json!({
            "result": {
                "verdict": "tied",
                "authority": false,
                "candidates": [
                    {"surface": "время"},
                    {"surface": "времена"}
                ]
            }
        });

        assert_eq!(authoritative_restore_surface(&winner), Some("время"));
        assert_eq!(authoritative_restore_surface(&tied), None);
    }

    #[test]
    fn discover_installed_package_prefers_newest_receipt() {
        let _lock = env_lock();
        let unique = format!(
            "lay-l11-service-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let root = env::temp_dir().join(unique);
        fs::create_dir_all(&root).expect("create temp model dir");
        let old_artifact = root.join("old.v7.bin");
        let new_artifact = root.join("new.v7.bin");
        fs::write(&old_artifact, b"old").expect("write old artifact");
        fs::write(&new_artifact, b"new").expect("write new artifact");
        fs::write(
            root.join("A.installed.json"),
            serde_json::json!({
                "package_id": "A",
                "installed_artifact": old_artifact,
                "runtime_authority": false
            })
            .to_string(),
        )
        .expect("write old receipt");
        std::thread::sleep(Duration::from_millis(5));
        fs::write(
            root.join("B.installed.json"),
            serde_json::json!({
                "package_id": "B",
                "installed_artifact": new_artifact,
                "runtime_authority": false
            })
            .to_string(),
        )
        .expect("write new receipt");

        let previous = env::var_os("LAY_L11_MODEL_DIR");
        env::set_var("LAY_L11_MODEL_DIR", &root);
        let discovered = discover_installed_l11_package()
            .expect("discover")
            .expect("installed package");
        assert_eq!(discovered.package_id, "B");
        match previous {
            Some(value) => env::set_var("LAY_L11_MODEL_DIR", value),
            None => env::remove_var("LAY_L11_MODEL_DIR"),
        }
        let _ = fs::remove_dir_all(root);
    }

    fn persistent_test_socket(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "lay-l11-{name}-{}-{}.sock",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ))
    }

    fn read_lattice_request(stream: &UnixStream) -> L1ServiceRequest {
        let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("read lattice request");
        serde_json::from_str(line.trim_end()).expect("decode lattice request")
    }

    fn write_lattice_response(stream: &mut UnixStream, surface: String) {
        serde_json::to_writer(
            &mut *stream,
            &L1ServiceResponse::Lattice {
                seeds: vec![L11SeedSurface {
                    terminal_id: Some(7),
                    surface,
                    authority: false,
                    score_milli: 1_000,
                }],
            },
        )
        .expect("encode lattice response");
        stream.write_all(b"\n").expect("write response newline");
        stream.flush().expect("flush lattice response");
    }

    #[test]
    fn persistent_lattice_client_reuses_one_accepted_connection() {
        let socket_path = persistent_test_socket("reuse");
        let listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind socket");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept connection");
            let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
            for _ in 0..2 {
                let mut line = String::new();
                reader.read_line(&mut line).expect("read lattice request");
                let request: L1ServiceRequest =
                    serde_json::from_str(line.trim_end()).expect("decode request");
                let L1ServiceRequest::Lattice { surface, .. } = request else {
                    panic!("persistent route accepted a mutating request");
                };
                write_lattice_response(&mut stream, surface);
            }
        });

        let mut slot = None;
        for surface in ["alpha", "beta"] {
            let response = request_l11_lattice_with_slot(
                &mut slot,
                &socket_path,
                surface,
                8,
                Duration::from_secs(1),
            )
            .expect("persistent lattice request");
            let L1ServiceResponse::Lattice { seeds } = response else {
                panic!("unexpected response");
            };
            assert_eq!(seeds[0].surface, surface);
        }
        server.join().expect("server thread");
        let _ = fs::remove_file(socket_path);
    }

    #[test]
    fn persistent_lattice_client_reconnects_once_after_transport_failure() {
        let socket_path = persistent_test_socket("reconnect");
        let listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind socket");
        let server = std::thread::spawn(move || {
            let (first, _) = listener.accept().expect("accept failed connection");
            assert!(matches!(
                read_lattice_request(&first),
                L1ServiceRequest::Lattice { .. }
            ));
            drop(first);

            let (mut second, _) = listener.accept().expect("accept retry connection");
            let L1ServiceRequest::Lattice { surface, .. } = read_lattice_request(&second) else {
                panic!("retry route accepted a mutating request");
            };
            write_lattice_response(&mut second, surface);
        });

        let mut slot = None;
        let response = request_l11_lattice_with_slot(
            &mut slot,
            &socket_path,
            "gamma",
            8,
            Duration::from_secs(1),
        )
        .expect("single reconnect");
        let L1ServiceResponse::Lattice { seeds } = response else {
            panic!("unexpected response");
        };
        assert_eq!(seeds[0].surface, "gamma");
        server.join().expect("server thread");
        let _ = fs::remove_file(socket_path);
    }
}
