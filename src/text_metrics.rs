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

pub(crate) fn current_token_boundary_split(original: &str, replacement: &str) -> bool {
    let original_words = crate::word_reader::normalized_text_words(original);
    let replacement_words = crate::word_reader::normalized_text_words(replacement);
    if original_words.is_empty() || replacement_words.len() != original_words.len() + 1 {
        return false;
    }
    let split_idx = original_words.len() - 1;
    original_words[..split_idx] == replacement_words[..split_idx]
        && original_words[split_idx]
            == format!(
                "{}{}",
                replacement_words[split_idx],
                replacement_words[split_idx + 1]
            )
}

pub(crate) fn current_token_boundary_split_or_repair(original: &str, replacement: &str) -> bool {
    current_token_boundary_split(original, replacement)
        || current_token_repaired_boundary_split(original, replacement)
}

pub(crate) fn current_token_repaired_boundary_split(original: &str, replacement: &str) -> bool {
    let original_words = crate::word_reader::normalized_text_words(original);
    let replacement_words = crate::word_reader::normalized_text_words(replacement);
    if original_words.is_empty()
        || replacement_words.len() <= original_words.len()
        || replacement_words.len() > original_words.len() + 2
    {
        return false;
    }
    let split_idx = original_words.len() - 1;
    if original_words[..split_idx] != replacement_words[..split_idx] {
        return false;
    }
    confident_boundary_split_sequence(&original_words[split_idx], &replacement_words[split_idx..])
}

fn confident_boundary_split_sequence(original: &str, parts: &[String]) -> bool {
    match parts {
        [left, right] => confident_boundary_split_pair(original, left, right, true),
        [first, middle, last] => {
            let joined = format!("{first}{middle}{last}");
            let original_known = crate::phrase_lexicon::is_known_russian_phrase_part(original);
            let all_known = [first.as_str(), middle.as_str(), last.as_str()]
                .into_iter()
                .all(crate::phrase_lexicon::is_known_russian_phrase_part);
            let has_function_word = [first.as_str(), middle.as_str(), last.as_str()]
                .into_iter()
                .any(crate::phrase_lexicon::is_short_russian_function_word);
            let has_stable_content = [first.as_str(), middle.as_str(), last.as_str()]
                .into_iter()
                .any(|word| word.chars().count() >= 3);
            !original_known
                && all_known
                && has_function_word
                && has_stable_content
                && damerau_levenshtein(original, &joined) <= 2
        }
        _ => false,
    }
}

