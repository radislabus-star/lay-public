//! Exact typing replacements and promoted user corrections.
//!
//! This module owns user-configured exact rules, promoted learning rules, and
//! visual `b` replacement in Russian context. It intentionally does not decide
//! layout scope or smart-tail strategy.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::lexicon::{visual_b_after_ascii_replacement, visual_b_default_replacement};
use crate::text_case::apply_phrase_case;
use crate::word_reader::{split_edge_whitespace, split_word_punctuation, split_ws_segments};

pub const REPLACEMENTS_PATH: &str = ".config/lay/replacements.json";

pub(crate) fn warm_up() {
    let _ = replacement_rules().len();
    let _ = promoted_replacement_rules()
        .lock()
        .map(|rules| rules.len())
        .unwrap_or_default();
}

pub fn apply_auto_replace(original: &str, target: &str) -> Option<String> {
    let (target_leading, target_core, target_trailing) = split_edge_whitespace(target);
    if target_core.is_empty() {
        return None;
    }

    if let Some(visual) = replace_visual_b_in_context(original, target) {
        return Some(visual);
    }

    replacement_for_token(target_core).map(|replacement| {
        let mut out = String::with_capacity(target.len().max(replacement.len()));
        out.push_str(target_leading);
        out.push_str(&replacement);
        out.push_str(target_trailing);
        out
    })
}

pub(crate) fn replacement_for_token(token: &str) -> Option<String> {
    if let Some(replacement) = promoted_replacement_for_token(token) {
        return Some(replacement);
    }

    if let Some(replacement) = replacement_rules().get(token) {
        return Some(replacement.clone());
    }

    let lower = token.to_lowercase();
    if lower == token {
        return None;
    }
    replacement_rules()
        .get(&lower)
        .map(|replacement| apply_phrase_case(token, replacement))
}

pub fn promoted_replacement_for_token(token: &str) -> Option<String> {
    let rules = promoted_replacement_rules().lock().ok()?;
    if let Some(replacement) = rules.get(token) {
        return Some(replacement.clone());
    }

    let lower = token.to_lowercase();
    if lower == token {
        return None;
    }
    rules
        .get(&lower)
        .map(|replacement| apply_phrase_case(token, replacement))
}

fn promoted_replacement_rules() -> &'static Mutex<HashMap<String, String>> {
    static RULES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    RULES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn remember_promoted_replacement(from: &str, to: &str) {
    if let Ok(mut rules) = promoted_replacement_rules().lock() {
        rules.insert(from.to_string(), to.to_string());
    }
}

fn replace_visual_b_in_context(original: &str, target: &str) -> Option<String> {
    if !contains_visual_b_word(original) {
        return None;
    }

    let base = if has_cyrillic_text(original) {
        original
    } else {
        target
    };
    replace_visual_b_words(original, base)
}

pub(crate) fn replace_visual_b_words(original: &str, base: &str) -> Option<String> {
    let original_segments = split_ws_segments(original);
    let base_segments = split_ws_segments(base);
    if original_segments.len() != base_segments.len() {
        return None;
    }

    let mut changed = false;
    let mut out = String::with_capacity(base.len());
    for (idx, ((orig, orig_ws), (base_part, base_ws))) in original_segments
        .iter()
        .zip(base_segments.iter())
        .enumerate()
    {
        if orig_ws != base_ws {
            return None;
        }
        if *orig_ws {
            out.push_str(base_part);
            continue;
        }

        let replacement = match *orig {
            "b" => Some(visual_b_replacement(
                &original_segments,
                &base_segments,
                idx,
                false,
            )),
            "B" => Some(visual_b_replacement(
                &original_segments,
                &base_segments,
                idx,
                true,
            )),
            _ => None,
        };
        if let Some(replacement) = replacement {
            changed = true;
            out.push_str(&replacement);
        } else {
            out.push_str(base_part);
        }
    }

    if changed {
        Some(out)
    } else {
        None
    }
}

fn visual_b_replacement(
    original_segments: &[(&str, bool)],
    base_segments: &[(&str, bool)],
    idx: usize,
    uppercase: bool,
) -> String {
    let prev = previous_word_segment(original_segments, idx);
    let base = base_segments.get(idx).map(|(text, _)| *text).unwrap_or("");

    let wants_layout_i = prev.is_some_and(is_ascii_word_token)
        || (base.eq_ignore_ascii_case(visual_b_after_ascii_replacement())
            && prev.is_some_and(is_ascii_word_token));
    let replacement = if wants_layout_i {
        visual_b_after_ascii_replacement()
    } else {
        visual_b_default_replacement()
    };
    if uppercase {
        replacement.to_uppercase()
    } else {
        replacement.to_string()
    }
}

fn previous_word_segment<'a>(segments: &'a [(&str, bool)], idx: usize) -> Option<&'a str> {
    segments[..idx]
        .iter()
        .rev()
        .find_map(|(text, is_ws)| (!*is_ws).then_some(*text))
}

fn is_ascii_word_token(token: &str) -> bool {
    let (_, core, _) = split_word_punctuation(token);
    !core.is_empty()
        && core.chars().any(|ch| ch.is_ascii_alphabetic())
        && core
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || matches!(ch, '-' | '_' | '.'))
}

pub fn contains_visual_b_word(text: &str) -> bool {
    text.split_whitespace()
        .any(|word| word == "b" || word == "B")
}

fn has_cyrillic_text(text: &str) -> bool {
    text.chars().any(|ch| matches!(ch, 'А'..='я' | 'ё' | 'Ё'))
}

fn replacement_rules() -> &'static HashMap<String, String> {
    static RULES: OnceLock<HashMap<String, String>> = OnceLock::new();
    RULES.get_or_init(|| {
        let mut rules = HashMap::new();
        #[cfg(test)]
        rules.extend(crate::typing_assist_test_fixtures::replacement_rules());

        if let Some(home) = std::env::var_os("HOME") {
            let path = std::path::PathBuf::from(home).join(REPLACEMENTS_PATH);
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(custom) = serde_json::from_str::<HashMap<String, String>>(&text) {
                    rules.extend(custom);
                }
            }
        }

        rules
    })
}
