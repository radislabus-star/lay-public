use super::llmwave::{self, LlmWaveMemory, LlmWaveNextTokenScore};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct L3PhraseGateReport {
    pub(crate) decision: L3PhraseGateDecision,
    pub(crate) score: f32,
    pub(crate) support: usize,
    pub(crate) width: usize,
    pub(crate) sequential_score: f32,
    pub(crate) scene_score: f32,
    pub(crate) competition_margin: f32,
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
    if !llmwave::default_memory_is_warm() {
        return vec![None; replacements.len()];
    }
    llmwave::with_default_memory(|memory| {
        evaluate_candidates_with_memory(original, replacements, memory)
    })
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
    if context.pop().is_none() || context.len() < 2 {
        return vec![None; replacements.len()];
    }
    let original_tokens = llmwave::tokenize(original);
    let candidate_tokens = replacements
        .iter()
        .map(|replacement| context_preserving_next_token(&original_tokens, replacement))
        .collect::<Vec<_>>();
    evaluate_context_field_with_memory(&context, &candidate_tokens, memory)
}

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
    if context_tokens.len() < 2 || memory.is_empty() || candidate_tokens.is_empty() {
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
                score: item.field_score,
                support: item.support,
                width: item.width,
                sequential_score: item.sequential_score,
                scene_score: item.scene_score,
                competition_margin,
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
    if replacement_tokens.len() != original_tokens.len() || original_tokens.len() < 3 {
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
    fn ignores_single_token_context() {
        let memory = LlmWaveMemory::from_text("wave и interest");
        assert_eq!(
            evaluate_candidate_with_memory("wave b ", "wave и ", &memory),
            None
        );
    }
}
