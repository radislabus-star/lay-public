use super::*;
use crate::typing_assist_test_fixtures::{first_fixture_row, fixture_rows, text_replacement};

fn apply_plan(original: &str, plan: &TextReplacement) -> String {
    apply_replacement_plan_to_text(original, plan)
}

fn assert_autocorrect_sequence(name: &str) {
    let rows = fixture_rows(name);
    let header = rows.first().expect("sequence header fixture");
    assert!(
        header.len() >= 4,
        "sequence header must have at least 4 fields"
    );

    let mut text = header[0].clone();
    for row in rows.iter().skip(1) {
        assert_eq!(row.len(), 2, "sequence step must be original/replacement");
        text.push_str(&row[0]);
        let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("correction");
        text = apply_plan(&text, &plan);
    }
    if let Some(append) = header.get(4) {
        text.push_str(append);
    }

    assert_eq!(text, header[1]);
    assert_eq!(
        text.matches(' ').count(),
        header[2].parse::<usize>().expect("spaces")
    );
    assert_eq!(
        text.split_whitespace().count(),
        header[3].parse::<usize>().expect("words")
    );
}

fn autocorrect_safety_fixture(case_name: &str) -> Vec<String> {
    let mut row = fixture_rows("text_edit_autocorrect_safety.tsv")
        .into_iter()
        .find(|row| row.first().is_some_and(|name| name == case_name))
        .unwrap_or_else(|| panic!("missing safety fixture case: {case_name}"));
    row.resize(9, String::new());
    row
}

#[test]
fn plans_minimal_two_word_prefix_and_suffix_edits() {
    for row in fixture_rows("text_edit_minimal_plan.tsv") {
        assert_eq!(row.len(), 6, "minimal-plan fixture must be TSV");
        assert_eq!(
            plan_text_replacement(&row[0], &row[1]),
            Some(text_replacement(
                row[2].parse().expect("move_left"),
                row[3].parse().expect("backspaces"),
                &row[4],
                row[5].parse().expect("move_right"),
            ))
        );
    }
}

#[test]
fn committed_tail_plan_preserves_typed_trailing_space_boundary() {
    for row in fixture_rows("text_edit_committed_tail_boundary.tsv") {
        assert_eq!(row.len(), 2, "tail-boundary fixture must be TSV");
        let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
        assert_eq!(apply_plan(&row[0], &plan), row[1]);
        assert!(committed_separator_is_preserved(&row[0], &row[1]));
        assert!(
            plan.move_right > 0,
            "committed plan should keep the typed separator on screen when possible"
        );
    }
}

#[test]
fn committed_tail_long_word_keeps_stable_suffix_before_space() {
    let row = first_fixture_row("text_edit_long_word.tsv");
    assert_eq!(row.len(), 2, "long-word fixture must be TSV");
    let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
    assert_eq!(apply_plan(&row[0], &plan), row[1]);
    assert!(plan.move_right > 0);
    assert!(plan.backspaces < row[0].chars().count() as u32);
}

#[test]
fn committed_tail_full_token_plan_avoids_middle_edit_for_ime_autocorrect() {
    let original = "следущий ";
    let replacement = "следующий ";
    let minimal = plan_committed_tail_replacement(original, replacement).expect("minimal");
    let full_token =
        plan_committed_tail_full_token_replacement(original, replacement).expect("full token");

    assert_eq!(apply_plan(original, &minimal), replacement);
    assert_eq!(minimal.move_left, 4);
    assert_eq!(minimal.insert, "ю");
    assert_eq!(apply_plan(original, &full_token), replacement);
    assert_eq!(full_token, text_replacement(1, 8, "следующий", 1));
}

#[test]
fn autocorrect_safety_blocks_middle_suffix_plan_but_allows_full_token_plan() {
    let original = "следущий ";
    let replacement = "следующий ";
    let minimal = plan_committed_tail_replacement(original, replacement).expect("minimal");
    let full_token =
        plan_committed_tail_full_token_replacement(original, replacement).expect("full token");

    let minimal_safety = autocorrect_edit_safety(
        original,
        replacement,
        &minimal,
        Some("L2SurfaceMotifCell32"),
        Some("typo"),
    );
    let full_safety = autocorrect_edit_safety(
        original,
        replacement,
        &full_token,
        Some("L2SurfaceMotifCell32"),
        Some("typo"),
    );

    assert!(!minimal_safety.allow_apply);
    assert_eq!(
        minimal_safety.reason,
        "unsafe_middle_suffix_autocorrect_plan"
    );
    assert!(full_safety.allow_apply, "full_safety={full_safety:?}");
}

