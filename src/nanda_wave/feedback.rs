use super::options::WaveOptions;
use super::signal::{LayerTrace, WordCandidate};
use crate::candidate_contract::CandidateOrigin;

pub const L3_FEEDBACK_CELL: &str = "L3FeedbackCell32";

#[derive(Debug, Clone, PartialEq)]
pub struct FeedbackAdjustment {
    pub source: &'static str,
    pub(crate) origin: CandidateOrigin,
    pub energy_delta: f32,
    pub risk_delta: f32,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct L3Feedback {
    pub adjustments: Vec<FeedbackAdjustment>,
}

impl L3Feedback {
    pub fn is_empty(&self) -> bool {
        self.adjustments.is_empty()
    }
}

pub fn derive_l3_feedback(
    original: &str,
    candidates: &[WordCandidate],
    options: &WaveOptions,
) -> (Vec<LayerTrace>, L3Feedback) {
    if !options.is_enabled(L3_FEEDBACK_CELL) {
        return (Vec::new(), L3Feedback::default());
    }

    let mut feedback = L3Feedback::default();
    if candidates.iter().any(|candidate| {
        matches!(
            candidate.origin,
            CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
        )
    }) && !looks_like_technical_tail(original)
    {
        feedback.adjustments.push(FeedbackAdjustment {
            source: "LayoutWordCell32",
            origin: CandidateOrigin::Layout,
            energy_delta: options.scale_l3_delta(0.04),
            risk_delta: options.scale_l3_delta(-0.02),
            reason: "layout_mode_supported_by_phrase",
        });
    }
    let trace = (!feedback.is_empty()).then(|| LayerTrace {
        name: L3_FEEDBACK_CELL,
        summary: feedback_summary(&feedback),
    });
    (trace.into_iter().collect(), feedback)
}

pub fn apply_l3_feedback(candidates: &mut [WordCandidate], feedback: &L3Feedback) {
    for candidate in candidates {
        for adjustment in feedback
            .adjustments
            .iter()
            .filter(|adjustment| adjustment.origin == candidate.origin)
        {
            candidate.energy = (candidate.energy + adjustment.energy_delta).clamp(0.0, 1.0);
            candidate.risk = (candidate.risk + adjustment.risk_delta).clamp(0.0, 1.0);
            candidate
                .support
                .push(format!("l3-feedback:{}", adjustment.reason));
        }
    }
}

fn feedback_summary(feedback: &L3Feedback) -> String {
    let adjustments = feedback
        .adjustments
        .iter()
        .map(|item| {
            format!(
                "{} energy_delta={:.3} risk_delta={:.3} reason={}",
                item.source, item.energy_delta, item.risk_delta, item.reason
            )
        })
        .collect::<Vec<_>>();
    format!("adjust=[{}]", adjustments.join("; "))
}

fn looks_like_technical_tail(text: &str) -> bool {
    text.contains("://")
        || text.contains('=')
        || text
            .split_whitespace()
            .any(|token| token.starts_with('-') && token != "-" && token.chars().count() > 1)
}
