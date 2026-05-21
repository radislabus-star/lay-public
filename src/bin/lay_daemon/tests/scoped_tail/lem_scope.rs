use super::*;

#[test]
fn scoped_tail_does_not_turn_valid_ascii_hyphen_tail_into_bad_russian() {
    let mut buffer = WordBuffer::new();
    push_keys(&mut buffer, &[KeyCode::KEY_D], true);
    buffer.handle_space();
    let mut current_events = vec![key_event(KeyCode::KEY_W, true)];
    current_events.extend([
        KeyEvent {
            keycode: KeyCode::KEY_W.code(),
            shift: true,
            layout_is_ru: false,
        },
        key_event(KeyCode::KEY_I, false),
        key_event(KeyCode::KEY_MINUS, false),
        key_event(KeyCode::KEY_F, false),
        key_event(KeyCode::KEY_I, false),
    ]);
    for event in &current_events {
        buffer.push(*event);
    }
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let left = map_events_to_layout(&[key_event(KeyCode::KEY_D, true)], true);
    let current_original = map_original_events(&current_events);
    let current_wrong_layout = map_events_to_layout(&current_events, true);

    assert_eq!(
        map_original_events(&events),
        format!("{left} {current_original}")
    );
    assert_ne!(
        decide_scoped_tail_correction(&events),
        Some(format!("{left} {current_wrong_layout}"))
    );
}

#[test]
fn scoped_tail_converts_confident_bad_previous_word() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_H,
            KeyCode::KEY_B,
            KeyCode::KEY_D,
            KeyCode::KEY_T,
            KeyCode::KEY_N,
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_V, KeyCode::KEY_B, KeyCode::KEY_H],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("привет мир".to_string())
    );
}

#[test]
fn scoped_tail_keeps_unknown_previous_word() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_F,
            KeyCode::KEY_O,
            KeyCode::KEY_O,
            KeyCode::KEY_B,
            KeyCode::KEY_A,
            KeyCode::KEY_R,
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_G, KeyCode::KEY_H, KeyCode::KEY_J],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("foobar про".to_string())
    );
}

#[test]
fn scoped_tail_generalizes_to_more_than_two_words() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_H,
            KeyCode::KEY_J,
            KeyCode::KEY_D,
            KeyCode::KEY_T,
            KeyCode::KEY_H,
            KeyCode::KEY_R,
            KeyCode::KEY_F,
        ],
        true,
    );
    buffer.handle_space();
    push_keys(&mut buffer, &[KeyCode::KEY_D], true);
    buffer.handle_space();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_D, true),
            (KeyCode::KEY_O, false),
            (KeyCode::KEY_U, false),
            (KeyCode::KEY_B, false),
            (KeyCode::KEY_L, false),
            (KeyCode::KEY_E, false),
        ],
        true,
    );
    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");

    assert_eq!(map_original_events(&events), "проверка в Вщгиду");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("проверка в Double".to_string())
    );
}

#[test]
fn scoped_tail_uses_lem_for_three_word_mixed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);
    buffer.handle_space();
    for ch in "ghbdtn".chars() {
        buffer.push(text_key_event(ch, false));
    }
    buffer.handle_space();
    for ch in "ntrcn".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    assert_eq!(map_original_events(&events), "good ghbdtn ntrcn");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("good привет текст".to_string())
    );
}

#[test]
fn scoped_tail_keeps_two_russian_words_and_flips_current_english_layout_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "как", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "котовые", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ашду", true);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "как котовые ашду");
    assert_eq!(replacement, "как котовые file");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 4,
            insert: "file".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_keeps_short_repeated_completed_word_and_flips_current_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "аа", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "слово", true);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "вот", true);

    let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
    let original = map_original_events(&events);
    let replacement =
        decide_scoped_tail_correction_with_lem(&events, true).expect("smart replacement");

    assert_eq!(original, "аа слово вот");
    assert_eq!(replacement, "аа слово djn");
    assert_eq!(
        plan_text_replacement(&original, &replacement),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 3,
            insert: "djn".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn scoped_tail_uses_lem_for_two_word_mixed_tail() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);
    buffer.handle_space();
    for ch in "ntrcn".chars() {
        buffer.push(text_key_event(ch, false));
    }

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let words = split_event_words(&events).expect("split words");
    let ranked = lay::lem::rank_candidates(
        &map_original_events(&events),
        scoped_tail_lem_candidates(&words, true, true),
    );

    assert_eq!(map_original_events(&events), "good ntrcn");
    assert_eq!(ranked[0].text, "good текст");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("good текст".to_string())
    );
}

