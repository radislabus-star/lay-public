//! Minimal text replacement planning shared by desktop frontends.
//!
//! The planner decides which already-typed prefix/suffix can stay on screen and
//! returns the smallest edit needed for replacing the bad middle range.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextReplacement {
    pub move_left: u32,
    pub backspaces: u32,
    pub insert: String,
    pub move_right: u32,
}

pub fn tail_chars(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

pub fn plan_text_replacement(original: &str, replacement: &str) -> Option<TextReplacement> {
    plan_text_replacement_with_options(original, replacement, true)
}

pub fn plan_committed_tail_replacement(
    original: &str,
    replacement: &str,
) -> Option<TextReplacement> {
    if original == replacement {
        return None;
    }

    let original_trailing_ws = original
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();
    let replacement_trailing_ws = replacement
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();

    if original_trailing_ws > 0 && replacement_trailing_ws > 0 {
        let original_len = original.chars().count();
        let replacement_len = replacement.chars().count();
        let original_body: String = original
            .chars()
            .take(original_len - original_trailing_ws)
            .collect();
        let replacement_body: String = replacement
            .chars()
            .take(replacement_len - replacement_trailing_ws)
            .collect();

        let original_body_spaces = original_body
            .chars()
            .filter(|ch| ch.is_whitespace())
            .count();
        let replacement_body_spaces = replacement_body
            .chars()
            .filter(|ch| ch.is_whitespace())
            .count();

        if replacement_body_spaces > original_body_spaces {
            return plan_text_replacement_with_options(original, replacement, true);
        }

        return Some(TextReplacement {
            move_left: original_trailing_ws as u32,
            backspaces: original_len.saturating_sub(original_trailing_ws) as u32,
            insert: replacement_body,
            move_right: original_trailing_ws as u32,
        });
    }

    // For committed non-whitespace boundaries, replace the full observed tail.
    Some(TextReplacement {
        move_left: 0,
        backspaces: original.chars().count() as u32,
        insert: replacement.to_string(),
        move_right: 0,
    })
}

pub fn ensure_committed_tail_spacing(original: &str, mut replacement: String) -> String {
    let Some(original_last) = original.chars().next_back() else {
        return replacement;
    };
    if original_last.is_whitespace()
        && !replacement
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    {
        replacement.push(original_last);
    }
    replacement
}

pub fn offset_replacement_plan_for_cursor(
    plan: &TextReplacement,
    cursor_offset: u32,
) -> TextReplacement {
    if cursor_offset == 0 {
        return plan.clone();
    }

    TextReplacement {
        move_left: plan.move_left.saturating_add(cursor_offset),
        backspaces: plan.backspaces,
        insert: plan.insert.clone(),
        move_right: plan.move_right.saturating_add(cursor_offset),
    }
}

pub fn plan_committed_whitespace_insertions(
    original: &str,
    replacement: &str,
    cursor_offset: u32,
) -> Option<Vec<TextReplacement>> {
    let original_trailing_ws = original
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();
    let replacement_trailing_ws = replacement
        .chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count();

    if original_trailing_ws == 0 || replacement_trailing_ws == 0 {
        return None;
    }

    let original_len = original.chars().count();
    let replacement_len = replacement.chars().count();
    let original_body: Vec<char> = original
        .chars()
        .take(original_len - original_trailing_ws)
        .collect();
    let replacement_body: Vec<char> = replacement
        .chars()
        .take(replacement_len - replacement_trailing_ws)
        .collect();

    let original_compact: String = original_body
        .iter()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    let replacement_compact: String = replacement_body
        .iter()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    if original_compact != replacement_compact {
        return None;
    }

    let mut original_idx = 0usize;
    let mut insert_positions = Vec::new();
    for replacement_ch in replacement_body {
        if replacement_ch.is_whitespace() {
            if original_body
                .get(original_idx)
                .is_some_and(|ch| ch.is_whitespace())
            {
                original_idx += 1;
            } else {
                insert_positions.push(original_idx);
            }
            continue;
        }

        if original_body.get(original_idx) != Some(&replacement_ch) {
            return None;
        }
        original_idx += 1;
    }

    if original_idx != original_body.len() || insert_positions.is_empty() {
        return None;
    }

    insert_positions.sort_unstable_by(|a, b| b.cmp(a));
    let mut inserted_to_right = 0u32;
    let mut plans = Vec::with_capacity(insert_positions.len());
    for position in insert_positions {
        let move_left = (original_len as u32)
            .saturating_add(cursor_offset)
            .saturating_add(inserted_to_right)
            .saturating_sub(position as u32);
        plans.push(TextReplacement {
            move_left,
            backspaces: 0,
            insert: " ".to_string(),
            move_right: move_left,
        });
        inserted_to_right = inserted_to_right.saturating_add(1);
    }
    Some(plans)
}

fn plan_text_replacement_with_options(
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

#[cfg(test)]
#[path = "text_edit_tests.rs"]
mod tests;
