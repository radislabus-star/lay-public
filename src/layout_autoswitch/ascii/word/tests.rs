use super::{correct_confident_wrong_layout_ascii_word, correct_wrong_layout_ascii_word};

#[test]
fn confident_ascii_layout_preserves_trailing_punctuation() {
    assert_eq!(
        correct_confident_wrong_layout_ascii_word("ghbdtn,").as_deref(),
        Some("привет,")
    );
}

#[test]
fn internal_layout_punctuation_still_converts_through_confident_path() {
    assert_eq!(
        correct_confident_wrong_layout_ascii_word("ghj,ktvf").as_deref(),
        Some("проблема")
    );
}

#[test]
fn wrong_layout_trailing_question_mark_can_convert_to_russian_comma() {
    assert_eq!(
        correct_confident_wrong_layout_ascii_word("ckjdf?").as_deref(),
        Some("слова,")
    );
}

#[test]
fn protected_english_token_with_punctuation_stays_ascii() {
    assert_eq!(correct_confident_wrong_layout_ascii_word("file,"), None);
    assert_eq!(correct_wrong_layout_ascii_word("file,"), None);
}
