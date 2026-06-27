//! Dynamic typing-assist policy.
//!
//! Static config says which rule families are generally allowed. This module
//! makes narrow per-context adjustments when the surrounding text gives a strong
//! signal that a normally risky rule is safe enough for live auto-replace.

#[path = "typing_context/context_window.rs"]
mod context_window;
#[path = "typing_context/layout_signal.rs"]
mod layout_signal;
#[path = "typing_context/pipeline.rs"]
mod pipeline;
#[path = "typing_context/syntax_guard.rs"]
mod syntax_guard;
#[path = "typing_context/tokens.rs"]
mod tokens;

pub use context_window::completed_tail_context;
pub use layout_signal::should_enable_ascii_to_ru_layout;
pub use pipeline::typing_assist_pipeline_for_context;
pub(crate) use syntax_guard::syntax_allows_candidate;

#[cfg(test)]
#[path = "typing_context_tests.rs"]
mod tests;
