//! Word recognition layer for correction safety.
//!
//! This module does not correct text. It classifies a token so higher layers can
//! decide whether an automatic correction is safe enough to apply.

#[path = "word_recognizer/identity.rs"]
mod identity;
#[path = "word_recognizer/lexicon.rs"]
mod lexicon;
#[path = "word_recognizer/risk.rs"]
mod risk;
#[path = "word_recognizer/script.rs"]
mod script;
#[path = "word_recognizer/technical.rs"]
mod technical;

pub use identity::{recognize_token, WordIdentity, WordKind, WordLanguage, WordScript};
pub use risk::{is_plain_layout_autocorrect_risky, is_probably_completed_natural_word};
pub use technical::{
    is_ascii_technical_or_brand_token, is_ascii_technical_token, is_ascii_titlecase_token,
    is_cli_option_token, is_mixed_case_ascii_brand, is_mixed_cyrillic_ascii_alpha_token,
    is_protected_ascii_token, is_upper_ascii_acronym,
};

pub fn warm_up() {
    lexicon::warm_up();
}

#[cfg(test)]
#[path = "word_recognizer_tests.rs"]
mod tests;
