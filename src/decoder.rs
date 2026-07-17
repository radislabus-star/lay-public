//! Correction decoder.
//!
//! The decoder is the single place where lay chooses *what* should happen to a
//! buffered text tail. Runtime backends still decide *how* to execute the edit:
//! uinput replay, text insert, an IME bridge, or a future compositor-native
//! replace operation.

mod edit_contract;
#[path = "decoder/edit_plan.rs"]
mod edit_plan;
#[path = "decoder/manual.rs"]
mod manual;
#[path = "decoder/types.rs"]
mod types;
#[path = "decoder/typing_tail.rs"]
mod typing_tail;

pub use edit_plan::DecoderEditPlan;
pub use manual::{decode_manual_tail, ManualDecodeRequest, ManualDecodeResult};
pub use types::{CorrectionSource, CorrectionTrigger, DecoderAction};

pub use typing_tail::{
    decode_enter_autocorrect_tail, decode_typing_assist_current_tail, decode_typing_assist_tail,
    decode_typing_assist_tail_with_context,
};

#[cfg(test)]
#[path = "decoder_tests.rs"]
mod tests;
