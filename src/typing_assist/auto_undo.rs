pub const NANDA_WAVE_RULE_ID: &str = "nanda_wave";

pub fn typing_correction_should_skip_auto_undo(
    _rule_id: Option<&str>,
    _original: &str,
    _replacement: &str,
) -> bool {
    false
}

pub fn typing_rule_should_skip_auto_undo(_rule_id: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typing_rule_graph::ids;

    #[test]
    fn layout_typing_rules_keep_double_shift_auto_undo() {
        assert!(!typing_rule_should_skip_auto_undo("layout_ru_to_en"));
        assert!(!typing_rule_should_skip_auto_undo("layout_en_to_ru"));
        assert!(!typing_rule_should_skip_auto_undo("layout_technical"));
    }

    #[test]
    fn non_layout_typing_rules_still_keep_auto_undo() {
        assert!(!typing_rule_should_skip_auto_undo(ids::SPLIT_WORD_PAIR));
        assert!(!typing_rule_should_skip_auto_undo(ids::MISSING_LETTER));
    }

    #[test]
    fn nanda_wave_layout_words_keep_double_shift_auto_undo() {
        assert!(!typing_correction_should_skip_auto_undo(
            Some(NANDA_WAVE_RULE_ID),
            "как дфн ",
            "как lay "
        ));
        assert!(!typing_correction_should_skip_auto_undo(
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
