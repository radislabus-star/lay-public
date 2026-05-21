use lay::keyboard::is_cyrillic_letter;
use lay::typing_assist::{
    is_cyrillic_word, is_known_russian_word_or_form, remember_promoted_replacement,
    REPLACEMENTS_PATH,
};
use lay::word_buffer::UserLearningCorrection;
use std::collections::BTreeMap;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{active_learning_log, log};

const LEARN_LOG_PATH: &str = ".local/share/lay/corrections.jsonl";
const LEARN_CANDIDATES_PATH: &str = ".local/share/lay/learning_candidates.json";
const LEARN_LOG_MAX_BYTES: u64 = 1024 * 1024;
const LEARN_LOG_KEEP_LINES: usize = 3000;
const LEARN_PROMOTION_THRESHOLD: u32 = 2;

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

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
}

pub(super) fn append_learning_log(
    kind: &str,
    from: &str,
    to: &str,
    replace_words: usize,
    words: usize,
) {
    if !active_learning_log() {
        return;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let path = std::path::PathBuf::from(home).join(LEARN_LOG_PATH);
    append_learning_log_to_path(&path, kind, from, to, replace_words, words);
}

pub(super) fn append_user_correction_learning_log(correction: &UserLearningCorrection) {
    if !active_learning_log() {
        return;
    }
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let home = std::path::PathBuf::from(home);
    let path = home.join(LEARN_LOG_PATH);
    append_user_correction_learning_log_to_path(&path, correction);
    match promote_user_correction_if_repeated(
        &home.join(LEARN_CANDIDATES_PATH),
        &home.join(REPLACEMENTS_PATH),
        correction,
    ) {
        LearningPromotion::Promoted { from, to } => {
            log(&format!("  learn: promoted exact rule {from:?} → {to:?}"));
        }
        LearningPromotion::Recorded { count, from, to } => {
            log(&format!(
                "  learn: candidate {from:?} → {to:?}, count={count}/{LEARN_PROMOTION_THRESHOLD}"
            ));
        }
        LearningPromotion::Skipped => {}
    }
}

pub(super) fn append_learning_log_to_path(
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
    };
    append_learning_entry_to_path(path, &entry);
}

pub(super) fn append_user_correction_learning_log_to_path(
    path: &std::path::Path,
    correction: &UserLearningCorrection,
) {
    let entry = LearningEntry {
        ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        kind: "user-correction",
        from: &correction.from,
        to: &correction.to,
        replace_words: correction.replace_words,
        words: correction.words,
        lay_kind: Some(&correction.lay_kind),
        lay_from: Some(&correction.lay_from),
        lay_to: Some(&correction.lay_to),
    };
    append_learning_entry_to_path(path, &entry);
}

