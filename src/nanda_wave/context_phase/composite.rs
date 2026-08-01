use std::fs;
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::{
    read_package, ContextPairPhaseProfile, ContextPhasePackage, MAX_HARD_PAIR_CENTERS_PER_BANK,
    MAX_PAIR_CENTERS_PER_BANK, MAX_PAIR_PROFILES,
};

pub(crate) const COMPOSITE_FORMAT: &str = "lay-l3-composite-v1";
pub(crate) const COMPACT_DELTA_COUNT: usize = 32;
pub(crate) const COMPACT_DELTA_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct L3DeltaEntry {
    pub(crate) path: PathBuf,
    pub(crate) bytes: u64,
    pub(crate) admitted_unix_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) proof_receipt: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) full_proof_receipt: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct L3CompositeManifest {
    pub(crate) format: String,
    pub(crate) base: PathBuf,
    #[serde(default)]
    pub(crate) deltas: Vec<L3DeltaEntry>,
}

#[derive(Clone, Debug)]
pub(crate) struct L3CompositeMemory {
    pub(super) package: ContextPhasePackage,
    pub(super) manifest_path: Option<PathBuf>,
    pub(super) base_path: PathBuf,
    pub(super) delta_paths: Vec<PathBuf>,
    pub(super) delta_bytes: u64,
    pub(super) manifest_stamp: u64,
}

impl L3CompositeMemory {
    pub(crate) fn from_package(path: &Path) -> io::Result<Self> {
        Ok(Self {
            package: read_package(path)?,
            manifest_path: None,
            base_path: path.to_path_buf(),
            delta_paths: Vec::new(),
            delta_bytes: 0,
            manifest_stamp: 0,
        })
    }

    pub(crate) fn empty(path: PathBuf) -> Self {
        Self {
            package: ContextPhasePackage::default(),
            manifest_path: None,
            base_path: path,
            delta_paths: Vec::new(),
            delta_bytes: 0,
            manifest_stamp: 0,
        }
    }

    pub(crate) fn load_manifest(path: &Path) -> io::Result<Self> {
        let manifest = read_manifest(path)?;
        let root = path.parent().unwrap_or_else(|| Path::new("."));
        let base_path = resolve(root, &manifest.base);
        let base = read_package(&base_path)?;
        let signature_schema = base.signature_schema;
        let mut deltas = Vec::with_capacity(manifest.deltas.len());
        let mut delta_paths = Vec::with_capacity(manifest.deltas.len());
        let mut delta_bytes = 0_u64;
        for entry in &manifest.deltas {
            let delta_path = resolve(root, &entry.path);
            let delta = read_package(&delta_path)?;
            if delta.signature_schema != signature_schema {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "L3 delta schema {} does not match base schema {}: {}",
                        delta.signature_schema,
                        signature_schema,
                        delta_path.display()
                    ),
                ));
            }
            let actual_bytes = fs::metadata(&delta_path)?.len();
            if actual_bytes != entry.bytes {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "L3 delta size changed: manifest={} actual={} path={}",
                        entry.bytes,
                        actual_bytes,
                        delta_path.display()
                    ),
                ));
            }
            delta_bytes = delta_bytes.saturating_add(actual_bytes);
            delta_paths.push(delta_path);
            deltas.push(delta);
        }
        // The common runtime manifest has no deltas. Re-running the shard
        // reducer in that case only duplicates the base package in anonymous
        // memory and changes no evidence.
        let package = if deltas.is_empty() {
            base
        } else {
            compose_base_with_deltas(base, deltas)
        };
        Ok(Self {
            package,
            manifest_path: Some(path.to_path_buf()),
            base_path,
            delta_paths,
            delta_bytes,
            manifest_stamp: file_stamp(path).unwrap_or_default(),
        })
    }

    pub(crate) fn package(&self) -> &ContextPhasePackage {
        &self.package
    }

    pub(crate) fn compose_delta_path(&self, path: &Path) -> io::Result<ContextPhasePackage> {
        let delta = read_package(path)?;
        if delta.signature_schema != self.package.signature_schema {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L3 delta signature schema does not match the composite baseline",
            ));
        }
        Ok(compose_base_with_deltas(self.package.clone(), vec![delta]))
    }

    pub(crate) fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": "l3_composite_memory",
            "loaded": true,
            "manifest": self.manifest_path,
            "base": self.base_path,
            "delta_count": self.delta_paths.len(),
            "delta_bytes": self.delta_bytes,
            "manifest_stamp": self.manifest_stamp,
            "compaction_recommended": self.delta_paths.len() >= COMPACT_DELTA_COUNT
                || self.delta_bytes >= COMPACT_DELTA_BYTES,
            "compaction_delta_count_threshold": COMPACT_DELTA_COUNT,
            "compaction_delta_bytes_threshold": COMPACT_DELTA_BYTES,
            "signature_schema": self.package.signature_schema,
            "semantic_states": self.package.semantic_states.len(),
            "candidate_profiles": self.package.profiles.len(),
            "pair_profiles": self.package.pair_profiles.len(),
            "transitions": self.package.transitions,
            "corpus_fragments": self.package.corpus_fragments,
            "runtime_authority": false,
        })
    }
}

