use super::*;

#[test]
fn replacement_plan_keeps_good_suffix_in_place() {
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
fn replacement_plan_keeps_good_prefix_in_place() {
    assert_eq!(
        plan_text_replacement("Главное Вщгиду", "Главное Double"),
        Some(TextReplacement {
            move_left: 0,
            backspaces: 6,
            insert: "Double".to_string(),
            move_right: 0,
        })
    );
}

#[test]
fn replacement_plan_replaces_single_bad_middle_token() {
    assert_eq!(
        plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача"),
        Some(TextReplacement {
            move_left: 11,
            backspaces: 1,
            insert: "Я".to_string(),
            move_right: 11,
        })
    );
}

#[test]
fn replacement_plan_deletes_duplicate_prefix_before_kept_suffix() {
    assert_eq!(
        plan_text_replacement("на ппредмет", "на предмет"),
        Some(TextReplacement {
            move_left: 6,
            backspaces: 1,
            insert: String::new(),
            move_right: 6,
        })
    );
}

#[test]
fn pending_auto_undo_restores_full_original_text() {
    let mut buffer = WordBuffer::new();
    buffer.remember_pending_auto_undo("typing-assist", "15р-16р ", "15h-16h ", 1, 1);

    let undo = buffer.take_pending_auto_undo().expect("pending undo");
    assert_eq!(
        pending_auto_undo_plan(&undo),
        TextReplacement {
            move_left: 0,
            backspaces: 8,
            insert: "15р-16р ".to_string(),
            move_right: 0,
        }
    );
}

#[test]
fn opposite_events_flip_each_key_own_layout_for_smart_mixed_tail() {
    let events = [
        key_event(KeyCode::KEY_H, true),
        key_event(KeyCode::KEY_R, true),
        key_event(KeyCode::KEY_F, true),
        key_event(KeyCode::KEY_SPACE, false),
        key_event(KeyCode::KEY_G, false),
        key_event(KeyCode::KEY_H, false),
        key_event(KeyCode::KEY_J, false),
    ];

    assert_eq!(map_original_events(&events), "рка ghj");
    assert_eq!(map_opposite_events(&events), "hrf про");
}

#[test]
fn smart_insert_layout_follows_result_text_tail() {
    assert!(preferred_layout_for_text("рка про", false));
    assert!(!preferred_layout_for_text("Главное Double", true));
    assert!(preferred_layout_for_text("AmoCRM Я тут задача", false));
}

#[test]
fn minimal_current_tail_insert_keeps_layout_for_inserted_symbol() {
    let current_tail_plan = TextReplacement {
        move_left: 0,
        backspaces: 1,
        insert: "$".to_string(),
        move_right: 0,
    };
    assert!(!layout_after_replacement_plan(
        &current_tail_plan,
        "только $",
        false
    ));

    let middle_plan = TextReplacement {
        move_left: 7,
        backspaces: 3,
        insert: "ТУТ".to_string(),
        move_right: 7,
    };
    assert!(!layout_after_replacement_plan(
        &middle_plan,
        "ТУТ DOUBLE",
        true
    ));
}

#[test]
fn target_layout_matches_cache_contract() {
    assert_eq!(target_layout(true), ("ru", "xkb:ru::rus"));
    assert_eq!(target_layout(false), ("us", "xkb:us::eng"));
}
