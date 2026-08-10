use lay::word_buffer::UserLearningCorrection;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{runtime_log, LEARN_LOG_MAX_BYTES};

#[derive(serde::Serialize)]
struct LearningEntry<'a> {
    ts: u64,
    kind: &'a str,
    from: &'a str,
    to: &'a str,
    replace_words: usize,
    words: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    lay_kind: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lay_from: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lay_to: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_target: Option<&'a str>,
}

pub(crate) fn append_learning_log_to_path(
    path: &std::path::Path,
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
) {
    let entry = LearningEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        kind,
        from,
        to,
        replace_words,
        words,
        lay_kind: None,
        lay_from: None,
        lay_to: None,
        user_target: None,
    };
    append_learning_entry_to_path(path, &entry);
}

pub(crate) fn append_user_correction_learning_log_to_path(
    path: &std::path::Path,
    correction: &UserLearningCorrection,
) {
    append_correction_learning_log_to_path(path, "user-correction", correction);
}

pub(crate) fn append_reverted_system_apply_learning_log_to_path(
    path: &std::path::Path,
    correction: &UserLearningCorrection,
) {
    append_correction_learning_log_to_path(path, "system-apply-reverted", correction);
}

fn append_correction_learning_log_to_path(
    path: &std::path::Path,
    kind: &str,
    correction: &UserLearningCorrection,
) {
    let user_target = correction.user_target();
    let entry = LearningEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        kind,
        from: &correction.from,
        to: &correction.to,
        replace_words: correction.replace_words,
        words: correction.words,
        lay_kind: Some(&correction.lay_kind),
        lay_from: Some(&correction.lay_from),
        lay_to: Some(&correction.lay_to),
        user_target: user_target.as_deref(),
    };
    append_learning_entry_to_path(path, &entry);
}

fn append_learning_entry_to_path(path: &std::path::Path, entry: &LearningEntry<'_>) {
    if entry.from == entry.to || entry.from.trim().is_empty() || entry.to.trim().is_empty() {
        return;
    }

    let Ok(mut line) = serde_json::to_string(&entry) else {
        return;
    };
    line.push('\n');

    match lay::private_file::append_private_text(path, &line) {
        Ok(()) => {
            compact_learning_log_if_needed(path);
            #[cfg(not(test))]
            lay::stats::record_learning_log_entry(entry.kind);
            runtime_log("  learn-log: correction saved");
        }
        Err(e) => runtime_log(&format!("learn-log open failed: {e}")),
    }
}

fn compact_learning_log_if_needed(path: &std::path::Path) {
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    if meta.len() <= LEARN_LOG_MAX_BYTES {
        return;
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let compacted = keep_jsonl_tail_bytes(&content, LEARN_LOG_MAX_BYTES as usize);
    if lay::private_file::write_private_text(path, &compacted).is_ok() {
        runtime_log("  learn-log: compacted");
    }
}

#[cfg(test)]
pub(crate) fn keep_last_jsonl_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    let mut out = lines[start..].join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

pub(crate) fn keep_jsonl_tail_bytes(content: &str, max_bytes: usize) -> String {
    if content.len() <= max_bytes {
        return content.to_string();
    }
    let start = content.len().saturating_sub(max_bytes);
    let start = content
        .char_indices()
        .find_map(|(idx, _)| (idx >= start).then_some(idx))
        .unwrap_or(start);
    let keep_from = content[..start]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(start);
    let tail = &content[keep_from..];
    if tail.len() <= max_bytes {
        return tail.to_string();
    }
    let hard_start = tail.len().saturating_sub(max_bytes);
    let hard_start = tail
        .char_indices()
        .find_map(|(idx, _)| (idx >= hard_start).then_some(idx))
        .unwrap_or(hard_start);
    tail[hard_start..].to_string()
}
