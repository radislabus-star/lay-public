use super::*;
use std::path::PathBuf;

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

fn fixture_rows(name: &str) -> Vec<Vec<String>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {path:?}: {err}"))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split('\t')
                .map(|field| field.replace("\\s", " "))
                .collect()
        })
        .collect()
}

#[test]
fn plans_minimal_two_word_prefix_and_suffix_edits() {
    for row in fixture_rows("text_edit_minimal_plan.tsv") {
        assert_eq!(row.len(), 6, "minimal-plan fixture must be TSV");
        assert_eq!(
            plan_text_replacement(&row[0], &row[1]),
            Some(TextReplacement {
                move_left: row[2].parse().expect("move_left"),
                backspaces: row[3].parse().expect("backspaces"),
                insert: row[4].clone(),
                move_right: row[5].parse().expect("move_right"),
            })
        );
    }
}

#[test]
fn committed_tail_plan_preserves_typed_trailing_space_boundary() {
    for row in fixture_rows("text_edit_committed_tail_boundary.tsv") {
        assert_eq!(row.len(), 2, "tail-boundary fixture must be TSV");
        let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
        assert_eq!(apply_plan(&row[0], &plan), row[1]);
        assert_eq!(plan.move_left, 0);
        assert_eq!(plan.backspaces, row[0].chars().count() as u32);
        assert_eq!(plan.insert, row[1]);
        assert_eq!(plan.move_right, 0);
    }
}

#[test]
fn committed_tail_long_word_replaces_whole_body_before_space() {
    let row = fixture_rows("text_edit_long_word.tsv")
        .into_iter()
        .next()
        .expect("long-word fixture");
    assert_eq!(row.len(), 2, "long-word fixture must be TSV");
    let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
    assert_eq!(apply_plan(&row[0], &plan), row[1]);
    assert_eq!(plan.move_left, 0);
    assert_eq!(plan.backspaces, row[0].chars().count() as u32);
    assert_eq!(plan.insert, row[1]);
    assert_eq!(plan.move_right, 0);
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
        assert_eq!(plan.move_left, 0);
        assert_eq!(plan.move_right, 0);
        assert_eq!(plan.backspaces, original.chars().count() as u32);
        assert!(
            plan.insert.ends_with(' '),
            "committed autocorrect reinserts the typed separator with the replacement"
        );
    }
}

#[test]
fn committed_tail_split_word_plan_inserts_internal_space() {
    for row in fixture_rows("text_edit_split_word_space.tsv") {
        assert_eq!(row.len(), 2, "split-word fixture must be TSV");
        let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
        assert_eq!(
            plan,
            TextReplacement {
                move_left: 0,
                backspaces: row[0].chars().count() as u32,
                insert: row[1].clone(),
                move_right: 0,
            }
        );
        assert_eq!(apply_plan(&row[0], &plan), row[1]);
    }
}

#[test]
fn committed_tail_split_word_plan_can_fix_small_typo_near_split() {
    let row = fixture_rows("text_edit_split_word_typo.tsv")
        .into_iter()
        .next()
        .expect("split-word typo fixture");
    let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");

    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: row[0].chars().count() as u32,
            insert: row[1].clone(),
            move_right: 0,
        }
    );
    assert_eq!(apply_plan(&row[0], &plan), row[1]);
}

#[test]
fn committed_tail_non_split_replacement_keeps_existing_space_boundary() {
    let row = fixture_rows("text_edit_non_split_boundary.tsv")
        .into_iter()
        .next()
        .expect("non-split fixture");
    let plan = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");

    assert_eq!(
        plan,
        TextReplacement {
            move_left: 0,
            backspaces: row[0].chars().count() as u32,
            insert: row[1].clone(),
            move_right: 0,
        }
    );
    assert_eq!(apply_plan(&row[0], &plan), row[1]);
}

#[test]
fn committed_tail_plan_can_be_shifted_behind_current_word() {
    let row = fixture_rows("text_edit_shifted_current.tsv")
        .into_iter()
        .next()
        .expect("shifted fixture");
    let base = plan_committed_tail_replacement(&row[0], &row[1]).expect("replacement");
    let shifted = offset_replacement_plan_for_cursor(&base, 6);

    assert_eq!(
        shifted,
        TextReplacement {
            move_left: 6,
            backspaces: row[0].chars().count() as u32,
            insert: row[1].clone(),
            move_right: 6,
        }
    );
    assert_eq!(apply_plan(&row[2], &shifted), row[3]);
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
    let row = fixture_rows("text_edit_many_glued.tsv")
        .into_iter()
        .next()
        .expect("many-glued fixture");
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
fn committed_tail_space_insertions_can_be_shifted_behind_current_word() {
    let row = fixture_rows("text_edit_space_insert_shifted.tsv")
        .into_iter()
        .next()
        .expect("space insert shifted fixture");
    let plans =
        plan_committed_whitespace_insertions(&row[0], &row[1], 6).expect("space insertion plans");

    assert_eq!(
        plans,
        vec![TextReplacement {
            move_left: 12,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: 12,
        }]
    );
    let mut text = row[2].clone();
    for plan in &plans {
        text = apply_plan(&text, plan);
    }
    assert_eq!(text, row[3]);
}

#[test]
fn committed_tail_space_insertions_reject_real_letter_changes() {
    let row = fixture_rows("text_edit_reject_letter_change.tsv")
        .into_iter()
        .next()
        .expect("reject fixture");
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
fn tail_chars_returns_unicode_tail() {
    assert_eq!(tail_chars("привет", 3), "вет");
    assert_eq!(tail_chars("hi", 10), "hi");
}
