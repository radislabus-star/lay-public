use super::*;
use crate::typing_assist_test_fixtures::{fixture_lines_from_str, fixture_rows};

#[test]
fn reads_edge_whitespace_and_token_punctuation() {
    assert_eq!(split_edge_whitespace("  привет, "), ("  ", "привет,", " "));
    assert_eq!(split_word_punctuation("(привет),"), ("(", "привет", "),"));
}

#[test]
fn reads_and_replaces_last_text_word_without_touching_boundaries() {
    assert_eq!(last_text_word("  ну привет, "), Some("привет".to_string()));
    assert_eq!(
        replace_last_text_word("  ну привет, ", "здравствуй").as_deref(),
        Some("  ну здравствуй, ")
    );
}

#[test]
fn splits_whitespace_segments_without_losing_boundaries() {
    assert_eq!(
        split_ws_segments("как  проверить"),
        vec![("как", false), ("  ", true), ("проверить", false)]
    );
}

#[test]
fn reads_cyrillic_glued_word_split_boundaries() {
    for row in fixture_rows("word_reader_split_boundaries.tsv") {
        assert_eq!(row.len(), 5, "split boundary fixture must be TSV");
        let left_len: usize = row[3].parse().expect("left len");
        let right_len: usize = row[4].parse().expect("right len");
        let splits = cyrillic_word_splits(&row[0]);
        assert!(
            splits.iter().any(|split| {
                split.left == row[1]
                    && split.right == row[2]
                    && split.left_len == left_len
                    && split.right_len == right_len
            }),
            "missing split for {:?}",
            row[0]
        );
    }
    for word in fixture_lines_from_str(include_str!(
        "../tests/fixtures/word_reader_split_reject.txt"
    )) {
        assert!(cyrillic_word_splits(&word).is_empty());
    }
}

#[test]
fn reads_multiword_cyrillic_segmentations() {
    for row in fixture_rows("word_reader_segmentations.tsv") {
        assert_eq!(row.len(), 3, "segmentation fixture must be TSV");
        let max_parts: usize = row[1].parse().expect("max parts");
        let expected = row[2].split('|').collect::<Vec<_>>();
        let segmentations = cyrillic_word_segmentations(&row[0], max_parts);
        assert!(
            segmentations
                .iter()
                .any(|parts| parts.iter().copied().eq(expected.iter().copied())),
            "missing segmentation for {:?}",
            row[0]
        );
    }
    for row in fixture_rows("word_reader_segmentation_reject.tsv") {
        assert_eq!(row.len(), 2, "segmentation reject fixture must be TSV");
        let max_parts: usize = row[1].parse().expect("max parts");
        assert!(cyrillic_word_segmentations(&row[0], max_parts).is_empty());
    }
}
