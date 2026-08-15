//! Cold checkpointed updater for the transactional L4 episode inbox.

use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

use super::compiler::{compile_observations, CrossSceneCompileConfig};
use super::format::{canonical_runtime_package, encode_package, read_package, write_package};
use super::merge::{merge_package_delta, require_v2};
use super::model::{CrossSceneCompileReport, L4CrossScenePackage};
use super::runtime::readout;
use super::segments::{delete_acknowledged, read_segments_after};
use super::usage_adapter::observations_from_segment;

#[derive(Clone, Debug, Default, Serialize)]
struct IncrementalProofReport {
    package_roundtrip_exact: bool,
    runtime_readout_parity: bool,
    automatic_apply_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct IncrementalUpdateReport {
    kind: &'static str,
    verdict: String,
    inbox: PathBuf,
    package: PathBuf,
    package_existed: bool,
    previous_applied_segment: u64,
    applied_segment: u64,
    segments_read: usize,
    episodes_read: usize,
    rows_read: usize,
    observations_compiled: usize,
    profiles_before: usize,
    profiles_after: usize,
    pair_profiles_before: usize,
    pair_profiles_after: usize,
    symbols_before: usize,
    symbols_after: usize,
    package_bytes_before: u64,
    package_bytes_after: u64,
    acknowledged_segments_deleted: usize,
    cleanup_pending: bool,
    cleanup_error: Option<String>,
    elapsed_us: u64,
    compile: CrossSceneCompileReport,
    proof: IncrementalProofReport,
    runtime_authority_changed: bool,
}

pub(crate) fn update_package_from_inbox(
    inbox: &Path,
    package_path: &Path,
    config: CrossSceneCompileConfig,
) -> io::Result<IncrementalUpdateReport> {
    let started = Instant::now();
    let _lock = acquire_update_lock(package_path)?;
    let package_existed = package_path.is_file();
    let package_bytes_before = fs::metadata(package_path)
        .map(|metadata| metadata.len())
        .unwrap_or_default();
    let base = if package_existed {
        read_package(package_path)?
    } else {
        L4CrossScenePackage::default()
    };
    require_v2(&base)?;
    let previous_applied_segment = base.applied_segment;
    let profiles_before = base.profiles.len();
    let pair_profiles_before = base.pair_profiles.len();
    let symbols_before = base.symbols.len();
    let segments = read_segments_after(inbox, previous_applied_segment)?;

    if segments.is_empty() {
        let (deleted, cleanup_error) = cleanup(inbox, previous_applied_segment);
        return Ok(IncrementalUpdateReport {
            kind: "l4_cross_scene_incremental_update",
            verdict: if cleanup_error.is_some() {
                "NOOP_CLEANUP_PENDING".to_string()
            } else {
                "NOOP".to_string()
            },
            inbox: inbox.to_path_buf(),
            package: package_path.to_path_buf(),
            package_existed,
            previous_applied_segment,
            applied_segment: previous_applied_segment,
            segments_read: 0,
            episodes_read: 0,
            rows_read: 0,
            observations_compiled: 0,
            profiles_before,
            profiles_after: profiles_before,
            pair_profiles_before,
            pair_profiles_after: pair_profiles_before,
            symbols_before,
            symbols_after: symbols_before,
            package_bytes_before,
            package_bytes_after: package_bytes_before,
            acknowledged_segments_deleted: deleted,
            cleanup_pending: cleanup_error.is_some(),
            cleanup_error,
            elapsed_us: elapsed_us(started),
            compile: CrossSceneCompileReport::default(),
            proof: IncrementalProofReport {
                package_roundtrip_exact: true,
                runtime_readout_parity: true,
                automatic_apply_count: 0,
            },
            runtime_authority_changed: false,
        });
    }

    let applied_segment = segments
        .last()
        .map(|segment| segment.segment_id)
        .expect("checked non-empty segments");
    let rows_read = segments.iter().map(|segment| segment.rows.len()).sum();
    let mut observations = Vec::with_capacity(rows_read);
    for segment in &segments {
        observations.extend(observations_from_segment(segment)?);
    }
    let (delta, compile) = compile_observations(&observations, config);
    let merged = merge_package_delta(base, delta, applied_segment)?;
    let candidate = canonical_runtime_package(&merged).map_err(invalid_data)?;
    let candidate_bytes = encode_package(&candidate);
    let roundtrip = canonical_runtime_package(&candidate).map_err(invalid_data)?;
    let package_roundtrip_exact = candidate_bytes == encode_package(&roundtrip);
    let runtime_readout_parity = observations.iter().all(|observation| {
        readout(&candidate, observation.input()) == readout(&roundtrip, observation.input())
    });
    let automatic_apply_count = observations
        .iter()
        .filter(|observation| {
            readout(&candidate, observation.input())
                .recommendation
                .automatic_apply()
        })
        .count();
    if !package_roundtrip_exact || !runtime_readout_parity || automatic_apply_count != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incremental L4 shadow proof failed before publication",
        ));
    }

    write_package(package_path, &candidate)?;
    let published = read_package(package_path)?;
    if published.applied_segment != applied_segment || encode_package(&published) != candidate_bytes
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "published L4 package did not preserve candidate checkpoint bytes",
        ));
    }
    let package_bytes_after = fs::metadata(package_path)?.len();
    let (deleted, cleanup_error) = cleanup(inbox, applied_segment);
    let cleanup_pending = cleanup_error.is_some();

    Ok(IncrementalUpdateReport {
        kind: "l4_cross_scene_incremental_update",
        verdict: if cleanup_pending {
            "UPDATED_SHADOW_CLEANUP_PENDING".to_string()
        } else {
            "UPDATED_SHADOW".to_string()
        },
        inbox: inbox.to_path_buf(),
        package: package_path.to_path_buf(),
        package_existed,
        previous_applied_segment,
        applied_segment,
        segments_read: segments.len(),
        episodes_read: segments.len(),
        rows_read,
        observations_compiled: observations.len(),
        profiles_before,
        profiles_after: published.profiles.len(),
        pair_profiles_before,
        pair_profiles_after: published.pair_profiles.len(),
        symbols_before,
        symbols_after: published.symbols.len(),
        package_bytes_before,
        package_bytes_after,
        acknowledged_segments_deleted: deleted,
        cleanup_pending,
        cleanup_error,
        elapsed_us: elapsed_us(started),
        compile,
        proof: IncrementalProofReport {
            package_roundtrip_exact,
            runtime_readout_parity,
            automatic_apply_count,
        },
        runtime_authority_changed: false,
    })
}

