use super::l2::{L2_SURFACE_COMPLETION_CELL, L2_SURFACE_MOTIF_CELL};
use super::lexical_attractor::LEXICAL_ATTRACTOR_CELL;
use super::options::WaveOptions;
use super::pattern_wave::{evaluate_pattern_wave, PATTERN_WAVE_CELL};
use super::signal::{LayerTrace, WaveDecision, WordCandidate};
use super::structural_relation::{evaluate_structural_relation, STRUCTURAL_RELATION_CELL};
use super::PHRASE_FORECAST_CELL;
#[cfg(test)]
use super::SEMANTIC_WORD_SOURCE;
use super::{l3_phrase_gate, llmwave};
use crate::candidate_contract::CandidateOrigin;
use crate::correction_core::TypingErrorClass;
use crate::keyboard::is_cyrillic_letter;
use crate::text_metrics::damerau_levenshtein;
use crate::word_reader::{is_cyrillic_letters_only, last_text_word_slice};

pub(crate) const L3_CONTEXT_FIELD_CELL: &str = "L3ContextField32";

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct L3ContextCandidateReadout {
    pub(crate) text: String,
    pub(crate) source: &'static str,
    pub(crate) disposition: &'static str,
    pub(crate) evidence: bool,
    pub(crate) score: f32,
    pub(crate) sequential_score: f32,
    pub(crate) scene_score: f32,
    pub(crate) support: usize,
    pub(crate) width: usize,
    pub(crate) competition_margin: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct L3ContextFieldReadout {
    pub(crate) context_tokens: usize,
    pub(crate) eligible: bool,
    pub(crate) memory_warm: bool,
    pub(crate) candidates: Vec<L3ContextCandidateReadout>,
}

pub(crate) fn l3_context_field_readout(
    original: &str,
    candidates: &[WordCandidate],
) -> L3ContextFieldReadout {
    let context_tokens = llmwave::tokenize(original).len().saturating_sub(1);
    let memory_warm = super::context_phase::default_memory_is_warm();
    let reports = evaluate_admitted_phrase_candidates(original, candidates, None);
    let candidates = candidates
        .iter()
        .zip(reports)
        .filter_map(|(candidate, report)| {
            let report = report?;
            let disposition = match report.decision {
                l3_phrase_gate::L3PhraseGateDecision::Neutral => "neutral",
                l3_phrase_gate::L3PhraseGateDecision::Support => "support",
                l3_phrase_gate::L3PhraseGateDecision::Suppress => "suppress",
            };
            Some(L3ContextCandidateReadout {
                text: candidate.text.clone(),
                source: candidate.source,
                disposition,
                evidence: report.support > 0
                    || report.sequential_score > 0.0
                    || report.scene_score > 0.0,
                score: report.score,
                sequential_score: report.sequential_score,
                scene_score: report.scene_score,
                support: report.support,
                width: report.width,
                competition_margin: report.competition_margin,
            })
        })
        .collect();
    L3ContextFieldReadout {
        context_tokens,
        eligible: context_tokens >= 1,
        memory_warm,
        candidates,
    }
}

pub fn run_l3(original: &str, candidates: &[WordCandidate]) -> (Vec<LayerTrace>, WaveDecision) {
    run_l3_with_options(original, candidates, &WaveOptions::default())
}

pub fn run_l3_with_options(
    original: &str,
    candidates: &[WordCandidate],
    options: &WaveOptions,
) -> (Vec<LayerTrace>, WaveDecision) {
    run_l3_inner(original, candidates, options, None)
}

fn run_l3_inner(
    original: &str,
    candidates: &[WordCandidate],
    options: &WaveOptions,
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> (Vec<LayerTrace>, WaveDecision) {
    if candidates.is_empty() {
        return (
            vec![LayerTrace {
                name: "L3",
                summary: "no word candidates".to_string(),
            }],
            WaveDecision::Keep {
                reason: "no_candidate",
            },
        );
    }

    let mut traces = Vec::new();
    let technical_keep = candidates
        .iter()
        .find(|candidate| candidate.origin == CandidateOrigin::Technical);
    let phrase_reports =
        if !options.is_enabled(L3_CONTEXT_FIELD_CELL) || options.l3_weight() <= f32::EPSILON {
            vec![None; candidates.len()]
        } else {
            evaluate_admitted_phrase_candidates(original, candidates, phrase_memory)
        };
    let best_readout = best_context_candidate(original, candidates, &phrase_reports);

    if options.is_enabled("TechnicalContextCell32") {
        if let Some(technical) = technical_keep {
            traces.push(LayerTrace {
                name: "TechnicalContextCell32",
                summary: format!(
                    "technical keep energy={:.3} risk={:.3}",
                    technical.energy, technical.risk
                ),
            });
            let protects_whole_tail = technical.text == original.trim_end();
            if protects_whole_tail
                && (best_readout.is_none()
                    || technical.energy
                        >= best_readout
                            .map(|index| candidates[index].energy)
                            .unwrap_or(0.0))
            {
                return (
                    traces,
                    WaveDecision::Veto {
                        reason: "technical_keep",
                    },
                );
            }
        }
    }

    let Some(candidate_index) = best_readout else {
        traces.extend(candidates.iter().zip(&phrase_reports).take(8).filter_map(
            |(candidate, report)| {
                context_candidate_blocker(original, candidate, report.as_ref()).map(|blocker| {
                    LayerTrace {
                        name: "L3ReadoutAdmissionCell32",
                        summary: format!(
                            "candidate source={} text={:?} blocker={blocker}",
                            candidate.source, candidate.text
                        ),
                    }
                })
            },
        ));
        return (
            traces,
            WaveDecision::Keep {
                reason: "no_layout_candidate",
            },
        );
    };
    let candidate = &candidates[candidate_index];

    if short_uppercase_layout_candidate_lacks_phrase_context(
        original,
        &candidate.text,
        candidate.origin,
    ) {
        traces.push(LayerTrace {
            name: "PhraseCell32",
            summary: format!(
                "candidate source={} veto=short_uppercase_layout_without_context",
                candidate.source
            ),
        });
        return (
            traces,
            WaveDecision::Keep {
                reason: "short_uppercase_layout_without_context",
            },
        );
    }

    if short_token_candidate_lacks_phrase_context(original, &candidate.text, candidate.origin) {
        traces.push(LayerTrace {
            name: "PhraseCell32",
            summary: format!(
                "candidate source={} veto=short_layout_without_phrase_context",
                candidate.source
            ),
        });
        return (
            traces,
            WaveDecision::Keep {
                reason: "short_layout_without_phrase_context",
            },
        );
    }

    let structural_report = options
        .is_enabled(STRUCTURAL_RELATION_CELL)
        .then(|| evaluate_structural_relation(original, candidate));
    if let Some(report) = structural_report.as_ref() {
        traces.push(LayerTrace {
            name: STRUCTURAL_RELATION_CELL,
            summary: report.summary(),
        });
        if report.vetoes() {
            return (
                traces,
                WaveDecision::Veto {
                    reason: "structural_relation_veto",
                },
            );
        }
    }

    let pattern_report = options
        .is_enabled(PATTERN_WAVE_CELL)
        .then(|| evaluate_pattern_wave(original, candidate));
    if let Some(report) = pattern_report.as_ref() {
        traces.push(LayerTrace {
            name: PATTERN_WAVE_CELL,
            summary: report.summary(),
        });
        if report.vetoes() {
            return (
                traces,
                WaveDecision::Veto {
                    reason: "pattern_wave_veto",
                },
            );
        }
    }

    let phrase_report = phrase_reports[candidate_index].clone();
    let confidence = adjusted_confidence(
        candidate,
        options,
        pattern_report.as_ref(),
        structural_report.as_ref(),
        phrase_report.as_ref(),
    );
    if options.is_enabled("PhraseCell32") {
        let phrase_suffix = phrase_report
            .as_ref()
            .map(|report| {
                format!(
                    " l3_phrase={} score={:.3} support={} width={}",
                    report.reason, report.score, report.support, report.width
                )
            })
            .unwrap_or_default();
        traces.push(LayerTrace {
            name: "PhraseCell32",
            summary: format!(
                "candidate source={} energy={:.3} risk={:.3} confidence={:.3}{}",
                candidate.source, candidate.energy, candidate.risk, confidence, phrase_suffix
            ),
        });
    }
    if options.is_enabled(PHRASE_FORECAST_CELL) && candidate.source == PHRASE_FORECAST_CELL {
        traces.push(LayerTrace {
            name: PHRASE_FORECAST_CELL,
            summary: format!(
                "forecast={:?} support={}",
                candidate.text,
                candidate.support.join(";")
            ),
        });
    }
    if options.is_enabled("MeshConsensusCell32") {
        traces.push(LayerTrace {
            name: "MeshConsensusCell32",
            summary: mesh_summary(confidence, original, &candidate.text),
        });
    }

    if !options.is_enabled("MeshConsensusCell32") {
        return (
            traces,
            WaveDecision::Keep {
                reason: "mesh_disabled",
            },
        );
    }

    if confidence >= 0.25 && candidate.text != original.trim_end() {
        (
            traces,
            WaveDecision::Suggest {
                text: preserve_trailing_separator(original, &candidate.text),
                confidence,
            },
        )
    } else {
        (
            traces,
            WaveDecision::Keep {
                reason: "low_confidence",
            },
        )
    }
}

fn best_context_candidate(
    original: &str,
    candidates: &[WordCandidate],
    phrase_reports: &[Option<l3_phrase_gate::L3PhraseGateReport>],
) -> Option<usize> {
    let same_script_l2_repair_present = strong_same_script_l2_repair_present(original, candidates);
    let strongest_typed_damage_rank = strongest_typed_damage_operator_rank(original, candidates);
    let sparse_omission_pressure = sparse_omission_lattice_pressure(original, candidates);
    let prefix_completion_pressure = prefix_completion_lattice_pressure(original, candidates);
    let boundary_split_pressure = boundary_split_lattice_pressure(original, candidates);
    let single_missing_repair_present = single_missing_repair_present(original, candidates);
    let single_letter_substitution_present =
        single_letter_substitution_present(original, candidates);
    let repeated_collapse_present = repeated_letter_collapse_present(original, candidates);
    let reference_backed_repair_present =
        reference_backed_typed_repair_present(original, candidates);
    let long_substitution_lattice_is_ambiguous =
        long_single_substitution_lattice_is_ambiguous(original, candidates);
    let nearest_transition = candidates
        .iter()
        .filter(|candidate| candidate_preserves_left_context(original, &candidate.text))
        .filter_map(|candidate| context_transition_distance(original, &candidate.text))
        .filter(|distance| *distance > 0)
        .min();
    let has_local_context_support =
        candidates
            .iter()
            .zip(phrase_reports)
            .any(|(candidate, report)| {
                report.as_ref().is_some_and(|report| {
                    report.decision == l3_phrase_gate::L3PhraseGateDecision::Support
                }) && context_candidate_pre_phrase_blocker(original, candidate).is_none()
                    && context_support_is_transition_local(
                        original,
                        candidate,
                        report.as_ref(),
                        nearest_transition,
                    )
            });
    candidates
        .iter()
        .zip(phrase_reports)
        .enumerate()
        .filter(|(_, (candidate, report))| {
            if same_script_l2_repair_present
                && cross_script_layout_projection(original, candidate)
                && !direct_layout_projection_supported(original, &candidate.text)
            {
                return false;
            }
            if boundary_split_should_yield_to_current_token_repair(original, candidate, candidates)
            {
                return false;
            }
            if long_substitution_lattice_is_ambiguous
                && long_single_substitution_drift_candidate(original, candidate)
            {
                return false;
            }
            if long_substitution_lattice_is_ambiguous
                && broad_l2_stem_drift_candidate(original, candidate)
            {
                return false;
            }
            let report = effective_phrase_report(report.as_ref(), has_local_context_support);
            context_candidate_selection_blocker(
                original,
                candidate,
                report,
                has_local_context_support,
            )
            .is_none()
                && context_support_is_transition_local(
                    original,
                    candidate,
                    report,
                    nearest_transition,
                )
        })
        .fold(
            None,
            |best: Option<(usize, f32)>, (index, (candidate, report))| {
                let report = effective_phrase_report(report.as_ref(), has_local_context_support);
                let score = l3_rank_score(original, candidate, report)
                    + typed_damage_competition_pressure(
                        original,
                        candidate,
                        strongest_typed_damage_rank,
                    )
                    + sparse_omission_prefix_pressure(
                        original,
                        candidate,
                        sparse_omission_pressure,
                    )
                    + prefix_completion_competition_pressure(
                        original,
                        candidate,
                        prefix_completion_pressure,
                    )
                    + boundary_split_competition_pressure(
                        original,
                        candidate,
                        boundary_split_pressure,
                    )
                    + single_missing_repair_competition_pressure(
                        original,
                        candidate,
                        single_missing_repair_present,
                    )
                    + single_letter_substitution_competition_pressure(
                        original,
                        candidate,
                        single_letter_substitution_present,
                    )
                    + repeated_letter_collapse_competition_pressure(
                        candidate,
                        repeated_collapse_present,
                    )
                    + unframed_substitution_competition_pressure(
                        original,
                        candidate,
                        reference_backed_repair_present,
                    );
                match best {
                    Some((best_index, best_score)) if score <= best_score => {
                        Some((best_index, best_score))
                    }
                    _ => Some((index, score)),
                }
            },
        )
        .map(|(index, _score)| index)
}

#[derive(Clone, Copy)]
struct SparseOmissionLatticePressure {
    candidate_count: usize,
    max_prefix_len: usize,
}

#[derive(Clone, Copy)]
struct PrefixCompletionLatticePressure {
    active: bool,
    best_confidence: f32,
}

#[derive(Clone, Copy)]
struct BoundarySplitLatticePressure {
    active: bool,
    best_confidence: f32,
}

fn sparse_omission_lattice_pressure(
    original: &str,
    candidates: &[WordCandidate],
) -> SparseOmissionLatticePressure {
    let Some(original_word) = last_token(original) else {
        return SparseOmissionLatticePressure {
            candidate_count: 0,
            max_prefix_len: 0,
        };
    };
    let original_lower = original_word.to_lowercase();
    let mut candidate_count = 0usize;
    let mut max_prefix_len = 0usize;
    for candidate in candidates.iter().filter(|candidate| {
        candidate_preserves_left_context(original, &candidate.text)
            && missing_material_candidate(original_word, candidate)
    }) {
        let Some(candidate_word) = last_token(&candidate.text) else {
            continue;
        };
        candidate_count += 1;
        max_prefix_len = max_prefix_len.max(crate::text_metrics::common_prefix_char_len(
            &original_lower,
            &candidate_word.to_lowercase(),
        ));
    }
    SparseOmissionLatticePressure {
        candidate_count,
        max_prefix_len,
    }
}

fn prefix_completion_lattice_pressure(
    original: &str,
    candidates: &[WordCandidate],
) -> PrefixCompletionLatticePressure {
    let Some(original_word) = last_token(original) else {
        return PrefixCompletionLatticePressure {
            active: false,
            best_confidence: 0.0,
        };
    };
    let original_lower = original_word.to_lowercase();
    if original_lower.chars().count() < 4 || !is_cyrillic_letters_only(&original_lower) {
        return PrefixCompletionLatticePressure {
            active: false,
            best_confidence: 0.0,
        };
    }
    let best_confidence = candidates
        .iter()
        .filter(|candidate| prefix_completion_candidate(&original_lower, candidate))
        .map(confidence)
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    PrefixCompletionLatticePressure {
        active: best_confidence >= 0.80,
        best_confidence,
    }
}

fn boundary_split_lattice_pressure(
    original: &str,
    candidates: &[WordCandidate],
) -> BoundarySplitLatticePressure {
    let best_confidence = candidates
        .iter()
        .filter(|candidate| verified_current_token_boundary_split_candidate(original, candidate))
        .map(confidence)
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);
    BoundarySplitLatticePressure {
        active: best_confidence >= 0.80,
        best_confidence,
    }
}

fn sparse_omission_prefix_pressure(
    original: &str,
    candidate: &WordCandidate,
    pressure: SparseOmissionLatticePressure,
) -> f32 {
    if pressure.candidate_count < 2 || pressure.max_prefix_len < 4 {
        return 0.0;
    }
    let Some(original_word) = last_token(original) else {
        return 0.0;
    };
    if !missing_material_candidate(original_word, candidate) {
        return 0.0;
    }
    let Some(candidate_word) = last_token(&candidate.text) else {
        return 0.0;
    };
    let prefix = crate::text_metrics::common_prefix_char_len(
        &original_word.to_lowercase(),
        &candidate_word.to_lowercase(),
    );
    if prefix == pressure.max_prefix_len {
        0.12
    } else {
        -0.18
    }
}

fn prefix_completion_competition_pressure(
    original: &str,
    candidate: &WordCandidate,
    pressure: PrefixCompletionLatticePressure,
) -> f32 {
    if !pressure.active {
        return 0.0;
    }
    let Some(original_word) = last_token(original) else {
        return 0.0;
    };
    let original_lower = original_word.to_lowercase();
    if prefix_completion_candidate(&original_lower, candidate) {
        return (0.12 + (pressure.best_confidence - 0.80) * 0.20).clamp(0.12, 0.18);
    }
    if candidate.origin == CandidateOrigin::L2Surface
        && candidate_preserves_left_context(original, &candidate.text)
        && last_token(&candidate.text).is_some_and(|candidate_word| {
            same_script_words(&original_lower, &candidate_word.to_lowercase())
                && !candidate_word.to_lowercase().starts_with(&original_lower)
        })
        && context_transition_distance(original, &candidate.text)
            .is_some_and(|distance| (1..=2).contains(&distance))
    {
        return -0.24;
    }
    0.0
}

fn boundary_split_competition_pressure(
    original: &str,
    candidate: &WordCandidate,
    pressure: BoundarySplitLatticePressure,
) -> f32 {
    if !pressure.active {
        return 0.0;
    }
    if verified_current_token_boundary_split_candidate(original, candidate) {
        return (0.24 + (pressure.best_confidence - 0.80) * 0.20).clamp(0.24, 0.30);
    }
    if candidate.origin == CandidateOrigin::L2Surface
        && candidate_preserves_left_context(original, &candidate.text)
        && context_transition_distance(original, &candidate.text)
            .is_some_and(|distance| distance >= 2)
    {
        return -0.18;
    }
    0.0
}

fn verified_current_token_boundary_split_candidate(
    original: &str,
    candidate: &WordCandidate,
) -> bool {
    current_token_boundary_split_candidate_shape(original, candidate)
        && context_candidate_pre_phrase_blocker(original, candidate).is_none()
}

fn current_token_boundary_split_candidate_shape(original: &str, candidate: &WordCandidate) -> bool {
    candidate.origin == CandidateOrigin::Boundary
        && crate::text_metrics::current_token_boundary_split_or_repair(original, &candidate.text)
}

fn prefix_completion_candidate(original_lower: &str, candidate: &WordCandidate) -> bool {
    if candidate.origin != CandidateOrigin::Completion
        || candidate.source != L2_SURFACE_COMPLETION_CELL
    {
        return false;
    }
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    let candidate_lower = candidate_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let candidate_len = candidate_lower.chars().count();
    candidate_lower.starts_with(original_lower)
        && candidate_len > original_len
        && candidate_len <= original_len + 8
}

fn missing_material_candidate(original_word: &str, candidate: &WordCandidate) -> bool {
    if !matches!(candidate.origin, CandidateOrigin::L2Surface)
        || has_typed_damage_operator_support(candidate)
            && !candidate
                .support
                .iter()
                .any(|item| item == "l2-operator:sparse-internal-multi-omission")
    {
        return false;
    }
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    same_script_words(original_word, candidate_word)
        && missing_material_transition_words(original_word, candidate_word)
}

fn missing_material_transition_words(original_word: &str, candidate_word: &str) -> bool {
    let original_len = original_word.chars().count();
    let candidate_len = candidate_word.chars().count();
    original_len >= 5
        && (2..=4).contains(&candidate_len.saturating_sub(original_len))
        && original_word.chars().next() == candidate_word.chars().next()
        && original_word.chars().last() == candidate_word.chars().last()
        && chars_are_subsequence(original_word, candidate_word)
}

fn suffix_missing_letter_transition_words(original_word: &str, candidate_word: &str) -> bool {
    candidate_word.chars().count() == original_word.chars().count() + 1
        && candidate_word.starts_with(original_word)
}

fn chars_are_subsequence(short: &str, long: &str) -> bool {
    let mut short_chars = short.chars();
    let mut needed = short_chars.next();
    if needed.is_none() {
        return true;
    }
    for ch in long.chars() {
        if Some(ch) == needed {
            needed = short_chars.next();
            if needed.is_none() {
                return true;
            }
        }
    }
    false
}

fn long_single_substitution_lattice_is_ambiguous(
    original: &str,
    candidates: &[WordCandidate],
) -> bool {
    let mut unique = std::collections::BTreeSet::new();
    for candidate in candidates
        .iter()
        .filter(|candidate| long_single_substitution_drift_candidate(original, candidate))
    {
        unique.insert(candidate.text.as_str());
        if unique.len() >= 2 {
            return true;
        }
    }
    false
}

fn long_single_substitution_drift_candidate(original: &str, candidate: &WordCandidate) -> bool {
    if candidate.origin != CandidateOrigin::L2Surface
        || !candidate_preserves_left_context(original, &candidate.text)
        || !candidate
            .support
            .iter()
            .any(|item| item == "l2-operator:single-letter-substitution")
    {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    original_word.chars().count() >= 7
        && original_word.chars().count() == candidate_word.chars().count()
        && same_script_words(original_word, candidate_word)
        && damerau_levenshtein(original_word, candidate_word) == 1
}

fn broad_l2_stem_drift_candidate(original: &str, candidate: &WordCandidate) -> bool {
    if candidate.origin != CandidateOrigin::L2Surface
        || !candidate_preserves_left_context(original, &candidate.text)
        || has_typed_damage_operator_support(candidate)
    {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    let original_lower = original_word.to_lowercase();
    let candidate_lower = candidate_word.to_lowercase();
    original_lower.chars().count() >= 7
        && same_script_words(&original_lower, &candidate_lower)
        && damerau_levenshtein(&original_lower, &candidate_lower) >= 2
        && crate::text_metrics::common_prefix_char_len(&original_lower, &candidate_lower) >= 4
}

fn safe_reorder_typed_damage(candidate: &WordCandidate) -> bool {
    candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:adjacent-transposition")
}

fn safe_orthographic_sign_typed_damage(candidate: &WordCandidate) -> bool {
    candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:orthographic-sign-repair")
}

fn strongest_typed_damage_operator_rank(original: &str, candidates: &[WordCandidate]) -> u8 {
    candidates
        .iter()
        .filter(|candidate| {
            candidate.origin == CandidateOrigin::L2Surface
                && candidate_preserves_left_context(original, &candidate.text)
                && confidence(candidate) >= 0.70
        })
        .filter_map(|candidate| typed_damage_operator_rank_for_transition(original, candidate))
        .max()
        .unwrap_or(0)
}

fn typed_damage_competition_pressure(
    original: &str,
    candidate: &WordCandidate,
    strongest_typed_damage_rank: u8,
) -> f32 {
    if strongest_typed_damage_rank == 0
        || candidate.origin != CandidateOrigin::L2Surface
        || !candidate_preserves_left_context(original, &candidate.text)
    {
        return 0.0;
    }
    let Some(original_word) = last_token(original) else {
        return 0.0;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return 0.0;
    };
    if !same_script_words(original_word, candidate_word) {
        return 0.0;
    }
    let candidate_rank =
        typed_damage_operator_rank_for_transition(original, candidate).unwrap_or(0);
    if candidate_rank < strongest_typed_damage_rank {
        if typed_damage_reference_prior(original, candidate) >= 0.14
            && reference_candidate_preserves_repair_frame(original, candidate)
        {
            return -0.04;
        }
        -0.22
    } else {
        0.0
    }
}

fn reference_candidate_preserves_repair_frame(original: &str, candidate: &WordCandidate) -> bool {
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    if !same_script_words(original_word, candidate_word) {
        return false;
    }
    let original_lower = original_word.to_lowercase();
    let candidate_lower = candidate_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let candidate_len = candidate_lower.chars().count();
    original_lower.chars().last() == candidate_lower.chars().last()
        || (candidate_len > original_len
            && crate::text_metrics::common_prefix_char_len(&original_lower, &candidate_lower) >= 4)
}

fn repeated_letter_collapse_present(original: &str, candidates: &[WordCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.origin == CandidateOrigin::L2Surface
            && candidate_preserves_left_context(original, &candidate.text)
            && candidate
                .support
                .iter()
                .any(|item| item == "l2-operator:repeated-letter-collapse")
    })
}

fn single_missing_repair_present(original: &str, candidates: &[WordCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.origin == CandidateOrigin::L2Surface
            && candidate_preserves_left_context(original, &candidate.text)
            && candidate.support.iter().any(|item| {
                matches!(
                    item.as_str(),
                    "l2-operator:single-internal-missing-letter"
                        | "l2-operator:single-missing-letter"
                )
            })
            && confidence(candidate) >= 0.80
    })
}

fn single_missing_repair_competition_pressure(
    original: &str,
    candidate: &WordCandidate,
    single_missing_present: bool,
) -> f32 {
    if !single_missing_present || candidate.origin != CandidateOrigin::L2Surface {
        return 0.0;
    }
    if candidate.support.iter().any(|item| {
        matches!(
            item.as_str(),
            "l2-operator:single-internal-missing-letter" | "l2-operator:single-missing-letter"
        )
    }) {
        if typed_damage_reference_prior(original, candidate) >= 0.14 {
            return 0.20;
        }
        return 0.10;
    }
    if candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:sparse-internal-multi-omission")
    {
        return -0.16;
    }
    0.0
}

fn reference_backed_typed_repair_present(original: &str, candidates: &[WordCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.origin == CandidateOrigin::L2Surface
            && candidate_preserves_left_context(original, &candidate.text)
            && confidence(candidate) >= 0.60
            && (typed_damage_reference_prior(original, candidate) >= 0.14
                || candidate.support.iter().any(|item| {
                    matches!(
                        item.as_str(),
                        "l2-operator:single-internal-missing-letter"
                            | "l2-operator:single-missing-letter"
                    )
                }) && confidence(candidate) >= 0.88)
    })
}

