use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(test))]
use std::sync::mpsc::{self, SyncSender, TrySendError};
#[cfg(not(test))]
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SEGMENT_SCHEMA: u8 = 1;
const MAX_SEGMENT_BYTES: usize = 256 * 1024;
const MAX_INBOX_BYTES: u64 = 16 * 1024 * 1024;
#[cfg(not(test))]
const INBOX_QUEUE_CAPACITY: usize = 64;
const SEGMENT_PREFIX: &str = "segment-";
const SEGMENT_SUFFIX: &str = ".json";
const SEQUENCE_FILE: &str = ".segment-sequence";

static QUEUED_EPISODES: AtomicU64 = AtomicU64::new(0);
static SEALED_EPISODES: AtomicU64 = AtomicU64::new(0);
static REJECTED_EPISODES: AtomicU64 = AtomicU64::new(0);
static FAILED_EPISODES: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EpisodeEnqueueStatus {
    Queued,
    Incomplete,
    Oversized,
    QueueFull,
    WriterUnavailable,
}

#[derive(Clone, Debug)]
struct PendingEpisode {
    episode_id: String,
    rows: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SealedSegment {
    pub(super) segment_id: u64,
    pub(super) episode_id: String,
    pub(super) rows: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SegmentEnvelope {
    schema: u8,
    segment_id: u64,
    episode_id: String,
    row_count: u32,
    payload_sha256: String,
    rows: Vec<serde_json::Value>,
}

#[cfg(not(test))]
static INBOX_SENDER: OnceLock<Option<SyncSender<PendingEpisode>>> = OnceLock::new();

pub(crate) fn enqueue_episode(rows: Vec<serde_json::Value>) -> EpisodeEnqueueStatus {
    let Some(pending) = validate_pending_episode(rows) else {
        REJECTED_EPISODES.fetch_add(1, Ordering::Relaxed);
        return EpisodeEnqueueStatus::Incomplete;
    };
    if serialized_envelope_len(0, &pending).is_none_or(|bytes| bytes > MAX_SEGMENT_BYTES) {
        REJECTED_EPISODES.fetch_add(1, Ordering::Relaxed);
        return EpisodeEnqueueStatus::Oversized;
    }
    enqueue_pending(pending)
}

#[cfg(not(test))]
fn enqueue_pending(pending: PendingEpisode) -> EpisodeEnqueueStatus {
    let Some(sender) = INBOX_SENDER.get_or_init(spawn_inbox_writer).as_ref() else {
        FAILED_EPISODES.fetch_add(1, Ordering::Relaxed);
        return EpisodeEnqueueStatus::WriterUnavailable;
    };
    match sender.try_send(pending) {
        Ok(()) => {
            QUEUED_EPISODES.fetch_add(1, Ordering::Relaxed);
            EpisodeEnqueueStatus::Queued
        }
        Err(TrySendError::Full(_)) => {
            REJECTED_EPISODES.fetch_add(1, Ordering::Relaxed);
            EpisodeEnqueueStatus::QueueFull
        }
        Err(TrySendError::Disconnected(_)) => {
            FAILED_EPISODES.fetch_add(1, Ordering::Relaxed);
            EpisodeEnqueueStatus::WriterUnavailable
        }
    }
}

#[cfg(test)]
fn enqueue_pending(_pending: PendingEpisode) -> EpisodeEnqueueStatus {
    QUEUED_EPISODES.fetch_add(1, Ordering::Relaxed);
    EpisodeEnqueueStatus::Queued
}

#[cfg(not(test))]
fn spawn_inbox_writer() -> Option<SyncSender<PendingEpisode>> {
    let root = default_inbox_path()?;
    let (sender, receiver) = mpsc::sync_channel::<PendingEpisode>(INBOX_QUEUE_CAPACITY);
    std::thread::Builder::new()
        .name("lay-l4-segment-writer".to_string())
        .spawn(move || {
            while let Ok(pending) = receiver.recv() {
                match seal_pending_episode(&root, pending) {
                    Ok(_) => {
                        SEALED_EPISODES.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        FAILED_EPISODES.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        })
        .ok()?;
    Some(sender)
}

pub(super) fn seal_episode_at(
    root: &Path,
    rows: Vec<serde_json::Value>,
) -> io::Result<SealedSegment> {
    let pending = validate_pending_episode(rows)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "incomplete L4 episode"))?;
    seal_pending_episode(root, pending)
}

fn seal_pending_episode(root: &Path, pending: PendingEpisode) -> io::Result<SealedSegment> {
    fs::create_dir_all(root)?;
    let status = inbox_disk_status(root)?;
    let sequence_floor = read_sequence_floor(root)?;
    let segment_id = status
        .highest_segment
        .max(sequence_floor)
        .checked_add(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "L4 segment id exhausted"))?;
    let envelope = build_envelope(segment_id, &pending)?;
    let bytes = serde_json::to_vec(&envelope).map_err(io::Error::other)?;
    if bytes.len() > MAX_SEGMENT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L4 episode exceeds segment budget",
        ));
    }
    if status.bytes.saturating_add(bytes.len() as u64) > MAX_INBOX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::StorageFull,
            "L4 segment inbox budget exhausted",
        ));
    }
    let path = segment_path(root, segment_id);
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "L4 segment identifier already exists",
        ));
    }
    crate::private_file::write_private_bytes_atomic(&path, &bytes)?;
    sync_directory(root)?;
    let sealed = read_segment_path(&path)?;
    if sealed.segment_id != segment_id || sealed.episode_id != pending.episode_id {
        let _ = fs::remove_file(&path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "published L4 segment identity mismatch",
        ));
    }
    advance_sequence_floor(root, segment_id)?;
    Ok(sealed)
}

