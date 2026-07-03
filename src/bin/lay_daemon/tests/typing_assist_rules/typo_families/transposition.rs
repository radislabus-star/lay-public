#[path = "transposition/sweep.rs"]
mod sweep;
use super::*;

#[test]
fn typing_assist_fixes_adjacent_transposition() {
    for row in fixture_rows("daemon_typing_assist_transposition.tsv") {
        assert_eq!(row.len(), 3, "transposition fixture must be TSV");
        let got = if row[2] == "tail" {
            apply_typing_assist_to_text_tail(&row[0])
        } else {
            apply_typing_assist_exact(&row[0])
        };
        assert_eq!(got, Some(row[1].clone()), "input={:?}", row[0]);
    }
}

#[test]
fn typing_assist_fixes_small_glued_words() {
    for row in fixture_rows("daemon_typing_assist_small_glued.tsv") {
        assert_eq!(row.len(), 2, "small glued fixture must be TSV");
        assert_eq!(apply_typing_assist_exact(&row[0]), Some(row[1].clone()));
    }
}
