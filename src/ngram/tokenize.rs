use super::Lang;

pub fn tokenize_text(text: &str, lang: Lang) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphabetic() && ch != '-')
        .filter_map(|word| normalize_word(word, lang))
        .collect()
}

pub(super) fn normalize_text(text: &str, lang: Lang) -> Option<String> {
    let tokens = tokenize_text(text, lang);
    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" "))
    }
}

pub(super) fn normalize_word(word: &str, lang: Lang) -> Option<String> {
    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return None;
    }
    if !word.chars().all(|ch| is_word_char(ch, lang)) {
        return None;
    }
    Some(word)
}

fn is_word_char(ch: char, lang: Lang) -> bool {
    match lang {
        Lang::Ru => matches!(ch, 'а'..='я' | 'ё' | '-'),
        Lang::En => ch.is_ascii_lowercase() || ch == '-',
    }
}
