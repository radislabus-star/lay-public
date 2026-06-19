//! Exact typing replacements and promoted user corrections.
//!
//! This module owns user-configured exact rules, promoted learning rules, and
//! visual `b` replacement in Russian context. It intentionally does not decide
//! layout scope or smart-tail strategy.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::layout_autoswitch::is_known_english_layout_autoswitch_word;
use crate::lexicon::{
    is_common_en_technical_word, visual_b_after_ascii_replacement, visual_b_default_replacement,
};
use crate::russian_lexicon::is_known_russian_word_or_form;
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
    apply_auto_replace_with_visual_b(original, target, true)
}

pub fn apply_manual_replay_auto_replace(original: &str, target: &str) -> Option<String> {
    apply_auto_replace_with_visual_b(original, target, false)
}

fn apply_auto_replace_with_visual_b(
    original: &str,
    target: &str,
    allow_visual_b: bool,
) -> Option<String> {
    let (target_leading, target_core, target_trailing) = split_edge_whitespace(target);
    if target_core.is_empty() {
        return None;
    }

    if allow_visual_b {
        if let Some(visual) = replace_visual_b_in_context(original, target) {
            return Some(visual);
        }
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
        return safe_promoted_replacement(token, replacement).then(|| replacement.clone());
    }

    let lower = token.to_lowercase();
    if lower == token {
        return None;
    }
    rules.get(&lower).and_then(|replacement| {
        safe_promoted_replacement(&lower, replacement)
            .then(|| apply_phrase_case(token, replacement))
    })
}

fn promoted_replacement_rules() -> &'static Mutex<HashMap<String, String>> {
    static RULES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    RULES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn remember_promoted_replacement(from: &str, to: &str) {
    if !safe_promoted_replacement(from, to) {
        return;
    }
    if let Ok(mut rules) = promoted_replacement_rules().lock() {
        rules.insert(from.to_string(), to.to_string());
    }
}

pub fn sanitize_replacement_rules_path(path: &Path) -> std::io::Result<usize> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(0);
    };
    let Ok(rules) = serde_json::from_str::<BTreeMap<String, String>>(&text) else {
        return Ok(0);
    };

    let original_len = rules.len();
    let safe = rules
        .into_iter()
        .filter(|(from, to)| safe_promoted_replacement(from, to))
        .collect::<BTreeMap<_, _>>();
    let removed = original_len.saturating_sub(safe.len());
    if removed == 0 {
        return Ok(0);
    }

    let text = serde_json::to_string_pretty(&safe).unwrap_or_else(|_| "{}".to_string());
    crate::private_file::write_private_text(path, &format!("{text}\n"))?;
    Ok(removed)
}

fn replace_visual_b_in_context(original: &str, target: &str) -> Option<String> {
    if !contains_visual_b_word(original) || !has_phrase_context(original) {
        return None;
    }

    let base = if has_cyrillic_text(original) {
        original
    } else {
        target
    };
    replace_visual_b_words(original, base)
}

fn has_phrase_context(text: &str) -> bool {
    text.split_whitespace().take(2).count() >= 2
}

pub(crate) fn replace_visual_b_words(original: &str, base: &str) -> Option<String> {
    if !has_phrase_context(original) {
        return None;
    }

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
            "b" if !latin_b_should_stay(&original_segments, idx, false) => Some(
                visual_b_replacement(&original_segments, &base_segments, idx, false),
            ),
            "B" if !latin_b_should_stay(&original_segments, idx, true) => Some(
                visual_b_replacement(&original_segments, &base_segments, idx, true),
            ),
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

fn latin_b_should_stay(segments: &[(&str, bool)], idx: usize, uppercase: bool) -> bool {
    let prev = previous_word_segment(segments, idx);
    let next = next_word_segment(segments, idx);
    uppercase && prev.is_some_and(is_ascii_word_token)
        || next.is_some_and(is_technical_ascii_word_token)
        || next.is_some_and(is_known_ascii_context_word)
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

fn next_word_segment<'a>(segments: &'a [(&str, bool)], idx: usize) -> Option<&'a str> {
    segments[idx + 1..]
        .iter()
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

fn is_technical_ascii_word_token(token: &str) -> bool {
    let (_, core, _) = split_word_punctuation(token);
    !core.is_empty() && is_common_en_technical_word(&core.to_ascii_lowercase())
}

fn is_known_ascii_context_word(token: &str) -> bool {
    let (_, core, _) = split_word_punctuation(token);
    !core.is_empty() && is_known_english_layout_autoswitch_word(&core.to_ascii_lowercase())
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
                    rules.extend(
                        custom
                            .into_iter()
                            .filter(|(from, to)| safe_promoted_replacement(from, to)),
                    );
                }
            }
        }

        rules
    })
}

pub fn safe_promoted_replacement(from: &str, to: &str) -> bool {
    let from_core = from.trim();
    let to_core = to.trim();
    if from_core.is_empty() || to_core.is_empty() || from_core == to_core {
        return false;
    }

    let from_words = from_core.split_whitespace().count();
    let to_words = to_core.split_whitespace().count();
    if from_words == 0 || to_words == 0 || to_words > 3 {
        return false;
    }
    if from_words > 1 {
        return true;
    }

    let from_letters = from_core.chars().filter(|ch| ch.is_alphabetic()).count();
    let to_letters = to_core.chars().filter(|ch| ch.is_alphabetic()).count();
    if to_letters + 1 < from_letters && !target_words_are_known(to_core) {
        return false;
    }
    if to_words == 1 && to_letters <= 3 && !target_words_are_known(to_core) {
        return false;
    }
    true
}

fn target_words_are_known(text: &str) -> bool {
    text.split_whitespace().all(|word| {
        let (_, core, _) = split_word_punctuation(word);
        !core.is_empty() && is_known_russian_word_or_form(&core.to_lowercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promoted_replacement_rejects_unknown_shorter_tail() {
        assert!(!safe_promoted_replacement("коти", "тки"));
    }

    #[test]
    fn promoted_replacement_allows_normal_typo_and_phrase_split() {
        assert!(safe_promoted_replacement("можн", "можно"));
        assert!(safe_promoted_replacement("нуда", "ну да"));
    }

    #[test]
    fn sanitize_replacement_rules_removes_unsafe_promotions() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-replacements-sanitize-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let path = tmp.join("replacements.json");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            &path,
            r#"{
  "коти": "тки",
  "можн": "можно",
  "нуда": "ну да"
}
"#,
        )
        .unwrap();

        assert_eq!(sanitize_replacement_rules_path(&path).unwrap(), 1);
        let rules: BTreeMap<String, String> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(!rules.contains_key("коти"));
        assert_eq!(rules.get("можн"), Some(&"можно".to_string()));
        assert_eq!(rules.get("нуда"), Some(&"ну да".to_string()));

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn visual_b_keeps_latin_labels_and_technical_context() {
        assert_eq!(
            replace_visual_b_words("wave b", "wave b").as_deref(),
            Some("wave и")
        );
        assert_eq!(replace_visual_b_words("vitamin B", "vitamin B"), None);
        assert_eq!(replace_visual_b_words("grade B", "grade B"), None);
        assert_eq!(replace_visual_b_words("HTML b tag", "HTML b tag"), None);
        assert_eq!(replace_visual_b_words("b tag", "b tag"), None);
    }
}
