use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock, TryLockError};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::runtime::L1RestorationHostStats;

const DEFAULT_SOCKET_NAME: &str = "lay-l11.sock";
const DEFAULT_MODEL_DIR_SUFFIX: &str = ".local/share/lay/nanda_wave/l1.1";
const ACTIVE_RECEIPT_NAME: &str = "active.installed.json";
const INSTALLED_RECEIPT_SCHEMA: &str = "lay.l11.installed-package.v1";
const QUALITY_PROOF_SCHEMA: &str = "lay.l11.typed-basin-quality-proof.v3";
const FULL_HELDOUT_PER_CLASS: u64 = 20_000;
const FIXED_DAMAGE_CLASS_COUNT: u64 = super::proof_matrix::DAMAGE_CLASSES.len() as u64;
const FULL_DAMAGED_CASES: u64 = FULL_HELDOUT_PER_CLASS * FIXED_DAMAGE_CLASS_COUNT;
const REQUIRED_QUALITY_GATES: &[&str] = &[
    "artifact_prerequisite_pass",
    "direct_v9_artifact",
    "v9_checksum_valid",
    "stored_exact_support_matches_rebuild",
    "package_dependencies_resolved",
    "package_isolation",
    "fixed_damaged_denominator_complete",
    "clean_denominator_complete",
    "full_fixed_denominator",
    "target_retention_complete",
    "unique_top1_every_class_strictly_gt_95_percent",
    "lattice_coverage_every_class_ge_99_percent",
    "clean_preservation_ge_99_9_percent",
    "false_authority_zero",
    "false_singleton_zero",
    "grounded_legacy_candidate_loss_zero",
    "conjunctive_full_quality_pass",
];

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
    schema: String,
    package_id: String,
    format: String,
    installed_artifact: PathBuf,
    artifact_bytes: u64,
    artifact_sha256: String,
    proof_receipt: PathBuf,
    proof_sha256: String,
    proof_verdict: String,
    runtime_authority: bool,
    runtime_admitted: bool,
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
    match slot.try_lock() {
        Ok(mut slot) => {
            request_l11_lattice_with_slot(&mut slot, socket_path, surface, limit, timeout)
        }
        Err(TryLockError::Poisoned(poisoned)) => {
            let mut slot = poisoned.into_inner();
            request_l11_lattice_with_slot(&mut slot, socket_path, surface, limit, timeout)
        }
        Err(TryLockError::WouldBlock) => send_l11_service_request_with_timeout(
            socket_path,
            &L1ServiceRequest::Lattice {
                surface: surface.to_string(),
                limit,
            },
            Some(timeout),
        ),
    }
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
    if env::var_os("LAY_L11_PACKAGE").is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "LAY_L11_PACKAGE direct artifact loading is not admitted; use a validated LAY_L11_RECEIPT",
        ));
    }

    let explicit_receipt = env::var_os("LAY_L11_RECEIPT").map(PathBuf::from);
    let receipt_path = explicit_receipt
        .clone()
        .unwrap_or_else(|| default_l11_model_dir().join(ACTIVE_RECEIPT_NAME));
    if !receipt_path.exists() && explicit_receipt.is_none() {
        return Ok(None);
    }
    validate_installed_l11_receipt(&receipt_path).map(Some)
}

pub fn admit_l11_service_artifact(artifact_path: &Path) -> io::Result<InstalledL11Package> {
    let package = discover_installed_l11_package()?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "no active L1.1 admission receipt is installed",
        )
    })?;
    if package.artifact_path != artifact_path {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "L1.1 reload artifact is not the active admitted package: {}",
                artifact_path.display()
            ),
        ));
    }
    Ok(package)
}

