use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const FORMAT: &str = "lay-l11-composite-v1";
const COMPACT_DELTA_COUNT: usize = 32;
const COMPACT_DELTA_BYTES: u64 = 32 * 1024 * 1024;
const MAX_DELTA_COUNT: usize = 64;
const MAX_TOMBSTONE_COUNT: usize = 100_000;
const MAX_TOMBSTONE_CHARS: usize = 128;
const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeltaEntry {
    path: PathBuf,
    bytes: u64,
    admitted_unix_ms: u64,
    proof_receipt: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TombstoneEntry {
    surface: String,
    admitted_unix_ms: u64,
    proof_receipt: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format: String,
    generation: u64,
    base: PathBuf,
    #[serde(default)]
    deltas: Vec<DeltaEntry>,
    #[serde(default)]
    tombstones: Vec<TombstoneEntry>,
}

#[derive(Clone, Debug)]
pub(super) struct CompositeSpec {
    pub(super) base_path: PathBuf,
    pub(super) delta_paths: Vec<PathBuf>,
    pub(super) tombstones: BTreeSet<String>,
    pub(super) manifest_bytes: u64,
    pub(super) generation: u64,
}

pub(super) fn load_spec(path: &Path) -> io::Result<Option<CompositeSpec>> {
    let Some(bytes) = read_json_candidate(path)? else {
        return Ok(None);
    };
    let manifest = match serde_json::from_slice::<Manifest>(&bytes) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(None),
    };
    validate_manifest(path, &manifest)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(Some(CompositeSpec {
        base_path: resolve(root, &manifest.base),
        delta_paths: manifest
            .deltas
            .iter()
            .map(|entry| resolve(root, &entry.path))
            .collect(),
        tombstones: manifest
            .tombstones
            .iter()
            .map(|entry| normalize_surface(&entry.surface))
            .collect(),
        manifest_bytes: bytes.len() as u64,
        generation: manifest.generation,
    }))
}

pub(super) fn manifest_generation(path: &Path) -> io::Result<Option<u64>> {
    let Some(bytes) = read_json_candidate(path)? else {
        return Ok(None);
    };
    let manifest = match serde_json::from_slice::<Manifest>(&bytes) {
        Ok(manifest) if manifest.format == FORMAT => manifest,
        _ => return Ok(None),
    };
    Ok(Some(manifest.generation))
}

pub fn initialize_manifest(
    manifest_path: &Path,
    base_path: &Path,
) -> io::Result<serde_json::Value> {
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let base_path = absolute_path(base_path)?;
    super::runtime::inspect_package_header(&base_path)?;
    let _lock = acquire_manifest_write_lock(manifest_path)?;
    let manifest = Manifest {
        format: FORMAT.to_string(),
        generation: 1,
        base: path_for_manifest(root, &base_path),
        deltas: Vec::new(),
        tombstones: Vec::new(),
    };
    write_manifest(manifest_path, &manifest)?;
    Ok(serde_json::json!({
        "kind": "l11_composite_manifest_initialized",
        "manifest": manifest_path,
        "base": base_path,
        "generation": manifest.generation,
        "runtime_authority": false,
    }))
}

pub fn admit_delta(
    manifest_path: &Path,
    delta_path: &Path,
    proof_receipt: &Path,
    scope: Option<&str>,
) -> io::Result<serde_json::Value> {
    require_proof_receipt(proof_receipt)?;
    let delta_path = absolute_path(delta_path)?;
    super::runtime::inspect_package_header(&delta_path)?;
    let _lock = acquire_manifest_write_lock(manifest_path)?;
    let mut manifest = read_manifest(manifest_path)?;
    if manifest.deltas.len() >= MAX_DELTA_COUNT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1.1 composite delta limit reached; compact before admitting another delta",
        ));
    }
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let stored_path = path_for_manifest(root, &delta_path);
    if manifest
        .deltas
        .iter()
        .any(|entry| entry.path == stored_path)
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("L1.1 delta is already admitted: {}", stored_path.display()),
        ));
    }
    let bytes = fs::metadata(&delta_path)?.len();
    manifest.generation = manifest.generation.saturating_add(1);
    manifest.deltas.push(DeltaEntry {
        path: stored_path,
        bytes,
        admitted_unix_ms: unix_time_ms(),
        proof_receipt: path_for_manifest(root, &absolute_path(proof_receipt)?),
        scope: scope.map(str::to_owned),
    });
    write_manifest(manifest_path, &manifest)?;
    Ok(admission_report(
        manifest_path,
        &manifest,
        serde_json::json!({
            "delta": delta_path,
            "delta_bytes": bytes,
        }),
    ))
}