pub(crate) fn confident_boundary_split_pair(
    original: &str,
    left: &str,
    right: &str,
    single_original_word: bool,
) -> bool {
    if original.is_empty() || left.is_empty() || right.is_empty() {
        return false;
    }

    let joined = format!("{left}{right}");
    let compact_equal = joined == original;
    let left_len = left.chars().count();
    let right_len = right.chars().count();
    let left_known = crate::phrase_lexicon::is_known_russian_phrase_part(left);
    let right_known = crate::phrase_lexicon::is_known_russian_phrase_part(right);
    let original_known = crate::phrase_lexicon::is_known_russian_phrase_part(original);
    let left_one_letter_function =
        left_len == 1 && crate::phrase_lexicon::is_one_letter_russian_function_word(left);
    let right_one_letter_function =
        right_len == 1 && crate::phrase_lexicon::is_one_letter_russian_function_word(right);
    let left_short_function = crate::phrase_lexicon::is_short_russian_function_word(left);
    let left_multi_letter_preposition =
        left_len > 1 && crate::phrase_lexicon::is_common_short_russian_preposition(left);
    let right_short_function = crate::phrase_lexicon::is_short_russian_function_word(right);

    if compact_equal {
        let both_stable_content =
            !original_known && left_known && right_known && left_len >= 4 && right_len >= 4;
        return (left_one_letter_function && right_known)
            || (right_one_letter_function && left_known)
            || (left_short_function && !left_multi_letter_preposition && right_known)
            || (left_known && right_short_function)
            || both_stable_content;
    }

    if original_known || !left_known || !right_known || !single_original_word {
        return false;
    }

    let distance = damerau_levenshtein(original, &joined);
    distance <= 2
        && ((left_len >= 4 && right_len >= 3)
            || (left_short_function && !left_multi_letter_preposition && right_len >= 4)
            || (right_short_function && left_len >= 4))
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

pub(crate) fn typed_damage_geometry_priority(input: &str, candidate: &str) -> u8 {
    if is_adjacent_transposition(input, candidate) {
        return 4;
    }
    if sparse_internal_omission_count(input, candidate).is_some() {
        return 3;
    }
    if candidate.chars().count() == input.chars().count() + 1
        && damerau_levenshtein(input, candidate) == 1
    {
        return 2;
    }
    0
}

/// True when one internal character moved to another internal position while
/// the word kept its boundaries and character mass.
pub(crate) fn is_single_internal_char_move(left: &str, right: &str) -> bool {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    if left.len() != right.len()
        || left.len() < 5
        || left.first() != right.first()
        || left.last() != right.last()
        || left == right
    {
        return false;
    }
    for from in 1..left.len().saturating_sub(1) {
        let mut without = left.clone();
        let moved = without.remove(from);
        for to in 1..without.len() {
            let mut candidate = without.clone();
            candidate.insert(to, moved);
            if candidate == right {
                return true;
            }
        }
    }
    false
}

pub(crate) fn score_to_milli(value: f32) -> i16 {
    (value * 1000.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

pub fn damerau_levenshtein(left: &str, right: &str) -> usize {
    damerau_levenshtein_impl(left, right, None)
}

pub(crate) fn damerau_levenshtein_bounded(
    left: &str,
    right: &str,
    maximum: usize,
) -> Option<usize> {
    let distance = damerau_levenshtein_impl(left, right, Some(maximum));
    (distance <= maximum).then_some(distance)
}

fn damerau_levenshtein_impl(left: &str, right: &str, maximum: Option<usize>) -> usize {
    let a = left.chars().collect::<Vec<_>>();
    let b = right.chars().collect::<Vec<_>>();
    if maximum.is_some_and(|maximum| a.len().abs_diff(b.len()) > maximum) {
        return maximum.unwrap_or_default().saturating_add(1);
    }
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let unreachable = maximum
        .map(|maximum| maximum.saturating_add(1))
        .unwrap_or(usize::MAX / 4);
    let mut previous_two = vec![unreachable; b.len() + 1];
    let mut previous = (0..=b.len()).collect::<Vec<_>>();
    let mut current = vec![unreachable; b.len() + 1];
    for i in 1..=a.len() {
        current.fill(unreachable);
        current[0] = i;
        let (start, end) = maximum.map_or((1, b.len()), |maximum| {
            (
                i.saturating_sub(maximum).max(1),
                i.saturating_add(maximum).min(b.len()),
            )
        });
        let mut row_minimum = current[0];
        for j in start..=end {
            let substitution = usize::from(a[i - 1] != b[j - 1]);
            let mut best = previous[j]
                .saturating_add(1)
                .min(current[j - 1].saturating_add(1))
                .min(previous[j - 1].saturating_add(substitution));
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(previous_two[j - 2].saturating_add(1));
            }
            current[j] = best;
            row_minimum = row_minimum.min(best);
        }
        if maximum.is_some_and(|maximum| row_minimum > maximum) {
            return maximum.unwrap_or_default().saturating_add(1);
        }
        std::mem::swap(&mut previous_two, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

pub fn sparse_internal_omission_count(input: &str, candidate: &str) -> Option<usize> {
    let input = input.chars().collect::<Vec<_>>();
    let candidate = candidate.chars().collect::<Vec<_>>();
    let omissions = candidate.len().checked_sub(input.len())?;
    if !(2..=3).contains(&omissions) || input.last() != candidate.last() {
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
        || omitted.iter().any(|index| *index + 1 == candidate.len())
        || omitted.windows(2).all(|pair| pair[1] == pair[0] + 1)
    {
        return None;
    }
    if input.first() != candidate.first()
        && omitted.iter().filter(|index| **index == 0).count() != 1
    {
        return None;
    }
    Some(omissions)
}

pub(crate) fn internal_char_confusion_preserves_frame(left: &str, right: &str) -> bool {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    let len = left_chars.len();
    if len != right_chars.len()
        || !(5..=16).contains(&len)
        || left_chars.first() != right_chars.first()
        || left_chars.last() != right_chars.last()
        || left_chars == right_chars
        || damerau_levenshtein(left, right) > 2
    {
        return false;
    }

    let common_prefix = common_prefix_char_len(left, right);
    let common_suffix = left_chars
        .iter()
        .rev()
        .zip(right_chars.iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    if common_prefix + common_suffix < len.saturating_sub(3) {
        return false;
    }

    let shared_mass = shared_char_mass(&left_chars, &right_chars);
    shared_mass + 1 >= len
}

fn shared_char_mass(left: &[char], right: &[char]) -> usize {
    let mut remaining = right.to_vec();
    let mut shared = 0usize;
    for ch in left {
        if let Some(index) = remaining.iter().position(|candidate| candidate == ch) {
            remaining.remove(index);
            shared += 1;
        }
    }
    shared
}

#[cfg(test)]
mod tests {
    use super::{
        current_token_boundary_split, current_token_boundary_split_or_repair,
        current_token_repaired_boundary_split, damerau_levenshtein, damerau_levenshtein_bounded,
        internal_char_confusion_preserves_frame, is_single_internal_char_move,
        sparse_internal_omission_count, transition_changed_token_count,
        transition_left_context_changed, typed_damage_geometry_priority,
    };

    #[test]
    fn rolling_damerau_preserves_exact_and_bounded_results() {
        let cases = [
            ("", "", 0),
            ("а", "", 1),
            ("ландо", "ладно", 1),
            ("перезарузка", "перезагрузка", 1),
            ("abcdef", "abcfed", 2),
            ("kitten", "sitting", 3),
        ];
        for (left, right, expected) in cases {
            assert_eq!(damerau_levenshtein(left, right), expected);
            for maximum in 0..=5 {
                assert_eq!(
                    damerau_levenshtein_bounded(left, right, maximum),
                    (expected <= maximum).then_some(expected)
                );
            }
        }
    }

    #[test]
    fn typed_damage_geometry_preserves_operator_strength() {
        assert!(
            typed_damage_geometry_priority("acbd", "abcd")
                > typed_damage_geometry_priority("acbd", "axbd")
        );
        assert!(
            typed_damage_geometry_priority("abde", "abcde")
                > typed_damage_geometry_priority("abde", "abce")
        );
    }

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
        assert_eq!(
            sparse_internal_omission_count("ффетивная", "эффективная"),
            Some(2)
        );
        assert_eq!(sparse_internal_omission_count("спть", "спать"), None);
        assert_eq!(
            sparse_internal_omission_count("переподчаю", "переподключаю"),
            None
        );
    }

    #[test]
    fn single_internal_char_move_is_a_typed_reorder_geometry() {
        assert!(is_single_internal_char_move("ктороый", "который"));
        assert!(is_single_internal_char_move("прдложение", "предложние"));
        assert!(!is_single_internal_char_move("абусед", "абсурд"));
        assert!(!is_single_internal_char_move("давай", "давай"));
    }

    #[test]
    fn internal_char_confusion_preserves_the_word_frame() {
        assert!(internal_char_confusion_preserves_frame(
            "абоенет",
            "абонент"
        ));
        assert!(internal_char_confusion_preserves_frame("ландо", "ладно"));
        assert!(!internal_char_confusion_preserves_frame("абснит", "магнит"));
        assert!(!internal_char_confusion_preserves_frame("абсу", "басу"));
    }

    #[test]
    fn current_token_boundary_split_does_not_touch_left_context() {
        assert!(current_token_boundary_split("тоесть ", "то есть "));
        assert!(current_token_boundary_split(
            "тоесть тоесть ",
            "тоесть то есть "
        ));
        assert!(!current_token_boundary_split(
            "тоесть тоесть ",
            "то есть тоесть "
        ));
        assert!(!current_token_boundary_split("то есть ", "тоесть "));
    }

    #[test]
    fn current_token_repaired_boundary_split_is_bounded_to_last_token() {
        assert!(current_token_repaired_boundary_split(
            "прблематут ",
            "проблема тут "
        ));
        assert!(current_token_boundary_split_or_repair(
            "вотидело ",
            "вот и дело "
        ));
        assert!(current_token_boundary_split_or_repair(
            "самоетоже ",
            "самое тоже "
        ));
        assert!(!current_token_repaired_boundary_split(
            "проблема тут ",
            "прблема тут "
        ));
        assert!(!current_token_repaired_boundary_split(
            "мы прблематут ",
            "проблема мы тут "
        ));
    }
}
