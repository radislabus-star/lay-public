use super::*;

#[test]
fn scoped_tail_flips_cyrillic_hyphen_technical_token_to_ascii() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        key_event(KeyCode::KEY_C, true),
        key_event(KeyCode::KEY_K, true),
        key_event(KeyCode::KEY_J, true),
        key_event(KeyCode::KEY_D, true),
        key_event(KeyCode::KEY_J, true),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    for event in technical_events {
        buffer.push(event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(&left_events, true);
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {typed_technical}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {target_technical}"))
    );
}

#[test]
fn scoped_tail_keeps_unknown_previous_word_and_flips_cyrillic_hyphen_technical_token() {
    let mut buffer = WordBuffer::new();
    let left_events = [
        KeyEvent {
            keycode: KeyCode::KEY_SEMICOLON.code(),
            shift: true,
            layout_is_ru: true,
        },
        key_event(KeyCode::KEY_SEMICOLON, true),
        key_event(KeyCode::KEY_SEMICOLON, true),
        key_event(KeyCode::KEY_SEMICOLON, true),
    ];
    for event in left_events {
        buffer.push(event);
    }
    buffer.handle_space();
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    for event in technical_events {
        buffer.push(event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(&left_events, true);
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {typed_technical}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {target_technical}"))
    );
}

#[test]
fn typing_assist_converts_wrong_layout_ascii_hyphen_token() {
    let technical_events = [
        key_event(KeyCode::KEY_W, true),
        key_event(KeyCode::KEY_I, true),
        key_event(KeyCode::KEY_MINUS, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_I, true),
    ];
    let typed_technical = map_events_to_layout(&technical_events, true);
    let target_technical = map_events_to_layout(&technical_events, false);
    assert_eq!(
        apply_typing_assist_exact(&format!("{typed_technical} ")),
        Some(format!("{target_technical} "))
    );
}

#[test]
fn typing_assist_keeps_natural_cyrillic_hyphen_words() {
    assert_eq!(apply_typing_assist("что-то ", true), None);
    assert_eq!(apply_typing_assist("кто-то ", true), None);
    assert_eq!(apply_typing_assist("где-то ", true), None);
    assert_eq!(apply_typing_assist("как-то ", true), None);
    assert_eq!(apply_typing_assist("из-за ", true), None);
    assert_eq!(apply_typing_assist("кока-коле ", true), None);
    assert_eq!(apply_typing_assist("код-дэ-вуар ", true), None);
    assert_eq!(apply_typing_assist("чек-лист! ", true), None);
    assert_eq!(apply_typing_assist("к-лист! ", true), None);
    assert_eq!(correct_wrong_layout_ascii_technical_token("из-за"), None);
    assert_eq!(
        correct_wrong_layout_ascii_technical_token("цш-аш"),
        Some("wi-fi".to_string())
    );
    assert_eq!(correct_wrong_layout_ascii_technical_token("15р-16р"), None);
    assert_eq!(apply_typing_assist("15р-16р ", true), None);
}

#[test]
fn plain_cyrillic_scope_word_does_not_become_ascii_technical_noise() {
    let events = [
        key_event(KeyCode::KEY_A, true),
        key_event(KeyCode::KEY_Q, true),
        key_event(KeyCode::KEY_DOT, true),
        key_event(KeyCode::KEY_Z, true),
    ];
    let original = map_events_to_layout(&events, true);
    let converted = map_events_to_layout(&events, false);

    assert!(original.chars().all(is_cyrillic_letter));
    assert!(is_ascii_technical_token(&converted));
    assert!(should_keep_plain_cyrillic_before_ascii_technical(
        &original, &converted
    ));
    assert_eq!(decide_completed_scope_word(&events), original);
}

#[test]
fn smart_scoped_tail_handles_large_mixed_language_pair_matrix() {
    let english_left = [
        "good", "test", "word", "live", "double", "text", "mode", "file", "code", "data",
    ];
    let russian_left = [
        "привет",
        "текст",
        "слово",
        "тест",
        "проверка",
        "можно",
        "нужно",
        "дальше",
        "хорошо",
        "пример",
    ];
    let russian_targets = [
        "привет",
        "текст",
        "слово",
        "тест",
        "проверка",
        "можно",
        "нужно",
        "дальше",
        "хорошо",
        "пример",
    ];
    let english_targets = [
        "good", "test", "word", "live", "double", "text", "mode", "file", "code", "data",
    ];

    let mut cases = 0;
    for left in english_left {
        for target in russian_targets {
            let typed = lay::dict::convert(target, lay::dict::Direction::Ru2Us);
            assert_smart_pair(left, false, &typed, false, &format!("{left} {target}"));
            cases += 1;
        }
    }

    for left in russian_left {
        for target in english_targets {
            let typed = lay::dict::convert(target, lay::dict::Direction::Us2Ru);
            assert_smart_pair(left, true, &typed, true, &format!("{left} {target}"));
            cases += 1;
        }
    }

    assert!(cases >= 100, "expected at least 100 mixed pair cases");
}

#[test]
fn scoped_tail_flips_current_visual_latin_word_with_cyrillic_c_homoglyph() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_C, false),
            (KeyCode::KEY_H, false),
            (KeyCode::KEY_E, false),
            (KeyCode::KEY_C, false),
            (KeyCode::KEY_K, false),
        ],
        false,
    );
    buffer.handle_space();
    buffer.push(key_event(KeyCode::KEY_C, true));
    buffer.push(key_event(KeyCode::KEY_H, false));
    buffer.push(key_event(KeyCode::KEY_E, false));
    buffer.push(key_event(KeyCode::KEY_C, false));
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "check сhec");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("check срус".to_string())
    );
}