fn single_letter_substitution_present(original: &str, candidates: &[WordCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.origin == CandidateOrigin::L2Surface
            && candidate_preserves_left_context(original, &candidate.text)
            && candidate
                .support
                .iter()
                .any(|item| item == "l2-operator:single-letter-substitution")
            && inferred_typed_damage_operator_rank(original, candidate)
                .is_some_and(|rank| rank >= 6)
            && confidence(candidate) >= 0.84
    })
}

fn single_letter_substitution_competition_pressure(
    original: &str,
    candidate: &WordCandidate,
    substitution_present: bool,
) -> f32 {
    if !substitution_present || candidate.origin != CandidateOrigin::L2Surface {
        return 0.0;
    }
    if candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:single-letter-substitution")
        && inferred_typed_damage_operator_rank(original, candidate).is_some_and(|rank| rank >= 6)
    {
        return 0.08;
    }
    if candidate.support.iter().any(|item| {
        matches!(
            item.as_str(),
            "l2-operator:single-internal-missing-letter" | "l2-operator:single-missing-letter"
        )
    }) {
        return -0.14;
    }
    0.0
}

fn repeated_letter_collapse_competition_pressure(
    candidate: &WordCandidate,
    collapse_present: bool,
) -> f32 {
    if !collapse_present || candidate.origin != CandidateOrigin::L2Surface {
        return 0.0;
    }
    if candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:repeated-letter-collapse")
    {
        return 0.12;
    }
    if candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:single-missing-letter")
    {
        return -0.14;
    }
    0.0
}

