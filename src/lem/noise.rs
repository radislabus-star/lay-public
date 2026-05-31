use crate::dict::{self, Direction};
use crate::text_metrics::damerau_levenshtein;

const WHOLE_LAYOUT_FLIP_COST: f64 = 0.15;
const TOKEN_LAYOUT_FLIP_COST: f64 = 0.25;
const MOVED_SPACES_COST: f64 = 0.35;
const EXTRA_REPEAT_REPAIR_COST: f64 = 0.08;
const SMALL_EDIT_DISTANCE_LIMIT: f64 = 2.0;
const SMALL_EDIT_BASE_COST: f64 = 0.10;
const SMALL_EDIT_DISTANCE_WEIGHT: f64 = 0.25;
const LARGE_EDIT_BASE_COST: f64 = 0.75;
const PARTIAL_TOKEN_FLIP_BASE_COST: f64 = 0.18;
const PARTIAL_TOKEN_FLIP_PER_TOKEN_COST: f64 = 0.10;

pub(super) fn noise_cost(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    if dict::convert(typed, Direction::Us2Ru) == candidate
        || dict::convert(typed, Direction::Ru2Us) == candidate
    {
        return WHOLE_LAYOUT_FLIP_COST;
    }
    if flip_tokens(typed) == candidate {
        return TOKEN_LAYOUT_FLIP_COST;
    }
    if let Some(cost) = partial_token_flip_cost(typed, candidate) {
        return cost;
    }
    if typed.replace(' ', "") == candidate.replace(' ', "") {
        return MOVED_SPACES_COST;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return EXTRA_REPEAT_REPAIR_COST;
    }

    let distance = damerau_levenshtein(typed, candidate) as f64;
    let scale = typed.chars().count().max(candidate.chars().count()).max(1) as f64;
    if distance <= SMALL_EDIT_DISTANCE_LIMIT {
        return SMALL_EDIT_BASE_COST + distance / scale * SMALL_EDIT_DISTANCE_WEIGHT;
    }
    LARGE_EDIT_BASE_COST + distance / scale
}

pub(super) fn removes_extra_repeated_letter(typed: &str, candidate: &str) -> bool {
    if typed == candidate || typed.chars().count() <= candidate.chars().count() {
        return false;
    }

    let typed_chars: Vec<char> = typed.chars().collect();
    for idx in 1..typed_chars.len() {
        if typed_chars[idx] != typed_chars[idx - 1] {
            continue;
        }
        let mut repaired = String::with_capacity(typed.len());
        for (pos, ch) in typed_chars.iter().enumerate() {
            if pos != idx {
                repaired.push(*ch);
            }
        }
        if repaired == candidate {
            return true;
        }
    }
    false
}

fn partial_token_flip_cost(typed: &str, candidate: &str) -> Option<f64> {
    let typed_tokens: Vec<&str> = typed.split_whitespace().collect();
    let candidate_tokens: Vec<&str> = candidate.split_whitespace().collect();
    if typed_tokens.len() != candidate_tokens.len() || typed_tokens.is_empty() {
        return None;
    }

    let mut changed = 0usize;
    for (typed_token, candidate_token) in typed_tokens.iter().zip(candidate_tokens.iter()) {
        if typed_token == candidate_token {
            continue;
        }
        if dict::convert(typed_token, dict::detect_direction(typed_token)) == *candidate_token {
            changed += 1;
            continue;
        }
        return None;
    }

    (changed > 0).then_some(
        PARTIAL_TOKEN_FLIP_BASE_COST + changed as f64 * PARTIAL_TOKEN_FLIP_PER_TOKEN_COST,
    )
}

fn flip_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| dict::convert(token, dict::detect_direction(token)))
        .collect::<Vec<_>>()
        .join(" ")
}
