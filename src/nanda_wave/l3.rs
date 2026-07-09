use super::l4_signed_memory::{l4_signed_memory_signal, L4SignedMemoryInput};
use super::lexical_attractor::LEXICAL_ATTRACTOR_CELL;
use super::options::WaveOptions;
use super::pattern_wave::{evaluate_pattern_wave, PATTERN_WAVE_CELL};
use super::signal::{LayerTrace, WaveDecision, WordCandidate};
use super::structural_relation::{evaluate_structural_relation, STRUCTURAL_RELATION_CELL};
use super::{l3_phrase_gate, llmwave};
use crate::text_metrics::damerau_levenshtein;

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
        .find(|candidate| candidate.source == "TechTokenCell32");
    let best_apply = best_apply_candidate(original, candidates, options, phrase_memory);

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
                && (best_apply.is_none()
                    || technical.energy >= best_apply.map(|item| item.energy).unwrap_or(0.0))
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

    let Some(candidate) = best_apply else {
        return (
            traces,
            WaveDecision::Keep {
                reason: "no_layout_candidate",
            },
        );
    };

    if short_uppercase_layout_candidate_lacks_phrase_context(
        original,
        &candidate.text,
        candidate.source,
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

    if short_token_candidate_lacks_phrase_context(original, &candidate.text, candidate.source) {
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

    let phrase_report = phrase_gate_report(original, &candidate.text, phrase_memory);
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
    if options.is_enabled(super::context_wave::PHRASE_FORECAST_CELL) {
        if let Some(summary) = super::context_wave::phrase_forecast_summary(original, candidate) {
            traces.push(LayerTrace {
                name: super::context_wave::PHRASE_FORECAST_CELL,
                summary,
            });
        }
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
            WaveDecision::Apply {
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

fn apply_source_enabled(source: &str, options: &WaveOptions) -> bool {
    match source {
        "LayoutWordCell32" => options.is_enabled("LayoutWordCell32"),
        "ShortTokenCell32" => options.is_enabled("ShortTokenCell32"),
        "BoundaryCell32" => options.is_enabled("BoundaryCell32"),
        "GrammarCell32" => options.is_enabled("GrammarCell32"),
        "PhraseCell32" => options.is_enabled("PhraseCell32"),
        "LearnedMemoryCell32" => options.is_enabled("LearnedMemoryCell32"),
        "CommonRuFixCell32" => options.is_enabled("CommonRuFixCell32"),
        "PhraseMemoryCell32" => options.is_enabled("PhraseMemoryCell32"),
        "UserMemoryCell32" => options.is_enabled("UserMemoryCell32"),
        source if source == super::l2::L2_SURFACE_MOTIF_CELL => {
            options.is_enabled(super::l2::L2_SURFACE_MOTIF_CELL)
        }
        source if source == super::l2::L2_SURFACE_COMPLETION_CELL => {
            options.is_enabled(super::l2::L2_SURFACE_COMPLETION_CELL)
        }
        source if source == super::lexical_attractor::LEXICAL_ATTRACTOR_CELL => {
            options.is_enabled(super::lexical_attractor::LEXICAL_ATTRACTOR_CELL)
        }
        source if source == super::context_wave::SEMANTIC_WORD_SOURCE => {
            options.is_enabled(super::context_wave::SEMANTIC_WORD_SOURCE)
        }
        _ => false,
    }
}

fn best_apply_candidate<'a>(
    original: &str,
    candidates: &'a [WordCandidate],
    options: &WaveOptions,
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> Option<&'a WordCandidate> {
    candidates
        .iter()
        .filter(|candidate| apply_source_enabled(candidate.source, options))
        .filter(|candidate| !semantic_candidate_lacks_surface_authority(original, candidate))
        .filter(|candidate| !word_form_candidate_lacks_autocorrect_authority(original, candidate))
        .filter(|candidate| !completion_candidate_lacks_autocorrect_authority(original, candidate))
        .filter(|candidate| !candidate_l4_signed_memory_vetoes_apply(original, candidate))
        .filter(|candidate| !phrase_gate_suppresses(original, &candidate.text, phrase_memory))
        .fold(None, |best: Option<(&'a WordCandidate, f32)>, candidate| {
            let score = l3_rank_score(original, candidate, phrase_memory);
            match best {
                Some((best_candidate, best_score)) if score <= best_score => {
                    Some((best_candidate, best_score))
                }
                _ => Some((candidate, score)),
            }
        })
        .map(|(candidate, _score)| candidate)
}

fn confidence(candidate: &WordCandidate) -> f32 {
    (candidate.energy - candidate.risk).clamp(0.0, 1.0)
}

fn l3_rank_score(
    original: &str,
    candidate: &WordCandidate,
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> f32 {
    let mut value = confidence(candidate);
    value += candidate_usage_context_prior(original, &candidate.text);
    value += candidate_l4_signed_bias(original, candidate);
    value += candidate_l4_scene_memory_bias(original, candidate, phrase_memory);
    if let Some(report) = phrase_gate_report(original, &candidate.text, phrase_memory) {
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

fn completion_candidate_lacks_autocorrect_authority(
    _original: &str,
    candidate: &WordCandidate,
) -> bool {
    if candidate.source != super::l2::L2_SURFACE_COMPLETION_CELL {
        return false;
    }
    // L2 surface completion is an IME/preedit suggestion source. It can expose
    // a continuation candidate, but it must not rewrite committed text on Space.
    true
}

fn word_form_candidate_lacks_autocorrect_authority(
    original: &str,
    candidate: &WordCandidate,
) -> bool {
    if !matches!(
        candidate.source,
        super::l2::L2_SURFACE_MOTIF_CELL | LEXICAL_ATTRACTOR_CELL
    ) {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return true;
    };
    let Some(replacement_word) = last_token(&candidate.text) else {
        return true;
    };
    if !is_cyrillic_word(original_word) || !is_cyrillic_word(replacement_word) {
        return true;
    }

    let original_lower = original_word.to_lowercase();
    let replacement_lower = replacement_word.to_lowercase();
    let original_known = crate::russian_lexicon::is_known_russian_word_or_form(&original_lower)
        || crate::lexicon::is_common_ru_word(&original_lower);
    let replacement_known =
        crate::russian_lexicon::is_known_russian_word_or_form(&replacement_lower)
            || crate::lexicon::is_common_ru_word(&replacement_lower);
    if original_known && replacement_known && original_lower != replacement_lower {
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
    if phase_admitted {
        return false;
    }
    original_lower.chars().count() < 7
        || distance > 3
        || !crate::lexicon::is_common_ru_word(&replacement_lower)
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

fn semantic_candidate_lacks_surface_authority(original: &str, candidate: &WordCandidate) -> bool {
    if candidate.source != super::context_wave::SEMANTIC_WORD_SOURCE {
        return false;
    }
    let Some(original_word) = last_token(original) else {
        return true;
    };
    let Some(replacement_word) = last_token(&candidate.text) else {
        return true;
    };
    if !is_cyrillic_word(original_word) || !is_cyrillic_word(replacement_word) {
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
    let prefix = common_prefix_len(&original_lower, &replacement_lower);
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

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn phrase_gate_suppresses(
    original: &str,
    replacement: &str,
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> bool {
    phrase_gate_report(original, replacement, phrase_memory)
        .is_some_and(|report| report.decision == l3_phrase_gate::L3PhraseGateDecision::Suppress)
}

fn phrase_gate_report(
    original: &str,
    replacement: &str,
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> Option<l3_phrase_gate::L3PhraseGateReport> {
    match phrase_memory {
        Some(memory) => {
            l3_phrase_gate::evaluate_candidate_with_memory(original, replacement, memory)
        }
        None => l3_phrase_gate::evaluate_default_candidate(original, replacement),
    }
}

fn short_token_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    source: &str,
) -> bool {
    if source != "ShortTokenCell32" {
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
        .any(|word| word.chars().any(is_cyrillic_char));
    let has_ascii_context = previous_words
        .iter()
        .any(|word| word.chars().any(|ch| ch.is_ascii_alphabetic()));
    has_ascii_context && !has_cyrillic_context
}

fn short_uppercase_layout_candidate_lacks_phrase_context(
    original: &str,
    replacement: &str,
    source: &str,
) -> bool {
    if source != "LayoutWordCell32" {
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
        .any(|word| word.chars().any(is_cyrillic_char))
}

fn last_token(text: &str) -> Option<&str> {
    text.split_whitespace().next_back()
}

fn is_cyrillic_char(ch: char) -> bool {
    matches!(ch, 'А'..='я' | 'ё' | 'Ё')
}

fn is_cyrillic_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(is_cyrillic_char)
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
    if options.is_enabled(super::context_wave::PHRASE_FORECAST_CELL)
        && candidate.source == super::context_wave::SEMANTIC_WORD_SOURCE
        && super::context_wave::phrase_forecast_summary(original, candidate).is_some()
    {
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
    value += candidate_l4_scene_memory_bias(original, candidate, None);
    value.clamp(0.0, 1.0)
}

fn candidate_l4_signed_bias(original: &str, candidate: &WordCandidate) -> f32 {
    let Some(signal) = candidate_l4_signed_signal(original, candidate) else {
        return 0.0;
    };
    (signal.signed_weight * 0.080).clamp(-0.080, 0.080)
}

fn candidate_l4_signed_memory_vetoes_apply(original: &str, candidate: &WordCandidate) -> bool {
    let Some(signal) = candidate_l4_signed_signal(original, candidate) else {
        return false;
    };
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
    let word = last_token(&candidate.text)?;
    let context = previous_context_tokens(original);
    let usage = super::usage_prior::cached_usage_prior_snapshot();
    Some(l4_signed_memory_signal(L4SignedMemoryInput {
        context: &context,
        source: candidate.source,
        operation: candidate_operation(candidate.source),
        word,
        usage: &usage,
    }))
}

fn candidate_l4_scene_memory_bias(
    original: &str,
    candidate: &WordCandidate,
    phrase_memory: Option<&llmwave::LlmWaveMemory>,
) -> f32 {
    let Some(word) = last_token(&candidate.text) else {
        return 0.0;
    };
    let context = previous_context_tokens(original);
    if context.len() < 3 {
        return 0.0;
    }
    let report = match phrase_memory {
        Some(memory) => memory.score_scene_token_report(&context, word),
        None if llmwave::default_memory_is_warm() => {
            llmwave::with_default_memory(|memory| memory.score_scene_token_report(&context, word))
        }
        None => None,
    };
    report
        .filter(|report| report.score >= 0.16 && report.support > 0)
        .map(|report| (report.score * 0.070).clamp(0.0, 0.080))
        .unwrap_or(0.0)
}

fn candidate_operation(source: &str) -> &'static str {
    match source {
        "LayoutWordCell32" => "layout",
        "BoundaryCell32" => "boundary",
        source if source == super::l2::L2_SURFACE_COMPLETION_CELL => "completion",
        _ => "replacement",
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

    #[test]
    fn applies_confident_layout_candidate() {
        let candidate = WordCandidate {
            text: "html вот".to_string(),
            source: "LayoutWordCell32",
            energy: 0.8,
            risk: 0.1,
            support: vec![],
        };
        let (_trace, decision) = run_l3("html djn ", &[candidate]);
        assert_eq!(decision.output(), Some("html вот "));
    }

    #[test]
    fn l3_does_not_autocomplete_short_prefix_without_memory_authority() {
        let candidates = [
            WordCandidate {
                text: "Ну давай".to_string(),
                source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
                energy: 0.856,
                risk: 0.060,
                support: vec![],
            },
            WordCandidate {
                text: "Ну даша".to_string(),
                source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
                energy: 0.856,
                risk: 0.060,
                support: vec![],
            },
        ];

        let (_trace, decision) = run_l3("Ну да ", &candidates);

        assert_eq!(
            decision,
            WaveDecision::Keep {
                reason: "no_layout_candidate"
            }
        );
    }

    #[test]
    fn l3_keeps_first_equal_candidate_instead_of_last_equal_candidate() {
        let candidates = [
            WordCandidate {
                text: "попаданий".to_string(),
                source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
                energy: 0.80,
                risk: 0.10,
                support: vec![],
            },
            WordCandidate {
                text: "попадали".to_string(),
                source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
                energy: 0.80,
                risk: 0.10,
                support: vec![],
            },
        ];

        let (_trace, decision) = run_l3("попадани ", &candidates);

        assert_eq!(decision.output(), Some("попаданий "));
    }

    #[test]
    fn l3_keeps_surface_completion_out_of_autocorrect() {
        let candidate = WordCandidate {
            text: "попаданий".to_string(),
            source: super::super::l2::L2_SURFACE_COMPLETION_CELL,
            energy: 0.856,
            risk: 0.060,
            support: vec![],
        };

        let (_trace, decision) = run_l3("попадани ", &[candidate]);

        assert_eq!(decision.output(), None);
    }

    #[test]
    fn keeps_short_layout_candidate_without_russian_phrase_context() {
        let candidate = WordCandidate {
            text: "wave и".to_string(),
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
            source: "TechTokenCell32",
            energy: 0.95,
            risk: 0.02,
            support: vec![],
        };
        let layout = WordCandidate {
            text: "git checkout -b туц".to_string(),
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
            source: "TechTokenCell32",
            energy: 0.95,
            risk: 0.02,
            support: vec![],
        };
        let layout = WordCandidate {
            text: "html вот api".to_string(),
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
            source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
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
            source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
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
            source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
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
                source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
                energy: 0.86,
                risk: 0.06,
                support: vec![],
            },
            WordCandidate {
                text: "на улице опять идёт дождь".to_string(),
                source: super::super::context_wave::SEMANTIC_WORD_SOURCE,
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
            .any(|item| item.summary.contains("l3_phrase=l3_phrase_memory_support")));
    }

    #[test]
    fn pattern_wave_is_visible_in_l3_trace() {
        let candidate = WordCandidate {
            text: "html вот api".to_string(),
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
                WaveDecision::Apply { text, .. } => Some(text),
                WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
            }
        }
    }
}
