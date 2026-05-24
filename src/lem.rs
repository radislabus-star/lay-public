//! Layout Error Metric scorer for already-built correction candidates.
//!
//! LEM does not generate free text. It scores deterministic candidates from the
//! daemon and helps choose the most natural short tail.

mod language;
mod noise;
mod rank;
mod score;
mod token;
mod types;
mod warmup;

pub use rank::{best_candidate, rank_candidates};
pub use token::is_known_text;
pub use types::ScoredCandidate;
pub use warmup::warm_up;
