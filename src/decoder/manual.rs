use crate::config::CorrectionEngine;
use crate::correction::Correction;
use crate::keyboard::KeyEvent;
use crate::typing_assist::{
    apply_auto_replace, decide_scoped_tail_correction_with_options, ScopedTailOptions,
};

use super::edit_plan::DecoderEditPlan;
use super::ranked::{choose_ranked_scoped_tail, RankedDecoderDecision};
use super::types::{CorrectionSource, CorrectionTrigger, DecoderAction};

#[derive(Debug, Clone, Copy)]
pub struct ManualDecodeRequest<'a> {
    pub events: &'a [KeyEvent],
    pub original: &'a str,
    pub converted: &'a str,
    pub engine: CorrectionEngine,
    pub force_replay: bool,
    pub auto_replace: bool,
    pub scoped_options: ScopedTailOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualDecodeResult {
    pub action: DecoderAction,
    pub edit: Option<DecoderEditPlan>,
    pub ranked: Option<RankedDecoderDecision>,
}

pub fn decode_manual_tail(request: ManualDecodeRequest<'_>) -> ManualDecodeResult {
    if request.force_replay || request.engine == CorrectionEngine::Replay {
        return maybe_apply_auto_replace(request, DecoderAction::ReplayAll, None);
    }

    let mut ranked = None;
    let action = if request.engine == CorrectionEngine::Smart {
        if let Some(decision) = choose_ranked_scoped_tail(request.events, request.scoped_options) {
            ranked = Some(decision.clone());
            DecoderAction::ReplaceText {
                replacement: decision.best.text,
                source: CorrectionSource::SmartText,
            }
        } else {
            decide_scoped_tail_correction_with_options(request.events, request.scoped_options)
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
        }
    } else {
        correction_to_action(crate::typing_assist::decide_correction(
            request.original,
            request.converted,
            request.engine,
        ))
    };

    maybe_apply_auto_replace(request, action, ranked)
}

fn maybe_apply_auto_replace(
    request: ManualDecodeRequest<'_>,
    action: DecoderAction,
    ranked: Option<RankedDecoderDecision>,
) -> ManualDecodeResult {
    if !matches!(action, DecoderAction::ReplayAll) || !request.auto_replace {
        return manual_decode_result(request.original, action, ranked);
    }

    let Some(replacement) = apply_auto_replace(request.original, request.converted) else {
        return manual_decode_result(request.original, action, ranked);
    };

    if replacement == request.original
        || replacement == request.converted
        || replacement.trim().is_empty()
    {
        return manual_decode_result(request.original, action, ranked);
    }

    manual_decode_result(
        request.original,
        DecoderAction::ReplaceText {
            replacement,
            source: CorrectionSource::AutoReplace,
        },
        ranked,
    )
}

fn manual_decode_result(
    original: &str,
    action: DecoderAction,
    ranked: Option<RankedDecoderDecision>,
) -> ManualDecodeResult {
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

    ManualDecodeResult {
        action,
        edit,
        ranked,
    }
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