fn validate_installed_l11_receipt(receipt_path: &Path) -> io::Result<InstalledL11Package> {
    let receipt_bytes = fs::read(receipt_path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "cannot read L1.1 receipt {}: {error}",
                receipt_path.display()
            ),
        )
    })?;
    let receipt: InstalledL11Receipt = serde_json::from_slice(&receipt_bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid L1.1 receipt {}: {error}", receipt_path.display()),
        )
    })?;
    if receipt.schema != INSTALLED_RECEIPT_SCHEMA
        || receipt.package_id.trim().is_empty()
        || !receipt
            .package_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-'))
        || !(receipt.format.starts_with("V7")
            || receipt.format.starts_with("V8")
            || receipt.format.starts_with("V9"))
        || receipt.runtime_authority
        || !receipt.runtime_admitted
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("L1.1 receipt is not admitted: {}", receipt_path.display()),
        ));
    }
    if !receipt.installed_artifact.is_absolute() || !receipt.proof_receipt.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1.1 receipt artifact and proof paths must be absolute",
        ));
    }
    let artifact_bytes = fs::metadata(&receipt.installed_artifact)?.len();
    if artifact_bytes != receipt.artifact_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "L1.1 artifact size mismatch: receipt={} actual={artifact_bytes}",
                receipt.artifact_bytes
            ),
        ));
    }
    let actual_sha256 = streaming_sha256(&receipt.installed_artifact)?;
    if !valid_sha256(&receipt.artifact_sha256)
        || !actual_sha256.eq_ignore_ascii_case(&receipt.artifact_sha256)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1.1 artifact SHA-256 does not match its receipt",
        ));
    }
    let proof_bytes = fs::read(&receipt.proof_receipt)?;
    let proof = serde_json::from_slice::<serde_json::Value>(&proof_bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid L1.1 proof receipt: {error}"),
        )
    })?;
    let actual_proof_sha256 = sha256_bytes(&proof_bytes);
    if !valid_sha256(&receipt.proof_sha256)
        || !actual_proof_sha256.eq_ignore_ascii_case(&receipt.proof_sha256)
        || !admitted_proof_verdict(&receipt.proof_verdict)
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "L1.1 proof receipt hash or admitted verdict is invalid",
        ));
    }
    validate_l11_proof_claims(&proof, &receipt)?;
    Ok(InstalledL11Package {
        package_id: receipt.package_id,
        receipt_path: receipt_path.to_path_buf(),
        artifact_path: receipt.installed_artifact,
    })
}

fn streaming_sha256(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn admitted_proof_verdict(value: &str) -> bool {
    matches!(value, "PASS_shadow" | "PASS_C_QUALITY")
}

fn validate_l11_proof_claims(
    proof: &serde_json::Value,
    receipt: &InstalledL11Receipt,
) -> io::Result<()> {
    if receipt.format.starts_with("V9") && receipt.proof_verdict != "PASS_C_QUALITY" {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "L1.1 V9 packages require a full PASS_C_QUALITY proof",
        ));
    }
    let admitted = match receipt.proof_verdict.as_str() {
        "PASS_C_QUALITY" => {
            proof.get("schema").and_then(serde_json::Value::as_str) == Some(QUALITY_PROOF_SCHEMA)
                && proof.get("verdict").and_then(serde_json::Value::as_str)
                    == Some("PASS_C_QUALITY")
                && proof
                    .pointer("/artifact/format")
                    .and_then(serde_json::Value::as_str)
                    == Some("V9")
                && proof
                    .pointer("/artifact/package_bytes")
                    .and_then(serde_json::Value::as_u64)
                    == Some(receipt.artifact_bytes)
                && proof
                    .pointer("/artifact/package_sha256_before")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|digest| digest.eq_ignore_ascii_case(&receipt.artifact_sha256))
                && proof
                    .pointer("/artifact/package_sha256_after")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|digest| digest.eq_ignore_ascii_case(&receipt.artifact_sha256))
                && proof
                    .pointer("/artifact/package_bytes_unchanged")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                && proof
                    .pointer("/configuration/lattice_projection_limit")
                    .and_then(serde_json::Value::as_u64)
                    == Some(super::L11_LIVE_LATTICE_LIMIT as u64)
                && complete_quality_proof_contract(proof)
                && proof
                    .pointer("/claim_boundary/full_quality_matrix_tested")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                && proof
                    .pointer("/claim_boundary/full_quality_claimed")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
        }
        "PASS_shadow" => {
            proof
                .pointer("/fixed_proof/verdict")
                .and_then(serde_json::Value::as_str)
                == Some("PASS_shadow")
                && proof
                    .pointer("/package/bytes")
                    .and_then(serde_json::Value::as_u64)
                    == Some(receipt.artifact_bytes)
                && proof
                    .pointer("/package/sha256")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|digest| digest.eq_ignore_ascii_case(&receipt.artifact_sha256))
        }
        _ => false,
    };
    if admitted {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "L1.1 proof claims do not admit artifact {} under verdict {}",
                receipt.installed_artifact.display(),
                receipt.proof_verdict
            ),
        ))
    }
}

