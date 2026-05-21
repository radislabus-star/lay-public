//! Shared Russian prefix facts used by lexical recognition and phrase reading.

pub(crate) const DERIVATIONAL_PREFIXES: &[&str] = &[
    "анти",
    "вне",
    "внутри",
    "квази",
    "меж",
    "недо",
    "пере",
    "полу",
    "псевдо",
    "сверх",
];

pub(crate) fn is_derivational_prefix_fragment(left: &str, right: &str) -> bool {
    right.chars().count() >= 5 && DERIVATIONAL_PREFIXES.contains(&left)
}
