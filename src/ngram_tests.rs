use super::*;
use crate::typing_assist_test_fixtures::{fixture_lines_from_str, fixture_row_by_id, fixture_rows};

fn ru_test_model() -> CharNgramModel {
    CharNgramModel::train(
        Lang::Ru,
        fixture_lines_from_str(include_str!("../tests/fixtures/ngram_ru_train_words.txt")),
    )
}

fn local_score_is_better(label: &str) {
    let model = ru_test_model();
    let row = fixture_row_by_id("ngram_local_score_pairs.tsv", label);
    assert_eq!(row.len(), 3, "local ngram fixture must be TSV");
    assert!(
        model.score_text(&row[1]) > model.score_text(&row[2]),
        "{}={} {}={}",
        row[1],
        model.score_text(&row[1]),
        row[2],
        model.score_text(&row[2])
    );
}

#[test]
fn scores_good_word_above_transposed_typo() {
    local_score_is_better("transposed_typo");
}

#[test]
fn scores_common_word_above_rare_transposition() {
    local_score_is_better("rare_transposition");
}

#[test]
fn scores_merged_word_above_accidental_split() {
    local_score_is_better("accidental_split");
}

#[test]
fn global_ru_model_can_rank_local_words() {
    for row in fixture_rows("ngram_global_margin.tsv") {
        assert_eq!(row.len(), 4, "global ngram fixture must be TSV");
        let min_margin: f64 = row[3].parse().expect("min margin");
        assert!(
            ru_candidate_margin(&row[1], &row[2]) > min_margin,
            "label={} margin={} min={min_margin}",
            row[0],
            ru_candidate_margin(&row[1], &row[2])
        );
    }
}