fn unframed_substitution_competition_pressure(
    original: &str,
    candidate: &WordCandidate,
    reference_backed_repair_present: bool,
) -> f32 {
    if !reference_backed_repair_present || candidate.origin != CandidateOrigin::L2Surface {
        return 0.0;
    }
    let weak_reference_substitution = candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:single-letter-substitution")
        && typed_damage_reference_prior(original, candidate) < 0.14;
    if weak_reference_substitution {
        -0.28
    } else {
        0.0
    }
}

fn typed_damage_operator_rank_for_transition(
    original: &str,
    candidate: &WordCandidate,
) -> Option<u8> {
    let support_rank = typed_damage_operator_rank(candidate);
    let shape_rank = inferred_typed_damage_operator_rank(original, candidate);
    support_rank.max(shape_rank)
}

fn inferred_typed_damage_operator_rank(original: &str, candidate: &WordCandidate) -> Option<u8> {
    let original_word = last_token(original)?;
    let candidate_word = last_token(&candidate.text)?;
    if !same_script_words(original_word, candidate_word) {
        return None;
    }
    if crate::text_metrics::sparse_internal_omission_count(original_word, candidate_word).is_some()
    {
        return Some(6);
    }
    if missing_material_transition_words(original_word, candidate_word) {
        return Some(6);
    }
    if crate::text_metrics::is_single_internal_char_move(original_word, candidate_word) {
        return Some(7);
    }
    if crate::text_metrics::internal_char_confusion_preserves_frame(original_word, candidate_word) {
        return Some(6);
    }
    None
}

