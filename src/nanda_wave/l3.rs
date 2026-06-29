use super::options::WaveOptions;
use super::pattern_wave::{evaluate_pattern_wave, PATTERN_WAVE_CELL};
use super::signal::{LayerTrace, WaveDecision, WordCandidate};
use super::structural_relation::{evaluate_structural_relation, STRUCTURAL_RELATION_CELL};

pub fn run_l3(original: &str, candidates: &[WordCandidate]) -> (Vec<LayerTrace>, WaveDecision) {
    run_l3_with_options(original, candidates, &WaveOptions::default())
}

pub fn run_l3_with_options(
    original: &str,
    candidates: &[WordCandidate],
    options: &WaveOptions,
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
    let best_apply = candidates
        .iter()
        .filter(|candidate| apply_source_enabled(candidate.source, options))
        .max_by(|left, right| {
            confidence(left)
                .total_cmp(&confidence(right))
                .then_with(|| left.energy.total_cmp(&right.energy))
        });

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

    let confidence = adjusted_confidence(
        original,
        candidate,
        options,
        pattern_report.as_ref(),
        structural_report.as_ref(),
    );
    if options.is_enabled("PhraseCell32") {
        traces.push(LayerTrace {
            name: "PhraseCell32",
            summary: format!(
                "candidate source={} energy={:.3} risk={:.3} confidence={:.3}",
                candidate.source, candidate.energy, candidate.risk, confidence
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
        source if source == super::context_wave::SEMANTIC_WORD_SOURCE => {
            options.is_enabled(super::context_wave::SEMANTIC_WORD_SOURCE)
        }
        _ => false,
    }
}

fn confidence(candidate: &WordCandidate) -> f32 {
    (candidate.energy - candidate.risk).clamp(0.0, 1.0)
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
        .trim_end()
        .split_whitespace()
        .take(
            original
                .trim_end()
                .split_whitespace()
                .count()
                .saturating_sub(1),
        )
        .collect::<Vec<_>>();
    let has_cyrillic_context = previous_words
        .iter()
        .any(|word| word.chars().any(is_cyrillic_char));
    let has_ascii_context = previous_words
        .iter()
        .any(|word| word.chars().any(|ch| ch.is_ascii_alphabetic()));
    has_ascii_context && !has_cyrillic_context
}

fn last_token(text: &str) -> Option<&str> {
    text.trim_end().split_whitespace().next_back()
}

fn is_cyrillic_char(ch: char) -> bool {
    matches!(ch, 'А'..='я' | 'ё' | 'Ё')
}

fn adjusted_confidence(
    original: &str,
    candidate: &WordCandidate,
    options: &WaveOptions,
    pattern_report: Option<&super::pattern_wave::PatternWaveReport>,
    structural_report: Option<&super::structural_relation::StructuralRelationReport>,
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
