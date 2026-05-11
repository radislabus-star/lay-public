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
    // After-space corrections run while the user's typed separator is already
    // present in the target field. Keep that trailing whitespace in place when
    // possible instead of deleting and retyping it; this avoids races where a
    // fast replacement loses the boundary and glues the next word.
    plan_text_replacement(original, replacement)
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

    fn apply_plan(original: &str, plan: &TextReplacement) -> String {
        let mut chars: Vec<char> = original.chars().collect();
        let mut cursor = chars.len().saturating_sub(plan.move_left as usize);
        let delete_start = cursor.saturating_sub(plan.backspaces as usize);
        chars.splice(delete_start..cursor, plan.insert.chars());
        cursor = delete_start + plan.insert.chars().count();
        cursor = (cursor + plan.move_right as usize).min(chars.len());
        chars[..cursor]
            .iter()
            .chain(chars[cursor..].iter())
            .collect()
    }

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
    fn committed_tail_plan_preserves_trailing_space_boundary() {
        assert_eq!(
            plan_committed_tail_replacement("double b ", "double и "),
            Some(TextReplacement {
                move_left: 1,
                backspaces: 1,
                insert: "и".to_string(),
                move_right: 1,
            })
        );
        assert_eq!(
            plan_committed_tail_replacement("чтобы точнр ", "чтобы точно "),
            Some(TextReplacement {
                move_left: 1,
                backspaces: 1,
                insert: "о".to_string(),
                move_right: 1,
            })
        );
    }

    #[test]
    fn committed_tail_sentence_plans_keep_space_with_mixed_language_text() {
        for (original, replacement) in [
            ("пишу README и double b ", "пишу README и double и "),
            ("дальше буду точнр ", "дальше буду точно "),
            ("API работает нормальнр ", "API работает нормально "),
        ] {
            let plan = plan_committed_tail_replacement(original, replacement).expect("replacement");
            assert_eq!(apply_plan(original, &plan), replacement);
            assert_eq!(original.ends_with(' '), replacement.ends_with(' '));
            assert_eq!(plan.move_right, 1, "space boundary must stay on screen");
        }
    }

    #[test]
    fn committed_tail_split_word_plan_inserts_only_missing_space() {
        let plan =
            plan_committed_tail_replacement("чтобыточно ", "чтобы точно ").expect("replacement");

        assert_eq!(
            plan,
            TextReplacement {
                move_left: 6,
                backspaces: 0,
                insert: " ".to_string(),
                move_right: 6,
            }
        );
        assert_eq!(apply_plan("чтобыточно ", &plan), "чтобы точно ");
    }

    #[test]
    fn committed_tail_spacing_is_restored_before_planning() {
        assert_eq!(
            ensure_committed_tail_spacing("double b ", "double и".to_string()),
            "double и "
        );
        assert_eq!(
            ensure_committed_tail_spacing("plain", "plain".to_string()),
            "plain"
        );
    }

    #[test]
    fn tail_chars_returns_unicode_tail() {
        assert_eq!(tail_chars("привет", 3), "вет");
        assert_eq!(tail_chars("hi", 10), "hi");
    }
}
