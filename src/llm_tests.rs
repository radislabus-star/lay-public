use super::*;
use crate::typing_assist_test_fixtures::{first_fixture_row, fixture_row_by_id, fixture_rows};

fn tokenwise_case(id: &str) -> Vec<String> {
    fixture_row_by_id("llm_tokenwise_mixed.tsv", id)
}

#[test]
fn parses_letter_choices() {
    assert_eq!(parse_choice("A"), Some(Choice::Original));
    assert_eq!(parse_choice(" B"), Some(Choice::Converted));
    assert_eq!(parse_choice("A:"), Some(Choice::Original));
    assert_eq!(parse_choice("Answer: B"), Some(Choice::Converted));
    assert_eq!(parse_choice("To convert"), None);
}

#[test]
fn parses_openai_chat_choice_response() {
    let resp: OpenAiChatResponse =
        serde_json::from_str(r#"{"choices":[{"message":{"content":"B"}}]}"#).unwrap();
    assert_eq!(
        resp.choices
            .first()
            .and_then(|choice| choice.message.content.as_deref())
            .and_then(parse_choice),
        Some(Choice::Converted)
    );
}

#[test]
fn parses_anthropic_text_choice_response() {
    let resp: AnthropicResponse =
        serde_json::from_str(r#"{"content":[{"type":"text","text":"A"}]}"#).unwrap();
    assert_eq!(
        resp.content
            .iter()
            .filter(|part| part.kind == "text")
            .filter_map(|part| part.text.as_deref())
            .find_map(parse_choice),
        Some(Choice::Original)
    );
}

#[test]
fn token_consensus_lets_llm_veto_converted_choice() {
    let veto = |_original: &str, _converted: &str| Ok(Some(Choice::Original));
    assert_eq!(
        choose_token_consensus_with_chooser("dsdjlbv", "выводим", veto).unwrap(),
        Some("dsdjlbv".to_string())
    );
}

#[test]
fn token_consensus_accepts_converted_when_llm_agrees() {
    let agree = |_original: &str, _converted: &str| Ok(Some(Choice::Converted));
    assert_eq!(
        choose_token_consensus_with_chooser("dsdjlbv", "выводим", agree).unwrap(),
        Some("выводим".to_string())
    );
}

#[test]
fn token_consensus_keeps_protected_ascii_without_model() {
    let panic_chooser =
        |_original: &str, _converted: &str| panic!("protected token must not ask model");
    assert_eq!(
        choose_token_consensus_with_chooser("AmoCRM", "ФьщСКЬ", panic_chooser).unwrap(),
        Some("AmoCRM".to_string())
    );
}

#[test]
fn repairs_mixed_russian_with_latin_islands() {
    for row in fixture_rows("llm_mixed_script_repair.tsv") {
        assert_eq!(row.len(), 2, "mixed-script fixture must be TSV");
        assert_eq!(repair_mixed_script(&row[0]), Some(row[1].clone()));
    }
}

#[test]
fn does_not_glue_long_latin_tail_to_russian_word() {
    let row = first_fixture_row("llm_no_latin_tail_glue.tsv");
    assert_eq!(repair_mixed_script(&row[0]), None);
    assert_eq!(
        convert_hybrid(&row[0], &row[1]).unwrap(),
        Some(row[2].clone())
    );
}

#[test]
fn does_not_treat_cyrillic_layout_word_with_latin_tail_as_ascii_brand() {
    assert_eq!(repair_mixed_script("ВщгиDo"), None);
    assert_eq!(
        convert_hybrid("в ВщгиDo", "d DoubDo").unwrap(),
        Some("в ВщгиDo".to_string())
    );
}

#[test]
fn repairs_mixed_ascii_brand_tokens_before_layout_islands() {
    for row in fixture_rows("llm_mixed_brand_repair.tsv") {
        assert_eq!(row.len(), 2, "mixed brand fixture must be TSV");
        assert_eq!(repair_mixed_script(&row[0]), Some(row[1].clone()));
    }
}

#[test]
fn keeps_plain_bilingual_text() {
    for row in fixture_rows("llm_plain_bilingual_keep.txt") {
        assert_eq!(row.len(), 1, "plain bilingual fixture must have one field");
        assert_eq!(repair_mixed_script(&row[0]), None);
    }
}

#[test]
fn hybrid_keeps_plain_bilingual_text_without_model() {
    for row in fixture_rows("llm_hybrid_plain_bilingual.tsv") {
        assert_eq!(row.len(), 3, "hybrid bilingual fixture must be TSV");
        assert_eq!(
            convert_hybrid(&row[0], &row[1]).unwrap(),
            Some(row[2].clone())
        );
    }
}

#[test]
fn hybrid_keeps_valid_russian_phrase_without_partial_single_letter_flip() {
    let row = first_fixture_row("llm_hybrid_valid_russian.tsv");
    assert_eq!(
        convert_hybrid(&row[0], &row[1]).unwrap(),
        Some(row[2].clone())
    );
}

#[test]
fn hybrid_keeps_domain_and_converts_neighbor_word() {
    let row = first_fixture_row("llm_hybrid_domain.tsv");
    assert_eq!(
        convert_hybrid(&row[0], &row[1]).unwrap(),
        Some(row[2].clone())
    );
}

#[test]
fn hybrid_keeps_mixed_case_ascii_brand_and_converts_neighbor_letter() {
    let row = first_fixture_row("llm_hybrid_brand.tsv");
    assert_eq!(
        convert_hybrid(&row[0], &row[1]).unwrap(),
        Some(row[2].clone())
    );
}

#[test]
fn tokenwise_hybrid_keeps_good_word_and_converts_bad_neighbor() {
    let row = tokenwise_case("llm_veto");
    let protected = row[3].clone();
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, _| {
        Ok(Some(if original == protected {
            Choice::Original
        } else {
            Choice::Converted
        }))
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_converts_unknown_long_all_caps_neighbor() {
    let row = tokenwise_case("all_caps_neighbor");
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_keeps_unknown_all_caps_brand_when_converted_is_garbage() {
    let row = tokenwise_case("all_caps_brand");
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_converts_all_obvious_layout_garbage() {
    let row = tokenwise_case("all_bad");
    let result =
        choose_mixed_token_candidate(&row[1], &row[2], |_, _| Ok(Some(Choice::Converted))).unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_converts_all_obviously_bad_words_without_model() {
    let row = tokenwise_case("all_bad_no_model");
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_keeps_all_obviously_good_words_without_model() {
    let row = tokenwise_case("all_good_no_model");
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_keeps_obviously_good_russian_without_asking_model() {
    let row = tokenwise_case("dictionary_before_model_ru");
    let protected = row[3].clone();
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, _| {
        assert_ne!(original, protected);
        Ok(Some(Choice::Converted))
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn tokenwise_hybrid_uses_dictionaries_before_model() {
    let row = tokenwise_case("dictionary_before_model_main");
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}

#[test]
fn token_hybrid_keeps_good_previous_word_or_converts_bad_one() {
    assert_eq!(
        choose_token_hybrid("в", "d").unwrap(),
        Some("в".to_string())
    );
    assert_eq!(
        choose_token_hybrid("ghbdtn", "привет").unwrap(),
        Some("привет".to_string())
    );
    assert_eq!(
        choose_token_hybrid("DOUBLE", "ВЩГИДУ").unwrap(),
        Some("DOUBLE".to_string())
    );
}

#[test]
fn tokenwise_hybrid_converts_bad_mixed_layout_neighbor_only() {
    let row = tokenwise_case("bad_mixed_neighbor");
    let result = choose_mixed_token_candidate(&row[1], &row[2], |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some(row[4].clone()));
}
