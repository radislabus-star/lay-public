use super::feedback::OnlineState;
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::thread;
use std::time::Duration;

const TAIL_HASHES: usize = 32;
const SNAPSHOT_ATTEMPTS: usize = 5;
const SNAPSHOT_SETTLE: Duration = Duration::from_millis(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JournalReadMode {
    Append,
    Compacted,
    Reanchored,
}

pub(super) struct JournalBatch {
    pub(super) text: String,
    pub(super) mode: JournalReadMode,
    pub(super) overlap_lines: usize,
}

pub(super) struct JournalSnapshot {
    text: String,
    device: u64,
    inode: u64,
    complete_bytes: u64,
    tail_hashes: Vec<u64>,
}

impl JournalSnapshot {
    pub(super) fn text(&self) -> &str {
        &self.text
    }

    pub(super) fn complete_bytes(&self) -> u64 {
        self.complete_bytes
    }

    pub(super) fn anchor(self, state: &mut OnlineState) {
        state.source_device = self.device;
        state.source_inode = self.inode;
        state.source_offset = self.complete_bytes;
        state.source_tail_hashes = self.tail_hashes;
    }
}

pub(super) fn initialize_cursor(path: &Path, state: &mut OnlineState) -> io::Result<bool> {
    if state.source_inode != 0 {
        return Ok(false);
    }
    read_full_snapshot(path)?.anchor(state);
    Ok(true)
}

pub(super) fn read_full_snapshot(path: &Path) -> io::Result<JournalSnapshot> {
    read_stable_snapshot(path)
}

pub(super) fn read_new_events(path: &Path, state: &mut OnlineState) -> io::Result<JournalBatch> {
    let snapshot = read_stable_snapshot(path)?;
    if snapshot.device == 0 {
        return Ok(JournalBatch {
            text: String::new(),
            mode: JournalReadMode::Append,
            overlap_lines: 0,
        });
    }

    let previous_tail = state.source_tail_hashes.clone();
    let current_lines = snapshot.text.lines().collect::<Vec<_>>();
    let current_hashes = current_lines
        .iter()
        .map(|line| line_hash(line))
        .collect::<Vec<_>>();
    if previous_tail.is_empty() && state.source_offset == 0 {
        let text = lines_as_jsonl(&current_lines);
        snapshot.anchor(state);
        return Ok(JournalBatch {
            text,
            mode: JournalReadMode::Append,
            overlap_lines: 0,
        });
    }
    let overlap = overlap_end(&previous_tail, &current_hashes);
    let (mode, overlap_lines, text) = match overlap {
        Some((end, matched)) => {
            let compacted = state.source_device != snapshot.device
                || state.source_inode != snapshot.inode
                || snapshot.complete_bytes < state.source_offset
                || matched < previous_tail.len();
            (
                if compacted {
                    JournalReadMode::Compacted
                } else {
                    JournalReadMode::Append
                },
                matched,
                lines_as_jsonl(&current_lines[end..]),
            )
        }
        None => (JournalReadMode::Reanchored, 0, String::new()),
    };
    snapshot.anchor(state);
    Ok(JournalBatch {
        text,
        mode,
        overlap_lines,
    })
}

fn read_stable_snapshot(path: &Path) -> io::Result<JournalSnapshot> {
    for _ in 0..SNAPSHOT_ATTEMPTS {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(empty_snapshot()),
            Err(error) => return Err(error),
        };
        let before = file.metadata()?;
        let mut bytes = Vec::with_capacity(before.len() as usize);
        file.read_to_end(&mut bytes)?;
        let after_read = file.metadata()?;
        thread::sleep(SNAPSHOT_SETTLE);
        let settled = file.metadata()?;
        if same_stamp(&before, &after_read)
            && same_stamp(&after_read, &settled)
            && bytes.len() as u64 == settled.len()
        {
            return Ok(snapshot_from_bytes(&settled, &bytes));
        }
    }
    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "usage journal did not reach a stable snapshot",
    ))
}

fn same_stamp(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
}

fn empty_snapshot() -> JournalSnapshot {
    JournalSnapshot {
        text: String::new(),
        device: 0,
        inode: 0,
        complete_bytes: 0,
        tail_hashes: Vec::new(),
    }
}