fn complete_quality_proof_contract(proof: &serde_json::Value) -> bool {
    let configuration = |field: &str| {
        proof
            .pointer(&format!("/configuration/{field}"))
            .and_then(serde_json::Value::as_u64)
    };
    if configuration("heldout_per_class") != Some(FULL_HELDOUT_PER_CLASS)
        || configuration("fixed_damage_classes") != Some(FIXED_DAMAGE_CLASS_COUNT)
        || configuration("selected_damage_classes") != Some(FIXED_DAMAGE_CLASS_COUNT)
        || configuration("expected_damaged_cases") != Some(FULL_DAMAGED_CASES)
        || configuration("clean_limit") != Some(0)
        || !proof
            .pointer("/configuration/damage_class_filter")
            .is_some_and(serde_json::Value::is_null)
        || REQUIRED_QUALITY_GATES.iter().any(|gate| {
            proof
                .pointer(&format!("/gates/{gate}"))
                .and_then(serde_json::Value::as_bool)
                != Some(true)
        })
    {
        return false;
    }

    let Some(primary_centers) = proof
        .pointer("/artifact/primary_centers")
        .and_then(serde_json::Value::as_u64)
        .filter(|count| *count > 0)
    else {
        return false;
    };
    let Some(classes) = proof
        .pointer("/damaged_quality/classes")
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    if classes.len() as u64 != FIXED_DAMAGE_CLASS_COUNT
        || !super::proof_matrix::DAMAGE_CLASSES
            .iter()
            .all(|class| classes.get(*class).is_some_and(complete_quality_class))
    {
        return false;
    }

    proof_u64(proof, "/damaged_quality/aggregate/cases") == Some(FULL_DAMAGED_CASES)
        && proof_u64(
            proof,
            "/damaged_quality/aggregate/target_retained_complete_field",
        ) == Some(FULL_DAMAGED_CASES)
        && proof_u64(proof, "/damaged_quality/aggregate/false_authority") == Some(0)
        && proof_u64(proof, "/damaged_quality/aggregate/false_singleton") == Some(0)
        && proof_u64(
            proof,
            "/damaged_quality/aggregate/runtime_observer/grounded_candidate_losses_from_exact_field",
        ) == Some(0)
        && proof_u64(proof, "/clean_quality/cases") == Some(primary_centers)
        && proof_u64(proof, "/clean_quality/preserved").is_some_and(|preserved| {
            preserved <= primary_centers
                && ratio_at_least_u64(preserved, primary_centers, 999, 1_000)
        })
}

fn complete_quality_class(class: &serde_json::Value) -> bool {
    let cases = proof_u64(class, "/cases");
    let objective_cases = proof_u64(class, "/objective_unique_cases");
    let bounded = proof_u64(class, "/target_in_bounded_lattice");
    let top1 = proof_u64(class, "/unique_top1");
    cases == Some(FULL_HELDOUT_PER_CLASS)
        && proof_u64(class, "/target_retained_complete_field") == cases
        && bounded.zip(cases).is_some_and(|(retained, total)| {
            retained <= total && ratio_at_least_u64(retained, total, 99, 100)
        })
        && top1.zip(objective_cases).is_some_and(|(top1, total)| {
            cases.is_some_and(|cases| total <= cases)
                && top1 <= total
                && ratio_strictly_above_u64(top1, total, 95, 100)
        })
        && proof_u64(class, "/false_authority") == Some(0)
        && proof_u64(class, "/false_singleton") == Some(0)
        && class
            .pointer("/gates/unique_top1_strictly_gt_95_percent")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        && class
            .pointer("/gates/lattice_coverage_ge_99_percent")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
}

fn proof_u64(value: &serde_json::Value, pointer: &str) -> Option<u64> {
    value.pointer(pointer).and_then(serde_json::Value::as_u64)
}

