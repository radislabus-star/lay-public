use crate::text_metrics::{damerau_levenshtein, has_cyrillic, without_whitespace};

use super::types::{TypingCandidate, TypingCandidateFamily, TypingCandidateScore};

impl TypingCandidate {
    pub fn new(rule_id: &str, priority: i32, original: &str, replacement: String) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            priority,
            score: score_typing_candidate(original, &replacement, rule_id, priority),
            replacement,
        }
    }
}

pub fn classify_typing_rule(rule_id: &str) -> TypingCandidateFamily {
    crate::typing_rule_graph::typing_rule_family(rule_id).unwrap_or(TypingCandidateFamily::Unknown)
}

pub fn score_typing_candidate(
    original: &str,
    replacement: &str,
    rule_id: &str,
    priority: i32,
) -> TypingCandidateScore {
    let family = classify_typing_rule(rule_id);
    let family_weight = family_weight(rule_id, family);
    let language_delta = language_delta(original, replacement);
    let structure_bonus = structure_bonus(original, replacement);
    let edit_penalty = edit_penalty(original, replacement);
    let intervention_penalty = intervention_penalty(original, replacement);
    let priority_bonus = priority_bonus(priority);
    let total = family_weight + language_delta + structure_bonus + priority_bonus
        - edit_penalty
        - intervention_penalty;

    TypingCandidateScore {
        total,
        family,
        family_weight,
        language_delta,
        structure_bonus,
        edit_penalty,
        intervention_penalty,
        priority_bonus,
    }
}

fn family_weight(rule_id: &str, family: TypingCandidateFamily) -> f64 {
    crate::typing_rule_graph::typing_rule_family_weight(rule_id, family)
}

fn language_delta(original: &str, replacement: &str) -> f64 {
    if !has_cyrillic(original) && !has_cyrillic(replacement) {
        return 0.0;
    }
    crate::ngram::ru_candidate_margin(replacement, original).clamp(-4.0, 8.0) * 0.35
}

fn structure_bonus(original: &str, replacement: &str) -> f64 {
    let original_internal_spaces = internal_space_count(original);
    let replacement_internal_spaces = internal_space_count(replacement);
    let original_compact = without_whitespace(original);
    let replacement_compact = without_whitespace(replacement);

    if replacement_internal_spaces > original_internal_spaces
        && original_compact == replacement_compact
    {
        return 4.0;
    }
    if replacement_internal_spaces > original_internal_spaces {
        return 0.0;
    }
    0.0
}

fn edit_penalty(original: &str, replacement: &str) -> f64 {
    let original_chars = original.chars().count().max(1);
    let replacement_chars = replacement.chars().count().max(1);
    let distance = damerau_levenshtein(original, replacement) as f64;
    let scale = original_chars.max(replacement_chars) as f64;
    (distance / scale).min(1.0) * 4.0
}

fn intervention_penalty(original: &str, replacement: &str) -> f64 {
    if original == replacement {
        0.0
    } else {
        0.25
    }
}

fn priority_bonus(priority: i32) -> f64 {
    if priority <= 0 {
        return 0.0;
    }
    1.0 / priority as f64
}

fn internal_space_count(text: &str) -> usize {
    let trimmed = text.trim();
    trimmed.chars().filter(|ch| ch.is_whitespace()).count()
}
