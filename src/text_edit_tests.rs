use super::*;

fn apply_plan(original: &str, plan: &TextReplacement) -> String {
    let mut chars: Vec<char> = original.chars().collect();
    let mut cursor = chars.len().saturating_sub(plan.move_left as usize);
    let delete_start = cursor.saturating_sub(plan.backspaces as usize);
    chars.splice(delete_start..cursor, plan.insert.chars());
    cursor = delete_start + plan.insert.chars().count();
    cursor = (cursor + plan.move_right as usize).min(chars.len());
    chars[..cursor]
        .iter()
        .chain(chars[cursor..].iter())
        .collect()
}

#[test]
fn plans_minimal_two_word_prefix_and_suffix_edits() {
    assert_eq!(
        plan_text_replacement("NEN DOUBLE", "ТУТ DOUBLE"),
        Some(TextReplacement {
            move_left: 7,
            backspaces: 3,
            insert: "ТУТ".to_string(),
            move_right: 7,
        })
    );
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
fn committed_tail_plan_preserves_typed_trailing_space_boundary() {
    assert_eq!(
        plan_committed_tail_replacement("double b ", "double и "),
        Some(TextReplacement {
            move_left: 1,
            backspaces: 8,
            insert: "double и".to_string(),
            move_right: 1,
        })
    );
    assert_eq!(
        plan_committed_tail_replacement("чтобы точнр ", "чтобы точно "),
        Some(TextReplacement {
            move_left: 1,
            backspaces: 11,
            insert: "чтобы точно".to_string(),
            move_right: 1,
        })
    );
    assert_eq!(
        plan_committed_tail_replacement("ОФФИЦИАЛЬНОМ ", "ОФИЦИАЛЬНОМ "),
        Some(TextReplacement {
            move_left: 1,
            backspaces: 12,
            insert: "ОФИЦИАЛЬНОМ".to_string(),
            move_right: 1,
        })
    );
}

#[test]
fn committed_tail_long_word_replaces_whole_body_before_space() {
    assert_eq!(
        plan_committed_tail_replacement("переиспользоватся ", "переиспользоваться "),
        Some(TextReplacement {
            move_left: 1,
            backspaces: 17,
            insert: "переиспользоваться".to_string(),
            move_right: 1,
        })
    );
}

#[test]
fn committed_tail_sentence_plans_keep_space_with_mixed_language_text() {
    for (original, replacement) in [
        ("пишу README и double b ", "пишу README и double и "),
        ("дальше буду точнр ", "дальше буду точно "),
        ("API работает нормальнр ", "API работает нормально "),
    ] {
        let plan = plan_committed_tail_replacement(original, replacement).expect("replacement");
        assert_eq!(apply_plan(original, &plan), replacement);
        assert_eq!(original.ends_with(' '), replacement.ends_with(' '));
        assert_eq!(plan.move_right, 1, "space boundary must be preserved");
        assert!(
            !plan.insert.ends_with(' '),
            "space boundary must stay in the field, not be reinserted"
        );
    }
}

#[test]
fn committed_tail_split_word_plan_inserts_internal_space() {
    let plan = plan_committed_tail_replacement("чтобыточно ", "чтобы точно ").expect("replacement");

    assert_eq!(
        plan,
        TextReplacement {
            move_left: 6,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 6,
        }
    );
    assert_eq!(apply_plan("чтобыточно ", &plan), "чтобы точно ");

    let plan = plan_committed_tail_replacement("тожесамое ", "тоже самое ").expect("replacement");
    assert_eq!(
        plan,
        TextReplacement {
            move_left: 6,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 6,
        }
    );
    assert_eq!(apply_plan("тожесамое ", &plan), "тоже самое ");
}

#[test]
fn committed_tail_split_word_plan_can_fix_small_typo_near_split() {
    let plan = plan_committed_tail_replacement("тоесамое ", "тоже самое ").expect("replacement");

    assert_eq!(
        plan,
        TextReplacement {
            move_left: 6,
            backspaces: 1,
            insert: "же ".to_string(),
            move_right: 6,
        }
    );
    assert_eq!(apply_plan("тоесамое ", &plan), "тоже самое ");
}

#[test]
fn committed_tail_non_split_replacement_keeps_existing_space_boundary() {
    let plan = plan_committed_tail_replacement("тожесамое ", "ТОЖЕСАМОЕ ").expect("replacement");

    assert_eq!(
        plan,
        TextReplacement {
            move_left: 1,
            backspaces: 9,
            insert: "ТОЖЕСАМОЕ".to_string(),
            move_right: 1,
        }
    );
    assert_eq!(apply_plan("тожесамое ", &plan), "ТОЖЕСАМОЕ ");
}

#[test]
fn committed_tail_plan_can_be_shifted_behind_current_word() {
    let base = plan_committed_tail_replacement("тожесамое ", "тоже самое ").expect("replacement");
    let shifted = offset_replacement_plan_for_cursor(&base, 6);

    assert_eq!(
        shifted,
        TextReplacement {
            move_left: 12,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 12,
        }
    );
    assert_eq!(
        apply_plan("тожесамое склено", &shifted),
        "тоже самое склено"
    );
}

#[test]
fn committed_tail_space_insertions_handle_many_glued_words() {
    let plans = plan_committed_whitespace_insertions(
        "янебудузавастожесамое ",
        "я не буду за вас тоже самое ",
        0,
    )
    .expect("space insertion plans");

    let mut text = "янебудузавастожесамое ".to_string();
    for plan in &plans {
        text = apply_plan(&text, plan);
    }
    assert_eq!(text, "я не буду за вас тоже самое ");
    assert_eq!(plans.len(), 6);
    assert!(plans.iter().all(|plan| plan.backspaces == 0));
}

#[test]
fn committed_tail_space_insertions_can_be_shifted_behind_current_word() {
    let plans = plan_committed_whitespace_insertions("тожесамое ", "тоже самое ", 6)
        .expect("space insertion plans");

    assert_eq!(
        plans,
        vec![TextReplacement {
            move_left: 12,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 12,
        }]
    );
    let mut text = "тожесамое склено".to_string();
    for plan in &plans {
        text = apply_plan(&text, plan);
    }
    assert_eq!(text, "тоже самое склено");
}

#[test]
fn committed_tail_space_insertions_reject_real_letter_changes() {
    assert_eq!(
        plan_committed_whitespace_insertions("тоесамое ", "тоже самое ", 0),
        None
    );
}

#[test]
fn committed_tail_spacing_is_restored_before_planning() {
    assert_eq!(
        ensure_committed_tail_spacing("double b ", "double и".to_string()),
        "double и "
    );
    assert_eq!(
        ensure_committed_tail_spacing("plain", "plain".to_string()),
        "plain"
    );
}

#[test]
fn tail_chars_returns_unicode_tail() {
    assert_eq!(tail_chars("привет", 3), "вет");
    assert_eq!(tail_chars("hi", 10), "hi");
}
