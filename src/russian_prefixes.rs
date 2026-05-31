//! Shared Russian prefix facts used by lexical recognition and phrase reading.

use crate::data_lines::data_lines;

const DERIVATIONAL_PREFIXES_DATA: &str =
    include_str!("../data/lexicon/russian_derivational_prefixes.txt");

pub(crate) fn derivational_prefixes() -> impl Iterator<Item = &'static str> {
    data_lines(DERIVATIONAL_PREFIXES_DATA)
}

pub(crate) fn is_derivational_prefix_fragment(left: &str, right: &str) -> bool {
    right.chars().count() >= 5 && derivational_prefixes().any(|prefix| prefix == left)
}
