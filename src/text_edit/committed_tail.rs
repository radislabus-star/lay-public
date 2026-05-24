use super::diff_plan::plan_text_replacement_with_options;
use super::types::TextReplacement;

pub fn plan_committed_tail_replacement(
    original: &str,
    replacement: &str,
) -> Option<TextReplacement> {
    if original == replacement {
        return None;
    }

    let original_trailing_ws = trailing_whitespace_chars(original);
    let replacement_trailing_ws = trailing_whitespace_chars(replacement);

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

pub fn committed_separator_is_preserved(original: &str, replacement: &str) -> bool {
    let original_last = original.chars().next_back();
    let replacement_last = replacement.chars().next_back();

    match original_last {
        Some(ch) if ch.is_whitespace() => replacement_last == Some(ch),
        _ => true,
    }
}

pub fn plan_committed_whitespace_insertions(
    original: &str,
    replacement: &str,
    cursor_offset: u32,
) -> Option<Vec<TextReplacement>> {
    let original_trailing_ws = trailing_whitespace_chars(original);
    let replacement_trailing_ws = trailing_whitespace_chars(replacement);

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

fn trailing_whitespace_chars(text: &str) -> usize {
    text.chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count()
}
