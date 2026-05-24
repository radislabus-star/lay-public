use crate::dict::{self, Direction};
use crate::text_metrics::damerau_levenshtein;

pub(super) fn noise_cost(typed: &str, candidate: &str) -> f64 {
    if typed == candidate {
        return 0.0;
    }
    if dict::convert(typed, Direction::Us2Ru) == candidate
        || dict::convert(typed, Direction::Ru2Us) == candidate
    {
        return 0.15;
    }
    if flip_tokens(typed) == candidate {
        return 0.25;
    }
    if let Some(cost) = partial_token_flip_cost(typed, candidate) {
        return cost;
    }
    if typed.replace(' ', "") == candidate.replace(' ', "") {
        return 0.35;
    }
    if removes_extra_repeated_letter(typed, candidate) {
        return 0.08;
    }

    let distance = damerau_levenshtein(typed, candidate) as f64;
    let scale = typed.chars().count().max(candidate.chars().count()).max(1) as f64;
    if distance <= 2.0 {
        return 0.10 + distance / scale * 0.25;
    }
    0.75 + distance / scale
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

    (changed > 0).then_some(0.18 + changed as f64 * 0.10)
}

fn flip_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| dict::convert(token, dict::detect_direction(token)))
        .collect::<Vec<_>>()
        .join(" ")
}
