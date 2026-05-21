//! Small Russian character helpers shared by correction layers.

pub(crate) fn is_russian_vowel(ch: char) -> bool {
    matches!(
        ch,
        'а' | 'е'
            | 'ё'
            | 'и'
            | 'о'
            | 'у'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'А'
            | 'Е'
            | 'Ё'
            | 'И'
            | 'О'
            | 'У'
            | 'Ы'
            | 'Э'
            | 'Ю'
            | 'Я'
    )
}

pub(crate) fn same_letter_ignore_case(left: char, right: char) -> bool {
    left.to_lowercase().to_string() == right.to_lowercase().to_string()
}
