//! Smart scoped-tail correction for manual layout rescue.
//!
//! This facade owns the public scoped-tail contract. Concrete responsibilities
//! live in submodules: scope policy, LEM candidate generation, completed-word
//! decisions and physical word flipping.

use evdev::KeyCode;

use crate::config::CorrectionEngine;
use crate::correction::Correction;
use crate::keyboard::{map_original_events, split_event_words, KeyEvent};
use crate::lem::ScoredCandidate;

mod completed_word;
mod lem_candidates;
mod scope_policy;
mod word_flip;

pub use completed_word::decide_completed_scope_word;
pub use lem_candidates::scoped_tail_lem_candidates;
pub use scope_policy::{effective_replace_words, should_force_replay_for_short_fragment};
pub use word_flip::repair_cyrillic_prefix_before_ascii_tail;

use completed_word::short_completed_tail_layout_flip;
use word_flip::flip_word_events;

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
    if options.lem_enabled {
        if let Some(best_text) =
            best_lem_scoped_tail(&words, &original, has_trailing_space, options)
        {
            return Some(best_text);
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

fn word_has_mixed_layouts(word: &[KeyEvent]) -> bool {
    let Some(first) = word.first().map(|event| event.layout_is_ru) else {
        return false;
    };
    word.iter().any(|event| event.layout_is_ru != first)
}

pub(crate) fn rank_scoped_tail_lem_candidates(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<(String, Vec<ScoredCandidate>)> {
    if !options.lem_enabled {
        return None;
    }

    let words = split_event_words(events)?;
    if words.len() < 2 {
        return None;
    }

    let original = map_original_events(events);
    let has_trailing_space = events
        .last()
        .is_some_and(|event| event.keycode == KeyCode::KEY_SPACE.code());
    let ranked = rank_lem_scoped_tail_words(&words, &original, has_trailing_space, options);
    Some((original, ranked))
}

fn best_lem_scoped_tail(
    words: &[&[KeyEvent]],
    original: &str,
    has_trailing_space: bool,
    options: ScopedTailOptions,
) -> Option<String> {
    let ranked = rank_lem_scoped_tail_words(words, original, has_trailing_space, options);
    let best = ranked.first()?;
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
    (best_text != original && !best_text.trim().is_empty()).then_some(best_text)
}

fn rank_lem_scoped_tail_words(
    words: &[&[KeyEvent]],
    original: &str,
    has_trailing_space: bool,
    options: ScopedTailOptions,
) -> Vec<ScoredCandidate> {
    let candidates =
        scoped_tail_lem_candidates(words, !has_trailing_space, options.allow_layout_auto)
            .into_iter()
            .map(|candidate| {
                if has_trailing_space {
                    format!("{candidate} ")
                } else {
                    candidate
                }
            });
    crate::lem::rank_candidates(original, candidates)
}
