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
    let original_ends_with_space = original
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace);
    let replacement_ends_with_space = replacement
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace);
    if !original_ends_with_space || !replacement_ends_with_space {
        return plan_text_replacement(original, replacement);
    }

    plan_text_replacement_with_options(original, replacement, false)
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
mod tests {
    use super::*;

    #[test]
    fn plans_minimal_two_word_prefix_and_suffix_edits() {
        assert_eq!(
            plan_text_replacement("NEN DOUBLE", "ТУТ DOUBLE"),
            Some(TextReplacement {
                move_left: 7,
                backspaces: 3,
                insert: "ТУТ".to_string(),
                move_right: 7,
            })
        );
        assert_eq!(
            plan_text_replacement("AmoCRM Z тут задача", "AmoCRM Я тут задача"),
            Some(TextReplacement {
                move_left: 11,
                backspaces: 1,
                insert: "Я".to_string(),
                move_right: 11,
            })
        );
    }

    #[test]
    fn committed_tail_plan_reinserts_trailing_space() {
        assert_eq!(
            plan_committed_tail_replacement("double b ", "double и "),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 2,
                insert: "и ".to_string(),
                move_right: 0,
            })
        );
        assert_eq!(
            plan_committed_tail_replacement("чтобы точнр ", "чтобы точно "),
            Some(TextReplacement {
                move_left: 0,
                backspaces: 2,
                insert: "о ".to_string(),
                move_right: 0,
            })
        );
    }

    #[test]
    fn tail_chars_returns_unicode_tail() {
        assert_eq!(tail_chars("привет", 3), "вет");
        assert_eq!(tail_chars("hi", 10), "hi");
    }
}
