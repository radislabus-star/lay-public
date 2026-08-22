//! Stable library front door for live typing clients.
//!
//! Daemon and IME adapters use this module instead of reaching into L2/L3/L4
//! implementation modules. Lab and proof binaries may still inspect the
//! underlying layers directly.

mod runtime;

pub mod candidate;

pub use candidate::{
    is_allowed_visible_completion_suffix, is_ascii_layout_letter_symbol, is_command_like_long_tail,
    phrase_candidate_suffix, preedit_suffix_context_and_word, push_unique_ascii_known_suffix,
    push_unique_suffix, select_ime_candidate_proposals, select_ime_candidate_suffixes,
    should_query_llmwave_phrase_suffix, ImeCandidateProposal, ImeCandidateReadoutRequest,
    ImeCandidateSource,
};
pub use runtime::{
    L11ServiceEnsureReport, LiveCompletionCandidate, LiveCompletionReadout, LiveCompletionRequest,
    LiveCompletionTiming, ObservedSystemTransition, PhraseForecastCandidate, TypingCpu,
    TypingCpuOptions,
};
