//! Unified candidate admission for live typing.
//!
//! L1/L2/L3 may produce many plausible surfaces. This module decides which of
//! them is allowed to become a live IME completion. It does not output text and
//! does not apply edits.

use crate::keyboard::is_cyrillic_letter;

use super::l2::{self, L2ImeWordCandidateKind};

#[derive(Debug, Clone, PartialEq)]
pub struct LiveCompletionRequest<'a> {
    pub context_prefix: &'a str,
    pub partial: &'a str,
    pub max_suffix_chars: usize,
    pub allow_short_lexical: bool,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveCompletionCandidate {
    pub surface: String,
    pub suffix: String,
    pub score: f32,
    pub source: &'static str,
}

pub fn live_completion_candidates(
    request: LiveCompletionRequest<'_>,
) -> Vec<LiveCompletionCandidate> {
    if request.limit == 0 || request.max_suffix_chars == 0 {
        return Vec::new();
    }
    let partial = request.partial.to_lowercase();
    let partial_len = partial.chars().count();
    if !(2..=18).contains(&partial_len) || !partial.chars().all(is_cyrillic_letter) {
        return Vec::new();
    }

    let mut raw = if partial_len <= 4 {
        l2::ime_l2_short_seed_word_candidates(
            request.context_prefix,
            &partial,
            request.limit.saturating_mul(3).max(request.limit),
        )
    } else {
        Vec::new()
    };
    raw.extend(l2::ime_l2_word_candidates(
        request.context_prefix,
        &partial,
        request.limit.saturating_mul(4).max(request.limit),
    ));
    let has_completion = raw
        .iter()
        .any(|candidate| candidate.kind == L2ImeWordCandidateKind::Completion);
    let has_strong_replacement = raw.iter().any(|candidate| {
        candidate.kind == L2ImeWordCandidateKind::Replacement && candidate.score >= 650
    });
    if !has_completion && !has_strong_replacement {
        raw.extend(l2::ime_l2_foundation_prefix_candidates(
            request.context_prefix,
            &partial,
            request.limit.saturating_mul(4).max(request.limit),
        ));
    }
    raw.extend(l2::ime_l2_generated_form_prefix_candidates(
        request.context_prefix,
        &partial,
        request.limit.saturating_mul(4).max(request.limit),
    ));

    let context_tokens = super::llmwave::tokenize(request.context_prefix);
    let mut candidates = raw
        .into_iter()
        .filter(|candidate| candidate.kind == L2ImeWordCandidateKind::Completion)
        .filter_map(|candidate| {
            let suffix = candidate.surface.strip_prefix(&partial)?.to_string();
            let suffix_len = suffix.chars().count();
            if suffix.is_empty() || suffix_len > request.max_suffix_chars {
                return None;
            }
            let usage = super::usage_prior::word_usage_prior_cached(&candidate.surface);
            let context_usage = super::usage_prior::context_word_usage_prior_cached(
                &context_tokens,
                &candidate.surface,
            );
            let accepted = super::usage_prior::accepted_word_usage_count_cached(&candidate.surface);
            let common = crate::lexicon::is_common_ru_word(&candidate.surface);
            let hot = crate::lexicon::is_ime_hot_ru_word(&candidate.surface);
            let structural = structural_support(
                candidate.score,
                candidate.l1_overlap,
                candidate.l2_overlap,
                candidate.motif_overlap,
            );

            if !live_completion_has_authority(LiveCompletionAuthority {
                partial_len,
                suffix_len,
                allow_short_lexical: request.allow_short_lexical,
                structural,
                usage,
                context_usage,
                accepted,
                common,
                hot,
            }) {
                return None;
            }

            let score = (0.22
                + structural
                + usage * 1.40
                + context_usage * 1.85
                + (accepted.min(12) as f32 * 0.012)
                + if common { 0.055 } else { 0.0 }
                + if hot { 0.045 } else { 0.0 }
                + (partial_len.min(8) as f32 * 0.018))
                .clamp(0.0, 1.0);

            Some(LiveCompletionCandidate {
                surface: candidate.surface,
                suffix,
                score,
                source: "L2LiveCandidateGate32",
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| {
                left.suffix
                    .chars()
                    .count()
                    .cmp(&right.suffix.chars().count())
            })
            .then_with(|| left.surface.cmp(&right.surface))
    });
    candidates.dedup_by(|left, right| left.surface == right.surface || left.suffix == right.suffix);
    candidates.truncate(request.limit);
    candidates
}

#[derive(Debug, Clone, Copy)]
struct LiveCompletionAuthority {
    partial_len: usize,
    suffix_len: usize,
    allow_short_lexical: bool,
    structural: f32,
    usage: f32,
    context_usage: f32,
    accepted: u32,
    common: bool,
    hot: bool,
}

fn live_completion_has_authority(input: LiveCompletionAuthority) -> bool {
    let usage_signal = input.usage >= 0.035 || input.context_usage >= 0.025 || input.accepted >= 2;
    let lexical_signal = input.common || input.hot;
    let structural_signal = input.structural >= 0.34;

    if input.partial_len <= 2 {
        return input.allow_short_lexical
            && (usage_signal || input.context_usage >= 0.018 || input.hot || input.common)
            && input.suffix_len <= 8;
    }
    if input.partial_len == 3 {
        return usage_signal
            || structural_signal
            || (input.allow_short_lexical && lexical_signal && input.suffix_len <= 7);
    }
    if input.partial_len == 4 {
        return usage_signal || structural_signal || lexical_signal;
    }
    usage_signal || structural_signal || lexical_signal
}

fn structural_support(
    score: u32,
    l1_overlap: usize,
    l2_overlap: usize,
    motif_overlap: usize,
) -> f32 {
    let score_part = (score as f32 / 1600.0).clamp(0.0, 0.45);
    let l1_part = (l1_overlap.min(10) as f32) * 0.012;
    let l2_part = (l2_overlap.min(8) as f32) * 0.025;
    let motif_part = (motif_overlap.min(4) as f32) * 0.055;
    (score_part + l1_part + l2_part + motif_part).clamp(0.0, 0.72)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(context_prefix: &'a str, partial: &'a str) -> LiveCompletionRequest<'a> {
        LiveCompletionRequest {
            context_prefix,
            partial,
            max_suffix_chars: 24,
            allow_short_lexical: true,
            limit: 12,
        }
    }

    #[test]
    fn live_gate_returns_prefix_preserving_candidates() {
        let candidates = live_completion_candidates(request("я хочу ", "пров"));
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface.starts_with("пров")),
            "live IME must only show prefix-preserving completions: {candidates:?}"
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.surface.starts_with("провер")),
            "expected useful провер* candidates, got {candidates:?}"
        );
    }

    #[test]
    fn live_gate_keeps_replacement_out_of_suffix_lane() {
        let candidates = live_completion_candidates(request("", "звгрузи"));
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.surface != "загрузи"),
            "replacement must wait for boundary autocorrect, not live suffix: {candidates:?}"
        );
    }

    #[test]
    fn live_gate_prefers_exact_prefix_continuation_from_foundation() {
        let candidates = live_completion_candidates(request("как будто нет ", "кандидат"));
        assert!(
            candidates
                .first()
                .is_some_and(|candidate| candidate.surface.starts_with("кандидат")),
            "кандидат should get foundation continuations first, got {candidates:?}"
        );
    }
}
