use crate::keyboard::{map_original_events, KeyEvent};
use crate::typing_pipeline::select_typing_assist;
use crate::word_reader::split_word_punctuation;

use super::completed_word::{
    completed_scope_flip_target_is_known, decide_completed_scope_word,
    deterministic_completed_scope_repair, is_short_repeated_completed_scope_word,
    stable_completed_scope_original,
};
use super::word_flip::flip_word_events;

pub fn scoped_tail_lem_candidates(
    words: &[&[KeyEvent]],
    last_word_is_current: bool,
    allow_layout_auto: bool,
) -> Vec<String> {
    let mut states: Vec<Vec<String>> = Vec::with_capacity(words.len());
    for (idx, word) in words.iter().enumerate() {
        let is_current_tail = last_word_is_current && idx + 1 == words.len();
        let is_last_completed_tail = !last_word_is_current && idx + 1 == words.len();
        states.push(scoped_word_lem_options(
            word,
            is_current_tail,
            is_last_completed_tail,
            allow_layout_auto,
        ));
    }

    let mut out = Vec::new();
    build_phrase_candidates(&states, 0, &mut Vec::new(), &mut out);
    out
}

fn scoped_word_lem_options(
    word: &[KeyEvent],
    is_current_tail: bool,
    is_last_completed_tail: bool,
    allow_layout_auto: bool,
) -> Vec<String> {
    let original = map_original_events(word);
    let mut out = Vec::new();
    if is_current_tail {
        push_unique_string(&mut out, original);
        push_unique_string(&mut out, flip_word_events(word));
        return out;
    }

    if let Some(repaired) = confident_completed_scope_repair(&original) {
        push_unique_string(&mut out, repaired);
        return out;
    }

    push_unique_string(&mut out, original.clone());
    if stable_completed_scope_original(&original)
        || is_short_repeated_completed_scope_word(&original)
    {
        return out;
    }
    push_unique_string(&mut out, decide_completed_scope_word(word));
    if let Some(repaired) = select_typing_assist(&format!("{original} "), allow_layout_auto) {
        push_unique_string(&mut out, repaired.trim().to_string());
    }
    let flipped = flip_word_events(word);
    if should_offer_completed_scope_flip(&original, &flipped)
        || should_offer_explicit_manual_tail_flip(is_last_completed_tail, &original, &flipped)
    {
        push_unique_string(&mut out, flipped);
    }
    out
}

fn should_offer_explicit_manual_tail_flip(
    is_last_completed_tail: bool,
    original: &str,
    flipped: &str,
) -> bool {
    if !is_last_completed_tail || stable_completed_scope_original(original) {
        return false;
    }

    let (_, flipped_word, _) = split_word_punctuation(flipped);
    !flipped_word.is_empty()
        && flipped_word.is_ascii()
        && flipped_word.chars().all(|ch| ch.is_ascii_alphabetic())
}

fn confident_completed_scope_repair(original: &str) -> Option<String> {
    crate::llm::repair_mixed_script(original)
        .or_else(|| deterministic_completed_scope_repair(original))
}

fn should_offer_completed_scope_flip(original: &str, flipped: &str) -> bool {
    if stable_completed_scope_original(original) {
        return false;
    }

    let (_, flipped_word, _) = split_word_punctuation(flipped);
    if flipped_word.is_empty() {
        return false;
    }

    completed_scope_flip_target_is_known(flipped, flipped_word)
}

fn build_phrase_candidates(
    states: &[Vec<String>],
    idx: usize,
    current: &mut Vec<String>,
    out: &mut Vec<String>,
) {
    if idx == states.len() {
        push_unique_string(out, current.join(" "));
        return;
    }
    for option in &states[idx] {
        current.push(option.clone());
        build_phrase_candidates(states, idx + 1, current, out);
        current.pop();
    }
}

fn push_unique_string(out: &mut Vec<String>, value: String) {
    if !value.trim().is_empty() && !out.iter().any(|item| item == &value) {
        out.push(value);
    }
}
