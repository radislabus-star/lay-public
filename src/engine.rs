//! Platform-neutral correction engine.
//!
//! Desktop adapters provide buffered key events and current policy. The engine
//! decides what should happen, but never performs DBus calls, uinput replay,
//! layout switching, sleeps, logging, or file I/O.

use crate::config::CorrectionEngine;
use crate::decoder::{decode_manual_tail, DecoderAction, DecoderEditPlan, ManualDecodeRequest};
use crate::keyboard::{preferred_layout_for_text, replay_layout_decision, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub struct ManualCorrectionPolicy {
    pub engine: CorrectionEngine,
    pub force_replay: bool,
    pub auto_replace: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ManualCorrectionInput<'a> {
    pub events: &'a [KeyEvent],
    pub original: &'a str,
    pub converted: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualCorrectionDecision {
    pub action: DecoderAction,
    pub edit: Option<DecoderEditPlan>,
    pub replay_target_is_ru: bool,
    pub replay_mixed_layouts: bool,
    pub output_text: String,
    pub output_target_is_ru: bool,
}

pub fn decide_manual_correction(
    input: ManualCorrectionInput<'_>,
    policy: ManualCorrectionPolicy,
) -> ManualCorrectionDecision {
    let replay = replay_layout_decision(input.events);
    let decoded = decode_manual_tail(ManualDecodeRequest {
        events: input.events,
        original: input.original,
        converted: input.converted,
        engine: policy.engine,
        force_replay: policy.force_replay,
        auto_replace: policy.auto_replace,
    });

    let output_text = decoded
        .action
        .replacement_text()
        .unwrap_or(input.converted)
        .to_string();
    let output_target_is_ru = preferred_layout_for_text(&output_text, replay.target_is_ru);

    ManualCorrectionDecision {
        action: decoded.action,
        edit: decoded.edit,
        replay_target_is_ru: replay.target_is_ru,
        replay_mixed_layouts: replay.mixed_layouts,
        output_text,
        output_target_is_ru,
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;

#[cfg(test)]
mod alternating_stress_tests {
    use super::{decide_manual_correction, ManualCorrectionInput, ManualCorrectionPolicy};
    use crate::config::CorrectionEngine;
    use crate::decoder::DecoderAction;
    use crate::keyboard::{
        map_events_to_layout, map_original_events, replay_layout_decision, text_to_key_events,
    };

    #[test]
    fn manual_decoder_uses_ranked_known_ascii_layout_targets() {
        let input = "сегодня цщкв потом ашду дальше еукьштфд здесь ыефегы рядом пше згыр";
        let expected = "сегодня word потом file дальше terminal здесь status рядом git push";
        let events = text_to_key_events(input, false).expect("events");
        let original = map_original_events(&events);
        let replay = replay_layout_decision(&events);
        let converted = map_events_to_layout(&events, replay.target_is_ru);
        let decision = decide_manual_correction(
            ManualCorrectionInput {
                events: &events,
                original: &original,
                converted: &converted,
            },
            ManualCorrectionPolicy {
                engine: CorrectionEngine::Smart,
                force_replay: false,
                auto_replace: true,
            },
        );
        let got = match decision.action {
            DecoderAction::KeepOriginal => original.clone(),
            DecoderAction::ReplayAll => converted.clone(),
            DecoderAction::ReplaceText { replacement, .. } => replacement,
        };

        assert_eq!(
            got, expected,
            "original={original:?} converted={converted:?}"
        );
    }
}
