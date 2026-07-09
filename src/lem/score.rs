use super::language::language_score;
use super::noise::{noise_cost, removes_extra_repeated_letter};
use super::token::{has_ascii_layout_letter_punctuation, is_known_text, is_known_word, trim_token};
use super::ScoredCandidate;
use crate::text_metrics::{common_replacement_span, normalized_edit_distance, without_whitespace};

const NOISE_LOG_BASE: f64 = 1.0;
const EDIT_DISTANCE_WEIGHT: f64 = 0.25;

const KNOWN_TO_KNOWN_TOKEN_PENALTY: f64 = 4.0;
const KNOWN_TO_UNKNOWN_TOKEN_PENALTY: f64 = 2.0;
const BASE_INTERVENTION_PENALTY: f64 = 0.08;
const TOUCHED_CHAR_PENALTY: f64 = 0.006;
const REMOVED_SPACE_PENALTY: f64 = 6.0;
const ADDED_SPACE_PENALTY: f64 = 0.08;

const PURE_SPLIT_BONUS: f64 = 1.45;
const MOVED_SPACE_BONUS: f64 = 0.30;
const EXTRA_REPEAT_REPAIR_BONUS: f64 = 0.25;
const MORE_KNOWN_TOKENS_BONUS: f64 = 0.45;
const KEEP_VALID_SOURCE_BONUS: f64 = 1.25;

pub(super) fn score_candidate(typed: &str, candidate: String) -> ScoredCandidate {
    let language = language_score(&candidate);
    let noise = noise_cost(typed, &candidate);
    let edit = normalized_edit_distance(typed, &candidate);
    let intervention = intervention_penalty(typed, &candidate);
    let total =
        language + structure_bonus(typed, &candidate) + keep_valid_source_bonus(typed, &candidate)
            - (NOISE_LOG_BASE + noise).ln()
            - edit * EDIT_DISTANCE_WEIGHT
            - intervention;
    ScoredCandidate {
        text: candidate,
        total,
        language,
        noise,
        edit,
        intervention,
    }
}

fn intervention_penalty(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    let mut protected_penalty = 0.0;
    let typed_tokens: Vec<&str> = typed.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    let same_letters_with_moved_spaces = without_whitespace(typed) == without_whitespace(candidate);
    if typed_tokens.len() == candidate_tokens.len() && !same_letters_with_moved_spaces {
        for (typed_token, candidate_token) in typed_tokens.iter().zip(candidate_tokens.iter()) {
            let typed_raw = *typed_token;
            let typed_token = trim_token(typed_token);
            let candidate_token = trim_token(candidate_token);
            if typed_token != candidate_token
                && is_known_word(typed_token)
                && !has_ascii_layout_letter_punctuation(typed_raw)
            {
                protected_penalty += if is_known_word(candidate_token) {
                    KNOWN_TO_KNOWN_TOKEN_PENALTY
                } else {
                    KNOWN_TO_UNKNOWN_TOKEN_PENALTY
                };
            }
        }
    }

    let touched = common_replacement_span(typed, candidate) as f64;
    let mut penalty =
        BASE_INTERVENTION_PENALTY + touched * TOUCHED_CHAR_PENALTY + protected_penalty;
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    if candidate_spaces < typed_spaces {
        penalty += (typed_spaces - candidate_spaces) as f64 * REMOVED_SPACE_PENALTY;
    } else if candidate_spaces > typed_spaces {
        penalty += (candidate_spaces - typed_spaces) as f64 * ADDED_SPACE_PENALTY;
    }
    penalty
}

fn structure_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    let same_letters_with_moved_spaces = without_whitespace(typed) == without_whitespace(candidate);
    if typed_spaces == 0 && candidate_spaces > 0 && same_letters_with_moved_spaces {
        return PURE_SPLIT_BONUS;
    }
    if typed_spaces == candidate_spaces && same_letters_with_moved_spaces {
        return MOVED_SPACE_BONUS;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return EXTRA_REPEAT_REPAIR_BONUS;
    }

    let typed_known = typed
        .split_whitespace()
        .filter(|token| is_known_word(trim_token(token)))
        .count();
    let candidate_known = candidate
        .split_whitespace()
        .filter(|token| is_known_word(trim_token(token)))
        .count();
    if typed_spaces == candidate_spaces && candidate_known > typed_known {
        return MORE_KNOWN_TOKENS_BONUS * (candidate_known - typed_known) as f64;
    }
    0.0
}

fn keep_valid_source_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate && is_known_text(typed) {
        KEEP_VALID_SOURCE_BONUS
    } else {
        0.0
    }
}
