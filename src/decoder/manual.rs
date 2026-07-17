use crate::config::CorrectionEngine;
use crate::correction::Correction;
use crate::keyboard::KeyEvent;
use crate::typing_assist::{apply_manual_replay_auto_replace, decide_scoped_tail_correction};

use super::edit_plan::DecoderEditPlan;
use super::types::{CorrectionSource, CorrectionTrigger, DecoderAction};

#[derive(Debug, Clone, Copy)]
pub struct ManualDecodeRequest<'a> {
    pub events: &'a [KeyEvent],
    pub original: &'a str,
    pub converted: &'a str,
    pub engine: CorrectionEngine,
    pub force_replay: bool,
    pub auto_replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualDecodeResult {
    pub action: DecoderAction,
    pub edit: Option<DecoderEditPlan>,
}

pub fn decode_manual_tail(request: ManualDecodeRequest<'_>) -> ManualDecodeResult {
    if request.force_replay || request.engine == CorrectionEngine::Replay {
        return maybe_apply_auto_replace(request, DecoderAction::ReplayAll);
    }

    let action = if request.engine == CorrectionEngine::Smart {
        decide_scoped_tail_correction(request.events)
            .filter(|text| !text.trim().is_empty())
            .map(|replacement| DecoderAction::ReplaceText {
                replacement,
                source: CorrectionSource::SmartText,
            })
            .unwrap_or_else(|| {
                correction_to_action(crate::typing_assist::decide_correction(
                    request.original,
                    request.converted,
                    request.engine,
                ))
            })
    } else {
        correction_to_action(crate::typing_assist::decide_correction(
            request.original,
            request.converted,
            request.engine,
        ))
    };

    maybe_apply_auto_replace(request, action)
}

fn maybe_apply_auto_replace(
    request: ManualDecodeRequest<'_>,
    action: DecoderAction,
) -> ManualDecodeResult {
    if !matches!(action, DecoderAction::ReplayAll) || !request.auto_replace {
        return manual_decode_result(request.original, action);
    }

    let Some(replacement) = apply_manual_replay_auto_replace(request.original, request.converted)
    else {
        return manual_decode_result(request.original, action);
    };

    if replacement == request.original
        || replacement == request.converted
        || replacement.trim().is_empty()
    {
        return manual_decode_result(request.original, action);
    }

    manual_decode_result(
        request.original,
        DecoderAction::ReplaceText {
            replacement,
            source: CorrectionSource::AutoReplace,
        },
    )
}

fn manual_decode_result(original: &str, action: DecoderAction) -> ManualDecodeResult {
    let edit = match &action {
        DecoderAction::ReplaceText {
            replacement,
            source,
        } if !replacement.trim().is_empty() => DecoderEditPlan::committed_tail(
            CorrectionTrigger::Manual,
            original,
            replacement,
            *source,
        ),
        DecoderAction::KeepOriginal
        | DecoderAction::ReplayAll
        | DecoderAction::ReplaceText { .. } => None,
    };

    ManualDecodeResult { action, edit }
}

fn correction_to_action(correction: Correction) -> DecoderAction {
    match correction {
        Correction::ReplayAll => DecoderAction::ReplayAll,
        Correction::InsertText(replacement) if replacement.trim().is_empty() => {
            DecoderAction::KeepOriginal
        }
        Correction::InsertText(replacement) => DecoderAction::ReplaceText {
            replacement,
            source: CorrectionSource::SmartText,
        },
    }
}