#[test]
fn autocorrect_safety_blocks_cursor_underflow_edit_plan() {
    let original = "провекрытое ";
    let replacement = "крытое ";
    let plan = text_replacement(5, 11, "крытое", 5);

    assert!(!replacement_plan_matches(original, replacement, &plan));
    assert_eq!(apply_plan(original, &plan), original);

    let safety = autocorrect_edit_safety(
        original,
        replacement,
        &plan,
        Some("glued_phrase"),
        Some("glued-words"),
    );

    assert!(!safety.allow_apply);
    assert_eq!(safety.reason, "invalid_edit_plan_cursor_bounds");
}

#[test]
fn autocorrect_safety_blocks_dry_run_mismatched_edit_plan() {
    let original = "провекрытое ";
    let replacement = "крытое ";
    let plan = text_replacement(5, 7, "крытое", 5);

    assert_eq!(apply_plan(original, &plan), "крытоеытое ");
    assert!(!replacement_plan_matches(original, replacement, &plan));

    let safety = autocorrect_edit_safety(
        original,
        replacement,
        &plan,
        Some("glued_phrase"),
        Some("glued-words"),
    );

    assert!(!safety.allow_apply);
    assert_eq!(safety.reason, "edit_plan_dry_run_mismatch");
}

#[test]
fn committed_tail_full_token_plan_can_be_shifted_behind_current_word() {
    let original = "следущий ";
    let replacement = "следующий ";
    let full_token =
        plan_committed_tail_full_token_replacement(original, replacement).expect("full token");
    let shifted = offset_replacement_plan_for_cursor(&full_token, 5);
    let original_with_next = ["следущий", "слово"].join(" ");
    let replacement_with_next = ["следующий", "слово"].join(" ");

    assert_eq!(shifted, text_replacement(6, 8, "следующий", 6));
    assert_eq!(
        apply_plan(&original_with_next, &shifted),
        replacement_with_next
    );
}

#[test]
fn committed_tail_sentence_plans_preserve_already_typed_space() {
    for row in fixture_rows("text_edit_sentence_plans.tsv") {
        assert_eq!(row.len(), 2, "sentence-plan fixture must be TSV");
        let original = &row[0];
        let replacement = &row[1];
        let plan = plan_committed_tail_replacement(original, replacement).expect("replacement");
        assert_eq!(apply_plan(original, &plan), *replacement);
        assert!(replacement_plan_matches(original, replacement, &plan));
        assert!(committed_separator_is_preserved(original, replacement));
        assert_eq!(original.ends_with(' '), replacement.ends_with(' '));
        assert!(
            plan.move_right > 0,
            "committed autocorrect keeps already typed suffix text instead of retyping it"
        );
    }
}

#[test]
fn committed_tail_split_word_plan_inserts_internal_space() {
    for row in fixture_rows("text_edit_split_word_space.tsv") {
        assert_eq!(row.len(), 2, "split-word fixture must be TSV");
        let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
        assert_eq!(apply_plan(&row[0], &plan), row[1]);
        assert_eq!(plan.backspaces, 0);
        assert_eq!(plan.insert, " ");
        assert!(plan.move_left > 0);
        assert_eq!(plan.move_left, plan.move_right);
    }
}

#[test]
fn committed_tail_split_word_plan_can_fix_small_typo_near_split() {
    let row = first_fixture_row("text_edit_split_word_typo.tsv");
    let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");

    assert_eq!(apply_plan(&row[0], &plan), row[1]);
    assert!(plan.move_left > 0);
    assert_eq!(plan.move_left, plan.move_right);
    assert!(plan.insert.contains(' '));
}

#[test]
fn committed_tail_non_split_replacement_keeps_existing_space_boundary() {
    let row = first_fixture_row("text_edit_non_split_boundary.tsv");
    let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");

    assert_eq!(apply_plan(&row[0], &plan), row[1]);
    assert!(plan.move_right > 0);
}

