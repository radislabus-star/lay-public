use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use lay::nanda_wave::packet::{write_learned_packet, LearnedPacketEntry, PacketWriteReport};

const DEFAULT_CORRECTION_LOG_PATH: &str = ".local/share/lay/corrections.jsonl";
const EXPERIENCE_KIND: &str = "nanda_correction_experience_v1";

#[derive(Debug, Clone, Deserialize)]
struct LearningLogEntry {
    #[serde(default)]
    ts: u64,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: String,
    #[serde(default)]
    replace_words: usize,
    #[serde(default)]
    words: usize,
    #[serde(default)]
    lay_kind: Option<String>,
    #[serde(default)]
    lay_from: Option<String>,
    #[serde(default)]
    lay_to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorrectionExperience {
    pub kind: &'static str,
    pub ts: u64,
    pub signal: CorrectionSignal,
    pub source_kind: String,
    pub original: String,
    pub expected: String,
    pub operation: String,
    pub count_weight: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionSignal {
    AppliedObserved,
    UserAcceptedFix,
    UserRejectedLayOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningShadowReport {
    pub input: PathBuf,
    pub raw_lines: usize,
    pub invalid_lines: usize,
    pub experiences: usize,
    pub accepted_signals: usize,
    pub rejected_signals: usize,
    pub candidate_entries: usize,
    pub ready_entries: usize,
    pub min_count: usize,
    pub by_signal: BTreeMap<CorrectionSignal, usize>,
    pub rejected_pairs: Vec<LearnedPacketEntry>,
    pub ready: Vec<LearnedPacketEntry>,
}

pub fn default_correction_log_path() -> Option<PathBuf> {
    std::env::var_os("LAY_LEARNING_LOG")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(DEFAULT_CORRECTION_LOG_PATH))
        })
}

pub fn learning_shadow_report(path: &Path, min_count: usize) -> io::Result<LearningShadowReport> {
    let text = std::fs::read_to_string(path)?;
    Ok(learning_shadow_report_from_str(
        path.to_path_buf(),
        &text,
        min_count,
    ))
}

pub fn learning_shadow_report_from_str(
    input: PathBuf,
    text: &str,
    min_count: usize,
) -> LearningShadowReport {
    let min_count = min_count.max(1);
    let mut raw_lines = 0usize;
    let mut invalid_lines = 0usize;
    let mut experiences = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        raw_lines += 1;
        match serde_json::from_str::<LearningLogEntry>(line) {
            Ok(entry) => experiences.extend(experiences_from_log(entry)),
            Err(_) => invalid_lines += 1,
        }
    }
    summarize_experiences(input, raw_lines, invalid_lines, experiences, min_count)
}

pub fn pack_correction_learning(
    input: &Path,
    out: &Path,
    min_count: usize,
) -> io::Result<(LearningShadowReport, PacketWriteReport)> {
    let report = learning_shadow_report(input, min_count)?;
    let write = write_learned_packet(out, &report.ready)?;
    Ok((report, write))
}

fn experiences_from_log(entry: LearningLogEntry) -> Vec<CorrectionExperience> {
    if entry.from.trim().is_empty() || entry.to.trim().is_empty() || entry.from == entry.to {
        return Vec::new();
    }
    if entry.kind == "user-correction" {
        return user_correction_experiences(entry);
    }
    let Some(experience) = build_experience(
        entry.ts,
        CorrectionSignal::AppliedObserved,
        entry.kind,
        entry.from,
        entry.to,
        entry.replace_words.max(entry.words).max(1),
    ) else {
        return Vec::new();
    };
    vec![experience]
}

fn user_correction_experiences(entry: LearningLogEntry) -> Vec<CorrectionExperience> {
    let mut out = Vec::new();
    if let Some(experience) = build_experience(
        entry.ts,
        CorrectionSignal::UserAcceptedFix,
        entry.kind.clone(),
        entry.from.clone(),
        entry.to.clone(),
        entry.replace_words.max(entry.words).max(1),
    ) {
        out.push(experience);
    }
    if let (Some(lay_kind), Some(lay_from), Some(lay_to)) =
        (entry.lay_kind, entry.lay_from, entry.lay_to)
    {
        if let Some(rejected) = build_experience(
            entry.ts,
            CorrectionSignal::UserRejectedLayOutput,
            lay_kind,
            lay_from,
            lay_to,
            entry.replace_words.max(entry.words).max(1),
        ) {
            out.push(rejected);
        }
    }
    out
}

fn build_experience(
    ts: u64,
    signal: CorrectionSignal,
    source_kind: String,
    original: String,
    expected: String,
    count_weight: usize,
) -> Option<CorrectionExperience> {
    if !safe_learning_pair(&original, &expected) {
        return None;
    }
    Some(CorrectionExperience {
        kind: EXPERIENCE_KIND,
        ts,
        signal,
        source_kind,
        operation: classify_operation(&original, &expected),
        original,
        expected,
        count_weight: count_weight.max(1),
    })
}