fn compose_base_with_deltas(
    mut base: ContextPhasePackage,
    mut deltas: Vec<ContextPhasePackage>,
) -> ContextPhasePackage {
    let base_global_threshold = base.global_threshold_micro;
    let base_competition_threshold = base.competition_threshold_micro;
    let base_pairwise_threshold = base.pairwise_threshold_micro;
    let base_profile_thresholds = base
        .profiles
        .iter()
        .map(|profile| (profile.token_hash, profile.threshold_micro))
        .collect::<std::collections::BTreeMap<_, _>>();
    let base_signature_thresholds = base
        .signature_profiles
        .iter()
        .map(|profile| (profile.token_hash, profile.threshold_micro))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut pair_profiles = std::mem::take(&mut base.pair_profiles);
    for delta in &mut deltas {
        // A small shard contributes evidence, not global calibration.
        // Otherwise one narrow update can replace the energy scale learned
        // from the immutable broad corpus.
        for profile in &mut delta.profiles {
            profile.threshold_micro = base_profile_thresholds
                .get(&profile.token_hash)
                .copied()
                .unwrap_or(base_global_threshold);
        }
        for profile in &mut delta.signature_profiles {
            profile.threshold_micro = base_signature_thresholds
                .get(&profile.token_hash)
                .copied()
                .unwrap_or(base_global_threshold);
        }
        delta.global_threshold_micro = base_global_threshold;
        delta.competition_threshold_micro = base_competition_threshold;
        delta.pairwise_threshold_micro = base_pairwise_threshold;
        for pair in std::mem::take(&mut delta.pair_profiles) {
            append_composite_pair_profile(&mut pair_profiles, pair);
        }
    }
    let mut packages = Vec::with_capacity(deltas.len() + 1);
    packages.push(base);
    packages.extend(deltas);
    let mut package = ContextPhasePackage::merge_shards_with_min_surface_support(packages, 1).0;
    package.global_threshold_micro = base_global_threshold;
    package.competition_threshold_micro = base_competition_threshold;
    package.pairwise_threshold_micro = base_pairwise_threshold;
    package.pair_profiles = pair_profiles;
    package
}

