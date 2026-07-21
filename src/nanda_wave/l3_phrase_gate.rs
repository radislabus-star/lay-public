use super::llmwave::{self, LlmWaveMemory, LlmWaveNextTokenScore};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct L3PhraseGateReport {
    pub(crate) decision: L3PhraseGateDecision,
    pub(crate) source: &'static str,
    pub(crate) score: f32,
    pub(crate) rank_energy: f32,
    pub(crate) support: usize,
    pub(crate) width: usize,
    pub(crate) sequential_score: f32,
    pub(crate) scene_score: f32,
    pub(crate) competition_margin: f32,
    pub(crate) positive_micro: i64,
    pub(crate) anti_micro: i64,
    pub(crate) threshold_micro: i64,
    pub(crate) relation_class: u64,
    /// A directed L3 lattice certificate: this candidate beat every known
    /// competitor in the same scene. It is stronger than unary phrase support.
    pub(crate) pairwise_certified: bool,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L3PhraseGateDecision {
    Neutral,
    Support,
    Suppress,
}

#[derive(Debug, Clone, Copy, Default)]
struct CandidateContextEvidence {
    sequential_score: f32,
    scene_score: f32,
    field_score: f32,
    support: usize,
    width: usize,
}

pub(crate) fn evaluate_default_candidates(
    original: &str,
    replacements: &[&str],
) -> Vec<Option<L3PhraseGateReport>> {
    evaluate_candidates_with_phase(original, replacements)
}

fn evaluate_candidates_with_phase(
    original: &str,
    replacements: &[&str],
) -> Vec<Option<L3PhraseGateReport>> {
    let context_tokens = llmwave::tokenize(original).len().saturating_sub(1);
    reports_from_phase_readouts(
        context_tokens,
        super::context_phase::readout_default_candidates(original, replacements),
    )
}

pub(crate) fn evaluate_context_candidates_default(
    context_tokens: &[String],
    next_tokens: &[&str],
) -> Vec<Option<L3PhraseGateReport>> {
    reports_from_phase_readouts(
        context_tokens.len(),
        super::context_phase::default_memory().score_candidates(context_tokens, next_tokens),
    )
}

pub(super) fn reports_from_phase_readouts(
    context_tokens: usize,
    readouts: Vec<super::context_phase::ContextPhaseReadout>,
) -> Vec<Option<L3PhraseGateReport>> {
    readouts
        .into_iter()
        .map(|readout| {
            if !readout.profile_present {
                return None;
            }
            let decision = match readout.disposition {
                super::context_phase::ContextPhaseDisposition::Support => {
                    L3PhraseGateDecision::Support
                }
                super::context_phase::ContextPhaseDisposition::Suppress => {
                    L3PhraseGateDecision::Suppress
                }
                super::context_phase::ContextPhaseDisposition::Neutral
                | super::context_phase::ContextPhaseDisposition::Unavailable => {
                    L3PhraseGateDecision::Neutral
                }
            };
            let normalized_margin = (readout.margin_micro as f32 / 1_000_000.0).clamp(-1.0, 1.0);
            let rank_energy = match decision {
                L3PhraseGateDecision::Support => normalized_margin.max(0.0) * 0.16,
                L3PhraseGateDecision::Suppress => -normalized_margin.abs().max(0.10) * 0.16,
                L3PhraseGateDecision::Neutral => 0.0,
            };
            Some(L3PhraseGateReport {
                decision,
                source: "learned_context_phase",
                score: normalized_margin,
                rank_energy,
                support: readout.positive_examples as usize,
                width: context_tokens,
                sequential_score: readout.positive_micro as f32 / 1_000_000.0,
                scene_score: readout.semantic_support as f32,
                competition_margin: readout.competition_margin_micro as f32 / 1_000_000.0,
                positive_micro: readout.positive_micro,
                anti_micro: readout.anti_micro,
                threshold_micro: readout.threshold_micro,
                relation_class: readout.relation_class,
                pairwise_certified: readout.pairwise_certified,
                reason: match decision {
                    L3PhraseGateDecision::Support => "l3_context_phase_support",
                    L3PhraseGateDecision::Suppress => "l3_context_phase_suppress",
                    L3PhraseGateDecision::Neutral => "l3_context_phase_neutral",
                },
            })
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn evaluate_candidate_with_memory(
    original: &str,
    replacement: &str,
    memory: &LlmWaveMemory,
) -> Option<L3PhraseGateReport> {
    evaluate_candidates_with_memory(original, &[replacement], memory)
        .into_iter()
        .next()
        .flatten()
}

pub(crate) fn evaluate_candidates_with_memory(
    original: &str,
    replacements: &[&str],
    memory: &LlmWaveMemory,
) -> Vec<Option<L3PhraseGateReport>> {
    if memory.is_empty() || replacements.is_empty() {
        return vec![None; replacements.len()];
    }
    let mut context = llmwave::tokenize(original);
    if context.pop().is_none() || context.is_empty() {
        return vec![None; replacements.len()];
    }
    let original_tokens = llmwave::tokenize(original);
    let candidate_tokens = replacements
        .iter()
        .map(|replacement| context_preserving_next_token(&original_tokens, replacement))
        .collect::<Vec<_>>();
    evaluate_context_field_with_memory(&context, &candidate_tokens, memory)
}

#[cfg(test)]
pub(crate) fn evaluate_context_candidates_with_memory(
    context_tokens: &[String],
    next_tokens: &[&str],
    memory: &LlmWaveMemory,
) -> Vec<Option<L3PhraseGateReport>> {
    let candidates = next_tokens
        .iter()
        .map(|token| Some((*token).to_string()))
        .collect::<Vec<_>>();
    evaluate_context_field_with_memory(context_tokens, &candidates, memory)
}

fn evaluate_context_field_with_memory(
    context_tokens: &[String],
    candidate_tokens: &[Option<String>],
    memory: &LlmWaveMemory,
) -> Vec<Option<L3PhraseGateReport>> {
    if context_tokens.is_empty() || memory.is_empty() || candidate_tokens.is_empty() {
        return vec![None; candidate_tokens.len()];
    }

    let valid_tokens = candidate_tokens
        .iter()
        .filter_map(Option::as_deref)
        .collect::<Vec<_>>();
    if valid_tokens.is_empty() {
        return vec![None; candidate_tokens.len()];
    }
    let sequential = memory.score_next_tokens_report(context_tokens, &valid_tokens);
    let scene = memory.score_scene_tokens_report(context_tokens, &valid_tokens);
    let mut evidence = Vec::with_capacity(candidate_tokens.len());
    let mut valid_index = 0usize;
    for candidate in candidate_tokens {
        if candidate.is_none() {
            evidence.push(None);
            continue;
        }
        evidence.push(Some(context_evidence(
            sequential.get(valid_index).and_then(Option::as_ref),
            scene.get(valid_index).and_then(Option::as_ref),
        )));
        valid_index += 1;
    }

    let mut ranked = evidence
        .iter()
        .enumerate()
        .filter_map(|(index, item)| item.map(|item| (index, item.field_score)))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.total_cmp(&left.1));
    let best = ranked.first().copied();
    let runner_up_score = ranked.get(1).map(|(_, score)| *score).unwrap_or(0.0);
    let best_margin = best
        .map(|(_, score)| (score - runner_up_score).max(0.0))
        .unwrap_or(0.0);
    let best_has_authority = best.is_some_and(|(index, score)| {
        let item = evidence[index].unwrap_or_default();
        item.support >= 2 && score >= 0.16 && (ranked.len() == 1 || best_margin >= 0.02)
    });

    evidence
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let item = item?;
            let (decision, reason, competition_margin) = match best {
                Some((best_index, _)) if best_has_authority && index == best_index => (
                    L3PhraseGateDecision::Support,
                    "l3_context_field_support",
                    best_margin,
                ),
                Some((_, best_score))
                    if best_has_authority
                        && item.field_score + (best_score * 0.25).max(0.04) < best_score =>
                {
                    (
                        L3PhraseGateDecision::Suppress,
                        "l3_context_field_competitor",
                        item.field_score - best_score,
                    )
                }
                _ => (
                    L3PhraseGateDecision::Neutral,
                    "l3_context_field_neutral",
                    0.0,
                ),
            };
            Some(L3PhraseGateReport {
                decision,
                source: "legacy_phrase_memory",
                score: item.field_score,
                rank_energy: match decision {
                    L3PhraseGateDecision::Support => item.field_score * 0.16,
                    L3PhraseGateDecision::Suppress => -0.14,
                    L3PhraseGateDecision::Neutral => 0.0,
                },
                support: item.support,
                width: item.width,
                sequential_score: item.sequential_score,
                scene_score: item.scene_score,
                competition_margin,
                positive_micro: (item.field_score * 1_000_000.0).round() as i64,
                anti_micro: 0,
                threshold_micro: 160_000,
                relation_class: 0,
                pairwise_certified: false,
                reason,
            })
        })
        .collect()
}

fn context_evidence(
    sequential: Option<&LlmWaveNextTokenScore>,
    scene: Option<&LlmWaveNextTokenScore>,
) -> CandidateContextEvidence {
    let sequential_score = reliable_score(sequential);
    let scene_score = reliable_score(scene);
    CandidateContextEvidence {
        sequential_score,
        scene_score,
        field_score: constructive_interference(sequential_score, scene_score),
        // Sequential and scene lanes can observe the same training transition;
        // max keeps that evidence from being counted twice.
        support: sequential
            .map(|item| item.support)
            .unwrap_or_default()
            .max(scene.map(|item| item.support).unwrap_or_default()),
        width: sequential
            .map(|item| item.width)
            .unwrap_or_default()
            .max(scene.map(|item| item.width).unwrap_or_default()),
    }
}

fn reliable_score(report: Option<&LlmWaveNextTokenScore>) -> f32 {
    let Some(report) = report else {
        return 0.0;
    };
    let support = report.support as f32;
    let reliability = support / (support + 2.0);
    (report.score * reliability).clamp(0.0, 1.0)
}

fn constructive_interference(sequential: f32, scene: f32) -> f32 {
    (1.0 - (1.0 - sequential) * (1.0 - scene)).clamp(0.0, 1.0)
}

fn context_preserving_next_token(original_tokens: &[String], replacement: &str) -> Option<String> {
    let replacement_tokens = llmwave::tokenize(replacement);
    if replacement_tokens.len() != original_tokens.len() || original_tokens.len() < 2 {
        return None;
    }
    let context_len = original_tokens.len() - 1;
    if original_tokens[..context_len] != replacement_tokens[..context_len] {
        return None;
    }
    replacement_tokens.last().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_candidate_when_phrase_memory_matches_context() {
        let memory = LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nсегодня на улице опять идёт дождь\nвечером на улице опять идёт дождь",
        );
        let report = evaluate_candidate_with_memory(
            "на улице опять идёт дом ",
            "на улице опять идёт дождь ",
            &memory,
        )
        .expect("phrase report");

        assert_eq!(report.decision, L3PhraseGateDecision::Support);
        assert_eq!(report.reason, "l3_context_field_support");
        assert!(report.score >= 0.16);
        assert!(report.support >= 2);
    }

    #[test]
    fn suppresses_candidate_when_strong_phrase_memory_points_elsewhere() {
        let memory = LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nсегодня на улице опять идёт дождь\nвечером на улице опять идёт дождь\nзавтра на улице опять идёт дождь",
        );
        let reports = evaluate_candidates_with_memory(
            "на улице опять идёт д ",
            &["на улице опять идёт дом ", "на улице опять идёт дождь "],
            &memory,
        );

        assert_eq!(
            reports[0].as_ref().map(|report| report.decision),
            Some(L3PhraseGateDecision::Suppress)
        );
        assert_eq!(
            reports[1].as_ref().map(|report| report.decision),
            Some(L3PhraseGateDecision::Support)
        );
    }

    #[test]
    fn singleton_phrase_memory_stays_neutral_for_apply_authority() {
        let memory = LlmWaveMemory::from_text("на улице опять идёт дождь");
        let report = evaluate_candidate_with_memory(
            "на улице опять идёт дом ",
            "на улице опять идёт дождь ",
            &memory,
        )
        .expect("phrase report");

        assert_eq!(report.decision, L3PhraseGateDecision::Neutral);
    }

    #[test]
    fn whole_scene_can_support_without_exact_suffix_ngram() {
        let memory = LlmWaveMemory::from_text(
            "поставщик платежи подтвердил\nнаш поставщик платежи рассчитал\nдругой поставщик платежи указал\nлес деревья растут",
        );
        let context = llmwave::tokenize("наш поставщик уточнил итоговые");
        assert!(memory
            .score_next_token_report(&context, "платежи")
            .is_none());

        let reports =
            evaluate_context_candidates_with_memory(&context, &["деревья", "платежи"], &memory);
        assert_eq!(
            reports[1].as_ref().map(|report| report.decision),
            Some(L3PhraseGateDecision::Support),
            "reports={reports:?}"
        );
        assert!(reports[1]
            .as_ref()
            .is_some_and(|report| report.scene_score > report.sequential_score));
    }

    #[test]
    fn single_token_context_is_visible_but_cannot_grant_apply_authority() {
        let memory = LlmWaveMemory::from_text("wave и interest");
        let report = evaluate_candidate_with_memory("wave b ", "wave и ", &memory)
            .expect("single-token context report");

        assert_eq!(report.decision, L3PhraseGateDecision::Neutral);
        assert_eq!(report.rank_energy, 0.0);
    }
}
