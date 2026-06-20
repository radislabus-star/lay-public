//! Shared text-correction decision facade.
//!
//! Runtime backends still own output and state. This module only answers one
//! question: should this completed text be replaced, and by which engine?

use crate::config::{CorrectionSafety, TypingAssistRuleConfig};
use crate::nanda_wave::{run_wave_trace, WaveDecision};
use crate::typing_assist::apply_typing_assist_with_pipeline;
use crate::typing_context::typing_assist_pipeline_for_context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionMode {
    DeterministicOnly,
    NandaOnly,
    DeterministicThenNanda,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionDecisionSource {
    Deterministic,
    Nanda,
}

#[derive(Debug, Clone)]
pub struct CorrectionRequest<'a> {
    pub text: &'a str,
    pub auto_replace: bool,
    pub typing_assist: bool,
    pub auto_switch_layout: bool,
    pub correction_safety: CorrectionSafety,
    pub typing_assist_pipeline: &'a [TypingAssistRuleConfig],
    pub nanda_autocorrect: bool,
    pub mode: CorrectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDecision {
    pub replacement: String,
    pub source: CorrectionDecisionSource,
}

pub fn decide_text_correction(req: CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    let decision = match req.mode {
        CorrectionMode::DeterministicOnly => deterministic_text_correction(&req),
        CorrectionMode::NandaOnly => nanda_text_correction(&req),
        CorrectionMode::DeterministicThenNanda => {
            deterministic_text_correction(&req).or_else(|| nanda_text_correction(&req))
        }
    }?;
    safe_replacement(req.text, &decision.replacement).then_some(decision)
}

fn deterministic_text_correction(req: &CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    if !(req.auto_replace || req.typing_assist || req.auto_switch_layout) {
        return None;
    }

    let pipeline = typing_assist_pipeline_for_context(
        req.auto_replace,
        req.correction_safety,
        req.typing_assist_pipeline,
        req.text,
    );
    apply_typing_assist_with_pipeline(req.text, req.auto_switch_layout, &pipeline).map(
        |replacement| CorrectionDecision {
            replacement,
            source: CorrectionDecisionSource::Deterministic,
        },
    )
}

fn nanda_text_correction(req: &CorrectionRequest<'_>) -> Option<CorrectionDecision> {
    if !req.nanda_autocorrect {
        return None;
    }

    match run_wave_trace(req.text).decision {
        WaveDecision::Apply { text, .. } if text != req.text => Some(CorrectionDecision {
            replacement: text,
            source: CorrectionDecisionSource::Nanda,
        }),
        WaveDecision::Apply { .. } | WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
    }
}

fn safe_replacement(original: &str, replacement: &str) -> bool {
    replacement != original && !has_contiguous_mixed_alpha_token(replacement)
}

fn has_contiguous_mixed_alpha_token(text: &str) -> bool {
    text.split_whitespace().any(|token| {
        token
            .split(|ch: char| !ch.is_alphabetic())
            .filter(|segment| !segment.is_empty())
            .any(segment_has_ascii_and_cyrillic)
    })
}

fn segment_has_ascii_and_cyrillic(segment: &str) -> bool {
    let mut has_ascii = false;
    let mut has_cyrillic = false;
    for ch in segment.chars() {
        if ch.is_ascii_alphabetic() {
            has_ascii = true;
        } else if is_cyrillic_alpha(ch) {
            has_cyrillic = true;
        }
        if has_ascii && has_cyrillic {
            return true;
        }
    }
    false
}

fn is_cyrillic_alpha(ch: char) -> bool {
    ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch) || ch == 'ё' || ch == 'Ё'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_typing_assist_pipeline;

    fn request<'a>(
        text: &'a str,
        pipeline: &'a [TypingAssistRuleConfig],
        mode: CorrectionMode,
    ) -> CorrectionRequest<'a> {
        CorrectionRequest {
            text,
            auto_replace: true,
            typing_assist: true,
            auto_switch_layout: true,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: pipeline,
            nanda_autocorrect: true,
            mode,
        }
    }

    #[test]
    fn deterministic_mode_corrects_wrong_layout_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(request(
            "lfdfq ",
            &pipeline,
            CorrectionMode::DeterministicOnly,
        ))
        .unwrap();
        assert_eq!(decision.replacement, "давай ");
        assert_eq!(decision.source, CorrectionDecisionSource::Deterministic);
    }

    #[test]
    fn nanda_mode_corrects_wave_writer_text() {
        let pipeline = default_typing_assist_pipeline();
        let decision =
            decide_text_correction(request("тфтвф ", &pipeline, CorrectionMode::NandaOnly))
                .expect("nanda should produce a layout candidate");
        assert_eq!(decision.replacement, "nanda ");
        assert_eq!(decision.source, CorrectionDecisionSource::Nanda);
    }

    #[test]
    fn disabled_runtime_flags_keep_original() {
        let pipeline = default_typing_assist_pipeline();
        let decision = decide_text_correction(CorrectionRequest {
            text: "lfdfq ",
            auto_replace: false,
            typing_assist: false,
            auto_switch_layout: false,
            correction_safety: CorrectionSafety::Experimental,
            typing_assist_pipeline: &pipeline,
            nanda_autocorrect: false,
            mode: CorrectionMode::DeterministicThenNanda,
        });
        assert_eq!(decision, None);
    }

    #[test]
    fn safe_replacement_rejects_contiguous_mixed_script_words() {
        assert!(!safe_replacement("don ", "dот "));
        assert!(!safe_replacement("ghbdtn ", "gривет "));
        assert!(!safe_replacement("fdnjpfvtyf ", "fавтозамена "));
    }

    #[test]
    fn safe_replacement_allows_mixed_language_tokens_split_by_punctuation() {
        assert!(safe_replacement("QR-rjlf ", "QR-коды "));
        assert!(safe_replacement("html djn ", "html вот "));
    }
}
