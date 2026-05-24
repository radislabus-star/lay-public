use super::cases::Case;
use lay::dict::{self, Direction};
use lay::lem::{self, ScoredCandidate};
use lay::word_recognizer::is_ascii_technical_or_brand_token;

pub(crate) fn rank_candidates(case: &Case) -> Vec<ScoredCandidate> {
    lem::rank_candidates(&case.typed, candidate_texts(case))
}

fn candidate_texts(case: &Case) -> Vec<String> {
    let mut out = Vec::new();
    push_candidate(&mut out, &case.typed);
    push_candidate(&mut out, &case.expected);
    push_candidate(&mut out, &dict::convert(&case.typed, Direction::Us2Ru));
    push_candidate(&mut out, &dict::convert(&case.typed, Direction::Ru2Us));
    push_candidate(&mut out, &flip_tokens(&case.typed));
    push_candidate(&mut out, &case.typed.replace(' ', ""));
    push_candidate(&mut out, &case.expected.replace(' ', ""));
    push_candidate(&mut out, &case.expected.replace("  ", " "));
    for candidate in moved_space_candidates(&case.typed) {
        push_candidate(&mut out, &candidate);
    }
    for candidate in repeated_letter_candidates(&case.typed) {
        push_candidate(&mut out, &candidate);
    }
    out
}

fn push_candidate(out: &mut Vec<String>, text: &str) {
    let text = text.trim().to_string();
    if !text.is_empty() && !out.iter().any(|item| item == &text) {
        out.push(text);
    }
}

fn flip_tokens(text: &str) -> String {
    text.split_whitespace()
        .map(|token| dict::convert(token, dict::detect_direction(token)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn moved_space_candidates(text: &str) -> Vec<String> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::new();
    for idx in 0..tokens.len().saturating_sub(1) {
        let left = tokens[idx];
        let right = tokens[idx + 1];
        if left.chars().count() < 2 || right.chars().count() < 2 {
            continue;
        }
        if is_ascii_technical_or_brand_token(left) {
            continue;
        }

        let mut right_chars = right.chars();
        let Some(moved) = right_chars.next() else {
            continue;
        };
        let repaired_left = format!("{left}{moved}");
        let repaired_right = right_chars.collect::<String>();
        if repaired_right.is_empty() {
            continue;
        }

        let mut candidate = tokens.clone();
        candidate[idx] = &repaired_left;
        candidate[idx + 1] = &repaired_right;
        out.push(candidate.join(" "));
    }
    out
}

fn repeated_letter_candidates(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    for idx in 1..chars.len() {
        if chars[idx] != chars[idx - 1] {
            continue;
        }
        let mut candidate = String::with_capacity(text.len());
        for (pos, ch) in chars.iter().enumerate() {
            if pos != idx {
                candidate.push(*ch);
            }
        }
        out.push(candidate);
    }
    out
}
