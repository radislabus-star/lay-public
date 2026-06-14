//! Candidate ranking for typing assist.
//!
//! Correction rules should generate possible replacements. This module is the
//! narrow place that decides which generated candidate is the safest/best one.

#[path = "typing_candidate/confidence.rs"]
mod confidence;
#[path = "typing_candidate/nanda.rs"]
mod nanda;
#[path = "typing_candidate/ranking.rs"]
mod ranking;
#[path = "typing_candidate/scoring.rs"]
mod scoring;
#[path = "typing_candidate/types.rs"]
mod types;

pub use confidence::classify_typing_confidence;
pub use nanda::{rank_with_microbrain, rank_with_microbrain_trace, rank_with_nanda};
pub use ranking::{choose_typing_candidate, rank_typing_candidates};
pub use scoring::{classify_typing_rule, score_typing_candidate};
pub use types::{
    TypingCandidate, TypingCandidateDecision, TypingCandidateFamily, TypingCandidateScore,
    TypingDecisionConfidence,
};

#[cfg(test)]
#[path = "typing_candidate_tests.rs"]
mod tests;