pub(super) fn read_segments_after(
    root: &Path,
    applied_segment: u64,
) -> io::Result<Vec<SealedSegment>> {
    let mut paths = segment_paths(root)?;
    paths.sort_by_key(|(segment_id, _)| *segment_id);
    let mut segments = Vec::new();
    let mut previous = applied_segment;
    for (segment_id, path) in paths {
        if segment_id <= applied_segment {
            continue;
        }
        if segment_id <= previous {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L4 segment order is not strictly increasing",
            ));
        }
        let segment = read_segment_path(&path)?;
        if segment.segment_id != segment_id {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L4 segment filename identity mismatch",
            ));
        }
        previous = segment_id;
        segments.push(segment);
    }
    Ok(segments)
}

pub(super) fn delete_acknowledged(root: &Path, applied_segment: u64) -> io::Result<usize> {
    advance_sequence_floor(root, applied_segment)?;
    let mut deleted = 0usize;
    for (segment_id, path) in segment_paths(root)? {
        if segment_id <= applied_segment {
            fs::remove_file(path)?;
            deleted = deleted.saturating_add(1);
        }
    }
    if deleted > 0 {
        sync_directory(root)?;
    }
    Ok(deleted)
}

pub(crate) fn status_json(path: Option<&Path>) -> serde_json::Value {
    let root = path
        .map(Path::to_path_buf)
        .or_else(default_inbox_path)
        .unwrap_or_default();
    match inbox_disk_status(&root).and_then(|status| {
        read_sequence_floor(&root).map(|sequence_floor| (status, sequence_floor))
    }) {
        Ok((status, sequence_floor)) => serde_json::json!({
            "path": root,
            "segments": status.segments,
            "bytes": status.bytes,
            "highest_segment": status.highest_segment,
            "sequence_floor": sequence_floor,
            "next_segment": status.highest_segment.max(sequence_floor).saturating_add(1),
            "budget_bytes": MAX_INBOX_BYTES,
            "queued_episodes": QUEUED_EPISODES.load(Ordering::Relaxed),
            "sealed_episodes": SEALED_EPISODES.load(Ordering::Relaxed),
            "rejected_episodes": REJECTED_EPISODES.load(Ordering::Relaxed),
            "failed_episodes": FAILED_EPISODES.load(Ordering::Relaxed),
            "learning_paused": status.bytes >= MAX_INBOX_BYTES,
        }),
        Err(error) => serde_json::json!({
            "path": root,
            "error": error.to_string(),
            "queued_episodes": QUEUED_EPISODES.load(Ordering::Relaxed),
            "sealed_episodes": SEALED_EPISODES.load(Ordering::Relaxed),
            "rejected_episodes": REJECTED_EPISODES.load(Ordering::Relaxed),
            "failed_episodes": FAILED_EPISODES.load(Ordering::Relaxed),
            "learning_paused": true,
        }),
    }
}