#[test]
fn committed_tail_plan_can_be_shifted_behind_current_word() {
    let row = first_fixture_row("text_edit_shifted_current.tsv");
    let base = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
    let shifted = offset_replacement_plan_for_cursor(&base, 6);

    assert_eq!(apply_plan(&row[2], &shifted), row[3]);
    assert_eq!(shifted.move_left, base.move_left + 6);
    assert_eq!(shifted.move_right, base.move_right + 6);
}

#[test]
fn committed_tail_layout_word_shifted_behind_current_russian_context() {
    let row = first_fixture_row("text_edit_shifted_layout_tail.tsv");
    let base = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
    let cursor_offset = row[4].parse().expect("cursor_offset");
    let shifted = offset_replacement_plan_for_cursor(&base, cursor_offset);

    assert_eq!(apply_plan(&row[2], &shifted), row[3]);
    assert_eq!(shifted.move_left, base.move_left + cursor_offset);
    assert_eq!(shifted.move_right, base.move_right + cursor_offset);
}

#[test]
fn committed_tail_context_edit_does_not_retype_stable_prefix() {
    let plan = plan_committed_tail_replacement("aa bb x ", "aa bb y ").expect("replacement");

    assert_eq!(plan, text_replacement(1, 1, "y", 1));
    assert_eq!(apply_plan("aa bb x ", &plan), "aa bb y ");
}

#[test]
fn committed_tail_autocorrect_keeps_port_sequence_spaces() {
    assert_autocorrect_sequence("text_edit_autocorrect_port_sequence.tsv");
}

#[test]
fn committed_tail_autocorrect_reinserts_space_before_uncorrected_next_word() {
    assert_autocorrect_sequence("text_edit_autocorrect_html_next.tsv");
}

#[test]
fn committed_tail_autocorrect_keeps_one_space_after_each_replacement() {
    assert_autocorrect_sequence("text_edit_autocorrect_one_space.tsv");
}

#[test]
fn committed_tail_space_insertions_handle_many_glued_words() {
    let row = first_fixture_row("text_edit_many_glued.tsv");
    let plans =
        plan_committed_whitespace_insertions(&row[0], &row[1], 0).expect("space insertion plans");

    let mut text = row[0].clone();
    for plan in &plans {
        text = apply_plan(&text, plan);
    }
    assert_eq!(text, row[1]);
    assert_eq!(plans.len(), 6);
    assert!(plans.iter().all(|plan| plan.backspaces == 0));
}

#[test]
fn autocorrect_safety_blocks_plain_previous_word_fix_without_boundary_proof() {
    let row = autocorrect_safety_fixture("plain_previous_word_fix");
    let original = &row[1];
    let replacement = &row[2];
    let plan = plan_committed_tail_replacement(original, replacement).expect("plan");
    let safety =
        autocorrect_edit_safety(original, replacement, &plan, Some(&row[3]), Some(&row[4]));

    assert!(!safety.allow_apply);
    assert!(original.split_whitespace().count() > 1);
    assert!(!safety.boundary_changed);
    assert!(safety.changes_non_last_word);
    assert_eq!(safety.reason, row[8]);
}

#[test]
fn autocorrect_safety_blocks_unproven_word_boundary_split() {
    let row = autocorrect_safety_fixture("unproven_word_boundary_split");
    let original = &row[1];
    let replacement = &row[2];
    let plan = plan_committed_tail_replacement(original, replacement).expect("plan");
    let safety =
        autocorrect_edit_safety(original, replacement, &plan, Some(&row[3]), Some(&row[4]));

    assert!(!safety.allow_apply);
    assert!(original.split_whitespace().count() > 1);
    assert!(safety.boundary_changed);
    assert_eq!(safety.reason, row[8]);
}

#[test]
fn autocorrect_safety_allows_proven_boundary_split() {
    let row = autocorrect_safety_fixture("proven_boundary_split");
    let original = &row[1];
    let replacement = &row[2];
    let plan = plan_committed_tail_replacement(original, replacement).expect("plan");
    let safety =
        autocorrect_edit_safety(original, replacement, &plan, Some(&row[3]), Some(&row[4]));

    assert!(safety.allow_apply);
    assert!(safety.boundary_changed);
}

