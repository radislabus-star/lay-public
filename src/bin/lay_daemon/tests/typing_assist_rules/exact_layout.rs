use super::*;

#[test]
fn applies_builtin_auto_replace_with_trailing_space() {
    for row in fixture_rows("daemon_typing_assist_builtin_auto_replace_fix.tsv") {
        assert_eq!(row.len(), 3, "builtin auto-replace fixture must be TSV");
        assert_eq!(
            apply_auto_replace(&row[0], &row[1]),
            Some(row[2].clone()),
            "original={:?} target={:?}",
            row[0],
            row[1]
        );
    }
    for row in fixture_rows("daemon_typing_assist_builtin_auto_replace_keep.tsv") {
        assert_eq!(
            row.len(),
            2,
            "builtin auto-replace keep fixture must be TSV"
        );
        assert_eq!(apply_auto_replace(&row[0], &row[1]), None);
    }
}

#[test]
fn typing_assist_uses_exact_rules_only() {
    for row in fixture_rows("daemon_typing_assist_exact_rules_fix.tsv") {
        assert_eq!(row.len(), 2, "exact-rule fixture must be TSV");
        assert_eq!(select_typing_assist_exact(&row[0]), Some(row[1].clone()));
    }
    for input in fixture_lines("daemon_typing_assist_exact_rules_keep.txt") {
        assert_eq!(select_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn russian_suffix_forms_are_known_candidates() {
    for input in fixture_lines("daemon_typing_assist_known_forms.txt") {
        assert!(
            is_known_russian_word_or_form(&input),
            "known form missing: {input:?}"
        );
    }
}

#[test]
fn typing_assist_auto_switch_blocks_plain_layout_words_and_keeps_explicit_cases() {
    for input in fixture_lines("daemon_typing_assist_auto_switch_blocked.txt") {
        assert_eq!(
            select_typing_assist(&input, true),
            None,
            "plain layout word must not be auto-switched: {input:?}"
        );
    }

    for row in fixture_rows("daemon_typing_assist_tail_cases.tsv") {
        assert_eq!(row.len(), 2, "tail cases fixture must be TSV");
        assert_eq!(
            apply_typing_assist_to_text_tail(&row[0]),
            Some(row[1].clone())
        );
    }
    for row in fixture_rows("daemon_typing_assist_layout_explicit.tsv") {
        assert_eq!(row.len(), 2, "layout explicit fixture must be TSV");
        assert_eq!(select_typing_assist(&row[0], true), Some(row[1].clone()));
    }
}

#[test]
fn typing_assist_auto_replace_off_keeps_layout_only_rules() {
    let pipeline =
        typing_assist_pipeline_for_auto_replace(false, &default_typing_assist_pipeline());

    for row in fixture_rows("daemon_typing_assist_auto_replace_off_fix.tsv") {
        assert_eq!(row.len(), 2, "auto-replace-off fix fixture must be TSV");
        assert_eq!(
            select_typing_assist_with_pipeline(&row[0], true, &pipeline),
            Some(row[1].clone())
        );
    }
    for row in fixture_rows("daemon_typing_assist_auto_replace_off_keep.tsv") {
        assert_eq!(row.len(), 2, "auto-replace-off keep fixture must be TSV");
        let allow_layout = row[1] == "true";
        assert_eq!(
            select_typing_assist_with_pipeline(&row[0], allow_layout, &pipeline),
            None
        );
    }
}

#[test]
fn typing_assist_auto_replace_pipeline_avoids_risky_deletions() {
    let pipeline = typing_assist_pipeline_for_auto_replace(true, &default_typing_assist_pipeline());

    for row in fixture_rows("daemon_typing_assist_risky_pipeline_fix.tsv") {
        assert_eq!(row.len(), 2, "risky-pipeline fix fixture must be TSV");
        assert_eq!(
            select_typing_assist_with_pipeline(&row[0], false, &pipeline),
            Some(row[1].clone())
        );
    }
    for input in fixture_lines("daemon_typing_assist_risky_pipeline_keep.txt") {
        assert_eq!(
            select_typing_assist_with_pipeline(&input, false, &pipeline),
            None,
            "input={input:?}"
        );
    }
}

#[test]
fn typing_assist_prefers_reflexive_verb_fix_over_extra_letter_guess() {
    for row in fixture_rows("daemon_typing_assist_reflexive_verb_fix.tsv") {
        assert_eq!(row.len(), 2, "reflexive verb fixture must be TSV");
        assert_eq!(correct_extra_letters(&row[0]), None);
        assert_eq!(
            select_typing_assist(&format!("{} ", row[0]), false),
            Some(format!("{} ", row[1]))
        );
    }
}

#[test]
fn typing_assist_auto_switch_keeps_english_and_protected_ascii() {
    for input in fixture_lines("daemon_typing_assist_auto_switch_keep.txt") {
        assert_eq!(select_typing_assist(&input, true), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_keeps_user_protected_ascii_words_when_configured() {
    if std::env::var_os("LAY_TEST_USER_PROTECTED_ASCII").is_none() {
        return;
    }

    for input in fixture_lines("daemon_typing_assist_user_protected_keep.txt") {
        assert_eq!(select_typing_assist(&input, true), None, "input={input:?}");
    }
}