pub fn admit_tombstone(
    manifest_path: &Path,
    surface: &str,
    proof_receipt: &Path,
    scope: Option<&str>,
) -> io::Result<serde_json::Value> {
    require_proof_receipt(proof_receipt)?;
    let surface = normalize_surface(surface);
    if surface.is_empty()
        || surface.chars().count() > MAX_TOMBSTONE_CHARS
        || surface.chars().any(char::is_control)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L1.1 tombstone surface is empty, oversized or contains control characters",
        ));
    }
    let _lock = acquire_manifest_write_lock(manifest_path)?;
    let mut manifest = read_manifest(manifest_path)?;
    if manifest.tombstones.len() >= MAX_TOMBSTONE_COUNT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1.1 composite tombstone limit reached; compact before admitting another tombstone",
        ));
    }
    if manifest
        .tombstones
        .iter()
        .any(|entry| normalize_surface(&entry.surface) == surface)
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("L1.1 surface is already tombstoned: {surface}"),
        ));
    }
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    manifest.generation = manifest.generation.saturating_add(1);
    manifest.tombstones.push(TombstoneEntry {
        surface: surface.clone(),
        admitted_unix_ms: unix_time_ms(),
        proof_receipt: path_for_manifest(root, &absolute_path(proof_receipt)?),
        scope: scope.map(str::to_owned),
    });
    write_manifest(manifest_path, &manifest)?;
    Ok(admission_report(
        manifest_path,
        &manifest,
        serde_json::json!({ "tombstone": surface }),
    ))
}

fn admission_report(
    manifest_path: &Path,
    manifest: &Manifest,
    change: serde_json::Value,
) -> serde_json::Value {
    let delta_bytes = manifest.deltas.iter().map(|entry| entry.bytes).sum::<u64>();
    serde_json::json!({
        "kind": "l11_composite_admission",
        "manifest": manifest_path,
        "generation": manifest.generation,
        "change": change,
        "base_rewritten": false,
        "delta_count": manifest.deltas.len(),
        "delta_bytes": delta_bytes,
        "tombstone_count": manifest.tombstones.len(),
        "compaction_recommended": manifest.deltas.len() >= COMPACT_DELTA_COUNT
            || delta_bytes >= COMPACT_DELTA_BYTES,
        "runtime_authority": false,
    })
}

fn read_manifest(path: &Path) -> io::Result<Manifest> {
    let bytes = read_json_candidate(path)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "L1.1 composite manifest is not a JSON object",
        )
    })?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    validate_manifest(path, &manifest)?;
    Ok(manifest)
}

fn read_json_candidate(path: &Path) -> io::Result<Option<Vec<u8>>> {
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut first = [0_u8; 1];
    if file.read(&mut first)? != 1 || first[0] != b'{' {
        return Ok(None);
    }
    let manifest_bytes = file.metadata()?.len();
    if manifest_bytes > MAX_MANIFEST_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("L1.1 composite manifest exceeds {MAX_MANIFEST_BYTES} bytes"),
        ));
    }
    let mut bytes = Vec::with_capacity(manifest_bytes as usize);
    bytes.push(first[0]);
    file.read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