fn summarize_experiences(
    input: PathBuf,
    raw_lines: usize,
    invalid_lines: usize,
    experiences: Vec<CorrectionExperience>,
    min_count: usize,
) -> LearningShadowReport {
    let mut by_signal = BTreeMap::<CorrectionSignal, usize>::new();
    let mut accepted = BTreeMap::<(String, String, String), usize>::new();
    let mut rejected = BTreeSet::<(String, String)>::new();
    for experience in &experiences {
        *by_signal.entry(experience.signal).or_default() += 1;
        match experience.signal {
            CorrectionSignal::AppliedObserved | CorrectionSignal::UserAcceptedFix => {
                *accepted
                    .entry((
                        experience.original.clone(),
                        experience.expected.clone(),
                        experience.operation.clone(),
                    ))
                    .or_default() += experience.count_weight.max(1);
            }
            CorrectionSignal::UserRejectedLayOutput => {
                rejected.insert((experience.original.clone(), experience.expected.clone()));
            }
        }
    }
    let rejected_pairs = rejected
        .iter()
        .map(|(original, expected)| LearnedPacketEntry {
            original: original.clone(),
            expected: expected.clone(),
            operation: "rejected".to_string(),
            count: 1,
        })
        .collect::<Vec<_>>();
    let mut ready = accepted
        .into_iter()
        .filter_map(|((original, expected, operation), count)| {
            (count >= min_count && !rejected.contains(&(original.clone(), expected.clone())))
                .then_some(LearnedPacketEntry {
                    original,
                    expected,
                    operation,
                    count,
                })
        })
        .collect::<Vec<_>>();
    ready.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.original.cmp(&right.original))
            .then_with(|| left.expected.cmp(&right.expected))
    });
    LearningShadowReport {
        input,
        raw_lines,
        invalid_lines,
        experiences: experiences.len(),
        accepted_signals: by_signal
            .iter()
            .filter(|(signal, _count)| **signal != CorrectionSignal::UserRejectedLayOutput)
            .map(|(_signal, count)| *count)
            .sum(),
        rejected_signals: *by_signal
            .get(&CorrectionSignal::UserRejectedLayOutput)
            .unwrap_or(&0),
        candidate_entries: ready.len() + rejected_pairs.len(),
        ready_entries: ready.len(),
        min_count,
        by_signal,
        rejected_pairs,
        ready,
    }
}

fn safe_learning_pair(original: &str, expected: &str) -> bool {
    let original = original.trim();
    let expected = expected.trim();
    if original.is_empty()
        || expected.is_empty()
        || original == expected
        || original.len() > 160
        || expected.len() > 160
        || contains_unsafe_learning_text(original)
        || contains_unsafe_learning_text(expected)
    {
        return false;
    }
    let original_words = original.split_whitespace().count().max(1);
    let expected_words = expected.split_whitespace().count().max(1);
    original_words <= 8 && expected_words <= 8 && original_words.abs_diff(expected_words) <= 3
}

fn contains_unsafe_learning_text(text: &str) -> bool {
    text.contains("://")
        || text.contains('@')
        || text.contains('=')
        || text.chars().any(|ch| ch.is_control())
        || text.split_whitespace().any(|token| {
            token.starts_with('-')
                || token.chars().filter(|ch| ch.is_ascii_punctuation()).count() >= 2
        })
}

fn classify_operation(original: &str, expected: &str) -> String {
    let original_words = original.split_whitespace().count();
    let expected_words = expected.split_whitespace().count();
    if original_words != expected_words {
        return "split".to_string();
    }
    let original_trimmed = original.trim();
    let expected_trimmed = expected.trim();
    let converted = lay::dict::convert(
        original_trimmed,
        lay::dict::detect_direction(original_trimmed),
    );
    if converted == expected_trimmed {
        return "layout".to_string();
    }
    if original_trimmed
        .chars()
        .any(lay::keyboard::is_cyrillic_letter)
        && expected_trimmed
            .chars()
            .any(lay::keyboard::is_cyrillic_letter)
    {
        return "typo".to_string();
    }
    "other".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_report_promotes_repeated_accepted_fix() {
        let text = r#"{"ts":1,"kind":"typing-assist","from":"эсперемнт ","to":"эксперимент ","replace_words":1,"words":1}
{"ts":2,"kind":"typing-assist","from":"эсперемнт ","to":"эксперимент ","replace_words":1,"words":1}
"#;
        let report = learning_shadow_report_from_str(PathBuf::from("test.jsonl"), text, 1);
        assert_eq!(report.raw_lines, 2);
        assert_eq!(report.ready_entries, 1);
        assert_eq!(report.ready[0].original, "эсперемнт ");
        assert_eq!(report.ready[0].expected, "эксперимент ");
        assert_eq!(report.ready[0].operation, "typo");
        assert_eq!(report.ready[0].count, 2);
    }

    #[test]
    fn shadow_report_blocks_rejected_lay_output() {
        let text = r#"{"ts":1,"kind":"typing-assist","from":"смотри ","to":"смотрин ","replace_words":1,"words":1}
{"ts":2,"kind":"typing-assist","from":"смотри ","to":"смотрин ","replace_words":1,"words":1}
{"ts":3,"kind":"user-correction","from":"смотрин ","to":"смотри ","replace_words":1,"words":1,"lay_kind":"typing-assist","lay_from":"смотри ","lay_to":"смотрин "}
"#;
        let report = learning_shadow_report_from_str(PathBuf::from("test.jsonl"), text, 1);
        assert_eq!(report.rejected_signals, 1);
        assert!(report
            .ready
            .iter()
            .all(|entry| entry.expected != "смотрин "));
        assert!(report
            .ready
            .iter()
            .any(|entry| entry.original == "смотрин " && entry.expected == "смотри "));
    }

    #[test]
    fn shadow_report_rejects_technical_noise() {
        let text = r#"{"ts":1,"kind":"typing-assist","from":"git checkout -b test","to":"git checkout в test","replace_words":4,"words":4}"#;
        let report = learning_shadow_report_from_str(PathBuf::from("test.jsonl"), text, 1);
        assert_eq!(report.experiences, 0);
        assert_eq!(report.ready_entries, 0);
    }
}
