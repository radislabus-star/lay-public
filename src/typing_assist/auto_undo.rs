pub const NANDA_WAVE_RULE_ID: &str = "nanda_wave";

pub fn typing_correction_should_skip_auto_undo(
    rule_id: Option<&str>,
    original: &str,
    replacement: &str,
) -> bool {
    match rule_id {
        Some(id) if typing_rule_should_skip_auto_undo(id) => true,
        Some(NANDA_WAVE_RULE_ID) => pure_alpha_layout_tail(original, replacement),
        _ => false,
    }
}

pub fn typing_rule_should_skip_auto_undo(rule_id: &str) -> bool {
    use crate::typing_rule_graph::ids;
    matches!(
        rule_id,
        ids::FAST_LAYOUT_EN_TO_RU
            | ids::LAYOUT_RU_TO_EN
            | ids::LAYOUT_EN_TO_RU
            | ids::CONTEXTUAL_LAYOUT_EN_TO_RU
            | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU
            | ids::EXPERIMENTAL_LAYOUT_RU_TO_EN
            | ids::LAYOUT_TECHNICAL
    )
}

fn pure_alpha_layout_tail(original: &str, replacement: &str) -> bool {
    let original_words: Vec<_> = original.split_whitespace().collect();
    let replacement_words: Vec<_> = replacement.split_whitespace().collect();
    if original_words.len() != replacement_words.len() {
        return false;
    }
    let mut changed = false;
    for (from, to) in original_words.iter().zip(replacement_words.iter()) {
        if from == to {
            continue;
        }
        if !pure_alpha_layout_word(from, to) {
            return false;
        }
        changed = true;
    }
    changed
}

fn pure_alpha_layout_word(from: &str, to: &str) -> bool {
    if !from.chars().all(|ch| ch.is_alphabetic()) || !to.chars().all(|ch| ch.is_alphabetic()) {
        return false;
    }
    crate::dict::convert(from, crate::dict::Direction::Ru2Us) == to
        || crate::dict::convert(from, crate::dict::Direction::Us2Ru) == to
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_rule_graph::ids;

    #[test]
    fn layout_typing_rules_skip_auto_undo() {
        assert!(typing_rule_should_skip_auto_undo(ids::LAYOUT_RU_TO_EN));
        assert!(typing_rule_should_skip_auto_undo(ids::LAYOUT_EN_TO_RU));
        assert!(typing_rule_should_skip_auto_undo(ids::LAYOUT_TECHNICAL));
    }

    #[test]
    fn non_layout_typing_rules_keep_auto_undo() {
        assert!(!typing_rule_should_skip_auto_undo(ids::SPLIT_WORD_PAIR));
        assert!(!typing_rule_should_skip_auto_undo(ids::MISSING_LETTER));
    }

    #[test]
    fn nanda_wave_layout_words_skip_auto_undo() {
        assert!(typing_correction_should_skip_auto_undo(
            Some(NANDA_WAVE_RULE_ID),
            "как дфн ",
            "как lay "
        ));
        assert!(typing_correction_should_skip_auto_undo(
            Some(NANDA_WAVE_RULE_ID),
            "рядом ашду ",
            "рядом file "
        ));
    }

    #[test]
    fn nanda_wave_mixed_tokens_keep_auto_undo() {
        assert!(!typing_correction_should_skip_auto_undo(
            Some(NANDA_WAVE_RULE_ID),
            "15р-16р ",
            "15h-16h "
        ));
        assert!(!typing_correction_should_skip_auto_undo(
            Some(NANDA_WAVE_RULE_ID),
            "plain ",
            "other "
        ));
    }
}
