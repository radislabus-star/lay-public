//! After-space typing-assist pipeline.
//!
//! This module owns only rule ordering and candidate arbitration for completed
//! text passed in by the runtime. Smart manual scope correction can reuse it
//! without depending on the public `typing_assist` facade.

mod candidates;
mod engine;
mod rule_order;
mod types;
mod warmup;

pub use engine::{
    apply_typing_assist, apply_typing_assist_exact, apply_typing_assist_with_pipeline,
    explain_typing_assist_with_pipeline,
};
pub use types::{TypingAssistExplanation, TypingRuleEvaluation};
pub use warmup::{warm_up, warm_up_hot};

#[cfg(test)]
#[path = "typing_pipeline_tests.rs"]
mod tests;
