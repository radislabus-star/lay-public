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

use std::sync::atomic::{AtomicBool, Ordering};

static RUNTIME_ENABLED: AtomicBool = AtomicBool::new(true);

pub use rank::{best_candidate, rank_candidates, rank_candidates_with_language_weight};
pub use token::is_known_text;
pub use types::ScoredCandidate;
pub use warmup::warm_up;

pub fn set_runtime_enabled(enabled: bool) {
    RUNTIME_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn runtime_enabled() -> bool {
    RUNTIME_ENABLED.load(Ordering::Relaxed)
}
