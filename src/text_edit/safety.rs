use super::types::TextReplacement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPlanSafetyReport {
    pub deleted_text: String,
    pub inserted_text: String,
    pub deleted_contains_space: bool,
    pub inserted_contains_space: bool,
    pub insertion_splits_word: bool,
    pub word_count_changed: bool,
    pub boundary_changed: bool,
    pub changes_non_last_word: bool,
    pub would_touch_words: usize,
    pub allow_apply: bool,
    pub reason: &'static str,
}

pub fn autocorrect_edit_safety(
    original: &str,
    replacement: &str,
    plan: &TextReplacement,
    selected_source_id: Option<&str>,
    selected_error_class: Option<&str>,
) -> EditPlanSafetyReport {
    let original_chars = original.chars().collect::<Vec<_>>();
    let cursor = original_chars.len().saturating_sub(plan.move_left as usize);
    let delete_start = cursor.saturating_sub(plan.backspaces as usize);
    let deleted_text = original_chars[delete_start..cursor]
        .iter()
        .collect::<String>();
    let inserted_text = plan.insert.clone();

    let original_word_count = original.split_whitespace().count();
    let multiword_original = original_word_count > 1;
    let deleted_contains_space = deleted_text.chars().any(char::is_whitespace);
    let inserted_contains_space = inserted_text.chars().any(char::is_whitespace);
    let deleted_core_contains_space = core_contains_space(&deleted_text);
    let inserted_core_contains_space = core_contains_space(&inserted_text);
    let insertion_splits_word =
        inserted_contains_space && insertion_point_is_inside_word(&original_chars, delete_start);
    let word_count_changed =
        original.split_whitespace().count() != replacement.split_whitespace().count();
    let boundary_changed = word_count_changed
        || deleted_core_contains_space
        || inserted_core_contains_space
        || insertion_splits_word;
    let changes_non_last_word = changed_non_last_word(original, replacement);
    let would_touch_words = touched_word_count(&original_chars, delete_start, cursor);
    let trailing_ws = trailing_whitespace_chars(original);
    let rewrites_inside_committed_tail =
        (plan.backspaces > 0 || !plan.insert.is_empty()) && plan.move_left as usize > trailing_ws;

    let boundary_proof = boundary_proof_source(selected_source_id, selected_error_class);
    let layout_phrase = selected_error_class == Some("wrong_layout");
    let semantic_source = selected_source_id.is_some_and(|source| {
        crate::correction_source_contract::is_l3_context_source(source)
            || crate::correction_source_contract::is_l2_surface_source(source)
    });

    let strong_boundary_shape =
        !boundary_changed || layout_phrase || strong_boundary_edit_shape(original, replacement);

    let (allow_apply, reason) = if multiword_original && !boundary_proof {
        (false, "unsafe_multiword_autocorrect_scope")
    } else if rewrites_inside_committed_tail && !(boundary_proof || layout_phrase) {
        (false, "unsafe_middle_suffix_autocorrect_plan")
    } else if boundary_changed && !(boundary_proof || layout_phrase) {
        (false, "unsafe_boundary_edit_without_proof")
    } else if boundary_changed && !strong_boundary_shape {
        (false, "weak_boundary_edit_shape")
    } else if changes_non_last_word && semantic_source {
        (false, "semantic_multiword_left_context_edit")
    } else {
        (true, "safe_edit_plan")
    };

    EditPlanSafetyReport {
        deleted_text,
        inserted_text,
        deleted_contains_space,
        inserted_contains_space,
        insertion_splits_word,
        word_count_changed,
        boundary_changed,
        changes_non_last_word,
        would_touch_words,
        allow_apply,
        reason,
    }
}

fn boundary_proof_source(source_id: Option<&str>, error_class: Option<&str>) -> bool {
    source_id.is_some_and(|source| {
        crate::correction_source_contract::is_boundary_source(source) || source == "PhraseCell32"
    }) || matches!(error_class, Some("split-word" | "glued-words"))
}

fn strong_boundary_edit_shape(original: &str, replacement: &str) -> bool {
    let original_words = normalized_words(original);
    let replacement_words = normalized_words(replacement);
    if replacement_words.len() != original_words.len().saturating_add(1) {
        return false;
    }

    for idx in 0..original_words.len() {
        if original_words[..idx] != replacement_words[..idx] {
            continue;
        }
        if original_words[idx + 1..] != replacement_words[idx + 2..] {
            continue;
        }
        let original_word = &original_words[idx];
        let left = &replacement_words[idx];
        let right = &replacement_words[idx + 1];
        if confident_split_pair(original_word, left, right, original_words.len() == 1) {
            return true;
        }
    }
    false
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let (_, word, _) = crate::word_reader::split_word_punctuation(token);
            (!word.is_empty()).then(|| word.to_lowercase())
        })
        .collect()
}

fn confident_split_pair(
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
        return (left_one_letter_function && right_known)
            || (right_one_letter_function && left_known)
            || (left_short_function && !left_multi_letter_preposition && right_known)
            || (left_known && right_short_function);
    }

    if original_known || !left_known || !right_known || !single_original_word {
        return false;
    }

    let distance = crate::text_metrics::damerau_levenshtein(original, &joined);
    distance <= 2
        && ((left_len >= 4 && right_len >= 3)
            || (left_short_function && !left_multi_letter_preposition && right_len >= 4)
            || (right_short_function && left_len >= 4))
}

fn insertion_point_is_inside_word(chars: &[char], idx: usize) -> bool {
    if idx == 0 || idx >= chars.len() {
        return false;
    }
    let left = chars.get(idx.saturating_sub(1)).copied();
    let right = chars.get(idx).copied();
    left.is_some_and(|ch| !ch.is_whitespace()) && right.is_some_and(|ch| !ch.is_whitespace())
}

fn changed_non_last_word(original: &str, replacement: &str) -> bool {
    let original_words = original.split_whitespace().collect::<Vec<_>>();
    let replacement_words = replacement.split_whitespace().collect::<Vec<_>>();
    if original_words.len() < 2 || original_words.len() != replacement_words.len() {
        return false;
    }
    original_words[..original_words.len() - 1] != replacement_words[..replacement_words.len() - 1]
}

fn touched_word_count(chars: &[char], start: usize, end: usize) -> usize {
    if start == end {
        return usize::from(insertion_point_is_inside_word(chars, start));
    }
    let mut count = 0usize;
    let mut in_word = false;
    for ch in &chars[start..end] {
        if ch.is_whitespace() {
            in_word = false;
        } else if !in_word {
            count += 1;
            in_word = true;
        }
    }
    count
}

fn trailing_whitespace_chars(text: &str) -> usize {
    text.chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count()
}

fn core_contains_space(text: &str) -> bool {
    text.trim_matches(char::is_whitespace)
        .chars()
        .any(char::is_whitespace)
}
