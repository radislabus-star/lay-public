//! Smart scoped-tail correction for manual layout rescue.
//!
//! This module owns multi-word scope decisions, LEM candidate generation, and
//! minimal smart-tail choices. It does not own daemon runtime or typing-assist
//! after-space rule order.

use evdev::KeyCode;

use crate::config::CorrectionEngine;
use crate::correction::Correction;
use crate::keyboard::{
    is_cyrillic_letter, map_events_to_layout, map_original_events,
    mixed_visual_latin_word_target_layout, original_event_char, replay_layout_decision,
    split_event_words, KeyEvent,
};
use crate::layout_autoswitch::{
    correct_duplicate_layout_prefix_on_ascii_token, correct_wrong_layout_ascii_technical_token,
    correct_wrong_layout_ascii_word, is_cyrillic_hyphenated_word_for_layout,
    is_known_english_layout_autoswitch_word, is_known_russian_layout_autoswitch_word,
    is_protected_ascii_layout_token, should_keep_plain_cyrillic_before_ascii_technical,
};
use crate::russian_chars::same_letter_ignore_case;
use crate::typing_pipeline::apply_typing_assist;
use crate::typing_replacements::contains_visual_b_word;
use crate::word_buffer::{WordBuffer, MAX_REPLACE_WORDS};
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};
use crate::word_recognizer::is_ascii_technical_token;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopedTailOptions {
    pub lem_enabled: bool,
    pub allow_layout_auto: bool,
}

impl Default for ScopedTailOptions {
    fn default() -> Self {
        Self {
            lem_enabled: false,
            allow_layout_auto: true,
        }
    }
}

fn should_expand_auto_replace_context(buf: &WordBuffer) -> bool {
    let Some((events, _)) = buf.what_to_replay(2) else {
        return false;
    };
    contains_visual_b_word(&map_original_events(&events))
}

pub fn should_force_replay_for_short_fragment(text: &str) -> bool {
    let mut words = text.split_whitespace();
    let Some(word) = words.next() else {
        return false;
    };
    words.next().is_none() && (1..=2).contains(&word.chars().count())
}

pub fn effective_replace_words(
    buf: &WordBuffer,
    replace_words: usize,
    engine: CorrectionEngine,
    auto_replace: bool,
) -> usize {
    let replace_words = replace_words.clamp(1, MAX_REPLACE_WORDS);
    if engine == CorrectionEngine::Replay && auto_replace && should_expand_auto_replace_context(buf)
    {
        return replace_words.max(2);
    }
    replace_words
}

pub fn decide_correction(original: &str, converted: &str, engine: CorrectionEngine) -> Correction {
    if engine == CorrectionEngine::Replay || original == converted {
        return Correction::ReplayAll;
    }
    if original.split_whitespace().count() <= 1 {
        return Correction::ReplayAll;
    }

    match crate::llm::convert_hybrid(original, converted) {
        // Manual double-Shift is an explicit user command. If smart says
        // "original is fine", still allow the user to toggle the selected text.
        Ok(Some(text)) if text == original => Correction::ReplayAll,
        Ok(Some(text)) if text == converted => Correction::ReplayAll,
        Ok(Some(text)) if !text.trim().is_empty() => Correction::InsertText(text),
        Ok(_) => Correction::ReplayAll,
        Err(_) => Correction::ReplayAll,
    }
}

pub fn decide_scoped_tail_correction(events: &[KeyEvent]) -> Option<String> {
    decide_scoped_tail_correction_with_options(events, ScopedTailOptions::default())
}

pub fn decide_scoped_tail_correction_with_lem(
    events: &[KeyEvent],
    enabled: bool,
) -> Option<String> {
    decide_scoped_tail_correction_with_options(
        events,
        ScopedTailOptions {
            lem_enabled: enabled,
            allow_layout_auto: true,
        },
    )
}

pub fn decide_scoped_tail_correction_with_options(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<String> {
    let words = split_event_words(events)?;
    if words.len() < 2 {
        return None;
    }

    let original = map_original_events(events);
    let has_trailing_space = events
        .last()
        .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code());
    if options.lem_enabled {
        let candidates =
            scoped_tail_lem_candidates(&words, !has_trailing_space, options.allow_layout_auto)
                .into_iter()
                .map(|candidate| {
                    if has_trailing_space {
                        format!("{candidate} ")
                    } else {
                        candidate
                    }
                });
        let ranked = crate::lem::rank_candidates(&original, candidates);
        if let Some(best) = ranked.first() {
            let margin = ranked
                .get(1)
                .map(|second| best.total - second.total)
                .unwrap_or(f64::INFINITY);
            let _ = (
                margin,
                best.language,
                best.noise,
                best.edit,
                best.intervention,
            );
            let mut best_text = best.text.clone();
            if has_trailing_space && !best_text.ends_with(' ') {
                best_text.push(' ');
            }
            if best_text != original && !best_text.trim().is_empty() {
                return Some(best_text);
            }
        }
    }

    let mut out = String::with_capacity(original.len());
    for (idx, word) in words.iter().enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        if idx + 1 == words.len() && !has_trailing_space {
            out.push_str(&flip_word_events(word));
        } else if idx + 1 == words.len() {
            out.push_str(
                &short_completed_tail_layout_flip(word)
                    .unwrap_or_else(|| decide_completed_scope_word(word)),
            );
        } else {
            out.push_str(&decide_completed_scope_word(word));
        }
    }
    if has_trailing_space {
        out.push(' ');
    }

    if out != original && !out.trim().is_empty() {
        Some(out)
    } else {
        None
    }
}

