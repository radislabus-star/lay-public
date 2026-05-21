//! Candidate ranking for typing assist.
//!
//! Correction rules should generate possible replacements. This module is the
//! narrow place that decides which generated candidate is the safest/best one.

use crate::text_metrics::{damerau_levenshtein, has_cyrillic, without_whitespace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingCandidateFamily {
    Exact,
    Visual,
    Layout,
    Structural,
    Typo,
    Cleanup,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingCandidateScore {
    pub total: f64,
    pub family: TypingCandidateFamily,
    pub family_weight: f64,
    pub language_delta: f64,
    pub structure_bonus: f64,
    pub edit_penalty: f64,
    pub intervention_penalty: f64,
    pub priority_bonus: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypingCandidate {
    pub rule_id: String,
    pub priority: i32,
    pub replacement: String,
    pub score: TypingCandidateScore,
}

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

pub fn choose_typing_candidate<I>(candidates: I) -> Option<TypingCandidate>
where
    I: IntoIterator<Item = TypingCandidate>,
{
    let mut best: Option<TypingCandidate> = None;

    for candidate in candidates {
        if candidate.replacement.trim().is_empty() {
            continue;
        }
        let better = match best.as_ref() {
            Some(current) => candidate_is_better(&candidate, current),
            None => true,
        };
        if better {
            best = Some(candidate);
        }
    }

    best
}

pub fn classify_typing_rule(rule_id: &str) -> TypingCandidateFamily {
    crate::typing_rule_graph::find_typing_rule(rule_id)
        .map(|rule| rule.family)
        .unwrap_or(TypingCandidateFamily::Unknown)
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

fn candidate_is_better(left: &TypingCandidate, right: &TypingCandidate) -> bool {
    const EPSILON: f64 = 0.000_001;
    let diff = left.score.total - right.score.total;
    if diff.abs() > EPSILON {
        return diff > 0.0;
    }
    if left.priority != right.priority {
        return left.priority < right.priority;
    }
    if left.rule_id != right.rule_id {
        return left.rule_id < right.rule_id;
    }
    left.replacement < right.replacement
}

fn family_weight(rule_id: &str, family: TypingCandidateFamily) -> f64 {
    if rule_id == "moved_prefix_pair" {
        return 98.0;
    }
    if rule_id == "verb_ending" {
        return 92.0;
    }

    match family {
        TypingCandidateFamily::Exact => 120.0,
        TypingCandidateFamily::Visual => 115.0,
        TypingCandidateFamily::Layout => 105.0,
        TypingCandidateFamily::Structural => 78.0,
        TypingCandidateFamily::Typo => 84.0,
        TypingCandidateFamily::Cleanup => 70.0,
        TypingCandidateFamily::Unknown => 40.0,
    }
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

#[cfg(test)]
#[path = "typing_candidate_tests.rs"]
mod tests;