#[test]
fn autocorrect_safety_requires_strong_boundary_shape() {
    for case_name in [
        "proven_negative_boundary_split",
        "weak_known_word_boundary_split",
        "weak_dirty_boundary_split",
        "safe_internal_glued_phrase_repair",
        "safe_internal_glued_phrase_transposition",
    ] {
        let row = autocorrect_safety_fixture(case_name);
        let original = &row[1];
        let replacement = &row[2];
        let plan = plan_committed_tail_replacement(original, replacement).expect("plan");
        let safety =
            autocorrect_edit_safety(original, replacement, &plan, Some(&row[3]), Some(&row[4]));

        assert_eq!(
            safety.allow_apply,
            row[5].parse::<bool>().expect("allow_apply"),
            "case={case_name} safety={safety:?}"
        );
        assert_eq!(
            safety.boundary_changed,
            row[6].parse::<bool>().expect("boundary")
        );
        if !row[8].is_empty() {
            assert_eq!(safety.reason, row[8]);
        }
    }
}

#[test]
fn autocorrect_safety_blocks_semantic_left_context_rewrite() {
    let row = autocorrect_safety_fixture("semantic_left_context_rewrite");
    let original = &row[1];
    let replacement = &row[2];
    let plan = plan_committed_tail_replacement(original, replacement).expect("plan");
    let safety =
        autocorrect_edit_safety(original, replacement, &plan, Some(&row[3]), Some(&row[4]));

    assert!(!safety.allow_apply);
    assert!(original.split_whitespace().count() > 1);
    assert!(safety.changes_non_last_word);
    assert_eq!(safety.reason, row[8]);
}

#[test]
fn committed_tail_space_insertions_can_be_shifted_behind_current_word() {
    let row = first_fixture_row("text_edit_space_insert_shifted.tsv");
    let plans =
        plan_committed_whitespace_insertions(&row[0], &row[1], 6).expect("space insertion plans");

    assert_eq!(plans, vec![text_replacement(12, 0, " ", 12)]);
    let mut text = row[2].clone();
    for plan in &plans {
        text = apply_plan(&text, plan);
    }
    assert_eq!(text, row[3]);
}

#[test]
fn committed_tail_space_insertions_reject_real_letter_changes() {
    let row = first_fixture_row("text_edit_reject_letter_change.tsv");
    assert_eq!(
        plan_committed_whitespace_insertions(&row[0], &row[1], 0),
        None
    );
}

#[test]
fn committed_tail_spacing_is_restored_before_planning() {
    for row in fixture_rows("text_edit_spacing_restore.tsv") {
        assert_eq!(row.len(), 3, "spacing-restore fixture must be TSV");
        assert_eq!(
            ensure_committed_tail_spacing(&row[0], row[1].clone()),
            row[2]
        );
    }
}

#[test]
fn committed_separator_contract_detects_eaten_space() {
    for row in fixture_rows("text_edit_separator_contract.tsv") {
        assert_eq!(row.len(), 3, "separator-contract fixture must be TSV");
        assert_eq!(
            committed_separator_is_preserved(&row[0], &row[1]),
            row[2] == "true"
        );
    }
}

#[test]
fn committed_tail_plans_apply_exactly_to_replacement() {
    for row in fixture_rows("text_edit_tail_plan_exact.tsv") {
        assert_eq!(row.len(), 2, "tail-plan fixture must be TSV");
        let original = &row[0];
        let replacement = &row[1];
        let plan = plan_committed_tail_replacement(original, replacement)
            .unwrap_or_else(|| panic!("replacement plan for {original:?}"));
        assert!(
            replacement_plan_matches(original, replacement, &plan),
            "{original:?} -> {replacement:?} via {plan:?}"
        );
        assert!(committed_separator_is_preserved(original, replacement));
    }
}

#[test]
fn committed_tail_last_token_plan_keeps_left_context_fixed() {
    let original = "я прохоил ";
    let replacement = "я проходил ";
    let plan = plan_committed_tail_last_token_replacement(original, replacement).expect("plan");

    assert_eq!(plan, text_replacement(1, 7, "проходил", 1));
    assert_eq!(apply_plan(original, &plan), replacement);
}

#[test]
fn committed_tail_last_token_plan_rejects_boundary_rewrite() {
    assert_eq!(
        plan_committed_tail_last_token_replacement("ябыл ", "я был "),
        None
    );
}

#[test]
fn tail_chars_returns_unicode_tail() {
    assert_eq!(tail_chars("привет", 3), "вет");
    assert_eq!(tail_chars("hi", 10), "hi");
}
