use crate::phrase_lexicon::is_known_russian_phrase_part;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation, split_ws_segments};
use crate::word_recognizer::{recognize_token, WordKind, WordScript};

const CONTEXT_SCAN_WORDS: usize = 3;

pub(crate) fn correct_contextual_ascii_conjunction_i(text: &str) -> Option<String> {
    let segments = split_ws_segments(text);
    let word_indices: Vec<usize> = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect();

    if word_indices.len() < 3 {
        return None;
    }

    let mut replacement_index = None;
    for (position, segment_idx) in word_indices.iter().copied().enumerate() {
        let (segment, _) = segments[segment_idx];
        if !is_ascii_b_conjunction_candidate(segment) {
            continue;
        }
        if !side_has_russian_phrase_support(&segments, word_indices[..position].iter().rev()) {
            continue;
        }
        if !side_has_russian_phrase_support(&segments, word_indices[position + 1..].iter()) {
            continue;
        }
        if replacement_index.replace(segment_idx).is_some() {
            return None;
        }
    }

    let replacement_index = replacement_index?;
    let mut out = String::with_capacity(text.len() + "и".len());
    for (idx, (segment, is_ws)) in segments.iter().enumerate() {
        if idx == replacement_index && !*is_ws {
            out.push('и');
        } else {
            out.push_str(segment);
        }
    }

    (out != text).then_some(out)
}

fn is_ascii_b_conjunction_candidate(token: &str) -> bool {
    let (leading, word, trailing) = split_word_punctuation(token);
    leading.is_empty() && trailing.is_empty() && word == "b"
}

fn side_has_russian_phrase_support<'a, I>(segments: &[(&str, bool)], word_indices: I) -> bool
where
    I: Iterator<Item = &'a usize>,
{
    for (scanned, segment_idx) in word_indices.enumerate() {
        if scanned >= CONTEXT_SCAN_WORDS {
            break;
        }
        let token = segments[*segment_idx].0;
        if is_hard_context_barrier(token) {
            return false;
        }
        if is_russian_phrase_support(token) {
            return true;
        }
    }
    false
}

fn is_russian_phrase_support(token: &str) -> bool {
    let (_, word, _) = split_word_punctuation(token);
    if word.is_empty() || !is_cyrillic_word(word) {
        return false;
    }
    let lower = word.to_lowercase();
    is_known_russian_phrase_part(&lower) || lower.chars().count() >= 4
}

fn is_hard_context_barrier(token: &str) -> bool {
    let identity = recognize_token(token);
    identity.kind == WordKind::CliOption
        || (identity.script == WordScript::Ascii && !identity.technical)
        || matches!(
            identity.script,
            WordScript::Mixed | WordScript::Numeric | WordScript::Other
        )
        || has_hard_ascii_separator(identity.core)
}

fn has_hard_ascii_separator(core: &str) -> bool {
    core.chars().any(|ch| {
        matches!(
            ch,
            '/' | '\\' | '@' | '#' | '$' | '%' | '^' | '&' | '|' | '='
        )
    })
}
