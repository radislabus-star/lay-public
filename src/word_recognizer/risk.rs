use super::identity::{recognize_token, WordKind, WordScript};

pub fn is_plain_layout_autocorrect_risky(original: &str, replacement: &str) -> bool {
    let original = recognize_token(original);
    let replacement = recognize_token(replacement);

    if original.kind == WordKind::Empty || replacement.kind == WordKind::Empty {
        return true;
    }
    if original.kind == WordKind::CliOption || replacement.kind == WordKind::CliOption {
        return true;
    }
    if original.technical || replacement.technical {
        return false;
    }
    if original.kind == WordKind::MixedScript || replacement.kind == WordKind::MixedScript {
        return false;
    }
    if original.script == WordScript::Cyrillic
        && replacement.script == WordScript::Ascii
        && !original.known_ru
        && replacement.known_en
    {
        return false;
    }

    original.is_plain_word() && replacement.is_plain_word()
}
