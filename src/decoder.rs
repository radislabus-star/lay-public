//! Correction decoder.
//!
//! The decoder is the single place where lay chooses *what* should happen to a
//! buffered text tail. Runtime backends still decide *how* to execute the edit:
//! uinput replay, text insert, an IME bridge, or a future compositor-native
//! replace operation.

#[path = "decoder/edit_plan.rs"]
mod edit_plan;
#[path = "decoder/manual.rs"]
mod manual;
#[path = "decoder/ranked.rs"]
mod ranked;
#[path = "decoder/types.rs"]
mod types;
#[path = "decoder/typing_tail.rs"]
mod typing_tail;

pub use edit_plan::DecoderEditPlan;
pub use manual::{decode_manual_tail, ManualDecodeRequest, ManualDecodeResult};
pub use ranked::{
    choose_ranked_scoped_tail, rank_scoped_tail_candidates, RankedDecoderCandidate,
    RankedDecoderDecision,
};
pub use types::{CorrectionSource, CorrectionTrigger, DecoderAction};

pub use typing_tail::{
    decode_enter_autocorrect_tail, decode_typing_assist_current_tail, decode_typing_assist_tail,
    decode_typing_assist_tail_with_context,
};

#[cfg(test)]
#[path = "decoder_tests.rs"]
mod tests;
