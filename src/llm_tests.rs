use super::*;

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
    assert_eq!(
        repair_mixed_script("добавm d LLM"),
        Some("добавь в LLM".to_string())
    );
    assert_eq!(
        repair_mixed_script("ПРОВTHM WORD"),
        Some("ПРОВЕРЬ WORD".to_string())
    );
    assert_eq!(repair_mixed_script("ОБYJDB"), Some("ОБНОВИ".to_string()));
}

#[test]
fn does_not_glue_long_latin_tail_to_russian_word() {
    assert_eq!(repair_mixed_script("проверкаhrf ghj"), None);
    assert_eq!(
        convert_hybrid("проверкаhrf ghj", "ghjdthrfhrf ghj").unwrap(),
        Some("проверкаhrf ghj".to_string())
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
    assert_eq!(
        repair_mixed_script("AьщСКЬ Z"),
        Some("AmoCRM Я".to_string())
    );
    assert_eq!(
        repair_mixed_script("AmoСКЬ Z"),
        Some("AmoCRM Я".to_string())
    );
}

#[test]
fn keeps_plain_bilingual_text() {
    assert_eq!(repair_mixed_script("hello мир"), None);
    assert_eq!(repair_mixed_script("API для LLM"), None);
}

#[test]
fn hybrid_keeps_plain_bilingual_text_without_model() {
    assert_eq!(
        convert_hybrid("hello мир", "руддщ vbh").unwrap(),
        Some("hello мир".to_string())
    );
    assert_eq!(
        convert_hybrid("API для LLM", "ФЗШ lkz ДДЬ").unwrap(),
        Some("API для LLM".to_string())
    );
}

#[test]
fn hybrid_keeps_valid_russian_phrase_without_partial_single_letter_flip() {
    assert_eq!(
        convert_hybrid("в доме", "d ljvt").unwrap(),
        Some("в доме".to_string())
    );
}

#[test]
fn hybrid_keeps_domain_and_converts_neighbor_word() {
    assert_eq!(
        convert_hybrid("conecargo.ru cj,bhfq", "сщтусфкпщюкг собирай").unwrap(),
        Some("conecargo.ru собирай".to_string())
    );
}

#[test]
fn hybrid_keeps_mixed_case_ascii_brand_and_converts_neighbor_letter() {
    assert_eq!(
        convert_hybrid("AmoCRM Z", "ФьщСКЬ Я").unwrap(),
        Some("AmoCRM Я".to_string())
    );
}

#[test]
fn tokenwise_hybrid_keeps_good_word_and_converts_bad_neighbor() {
    let result = choose_mixed_token_candidate(
        "Главная Вщгиду",
        "Ukfdyfz Double",
        |original, _| {
            Ok(Some(if original == "Главная" {
                Choice::Original
            } else {
                Choice::Converted
            }))
        },
    )
    .unwrap();

    assert_eq!(result, Some("Главная Double".to_string()));
}

#[test]
fn tokenwise_hybrid_converts_unknown_long_all_caps_neighbor() {
    let result = choose_mixed_token_candidate(
        "DOUBLE DUBLE",
        "ВЩГИДУ ВГИДУ",
        |original, converted| {
            panic!("model should not be called for {original:?} -> {converted:?}");
        },
    )
    .unwrap();

    assert_eq!(result, Some("DOUBLE ВГИДУ".to_string()));
}

#[test]
fn tokenwise_hybrid_keeps_unknown_all_caps_brand_when_converted_is_garbage() {
    let result =
        choose_mixed_token_candidate("AMOCRM Z", "ФЬЩСКЬ Я", |original, converted| {
            panic!("model should not be called for {original:?} -> {converted:?}");
        })
        .unwrap();

    assert_eq!(result, Some("AMOCRM Я".to_string()));
}

#[test]
fn tokenwise_hybrid_converts_all_obvious_layout_garbage() {
    let result = choose_mixed_token_candidate("руддщ цщкдв", "hello world", |_, _| {
        Ok(Some(Choice::Converted))
    })
    .unwrap();

    assert_eq!(result, Some("hello world".to_string()));
}

#[test]
fn tokenwise_hybrid_converts_all_obviously_bad_words_without_model() {
    let result = choose_mixed_token_candidate(
        "dsdjlbv ldf",
        "выводим два",
        |original, converted| {
            panic!("model should not be called for {original:?} -> {converted:?}");
        },
    )
    .unwrap();

    assert_eq!(result, Some("выводим два".to_string()));
}

#[test]
fn tokenwise_hybrid_keeps_all_obviously_good_words_without_model() {
    let result = choose_mixed_token_candidate(
        "выводим два",
        "dsdjlbv ldf",
        |original, converted| {
            panic!("model should not be called for {original:?} -> {converted:?}");
        },
    )
    .unwrap();

    assert_eq!(result, Some("выводим два".to_string()));
}

#[test]
fn tokenwise_hybrid_keeps_obviously_good_russian_without_asking_model() {
    let result = choose_mixed_token_candidate(
        "Главная Вщгиду",
        "Ukfdyfz Double",
        |original, _| {
            assert_ne!(original, "Главная");
            Ok(Some(Choice::Converted))
        },
    )
    .unwrap();

    assert_eq!(result, Some("Главная Double".to_string()));
}

#[test]
fn tokenwise_hybrid_uses_dictionaries_before_model() {
    let result = choose_mixed_token_candidate(
        "Главное Вщгиду",
        "Ukfdyjt Double",
        |original, converted| {
            panic!("model should not be called for {original:?} -> {converted:?}");
        },
    )
    .unwrap();

    assert_eq!(result, Some("Главное Double".to_string()));
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
    let result = choose_mixed_token_candidate("рка ghj", "hrf про", |original, converted| {
        panic!("model should not be called for {original:?} -> {converted:?}");
    })
    .unwrap();

    assert_eq!(result, Some("рка про".to_string()));
}