fn append_composite_pair_profile(
    profiles: &mut Vec<ContextPairPhaseProfile>,
    incoming: ContextPairPhaseProfile,
) {
    let key = (incoming.low_hash, incoming.high_hash);
    match profiles.binary_search_by_key(&key, |profile| (profile.low_hash, profile.high_hash)) {
        Ok(index) => {
            let existing = &mut profiles[index];
            append_center_bank(
                &mut existing.low_wins,
                incoming.low_wins,
                MAX_PAIR_CENTERS_PER_BANK,
            );
            append_center_bank(
                &mut existing.high_wins,
                incoming.high_wins,
                MAX_PAIR_CENTERS_PER_BANK,
            );
            append_center_bank(
                &mut existing.hard_low_wins,
                incoming.hard_low_wins,
                MAX_HARD_PAIR_CENTERS_PER_BANK,
            );
            append_center_bank(
                &mut existing.hard_high_wins,
                incoming.hard_high_wins,
                MAX_HARD_PAIR_CENTERS_PER_BANK,
            );
        }
        Err(index) if profiles.len() < MAX_PAIR_PROFILES => {
            let mut bounded = ContextPairPhaseProfile {
                low_hash: incoming.low_hash,
                high_hash: incoming.high_hash,
                ..ContextPairPhaseProfile::default()
            };
            append_center_bank(
                &mut bounded.low_wins,
                incoming.low_wins,
                MAX_PAIR_CENTERS_PER_BANK,
            );
            append_center_bank(
                &mut bounded.high_wins,
                incoming.high_wins,
                MAX_PAIR_CENTERS_PER_BANK,
            );
            append_center_bank(
                &mut bounded.hard_low_wins,
                incoming.hard_low_wins,
                MAX_HARD_PAIR_CENTERS_PER_BANK,
            );
            append_center_bank(
                &mut bounded.hard_high_wins,
                incoming.hard_high_wins,
                MAX_HARD_PAIR_CENTERS_PER_BANK,
            );
            profiles.insert(index, bounded);
        }
        Err(_) => {}
    }
}

fn append_center_bank(
    target: &mut Vec<super::super::phase_field::PhaseCenter>,
    incoming: Vec<super::super::phase_field::PhaseCenter>,
    max_centers: usize,
) {
    for mut center in incoming {
        center.materialize_sum();
        if target.len() < max_centers {
            target.push(center);
            continue;
        }
        let Some(index) = target
            .iter_mut()
            .enumerate()
            .map(|(index, current)| {
                current.materialize_sum();
                (
                    index,
                    super::vector_phase_coherence(&center.center, &current.center),
                )
            })
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            })
            .map(|(index, _)| index)
        else {
            continue;
        };
        let current = &mut target[index];
        super::add_phase_vector(&mut current.sum, &center.sum);
        current.center = super::phase_center_from_sum(&current.sum);
        current.support = current.support.saturating_add(center.support);
    }
}

pub(crate) fn read_manifest(path: &Path) -> io::Result<L3CompositeManifest> {
    let bytes = fs::read(path)?;
    let manifest: L3CompositeManifest = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if manifest.format != COMPOSITE_FORMAT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported L3 composite format: {}", manifest.format),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for delta in &manifest.deltas {
        if !seen.insert(delta.path.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("duplicate L3 delta path: {}", delta.path.display()),
            ));
        }
    }
    Ok(manifest)
}

pub(crate) fn initialize_manifest(manifest_path: &Path, base_path: &Path) -> io::Result<()> {
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let resolved_base = absolute_path(base_path)?;
    let _ = read_package(&resolved_base)?;
    let manifest = L3CompositeManifest {
        format: COMPOSITE_FORMAT.to_string(),
        base: path_for_manifest(root, &resolved_base),
        deltas: Vec::new(),
    };
    write_manifest(manifest_path, &manifest)
}

pub(crate) fn admit_delta(
    manifest_path: &Path,
    delta_path: &Path,
    proof_receipt: Option<&Path>,
    scope: Option<&str>,
) -> io::Result<serde_json::Value> {
    admit_delta_with_full_proof(manifest_path, delta_path, proof_receipt, None, scope)
}

