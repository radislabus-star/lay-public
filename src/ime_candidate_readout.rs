//! Shared IME candidate readout helpers.
//!
//! Candidate scoring lives here so IBus stays an adapter that renders a
//! readout instead of owning Bayes or phrase-ranking policy.

use crate::nanda_wave::UsagePriorSnapshot;
use crate::word_reader::split_last_alphabetic_token;

#[derive(Debug, Clone)]
pub struct RankedImeSuffix {
    pub suffix: String,
    pub score: f32,
    pub order: usize,
}

pub fn push_unique_suffix(candidates: &mut Vec<String>, suffix: Option<String>) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty()
        || !is_allowed_visible_completion_suffix(&suffix)
        || candidates.iter().any(|candidate| candidate == &suffix)
    {
        return;
    }
    candidates.push(suffix);
}

pub fn push_unique_ranked_suffix(
    candidates: &mut Vec<RankedImeSuffix>,
    suffix: Option<String>,
    score: f32,
) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty() || !is_allowed_visible_completion_suffix(&suffix) {
        return;
    }
    if let Some(existing) = candidates
        .iter_mut()
        .find(|candidate| candidate.suffix == suffix)
    {
        if score > existing.score {
            existing.score = score;
        }
        return;
    }
    let order = candidates.len();
    candidates.push(RankedImeSuffix {
        suffix,
        score,
        order,
    });
}

pub fn push_unique_ascii_known_suffix(candidates: &mut Vec<String>, token: &str, suffix: String) {
    if suffix.is_empty() || candidates.iter().any(|candidate| candidate == &suffix) {
        return;
    }
    let completed = format!("{token}{suffix}").to_ascii_lowercase();
    let one_ascii_char =
        suffix.chars().count() == 1 && suffix.chars().all(|ch| ch.is_ascii_alphabetic());
    if one_ascii_char && !crate::lexicon::is_common_en_technical_word(&completed) {
        return;
    }
    if one_ascii_char || is_allowed_visible_completion_suffix(&suffix) {
        candidates.push(suffix);
    }
}

pub fn is_allowed_visible_completion_suffix(suffix: &str) -> bool {
    let trimmed = suffix.trim();
    let mut chars = trimmed.chars();
    let Some(ch) = chars.next() else {
        return false;
    };
    if chars.next().is_some() {
        return true;
    }
    matches!(ch, 'и' | 'я' | 'I' | 'a')
}

pub fn is_noisy_first_russian_prefix(prefix: &str) -> bool {
    matches!(prefix, "нев" | "инт")
}

pub fn is_command_like_long_tail(tail: &str) -> bool {
    let mut word_count = 0usize;
    let mut uppercase = 0usize;
    let mut lowercase = 0usize;
    for token in tail.split_whitespace() {
        let mut has_alpha = false;
        for ch in token.chars().filter(|ch| ch.is_alphabetic()) {
            has_alpha = true;
            if ch.is_uppercase() {
                uppercase += 1;
            }
            if ch.is_lowercase() {
                lowercase += 1;
            }
        }
        if has_alpha {
            word_count += 1;
        }
    }
    word_count >= 4 && uppercase >= 12 && uppercase >= lowercase.saturating_mul(2).max(1)
}

pub fn compare_suffix_len_for_prefix(
    partial_len: usize,
    left: &str,
    right: &str,
) -> std::cmp::Ordering {
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    if partial_len <= 3 {
        return right_len.cmp(&left_len);
    }
    left_len.cmp(&right_len)
}

pub fn preedit_suffix_bayes_score(
    usage: &UsagePriorSnapshot,
    tail: &str,
    suffix: &str,
    base: f32,
) -> f32 {
    let Some((context, word)) = preedit_suffix_context_and_word(tail, suffix) else {
        return base;
    };
    (base
        + usage.word_prior(&word) * 1.45
        + usage.context_word_prior(&context, &word) * 1.90
        + usage.accepted_word_count(&word).min(12) as f32 * 0.018)
        .clamp(0.0, 1.0)
}

pub fn preedit_suffix_context_and_word(tail: &str, suffix: &str) -> Option<(Vec<String>, String)> {
    let tail = tail.trim_end();
    let suffix_starts_new_word = suffix.chars().next().is_some_and(char::is_whitespace);
    if suffix_starts_new_word || tail.is_empty() {
        let word = suffix.split_whitespace().next()?.to_lowercase();
        let context = crate::nanda_wave::llmwave::tokenize(tail);
        return Some((context, word));
    }
    let (prefix, partial) = split_last_alphabetic_token(tail)?;
    let suffix_word_part = suffix.split_whitespace().next().unwrap_or(suffix);
    let word = format!(
        "{}{}",
        partial.to_lowercase(),
        suffix_word_part.to_lowercase()
    );
    let context = crate::nanda_wave::llmwave::tokenize(prefix);
    Some((context, word))
}

pub fn phrase_candidate_suffix(
    tail: &str,
    candidate: &str,
    max_suffix_chars: usize,
) -> Option<String> {
    let suffix = candidate.strip_prefix(tail)?;
    let suffix = if tail.ends_with(char::is_whitespace) {
        suffix.trim_start_matches(char::is_whitespace)
    } else {
        suffix
    };
    let suffix = next_word_suffix(suffix)?;
    (!suffix.is_empty() && suffix.chars().count() <= max_suffix_chars).then_some(suffix)
}

pub fn should_query_llmwave_phrase_suffix(tail: &str) -> bool {
    if tail.ends_with(char::is_whitespace) {
        return true;
    }
    let trimmed = tail.trim_end();
    let Some((left, token)) = trimmed.rsplit_once(char::is_whitespace) else {
        return false;
    };
    let token_chars = token.chars().count();
    (1..=6).contains(&token_chars)
        && !left.split_whitespace().collect::<Vec<_>>().is_empty()
        && token.chars().all(|ch| ch.is_alphabetic())
}

fn next_word_suffix(suffix: &str) -> Option<String> {
    let leading_space = suffix.chars().next().is_some_and(char::is_whitespace);
    let word = suffix.split_whitespace().next()?;
    if leading_space {
        Some(format!(" {word}"))
    } else {
        Some(word.to_string())
    }
}
