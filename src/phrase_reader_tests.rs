use super::{
    correct_contextual_glued_tail, correct_glued_russian_phrase, correct_moved_prefix_letter_pair,
    correct_split_word_pair, propose_moved_prefix_letter_pair,
};
use crate::typing_assist_test_fixtures::fixture_rows;
use crate::word_reader::{is_cyrillic_word, split_word_punctuation};

const CLEAN_BOUNDARY_CORPUS: &str = include_str!("../data/nanda_llmwave_seed_phrases.txt");

#[test]
fn splits_confident_glued_phrase_without_daemon_runtime() {
    for row in fixture_rows("phrase_reader_glued.tsv") {
        assert_eq!(row.len(), 2, "glued phrase fixture must be TSV");
        assert_eq!(correct_glued_russian_phrase(&row[0]), Some(row[1].clone()));
    }
}

#[test]
fn glued_phrase_defers_to_whole_word_typo_candidate() {
    for row in fixture_rows("phrase_reader_glued_keep.txt") {
        assert_eq!(row.len(), 1, "glued keep fixture must have one field");
        assert_eq!(correct_glued_russian_phrase(&row[0]), None);
    }
}

#[test]
fn splits_short_function_glued_to_be_form() {
    let row = fixture_rows("phrase_reader_glued_function_words.tsv")
        .into_iter()
        .next()
        .expect("function-word glued fixture");
    assert_eq!(row.len(), 2, "function-word glued fixture must be TSV");
    assert_eq!(correct_glued_russian_phrase(&row[0]), Some(row[1].clone()));
}

#[test]
fn splits_contextual_glued_tail_in_short_phrase() {
    for row in fixture_rows("phrase_reader_contextual_glued.tsv") {
        assert_eq!(row.len(), 2, "contextual glued fixture must be TSV");
        assert_eq!(correct_contextual_glued_tail(&row[0]), Some(row[1].clone()));
    }
    for row in fixture_rows("phrase_reader_contextual_keep.tsv") {
        assert_eq!(row.len(), 1, "contextual keep fixture must have one field");
        assert_eq!(correct_contextual_glued_tail(&row[0]), None);
    }
}

#[test]
fn merges_accidental_split_word_but_keeps_normal_pair() {
    for row in fixture_rows("phrase_reader_split_pair.tsv") {
        assert_eq!(row.len(), 2, "split pair fixture must be TSV");
        assert_eq!(correct_split_word_pair(&row[0]), Some(row[1].clone()));
    }
    for row in fixture_rows("phrase_reader_split_pair_keep.tsv") {
        assert_eq!(row.len(), 1, "split pair keep fixture must have one field");
        assert_eq!(correct_split_word_pair(&row[0]), None);
    }
}

#[test]
fn moves_next_word_prefix_back_when_phrase_score_is_confident() {
    for row in fixture_rows("phrase_reader_moved_prefix.tsv") {
        assert_eq!(row.len(), 2, "moved prefix fixture must be TSV");
        assert_eq!(
            correct_moved_prefix_letter_pair(&row[0]),
            Some(row[1].clone())
        );
    }
}

#[test]
fn keeps_normal_two_word_boundaries() {
    for row in fixture_rows("phrase_reader_moved_prefix_keep.txt") {
        assert_eq!(
            row.len(),
            1,
            "moved-prefix keep fixture must have one field"
        );
        assert_eq!(correct_moved_prefix_letter_pair(&row[0]), None);
    }
}

#[test]
fn boundary_shift_field_replays_clean_and_synthetic_corpus_pairs() {
    let mut clean_pairs = Vec::new();
    for line in CLEAN_BOUNDARY_CORPUS.lines() {
        let words = line
            .split_whitespace()
            .filter_map(|token| {
                let (_, word, _) = split_word_punctuation(token);
                is_cyrillic_word(word).then(|| word.to_lowercase())
            })
            .collect::<Vec<_>>();
        clean_pairs.extend(
            words
                .windows(2)
                .map(|pair| (pair[0].clone(), pair[1].clone())),
        );
    }

    let false_applies = clean_pairs
        .iter()
        .filter_map(|(left, right)| {
            let clean = format!("{left} {right}");
            correct_moved_prefix_letter_pair(&clean).map(|replacement| {
                let replacement_words = replacement.split_whitespace().collect::<Vec<_>>();
                let original_mass = boundary_pair_mass(left, right);
                let replacement_mass =
                    boundary_pair_mass(replacement_words[0], replacement_words[1]);
                (
                    clean,
                    replacement,
                    replacement_mass.saturating_sub(original_mass),
                )
            })
        })
        .collect::<Vec<_>>();

    let mut eligible = 0usize;
    let mut proposed = 0usize;
    let mut recovered = 0usize;
    let mut proposal_misses = Vec::new();
    for (left, right) in &clean_pairs {
        let mut left_chars = left.chars().collect::<Vec<_>>();
        if left_chars.len() < 2 || right.chars().count() < 2 {
            continue;
        }
        let moved = left_chars.pop().expect("left has at least two characters");
        let dirty_left = left_chars.into_iter().collect::<String>();
        let dirty_right = format!("{moved}{right}");
        // The corruption operator can land on another independently attested
        // surface (for example `режим не -> режи мне`). Such a signal is
        // ambiguous, so it belongs to Tied/ABSTAIN rather than restoration
        // recall.
        if crate::russian_lexicon::has_clean_russian_surface_certificate(&dirty_right) {
            continue;
        }
        let dirty = format!("{dirty_left} {dirty_right}");
        let expected = format!("{left} {right}");
        eligible += 1;
        if propose_moved_prefix_letter_pair(&dirty).as_deref() == Some(expected.as_str()) {
            proposed += 1;
        } else {
            proposal_misses.push((dirty.clone(), expected.clone()));
        }
        if correct_moved_prefix_letter_pair(&dirty).as_deref() == Some(expected.as_str()) {
            recovered += 1;
        }
    }

    eprintln!(
        "boundary-shift corpus clean={} false_applies={} synthetic={} proposed={} recovered={} proposal_misses={}",
        clean_pairs.len(),
        false_applies.len(),
        eligible,
        proposed,
        recovered,
        proposal_misses.len()
    );
    assert!(clean_pairs.len() >= 100, "clean pair denominator too small");
    assert!(false_applies.is_empty(), "false applies: {false_applies:?}");
    assert!(eligible >= 80, "synthetic pair denominator too small");
    assert!(
        proposed.saturating_mul(1_000) / eligible >= 980,
        "proposal recall below 98%: proposed={proposed} eligible={eligible} misses={proposal_misses:?}"
    );
}

fn boundary_pair_mass(left: &str, right: &str) -> u32 {
    let field = crate::hot_field::HotFieldSnapshot::current();
    field
        .surface_phase_readout(left)
        .transition_mass()
        .saturating_add(field.surface_phase_readout(right).transition_mass())
}
