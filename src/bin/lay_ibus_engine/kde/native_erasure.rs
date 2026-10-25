//! Expected document range after ordinary KDE input erases a new word.
//!
//! This supplies an observation shape, never edit permission. The common
//! native-erasure receipt must still observe the actual shortened document.
use crate::engine::SurroundingTextSnapshot;
use lay::desktop::LayoutBackend;

pub(crate) fn supports(backend: LayoutBackend) -> bool {
    backend == LayoutBackend::Kde
}

pub(crate) fn closed_word_after_erasure(
    backend: LayoutBackend,
    actual_before: &SurroundingTextSnapshot,
    tail_before: &str,
    tail_after: &str,
    erased_word: &str,
    observed_suffix_chars: u32,
) -> Option<String> {
    if !supports(backend)
        || observed_suffix_chars != 1
        || erased_word.chars().count() != 1
        || erased_word.chars().any(char::is_whitespace)
        || tail_before.strip_suffix(erased_word) != Some(tail_after)
        || !tail_after.ends_with(char::is_whitespace)
        || actual_before.has_selection()
        || actual_before.cursor_pos as usize != actual_before.text.chars().count()
    {
        return None;
    }
    let observed_tail = actual_before.suffix_before_cursor(tail_before.chars().count())?;
    // Rich clients can change the immediately preceding owned Space's
    // presentation when an ordinary letter is appended. Keep all other
    // characters literal; this is a range hypothesis, not an acknowledgement.
    let separator = tail_before.chars().count().checked_sub(2)?;
    if !same_with_owned_space(tail_before, &observed_tail, separator) {
        return None;
    }
    let end = tail_after.trim_end_matches(char::is_whitespace).len();
    if end == 0 {
        return None;
    }
    let start = tail_after[..end]
        .char_indices()
        .rev()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index + ch.len_utf8()))
        .unwrap_or(0);
    let closed = &tail_after[start..];
    if closed.chars().count() > crate::preedit::PREEDIT_TAIL_LIMIT {
        return None;
    }
    // A truncated mirror must not manufacture the beginning of a word.
    let document_start = actual_before
        .text
        .chars()
        .count()
        .checked_sub(1 + closed.chars().count())?;
    if document_start
        .checked_sub(1)
        .and_then(|index| actual_before.text.chars().nth(index))
        .is_some_and(|ch| !ch.is_whitespace())
    {
        return None;
    }
    Some(closed.to_owned())
}

/// The whole result of one observed native erasure must still match. Only
/// its final owned Space may return to the mirror's already witnessed form.
/// The caller retains all owner, Reset, caret, selection and revision checks.
pub(crate) fn closed_erasure_presentation_matches(
    backend: LayoutBackend,
    expected: &str,
    observed: &str,
    owned_closed: &str,
) -> bool {
    supports(backend)
        && !owned_closed.is_empty()
        && owned_closed.ends_with([' ', '\u{a0}'])
        && observed.ends_with(owned_closed)
        && expected
            .chars()
            .count()
            .checked_sub(1)
            .is_some_and(|slot| same_with_owned_space(expected, observed, slot))
}

fn same_with_owned_space(expected: &str, observed: &str, slot: usize) -> bool {
    let mut actual = observed.chars();
    expected.chars().enumerate().all(|(index, ch)| {
        actual.next().is_some_and(|other| {
            ch == other
                || (index == slot
                    && matches!(ch, ' ' | '\u{a0}')
                    && matches!(other, ' ' | '\u{a0}'))
        })
    }) && actual.next().is_none()
}
