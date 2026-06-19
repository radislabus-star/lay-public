use crate::layout_autoswitch::is_known_english_layout_autoswitch_word;
use crate::word_reader::split_word_punctuation;
use crate::word_recognizer::is_protected_ascii_token;

pub(super) fn should_keep_latin_b_context(segments: &[(&str, bool)], idx: usize) -> bool {
    let Some((token, false)) = segments.get(idx) else {
        return false;
    };
    if !matches!(*token, "b" | "B") {
        return false;
    }

    let prev = previous_word_segment(segments, idx);
    let next = next_word_segment(segments, idx);
    (*token == "B" && prev.is_some_and(is_ascii_context_word))
        || (prev.is_some_and(is_ascii_context_word) && next.is_some_and(is_ascii_context_word))
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

fn is_ascii_context_word(token: &str) -> bool {
    let (_, core, _) = split_word_punctuation(token);
    !core.is_empty()
        && (is_protected_ascii_token(core)
            || is_known_english_layout_autoswitch_word(&core.to_ascii_lowercase()))
}
