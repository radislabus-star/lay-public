//! Russian typo correction rules for typing assist.
//!
//! This module owns local word-level typo candidates only: missing letters,
//! extra letters, adjacent swaps, repeated letters, vowel confusion and nearby
//! keyboard substitutions. It does not know about daemon runtime or text output.

#[path = "ru_typo/case_rule.rs"]
mod case_rule;
#[path = "ru_typo/coverage.rs"]
mod coverage;
#[path = "ru_typo/extra.rs"]
mod extra;
mod guards;
#[path = "ru_typo/hard_sign.rs"]
mod hard_sign;
mod keyboard;
#[path = "ru_typo/missing.rs"]
mod missing;
#[path = "ru_typo/repeated.rs"]
mod repeated;
#[path = "ru_typo/substitution.rs"]
mod substitution;
#[path = "ru_typo/thresholds.rs"]
mod thresholds;
#[path = "ru_typo/transposition.rs"]
mod transposition;
#[path = "ru_typo/verb.rs"]
mod verb;
#[path = "ru_typo/vowel.rs"]
mod vowel;

pub(crate) use case_rule::correct_cyrillic_word_case;
pub(crate) use coverage::has_plausible_russian_typo_candidate;
pub use extra::{correct_extra_letters, repair_extra_letters_after_layout};
pub(crate) use hard_sign::correct_hard_sign_typo;
pub use keyboard::are_ru_keyboard_neighbors;
pub use missing::correct_missing_letter;
pub(crate) use missing::safe_missing_letter_candidates;
pub(crate) use repeated::correct_repeated_letter;
pub(crate) use substitution::correct_single_letter_substitution;
pub(crate) use transposition::correct_adjacent_transposition;
pub(crate) use verb::correct_verb_ending_confusion;
pub(crate) use vowel::{correct_contextual_past_tense_vowel_confusion, correct_vowel_confusion};
