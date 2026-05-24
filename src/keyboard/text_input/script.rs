pub fn is_cyrillic_letter(ch: char) -> bool {
    matches!(ch, 'А'..='я' | 'ё' | 'Ё')
}

#[inline]
pub fn preferred_layout_for_text(text: &str, fallback_is_ru: bool) -> bool {
    text.chars()
        .rev()
        .find_map(|ch| {
            if is_cyrillic_letter(ch) {
                Some(true)
            } else if ch.is_ascii_alphabetic() {
                Some(false)
            } else {
                None
            }
        })
        .unwrap_or(fallback_is_ru)
}
