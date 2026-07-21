use crate::text_metrics::{damerau_levenshtein, has_cyrillic, without_whitespace};
use crate::typing_rule_graph::ids;
use crate::word_reader::last_text_word;

use super::types::{TypingCandidate, TypingCandidateFamily, TypingCandidateScore};

const LANGUAGE_MARGIN_MIN: f64 = -4.0;
const LANGUAGE_MARGIN_MAX: f64 = 8.0;
const LANGUAGE_MARGIN_WEIGHT: f64 = 0.35;
const PURE_SPACE_REPAIR_BONUS: f64 = 4.0;
const MAX_EDIT_PENALTY: f64 = 4.0;
const CHANGED_TEXT_PENALTY: f64 = 0.25;

impl TypingCandidate {
    pub fn new(rule_id: &str, priority: i32, original: &str, replacement: String) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            priority,
            score: score_typing_candidate(original, &replacement, rule_id, priority),
            replacement,
        }
    }

    pub(crate) fn is_safe_for(&self, original: &str) -> bool {
        crate::typing_rule_graph::typing_rule_candidate_is_safe(
            &self.rule_id,
            original,
            &self.replacement,
        )
    }
}

pub fn classify_typing_rule(rule_id: &str) -> TypingCandidateFamily {
    if matches!(rule_id, ids::PERSONAL_PHRASE | ids::PERSONAL_TOKEN) {
        return TypingCandidateFamily::Exact;
    }
    crate::typing_rule_graph::typing_rule_family(rule_id).unwrap_or(TypingCandidateFamily::Unknown)
}

pub fn score_typing_candidate(
    original: &str,
    replacement: &str,
    rule_id: &str,
    priority: i32,
) -> TypingCandidateScore {
    let text = CandidateTextPair::new(original, replacement);
    let family = classify_typing_rule(rule_id);
    let family_weight = family_weight(rule_id, family);
    let language_delta = language_delta(&text);
    let structure_bonus = structure_bonus(&text);
    let lexical_prior_bonus = lexical_prior_bonus(&text);
    let weak_grammar_penalty = weak_grammar_penalty(rule_id, &text);
    let edit_penalty = edit_penalty(&text);
    let intervention_penalty = intervention_penalty(&text);
    let priority_bonus = priority_bonus(priority);
    let total =
        family_weight + language_delta + structure_bonus + lexical_prior_bonus + priority_bonus
            - edit_penalty
            - intervention_penalty;
    let total = total - weak_grammar_penalty;

    TypingCandidateScore {
        total,
        family,
        family_weight,
        language_delta,
        structure_bonus,
        lexical_prior_bonus,
        weak_grammar_penalty,
        edit_penalty,
        intervention_penalty,
        priority_bonus,
    }
}

fn family_weight(rule_id: &str, family: TypingCandidateFamily) -> f64 {
    crate::typing_rule_graph::typing_rule_family_weight(rule_id, family)
}

struct CandidateTextPair<'a> {
    original: &'a str,
    replacement: &'a str,
}

impl<'a> CandidateTextPair<'a> {
    fn new(original: &'a str, replacement: &'a str) -> Self {
        Self {
            original,
            replacement,
        }
    }

    fn changed(&self) -> bool {
        self.original != self.replacement
    }

    fn has_cyrillic(&self) -> bool {
        has_cyrillic(self.original) || has_cyrillic(self.replacement)
    }

    fn original_char_len(&self) -> usize {
        self.original.chars().count().max(1)
    }

    fn replacement_char_len(&self) -> usize {
        self.replacement.chars().count().max(1)
    }

    fn original_internal_spaces(&self) -> usize {
        internal_space_count(self.original)
    }

    fn replacement_internal_spaces(&self) -> usize {
        internal_space_count(self.replacement)
    }

    fn preserves_compact_text(&self) -> bool {
        without_whitespace(self.original) == without_whitespace(self.replacement)
    }
}

fn language_delta(text: &CandidateTextPair<'_>) -> f64 {
    if !text.has_cyrillic() {
        return 0.0;
    }
    let margin = crate::ngram::ru_candidate_margin(text.replacement, text.original);
    if !margin.is_finite() {
        return 0.0;
    }

    margin.clamp(LANGUAGE_MARGIN_MIN, LANGUAGE_MARGIN_MAX) * LANGUAGE_MARGIN_WEIGHT
}

fn structure_bonus(text: &CandidateTextPair<'_>) -> f64 {
    if text.replacement_internal_spaces() > text.original_internal_spaces()
        && text.preserves_compact_text()
    {
        return PURE_SPACE_REPAIR_BONUS;
    }
    if text.replacement_internal_spaces() > text.original_internal_spaces() {
        return 0.0;
    }
    0.0
}

fn lexical_prior_bonus(text: &CandidateTextPair<'_>) -> f64 {
    let Some(word) = last_text_word(text.replacement) else {
        return 0.0;
    };
    let lower = word.to_lowercase();
    let mut bonus = 0.0;
    if crate::lexicon::is_common_ru_word(&lower) {
        bonus += 8.0;
    } else if crate::typing_transition::state::word_has_common_usage_authority(&lower) {
        bonus += 4.0;
    }
    if let Some(rank) = crate::nanda_wave::l2::l2_surface_foundation_rank(&lower) {
        bonus += match rank {
            0..=999 => 6.0,
            1000..=9_999 => 3.0,
            10_000..=49_999 => 1.0,
            _ => 0.0,
        };
    }
    bonus
}

fn weak_grammar_penalty(rule_id: &str, text: &CandidateTextPair<'_>) -> f64 {
    if !matches!(rule_id, ids::VERB_ENDING | ids::VOWEL_CONFUSION) {
        return 0.0;
    }
    let Some(word) = last_text_word(text.replacement) else {
        return 0.0;
    };
    let lower = word.to_lowercase();
    let has_authority = crate::lexicon::is_common_ru_word(&lower)
        || crate::typing_transition::state::word_has_common_usage_authority(&lower)
        || crate::nanda_wave::l2::l2_surface_foundation_rank(&lower)
            .is_some_and(|rank| rank < 50_000);
    if has_authority {
        0.0
    } else {
        12.0
    }
}

fn edit_penalty(text: &CandidateTextPair<'_>) -> f64 {
    let original_chars = text.original_char_len();
    let replacement_chars = text.replacement_char_len();
    let distance = damerau_levenshtein(text.original, text.replacement) as f64;
    let scale = original_chars.max(replacement_chars) as f64;
    (distance / scale).min(1.0) * MAX_EDIT_PENALTY
}

fn intervention_penalty(text: &CandidateTextPair<'_>) -> f64 {
    if text.changed() {
        CHANGED_TEXT_PENALTY
    } else {
        0.0
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