fn typed_damage_operator_rank(candidate: &WordCandidate) -> Option<u8> {
    candidate
        .support
        .iter()
        .filter_map(|item| {
            let rank = match item.as_str() {
                "l2-operator:adjacent-transposition" => 7,
                "l2-operator:single-internal-missing-letter" => 6,
                "l2-operator:single-missing-letter" => 6,
                "l2-operator:repeated-letter-collapse" => 7,
                "l2-operator:single-letter-substitution" => 5,
                "l2-operator:sparse-internal-multi-omission" => 4,
                "l2-operator:orthographic-sign-repair" => 4,
                "l2-operator:internal-extra-fragment" => 3,
                _ => return None,
            };
            Some(rank)
        })
        .max()
}

fn strong_same_script_l2_repair_present(original: &str, candidates: &[WordCandidate]) -> bool {
    let Some(original_word) = last_token(original) else {
        return false;
    };
    candidates.iter().any(|candidate| {
        if candidate.origin != CandidateOrigin::L2Surface
            || !candidate_preserves_left_context(original, &candidate.text)
            || confidence(candidate) < 0.70
            || context_candidate_pre_phrase_blocker(original, candidate).is_some()
        {
            return false;
        }
        let Some(candidate_word) = last_token(&candidate.text) else {
            return false;
        };
        same_script_words(original_word, candidate_word)
            && context_transition_distance(original, &candidate.text).is_some_and(|distance| {
                (1..=3).contains(&distance) || has_typed_damage_operator_support(candidate)
            })
    })
}

