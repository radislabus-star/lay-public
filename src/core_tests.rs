use super::*;
use crate::typing_assist_test_fixtures::text_replacement;

fn key_events(text: &str, layout_is_ru: bool) -> Vec<KeyEvent> {
    text_to_key_events(text, layout_is_ru).expect("core facade fixture must be typable")
}

#[test]
fn facade_exposes_layout_conversion_and_backend_detection() {
    assert_eq!(convert("ghbdtn", Direction::Us2Ru), "привет");
    assert_eq!(
        resolve_layout_backend("auto", Some("KDE"), Some("plasma"), Some("wayland")),
        LayoutBackend::Kde
    );
    assert!(is_ru_layout_id("xkb:ru::rus"));
}

#[test]
fn facade_exposes_candidate_scoring() {
    let decision = rank_typing_candidates([
        TypingCandidate::new("missing_letter", 10, "кторое ", "которое ".to_string()),
        TypingCandidate::new("glued_phrase", 200, "кторое ", "к торое ".to_string()),
    ])
    .expect("typing decision");
    assert!(decision.margin.is_finite());
    assert!(matches!(
        classify_typing_confidence(true, Some(decision.margin), 0.0),
        TypingDecisionConfidence::Strong
    ));
}

#[test]
fn facade_exposes_minimal_text_replacement() {
    let plan = plan_text_replacement("NEN DOUBLE", "ТУТ DOUBLE");
    let plan = plan.expect("replacement plan");
    assert!(replacement_plan_matches("NEN DOUBLE", "ТУТ DOUBLE", &plan));
    assert_eq!(
        apply_replacement_plan_to_text("NEN DOUBLE", &plan),
        "ТУТ DOUBLE"
    );
    assert!(committed_separator_is_preserved("кторое ", "которое "));
    assert!(!committed_separator_is_preserved("кторое ", "которое"));
    assert_eq!(plan, text_replacement(7, 3, "ТУТ", 7));
}

#[test]
fn facade_exposes_correction_contract() {
    assert!(Correction::InsertText("Double".to_string()).is_insert_text());
    assert!(!Correction::ReplayAll.is_insert_text());
}

#[test]
fn facade_exposes_decoder_contract() {
    let events = key_events("good", false);
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: "good",
        converted: "пщщв",
        engine: CorrectionEngine::Smart,
        force_replay: true,
        auto_replace: true,
    });

    assert_eq!(result.action, DecoderAction::ReplayAll);
}

#[test]
fn facade_exposes_physical_keyboard_mapping() {
    let events = key_events("ltkfq", false);

    assert_eq!(map_original_events(&events), "ltkfq");
    assert_eq!(map_events_to_layout(&events, true), "делай");
}

#[test]
fn facade_exposes_text_to_uinput_runs() {
    let runs = text_to_uinput_runs("Привет Double", true).expect("runs");

    assert_eq!(runs.len(), 2);
    assert!(runs[0].target_is_ru);
    assert!(!runs[1].target_is_ru);
    assert!(preferred_layout_for_text("AmoCRM Я", false));
}

#[test]
fn facade_exposes_text_backend_contract() {
    assert_eq!(
        TextBackendPreference::parse("ime"),
        TextBackendPreference::Ime
    );
    assert_eq!(
        ImeReplaceRequest::committed_tail("мы сами ", "мы сами ").backspaces,
        0
    );
    let request = ImeReplaceRequest::committed_tail("мы сами ", "мы самы ");
    assert_eq!((request.backspaces, request.text.as_str()), (2, "ы "));
    assert!(TextBackendCapabilities::ime().can_atomic_replace());
    assert_eq!(
        TextBackendCapabilities::uinput().replace,
        TextReplaceCapability::KeyReplay
    );
}

#[test]
fn facade_exposes_replay_layout_decision() {
    let events = key_events("lt", false);

    assert_eq!(
        replay_layout_decision(&events),
        ReplayLayoutDecision {
            target_is_ru: true,
            mixed_layouts: false,
        }
    );
}

#[test]
fn facade_exposes_word_event_splitting_and_text_tail() {
    let events = key_events("a b", false);
    let words = split_event_words(&events).expect("words");

    assert_eq!(words.len(), 2);
    assert_eq!(tail_chars("привет", 3), "вет");
}

#[test]
fn facade_exposes_word_buffer() {
    let mut buffer = WordBuffer::new();
    buffer.push(key_events("l", false).remove(0));

    let (events, backspaces) = buffer.what_to_replay(MAX_REPLACE_WORDS).expect("tail");

    assert_eq!(backspaces, 1);
    assert_eq!(map_original_events(&events), "l");
}
