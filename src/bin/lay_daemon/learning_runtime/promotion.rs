use lay::keyboard::is_cyrillic_letter;
use lay::typing_assist::{
    is_cyrillic_word, is_known_russian_word_or_form, remember_promoted_replacement,
};
use lay::word_buffer::UserLearningCorrection;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{runtime_log, LEARN_PROMOTION_THRESHOLD};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LearningPromotion {
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

pub(crate) fn promote_user_correction_if_repeated(
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
            runtime_log(&format!("learn promotion failed: {e}"));
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
    let text = serde_json::to_string_pretty(candidates).unwrap_or_else(|_| "{}".to_string());
    lay::private_file::write_private_text(path, &format!("{text}\n"))
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
    let text = serde_json::to_string_pretty(&rules).map_err(|e| e.to_string())?;
    lay::private_file::write_private_text(path, &format!("{text}\n")).map_err(|e| e.to_string())?;
    Ok(true)
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
