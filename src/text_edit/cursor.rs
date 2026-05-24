use super::types::TextReplacement;

pub fn offset_replacement_plan_for_cursor(
    plan: &TextReplacement,
    cursor_offset: u32,
) -> TextReplacement {
    if cursor_offset == 0 {
        return plan.clone();
    }

    TextReplacement {
        move_left: plan.move_left.saturating_add(cursor_offset),
        backspaces: plan.backspaces,
        insert: plan.insert.clone(),
        move_right: plan.move_right.saturating_add(cursor_offset),
    }
}
