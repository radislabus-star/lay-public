use crate::phrase_lexicon::is_known_russian_phrase_part;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation, split_ws_segments};
use crate::word_recognizer::{recognize_token, WordKind, WordScript};

const CONTEXT_SCAN_WORDS: usize = 3;

pub(crate) fn correct_contextual_ascii_preposition_v(text: &str) -> Option<String> {
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
        if !is_ascii_d_preposition_candidate(segment) {
            continue;
        }
        if !left_has_russian_phrase_support(&segments, word_indices[..position].iter().rev()) {
            continue;
        }
        if !right_has_preposition_object_support(&segments, word_indices[position + 1..].iter()) {
            continue;
        }
        if !nearby_has_technical_anchor(
            &segments,
            word_indices[..position]
                .iter()
                .rev()
                .chain(word_indices[position + 1..].iter()),
        ) {
            continue;
        }
        if replacement_index.replace(segment_idx).is_some() {
            return None;
        }
    }

    let replacement_index = replacement_index?;
    let mut out = String::with_capacity(text.len() + "в".len());
    for (idx, (segment, is_ws)) in segments.iter().enumerate() {
        if idx == replacement_index && !*is_ws {
            out.push('в');
        } else {
            out.push_str(segment);
        }
    }

    (out != text).then_some(out)
}

fn is_ascii_d_preposition_candidate(token: &str) -> bool {
    let (leading, word, trailing) = split_word_punctuation(token);
    leading.is_empty() && trailing.is_empty() && word == "d"
}

fn left_has_russian_phrase_support<'a, I>(segments: &[(&str, bool)], word_indices: I) -> bool
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
        if is_strong_left_russian_phrase_support(token) {
            return true;
        }
    }
    false
}

fn right_has_preposition_object_support<'a, I>(segments: &[(&str, bool)], word_indices: I) -> bool
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
        if is_russian_phrase_support(token) || is_technical_ascii_object(token) {
            return true;
        }
    }
    false
}

fn nearby_has_technical_anchor<'a, I>(segments: &[(&str, bool)], word_indices: I) -> bool
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
        if is_technical_ascii_object(token) {
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

fn is_strong_left_russian_phrase_support(token: &str) -> bool {
    let (_, word, _) = split_word_punctuation(token);
    if word.is_empty() || !is_cyrillic_word(word) {
        return false;
    }
    let lower = word.to_lowercase();
    lower.chars().count() >= 4 && is_known_russian_phrase_part(&lower)
}

fn is_technical_ascii_object(token: &str) -> bool {
    let identity = recognize_token(token);
    identity.script == WordScript::Ascii && identity.technical
}

fn is_hard_context_barrier(token: &str) -> bool {
    let identity = recognize_token(token);
    identity.kind == WordKind::CliOption
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
