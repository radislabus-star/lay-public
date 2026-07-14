//! L2 wave-peak scoring for correction candidates.
//!
//! Candidate producers still describe possible operators, but this layer
//! converts every proposed replacement into a word-center peak. The decision
//! core can then rank peaks instead of trusting a rule id as authority.

use crate::correction_core::TypingErrorClass;
use crate::correction_source_contract::{CandidateOrigin, CorrectionSourceRole};
use crate::text_metrics::{damerau_levenshtein, has_cyrillic, has_latin};
use crate::word_reader::{last_text_word, split_word_punctuation};

use super::l2::{self, L2ImeWordCandidateKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct L2WavePeakScore {
    pub(crate) signal: f32,
    pub(crate) rank_bonus: f32,
    pub(crate) positive_milli: i16,
    pub(crate) negative_milli: i16,
    pub(crate) uncertainty_milli: i16,
    pub(crate) reason: &'static str,
}

pub(crate) fn score_correction_peak(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
    candidate_count: usize,
) -> L2WavePeakScore {
    let original_word = normalized_last_word(original);
    let replacement_word = normalized_last_word(replacement);
    if replacement_word.is_empty() {
        return L2WavePeakScore::neutral("empty_replacement_peak");
    }

    let role = if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::MixedScript
    ) {
        CorrectionSourceRole::Layout
    } else {
        origin.source_role()
    };
    let usage = super::cached_usage_prior_snapshot();
    let context = context_words_before_last(original);
    let usage_prior = usage.word_prior(&replacement_word);
    let context_prior = usage.context_word_prior(&context, &replacement_word);
    let rejected = usage.rejected_word_prior(&replacement_word)
        + usage.context_rejected_word_prior(&context, &replacement_word);
    let center = center_resonance(&context, &original_word, &replacement_word);
    let foundation = foundation_resonance(&replacement_word);
    let layout = layout_resonance(role, &original_word, &replacement_word);
    let operator = operator_resonance(error_class, role);
    let known = known_surface_mass(&replacement_word);

    let mut positive =
        center + foundation + layout + operator + known + usage_prior * 1.30 + context_prior * 1.75;
    if candidate_count >= 2 {
        positive += 0.035;
    }

    let drift_negative = drift_anti_wave(role, &original_word, &replacement_word);
    let mut negative = (rejected * 0.90).clamp(0.0, 0.40);
    negative += drift_negative;
    negative += unknown_surface_penalty(role, &replacement_word);

    let mut uncertainty =
        peak_uncertainty(positive, negative, center, layout, foundation, usage_prior);
    if drift_negative >= 0.80 {
        uncertainty = uncertainty.max(0.55);
    }
    let signal = (positive - negative - uncertainty * 0.12).clamp(-1.0, 1.0);
    let rank_bonus = (signal * 0.28).clamp(-0.22, 0.28);

    L2WavePeakScore {
        signal,
        rank_bonus,
        positive_milli: score_to_milli(positive),
        negative_milli: score_to_milli(negative),
        uncertainty_milli: score_to_milli(uncertainty),
        reason: peak_reason(
            center,
            layout,
            foundation,
            usage_prior,
            context_prior,
            rejected,
        ),
    }
}

pub(crate) fn score_live_completion_peak(
    partial: &str,
    surface: &str,
    structural: f32,
    usage_prior: f32,
    context_prior: f32,
    accepted_count: u32,
    rejected: f32,
) -> L2WavePeakScore {
    let foundation = live_foundation_resonance(surface);
    let known = live_known_surface_mass(surface);
    let prefix_fit = prefix_fit_mass(partial, surface);
    let accepted = accepted_count.min(20) as f32 * 0.030;
    let positive = structural
        + prefix_fit
        + foundation
        + known
        + accepted
        + usage_prior * 1.25
        + context_prior * 1.85;
    let negative = (rejected * 0.90).clamp(0.0, 0.45)
        + live_completion_anti_wave(partial, surface, prefix_fit);
    let uncertainty = peak_uncertainty(
        positive,
        negative,
        structural,
        0.0,
        foundation,
        usage_prior.max(context_prior),
    );
    let signal = (positive - negative - uncertainty * 0.10).clamp(-1.0, 1.0);
    let rank_bonus = (signal * 0.24).clamp(-0.18, 0.24);

    L2WavePeakScore {
        signal,
        rank_bonus,
        positive_milli: score_to_milli(positive),
        negative_milli: score_to_milli(negative),
        uncertainty_milli: score_to_milli(uncertainty),
        reason: live_peak_reason(structural, foundation, usage_prior, context_prior, rejected),
    }
}

impl L2WavePeakScore {
    fn neutral(reason: &'static str) -> Self {
        Self {
            signal: 0.0,
            rank_bonus: 0.0,
            positive_milli: 0,
            negative_milli: 0,
            uncertainty_milli: 1000,
            reason,
        }
    }
}

