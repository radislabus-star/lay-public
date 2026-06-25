pub(super) fn looks_like_single_prefixed_verb(lower: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "пере", "недо", "пред", "про", "при", "под", "над", "без", "раз", "рас", "воз", "вос",
        "до", "за", "на", "от", "по", "вы", "об",
    ];
    const VERB_TAILS: &[&str] = &[
        "ется",
        "ётся",
        "атся",
        "ятся",
        "уется",
        "ается",
        "яется",
        "ывает",
        "ивает",
        "ешь",
        "ишь",
        "ает",
        "яет",
        "ует",
        "ит",
        "ет",
    ];
    lower.chars().count() >= 8
        && PREFIXES.iter().any(|prefix| lower.starts_with(prefix))
        && VERB_TAILS.iter().any(|tail| lower.ends_with(tail))
}
