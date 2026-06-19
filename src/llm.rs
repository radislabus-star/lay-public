//! Optional local model arbiter for already-built layout candidates.
//!
//! The daemon builds deterministic candidates first. A configured backend can
//! only vote between those candidates with a short A/B answer; invalid answers
//! are ignored. The default backend is `off`. Direct GGUF loading requires the
//! `direct-llm` feature.

mod consensus;
mod hybrid;
mod latin_b_context;
mod token_choice;
mod tokenwise;

use crate::llm_backend::{choose_candidate, Choice};
#[cfg(test)]
pub(crate) use crate::llm_backend::{parse_choice, AnthropicResponse, OpenAiChatResponse};
pub use crate::mixed_script_repair::repair_mixed_script;
pub use consensus::{choose_token_consensus, choose_token_hybrid};
pub use hybrid::convert_hybrid;

#[cfg(test)]
use consensus::choose_token_consensus_with_chooser;
#[cfg(test)]
use tokenwise::choose_mixed_token_candidate;

pub fn convert(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let direction = crate::dict::detect_direction(text);
    let converted = crate::dict::convert(text, direction);
    choose_candidate(text, &converted).map(|choice| match choice {
        Some(Choice::Original) => text.to_string(),
        Some(Choice::Converted) | None => converted,
    })
}

pub fn warm_up() -> Result<(), Box<dyn std::error::Error>> {
    crate::llm_backend::warm_up()
}

pub fn model_backend_enabled() -> bool {
    crate::llm_backend::model_backend_enabled()
}

#[cfg(test)]
#[path = "llm_tests.rs"]
mod tests;