fn normalized_last_word(text: &str) -> String {
    last_text_word(text)
        .map(|word| {
            let (_, core, _) = split_word_punctuation(&word);
            core.to_lowercase()
        })
        .unwrap_or_default()
}

fn context_words_before_last(text: &str) -> Vec<String> {
    let mut words = text
        .split_whitespace()
        .filter_map(|token| {
            let (_, word, _) = split_word_punctuation(token);
            (!word.is_empty()).then(|| word.to_lowercase())
        })
        .collect::<Vec<_>>();
    words.pop();
    words
}

fn center_resonance(context: &[String], original: &str, replacement: &str) -> f32 {
    if original.chars().count() < 2 || !original.chars().all(crate::keyboard::is_cyrillic_letter) {
        return 0.0;
    }
    let context_prefix = if context.is_empty() {
        String::new()
    } else {
        format!("{} ", context.join(" "))
    };
    l2::ime_l2_word_candidates(&context_prefix, original, 16)
        .into_iter()
        .find(|candidate| candidate.surface == replacement)
        .map(|candidate| {
            let structural = (candidate.score as f32 / 1800.0).clamp(0.0, 0.58);
            let overlap = (candidate.l1_overlap as f32 * 0.018
                + candidate.l2_overlap as f32 * 0.030
                + candidate.motif_overlap as f32 * 0.070)
                .clamp(0.0, 0.22);
            let kind_bonus = match candidate.kind {
                L2ImeWordCandidateKind::Replacement => 0.12,
                L2ImeWordCandidateKind::Completion => 0.06,
            };
            (structural + overlap + kind_bonus).clamp(0.0, 0.86)
        })
        .unwrap_or(0.0)
}

fn foundation_resonance(word: &str) -> f32 {
    l2::l2_surface_foundation_rank(word)
        .map(|rank| match rank {
            0..=999 => 0.30,
            1000..=9_999 => 0.22,
            10_000..=49_999 => 0.12,
            _ => 0.05,
        })
        .unwrap_or(0.0)
}

fn live_foundation_resonance(word: &str) -> f32 {
    if crate::lexicon::is_common_ru_word(word) {
        0.12
    } else {
        0.0
    }
}

fn layout_resonance(role: CorrectionSourceRole, original: &str, replacement: &str) -> f32 {
    if role != CorrectionSourceRole::Layout {
        return 0.0;
    }
    let projected = has_latin(original) != has_latin(replacement)
        || has_cyrillic(original) != has_cyrillic(replacement)
        || crate::russian_lexicon::is_known_russian_word_or_form(replacement)
        || crate::lexicon::is_common_ru_word(replacement);
    if projected {
        0.64
    } else {
        0.42
    }
}

fn operator_resonance(error_class: TypingErrorClass, role: CorrectionSourceRole) -> f32 {
    match role {
        CorrectionSourceRole::Layout => 0.18,
        CorrectionSourceRole::Boundary => 0.16,
        CorrectionSourceRole::DeterministicTypo => 0.12,
        CorrectionSourceRole::L2Surface => 0.10,
        CorrectionSourceRole::L3Context => 0.08,
        CorrectionSourceRole::Completion => 0.06,
        CorrectionSourceRole::Technical | CorrectionSourceRole::Unknown => match error_class {
            TypingErrorClass::WrongLayout | TypingErrorClass::MixedScript => 0.10,
            _ => 0.0,
        },
    }
}

fn known_surface_mass(word: &str) -> f32 {
    if crate::lexicon::is_common_ru_word(word) {
        0.14
    } else if crate::russian_lexicon::is_known_russian_word_or_form(word) {
        0.10
    } else if crate::typing_transition::state::word_has_common_usage_authority(word) {
        0.08
    } else {
        0.0
    }
}

fn live_known_surface_mass(word: &str) -> f32 {
    let authority = crate::hot_field::HotFieldSnapshot::current()
        .word_readout(word)
        .authority;
    match authority {
        crate::hot_field::HotWordAuthority::CommonSurface => 0.14,
        crate::hot_field::HotWordAuthority::L2SurfaceCenter
        | crate::hot_field::HotWordAuthority::L2FormCenter => 0.10,
        crate::hot_field::HotWordAuthority::UserUsage => 0.08,
        crate::hot_field::HotWordAuthority::Unknown => {
            if crate::lexicon::is_ime_hot_ru_word(word) {
                0.10
            } else {
                0.0
            }
        }
    }
}

fn prefix_fit_mass(partial: &str, surface: &str) -> f32 {
    if surface.starts_with(partial) {
        let partial_len = partial.chars().count();
        let surface_len = surface.chars().count();
        if surface_len > partial_len {
            return (0.16 + partial_len.min(8) as f32 * 0.018).clamp(0.0, 0.32);
        }
    }
    0.0
}