#[test]
fn scoped_tail_removes_duplicate_layout_prefix_from_completed_ascii_technical_token() {
    let mut buffer = WordBuffer::new();
    let mut completed_events = vec![key_event(KeyCode::KEY_W, true)];
    completed_events.extend(key_events(&ascii_hyphen_token_keycodes(), false));
    for event in &completed_events {
        buffer.push(*event);
    }
    buffer.handle_space();
    let current_events = key_events(&[KeyCode::KEY_G, KeyCode::KEY_H, KeyCode::KEY_J], false);
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let completed_original = map_original_events(&completed_events);
    let current_original = map_original_events(&current_events);
    let completed_repaired = correct_duplicate_layout_prefix_on_ascii_token(&completed_original)
        .expect("duplicate prefix repair");
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{completed_original} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{completed_repaired} {current_target}"))
    );
}

#[test]
fn scoped_tail_keeps_ascii_hyphen_word_and_flips_current_short_tail() {
    let mut buffer = WordBuffer::new();
    let completed_events = key_events(&ascii_hyphen_token_keycodes(), false);
    for event in &completed_events {
        buffer.push(*event);
    }
    buffer.handle_space();
    let current_events = key_events(&[KeyCode::KEY_Y, KeyCode::KEY_E], false);
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let completed_original = map_original_events(&completed_events);
    let current_original = map_original_events(&current_events);
    let current_target = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{completed_original} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{completed_original} {current_target}"))
    );
    assert_eq!(
        plan_text_replacement(
            &format!("{completed_original} {current_original}"),
            &format!("{completed_original} {current_target}")
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
fn trailing_space_scope_keeps_ascii_hyphen_word_and_flips_last_short_word() {
    let mut buffer = WordBuffer::new();
    let completed_events = key_events(&ascii_hyphen_token_keycodes(), false);
    for event in &completed_events {
        buffer.push(*event);
    }
    buffer.handle_space();
    let current_events = key_events(&[KeyCode::KEY_Y, KeyCode::KEY_E], false);
    for event in &current_events {
        buffer.push(*event);
    }
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 2, CorrectionEngine::Smart, true);
    let (events, backspaces) = buffer.what_to_replay(scope).expect("last word tail");
    let left = map_original_events(&completed_events);
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
fn scoped_tail_collapses_cyrillic_prefix_before_ascii_hyphen_tail() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_C,
            KeyCode::KEY_K,
            KeyCode::KEY_J,
            KeyCode::KEY_D,
            KeyCode::KEY_J,
        ],
        true,
    );
    buffer.handle_space();
    let mut current_events = vec![key_event(KeyCode::KEY_G, true)];
    current_events.extend(key_events(
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_F,
            KeyCode::KEY_H,
            KeyCode::KEY_F,
            KeyCode::KEY_MINUS,
            KeyCode::KEY_G,
            KeyCode::KEY_F,
            KeyCode::KEY_H,
            KeyCode::KEY_F,
        ],
        false,
    ));
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(
        &[
            key_event(KeyCode::KEY_C, true),
            key_event(KeyCode::KEY_K, true),
            key_event(KeyCode::KEY_J, true),
            key_event(KeyCode::KEY_D, true),
            key_event(KeyCode::KEY_J, true),
        ],
        true,
    );
    let current_original = map_original_events(&current_events);
    let current_target =
        repair_cyrillic_prefix_before_ascii_tail(&current_events).expect("prefix collapse repair");

    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original}")
    );
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_target}"))
    );
}

#[test]
fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_word_and_keeps_undo() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "Иракскую", true);
    buffer.handle_space();
    buffer.push(text_key_event('к', true));
    for ch in "jrf-rjke".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, "Иракскую кjrf-rjke");
    assert_eq!(replacement, "Иракскую кока-колу");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: 8,
            insert: "ока-колу".to_string(),
            move_right: 0,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), "кока-колу");
    assert_eq!(undo_backspaces, 9);
    assert!(!undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, false), "rjrf-rjke");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn scoped_tail_repairs_mixed_cyrillic_prefix_ascii_hyphen_dative_word() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "Иракскую", true);
    buffer.handle_space();
    buffer.push(text_key_event('к', true));
    for ch in "jrf-rjkt".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, "Иракскую кjrf-rjkt");
    assert_eq!(replacement, "Иракскую кока-коле");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: 8,
            insert: "ока-коле".to_string(),
            move_right: 0,
        }
    );
    assert!(buffer.remember_replacement_last_word_for_replay(&events, &plan, &replacement));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), "кока-коле");
    assert_eq!(undo_backspaces, 9);
    assert!(!undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, false), "rjrf-rjkt");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn replacement_last_word_memory_ignores_middle_insert_plan() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "AmoCRM", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "Z", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "тут", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "задача", true);

    let (events, _) = buffer.what_to_replay(4).expect("four-word tail");
    let plan = plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача")
        .expect("middle replacement plan");

    assert_eq!(plan.move_right, 11);
    assert!(!buffer.remember_replacement_last_word_for_replay(
        &events,
        &plan,
        "AmoCRM Я тут задача"
    ));
}
