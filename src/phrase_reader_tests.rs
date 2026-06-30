use super::{
    correct_contextual_glued_tail, correct_glued_russian_phrase, correct_moved_prefix_letter_pair,
    correct_split_word_pair,
};
use crate::typing_assist_test_fixtures::fixture_rows;

#[test]
fn splits_confident_glued_phrase_without_daemon_runtime() {
    for row in fixture_rows("phrase_reader_glued.tsv") {
        assert_eq!(row.len(), 2, "glued phrase fixture must be TSV");
        assert_eq!(correct_glued_russian_phrase(&row[0]), Some(row[1].clone()));
    }
}

#[test]
fn glued_phrase_defers_to_whole_word_typo_candidate() {
    assert_eq!(correct_glued_russian_phrase("переиспользоватся"), None);
}

#[test]
fn splits_short_function_glued_to_be_form() {
    assert_eq!(
        correct_glued_russian_phrase("тоесть"),
        Some("то есть".to_string())
    );
}

#[test]
fn splits_contextual_glued_tail_in_short_phrase() {
    for row in fixture_rows("phrase_reader_contextual_glued.tsv") {
        assert_eq!(row.len(), 2, "contextual glued fixture must be TSV");
        assert_eq!(correct_contextual_glued_tail(&row[0]), Some(row[1].clone()));
    }
    for row in fixture_rows("phrase_reader_contextual_keep.tsv") {
        assert_eq!(row.len(), 1, "contextual keep fixture must have one field");
        assert_eq!(correct_contextual_glued_tail(&row[0]), None);
    }
}

#[test]
fn merges_accidental_split_word_but_keeps_normal_pair() {
    for row in fixture_rows("phrase_reader_split_pair.tsv") {
        assert_eq!(row.len(), 2, "split pair fixture must be TSV");
        assert_eq!(correct_split_word_pair(&row[0]), Some(row[1].clone()));
    }
    for row in fixture_rows("phrase_reader_split_pair_keep.tsv") {
        assert_eq!(row.len(), 1, "split pair keep fixture must have one field");
        assert_eq!(correct_split_word_pair(&row[0]), None);
    }
    assert_eq!(correct_split_word_pair("реально ое"), None);
    assert_eq!(correct_split_word_pair("я дам"), None);
    assert_eq!(correct_split_word_pair("я был"), None);
}

#[test]
fn moves_next_word_prefix_back_when_phrase_score_is_confident() {
    for row in fixture_rows("phrase_reader_moved_prefix.tsv") {
        assert_eq!(row.len(), 2, "moved prefix fixture must be TSV");
        assert_eq!(
            correct_moved_prefix_letter_pair(&row[0]),
            Some(row[1].clone())
        );
    }
}
