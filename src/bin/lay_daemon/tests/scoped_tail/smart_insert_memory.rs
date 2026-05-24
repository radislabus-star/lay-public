use super::*;

#[test]
fn smart_decision_keeps_good_word_and_converts_bad_neighbor() {
    assert_eq!(
        decide_correction("Главное Вщгиду", "Ukfdyjt Double", CorrectionEngine::Smart),
        Correction::InsertText("Главное Double".to_string())
    );
}

use crate::correction_memory_runtime::{
    remember_manual_text_correction, ManualTextCorrectionMemory,
};

#[test]
fn scoped_tail_keeps_good_previous_word_and_flips_current_fragment() {
    let mut buffer = WordBuffer::new();
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
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");

    assert_eq!(map_original_events(&events), "в Вщгиду");
    assert_eq!(
        decide_scoped_tail_correction(&events),
        Some("в Double".to_string())
    );
    assert_eq!(
        plan_text_replacement("в Вщгиду", "в Double"),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 6,
            insert: "Double".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn smart_insert_remembers_only_inserted_tail_for_immediate_undo() {
    let mut buffer = WordBuffer::new();
    push_key_events(
        &mut buffer,
        &[
            (KeyCode::KEY_G, true),
            (KeyCode::KEY_H, false),
            (KeyCode::KEY_J, false),
            (KeyCode::KEY_D, false),
            (KeyCode::KEY_T, false),
            (KeyCode::KEY_H, false),
            (KeyCode::KEY_R, false),
            (KeyCode::KEY_F, false),
        ],
        true,
    );
    buffer.handle_space();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_C,
            KeyCode::KEY_K,
            KeyCode::KEY_J,
            KeyCode::KEY_D,
            KeyCode::KEY_F,
        ],
        true,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, "Проверка слова");
    assert_eq!(replacement, "Проверка ckjdf");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: 5,
            insert: "ckjdf".to_string(),
            move_right: 0,
        }
    );
    assert!(buffer.remember_inserted_tail_for_replay(&events, &plan, false));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), "ckjdf");
    assert_eq!(undo_backspaces, 5);
    assert!(undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, true), "слова");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn smart_insert_remembers_last_word_after_full_tail_replace() {
    let mut buffer = WordBuffer::new();
    push_keys(
        &mut buffer,
        &[
            KeyCode::KEY_G,
            KeyCode::KEY_O,
            KeyCode::KEY_O,
            KeyCode::KEY_D,
        ],
        true,
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
        true,
    );
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = decide_scoped_tail_correction(&events).expect("smart replacement");
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    assert_eq!(original, "пщщв текст");
    assert_eq!(replacement, "good ntrcn");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: 10,
            insert: "good ntrcn".to_string(),
            move_right: 0,
        }
    );
    assert!(!buffer.remember_inserted_tail_for_replay(&events, &plan, false));
    assert!(buffer.remember_inserted_last_word_for_replay(&events, &plan));

    let (undo_events, undo_backspaces) = buffer.what_to_replay(2).expect("undo tail");
    let undo_decision = replay_layout_decision(&undo_events);
    assert_eq!(map_original_events(&undo_events), "ntrcn");
    assert_eq!(undo_backspaces, 5);
    assert!(undo_decision.target_is_ru);
    assert_eq!(map_events_to_layout(&undo_events, true), "текст");
    assert!(buffer.replay_toggle_ready());
}

#[test]
fn manual_text_correction_keeps_pending_full_undo() {
    let mut buffer = WordBuffer::new();
    push_text_as_layout(&mut buffer, "good", false);
    buffer.handle_space();
    push_text_as_layout(&mut buffer, "ntrcn", false);
    let (events, _) = buffer.what_to_replay(2).expect("two-word tail");
    let original = map_original_events(&events);
    let replacement = "good текст".to_string();
    let plan = plan_text_replacement(&original, &replacement).expect("minimal plan");

    remember_manual_text_correction(
        &mut buffer,
        ManualTextCorrectionMemory {
            events: &events,
            plan: &plan,
            original: &original,
            replacement: &replacement,
            kind: "smart-text",
            replace_words: 2,
            words: 2,
            inserted_layout_is_ru: Some(true),
        },
    );

    let undo = buffer.take_pending_auto_undo().expect("pending undo");
    assert_eq!(undo.original, "good ntrcn");
    assert_eq!(undo.replacement, "good текст");
    assert_eq!(
        pending_auto_undo_plan(&undo),
        TextReplacement {
            move_left: 0,
            backspaces: 10,
            insert: "good ntrcn".to_string(),
            move_right: 0,
        }
    );
}
