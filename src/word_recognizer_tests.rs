use super::*;
use crate::typing_assist_test_fixtures::{fixture_rows, parse_bool_fixture};

#[test]
fn recognizes_plain_words_and_technical_tokens() {
    for row in fixture_rows("word_recognizer_identity.tsv") {
        assert_eq!(row.len(), 7, "word recognizer fixture must be TSV");
        let identity = recognize_token(&row[0]);
        assert_eq!(identity.script, word_script(&row[1]), "token={:?}", row[0]);
        assert_eq!(identity.kind, word_kind(&row[2]), "token={:?}", row[0]);
        assert_eq!(
            identity.known_ru,
            parse_bool_fixture(&row[3]),
            "token={:?}",
            row[0]
        );
        assert_eq!(
            identity.known_en,
            parse_bool_fixture(&row[4]),
            "token={:?}",
            row[0]
        );
        assert_eq!(
            identity.protected,
            parse_bool_fixture(&row[5]),
            "token={:?}",
            row[0]
        );
        assert_eq!(
            identity.technical,
            parse_bool_fixture(&row[6]),
            "token={:?}",
            row[0]
        );
    }

    for row in fixture_rows("word_recognizer_predicates.tsv") {
        assert_eq!(
            row.len(),
            3,
            "word recognizer predicate fixture must be TSV"
        );
        let got = match row[0].as_str() {
            "ascii_technical" => is_ascii_technical_token(&row[1]),
            "ascii_technical_or_brand" => is_ascii_technical_or_brand_token(&row[1]),
            "protected_ascii" => is_protected_ascii_token(&row[1]),
            "ascii_titlecase" => is_ascii_titlecase_token(&row[1]),
            "mixed_cyrillic_ascii_alpha" => is_mixed_cyrillic_ascii_alpha_token(&row[1]),
            other => panic!("unknown word recognizer predicate: {other}"),
        };
        assert_eq!(got, parse_bool_fixture(&row[2]), "predicate={:?}", row[0]);
    }
}

#[test]
fn plain_layout_word_autocorrect_is_risky() {
    for row in fixture_rows("word_recognizer_plain_layout_risk.tsv") {
        assert_eq!(row.len(), 3, "plain layout risk fixture must be TSV");
        assert_eq!(
            is_plain_layout_autocorrect_risky(&row[0], &row[1]),
            parse_bool_fixture(&row[2]),
            "original={:?} replacement={:?}",
            row[0],
            row[1]
        );
    }
}

#[test]
fn technical_and_mixed_tokens_are_not_plain_layout_risk() {
    for row in fixture_rows("word_recognizer_technical_layout_risk.tsv") {
        assert_eq!(row.len(), 3, "technical layout risk fixture must be TSV");
        assert_eq!(
            is_plain_layout_autocorrect_risky(&row[0], &row[1]),
            parse_bool_fixture(&row[2]),
            "original={:?} replacement={:?}",
            row[0],
            row[1]
        );
    }
}

fn word_script(value: &str) -> WordScript {
    match value {
        "Cyrillic" => WordScript::Cyrillic,
        "Ascii" => WordScript::Ascii,
        "Mixed" => WordScript::Mixed,
        "Numeric" => WordScript::Numeric,
        "Other" => WordScript::Other,
        "Empty" => WordScript::Empty,
        other => panic!("unknown word script: {other}"),
    }
}

fn word_kind(value: &str) -> WordKind {
    match value {
        "PlainWord" => WordKind::PlainWord,
        "TechnicalToken" => WordKind::TechnicalToken,
        "CliOption" => WordKind::CliOption,
        "Number" => WordKind::Number,
        "MixedScript" => WordKind::MixedScript,
        "Other" => WordKind::Other,
        "Empty" => WordKind::Empty,
        other => panic!("unknown word kind: {other}"),
    }
}
