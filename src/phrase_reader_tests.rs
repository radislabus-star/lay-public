use super::{
    correct_contextual_glued_tail, correct_glued_russian_phrase, correct_moved_prefix_letter_pair,
    correct_split_word_pair,
};

#[test]
fn splits_confident_glued_phrase_without_daemon_runtime() {
    assert_eq!(
        correct_glued_russian_phrase("тожесамое"),
        Some("тоже самое".to_string())
    );
    assert_eq!(
        correct_glued_russian_phrase("янебудузавас"),
        Some("я не буду за вас".to_string())
    );
}

#[test]
fn glued_phrase_defers_to_whole_word_typo_candidate() {
    assert_eq!(correct_glued_russian_phrase("переиспользоватся"), None);
}

#[test]
fn splits_contextual_glued_tail_in_short_phrase() {
    assert_eq!(
        correct_contextual_glued_tail("у насесть"),
        Some("у нас есть".to_string())
    );
    assert_eq!(correct_contextual_glued_tail("ноне ты"), None);
}

#[test]
fn merges_accidental_split_word_but_keeps_normal_pair() {
    assert_eq!(correct_split_word_pair("я вно"), Some("явно".to_string()));
    assert_eq!(correct_split_word_pair("я язык"), None);
    assert_eq!(correct_split_word_pair("про сою"), None);
}

#[test]
fn moves_next_word_prefix_back_when_phrase_score_is_confident() {
    assert_eq!(
        correct_moved_prefix_letter_pair("дл япроверки"),
        Some("для проверки".to_string())
    );
}
