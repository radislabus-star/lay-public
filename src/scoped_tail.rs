//! Smart scoped-tail correction for manual layout rescue.
//!
//! This facade owns the public scoped-tail contract. Concrete responsibilities
//! live in submodules: scope policy, completed-word decisions and physical
//! word flipping.

use evdev::KeyCode;

use crate::config::CorrectionEngine;
use crate::correction::Correction;
use crate::keyboard::{map_original_events, split_event_words, KeyEvent};

mod completed_word;
mod scope_policy;
mod word_flip;

pub use completed_word::decide_completed_scope_word;
pub use scope_policy::{effective_replace_words, should_force_replay_for_short_fragment};
pub use word_flip::repair_cyrillic_prefix_before_ascii_tail;

use completed_word::short_completed_tail_layout_flip;
use word_flip::flip_word_events;

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
    let words = split_event_words(events)?;
    if words.len() < 2 {
        if let [word] = words.as_slice() {
            let original = map_original_events(events);
            if word_has_mixed_layouts(word) {
                let replacement = flip_word_events(word);
                if replacement != original && !replacement.trim().is_empty() {
                    return Some(replacement);
                }
            }
        }
        return None;
    }

    let original = map_original_events(events);
    let has_trailing_space = events
        .last()
        .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code());
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

fn word_has_mixed_layouts(word: &[KeyEvent]) -> bool {
    let Some(first) = word.first().map(|event| event.layout_is_ru) else {
        return false;
    };
    word.iter().any(|event| event.layout_is_ru != first)
}
