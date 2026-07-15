use super::*;

#[test]
fn typing_assist_merges_accidental_space_inside_word() {
    for row in fixture_rows("daemon_typing_assist_split_word_fix.tsv") {
        assert_eq!(row.len(), 2, "split-word fixture must be TSV");
        assert_eq!(
            select_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
    for input in fixture_lines("daemon_typing_assist_split_word_keep.txt") {
        assert_eq!(select_typing_assist_exact(&input), None, "input={input:?}");
    }
    for input in fixture_lines("daemon_typing_assist_split_word_keep_layout.txt") {
        assert_eq!(select_typing_assist(&input, true), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_splits_accidentally_glued_words() {
    for row in fixture_rows("daemon_typing_assist_glued_split_fix.tsv") {
        assert_eq!(row.len(), 2, "glued-split fixture must be TSV");
        assert_eq!(
            select_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
    for input in fixture_lines("daemon_typing_assist_glued_split_keep.txt") {
        assert_eq!(select_typing_assist_exact(&input), None, "input={input:?}");
    }
    for input in fixture_lines("daemon_typing_assist_glued_split_keep_tail.txt") {
        assert_eq!(
            apply_typing_assist_to_text_tail(&input),
            None,
            "input={input:?}"
        );
    }
}

#[test]
fn typing_assist_fixes_hard_sign_typos() {
    for row in fixture_rows("daemon_typing_assist_hard_sign_fix.tsv") {
        assert_eq!(row.len(), 2, "hard-sign fixture must be TSV");
        assert_eq!(
            select_typing_assist_exact(&row[0]),
            Some(row[1].clone()),
            "input={:?}",
            row[0]
        );
    }
    for input in fixture_lines("daemon_typing_assist_hard_sign_keep.txt") {
        assert_eq!(select_typing_assist_exact(&input), None, "input={input:?}");
    }
}

#[test]
fn typing_assist_moves_letter_from_next_word_back() {
    for row in fixture_rows("daemon_typing_assist_moved_prefix.tsv") {
        assert_eq!(row.len(), 3, "moved-prefix fixture must be TSV");
        let input = &row[0];
        let expected = &row[1];
        let use_tail = row[2] == "tail";
        let got = if use_tail {
            apply_typing_assist_to_text_tail(input)
        } else {
            select_typing_assist_exact(input)
        };
        assert_eq!(got, Some(expected.clone()), "input={input:?}");
    }
}