fn validate_manifest(path: &Path, manifest: &Manifest) -> io::Result<()> {
    if manifest.format != FORMAT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported L1.1 composite format: {}", manifest.format),
        ));
    }
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    if manifest.deltas.len() > MAX_DELTA_COUNT || manifest.tombstones.len() > MAX_TOMBSTONE_COUNT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L1.1 composite entry count exceeds its bounded runtime limit",
        ));
    }
    let base_path = resolve(root, &manifest.base);
    if !base_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("L1.1 composite base is missing: {}", base_path.display()),
        ));
    }
    let mut delta_paths = BTreeSet::new();
    for entry in &manifest.deltas {
        if !delta_paths.insert(entry.path.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("duplicate L1.1 delta path: {}", entry.path.display()),
            ));
        }
        let path = resolve(root, &entry.path);
        let actual = fs::metadata(&path)?.len();
        if actual != entry.bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "L1.1 delta size changed: manifest={} actual={} path={}",
                    entry.bytes,
                    actual,
                    path.display()
                ),
            ));
        }
        require_proof_receipt(&resolve(root, &entry.proof_receipt))?;
    }
    let mut tombstones = BTreeSet::new();
    for entry in &manifest.tombstones {
        let surface = normalize_surface(&entry.surface);
        if surface.is_empty()
            || surface.chars().count() > MAX_TOMBSTONE_CHARS
            || !tombstones.insert(surface)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "empty or duplicate L1.1 tombstone",
            ));
        }
        require_proof_receipt(&resolve(root, &entry.proof_receipt))?;
    }
    Ok(())
}

fn write_manifest(path: &Path, manifest: &Manifest) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(manifest).map_err(io::Error::other)?;
    bytes.push(b'\n');
    let temporary = path.with_extension(format!(
        "{}.tmp-{}",
        path.extension()
            .and_then(|value| value.to_str())
            .unwrap_or("json"),
        std::process::id()
    ));
    let mut temporary_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(&temporary)?;
    temporary_file.write_all(&bytes)?;
    temporary_file.sync_all()?;
    drop(temporary_file);
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

struct ManifestWriteLock {
    file: fs::File,
}

impl Drop for ManifestWriteLock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn acquire_manifest_write_lock(path: &Path) -> io::Result<ManifestWriteLock> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut lock_name = path
        .file_name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "l11.runtime.json".into());
    lock_name.push(".lock");
    let lock_path = path.with_file_name(lock_name);
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(lock_path)?;
    loop {
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } == 0 {
            return Ok(ManifestWriteLock { file });
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn require_proof_receipt(path: &Path) -> io::Result<()> {
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("L1.1 delta proof receipt is missing: {}", path.display()),
        ));
    }
    let receipt: serde_json::Value =
        serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)?;
    let verdict = receipt
        .get("verdict")
        .or_else(|| receipt.get("verdict_scope"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if verdict.starts_with("PASS") {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "L1.1 delta proof receipt is not PASS: verdict={verdict:?} path={}",
                path.display()
            ),
        ))
    }
}

fn normalize_surface(surface: &str) -> String {
    super::atoms::normalize_lexical_surface(surface)
}

fn resolve(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn path_for_manifest(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn normalize_tombstone_is_stable() {
        assert_eq!(normalize_surface("  ВРЕМЯ "), "время");
        assert_eq!(normalize_surface("ВРЕМЯ?!"), "время");
    }

    #[test]
    fn manifest_writer_lock_serializes_independent_transactions() {
        let root = std::env::temp_dir().join(format!(
            "lay-l11-manifest-lock-{}-{}",
            std::process::id(),
            unix_time_ms()
        ));
        let manifest = root.join("runtime.json");
        let first = acquire_manifest_write_lock(&manifest).expect("first writer lock");
        let second_manifest = manifest.clone();
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let second = std::thread::spawn(move || {
            let _lock = acquire_manifest_write_lock(&second_manifest).expect("second writer lock");
            acquired_tx.send(()).expect("report second lock");
        });

        assert!(matches!(
            acquired_rx.recv_timeout(Duration::from_millis(50)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        drop(first);
        acquired_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("second writer proceeds after unlock");
        second.join().expect("second writer thread");
        let _ = fs::remove_dir_all(root);
    }
}
