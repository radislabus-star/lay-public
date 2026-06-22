//! Non-blocking debug log writer for hot typing paths.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::OnceLock;
use std::time::Duration;

const ACTIVE_FLUSH_INTERVAL: Duration = Duration::from_millis(1000);
const IDLE_FLUSH_INTERVAL: Duration = Duration::from_millis(5000);
const IDLE_AFTER: Duration = Duration::from_millis(10_000);
const CHANNEL_CAPACITY_FALLBACK_DROP: usize = 2048;
const MAX_LOG_BYTES: u64 = 500 * 1024;

struct DebugLogLine {
    path: PathBuf,
    line: String,
}

static DEBUG_LOG_SENDER: OnceLock<Sender<DebugLogLine>> = OnceLock::new();

pub fn append_private_line(path: PathBuf, line: impl Into<String>) {
    let line = line.into();
    let sender = DEBUG_LOG_SENDER.get_or_init(spawn_debug_log_writer);
    let _ = sender.send(DebugLogLine { path, line });
}

fn spawn_debug_log_writer() -> Sender<DebugLogLine> {
    let (sender, receiver) = mpsc::channel::<DebugLogLine>();
    std::thread::Builder::new()
        .name("lay-debug-log-writer".to_string())
        .spawn(move || {
            let mut pending = BTreeMap::<PathBuf, String>::new();
            let mut pending_lines = 0usize;
            let mut last_line_at = std::time::Instant::now();
            loop {
                match receiver.recv_timeout(flush_interval(last_line_at)) {
                    Ok(line) => {
                        last_line_at = std::time::Instant::now();
                        push_pending(&mut pending, line);
                        pending_lines += 1;
                        if pending_lines >= CHANNEL_CAPACITY_FALLBACK_DROP {
                            flush_pending(&mut pending);
                            pending_lines = 0;
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if !pending.is_empty() {
                            flush_pending(&mut pending);
                            pending_lines = 0;
                        }
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

fn flush_interval(last_line_at: std::time::Instant) -> Duration {
    if last_line_at.elapsed() >= IDLE_AFTER {
        IDLE_FLUSH_INTERVAL
    } else {
        ACTIVE_FLUSH_INTERVAL
    }
}

fn push_pending(pending: &mut BTreeMap<PathBuf, String>, line: DebugLogLine) {
    let text = pending.entry(line.path).or_default();
    text.push_str(&line.line);
    if !line.line.ends_with('\n') {
        text.push('\n');
    }
}

fn flush_pending(pending: &mut BTreeMap<PathBuf, String>) {
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
    let keep_from = tail_line_boundary(&text, MAX_LOG_BYTES as usize);
    let compacted = &text[keep_from..];
    let _ = crate::private_file::write_private_text(path, compacted);
}

fn tail_line_boundary(text: &str, max_bytes: usize) -> usize {
    if text.len() <= max_bytes {
        return 0;
    }
    let start = text.len().saturating_sub(max_bytes);
    let start = text
        .char_indices()
        .find_map(|(idx, _)| (idx >= start).then_some(idx))
        .unwrap_or(start);
    text[..=start]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(start)
}

#[cfg(test)]
mod tests {
    use super::tail_line_boundary;

    #[test]
    fn tail_boundary_keeps_complete_lines() {
        let text = "one\ntwo\nthree\nfour\n";
        let start = tail_line_boundary(text, 10);
        assert_eq!(&text[start..], "three\nfour\n");
    }
}