pub fn scoped_tail_lem_candidates(
    words: &[&[KeyEvent]],
    last_word_is_current: bool,
    allow_layout_auto: bool,
) -> Vec<String> {
    let mut states: Vec<Vec<String>> = Vec::with_capacity(words.len());
    for (idx, word) in words.iter().enumerate() {
        let is_current_tail = last_word_is_current && idx + 1 == words.len();
        states.push(scoped_word_lem_options(
            word,
            is_current_tail,
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
    allow_layout_auto: bool,
) -> Vec<String> {
    let original = map_original_events(word);
    let mut out = Vec::new();
    if is_current_tail {
        push_unique_string(&mut out, flip_word_events(word));
        return out;
    }

    if let Some(repaired) = confident_completed_scope_repair(&original) {
        push_unique_string(&mut out, repaired);
        return out;
    }

    push_unique_string(&mut out, original.clone());
    if is_short_repeated_completed_scope_word(&original) {
        return out;
    }
    push_unique_string(&mut out, decide_completed_scope_word(word));
    if let Some(repaired) = apply_typing_assist(&format!("{original} "), allow_layout_auto) {
        push_unique_string(&mut out, repaired.trim().to_string());
    }
    let flipped = flip_word_events(word);
    if should_offer_completed_scope_flip(&original, &flipped) {
        push_unique_string(&mut out, flipped);
    }
    out
}

fn confident_completed_scope_repair(original: &str) -> Option<String> {
    crate::llm::repair_mixed_script(original)
        .or_else(|| correct_duplicate_layout_prefix_on_ascii_token(original))
        .or_else(|| correct_wrong_layout_ascii_technical_token(original))
        .or_else(|| correct_wrong_layout_ascii_word(original))
}

fn should_offer_completed_scope_flip(original: &str, flipped: &str) -> bool {
    if stable_completed_scope_original(original) {
        return false;
    }

    let (_, flipped_word, _) = split_word_punctuation(flipped);
    if flipped_word.is_empty() {
        return false;
    }

    let flipped_lower = flipped_word.to_lowercase();
    if is_cyrillic_word(flipped_word) {
        return is_known_russian_layout_autoswitch_word(&flipped_lower);
    }

    if flipped_word.is_ascii() {
        return is_known_english_layout_autoswitch_word(&flipped_word.to_ascii_lowercase())
            || is_ascii_technical_token(flipped);
    }

    false
}

fn short_completed_tail_layout_flip(word: &[KeyEvent]) -> Option<String> {
    let original = map_original_events(word);
    let (_, original_word, _) = split_word_punctuation(&original);
    let original_len = original_word.chars().count();
    if !(2..=4).contains(&original_len)
        || !is_cyrillic_word(original_word)
        || is_known_russian_layout_autoswitch_word(&original_word.to_lowercase())
    {
        return None;
    }

    let flipped = flip_word_events(word);
    let (_, flipped_word, _) = split_word_punctuation(&flipped);
    let flipped_len = flipped_word.chars().count();
    (flipped_word.is_ascii()
        && (2..=4).contains(&flipped_len)
        && flipped_word.chars().all(|ch| ch.is_ascii_alphabetic()))
    .then_some(flipped)
}

fn stable_completed_scope_original(original: &str) -> bool {
    let (_, word, _) = split_word_punctuation(original);
    if word.is_empty() {
        return false;
    }

    let lower = word.to_lowercase();
    if is_cyrillic_word(word) {
        return is_known_russian_layout_autoswitch_word(&lower);
    }

    if word.is_ascii() {
        let ascii_lower = word.to_ascii_lowercase();
        return is_protected_ascii_layout_token(word)
            || is_ascii_technical_token(original)
            || is_known_english_layout_autoswitch_word(&ascii_lower);
    }

    false
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

pub fn decide_completed_scope_word(word: &[KeyEvent]) -> String {
    let original = map_original_events(word);
    if is_single_cyrillic_completed_scope_word(&original) {
        return original;
    }
    if is_short_repeated_completed_scope_word(&original) {
        return original;
    }
    if let Some(repaired) = correct_duplicate_layout_prefix_on_ascii_token(&original) {
        return repaired;
    }
    if let Some(repaired) = correct_wrong_layout_ascii_technical_token(&original) {
        return repaired;
    }
    if let Some(repaired) = correct_wrong_layout_ascii_word(&original) {
        return repaired;
    }
    let converted = flip_word_events(word);
    if should_keep_plain_cyrillic_before_ascii_technical(&original, &converted) {
        return original;
    }
    let decision = if crate::llm::model_backend_enabled() {
        crate::llm::choose_token_consensus(&original, &converted)
    } else {
        crate::llm::choose_token_hybrid(&original, &converted)
    };
    match decision {
        Ok(Some(text)) if !text.trim().is_empty() => text,
        Ok(_) | Err(_) => original,
    }
}

fn is_single_cyrillic_completed_scope_word(word: &str) -> bool {
    let (_, core, _) = split_word_punctuation(word);
    let mut chars = core.chars();
    matches!(
        (chars.next(), chars.next()),
        (Some(ch), None) if is_cyrillic_letter(ch)
    )
}

fn is_short_repeated_completed_scope_word(original: &str) -> bool {
    let (_, word, _) = split_word_punctuation(original);
    let mut chars = word.chars();
    matches!(
        (chars.next(), chars.next(), chars.next()),
        (Some(first), Some(second), None)
            if first == second && (is_cyrillic_letter(first) || first.is_ascii_alphabetic())
    )
}

fn flip_word_events(word: &[KeyEvent]) -> String {
    if let Some(repaired) = repair_cyrillic_prefix_before_ascii_tail(word) {
        return repaired;
    }
    let original = map_original_events(word);
    if let Some(repaired) = correct_duplicate_layout_prefix_on_ascii_token(&original) {
        return repaired;
    }
    if let Some(target_is_ru) = mixed_visual_latin_word_target_layout(word) {
        return map_events_to_layout(word, target_is_ru);
    }
    if let Some(normalized) = normalize_mixed_word_to_last_layout(word) {
        return normalized;
    }
    let decision = replay_layout_decision(word);
    map_events_to_layout(word, decision.target_is_ru)
}

pub fn repair_cyrillic_prefix_before_ascii_tail(word: &[KeyEvent]) -> Option<String> {
    let first_event = word.first()?;
    let first = original_event_char(first_event)?;
    if !is_cyrillic_letter(first) || word.len() < 3 {
        return None;
    }

    let rest = &word[1..];
    let rest_original: String = rest.iter().filter_map(original_event_char).collect();
    if rest_original.chars().count() != rest.len()
        || !rest_original.is_ascii()
        || !rest_original.chars().any(|ch| ch.is_ascii_alphabetic())
        || !rest_original
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return None;
    }

    let all_ru = map_events_to_layout(word, true);
    if all_ru != map_original_events(word) && is_cyrillic_hyphenated_word_for_layout(&all_ru) {
        return Some(all_ru);
    }

    let mut chars = all_ru.chars();
    let first_ru = chars.next()?;
    let second_ru = chars.next()?;
    if !same_letter_ignore_case(first_ru, second_ru) {
        return None;
    }

    let mut candidate = String::new();
    candidate.push(first_ru);
    candidate.extend(chars);
    if candidate == all_ru || candidate == map_original_events(word) {
        return None;
    }
    is_cyrillic_hyphenated_word_for_layout(&candidate).then_some(candidate)
}

fn normalize_mixed_word_to_last_layout(word: &[KeyEvent]) -> Option<String> {
    let target_is_ru = word.last()?.layout_is_ru;
    if word.iter().all(|event| event.layout_is_ru == target_is_ru) {
        return None;
    }

    let mut out = String::new();
    let mut run_start = 0;
    let mut current_layout = word.first()?.layout_is_ru;
    for (idx, event) in word.iter().enumerate() {
        if event.layout_is_ru != current_layout {
            let run = map_events_to_layout(&word[run_start..idx], target_is_ru);
            push_with_overlap(&mut out, &run);
            run_start = idx;
            current_layout = event.layout_is_ru;
        }
    }
    let run = map_events_to_layout(&word[run_start..], target_is_ru);
    push_with_overlap(&mut out, &run);

    (!out.is_empty()).then_some(out)
}

fn push_with_overlap(out: &mut String, next: &str) {
    if out.is_empty() || next.is_empty() {
        out.push_str(next);
        return;
    }

    let out_chars: Vec<char> = out.chars().collect();
    let next_chars: Vec<char> = next.chars().collect();
    let max_overlap = out_chars.len().min(next_chars.len());
    let overlap = (1..=max_overlap)
        .rev()
        .find(|len| {
            out_chars[out_chars.len() - len..]
                .iter()
                .zip(&next_chars[..*len])
                .all(|(left, right)| left == right)
        })
        .unwrap_or(0);
    out.push_str(&next_chars[overlap..].iter().collect::<String>());
}
