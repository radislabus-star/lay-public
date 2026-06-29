use lay::keyboard::{is_cyrillic_letter, preferred_layout_for_text};

pub(crate) fn force_target_layout_for_replacement(original: &str, replacement: &str) -> bool {
    let original = script_signal(original);
    let replacement = script_signal(replacement);
    matches!(
        (original, replacement),
        (ScriptSignal::Ascii, ScriptSignal::Cyrillic)
            | (ScriptSignal::Cyrillic, ScriptSignal::Ascii)
            | (ScriptSignal::Mixed, ScriptSignal::Ascii)
            | (ScriptSignal::Mixed, ScriptSignal::Cyrillic)
    )
}

pub(crate) fn target_layout_for_replacement(replacement: &str, fallback_is_ru: bool) -> bool {
    preferred_layout_for_text(replacement, fallback_is_ru)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScriptSignal {
    None,
    Ascii,
    Cyrillic,
    Mixed,
}

fn script_signal(text: &str) -> ScriptSignal {
    let has_ascii = text.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_cyrillic = text.chars().any(is_cyrillic_letter);
    match (has_ascii, has_cyrillic) {
        (false, false) => ScriptSignal::None,
        (true, false) => ScriptSignal::Ascii,
        (false, true) => ScriptSignal::Cyrillic,
        (true, true) => ScriptSignal::Mixed,
    }
}

#[cfg(test)]
mod tests {
    use super::{force_target_layout_for_replacement, target_layout_for_replacement};

    #[test]
    fn ascii_to_cyrillic_replacement_forces_target_layout() {
        assert!(force_target_layout_for_replacement("ghbdtn ", "привет "));
        assert!(force_target_layout_for_replacement("djn ", "вот "));
        assert!(target_layout_for_replacement("привет ", false));
    }

    #[test]
    fn cyrillic_to_ascii_replacement_forces_target_layout() {
        assert!(force_target_layout_for_replacement("ашду ", "file "));
        assert!(!target_layout_for_replacement("file ", true));
    }

    #[test]
    fn same_script_typo_fix_does_not_force_layout() {
        assert!(!force_target_layout_for_replacement(
            "посмотерть ",
            "посмотреть "
        ));
        assert!(!force_target_layout_for_replacement("api ", "api "));
    }

    #[test]
    fn mixed_to_clean_script_replacement_forces_target_layout() {
        assert!(force_target_layout_for_replacement("gривет ", "привет "));
        assert!(force_target_layout_for_replacement("аpply ", "apply "));
    }

    #[test]
    fn mixed_replacements_do_not_force_layout_for_still_mixed_text() {
        assert!(!force_target_layout_for_replacement("djn api ", "вот api "));
    }
}
