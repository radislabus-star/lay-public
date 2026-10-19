//! Non-blocking debug log writer for hot typing paths.

#[cfg(not(test))]
use std::collections::BTreeMap;
use std::path::PathBuf;
#[cfg(not(test))]
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender, TrySendError};
#[cfg(not(test))]
use std::sync::OnceLock;
#[cfg(not(test))]
use std::time::Duration;

#[cfg(not(test))]
const ACTIVE_FLUSH_INTERVAL: Duration = Duration::from_millis(1000);
#[cfg(not(test))]
const IDLE_FLUSH_INTERVAL: Duration = Duration::from_millis(5000);
#[cfg(not(test))]
const IDLE_AFTER: Duration = Duration::from_millis(10_000);
#[cfg(not(test))]
const DEBUG_LOG_CHANNEL_CAPACITY: usize = 4096;
#[cfg(not(test))]
const PENDING_LINE_FLUSH_LIMIT: usize = 2048;
const MAX_LOG_BYTES: u64 = 500 * 1024;

pub fn ibus_runtime_trace_path() -> PathBuf {
    std::env::var_os("LAY_IBUS_TRACE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local/share/lay/ibus_engine_debug.jsonl")
        })
}

#[cfg(not(test))]
struct DebugLogLine {
    path: PathBuf,
    line: String,
}

#[cfg(not(test))]
static DEBUG_LOG_SENDER: OnceLock<SyncSender<DebugLogLine>> = OnceLock::new();

pub fn append_private_line(path: PathBuf, line: impl Into<String>) {
    let line = line.into();
    if !crate::config::runtime_debug_action_log() {
        return;
    }
    #[cfg(test)]
    {
        let text = if line.ends_with('\n') {
            line
        } else {
            format!("{line}\n")
        };
        if crate::private_file::append_private_text(&path, &text).is_ok() {
            compact_to_max_bytes(&path);
        }
    }
    #[cfg(not(test))]
    {
        let sender = DEBUG_LOG_SENDER.get_or_init(spawn_debug_log_writer);
        match sender.try_send(DebugLogLine { path, line }) {
            Ok(()) | Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {}
        }
    }
}

#[cfg(not(test))]
fn spawn_debug_log_writer() -> SyncSender<DebugLogLine> {
    let (sender, receiver) = mpsc::sync_channel::<DebugLogLine>(DEBUG_LOG_CHANNEL_CAPACITY);
    std::thread::Builder::new()
        .name("lay-debug-log-writer".to_string())
        .spawn(move || {
            let mut pending = BTreeMap::<PathBuf, String>::new();
            let mut pending_lines = 0usize;
            let mut last_line_at = std::time::Instant::now();
            let mut next_flush = last_line_at + ACTIVE_FLUSH_INTERVAL;
            loop {
                let timeout = next_flush.saturating_duration_since(std::time::Instant::now());
                match receiver.recv_timeout(timeout) {
                    Ok(line) => {
                        let was_empty = pending.is_empty();
                        last_line_at = std::time::Instant::now();
                        if was_empty {
                            next_flush = last_line_at + ACTIVE_FLUSH_INTERVAL;
                        }
                        push_pending(&mut pending, line);
                        pending_lines += 1;
                        if pending_lines >= PENDING_LINE_FLUSH_LIMIT {
                            flush_pending(&mut pending);
                            pending_lines = 0;
                            next_flush = std::time::Instant::now() + ACTIVE_FLUSH_INTERVAL;
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if !pending.is_empty() {
                            flush_pending(&mut pending);
                            pending_lines = 0;
                        }
                        next_flush = std::time::Instant::now() + flush_interval(last_line_at);
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        flush_pending(&mut pending);
                        break;
                    }
                }
            }
        })
        .expect("spawn lay debug log writer");
    sender
}

#[cfg(not(test))]
fn flush_interval(last_line_at: std::time::Instant) -> Duration {
    if last_line_at.elapsed() >= IDLE_AFTER {
        IDLE_FLUSH_INTERVAL
    } else {
        ACTIVE_FLUSH_INTERVAL
    }
}

#[cfg(not(test))]
fn push_pending(pending: &mut BTreeMap<PathBuf, String>, line: DebugLogLine) {
    let text = pending.entry(line.path).or_default();
    text.push_str(&line.line);
    if !line.line.ends_with('\n') {
        text.push('\n');
    }
}

#[cfg(not(test))]
fn flush_pending(pending: &mut BTreeMap<PathBuf, String>) {
    if !crate::config::runtime_debug_action_log() {
        pending.clear();
        return;
    }
    for (path, text) in std::mem::take(pending) {
        if crate::private_file::append_private_text(&path, &text).is_ok() {
            compact_to_max_bytes(&path);
        }
    }
}

fn compact_to_max_bytes(path: &PathBuf) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    if metadata.len() <= MAX_LOG_BYTES {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let compacted = tail_with_max_bytes(&text, MAX_LOG_BYTES as usize);
    let _ = crate::private_file::write_private_text(path, compacted);
}

fn tail_with_max_bytes(text: &str, max_bytes: usize) -> &str {
    let keep_from = tail_line_boundary(text, max_bytes);
    &text[keep_from..]
}

fn tail_line_boundary(text: &str, max_bytes: usize) -> usize {
    if text.len() <= max_bytes {
        return 0;
    }
    let start = text.len() - max_bytes;
    let bytes = text.as_bytes();
    if bytes[start - 1] == b'\n' {
        return start;
    }
    // Keep the newest complete records that fit; never retain a partial JSON line.
    // A newline boundary is also a UTF-8 boundary, even if the byte cut was not.
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|offset| start + offset + 1)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::{tail_line_boundary, tail_with_max_bytes};

    #[test]
    fn tail_boundary_keeps_complete_lines() {
        let text = "one\ntwo\nthree\nfour\n";
        let start = tail_line_boundary(text, 10);
        assert_eq!(&text[start..], "four\n");
        let exact_fit = tail_line_boundary(text, "three\nfour\n".len());
        assert_eq!(&text[exact_fit..], "three\nfour\n");
    }

    #[test]
    fn tail_with_max_bytes_never_exceeds_limit() {
        let text = "x".repeat(100);
        let compacted = tail_with_max_bytes(&text, 10);
        assert!(compacted.len() <= 10);
        assert!(
            compacted.is_empty(),
            "an oversized record must be dropped whole"
        );
    }

    #[test]
    fn tail_compaction_preserves_json_records_at_every_byte_budget() {
        let records = [
            "{\"event\":\"before\",\"text\":\"тест\"}\n",
            "{\"event\":\"key\",\"text\":\"я🙂\"}\n",
            "{\"event\":\"after\",\"text\":\"привет\"}\n",
        ];
        let text = records.concat();
        for budget in 0..=text.len() + 1 {
            let compacted = tail_with_max_bytes(&text, budget);
            assert!(compacted.len() <= budget, "budget={budget}");
            for line in compacted.lines() {
                serde_json::from_str::<serde_json::Value>(line)
                    .unwrap_or_else(|error| panic!("budget={budget}: {error}: {line}"));
            }
            let expected = (0..=records.len())
                .map(|start| records[start..].concat())
                .find(|suffix| suffix.len() <= budget)
                .unwrap();
            assert_eq!(compacted, expected, "budget={budget}");
        }
    }
}