fn ratio_at_least_u64(
    numerator: u64,
    denominator: u64,
    required_numerator: u64,
    required_denominator: u64,
) -> bool {
    denominator != 0
        && u128::from(numerator) * u128::from(required_denominator)
            >= u128::from(denominator) * u128::from(required_numerator)
}

fn ratio_strictly_above_u64(
    numerator: u64,
    denominator: u64,
    required_numerator: u64,
    required_denominator: u64,
) -> bool {
    denominator != 0
        && u128::from(numerator) * u128::from(required_denominator)
            > u128::from(denominator) * u128::from(required_numerator)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
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
        L1ServiceResponse::Error { message } => Err(io::Error::other(format!(
            "L1.1 restore request failed: {message}"
        ))),
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
                let package_mismatch = report.package_path != package.artifact_path;
                match report.status.as_str() {
                    "ready" if package_mismatch || stale_generation => {
                        let response = send_l11_service_request(
                            &socket,
                            &L1ServiceRequest::Reload {
                                memory: package.artifact_path.clone(),
                            },
                        )?;
                        match response {
                            L1ServiceResponse::Reload { report }
                                if report.package_path == package.artifact_path
                                    && (expected_generation == 0
                                        || report.manifest_generation == expected_generation) =>
                            {
                                return Ok(Some(L11ServiceEnsureReport::Reloaded {
                                    socket,
                                    package_path: package.artifact_path,
                                }));
                            }
                            L1ServiceResponse::Reload { report } => {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!(
                                        "L1.1 reload returned the wrong package snapshot: {}",
                                        report.package_path.display()
                                    ),
                                ));
                            }
                            L1ServiceResponse::Error { message } => {
                                return Err(io::Error::other(format!(
                                    "L1.1 reload failed: {message}"
                                )));
                            }
                            other => {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("unexpected L1.1 reload response: {other:?}"),
                                ));
                            }
                        }
                    }
                    "ready" => {
                        return Ok(Some(L11ServiceEnsureReport::Ready {
                            socket,
                            package_path: report.package_path,
                            status: report.status,
                        }));
                    }
                    "warming" if !package_mismatch && !stale_generation => {
                        return Ok(Some(L11ServiceEnsureReport::Ready {
                            socket,
                            package_path: report.package_path,
                            status: report.status,
                        }));
                    }
                    "warming" => {
                        return Err(io::Error::other(
                            "L1.1 service is warming a package other than the active admission",
                        ));
                    }
                    status => {
                        return Err(io::Error::other(format!(
                            "L1.1 service health is {status}: {}",
                            report.message.as_deref().unwrap_or("no detail")
                        )));
                    }
                }
            }
            Ok(L1ServiceResponse::Error { message }) => {
                return Err(io::Error::other(format!(
                    "L1.1 health request failed: {message}"
                )));
            }
            Err(_) => {}
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
            .stderr(Stdio::inherit());
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
    use std::time::SystemTime;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    fn quality_proof(package_bytes: u64, package_sha256: &str) -> serde_json::Value {
        let class = serde_json::json!({
            "cases": FULL_HELDOUT_PER_CLASS,
            "objective_unique_cases": FULL_HELDOUT_PER_CLASS,
            "target_retained_complete_field": FULL_HELDOUT_PER_CLASS,
            "target_in_bounded_lattice": 19_800,
            "unique_top1": 19_200,
            "false_authority": 0,
            "false_singleton": 0,
            "gates": {
                "unique_top1_strictly_gt_95_percent": true,
                "lattice_coverage_ge_99_percent": true
            }
        });
        let classes = super::super::proof_matrix::DAMAGE_CLASSES
            .iter()
            .map(|name| ((*name).to_string(), class.clone()))
            .collect::<serde_json::Map<_, _>>();
        serde_json::json!({
            "schema": QUALITY_PROOF_SCHEMA,
            "verdict": "PASS_C_QUALITY",
            "artifact": {
                "format": "V9",
                "package_bytes": package_bytes,
                "package_sha256_before": package_sha256,
                "package_sha256_after": package_sha256,
                "package_bytes_unchanged": true,
                "primary_centers": 1_000
            },
            "configuration": {
                "heldout_per_class": FULL_HELDOUT_PER_CLASS,
                "fixed_damage_classes": FIXED_DAMAGE_CLASS_COUNT,
                "selected_damage_classes": FIXED_DAMAGE_CLASS_COUNT,
                "damage_class_filter": null,
                "expected_damaged_cases": FULL_DAMAGED_CASES,
                "clean_limit": 0,
                "lattice_projection_limit": super::super::L11_LIVE_LATTICE_LIMIT
            },
            "damaged_quality": {
                "classes": classes,
                "aggregate": {
                    "cases": FULL_DAMAGED_CASES,
                    "target_retained_complete_field": FULL_DAMAGED_CASES,
                    "false_authority": 0,
                    "false_singleton": 0,
                    "runtime_observer": {
                        "grounded_candidate_losses_from_exact_field": 0
                    }
                }
            },
            "clean_quality": {
                "cases": 1_000,
                "preserved": 999
            },
            "gates": {
                "artifact_prerequisite_pass": true,
                "direct_v9_artifact": true,
                "v9_checksum_valid": true,
                "stored_exact_support_matches_rebuild": true,
                "package_dependencies_resolved": true,
                "package_isolation": true,
                "fixed_damaged_denominator_complete": true,
                "clean_denominator_complete": true,
                "full_fixed_denominator": true,
                "target_retention_complete": true,
                "unique_top1_every_class_strictly_gt_95_percent": true,
                "lattice_coverage_every_class_ge_99_percent": true,
                "clean_preservation_ge_99_9_percent": true,
                "false_authority_zero": true,
                "false_singleton_zero": true,
                "grounded_legacy_candidate_loss_zero": true,
                "conjunctive_full_quality_pass": true
            },
            "claim_boundary": {
                "full_quality_matrix_tested": true,
                "full_quality_claimed": true
            }
        })
    }

    fn write_quality_proof(path: &Path, package_bytes: u64, package_sha256: &str) -> Vec<u8> {
        let bytes = serde_json::to_vec(&quality_proof(package_bytes, package_sha256))
            .expect("encode quality proof");
        fs::write(path, &bytes).expect("write quality proof");
        bytes
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
    fn discover_installed_package_uses_only_the_integrity_bound_active_receipt() {
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
        let artifact = root.join("active.v9.bin");
        let proof = root.join("proof.json");
        fs::write(&artifact, b"verified artifact").expect("write artifact");
        let artifact_sha256 = streaming_sha256(&artifact).expect("hash artifact");
        let proof_bytes = write_quality_proof(&proof, 17, &artifact_sha256);
        fs::write(
            root.join("newer-but-not-active.installed.json"),
            serde_json::json!({
                "package_id": "unselected",
                "installed_artifact": root.join("missing.bin"),
                "runtime_authority": false
            })
            .to_string(),
        )
        .expect("write unselected receipt");
        fs::write(
            root.join(ACTIVE_RECEIPT_NAME),
            serde_json::json!({
                "schema": INSTALLED_RECEIPT_SCHEMA,
                "package_id": "active",
                "format": "V9 test",
                "installed_artifact": &artifact,
                "artifact_bytes": 17,
                "artifact_sha256": artifact_sha256,
                "proof_receipt": &proof,
                "proof_sha256": sha256_bytes(&proof_bytes),
                "proof_verdict": "PASS_C_QUALITY",
                "runtime_authority": false,
                "runtime_admitted": true
            })
            .to_string(),
        )
        .expect("write active receipt");

        let previous = env::var_os("LAY_L11_MODEL_DIR");
        let previous_receipt = env::var_os("LAY_L11_RECEIPT");
        let previous_package = env::var_os("LAY_L11_PACKAGE");
        env::remove_var("LAY_L11_RECEIPT");
        env::remove_var("LAY_L11_PACKAGE");
        env::set_var("LAY_L11_MODEL_DIR", &root);
        let discovered = discover_installed_l11_package()
            .expect("discover")
            .expect("installed package");
        assert_eq!(discovered.package_id, "active");
        assert_eq!(discovered.receipt_path, root.join(ACTIVE_RECEIPT_NAME));
        assert!(admit_l11_service_artifact(&root.join("active.v9.bin")).is_ok());
        assert_eq!(
            admit_l11_service_artifact(&root.join("unselected.v9.bin"))
                .expect_err("reload must use the active admitted artifact")
                .kind(),
            io::ErrorKind::PermissionDenied
        );

        let replacement = root.join("replacement.v9.bin");
        let replacement_proof = root.join("replacement.proof.json");
        fs::write(&replacement, b"replacement artifact").expect("write replacement artifact");
        let replacement_sha256 = streaming_sha256(&replacement).expect("hash replacement");
        let replacement_proof_bytes =
            write_quality_proof(&replacement_proof, 20, &replacement_sha256);
        fs::write(
            root.join(ACTIVE_RECEIPT_NAME),
            serde_json::json!({
                "schema": INSTALLED_RECEIPT_SCHEMA,
                "package_id": "replacement",
                "format": "V9 test",
                "installed_artifact": &replacement,
                "artifact_bytes": 20,
                "artifact_sha256": replacement_sha256,
                "proof_receipt": &replacement_proof,
                "proof_sha256": sha256_bytes(&replacement_proof_bytes),
                "proof_verdict": "PASS_C_QUALITY",
                "runtime_authority": false,
                "runtime_admitted": true
            })
            .to_string(),
        )
        .expect("replace active receipt");
        assert_eq!(
            admit_l11_service_artifact(&root.join("active.v9.bin"))
                .expect_err("the previous artifact must stop being admitted immediately")
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert!(admit_l11_service_artifact(&root.join("replacement.v9.bin")).is_ok());
        match previous {
            Some(value) => env::set_var("LAY_L11_MODEL_DIR", value),
            None => env::remove_var("LAY_L11_MODEL_DIR"),
        }
        match previous_receipt {
            Some(value) => env::set_var("LAY_L11_RECEIPT", value),
            None => env::remove_var("LAY_L11_RECEIPT"),
        }
        match previous_package {
            Some(value) => env::set_var("LAY_L11_PACKAGE", value),
            None => env::remove_var("LAY_L11_PACKAGE"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn direct_package_environment_override_is_rejected() {
        let _lock = env_lock();
        let previous_package = env::var_os("LAY_L11_PACKAGE");
        let previous_receipt = env::var_os("LAY_L11_RECEIPT");
        env::set_var("LAY_L11_PACKAGE", "/tmp/unadmitted-l11.bin");
        env::remove_var("LAY_L11_RECEIPT");

        let error = discover_installed_l11_package()
            .expect_err("direct package path must not bypass receipt admission");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("LAY_L11_PACKAGE"));

        match previous_package {
            Some(value) => env::set_var("LAY_L11_PACKAGE", value),
            None => env::remove_var("LAY_L11_PACKAGE"),
        }
        match previous_receipt {
            Some(value) => env::set_var("LAY_L11_RECEIPT", value),
            None => env::remove_var("LAY_L11_RECEIPT"),
        }
    }

    #[test]
    fn installed_receipt_rejects_missing_admission_or_identity_mismatch() {
        let unique = format!("lay-l11-receipt-reject-{}", std::process::id());
        let root = env::temp_dir().join(unique);
        fs::create_dir_all(&root).expect("create receipt fixture");
        let artifact = root.join("artifact.v9.bin");
        let proof = root.join("proof.json");
        fs::write(&artifact, b"artifact").expect("write artifact");
        let artifact_sha256 = streaming_sha256(&artifact).expect("hash artifact");
        let proof_bytes = write_quality_proof(&proof, 8, &artifact_sha256);
        let valid = serde_json::json!({
            "schema": INSTALLED_RECEIPT_SCHEMA,
            "package_id": "fixture",
            "format": "V9 test",
            "installed_artifact": &artifact,
            "artifact_bytes": 8,
            "artifact_sha256": artifact_sha256,
            "proof_receipt": &proof,
            "proof_sha256": sha256_bytes(&proof_bytes),
            "proof_verdict": "PASS_C_QUALITY",
            "runtime_authority": false,
            "runtime_admitted": true
        });
        let receipt_path = root.join(ACTIVE_RECEIPT_NAME);

        for (field, value) in [
            ("runtime_admitted", serde_json::json!(false)),
            ("artifact_bytes", serde_json::json!(9)),
            ("artifact_sha256", serde_json::json!("0".repeat(64))),
            ("proof_sha256", serde_json::json!("0".repeat(64))),
            ("proof_verdict", serde_json::json!("REJECT_other")),
            ("proof_verdict", serde_json::json!("PASS_C_SMOKE")),
            ("proof_verdict", serde_json::json!("PASSPORT")),
            (
                "proof_verdict",
                serde_json::json!("PASS_SHADOW_RUNTIME_MODEL_AUTHORITY_WATCH"),
            ),
        ] {
            let mut rejected = valid.clone();
            rejected[field] = value;
            fs::write(&receipt_path, rejected.to_string()).expect("write rejected receipt");
            assert!(
                validate_installed_l11_receipt(&receipt_path).is_err(),
                "{field}"
            );
        }

        fs::write(&receipt_path, valid.to_string()).expect("write valid receipt");
        assert!(validate_installed_l11_receipt(&receipt_path).is_ok());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn proof_verdict_admission_is_an_exact_protocol_enum() {
        assert!(admitted_proof_verdict("PASS_shadow"));
        assert!(admitted_proof_verdict("PASS_C_QUALITY"));
        for rejected in [
            "PASS_C_SMOKE",
            "PASS_SHADOW_RUNTIME_MODEL_AUTHORITY_WATCH",
            "PASSPORT",
            "PASS",
            "REJECT_C_QUALITY",
        ] {
            assert!(!admitted_proof_verdict(rejected), "{rejected}");
        }
    }

    #[test]
    fn installer_fixed_damage_classes_match_runtime_protocol() {
        let installer = include_str!("../../../scripts/install-l11-shadow-package.sh");
        let block = installer
            .split("# L11_FIXED_DAMAGE_CLASSES_BEGIN")
            .nth(1)
            .expect("installer class protocol start")
            .split("# L11_FIXED_DAMAGE_CLASSES_END")
            .next()
            .expect("installer class protocol end");
        let classes = block
            .lines()
            .filter_map(|line| {
                line.trim()
                    .strip_prefix('"')
                    .and_then(|line| line.strip_suffix("\","))
            })
            .collect::<Vec<_>>();

        assert_eq!(classes, super::super::proof_matrix::DAMAGE_CLASSES);
    }

    #[test]
    fn quality_proof_admission_rejects_every_missing_bound_claim() {
        let artifact_sha256 = "a".repeat(64);
        let receipt = InstalledL11Receipt {
            schema: INSTALLED_RECEIPT_SCHEMA.to_string(),
            package_id: "fixture".to_string(),
            format: "V9 test".to_string(),
            installed_artifact: PathBuf::from("/tmp/fixture.v9.bin"),
            artifact_bytes: 77,
            artifact_sha256: artifact_sha256.clone(),
            proof_receipt: PathBuf::from("/tmp/fixture.proof.json"),
            proof_sha256: "b".repeat(64),
            proof_verdict: "PASS_C_QUALITY".to_string(),
            runtime_authority: false,
            runtime_admitted: true,
        };
        let valid = quality_proof(receipt.artifact_bytes, &artifact_sha256);
        validate_l11_proof_claims(&valid, &receipt).expect("valid quality proof");

        for (pointer, value) in [
            (
                "/schema",
                serde_json::json!("lay.l11.typed-basin-quality-proof.v2"),
            ),
            ("/verdict", serde_json::json!("REJECT_C_QUALITY")),
            ("/artifact/format", serde_json::json!("V8")),
            ("/artifact/package_bytes", serde_json::json!(76)),
            (
                "/artifact/package_sha256_before",
                serde_json::json!("c".repeat(64)),
            ),
            (
                "/artifact/package_sha256_after",
                serde_json::json!("c".repeat(64)),
            ),
            (
                "/artifact/package_bytes_unchanged",
                serde_json::json!(false),
            ),
            (
                "/configuration/lattice_projection_limit",
                serde_json::json!(super::super::L11_LIVE_LATTICE_LIMIT + 1),
            ),
            (
                "/configuration/heldout_per_class",
                serde_json::json!(FULL_HELDOUT_PER_CLASS - 1),
            ),
            (
                "/configuration/selected_damage_classes",
                serde_json::json!(FIXED_DAMAGE_CLASS_COUNT - 1),
            ),
            (
                "/configuration/fixed_damage_classes",
                serde_json::json!(FIXED_DAMAGE_CLASS_COUNT - 1),
            ),
            (
                "/configuration/expected_damaged_cases",
                serde_json::json!(FULL_DAMAGED_CASES - 1),
            ),
            ("/configuration/clean_limit", serde_json::json!(1)),
            (
                "/damaged_quality/aggregate/false_authority",
                serde_json::json!(1),
            ),
            (
                "/damaged_quality/aggregate/false_singleton",
                serde_json::json!(1),
            ),
            (
                "/damaged_quality/aggregate/target_retained_complete_field",
                serde_json::json!(FULL_DAMAGED_CASES - 1),
            ),
            (
                "/damaged_quality/aggregate/runtime_observer/grounded_candidate_losses_from_exact_field",
                serde_json::json!(1),
            ),
            ("/clean_quality/preserved", serde_json::json!(998)),
            ("/clean_quality/cases", serde_json::json!(999)),
            (
                "/damaged_quality/classes/missing_letter/unique_top1",
                serde_json::json!(19_000),
            ),
            (
                "/damaged_quality/classes/missing_letter/target_in_bounded_lattice",
                serde_json::json!(19_799),
            ),
            (
                "/damaged_quality/classes/missing_letter/false_authority",
                serde_json::json!(1),
            ),
            (
                "/gates/conjunctive_full_quality_pass",
                serde_json::json!(false),
            ),
            (
                "/claim_boundary/full_quality_matrix_tested",
                serde_json::json!(false),
            ),
            (
                "/claim_boundary/full_quality_claimed",
                serde_json::json!(false),
            ),
        ] {
            let mut rejected = valid.clone();
            *rejected.pointer_mut(pointer).expect("proof field") = value;
            assert!(
                validate_l11_proof_claims(&rejected, &receipt).is_err(),
                "{pointer}"
            );
        }
        for gate in REQUIRED_QUALITY_GATES {
            let mut rejected = valid.clone();
            *rejected
                .pointer_mut(&format!("/gates/{gate}"))
                .expect("quality gate") = serde_json::json!(false);
            assert!(
                validate_l11_proof_claims(&rejected, &receipt).is_err(),
                "{gate}"
            );
        }

        let mut wrong_classes = valid;
        let classes = wrong_classes
            .pointer_mut("/damaged_quality/classes")
            .and_then(serde_json::Value::as_object_mut)
            .expect("quality classes");
        let class = classes.remove("missing_letter").expect("fixed class");
        classes.insert("unknown_damage_class".to_string(), class);
        assert!(validate_l11_proof_claims(&wrong_classes, &receipt).is_err());
    }

    #[test]
    fn legacy_shadow_proof_remains_bound_to_the_exact_package() {
        let artifact_sha256 = "d".repeat(64);
        let receipt = InstalledL11Receipt {
            schema: INSTALLED_RECEIPT_SCHEMA.to_string(),
            package_id: "legacy".to_string(),
            format: "V8 test".to_string(),
            installed_artifact: PathBuf::from("/tmp/legacy.v8.bin"),
            artifact_bytes: 91,
            artifact_sha256: artifact_sha256.clone(),
            proof_receipt: PathBuf::from("/tmp/legacy.proof.json"),
            proof_sha256: "e".repeat(64),
            proof_verdict: "PASS_shadow".to_string(),
            runtime_authority: false,
            runtime_admitted: true,
        };
        let valid = serde_json::json!({
            "fixed_proof": {"verdict": "PASS_shadow"},
            "package": {"bytes": 91, "sha256": artifact_sha256}
        });
        validate_l11_proof_claims(&valid, &receipt).expect("valid legacy proof");

        let mut v9_receipt = receipt.clone();
        v9_receipt.format = "V9 test".to_string();
        assert!(validate_l11_proof_claims(&valid, &v9_receipt).is_err());

        let mut rejected = valid;
        rejected["package"]["bytes"] = serde_json::json!(92);
        assert!(validate_l11_proof_claims(&rejected, &receipt).is_err());
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
