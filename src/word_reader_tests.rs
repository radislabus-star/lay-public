use super::*;

#[test]
fn reads_edge_whitespace_and_token_punctuation() {
    assert_eq!(split_edge_whitespace("  привет, "), ("  ", "привет,", " "));
    assert_eq!(split_word_punctuation("(привет),"), ("(", "привет", "),"));
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
    let splits = cyrillic_word_splits("тожесамое");
    assert!(splits.iter().any(|split| {
        split.left == "тоже"
            && split.right == "самое"
            && split.left_len == 4
            && split.right_len == 5
    }));
    assert!(cyrillic_word_splits("wi-fi").is_empty());
    assert!(cyrillic_word_splits("пара-пара").is_empty());
}

#[test]
fn reads_multiword_cyrillic_segmentations() {
    let segmentations = cyrillic_word_segmentations("янебудузавас", 5);
    assert!(segmentations
        .iter()
        .any(|parts| { parts.as_slice() == ["я", "не", "буду", "за", "вас"] }));
    let segmentations = cyrillic_word_segmentations("янебуду", 7);
    assert!(segmentations
        .iter()
        .any(|parts| { parts.as_slice() == ["я", "не", "буду"] }));
    assert!(cyrillic_word_segmentations("wi-fi", 5).is_empty());
}