fn cross_script_layout_projection(original: &str, candidate: &WordCandidate) -> bool {
    if !matches!(
        candidate.origin,
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
    ) {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    (is_cyrillic_letters_only(original_word)
        && candidate_word.chars().all(|ch| ch.is_ascii_alphabetic()))
        || (original_word.chars().all(|ch| ch.is_ascii_alphabetic())
            && is_cyrillic_letters_only(candidate_word))
}

fn boundary_split_should_yield_to_current_token_repair(
    original: &str,
    candidate: &WordCandidate,
    candidates: &[WordCandidate],
) -> bool {
    if candidate.origin != CandidateOrigin::Boundary
        || !crate::text_metrics::current_token_boundary_split_or_repair(original, &candidate.text)
    {
        return false;
    }
    let Some((_left, right)) = current_token_boundary_split_parts(original, &candidate.text) else {
        return false;
    };
    typed_current_token_repair_present(original, candidates)
        || boundary_split_masks_repeated_letter_repair(original, &right, candidates)
}

fn current_token_boundary_split_parts(
    original: &str,
    replacement: &str,
) -> Option<(String, String)> {
    let original_tokens = crate::word_reader::normalized_text_words(original);
    let replacement_tokens = crate::word_reader::normalized_text_words(replacement);
    if original_tokens.is_empty() || replacement_tokens.len() != original_tokens.len() + 1 {
        return None;
    }
    let idx = original_tokens.len() - 1;
    if original_tokens[..idx] != replacement_tokens[..idx] {
        return None;
    }
    Some((
        replacement_tokens.get(idx)?.to_string(),
        replacement_tokens.get(idx + 1)?.to_string(),
    ))
}

#[cfg(test)]
fn boundary_split_tail_is_weak(tail: &str) -> bool {
    let lower = tail.to_lowercase();
    !crate::lexicon::is_common_ru_word(&lower)
}

fn typed_current_token_repair_present(original: &str, candidates: &[WordCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.origin == CandidateOrigin::L2Surface
            && candidate.source != LEXICAL_ATTRACTOR_CELL
            && candidate_preserves_left_context(original, &candidate.text)
            && (has_typed_damage_operator_support(candidate)
                || inferred_typed_damage_operator_rank(original, candidate).is_some())
            && typed_damage_operator_rank_for_transition(original, candidate)
                .is_some_and(|rank| rank >= 6)
            && context_transition_distance(original, &candidate.text)
                .is_some_and(|distance| (1..=3).contains(&distance))
    })
}

fn boundary_split_masks_repeated_letter_repair(
    original: &str,
    split_tail: &str,
    candidates: &[WordCandidate],
) -> bool {
    let Some(original_word) = last_token(original) else {
        return false;
    };
    candidates.iter().any(|candidate| {
        candidate.origin == CandidateOrigin::L2Surface
            && candidate_preserves_left_context(original, &candidate.text)
            && candidate
                .support
                .iter()
                .any(|item| item == "l2-operator:repeated-letter-collapse")
            && last_token(&candidate.text).is_some_and(|candidate_word| {
                candidate_word.eq_ignore_ascii_case(split_tail)
                    && crate::typing_transition::action::classify_token_transition(
                        original_word,
                        candidate_word,
                        CandidateOrigin::L2Surface,
                        TypingErrorClass::Unknown,
                    ) == TypingErrorClass::RepeatedLetter
            })
    })
}

fn same_script_words(left: &str, right: &str) -> bool {
    (is_cyrillic_letters_only(left) && is_cyrillic_letters_only(right))
        || (left.chars().all(|ch| ch.is_ascii_alphabetic())
            && right.chars().all(|ch| ch.is_ascii_alphabetic()))
}

fn effective_phrase_report(
    report: Option<&l3_phrase_gate::L3PhraseGateReport>,
    has_local_context_support: bool,
) -> Option<&l3_phrase_gate::L3PhraseGateReport> {
    match report {
        Some(report)
            if report.decision == l3_phrase_gate::L3PhraseGateDecision::Suppress
                && !has_local_context_support =>
        {
            None
        }
        other => other,
    }
}

fn context_support_is_transition_local(
    original: &str,
    candidate: &WordCandidate,
    report: Option<&l3_phrase_gate::L3PhraseGateReport>,
    nearest_transition: Option<usize>,
) -> bool {
    if !report
        .is_some_and(|report| report.decision == l3_phrase_gate::L3PhraseGateDecision::Support)
    {
        return true;
    }
    let Some(nearest) = nearest_transition else {
        return true;
    };
    if current_token_boundary_split_candidate_shape(original, candidate) {
        return true;
    }
    if !candidate_preserves_left_context(original, &candidate.text) {
        return false;
    }
    let Some(distance) = context_transition_distance(original, &candidate.text) else {
        return false;
    };
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return false;
    };
    // Context may complete a prefix or recover omitted material. It may not
    // use one phrase association to support a farther destructive shortening
    // than the local transition field already supports.
    candidate_word.chars().count() >= original_word.chars().count() || distance <= nearest
}