#[test]
fn scoped_tail_keeps_good_russian_context_and_flips_current_acronym() {
    let cases = [
        ("ВСЁ", "ДЕЛАЙ", "ЛВУ", "KDE"),
        ("НУЖНО", "ДЕЛАТЬ", "ТЕАЫ", "NTFS"),
        ("ПРОСТО", "ДЕЛАЙ", "СЗГ", "CPU"),
    ];

    for (left1, left2, typed_tail, expected_tail) in cases {
        let mut buffer = WordBuffer::new();
        push_text_as_layout(&mut buffer, left1, true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, left2, true);
        buffer.handle_space();
        push_text_as_layout(&mut buffer, typed_tail, true);

        let (events, _) = buffer.what_to_replay(3).expect("three-word tail");
        let original = map_original_events(&events);
        let expected = format!("{left1} {left2} {expected_tail}");

        assert_eq!(
            decide_scoped_tail_correction_with_lem(&events, true),
            Some(expected.clone()),
            "original={original:?}"
        );
        assert_eq!(
            decide_scoped_tail_correction(&events),
            Some(expected.clone()),
            "original={original:?}"
        );
        assert_eq!(
            plan_text_replacement(&original, &expected),
            Some(TextReplacement {
                move_left: 0,
                backspaces: typed_tail.chars().count() as u32,
                insert: expected_tail.to_string(),
                move_right: 0,
            })
        );
    }
}

#[test]
fn scoped_tail_converts_apostrophe_layout_word_as_letter() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "'nj", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ckjdj", false);

    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);

    assert_eq!(original, "'nj ckjdj");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("это слово".to_string())
    );
}

#[test]
fn scoped_tail_handles_three_completed_words_with_typo() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "ljgecntv", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, ",ele", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ошибатся", true);
    buffer.handle_space();

    let scope = effective_replace_words(&buffer, 3, CorrectionEngine::Smart, true);
    let (events, _) = buffer.what_to_replay(scope).expect("three-word tail");

    assert_eq!(scope, 3);
    assert_eq!(map_original_events(&events), "ljgecntv ,ele ошибатся ");
    assert_eq!(
        decide_scoped_tail_correction_with_lem(&events, true),
        Some("допустем буду ошибаться ".to_string())
    );
}

#[test]
fn scoped_tail_keeps_live_and_flips_russian_current_tail() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_L, true),
            (KeyCode::KEY_I, false),
            (KeyCode::KEY_V, false),
            (KeyCode::KEY_E, false),
        ],
        false,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_L,
            KeyCode::KEY_B,
            KeyCode::KEY_C,
            KeyCode::KEY_N,
            KeyCode::KEY_H,
            KeyCode::KEY_B,
        ],
        false,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "Live lbcnhb");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("Live дистри".to_string())
    );
}

#[test]
fn scoped_tail_normalizes_mixed_current_word_to_last_layout() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_L, true),
            (KeyCode::KEY_I, false),
            (KeyCode::KEY_V, false),
            (KeyCode::KEY_E, false),
        ],
        false,
    );
    buffer.handle_space();
    push_keys(&mut buffer, &[KeyCode::KEY_L], false);
    push_keys(
        &mut buffer,
        &[KeyCode::KEY_L, KeyCode::KEY_B, KeyCode::KEY_C],
        true,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "Live lдис");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("Live дис".to_string())
    );
}

#[test]
fn scoped_tail_repairs_mixed_previous_ru_word_and_flips_current_tail() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_G, true),
            (KeyCode::KEY_H, true),
            (KeyCode::KEY_J, true),
            (KeyCode::KEY_D, true),
        ],
        true,
    );
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_T, true),
            (KeyCode::KEY_H, true),
            (KeyCode::KEY_M, true),
        ],
        false,
    );
    buffer.handle_space();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_W, true),
            (KeyCode::KEY_O, true),
            (KeyCode::KEY_R, true),
            (KeyCode::KEY_D, true),
        ],
        true,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "ПРОВTHM ЦЩКВ");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("ПРОВЕРЬ WORD".to_string())
    );
}
