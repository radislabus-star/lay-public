use super::*;

#[test]
fn scoped_tail_keeps_good_english_previous_word_and_flips_current_layout_word() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_O,
            KeyCode::KEY_O,
            KeyCode::KEY_D,
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_N,
            KeyCode::KEY_T,
            KeyCode::KEY_R,
            KeyCode::KEY_C,
            KeyCode::KEY_N,
        ],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "good ntrcn");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("good текст".to_string())
    );
}

#[test]
fn scoped_tail_keeps_good_russian_previous_word_and_flips_current_currency_symbol() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "только", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, ";", true);
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let target_is_ru = replay_layout_decision(&events).target_is_ru;
    let replay_target = map_events_to_layout(&events, target_is_ru);
    let decoded = decode_manual_tail(ManualDecodeRequest {
        events: &events,
        original: &original,
        converted: &replay_target,
        engine: CorrectionEngine::Smart,
        force_replay: false,
        auto_replace: true,
        scoped_options: ScopedTailOptions {
            lem_enabled: true,
            allow_layout_auto: true,
        },
    });

    assert_eq!(original, "только ;");
    assert_eq!(replay_target, "njkmrj $");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("только $".to_string())
    );
    assert_eq!(
        decoded.action,
        DecoderAction::ReplaceText {
            replacement: "только $".to_string(),
            source: CorrectionSource::SmartText,
        }
    );
    assert_eq!(
        decoded.edit.map(|edit| edit.plan),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 1,
            insert: "$".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_keeps_completed_ascii_title_word_and_flips_current_latin_keys() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        KeyEvent {
            keycode: KeyCode::KEY_D.code(),
            shift: true,
            layout_is_ru: false,
        },
        key_event(KeyCode::KEY_O, false),
        key_event(KeyCode::KEY_U, false),
        key_event(KeyCode::KEY_B, false),
        key_event(KeyCode::KEY_L, false),
        key_event(KeyCode::KEY_E, false),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let current_events = [
        key_event(KeyCode::KEY_N, false),
        key_event(KeyCode::KEY_J, false),
        key_event(KeyCode::KEY_SEMICOLON, false),
        key_event(KeyCode::KEY_T, false),
    ];
    for event in current_events {
        buffer.push(event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_original_events(&left_events);
    let current_original = map_original_events(&current_events);
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_target}"))
    );
    assert_eq!(
        plan_text_replacement(
            &format!("{left} {current_original}"),
            &format!("{left} {current_target}")
        ),
        Some(TextReplacement {
            move_left: 0,
            backspaces: current_original.chars().count() as u32,
            insert: current_target,
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_keeps_completed_russian_y_word_and_flips_current_latin_brand() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "протокол", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "испытаний", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Сщсф", true);
    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, "протокол испытаний Сщсф");
    assert_eq!(replacement, "протокол испытаний Coca");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 4,
            insert: "Coca".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_repairs_stale_layout_flag_inside_completed_russian_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "протокол", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "испытани", true);
    buffer.push(KeyEvent {
        keycode: KeyCode::KEY_Q.code(),
        shift: false,
        layout_is_ru: false,
    });
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Сщсф", true);
    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");

    assert_eq!(original, "протокол испытаниq Сщсф");
    assert_eq!(replacement, "протокол испытаний Coca");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 6,
            insert: "й Coca".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_keeps_single_completed_cyrillic_fragment_before_current_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "й", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Сщсф", true);
    let (events, _) = buffer.what_to_replay(3).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction_with_lem(&events, true)
        .or_else(|| decide_scoped_tail_correction(&events))
        .expect("smart replacement");

    assert_eq!(original, "й Сщсф");
    assert_eq!(replacement, "й Coca");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 4,
            insert: "Coca".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_trailing_space_keeps_previous_good_word_and_flips_current_completed_latin_keys() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        KeyEvent {
            keycode: KeyCode::KEY_D.code(),
            shift: true,
            layout_is_ru: false,
        },
        key_event(KeyCode::KEY_O, false),
        key_event(KeyCode::KEY_U, false),
        key_event(KeyCode::KEY_B, false),
        key_event(KeyCode::KEY_L, false),
        key_event(KeyCode::KEY_E, false),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let current_events = [
        key_event(KeyCode::KEY_N, false),
        key_event(KeyCode::KEY_J, false),
        key_event(KeyCode::KEY_SEMICOLON, false),
        key_event(KeyCode::KEY_T, false),
    ];
    for event in current_events {
        buffer.push(event);
    }
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");
    let left = map_original_events(&left_events);
    let current_original = map_original_events(&current_events);
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(scope, 2);
    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original} ")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_target} "))
    );
    assert_eq!(
        backspaces,
        (left.chars().count() + 1 + current_original.chars().count() + 1) as u32
    );
}

#[test]
fn scoped_tail_trailing_space_keeps_previous_russian_word_and_flips_completed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "открывал", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "цзы", true);
    buffer.handle_space();

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "открывал цзы ");
    assert_eq!(replacement, "открывал wps ");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 1,
            backspaces: 3,
            insert: "wps".to_string(),
            move_right: 1,
        })
    );
}
