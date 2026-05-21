use super::*;

#[test]
fn smart_decision_replays_single_valid_word_as_manual_toggle() {
    assert_eq!(
        decide_correction("DOUBLE", "ВЩГИДУ", CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn single_word_wrong_layout_replay_target_is_opposite_layout() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "ltkfq", false);
    let (events, backspaces) = buffer.what_to_replay(1).expect("single word");
    let decision = replay_layout_decision(&events);

    assert_eq!(backspaces, 5);
    assert_eq!(map_original_events(&events), "ltkfq");
    assert!(decision.target_is_ru);
    assert_eq!(map_target_events(&events, decision.target_is_ru), "делай");
    assert_eq!(
        decide_correction("ltkfq", "делай", CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn single_currency_tail_replays_ru_semicolon_as_us_dollar() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_4, false),
            (KeyCode::KEY_0, false),
            (KeyCode::KEY_0, false),
            (KeyCode::KEY_0, false),
            (KeyCode::KEY_4, true),
        ],
        true,
    );
    let (events, backspaces) = buffer.what_to_replay(1).expect("single word");
    let decision = replay_layout_decision(&events);
    let original = map_original_events(&events);
    let target = map_target_events(&events, decision.target_is_ru);

    assert_eq!(backspaces, 5);
    assert_eq!(original, "4000;");
    assert!(!decision.target_is_ru);
    assert_eq!(target, "4000$");
    assert_eq!(
        decide_correction(&original, &target, CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_replays_single_cyrillic_acronym_as_manual_toggle() {
    let events = [
        KeyEvent {
            keycode: KeyCode::KEY_L.code(),
            shift: true,
            layout_is_ru: true,
        },
        KeyEvent {
            keycode: KeyCode::KEY_L.code(),
            shift: true,
            layout_is_ru: true,
        },
        KeyEvent {
            keycode: KeyCode::KEY_M.code(),
            shift: true,
            layout_is_ru: true,
        },
    ];
    let decision = replay_layout_decision(&events);
    let original = map_original_events(&events);
    let target = map_events_to_layout(&events, decision.target_is_ru);

    assert_eq!(original, "ДДЬ");
    assert_eq!(target, "LLM");
    assert!(!decision.target_is_ru);
    assert_eq!(
        decide_correction(&original, &target, CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_replays_two_valid_words_as_manual_toggle() {
    assert_eq!(
        decide_correction("выводим два", "dsdjlbv ldf", CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_replays_valid_russian_preposition_phrase_as_manual_toggle() {
    assert_eq!(
        decide_correction("в доме", "d ljvt", CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_converts_mixed_layout_neighbor_only() {
    assert_eq!(
        decide_correction("рка ghj", "hrf про", CorrectionEngine::Smart),
        Correction::InsertText("рка про".to_string())
    );
    assert_eq!(
        decide_correction("проверка ghj", "ghjdthrf про", CorrectionEngine::Smart),
        Correction::InsertText("проверка про".to_string())
    );
}

#[test]
fn smart_decision_replays_protected_ascii_span_as_manual_toggle() {
    assert_eq!(
        decide_correction("AmoCRM Я", "ФьщСКЬ Z", CorrectionEngine::Smart),
        Correction::ReplayAll
    );
}

#[test]
fn smart_decision_repairs_brand_plus_letter_inside_larger_tail() {
    assert_eq!(
        decide_correction(
            "AmoCRM Z тут задача",
            "ФьщСКЬ Я nen pflfxf",
            CorrectionEngine::Smart
        ),
        Correction::InsertText("AmoCRM Я тут задача".to_string())
    );
}