fn context_transition_distance(original: &str, candidate: &str) -> Option<usize> {
    let original = last_token(original)?;
    let candidate = last_token(candidate)?;
    Some(damerau_levenshtein(original, candidate))
}

fn candidate_preserves_left_context(original: &str, candidate: &str) -> bool {
    let original_tokens = llmwave::tokenize(original);
    let candidate_tokens = llmwave::tokenize(candidate);
    let Some((_original_last, original_prefix)) = original_tokens.split_last() else {
        return false;
    };
    let Some((candidate_last, candidate_prefix)) = candidate_tokens.split_last() else {
        return false;
    };
    !candidate_last.is_empty() && original_prefix == candidate_prefix
}

fn context_candidate_blocker(
    original: &str,
    candidate: &WordCandidate,
    phrase_report: Option<&l3_phrase_gate::L3PhraseGateReport>,
) -> Option<&'static str> {
    context_candidate_selection_blocker(original, candidate, phrase_report, true)
}

fn context_candidate_selection_blocker(
    original: &str,
    candidate: &WordCandidate,
    phrase_report: Option<&l3_phrase_gate::L3PhraseGateReport>,
    allow_phrase_suppression: bool,
) -> Option<&'static str> {
    if let Some(reason) = context_candidate_pre_phrase_blocker(original, candidate) {
        return Some(reason);
    }
    if current_token_boundary_split_candidate_shape(original, candidate) {
        return None;
    }
    if allow_phrase_suppression
        && !has_typed_damage_operator_support(candidate)
        && phrase_gate_suppresses(phrase_report)
    {
        return Some("phrase_gate");
    }
    None
}

fn context_candidate_pre_phrase_blocker(
    original: &str,
    candidate: &WordCandidate,
) -> Option<&'static str> {
    let error_class = nanda_candidate_error_class(original, candidate);
    let action = crate::typing_transition::action::verify_action_operator(
        original,
        &candidate.text,
        error_class,
        candidate.origin,
    );
    if let Some(reason) = action.apply_blocker() {
        return Some(reason);
    }
    if semantic_candidate_lacks_surface_support(original, candidate, action.edit_operator) {
        return Some("semantic_surface_authority");
    }
    if word_form_candidate_lacks_surface_support(original, candidate, error_class) {
        return Some("word_form_authority");
    }
    None
}