fn snapshot_from_bytes(metadata: &std::fs::Metadata, bytes: &[u8]) -> JournalSnapshot {
    let complete = complete_prefix_len(bytes);
    let text = String::from_utf8_lossy(&bytes[..complete]).into_owned();
    let hashes = text.lines().map(line_hash).collect::<Vec<_>>();
    JournalSnapshot {
        text,
        device: metadata.dev(),
        inode: metadata.ino(),
        complete_bytes: complete as u64,
        tail_hashes: tail(&hashes),
    }
}

fn complete_prefix_len(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|index| index + 1)
        .unwrap_or(0)
}

fn line_hash(line: &str) -> u64 {
    line.as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

fn tail(hashes: &[u64]) -> Vec<u64> {
    hashes[hashes.len().saturating_sub(TAIL_HASHES)..].to_vec()
}

fn overlap_end(previous_tail: &[u64], current: &[u64]) -> Option<(usize, usize)> {
    let max = previous_tail.len().min(current.len());
    for matched in (1..=max).rev() {
        let expected = &previous_tail[previous_tail.len() - matched..];
        if let Some(start) = current
            .windows(matched)
            .rposition(|window| window == expected)
        {
            return Some((start + matched, matched));
        }
    }
    None
}

fn lines_as_jsonl(lines: &[&str]) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "lay-l3-journal-{name}-{}-{stamp}.jsonl",
            std::process::id()
        ))
    }

    #[test]
    fn append_reads_only_new_complete_lines() {
        let path = temp_path("append");
        fs::write(&path, b"a\n").unwrap();
        let mut state = OnlineState::default();
        assert!(initialize_cursor(&path, &mut state).unwrap());

        OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"b\npartial")
            .unwrap();
        let batch = read_new_events(&path, &mut state).unwrap();

        assert_eq!(batch.mode, JournalReadMode::Append);
        assert_eq!(batch.text, "b\n");
        assert_eq!(state.source_offset, 4);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn atomic_tail_compaction_reads_only_lines_after_overlap() {
        let path = temp_path("compact");
        let temporary = path.with_extension("tmp");
        fs::write(&path, b"a\nb\nc\n").unwrap();
        let mut state = OnlineState::default();
        initialize_cursor(&path, &mut state).unwrap();

        fs::write(&temporary, b"b\nc\nd\n").unwrap();
        fs::rename(&temporary, &path).unwrap();
        let batch = read_new_events(&path, &mut state).unwrap();

        assert_eq!(batch.mode, JournalReadMode::Compacted);
        assert_eq!(batch.overlap_lines, 2);
        assert_eq!(batch.text, "d\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn same_inode_truncate_compaction_reads_only_lines_after_overlap() {
        let path = temp_path("truncate");
        fs::write(&path, b"a\nb\nc\n").unwrap();
        let inode = fs::metadata(&path).unwrap().ino();
        let mut state = OnlineState::default();
        initialize_cursor(&path, &mut state).unwrap();

        fs::write(&path, b"b\nc\nd\n").unwrap();
        assert_eq!(fs::metadata(&path).unwrap().ino(), inode);
        let batch = read_new_events(&path, &mut state).unwrap();

        assert_eq!(batch.mode, JournalReadMode::Compacted);
        assert_eq!(batch.overlap_lines, 2);
        assert_eq!(batch.text, "d\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rotation_without_overlap_reanchors_without_replay() {
        let path = temp_path("reanchor");
        let temporary = path.with_extension("tmp");
        fs::write(&path, b"a\nb\n").unwrap();
        let mut state = OnlineState::default();
        initialize_cursor(&path, &mut state).unwrap();

        fs::write(&temporary, b"x\ny\n").unwrap();
        fs::rename(&temporary, &path).unwrap();
        let batch = read_new_events(&path, &mut state).unwrap();
        assert_eq!(batch.mode, JournalReadMode::Reanchored);
        assert!(batch.text.is_empty());

        OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"z\n")
            .unwrap();
        let batch = read_new_events(&path, &mut state).unwrap();
        assert_eq!(batch.text, "z\n");
        fs::remove_file(path).unwrap();
    }
}
