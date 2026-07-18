//! Shared text metrics for scorers and candidate arbitration.

use crate::keyboard::is_cyrillic_letter;

pub(crate) fn has_cyrillic(text: &str) -> bool {
    text.chars().any(is_cyrillic_letter)
}

pub(crate) fn has_latin(text: &str) -> bool {
    text.chars().any(|ch| ch.is_ascii_alphabetic())
}

pub fn without_whitespace(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

pub(crate) fn transition_changed_token_count(original: &str, replacement: &str) -> usize {
    let original = crate::word_reader::normalized_text_words(original);
    let replacement = crate::word_reader::normalized_text_words(replacement);
    if original.len() != replacement.len() {
        return original.len().max(replacement.len());
    }
    original
        .iter()
        .zip(replacement.iter())
        .filter(|(left, right)| left != right)
        .count()
}

pub(crate) fn transition_left_context_changed(original: &str, replacement: &str) -> bool {
    let original = crate::word_reader::normalized_text_words(original);
    let replacement = crate::word_reader::normalized_text_words(replacement);
    let original_prefix = original.get(..original.len().saturating_sub(1));
    let replacement_prefix = replacement.get(..replacement.len().saturating_sub(1));
    original_prefix != replacement_prefix
}

pub fn common_prefix_char_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

pub(crate) fn is_adjacent_transposition_chars(left: &[char], right: &[char]) -> bool {
    if left.len() != right.len() || left.len() < 2 {
        return false;
    }
    let differences = left
        .iter()
        .zip(right)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    matches!(differences.as_slice(), [first, second]
        if *second == *first + 1
            && left[*first] == right[*second]
            && left[*second] == right[*first])
}

pub fn is_adjacent_transposition(left: &str, right: &str) -> bool {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    is_adjacent_transposition_chars(&left, &right)
}

pub(crate) fn score_to_milli(value: f32) -> i16 {
    (value * 1000.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

pub fn damerau_levenshtein(left: &str, right: &str) -> usize {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let substitution = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + substitution);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(dp[i - 2][j - 2] + 1);
            }
            dp[i][j] = best;
        }
    }
    dp[a.len()][b.len()]
}

pub fn sparse_internal_omission_count(input: &str, candidate: &str) -> Option<usize> {
    let input = input.chars().collect::<Vec<_>>();
    let candidate = candidate.chars().collect::<Vec<_>>();
    let omissions = candidate.len().checked_sub(input.len())?;
    if !(2..=3).contains(&omissions)
        || input.first() != candidate.first()
        || input.last() != candidate.last()
    {
        return None;
    }

    let mut input_index = 0usize;
    let mut omitted = Vec::with_capacity(omissions);
    for (candidate_index, ch) in candidate.iter().enumerate() {
        if input.get(input_index) == Some(ch) {
            input_index += 1;
        } else {
            omitted.push(candidate_index);
        }
    }
    if input_index != input.len()
        || omitted.len() != omissions
        || omitted
            .iter()
            .any(|index| *index == 0 || *index + 1 == candidate.len())
        || omitted.windows(2).all(|pair| pair[1] == pair[0] + 1)
    {
        return None;
    }
    Some(omissions)
}

#[cfg(test)]
mod tests {
    use super::{
        sparse_internal_omission_count, transition_changed_token_count,
        transition_left_context_changed,
    };

    #[test]
    fn transition_metrics_share_one_word_boundary_definition() {
        assert_eq!(
            transition_changed_token_count("я прохоил ", "я проходил "),
            1
        );
        assert!(!transition_left_context_changed(
            "я прохоил ",
            "я проходил "
        ));

        assert_eq!(
            transition_changed_token_count("я прохоил ", "япроходил "),
            2
        );
        assert!(transition_left_context_changed("я прохоил ", "япроходил "));
    }

    #[test]
    fn sparse_internal_omissions_are_a_typed_edit_geometry() {
        assert_eq!(
            sparse_internal_omission_count("переподлчаю", "переподключаю"),
            Some(2)
        );
        assert_eq!(
            sparse_internal_omission_count("интелека", "интеллекта"),
            Some(2)
        );
        assert_eq!(sparse_internal_omission_count("спть", "спать"), None);
        assert_eq!(
            sparse_internal_omission_count("переподчаю", "переподключаю"),
            None
        );
    }
}