fn evaluate_admitted_phrase_candidates(
    original: &str,
    candidates: &[WordCandidate],
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> Vec<Option<l3_phrase_gate::L3PhraseGateReport>> {
    if candidates.is_empty() {
        return vec![None; candidates.len()];
    }
    // Context is evidence, not an execution capability.  It must observe the
    // entire L2 lattice before the action/verifier gate decides whether a
    // candidate may mutate text. The phrase adapter itself yields None for a
    // candidate that does not preserve the surrounding context.
    let replacements = candidates
        .iter()
        .map(|candidate| candidate.text.as_str())
        .collect::<Vec<_>>();
    match phrase_memory {
        Some(memory) => {
            l3_phrase_gate::evaluate_candidates_with_memory(original, &replacements, memory)
        }
        None => l3_phrase_gate::evaluate_default_candidates(original, &replacements),
    }
}

fn confidence(candidate: &WordCandidate) -> f32 {
    (candidate.energy - candidate.risk).clamp(0.0, 1.0)
}

fn l3_rank_score(
    original: &str,
    candidate: &WordCandidate,
    phrase_report: Option<&l3_phrase_gate::L3PhraseGateReport>,
) -> f32 {
    let mut value = confidence(candidate);
    value += boundary_operator_coherence(original, candidate);
    value += verified_operator_coherence(original, candidate);
    value += typed_damage_operator_coherence(original, candidate);
    value += typed_damage_reference_prior(original, candidate);
    value += candidate_usage_context_prior(original, &candidate.text);
    if let Some(report) = phrase_report {
        match report.decision {
            l3_phrase_gate::L3PhraseGateDecision::Support => {
                value += (0.18 + report.score * 0.12).clamp(0.18, 0.30);
            }
            l3_phrase_gate::L3PhraseGateDecision::Neutral => {}
            l3_phrase_gate::L3PhraseGateDecision::Suppress => {
                value -= 1.0;
            }
        }
    }
    value.clamp(-1.0, 1.0)
}

fn boundary_operator_coherence(original: &str, candidate: &WordCandidate) -> f32 {
    if candidate.origin != CandidateOrigin::Boundary
        || context_candidate_pre_phrase_blocker(original, candidate).is_some()
    {
        return 0.0;
    }
    0.12
}

fn typed_damage_operator_coherence(original: &str, candidate: &WordCandidate) -> f32 {
    if candidate.origin != CandidateOrigin::L2Surface
        || !candidate_preserves_left_context(original, &candidate.text)
        || context_candidate_pre_phrase_blocker(original, candidate).is_some()
    {
        return 0.0;
    }
    if candidate.support.iter().any(|item| {
        matches!(
            item.as_str(),
            "l2-operator:single-internal-missing-letter" | "l2-operator:single-missing-letter"
        )
    }) {
        return 0.20;
    }
    if candidate.support.iter().any(|item| {
        matches!(
            item.as_str(),
            "l2-operator:repeated-letter-collapse"
                | "l2-operator:internal-extra-fragment"
                | "l2-operator:sparse-internal-multi-omission"
                | "l2-operator:adjacent-transposition"
        )
    }) {
        return 0.18;
    }
    if inferred_typed_damage_operator_rank(original, candidate).is_some() {
        return 0.18;
    }
    if candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:orthographic-sign-repair")
    {
        return 0.10;
    }
    if candidate
        .support
        .iter()
        .any(|item| item == "l2-operator:single-letter-substitution")
    {
        return 0.04;
    }
    0.0
}

fn typed_damage_reference_prior(original: &str, candidate: &WordCandidate) -> f32 {
    if candidate.origin != CandidateOrigin::L2Surface
        || !candidate_preserves_left_context(original, &candidate.text)
        || context_candidate_pre_phrase_blocker(original, candidate).is_some()
    {
        return 0.0;
    }
    let Some(original_word) = last_token(original) else {
        return 0.0;
    };
    let Some(candidate_word) = last_token(&candidate.text) else {
        return 0.0;
    };
    if !same_script_words(original_word, candidate_word) {
        return 0.0;
    }
    if !context_transition_distance(original, &candidate.text).is_some_and(|distance| {
        (1..=4).contains(&distance) || has_typed_damage_operator_support(candidate)
    }) {
        return 0.0;
    }
    lexical_reference_prior(candidate_word)
}

fn lexical_reference_prior(word: &str) -> f32 {
    let lower = word.to_lowercase();
    if crate::lexicon::is_common_ru_word(&lower) {
        return 0.22;
    }
    if let Some(rank) = crate::lexicon::l2_surface_hot_ru_rank(&lower) {
        let hot_strength = 1.0 / (1.0 + rank as f32 / 60.0);
        return (0.10 + hot_strength * 0.18).clamp(0.10, 0.28);
    }
    if let Some(rank) = super::l2::l2_surface_foundation_rank(&lower) {
        let normalized_rank = ((rank as f32).min(20_000.0) / 20_000.0).sqrt();
        return (0.06 + (1.0 - normalized_rank) * 0.16).clamp(0.06, 0.22);
    }
    if crate::russian_lexicon::is_known_russian_word_or_form(&lower) {
        return 0.02;
    }
    0.0
}

fn verified_operator_coherence(original: &str, candidate: &WordCandidate) -> f32 {
    if !matches!(
        candidate.origin,
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
    ) {
        return 0.0;
    }
    let atoms = crate::transition_relation::TransitionRelationAtoms::for_operator(
        original,
        &candidate.text,
        crate::transition_relation::TransitionOperatorKind::LayoutProjection,
    );
    if atoms.verifier_passed() || direct_layout_projection_supported(original, &candidate.text) {
        0.30
    } else {
        -0.60
    }
}

fn direct_layout_projection_supported(original: &str, replacement: &str) -> bool {
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(replacement_word) = last_token(replacement) else {
        return false;
    };
    let replacement_lower = replacement_word.to_lowercase();
    if is_cyrillic_letters_only(original_word) {
        if crate::layout_autoswitch::correct_wrong_layout_cyrillic_word(original_word)
            .is_some_and(|projected| projected.to_lowercase() == replacement_lower)
        {
            return true;
        }
        let projected = crate::dict::convert(original_word, crate::dict::Direction::Ru2Us);
        return projected.eq_ignore_ascii_case(replacement_word)
            && ascii_layout_target_has_authority(&replacement_lower);
    }
    if original_word.chars().all(|ch| ch.is_ascii_alphabetic()) {
        if crate::layout_autoswitch::correct_wrong_layout_ascii_word(original_word)
            .is_some_and(|projected| projected.to_lowercase() == replacement_lower)
        {
            return true;
        }
        let projected = crate::dict::convert(original_word, crate::dict::Direction::Us2Ru);
        return projected.eq_ignore_ascii_case(replacement_word)
            && russian_layout_target_has_authority(&replacement_lower);
    }
    false
}

fn ascii_layout_target_has_authority(word: &str) -> bool {
    crate::lexicon::is_common_en_technical_word(word)
        || crate::layout_autoswitch::is_known_english_layout_autoswitch_word(word)
        || crate::word_recognizer::is_ascii_technical_or_brand_token(word)
}

fn russian_layout_target_has_authority(word: &str) -> bool {
    crate::russian_lexicon::is_known_russian_word_or_form(word)
        || crate::lexicon::is_common_ru_word(word)
}

fn word_form_candidate_lacks_surface_support(
    original: &str,
    candidate: &WordCandidate,
    error_class: TypingErrorClass,
) -> bool {
    if candidate.origin != CandidateOrigin::L2Surface {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return true;
    };
    let Some(replacement_word) = last_token(&candidate.text) else {
        return true;
    };
    if !is_cyrillic_letters_only(original_word) || !is_cyrillic_letters_only(replacement_word) {
        return true;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let typed_damage_operator_support = has_typed_damage_operator_support(candidate);
    let lexical_missing_material_transition = last_token(original)
        .zip(last_token(&candidate.text))
        .is_some_and(|(original_word, candidate_word)| {
            candidate.source == LEXICAL_ATTRACTOR_CELL
                && same_script_words(original_word, candidate_word)
                && missing_material_transition_words(original_word, candidate_word)
                && crate::text_metrics::common_prefix_char_len(
                    &original_word.to_lowercase(),
                    &candidate_word.to_lowercase(),
                ) >= 4
        });
    let typed_damage_transition = (candidate.source != LEXICAL_ATTRACTOR_CELL
        || lexical_missing_material_transition)
        && typed_damage_operator_rank_for_transition(original, candidate).is_some();
    let field = crate::hot_field::HotFieldSnapshot::current();
    let original_surface_known = field.input_surface_readout(&original_lower).is_known();
    let original_dictionary_known =
        crate::russian_lexicon::is_known_russian_word_or_form(&original_lower)
            || crate::russian_lexicon::is_reference_backed_russian_form(&original_lower);
    let original_known = original_surface_known || original_dictionary_known;
    if original_known
        && original_lower != replacement_lower
        && !safe_reorder_typed_damage(candidate)
        && !safe_orthographic_sign_typed_damage(candidate)
        && !suffix_missing_letter_transition_words(&original_lower, &replacement_lower)
        && (original_dictionary_known || !typed_damage_transition)
    {
        return true;
    }
    if long_initial_letter_drift(&original_lower, &replacement_lower) {
        return true;
    }
    if long_suffix_form_drift(&original_lower, &replacement_lower) {
        return true;
    }
    if typed_damage_error_class(error_class)
        && (typed_damage_operator_support || typed_damage_transition)
    {
        return false;
    }
    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1
        || single_adjacent_transposition(&original_lower, &replacement_lower)
        || typed_damage_operator_support
        || typed_damage_transition
    {
        return false;
    }
    let phase_admitted = candidate
        .support
        .iter()
        .any(|item| item.contains("l2-phase:") && item.contains("admitted=true"));
    let prefix = crate::text_metrics::common_prefix_char_len(&original_lower, &replacement_lower);
    if candidate.source == LEXICAL_ATTRACTOR_CELL
        && !typed_damage_operator_support
        && !typed_damage_transition
        && !phase_admitted
        && distance >= 2
        && original_lower.chars().count() >= 7
        && prefix >= 4
    {
        return true;
    }
    if phase_admitted && distance <= 2 && prefix >= 4 {
        return false;
    }
    if bounded_l2_surface_frame_repair_has_authority(
        &original_lower,
        &replacement_lower,
        candidate,
        distance,
        prefix,
        original_dictionary_known,
    ) {
        return false;
    }
    if distance > 2 {
        return true;
    }
    distance == 2
        && (prefix < 4
            || original_lower.chars().count() < 7
            || !crate::lexicon::is_common_ru_word(&replacement_lower))
}

fn bounded_l2_surface_frame_repair_has_authority(
    original: &str,
    replacement: &str,
    candidate: &WordCandidate,
    distance: usize,
    prefix: usize,
    original_dictionary_known: bool,
) -> bool {
    candidate.source == L2_SURFACE_MOTIF_CELL
        && !original_dictionary_known
        && confidence(candidate) >= 0.72
        && distance <= 2
        && prefix >= 2
        && original.chars().count() >= 7
        && replacement.chars().count() >= 7
        && crate::lexicon::is_common_ru_word(replacement)
}

fn long_suffix_form_drift(original: &str, replacement: &str) -> bool {
    let len = original.chars().count();
    len >= 7
        && replacement.chars().count() == len
        && damerau_levenshtein(original, replacement) == 1
        && crate::text_metrics::common_prefix_char_len(original, replacement) + 1 >= len
}

fn long_initial_letter_drift(original: &str, replacement: &str) -> bool {
    let original_chars = original.chars().collect::<Vec<_>>();
    let replacement_chars = replacement.chars().collect::<Vec<_>>();
    original_chars.len() >= 7
        && replacement_chars.len() == original_chars.len()
        && original_chars.first() != replacement_chars.first()
        && original_chars.get(1..) == replacement_chars.get(1..)
}

fn typed_damage_error_class(error_class: TypingErrorClass) -> bool {
    matches!(
        error_class,
        TypingErrorClass::MissingLetter
            | TypingErrorClass::SparseInternalMultiOmission
            | TypingErrorClass::ExtraLetter
            | TypingErrorClass::RepeatedLetter
            | TypingErrorClass::AdjacentTransposition
            | TypingErrorClass::LetterSubstitution
    )
}

fn has_typed_damage_operator_support(candidate: &WordCandidate) -> bool {
    candidate
        .support
        .iter()
        .any(|item| item.starts_with("l2-operator:"))
}

fn nanda_candidate_error_class(original: &str, candidate: &WordCandidate) -> TypingErrorClass {
    crate::typing_transition::action::classify_token_transition(
        original,
        &candidate.text,
        candidate.origin,
        TypingErrorClass::Unknown,
    )
}

fn candidate_usage_context_prior(original: &str, replacement: &str) -> f32 {
    let Some(word) = last_token(replacement) else {
        return 0.0;
    };
    let word = word.to_lowercase();
    if word.is_empty() {
        return 0.0;
    }
    let context = previous_context_tokens(original);
    (super::usage_prior::word_usage_prior_cached(&word)
        + super::usage_prior::context_word_usage_prior_cached(&context, &word))
    .clamp(0.0, 0.20)
}

fn previous_context_tokens(text: &str) -> Vec<String> {
    let mut words = llmwave::tokenize(text);
    words.pop();
    words
}

fn semantic_candidate_lacks_surface_support(
    original: &str,
    candidate: &WordCandidate,
    operator: crate::text_edit::TransitionOperator,
) -> bool {
    if candidate.origin != CandidateOrigin::L3Context {
        return false;
    }
    if !matches!(
        operator,
        crate::text_edit::TransitionOperator::ReplaceCurrentWord
            | crate::text_edit::TransitionOperator::PhraseTokenRepair
    ) {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return true;
    };
    let Some(replacement_word) = last_token(&candidate.text) else {
        return true;
    };
    if !is_cyrillic_letters_only(original_word) || !is_cyrillic_letters_only(replacement_word) {
        return false;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_len = original_lower.chars().count();
    let replacement_len = replacement_lower.chars().count();

    if replacement_len > original_len && replacement_lower.starts_with(&original_lower) {
        return false;
    }

    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1 {
        return false;
    }

    let max_len = original_len.max(replacement_len);
    let prefix = crate::text_metrics::common_prefix_char_len(&original_lower, &replacement_lower);
    let known_replacement =
        crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
            || crate::lexicon::is_common_ru_word(&replacement_lower);
    let known_original = crate::russian_lexicon::is_known_russian_word_or_form(&original_lower);

    if distance >= 2 && original_len == replacement_len {
        return true;
    }
    if distance == 2 && original_len <= 8 && prefix >= 4 && replacement_len <= original_len + 1 {
        return true;
    }
    if distance == 2 && max_len >= 7 && prefix >= 2 && known_replacement {
        return false;
    }
    if distance == 3
        && original_len >= 9
        && max_len >= 10
        && prefix >= 3
        && known_replacement
        && !known_original
    {
        return false;
    }
    true
}

fn phrase_gate_suppresses(report: Option<&l3_phrase_gate::L3PhraseGateReport>) -> bool {
    report.is_some_and(|report| report.decision == l3_phrase_gate::L3PhraseGateDecision::Suppress)
}

fn short_token_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(
        origin,
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
    ) {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(replacement_word) = last_token(replacement) else {
        return false;
    };
    if original_word.chars().count() != 1 || replacement_word.chars().count() != 1 {
        return false;
    }
    let previous_words = original
        .split_whitespace()
        .take(original.split_whitespace().count().saturating_sub(1))
        .collect::<Vec<_>>();
    let has_cyrillic_context = previous_words
        .iter()
        .any(|word| word.chars().any(is_cyrillic_letter));
    let has_ascii_context = previous_words
        .iter()
        .any(|word| word.chars().any(|ch| ch.is_ascii_alphabetic()));
    has_ascii_context && !has_cyrillic_context
}

fn short_uppercase_layout_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    origin: CandidateOrigin,
) -> bool {
    if !matches!(
        origin,
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo
    ) {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return false;
    };
    let Some(replacement_word) = last_token(replacement) else {
        return false;
    };
    if original_word.chars().count() > 2 || replacement_word.chars().count() > 2 {
        return false;
    }
    if !looks_like_uppercase_word(original_word) || !looks_like_uppercase_word(replacement_word) {
        return false;
    }
    !previous_context_has_cyrillic(original)
}

fn looks_like_uppercase_word(token: &str) -> bool {
    let letters = token
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .collect::<Vec<_>>();
    letters.len() >= 2 && letters.iter().all(|ch| ch.is_uppercase())
}

fn previous_context_has_cyrillic(text: &str) -> bool {
    let words = text.split_whitespace().collect::<Vec<_>>();
    words
        .iter()
        .take(words.len().saturating_sub(1))
        .any(|word| word.chars().any(is_cyrillic_letter))
}

fn last_token(text: &str) -> Option<&str> {
    last_text_word_slice(text)
}

fn single_adjacent_transposition(left: &str, right: &str) -> bool {
    let mut left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    if left_chars.len() != right_chars.len() || left_chars.len() < 2 || left_chars == right_chars {
        return false;
    }
    for index in 0..left_chars.len() - 1 {
        left_chars.swap(index, index + 1);
        if left_chars == right_chars {
            return true;
        }
        left_chars.swap(index, index + 1);
    }
    false
}

fn adjusted_confidence(
    candidate: &WordCandidate,
    options: &WaveOptions,
    pattern_report: Option<&super::pattern_wave::PatternWaveReport>,
    structural_report: Option<&super::structural_relation::StructuralRelationReport>,
    phrase_report: Option<&l3_phrase_gate::L3PhraseGateReport>,
) -> f32 {
    let mut value = confidence(candidate);
    if options.is_enabled(PHRASE_FORECAST_CELL) && candidate.source == PHRASE_FORECAST_CELL {
        value += options.scale_l3_delta(0.08);
    }
    if let Some(report) = pattern_report {
        value += options.scale_l3_delta(report.boost());
    }
    if let Some(report) = structural_report {
        value += options.scale_l3_delta(report.boost());
    }
    if let Some(report) = phrase_report {
        match report.decision {
            l3_phrase_gate::L3PhraseGateDecision::Support => {
                value += options.scale_l3_delta(0.10 + report.score * 0.08);
            }
            l3_phrase_gate::L3PhraseGateDecision::Neutral => {}
            l3_phrase_gate::L3PhraseGateDecision::Suppress => {
                value -= options.scale_l3_delta(0.40);
            }
        }
    }
    value.clamp(0.0, 1.0)
}

fn mesh_summary(confidence: f32, original: &str, candidate: &str) -> String {
    let relation = if original.split_whitespace().count() > 1 {
        "phrase"
    } else {
        "word"
    };
    format!("{relation} coherence={confidence:.3} candidate={candidate:?}")
}

fn preserve_trailing_separator(original: &str, candidate: &str) -> String {
    if original.ends_with(' ') && !candidate.ends_with(' ') {
        format!("{candidate} ")
    } else {
        candidate.to_string()
    }
}

#[cfg(test)]
mod tests;