fn live_completion_anti_wave(partial: &str, surface: &str, prefix_fit: f32) -> f32 {
    if prefix_fit > 0.0 {
        return 0.0;
    }
    let distance = damerau_levenshtein(partial, surface);
    if partial.chars().count() <= 4 && distance >= 2 {
        0.18
    } else {
        0.08
    }
}

fn drift_anti_wave(role: CorrectionSourceRole, original: &str, replacement: &str) -> f32 {
    if matches!(
        role,
        CorrectionSourceRole::Layout | CorrectionSourceRole::Boundary
    ) {
        return 0.0;
    }
    if original.is_empty() || replacement.is_empty() || original == replacement {
        return 0.0;
    }
    let same_script = original.chars().all(crate::keyboard::is_cyrillic_letter)
        && replacement.chars().all(crate::keyboard::is_cyrillic_letter);
    let both_known = crate::russian_lexicon::is_known_russian_word_or_form(original)
        && crate::russian_lexicon::is_known_russian_word_or_form(replacement);
    if same_script && both_known {
        return 0.92;
    }
    let distance = damerau_levenshtein(original, replacement);
    if same_script && original.chars().count() <= 6 && distance >= 2 {
        return 0.12;
    }
    0.0
}

fn unknown_surface_penalty(role: CorrectionSourceRole, word: &str) -> f32 {
    if matches!(
        role,
        CorrectionSourceRole::Layout | CorrectionSourceRole::Technical
    ) {
        return 0.0;
    }
    if word.chars().all(crate::keyboard::is_cyrillic_letter)
        && !crate::lexicon::is_common_ru_word(word)
        && !crate::russian_lexicon::is_known_russian_word_or_form(word)
        && l2::l2_surface_foundation_rank(word).is_none()
    {
        0.12
    } else {
        0.0
    }
}

fn peak_uncertainty(
    positive: f32,
    negative: f32,
    center: f32,
    layout: f32,
    foundation: f32,
    usage_prior: f32,
) -> f32 {
    let hard_evidence = center.max(layout).max(foundation).max(usage_prior * 2.0);
    let uncertainty = 1.0 - (positive - negative).clamp(0.0, 1.0);
    if hard_evidence >= 0.30 {
        (uncertainty * 0.55).clamp(0.0, 1.0)
    } else {
        uncertainty.clamp(0.0, 1.0)
    }
}

fn peak_reason(
    center: f32,
    layout: f32,
    foundation: f32,
    usage_prior: f32,
    context_prior: f32,
    rejected: f32,
) -> &'static str {
    if rejected >= 0.08 {
        "l2_wave_rejected_anti_peak"
    } else if center >= 0.20 {
        "l2_wave_center_resonance"
    } else if layout >= 0.20 {
        "l2_wave_layout_peak"
    } else if context_prior >= 0.030 {
        "l2_wave_context_usage_peak"
    } else if usage_prior >= 0.030 {
        "l2_wave_usage_peak"
    } else if foundation >= 0.10 {
        "l2_wave_foundation_peak"
    } else {
        "l2_wave_weak_peak"
    }
}

fn live_peak_reason(
    structural: f32,
    foundation: f32,
    usage_prior: f32,
    context_prior: f32,
    rejected: f32,
) -> &'static str {
    if rejected >= 0.08 {
        "l2_wave_live_rejected_anti_peak"
    } else if structural >= 0.28 {
        "l2_wave_live_center_peak"
    } else if context_prior >= 0.030 {
        "l2_wave_live_context_peak"
    } else if usage_prior >= 0.030 {
        "l2_wave_live_usage_peak"
    } else if foundation >= 0.10 {
        "l2_wave_live_foundation_peak"
    } else {
        "l2_wave_live_weak_peak"
    }
}

fn score_to_milli(value: f32) -> i16 {
    (value * 1000.0)
        .round()
        .clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

#[cfg(test)]
mod tests {
    use super::score_correction_peak;
    use crate::correction_core::TypingErrorClass;
    use crate::correction_source_contract::CandidateOrigin;

    #[test]
    fn layout_projection_forms_strong_wave_peak() {
        let score = score_correction_peak(
            "file ljgecnbv ",
            "file допустим ",
            TypingErrorClass::WrongLayout,
            CandidateOrigin::Layout,
            1,
        );

        assert!(score.signal > 0.65, "{score:?}");
        assert_eq!(score.reason, "l2_wave_layout_peak");
    }

    #[test]
    fn unknown_surface_without_center_stays_uncertain() {
        let score = score_correction_peak(
            "мы можем ",
            "мы модем ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
            1,
        );

        assert!(score.uncertainty_milli >= 400, "{score:?}");
        assert!(score.rank_bonus < 0.10, "{score:?}");
    }
}
