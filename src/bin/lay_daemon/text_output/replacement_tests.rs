use super::*;

fn text_replacement(
    move_left: u32,
    backspaces: u32,
    insert: impl Into<String>,
    move_right: u32,
) -> TextReplacement {
    TextReplacement {
        move_left,
        backspaces,
        insert: insert.into(),
        move_right,
    }
}

#[test]
fn minimal_current_tail_insert_keeps_layout_for_inserted_symbol() {
    let current_tail_plan = text_replacement(0, 1, "$", 0);
    assert!(!layout_after_replacement_plan(
        &current_tail_plan,
        "только $",
        false
    ));

    let middle_plan = text_replacement(7, 3, "ТУТ", 7);
    assert!(!layout_after_replacement_plan(
        &middle_plan,
        "ТУТ DOUBLE",
        true
    ));
}

#[test]
fn completed_mixed_tail_continues_in_previous_context_layout() {
    let completed_tail_plan = text_replacement(1, 6, "Wechat", 1);
    let mixed_context_layout =
        layout_after_replacement_plan(&completed_tail_plan, "текст в Wechat ", false);
    let english_context_layout =
        layout_after_replacement_plan(&completed_tail_plan, "file on off ", false);

    assert!(mixed_context_layout);
    assert!(!english_context_layout);
}

#[test]
fn middle_insert_does_not_claim_insert_layout_is_cursor_layout() {
    let middle_plan = text_replacement(7, 3, "ТУТ", 7);

    let cursor_layout = layout_after_replacement_plan(&middle_plan, "ТУТ DOUBLE", true);
    assert!(!cursor_layout);
}
