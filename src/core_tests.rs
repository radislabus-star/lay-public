use super::*;

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
    let best =
        best_candidate("ghbdtn", ["ghbdtn".to_string(), "привет".to_string()]).expect("candidate");
    assert_eq!(best.text, "привет");
}

#[test]
fn facade_exposes_minimal_text_replacement() {
    assert_eq!(
        plan_text_replacement("NEN DOUBLE", "ТУТ DOUBLE"),
        Some(TextReplacement {
            move_left: 7,
            backspaces: 3,
            insert: "ТУТ".to_string(),
            move_right: 7,
        })
    );
}

#[test]
fn facade_exposes_correction_contract() {
    assert!(Correction::InsertText("Double".to_string()).is_insert_text());
    assert!(!Correction::ReplayAll.is_insert_text());
}

#[test]
fn facade_exposes_decoder_contract() {
    let events = [
        KeyEvent {
            keycode: evdev::KeyCode::KEY_G.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_O.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_O.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_D.code(),
            shift: false,
            layout_is_ru: false,
        },
    ];
    let result = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: "good",
        converted: "пщщв",
        engine: CorrectionEngine::Smart,
        force_replay: true,
        auto_replace: true,
        scoped_options: crate::typing_assist::ScopedTailOptions::default(),
    });

    assert_eq!(result.action, DecoderAction::ReplayAll);
}

#[test]
fn facade_exposes_physical_keyboard_mapping() {
    let events = [
        KeyEvent {
            keycode: evdev::KeyCode::KEY_L.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_T.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_K.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_F.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_Q.code(),
            shift: false,
            layout_is_ru: false,
        },
    ];

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
        8
    );
    assert!(TextBackendCapabilities::ime().can_atomic_replace());
    assert_eq!(
        TextBackendCapabilities::uinput().replace,
        TextReplaceCapability::KeyReplay
    );
}

#[test]
fn facade_exposes_replay_layout_decision() {
    let events = [
        KeyEvent {
            keycode: evdev::KeyCode::KEY_L.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_T.code(),
            shift: false,
            layout_is_ru: false,
        },
    ];

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
    let events = [
        KeyEvent {
            keycode: evdev::KeyCode::KEY_A.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_SPACE.code(),
            shift: false,
            layout_is_ru: false,
        },
        KeyEvent {
            keycode: evdev::KeyCode::KEY_B.code(),
            shift: false,
            layout_is_ru: false,
        },
    ];
    let words = split_event_words(&events).expect("words");

    assert_eq!(words.len(), 2);
    assert_eq!(tail_chars("привет", 3), "вет");
}

#[test]
fn facade_exposes_word_buffer() {
    let mut buffer = WordBuffer::new();
    buffer.push(KeyEvent {
        keycode: evdev::KeyCode::KEY_L.code(),
        shift: false,
        layout_is_ru: false,
    });

    let (events, backspaces) = buffer.what_to_replay(MAX_REPLACE_WORDS).expect("tail");

    assert_eq!(backspaces, 1);
    assert_eq!(map_original_events(&events), "l");
}