fn append_learning_entry_to_path(path: &std::path::Path, entry: &LearningEntry<'_>) {
    if entry.from == entry.to || entry.from.trim().is_empty() || entry.to.trim().is_empty() {
        return;
    }

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log(&format!("⚠ learn-log mkdir failed: {e}"));
            return;
        }
    }

    let Ok(mut line) = serde_json::to_string(&entry) else {
        return;
    };
    line.push('\n');

    match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
    {
        Ok(mut f) => {
            if f.write_all(line.as_bytes()).is_ok() {
                compact_learning_log_if_needed(path);
                #[cfg(not(test))]
                lay::stats::record_learning_log_entry(entry.kind);
                log("  learn-log: correction saved");
            }
        }
        Err(e) => log(&format!("⚠ learn-log open failed: {e}")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LearningPromotion {
    Skipped,
    Recorded {
        from: String,
        to: String,
        count: u32,
    },
    Promoted {
        from: String,
        to: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct LearningCandidate {
    from: String,
    to: String,
    count: u32,
    first_ts: u64,
    last_ts: u64,
    promoted: bool,
}

pub(super) fn promote_user_correction_if_repeated(
    candidates_path: &std::path::Path,
    replacements_path: &std::path::Path,
    correction: &UserLearningCorrection,
) -> LearningPromotion {
    let Some((from, to)) = normalizable_learning_rule(correction) else {
        return LearningPromotion::Skipped;
    };

    let now = unix_timestamp();
    let key = format!("{from}\u{1f}{to}");
    let mut candidates = load_learning_candidates(candidates_path);
    let candidate = candidates.entry(key).or_insert_with(|| LearningCandidate {
        from: from.clone(),
        to: to.clone(),
        count: 0,
        first_ts: now,
        last_ts: now,
        promoted: false,
    });
    candidate.count = candidate.count.saturating_add(1);
    candidate.last_ts = now;

    if candidate.promoted {
        remember_promoted_replacement(&from, &to);
        let _ = save_learning_candidates(candidates_path, &candidates);
        return LearningPromotion::Promoted { from, to };
    }

    if candidate.count < LEARN_PROMOTION_THRESHOLD {
        let count = candidate.count;
        let _ = save_learning_candidates(candidates_path, &candidates);
        return LearningPromotion::Recorded { from, to, count };
    }

    match add_replacement_rule_to_path(replacements_path, &from, &to) {
        Ok(true) | Ok(false) => {
            candidate.promoted = true;
            remember_promoted_replacement(&from, &to);
            #[cfg(not(test))]
            lay::stats::record_learning_promotion();
            let _ = save_learning_candidates(candidates_path, &candidates);
            LearningPromotion::Promoted { from, to }
        }
        Err(e) => {
            log(&format!("⚠ learn promotion failed: {e}"));
            let _ = save_learning_candidates(candidates_path, &candidates);
            LearningPromotion::Skipped
        }
    }
}

fn normalizable_learning_rule(correction: &UserLearningCorrection) -> Option<(String, String)> {
    if correction.lay_kind == "layout-replay" {
        return None;
    }

    let from = correction.from.trim();
    let to = correction.to.trim();
    if from.is_empty() || to.is_empty() || from == to {
        return None;
    }
    if from.split_whitespace().count() != 1 || to.split_whitespace().count() > 3 {
        return None;
    }

    let from_lower = from.to_lowercase();
    let to_lower = to.to_lowercase();
    let from_letters = from_lower.chars().filter(|ch| ch.is_alphabetic()).count();
    let to_letters = to_lower.chars().filter(|ch| ch.is_alphabetic()).count();
    if from_letters < 4 || to_letters < 2 {
        return None;
    }
    if !is_cyrillic_word(&from_lower) {
        return None;
    }
    if !to_lower
        .chars()
        .all(|ch| is_cyrillic_letter(ch) || ch.is_whitespace() || ch == '-')
    {
        return None;
    }
    if is_known_russian_word_or_form(&from_lower) {
        return None;
    }

    Some((from_lower, to_lower))
}

fn load_learning_candidates(path: &std::path::Path) -> BTreeMap<String, LearningCandidate> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save_learning_candidates(
    path: &std::path::Path,
    candidates: &BTreeMap<String, LearningCandidate>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(candidates).unwrap_or_else(|_| "{}".to_string());
    std::fs::write(path, format!("{text}\n"))
}

fn add_replacement_rule_to_path(
    path: &std::path::Path,
    from: &str,
    to: &str,
) -> Result<bool, String> {
    let mut rules: BTreeMap<String, String> = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();

    if let Some(existing) = rules.get(from) {
        if existing == to {
            return Ok(false);
        }
        return Err(format!(
            "replacement conflict for {from:?}: existing {existing:?}, learned {to:?}"
        ));
    }

    rules.insert(from.to_string(), to.to_string());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(&rules).map_err(|e| e.to_string())?;
    std::fs::write(path, format!("{text}\n")).map_err(|e| e.to_string())?;
    Ok(true)
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
    let compacted = keep_last_jsonl_lines(&content, LEARN_LOG_KEEP_LINES);
    if std::fs::write(path, compacted).is_ok() {
        log("  learn-log: compacted");
    }
}

pub(super) fn keep_last_jsonl_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    let mut out = lines[start..].join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}