fn validate_pending_episode(rows: Vec<serde_json::Value>) -> Option<PendingEpisode> {
    if rows.is_empty() {
        return None;
    }
    let episode_id = rows.first()?.get("episode_id")?.as_str()?.trim();
    if episode_id.is_empty()
        || episode_id.len() > 128
        || rows.iter().any(|row| {
            row.get("schema").and_then(serde_json::Value::as_u64) != Some(3)
                || row.get("episode_id").and_then(serde_json::Value::as_str) != Some(episode_id)
        })
    {
        return None;
    }
    Some(PendingEpisode {
        episode_id: episode_id.to_string(),
        rows,
    })
}

fn serialized_envelope_len(segment_id: u64, pending: &PendingEpisode) -> Option<usize> {
    let envelope = build_envelope(segment_id, pending).ok()?;
    serde_json::to_vec(&envelope).ok().map(|bytes| bytes.len())
}

fn build_envelope(segment_id: u64, pending: &PendingEpisode) -> io::Result<SegmentEnvelope> {
    Ok(SegmentEnvelope {
        schema: SEGMENT_SCHEMA,
        segment_id,
        episode_id: pending.episode_id.clone(),
        row_count: pending
            .rows
            .len()
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "too many L4 episode rows"))?,
        payload_sha256: payload_sha256(&pending.episode_id, &pending.rows)?,
        rows: pending.rows.clone(),
    })
}

fn read_segment_path(path: &Path) -> io::Result<SealedSegment> {
    let bytes = fs::read(path)?;
    if bytes.len() > MAX_SEGMENT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L4 segment exceeds size budget",
        ));
    }
    let envelope: SegmentEnvelope = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if envelope.schema != SEGMENT_SCHEMA
        || envelope.segment_id == 0
        || envelope.row_count as usize != envelope.rows.len()
        || envelope.episode_id.trim().is_empty()
        || envelope.rows.iter().any(|row| {
            row.get("schema").and_then(serde_json::Value::as_u64) != Some(3)
                || row.get("episode_id").and_then(serde_json::Value::as_str)
                    != Some(envelope.episode_id.as_str())
        })
        || payload_sha256(&envelope.episode_id, &envelope.rows)? != envelope.payload_sha256
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid L4 sealed segment envelope",
        ));
    }
    Ok(SealedSegment {
        segment_id: envelope.segment_id,
        episode_id: envelope.episode_id,
        rows: envelope.rows,
    })
}

fn payload_sha256(episode_id: &str, rows: &[serde_json::Value]) -> io::Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(episode_id.as_bytes());
    hasher.update([0]);
    hasher.update(serde_json::to_vec(rows).map_err(io::Error::other)?);
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Clone, Copy, Debug, Default)]
struct InboxDiskStatus {
    segments: usize,
    bytes: u64,
    highest_segment: u64,
}

fn inbox_disk_status(root: &Path) -> io::Result<InboxDiskStatus> {
    let mut status = InboxDiskStatus::default();
    for (segment_id, path) in segment_paths(root)? {
        status.segments = status.segments.saturating_add(1);
        status.bytes = status.bytes.saturating_add(fs::metadata(path)?.len());
        status.highest_segment = status.highest_segment.max(segment_id);
    }
    Ok(status)
}

fn segment_paths(root: &Path) -> io::Result<Vec<(u64, PathBuf)>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(segment_id) = parse_segment_name(name) else {
            continue;
        };
        if !entry.file_type()?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L4 segment path is not a regular file",
            ));
        }
        paths.push((segment_id, path));
    }
    Ok(paths)
}