pub(crate) fn admit_delta_with_full_proof(
    manifest_path: &Path,
    delta_path: &Path,
    proof_receipt: Option<&Path>,
    full_proof_receipt: Option<&Path>,
    scope: Option<&str>,
) -> io::Result<serde_json::Value> {
    let _lock = acquire_manifest_write_lock(manifest_path)?;
    let mut manifest = read_manifest(manifest_path)?;
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let base = read_package(&resolve(root, &manifest.base))?;
    let resolved_delta = absolute_path(delta_path)?;
    let delta = read_package(&resolved_delta)?;
    if delta.signature_schema != base.signature_schema {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L3 delta signature schema does not match the immutable base",
        ));
    }
    let stored_path = path_for_manifest(root, &resolved_delta);
    if manifest
        .deltas
        .iter()
        .any(|entry| entry.path == stored_path)
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("L3 delta is already admitted: {}", stored_path.display()),
        ));
    }
    let bytes = fs::metadata(&resolved_delta)?.len();
    manifest.deltas.push(L3DeltaEntry {
        path: stored_path,
        bytes,
        admitted_unix_ms: unix_time_ms(),
        proof_receipt: proof_receipt.map(|path| path_for_manifest(root, path)),
        full_proof_receipt: full_proof_receipt.map(|path| path_for_manifest(root, path)),
        scope: scope.map(str::to_owned),
    });
    write_manifest_unlocked(manifest_path, &manifest)?;
    let total_delta_bytes = manifest.deltas.iter().map(|entry| entry.bytes).sum::<u64>();
    Ok(serde_json::json!({
        "kind": "l3_delta_admission",
        "manifest": manifest_path,
        "base_rewritten": false,
        "delta": resolved_delta,
        "delta_bytes": bytes,
        "delta_count": manifest.deltas.len(),
        "total_delta_bytes": total_delta_bytes,
        "targeted_proof_receipt": proof_receipt,
        "full_proof_receipt": full_proof_receipt,
        "compaction_recommended": manifest.deltas.len() >= COMPACT_DELTA_COUNT
            || total_delta_bytes >= COMPACT_DELTA_BYTES,
        "runtime_authority": false,
    }))
}

pub(crate) fn compact_manifest(
    manifest_path: &Path,
    output_base: &Path,
) -> io::Result<serde_json::Value> {
    let _lock = acquire_manifest_write_lock(manifest_path)?;
    let memory = L3CompositeMemory::load_manifest(manifest_path)?;
    let output_base = absolute_path(output_base)?;
    if output_base == memory.base_path {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L3 compaction must write a new base path before the manifest flips",
        ));
    }
    super::write_package(&output_base, memory.package())?;
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let manifest = L3CompositeManifest {
        format: COMPOSITE_FORMAT.to_string(),
        base: path_for_manifest(root, &output_base),
        deltas: Vec::new(),
    };
    write_manifest_unlocked(manifest_path, &manifest)?;
    Ok(serde_json::json!({
        "kind": "l3_composite_compaction",
        "manifest": manifest_path,
        "output_base": output_base,
        "compacted_deltas": memory.delta_paths.len(),
        "compacted_delta_bytes": memory.delta_bytes,
        "runtime_authority": false,
    }))
}

pub(crate) fn snapshot_manifest(
    manifest_path: &Path,
    output_base: &Path,
) -> io::Result<serde_json::Value> {
    let memory = L3CompositeMemory::load_manifest(manifest_path)?;
    let output_base = absolute_path(output_base)?;
    super::write_package(&output_base, memory.package())?;
    Ok(serde_json::json!({
        "kind": "l3_composite_snapshot",
        "manifest": manifest_path,
        "output_base": output_base,
        "included_deltas": memory.delta_paths.len(),
        "included_delta_bytes": memory.delta_bytes,
        "manifest_rewritten": false,
        "runtime_authority": false,
    }))
}

fn write_manifest(path: &Path, manifest: &L3CompositeManifest) -> io::Result<()> {
    let _lock = acquire_manifest_write_lock(path)?;
    write_manifest_unlocked(path, manifest)
}

