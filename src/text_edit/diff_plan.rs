use super::types::TextReplacement;

pub fn tail_chars(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

pub fn plan_text_replacement(original: &str, replacement: &str) -> Option<TextReplacement> {
    plan_text_replacement_with_options(original, replacement, true)
}

pub(super) fn plan_text_replacement_with_options(
    original: &str,
    replacement: &str,
    keep_trailing_whitespace_suffix: bool,
) -> Option<TextReplacement> {
    if original == replacement {
        return None;
    }

    let original_chars: Vec<char> = original.chars().collect();
    let replacement_chars: Vec<char> = replacement.chars().collect();

    let mut prefix = 0;
    while prefix < original_chars.len()
        && prefix < replacement_chars.len()
        && original_chars[prefix] == replacement_chars[prefix]
    {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < original_chars.len().saturating_sub(prefix)
        && suffix < replacement_chars.len().saturating_sub(prefix)
        && original_chars[original_chars.len() - 1 - suffix]
            == replacement_chars[replacement_chars.len() - 1 - suffix]
    {
        if !keep_trailing_whitespace_suffix
            && original_chars[original_chars.len() - 1 - suffix].is_whitespace()
        {
            break;
        }
        suffix += 1;
    }

    let original_end = original_chars.len() - suffix;
    let replacement_end = replacement_chars.len() - suffix;
    let backspaces = original_end.saturating_sub(prefix) as u32;
    let insert: String = replacement_chars[prefix..replacement_end].iter().collect();

    if backspaces == 0 && insert.is_empty() {
        return None;
    }

    Some(TextReplacement {
        move_left: suffix as u32,
        backspaces,
        insert,
        move_right: suffix as u32,
    })
}

pub fn apply_replacement_plan_to_text(original: &str, plan: &TextReplacement) -> String {
    let Some(applied) = try_apply_replacement_plan_to_text(original, plan) else {
        return original.to_string();
    };
    applied
}

fn try_apply_replacement_plan_to_text(original: &str, plan: &TextReplacement) -> Option<String> {
    let mut chars: Vec<char> = original.chars().collect();
    let cursor = checked_cursor(chars.len(), plan)?;
    let delete_start = cursor.checked_sub(plan.backspaces as usize)?;
    chars.splice(delete_start..cursor, plan.insert.chars());
    let cursor = delete_start + plan.insert.chars().count();
    let final_cursor = cursor.checked_add(plan.move_right as usize)?;
    if final_cursor > chars.len() {
        return None;
    }
    Some(chars.into_iter().collect())
}

pub fn replacement_plan_matches(original: &str, replacement: &str, plan: &TextReplacement) -> bool {
    try_apply_replacement_plan_to_text(original, plan).as_deref() == Some(replacement)
}

fn checked_cursor(original_len: usize, plan: &TextReplacement) -> Option<usize> {
    let move_left = plan.move_left as usize;
    if move_left > original_len {
        return None;
    }
    Some(original_len - move_left)
}
