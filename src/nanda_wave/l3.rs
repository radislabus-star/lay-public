use super::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
#[cfg(test)]
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
    let replacements = candidates
        .iter()
        .map(|candidate| candidate.text.as_str())
        .collect::<Vec<_>>();
    let phrase_reports = match phrase_memory {
        Some(memory) => {
            l3_phrase_gate::evaluate_candidates_with_memory(original, &replacements, memory)
        }
        None => l3_phrase_gate::evaluate_default_candidates(original, &replacements),
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
        original,
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
    candidates
        .iter()
        .zip(phrase_reports)
        .enumerate()
        .filter(|(_, (candidate, report))| {
            context_candidate_blocker(original, candidate, report.as_ref()).is_none()
        })
        .fold(
            None,
            |best: Option<(usize, f32)>, (index, (candidate, report))| {
                let score = l3_rank_score(original, candidate, report.as_ref());
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

fn context_candidate_blocker(
    original: &str,
    candidate: &WordCandidate,
    phrase_report: Option<&l3_phrase_gate::L3PhraseGateReport>,
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
    if word_form_candidate_lacks_surface_support(original, candidate) {
        return Some("word_form_authority");
    }
    if candidate_l4_signed_memory_vetoes_apply(original, candidate) {
        return Some("l4_signed_memory");
    }
    if phrase_gate_suppresses(phrase_report) {
        return Some("phrase_gate");
    }
    None
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
    value += verified_operator_coherence(original, candidate);
    value += candidate_usage_context_prior(original, &candidate.text);
    value += candidate_l4_signed_bias(original, candidate);
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
    if atoms.verifier_passed() {
        0.30
    } else {
        -0.60
    }
}

fn word_form_candidate_lacks_surface_support(original: &str, candidate: &WordCandidate) -> bool {
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
    if candidate_l4_signed_signal(original, candidate).is_some_and(|signal| {
        signal.transition_state_specific
            && signal.transition_attract_count > signal.transition_repel_count
    }) {
        return false;
    }
    let field = crate::hot_field::HotFieldSnapshot::current();
    let original_known = field.stable_form_readout(&original_lower).is_known();
    if original_known && original_lower != replacement_lower {
        return true;
    }
    let distance = damerau_levenshtein(&original_lower, &replacement_lower);
    if distance <= 1 || single_adjacent_transposition(&original_lower, &replacement_lower) {
        return false;
    }
    let phase_admitted = candidate
        .support
        .iter()
        .any(|item| item.contains("l2-phase:") && item.contains("admitted=true"));
    let prefix = crate::text_metrics::common_prefix_char_len(&original_lower, &replacement_lower);
    if phase_admitted && distance <= 2 && prefix >= 4 {
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
    original: &str,
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
    value += candidate_l4_signed_bias(original, candidate);
    value.clamp(0.0, 1.0)
}

fn candidate_l4_signed_bias(original: &str, candidate: &WordCandidate) -> f32 {
    let Some(signal) = candidate_l4_signed_signal(original, candidate) else {
        return 0.0;
    };
    l4_signed_bias(&signal)
}

fn candidate_l4_signed_memory_vetoes_apply(original: &str, candidate: &WordCandidate) -> bool {
    let Some(signal) = candidate_l4_signed_signal(original, candidate) else {
        return false;
    };
    l4_signed_signal_vetoes(&signal)
}

fn l4_signed_bias(signal: &super::l4_signed_memory::L4SignedMemorySignal) -> f32 {
    let global = signal.signed_weight * 0.060;
    let transition = if signal.transition_state_specific {
        (signal.transition_attraction - signal.transition_repulsion) * 0.90
    } else {
        0.0
    };
    (global + transition).clamp(-0.20, 0.20)
}

fn l4_signed_signal_vetoes(signal: &super::l4_signed_memory::L4SignedMemorySignal) -> bool {
    if signal.transition_state_specific
        && signal.transition_attract_count > signal.transition_repel_count
    {
        return false;
    }
    let rejected_more_than_accepted = signal.rejected > signal.accepted;
    let transition_repel = signal.transition_repel_count > signal.transition_attract_count
        && signal.transition_repel_count > 0;
    rejected_more_than_accepted
        && signal.signed_weight < 0.0
        && (signal.rejected >= 2 || signal.repulsion >= 0.045 || transition_repel)
}

fn candidate_l4_signed_signal(
    original: &str,
    candidate: &WordCandidate,
) -> Option<super::l4_signed_memory::L4SignedMemorySignal> {
    last_token(&candidate.text)?;
    let context = crate::typing_memory::transition_context_words(original, &candidate.text);
    let transition_target = crate::typing_memory::transition_target_text(original, &candidate.text);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    let state_id = crate::transition_relation::transition_state_id(original);
    Some(l4_signed_memory_signal(L4SignedMemoryInput {
        context: &context,
        source: candidate.origin.memory_key(),
        operation: candidate_operation(candidate.origin),
        state_word: &state_id,
        candidate_text: &transition_target,
        usage: &usage,
        surface: None,
    }))
}

fn candidate_operation(origin: CandidateOrigin) -> &'static str {
    match origin {
        CandidateOrigin::Layout | CandidateOrigin::LayoutThenTypo => "layout",
        CandidateOrigin::Boundary => "boundary",
        CandidateOrigin::Completion => "completion",
        CandidateOrigin::L2Surface
        | CandidateOrigin::L3Context
        | CandidateOrigin::DeterministicTypo
        | CandidateOrigin::Technical => "replacement",
    }
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
mod tests {
    use super::*;

    fn l4_signal() -> super::super::l4_signed_memory::L4SignedMemorySignal {
        super::super::l4_signed_memory::L4SignedMemorySignal {
            attraction: 0.10,
            repulsion: 0.18,
            signed_weight: -0.08,
            accepted: 6,
            rejected: 8,
            transition_attraction: 0.10,
            transition_repulsion: 0.0,
            transition_attract_count: 6,
            transition_repel_count: 0,
            transition_state_specific: true,
            reason: super::super::l4_signed_memory::L4SignedMemoryReason::TransitionAttracts,
            surface_status: super::super::l4_signed_memory::L4SurfaceStatus::Covered,
        }
    }

    #[test]
    fn exact_transition_acceptance_overrides_global_word_rejection() {
        let signal = l4_signal();

        assert!(!l4_signed_signal_vetoes(&signal));
        assert!(l4_signed_bias(&signal) > 0.0);
    }

    #[test]
    fn exact_transition_rejection_still_vetoes() {
        let mut signal = l4_signal();
        signal.transition_attraction = 0.0;
        signal.transition_repulsion = 0.12;
        signal.transition_attract_count = 0;
        signal.transition_repel_count = 8;
        signal.signed_weight = -0.20;

        assert!(l4_signed_signal_vetoes(&signal));
        assert!(l4_signed_bias(&signal) < 0.0);
    }

    #[test]
    fn punctuation_does_not_hide_word_form_authority() {
        let candidate = WordCandidate {
            text: " отстранилась!".to_string(),
            origin: CandidateOrigin::L2Surface,
            source: LEXICAL_ATTRACTOR_CELL,
            energy: 0.95,
            risk: 0.10,
            support: vec![],
        };

        assert!(!word_form_candidate_lacks_surface_support(
            " отсранилась! ",
            &candidate,
        ));
    }

    #[test]
    fn applies_confident_layout_candidate() {
        let candidate = WordCandidate {
            text: "html вот".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.8,
            risk: 0.1,
            support: vec![],
        };
        let (_trace, decision) = run_l3("html djn ", &[candidate]);
        assert_eq!(decision.output(), Some("html вот "));
    }

    #[test]
    fn applies_verified_single_word_layout_candidate() {
        let candidate = WordCandidate {
            text: "проверь".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 1.0,
            risk: 0.05,
            support: vec![],
        };

        let (_trace, decision) = run_l3("ghjdthm", &[candidate]);

        assert_eq!(decision.output(), Some("проверь"));
    }

    #[test]
    fn l3_ranks_short_prefix_completion_without_granting_apply_authority() {
        let candidates = [
            WordCandidate {
                text: "Ну давай".to_string(),
                origin: CandidateOrigin::Completion,
                source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
                energy: 0.856,
                risk: 0.060,
                support: vec![],
            },
            WordCandidate {
                text: "Ну даша".to_string(),
                origin: CandidateOrigin::Completion,
                source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
                energy: 0.856,
                risk: 0.060,
                support: vec![],
            },
        ];

        let (_trace, decision) = run_l3("Ну да ", &candidates);

        assert_eq!(decision.output(), Some("Ну давай "));
    }

    #[test]
    fn l3_keeps_first_equal_candidate_instead_of_last_equal_candidate() {
        let candidates = [
            WordCandidate {
                text: "попаданий".to_string(),
                origin: CandidateOrigin::L3Context,
                source: SEMANTIC_WORD_SOURCE,
                energy: 0.80,
                risk: 0.10,
                support: vec![],
            },
            WordCandidate {
                text: "попадали".to_string(),
                origin: CandidateOrigin::L3Context,
                source: SEMANTIC_WORD_SOURCE,
                energy: 0.80,
                risk: 0.10,
                support: vec![],
            },
        ];

        let (_trace, decision) = run_l3("попадани ", &candidates);

        assert_eq!(decision.output(), Some("попаданий "));
    }

    #[test]
    fn l3_exposes_surface_completion_as_non_mutating_readout() {
        let candidate = WordCandidate {
            text: "попаданий".to_string(),
            origin: CandidateOrigin::Completion,
            source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
            energy: 0.856,
            risk: 0.060,
            support: vec![],
        };

        let (_trace, decision) = run_l3("попадани ", &[candidate]);

        assert_eq!(decision.output(), Some("попаданий "));
    }

    #[test]
    fn keeps_short_layout_candidate_without_russian_phrase_context() {
        let candidate = WordCandidate {
            text: "wave и".to_string(),
            origin: CandidateOrigin::Layout,
            source: "ShortTokenCell32",
            energy: 0.9,
            risk: 0.46,
            support: vec![],
        };
        let (_trace, decision) = run_l3("wave b ", &[candidate]);

        assert_eq!(decision.output(), None);
        assert_eq!(
            decision,
            WaveDecision::Keep {
                reason: "short_layout_without_phrase_context"
            }
        );
    }

    #[test]
    fn l3_weight_scales_structural_boosts() {
        let candidate = WordCandidate {
            text: "html вот".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.32,
            risk: 0.10,
            support: vec![],
        };
        let (_trace, muted) = run_l3_with_options(
            "html djn ",
            std::slice::from_ref(&candidate),
            &WaveOptions::default().with_layer_weights(1.0, 0.0),
        );
        let (_trace, normal) =
            run_l3_with_options("html djn ", &[candidate], &WaveOptions::default());

        assert_ne!(muted, normal);
    }

    #[test]
    fn applies_boundary_candidate() {
        let candidate = WordCandidate {
            text: "у нас есть".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "BoundaryCell32",
            energy: 0.8,
            risk: 0.1,
            support: vec![],
        };
        let (_trace, decision) = run_l3("у насесть ", &[candidate]);
        assert_eq!(decision.output(), Some("у нас есть "));
    }

    #[test]
    fn technical_candidate_vetoes_layout_candidate() {
        let technical = WordCandidate {
            text: "git checkout -b new".to_string(),
            origin: CandidateOrigin::Technical,
            source: "TechTokenCell32",
            energy: 0.95,
            risk: 0.02,
            support: vec![],
        };
        let layout = WordCandidate {
            text: "git checkout -b туц".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.8,
            risk: 0.1,
            support: vec![],
        };
        let (_trace, decision) = run_l3("git checkout -b new ", &[technical, layout]);
        assert_eq!(decision.output(), None);
    }

    #[test]
    fn technical_tail_does_not_veto_phrase_layout_candidate() {
        let technical = WordCandidate {
            text: "api".to_string(),
            origin: CandidateOrigin::Technical,
            source: "TechTokenCell32",
            energy: 0.95,
            risk: 0.02,
            support: vec![],
        };
        let layout = WordCandidate {
            text: "html вот api".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.8,
            risk: 0.1,
            support: vec![],
        };
        let (_trace, decision) = run_l3("html djn api ", &[technical, layout]);
        assert_eq!(decision.output(), Some("html вот api "));
    }

    #[test]
    fn applies_split_memory_candidate() {
        let candidate = WordCandidate {
            text: "она есть".to_string(),
            origin: CandidateOrigin::Boundary,
            source: "PhraseMemoryCell32",
            energy: 0.82,
            risk: 0.11,
            support: vec![],
        };
        let (_trace, decision) = run_l3("онаесть ", &[candidate]);
        assert_eq!(decision.output(), Some("она есть "));
    }

    #[test]
    fn phrase_forecast_boosts_semantic_candidate() {
        let candidate = WordCandidate {
            text: "На улице опять идёт дождь".to_string(),
            origin: CandidateOrigin::Completion,
            source: PHRASE_FORECAST_CELL,
            energy: 0.30,
            risk: 0.10,
            support: vec![],
        };
        let (_trace, decision) = run_l3("На улице опять идёт д ", &[candidate]);
        assert_eq!(decision.output(), Some("На улице опять идёт дождь "));
    }

    #[test]
    fn semantic_word_candidate_needs_surface_authority() {
        let candidate = WordCandidate {
            text: "она спрашивая".to_string(),
            origin: CandidateOrigin::L3Context,
            source: SEMANTIC_WORD_SOURCE,
            energy: 0.90,
            risk: 0.10,
            support: vec![],
        };
        let (_trace, decision) = run_l3("она спраивтя ", &[candidate]);

        assert_eq!(decision.output(), None);
    }

    #[test]
    fn semantic_word_completion_keeps_l3_authority() {
        let candidate = WordCandidate {
            text: "на улице опять идёт дождь".to_string(),
            origin: CandidateOrigin::L3Context,
            source: SEMANTIC_WORD_SOURCE,
            energy: 0.50,
            risk: 0.10,
            support: vec![],
        };
        let (_trace, decision) = run_l3("на улице опять идёт д ", &[candidate]);

        assert_eq!(decision.output(), Some("на улице опять идёт дождь "));
    }

    #[test]
    fn l3_phrase_memory_reranks_competing_l2_candidates() {
        let memory = llmwave::LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nсегодня на улице опять идёт дождь\nвечером на улице опять идёт дождь\nзавтра на улице опять идёт дождь",
        );
        let candidates = vec![
            WordCandidate {
                text: "на улице опять идёт дом".to_string(),
                origin: CandidateOrigin::L3Context,
                source: SEMANTIC_WORD_SOURCE,
                energy: 0.86,
                risk: 0.06,
                support: vec![],
            },
            WordCandidate {
                text: "на улице опять идёт дождь".to_string(),
                origin: CandidateOrigin::L3Context,
                source: SEMANTIC_WORD_SOURCE,
                energy: 0.42,
                risk: 0.08,
                support: vec![],
            },
        ];
        let (trace, decision) = run_l3_inner(
            "на улице опять идёт д ",
            &candidates,
            &WaveOptions::default(),
            Some(&memory),
        );

        assert_eq!(decision.output(), Some("на улице опять идёт дождь "));
        assert!(trace
            .iter()
            .any(|item| item.summary.contains("l3_phrase=l3_context_field_support")));
    }

    #[test]
    fn pattern_wave_is_visible_in_l3_trace() {
        let candidate = WordCandidate {
            text: "html вот api".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.55,
            risk: 0.25,
            support: vec![],
        };
        let (trace, decision) = run_l3("html djn api ", &[candidate]);

        assert!(trace
            .iter()
            .any(|item| item.name == super::super::pattern_wave::PATTERN_WAVE_CELL));
        assert_eq!(decision.output(), Some("html вот api "));
    }

    #[test]
    fn pattern_wave_can_veto_layout_candidate_in_technical_shape() {
        let candidate = WordCandidate {
            text: "git вот".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.95,
            risk: 0.05,
            support: vec![],
        };
        let (_trace, decision) = run_l3("git djn ", &[candidate]);

        assert_eq!(decision.output(), None);
    }

    #[test]
    fn structural_relation_is_visible_in_l3_trace() {
        let candidate = WordCandidate {
            text: "пишу вот дальше".to_string(),
            origin: CandidateOrigin::Layout,
            source: "LayoutWordCell32",
            energy: 0.50,
            risk: 0.27,
            support: vec![],
        };
        let (trace, decision) = run_l3("пишу djn дальше ", &[candidate]);

        assert!(trace
            .iter()
            .any(|item| item.name == super::super::structural_relation::STRUCTURAL_RELATION_CELL));
        assert_eq!(decision.output(), Some("пишу вот дальше "));
    }

    trait DecisionOutput {
        fn output(&self) -> Option<&str>;
    }

    impl DecisionOutput for WaveDecision {
        fn output(&self) -> Option<&str> {
            match self {
                WaveDecision::Suggest { text, .. } => Some(text),
                WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
            }
        }
    }
}
