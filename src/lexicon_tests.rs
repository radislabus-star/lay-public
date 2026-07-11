use super::*;
use crate::typing_assist_test_fixtures::{fixture_lines_from_str, fixture_rows};

#[test]
fn lexical_data_loads_without_code_word_lists() {
    for row in fixture_rows("lexicon_smoke_words.tsv") {
        assert_eq!(row.len(), 2, "lexicon smoke fixture must be TSV");
        let kind = row[0].as_str();
        let word = row[1].as_str();
        match kind {
            "common_ru" => assert!(is_common_ru_word(word), "missing common_ru: {word:?}"),
            "common_en_technical" => assert!(
                is_common_en_technical_word(word),
                "missing common_en_technical: {word:?}"
            ),
            "ru_one_letter_function" => assert!(
                is_ru_one_letter_function_word(word),
                "missing ru_one_letter_function: {word:?}"
            ),
            "ru_single_letter_pronoun" => assert!(
                is_ru_single_letter_pronoun(word),
                "missing ru_single_letter_pronoun: {word:?}"
            ),
            "ru_short_pronoun" => {
                assert!(
                    is_ru_short_pronoun(word),
                    "missing ru_short_pronoun: {word:?}"
                )
            }
            "ru_short_preposition" => assert!(
                is_ru_short_preposition(word),
                "missing ru_short_preposition: {word:?}"
            ),
            "ru_short_function" => assert!(
                is_ru_short_function_word(word),
                "missing ru_short_function: {word:?}"
            ),
            "ru_hyphen_particle" => assert!(
                is_ru_hyphen_particle(word),
                "missing ru_hyphen_particle: {word:?}"
            ),
            "not_common_en_technical" => assert!(
                !is_common_en_technical_word(word),
                "unexpected common_en_technical: {word:?}"
            ),
            "visual_b_default" => assert_eq!(visual_b_default_replacement(), word),
            "visual_b_after_ascii" => assert_eq!(visual_b_after_ascii_replacement(), word),
            other => panic!("unknown lexicon smoke kind: {other}"),
        }
    }
}

#[test]
fn protected_ascii_words_parser_keeps_short_user_tokens() {
    let source = include_str!("../tests/fixtures/lexicon_protected_ascii_source.txt");
    let words = parse_ascii_word_data(source, 1);

    for word in fixture_lines_from_str(include_str!(
        "../tests/fixtures/lexicon_protected_ascii_expected.txt"
    )) {
        assert!(words.contains(&word), "missing protected word: {word:?}");
    }
    for word in fixture_lines_from_str(include_str!(
        "../tests/fixtures/lexicon_protected_ascii_rejected.txt"
    )) {
        assert!(
            !words.contains(&word),
            "unexpected protected word: {word:?}"
        );
    }
}

#[test]
fn hunspell_ru_parser_keeps_common_lowercase_words_without_flags() {
    let words = parse_hunspell_ru_words(
        "6\n\
         Пропер/I\n\
         привет/K\n\
         следующий/A\n\
         улучшить/BLRW\n\
         wi-fi\n\
         ЧПУ\n\
         я\n",
    );

    assert!(words.contains(&"привет".to_string()));
    assert!(words.contains(&"следующий".to_string()));
    assert!(words.contains(&"улучшить".to_string()));
    assert!(!words.contains(&"пропер".to_string()));
    assert!(!words.contains(&"wi-fi".to_string()));
    assert!(!words.contains(&"чпу".to_string()));
    assert!(!words.contains(&"я".to_string()));
}
