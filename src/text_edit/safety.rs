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

    let deleted_contains_space = deleted_text.chars().any(char::is_whitespace);
    let inserted_contains_space = inserted_text.chars().any(char::is_whitespace);
    let insertion_splits_word =
        inserted_contains_space && insertion_point_is_inside_word(&original_chars, delete_start);
    let word_count_changed =
        original.split_whitespace().count() != replacement.split_whitespace().count();
    let boundary_changed = word_count_changed
        || deleted_contains_space
        || inserted_contains_space
        || insertion_splits_word;
    let changes_non_last_word = changed_non_last_word(original, replacement);
    let would_touch_words = touched_word_count(&original_chars, delete_start, cursor);

    let boundary_proof = boundary_proof_source(selected_source_id, selected_error_class);
    let layout_phrase = selected_error_class == Some("wrong_layout");
    let semantic_source = matches!(
        selected_source_id,
        Some("SemanticWordCell32" | "L2SurfaceMotifCell32")
    );

    let (allow_apply, reason) = if boundary_changed && !(boundary_proof || layout_phrase) {
        (false, "unsafe_boundary_edit_without_proof")
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
    matches!(
        source_id,
        Some("BoundaryCell32" | "PhraseCell32" | "layout_phrase" | "experimental_layout_en_to_ru")
    ) || matches!(error_class, Some("split-word" | "glued-words"))
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