fn write_manifest_unlocked(path: &Path, manifest: &L3CompositeManifest) -> io::Result<()> {
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
        .unwrap_or_else(|| "l3.runtime.json".into());
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

pub(super) fn file_stamp(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    Ok(modified ^ metadata.len().rotate_left(17))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::context_phase::write_package;
    use crate::nanda_wave::phase_field::{PhaseCell, PhaseCenter};

    #[test]
    fn manifest_admission_keeps_base_immutable_and_loads_delta() {
        let root = std::env::temp_dir().join(format!(
            "lay-l3-composite-{}-{}",
            std::process::id(),
            unix_time_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let base_path = root.join("base.nwpc");
        let delta_path = root.join("delta-000001.nwpc");
        let manifest_path = root.join("manifest.json");
        let base = ContextPhasePackage {
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            transitions: 7,
            ..ContextPhasePackage::default()
        };
        let delta = ContextPhasePackage {
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            transitions: 3,
            ..ContextPhasePackage::default()
        };
        write_package(&base_path, &base).unwrap();
        write_package(&delta_path, &delta).unwrap();
        let before = fs::read(&base_path).unwrap();

        initialize_manifest(&manifest_path, &base_path).unwrap();
        admit_delta(&manifest_path, &delta_path, None, Some("test")).unwrap();
        let memory = L3CompositeMemory::load_manifest(&manifest_path).unwrap();

        assert_eq!(memory.package().transitions, 10);
        assert_eq!(memory.delta_paths.len(), 1);
        assert_eq!(fs::read(&base_path).unwrap(), before);

        let manifest_before = fs::read(&manifest_path).unwrap();
        let snapshot_path = root.join("snapshot.nwpc");
        snapshot_manifest(&manifest_path, &snapshot_path).unwrap();
        assert_eq!(fs::read(&manifest_path).unwrap(), manifest_before);
        assert_eq!(read_package(&snapshot_path).unwrap().transitions, 10);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_manifest_admissions_do_not_lose_a_delta() {
        let root = std::env::temp_dir().join(format!(
            "lay-l3-composite-concurrent-{}-{}",
            std::process::id(),
            unix_time_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let base_path = root.join("base.nwpc");
        let first_delta = root.join("delta-a.nwpc");
        let second_delta = root.join("delta-b.nwpc");
        let manifest_path = root.join("manifest.json");
        let package = ContextPhasePackage {
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            ..ContextPhasePackage::default()
        };
        write_package(&base_path, &package).unwrap();
        write_package(&first_delta, &package).unwrap();
        write_package(&second_delta, &package).unwrap();
        initialize_manifest(&manifest_path, &base_path).unwrap();

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
        let handles = [first_delta, second_delta].map(|delta| {
            let manifest = manifest_path.clone();
            let barrier = std::sync::Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                admit_delta(&manifest, &delta, None, Some("concurrent-test")).unwrap();
            })
        });
        barrier.wait();
        for handle in handles {
            handle.join().unwrap();
        }

        let manifest = read_manifest(&manifest_path).unwrap();
        assert_eq!(manifest.deltas.len(), 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn manifest_keeps_targeted_and_full_proof_receipts_for_delta() {
        let root = std::env::temp_dir().join(format!(
            "lay-l3-composite-proof-{}-{}",
            std::process::id(),
            unix_time_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let base_path = root.join("base.nwpc");
        let delta_path = root.join("delta.nwpc");
        let targeted_path = root.join("targeted.json");
        let full_path = root.join("full.json");
        let manifest_path = root.join("manifest.json");
        let package = ContextPhasePackage {
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            ..ContextPhasePackage::default()
        };
        write_package(&base_path, &package).unwrap();
        write_package(&delta_path, &package).unwrap();
        fs::write(&targeted_path, b"{}").unwrap();
        fs::write(&full_path, b"{}").unwrap();

        initialize_manifest(&manifest_path, &base_path).unwrap();
        admit_delta_with_full_proof(
            &manifest_path,
            &delta_path,
            Some(&targeted_path),
            Some(&full_path),
            Some("online-test"),
        )
        .unwrap();
        let manifest = read_manifest(&manifest_path).unwrap();

        assert_eq!(manifest.deltas.len(), 1);
        assert_eq!(
            resolve(&root, manifest.deltas[0].proof_receipt.as_ref().unwrap()),
            targeted_path
        );
        assert_eq!(
            resolve(
                &root,
                manifest.deltas[0].full_proof_receipt.as_ref().unwrap()
            ),
            full_path
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn empty_delta_manifest_loads_base_without_reducing_it_again() {
        let root = std::env::temp_dir().join(format!(
            "lay-l3-composite-empty-{}-{}",
            std::process::id(),
            unix_time_ms()
        ));
        fs::create_dir_all(&root).unwrap();
        let base_path = root.join("base.nwpc");
        let manifest_path = root.join("manifest.json");
        let base = ContextPhasePackage {
            semantic_states: vec![super::super::TokenSemanticState {
                token_hash: 11,
                support: 3,
                center: vec![PhaseCell { re: 0.25, im: -0.5 }; super::super::CELLS],
            }],
            profiles: vec![super::super::ContextCandidateProfile {
                token_hash: 17,
                positive_examples: 3,
                negative_examples: 0,
                threshold_micro: 42_000,
                positive: vec![PhaseCenter::from_center(
                    vec![PhaseCell { re: 0.5, im: 0.25 }; super::super::CELLS],
                    3,
                )],
                negative: Vec::new(),
                hard_negative: Vec::new(),
            }],
            transitions: 7,
            corpus_fragments: 5,
            global_threshold_micro: 31_000,
            competition_threshold_micro: 9_000,
            pairwise_threshold_micro: 4_000,
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            ..ContextPhasePackage::default()
        };
        write_package(&base_path, &base).unwrap();
        initialize_manifest(&manifest_path, &base_path).unwrap();

        let memory = L3CompositeMemory::load_manifest(&manifest_path).unwrap();

        assert!(memory.delta_paths.is_empty());
        assert_eq!(memory.package().semantic_states.len(), 1);
        assert_eq!(memory.package().profiles.len(), 1);
        assert_eq!(memory.package().transitions, 7);
        assert_eq!(memory.package().global_threshold_micro, 31_000);
        assert_eq!(memory.package().profiles[0].threshold_micro, 42_000);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delta_pair_centers_do_not_disappear_behind_a_full_base_bank() {
        let center = |re: f32| {
            PhaseCenter::from_center(vec![PhaseCell { re, im: 0.0 }; super::super::CELLS], 2)
        };
        let base = ContextPhasePackage {
            pair_profiles: vec![ContextPairPhaseProfile {
                low_hash: 11,
                high_hash: 17,
                low_wins: (0..16)
                    .map(|index| center(0.1 + index as f32 / 100.0))
                    .collect(),
                ..ContextPairPhaseProfile::default()
            }],
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            ..ContextPhasePackage::default()
        };
        let delta = ContextPhasePackage {
            pair_profiles: vec![ContextPairPhaseProfile {
                low_hash: 11,
                high_hash: 17,
                low_wins: vec![center(1.0)],
                ..ContextPairPhaseProfile::default()
            }],
            signature_schema: super::super::SIGNATURE_SCHEMA_RELATION_ROLES,
            ..ContextPhasePackage::default()
        };

        let composed = compose_base_with_deltas(base, vec![delta]);

        assert_eq!(
            composed.pair_profiles[0].low_wins.len(),
            MAX_PAIR_CENTERS_PER_BANK
        );
        assert!(composed.pair_profiles[0]
            .low_wins
            .iter()
            .any(|value| value.support == 4));
    }
}
