use crate::data_lines::data_lines;
use crate::ru_typo::{correct_missing_letter, fuzzy_known_word_candidates};
use crate::word_reader::{split_word_punctuation, split_ws_segments};

const CONTEXTUAL_FUZZY_PAIRS_DATA: &str =
    include_str!("../../data/lexicon/russian_contextual_fuzzy_pairs.tsv");

pub(crate) fn correct_contextual_fuzzy_pair(text: &str) -> Option<String> {
    let segments = split_ws_segments(text);
    let word_indices = segments
        .iter()
        .enumerate()
        .filter_map(|(idx, (_, is_ws))| (!*is_ws).then_some(idx))
        .collect::<Vec<_>>();
    if word_indices.len() < 2 {
        return None;
    }
    let left_idx = word_indices[word_indices.len() - 2];
    let right_idx = word_indices[word_indices.len() - 1];
    let (left_leading, left_word, left_trailing) = split_word_punctuation(segments[left_idx].0);
    let (right_leading, right_word, right_trailing) = split_word_punctuation(segments[right_idx].0);
    if left_word.is_empty()
        || right_word.is_empty()
        || !left_trailing.is_empty()
        || !right_leading.is_empty()
    {
        return None;
    }

    let right_lower = right_word.to_lowercase();
    let repaired_right = correct_missing_letter(&right_lower).unwrap_or(right_lower);
    let left_candidates = fuzzy_known_word_candidates(&left_word.to_lowercase());
    let mut matched_left = None;
    for candidate in left_candidates {
        if contextual_fuzzy_pairs().any(|(left_prefix, right_word)| {
            candidate.starts_with(left_prefix) && repaired_right == right_word
        }) {
            if matched_left.is_some() {
                return None;
            }
            matched_left = Some(candidate);
        }
    }
    let matched_left = matched_left?;

    let mut output = String::with_capacity(text.len() + matched_left.len() + repaired_right.len());
    for (idx, (segment, _is_ws)) in segments.iter().enumerate() {
        if idx == left_idx {
            output.push_str(left_leading);
            output.push_str(&crate::text_case::apply_word_case(left_word, &matched_left));
        } else if idx == right_idx {
            output.push_str(right_leading);
            output.push_str(&crate::text_case::apply_word_case(
                right_word,
                &repaired_right,
            ));
            output.push_str(right_trailing);
        } else {
            output.push_str(segment);
        }
    }
    (output != text).then_some(output)
}

fn contextual_fuzzy_pairs() -> impl Iterator<Item = (&'static str, &'static str)> {
    data_lines(CONTEXTUAL_FUZZY_PAIRS_DATA).filter_map(|line| line.split_once('\t'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextual_fuzzy_pair_abstains_without_a_represented_left_form() {
        assert_eq!(
            correct_contextual_fuzzy_pair("досвкйо лгистика").as_deref(),
            None
        );
    }
}
