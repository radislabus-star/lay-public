//! Case restoration helpers for deterministic text corrections.

pub(crate) fn apply_phrase_case(original: &str, replacement_lower: &str) -> String {
    if original.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        capitalize_first(replacement_lower)
    } else {
        replacement_lower.to_string()
    }
}

pub(crate) fn apply_word_case(original: &str, replacement_lower: &str) -> String {
    if original
        .chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_uppercase())
    {
        replacement_lower.to_uppercase()
    } else if original.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        capitalize_first(replacement_lower)
    } else {
        replacement_lower.to_string()
    }
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::new();
    out.extend(first.to_uppercase());
    out.push_str(chars.as_str());
    out
}