fn cleanup(inbox: &Path, applied_segment: u64) -> (usize, Option<String>) {
    match delete_acknowledged(inbox, applied_segment) {
        Ok(deleted) => (deleted, None),
        Err(error) => (0, Some(error.to_string())),
    }
}

fn invalid_data(error: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

struct UpdateLock {
    file: fs::File,
}

impl Drop for UpdateLock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn acquire_update_lock(package_path: &Path) -> io::Result<UpdateLock> {
    if let Some(parent) = package_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mut lock_name = package_path
        .file_name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "l4_cross_scene_v2.bin".into());
    lock_name.push(".update.lock");
    let lock_path = package_path.with_file_name(lock_name);
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(lock_path)?;
    loop {
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            return Ok(UpdateLock { file });
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            continue;
        }
        if error.kind() == io::ErrorKind::WouldBlock {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "another L4 incremental updater owns the package",
            ));
        }
        return Err(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_memory::TypingMemoryOutcome;
    use crate::typing_scene::{
        KeyboardGeometryId, LanguageId, LayoutId, SceneIdentityEvidence, ScriptFamily,
    };

    fn v3_row(episode: &str, outcome: TypingMemoryOutcome) -> serde_json::Value {
        let language = LanguageId::from_label("de").unwrap();
        let layout = LayoutId::from_label("xkb:de").unwrap();
        serde_json::json!({
            "schema": 3,
            "episode_id": episode,
            "context": ["wir", "arbeiten"],
            "from": "arbeitn",
            "to": "arbeiten",
            "operation": "replacement",
            "operation_code": 2,
            "operator": "other",
            "operator_code": 255,
            "source_language": "de",
            "source_language_id": language.code(),
            "target_language": "de",
            "target_language_id": language.code(),
            "source_layout": "xkb:de",
            "source_layout_id": layout.code(),
            "target_layout": "xkb:de",
            "target_layout_id": layout.code(),
            "source_script": "latin",
            "source_script_code": ScriptFamily::Latin.code(),
            "target_script": "latin",
            "target_script_code": ScriptFamily::Latin.code(),
            "keyboard_geometry": "pc105",
            "keyboard_geometry_id": KeyboardGeometryId::PC105.code(),
            "identity_evidence": "package",
            "identity_evidence_code": SceneIdentityEvidence::Package.code(),
            "sentence_language": "de",
            "sentence_language_id": language.code(),
            "sentence_language_support_milli": 900,
            "sentence_language_alternative_milli": 100,
            "sentence_language_observed_tokens": 2,
            "outcome": outcome.as_str(),
            "outcome_code": outcome.code()
        })
    }

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lay-l4-incremental-{name}-{}-{}",
            std::process::id(),
            crate::time::unix_timestamp()
        ))
    }

    #[test]
    fn updater_advances_checkpoint_and_merges_later_evidence_once() {
        let root = test_root("merge");
        let inbox = root.join("inbox");
        let package = root.join("l4_cross_scene_v2.bin");
        let first = super::super::segments::seal_episode_at(
            &inbox,
            vec![v3_row("episode-1", TypingMemoryOutcome::ConfirmedPositive)],
        )
        .unwrap();

        let first_report =
            update_package_from_inbox(&inbox, &package, CrossSceneCompileConfig::default())
                .unwrap();
        let first_package = read_package(&package).unwrap();

        assert_eq!(first.segment_id, 1);
        assert_eq!(first_report.applied_segment, 1);
        assert_eq!(first_package.applied_segment, 1);
        assert_eq!(first_package.positive_observations, 1);
        assert!(read_segments_after(&inbox, 0).unwrap().is_empty());

        let second = super::super::segments::seal_episode_at(
            &inbox,
            vec![v3_row("episode-2", TypingMemoryOutcome::ConfirmedNegative)],
        )
        .unwrap();
        let second_report =
            update_package_from_inbox(&inbox, &package, CrossSceneCompileConfig::default())
                .unwrap();
        let second_package = read_package(&package).unwrap();

        assert_eq!(second.segment_id, 2);
        assert_eq!(second_report.previous_applied_segment, 1);
        assert_eq!(second_report.applied_segment, 2);
        assert_eq!(second_package.positive_observations, 1);
        assert_eq!(second_package.negative_observations, 1);
        assert_eq!(second_package.profiles[0].positive_examples, 1);
        assert_eq!(second_package.profiles[0].negative_examples, 1);
        assert!(!second_package.profiles[0].positive.is_empty());
        assert!(!second_package.profiles[0].negative.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_segment_does_not_publish_or_acknowledge() {
        let root = test_root("reject");
        let inbox = root.join("inbox");
        let package = root.join("l4_cross_scene_v2.bin");
        super::super::segments::seal_episode_at(
            &inbox,
            vec![v3_row("episode-1", TypingMemoryOutcome::ConfirmedPositive)],
        )
        .unwrap();
        update_package_from_inbox(&inbox, &package, CrossSceneCompileConfig::default()).unwrap();
        let before = fs::read(&package).unwrap();
        let mut malformed = v3_row("episode-2", TypingMemoryOutcome::ConfirmedPositive);
        malformed["target_language_id"] = serde_json::json!(LanguageId::RUSSIAN.code());
        super::super::segments::seal_episode_at(&inbox, vec![malformed]).unwrap();

        assert!(
            update_package_from_inbox(&inbox, &package, CrossSceneCompileConfig::default())
                .is_err()
        );
        assert_eq!(fs::read(&package).unwrap(), before);
        assert_eq!(read_segments_after(&inbox, 1).unwrap().len(), 1);
        let _ = fs::remove_dir_all(root);
    }
}
