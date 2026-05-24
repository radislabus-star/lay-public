use super::language::language_score;
use super::noise::{noise_cost, removes_extra_repeated_letter};
use super::token::{has_ascii_layout_letter_punctuation, is_known_text, is_known_word, trim_token};
use super::ScoredCandidate;
use crate::text_metrics::{common_replacement_span, normalized_edit_distance, without_whitespace};

pub(super) fn score_candidate(typed: &str, candidate: String) -> ScoredCandidate {
    let language = language_score(&candidate);
    let noise = noise_cost(typed, &candidate);
    let edit = normalized_edit_distance(typed, &candidate);
    let intervention = intervention_penalty(typed, &candidate);
    let total =
        language + structure_bonus(typed, &candidate) + keep_valid_source_bonus(typed, &candidate)
            - (1.0 + noise).ln()
            - edit * 0.25
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
                    4.0
                } else {
                    2.0
                };
            }
        }
    }

    let touched = common_replacement_span(typed, candidate) as f64;
    let mut penalty = 0.08 + touched * 0.006 + protected_penalty;
    let typed_spaces = typed.chars().filter(|ch| ch.is_whitespace()).count();
    let candidate_spaces = candidate.chars().filter(|ch| ch.is_whitespace()).count();
    if candidate_spaces < typed_spaces {
        penalty += (typed_spaces - candidate_spaces) as f64 * 6.0;
    } else if candidate_spaces > typed_spaces {
        penalty += (candidate_spaces - typed_spaces) as f64 * 0.08;
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
        return 1.45;
    }
    if typed_spaces == candidate_spaces && same_letters_with_moved_spaces {
        return 0.30;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return 0.25;
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
        return 0.45;
    }
    0.0
}

fn keep_valid_source_bonus(typed: &str, candidate: &str) -> f64 {
    if typed == candidate && is_known_text(typed) {
        1.25
    } else {
        0.0
    }
}