fn parse_segment_name(name: &str) -> Option<u64> {
    name.strip_prefix(SEGMENT_PREFIX)?
        .strip_suffix(SEGMENT_SUFFIX)?
        .parse()
        .ok()
}

fn segment_path(root: &Path, segment_id: u64) -> PathBuf {
    root.join(format!("{SEGMENT_PREFIX}{segment_id:020}{SEGMENT_SUFFIX}"))
}

fn sequence_path(root: &Path) -> PathBuf {
    root.join(SEQUENCE_FILE)
}

fn read_sequence_floor(root: &Path) -> io::Result<u64> {
    let path = sequence_path(root);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L4 segment sequence is not a regular file",
        ));
    }
    let text = fs::read_to_string(path)?;
    text.trim().parse::<u64>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid L4 segment sequence floor",
        )
    })
}

fn advance_sequence_floor(root: &Path, floor: u64) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let current = read_sequence_floor(root)?;
    if current >= floor {
        return Ok(());
    }
    crate::private_file::write_private_bytes_atomic(
        &sequence_path(root),
        format!("{floor}\n").as_bytes(),
    )?;
    sync_directory(root)
}

fn sync_directory(root: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::File::open(root)?.sync_all()?;
    }
    Ok(())
}

fn default_inbox_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("LAY_L4_CROSS_SCENE_INBOX") {
        return Some(PathBuf::from(path));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/share/lay/nanda_wave/l4_cross_scene_inbox"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(episode: &str, index: u64) -> serde_json::Value {
        serde_json::json!({
            "schema": 3,
            "episode_id": episode,
            "row": index
        })
    }

    #[test]
    fn segment_is_immutable_complete_and_checksum_bound() {
        let root = std::env::temp_dir().join(format!(
            "lay-l4-segments-{}-{}",
            std::process::id(),
            crate::time::unix_timestamp()
        ));
        let first = seal_episode_at(&root, vec![row("episode-a", 1), row("episode-a", 2)])
            .expect("seal first episode");
        let second =
            seal_episode_at(&root, vec![row("episode-b", 1)]).expect("seal second episode");

        assert_eq!(first.segment_id, 1);
        assert_eq!(second.segment_id, 2);
        assert_eq!(read_segments_after(&root, 1).unwrap(), vec![second]);
        assert_eq!(delete_acknowledged(&root, 1).unwrap(), 1);
        assert_eq!(read_segments_after(&root, 1).unwrap().len(), 1);

        let remaining = segment_path(&root, 2);
        let mut bytes = fs::read(&remaining).unwrap();
        let index = bytes.len() / 2;
        bytes[index] ^= 1;
        fs::write(&remaining, bytes).unwrap();
        assert!(read_segments_after(&root, 1).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn mixed_or_missing_episode_rows_are_rejected_before_queueing() {
        assert_eq!(
            enqueue_episode(vec![row("a", 1), row("b", 2)]),
            EpisodeEnqueueStatus::Incomplete
        );
        assert_eq!(
            enqueue_episode(vec![serde_json::json!({"schema": 3})]),
            EpisodeEnqueueStatus::Incomplete
        );
    }

    #[test]
    fn acknowledged_deletion_does_not_reset_segment_identity() {
        let root = std::env::temp_dir().join(format!(
            "lay-l4-segment-sequence-{}-{}",
            std::process::id(),
            crate::time::unix_timestamp()
        ));
        let first = seal_episode_at(&root, vec![row("episode-a", 1)]).unwrap();
        assert_eq!(first.segment_id, 1);
        assert_eq!(delete_acknowledged(&root, 1).unwrap(), 1);

        let second = seal_episode_at(&root, vec![row("episode-b", 1)]).unwrap();

        assert_eq!(second.segment_id, 2);
        assert_eq!(read_sequence_floor(&root).unwrap(), 2);
        let status = status_json(Some(&root));
        assert_eq!(status["highest_segment"], 2);
        assert_eq!(status["sequence_floor"], 2);
        assert_eq!(status["next_segment"], 3);
        let _ = fs::remove_dir_all(root);
    }
}
