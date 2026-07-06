use super::llmwave::{self, LlmWaveMemory};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct L3PhraseGateReport {
    pub(crate) decision: L3PhraseGateDecision,
    pub(crate) score: f32,
    pub(crate) support: usize,
    pub(crate) width: usize,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L3PhraseGateDecision {
    Neutral,
    Support,
    Suppress,
}

pub(crate) fn evaluate_default_candidate(
    original: &str,
    replacement: &str,
) -> Option<L3PhraseGateReport> {
    if !llmwave::default_memory_is_warm() {
        return None;
    }
    llmwave::with_default_memory(|memory| {
        evaluate_candidate_with_memory(original, replacement, memory)
    })
}

pub(crate) fn evaluate_candidate_with_memory(
    original: &str,
    replacement: &str,
    memory: &LlmWaveMemory,
) -> Option<L3PhraseGateReport> {
    if memory.is_empty() || original == replacement {
        return None;
    }
    let original_tokens = llmwave::tokenize(original);
    let replacement_tokens = llmwave::tokenize(replacement);
    let (prefix, next) = changed_next_token(&original_tokens, &replacement_tokens)?;
    if prefix.len() < 2 {
        return None;
    }

    let candidate = memory.score_next_token_report(&prefix, &next);
    let best = memory
        .predict_phrase(&prefix.join(" "), 1, 4)
        .into_iter()
        .next();

    if let Some(score) = candidate.as_ref() {
        if score.score >= 0.42 && phrase_support_has_apply_mass(score.support) {
            return Some(L3PhraseGateReport {
                decision: L3PhraseGateDecision::Support,
                score: score.score,
                support: score.support,
                width: score.width,
                reason: "l3_phrase_memory_support",
            });
        }
    }

    let best_score = best.as_ref().map(|item| item.score).unwrap_or(0.0);
    let best_token = best
        .as_ref()
        .and_then(|item| item.tokens.get(prefix.len()))
        .map(String::as_str);
    let best_support = best.as_ref().map(|item| item.support).unwrap_or(0);
    if best_score >= 0.56
        && phrase_support_has_apply_mass(best_support)
        && best_token.is_some_and(|token| token != next)
    {
        return Some(L3PhraseGateReport {
            decision: L3PhraseGateDecision::Suppress,
            score: candidate.map(|score| score.score).unwrap_or(0.0),
            support: 0,
            width: 0,
            reason: "l3_phrase_memory_conflict",
        });
    }

    Some(L3PhraseGateReport {
        decision: L3PhraseGateDecision::Neutral,
        score: candidate.map(|score| score.score).unwrap_or(0.0),
        support: 0,
        width: 0,
        reason: "l3_phrase_memory_neutral",
    })
}

fn phrase_support_has_apply_mass(support: usize) -> bool {
    support >= 2
}

fn changed_next_token(
    original_tokens: &[String],
    replacement_tokens: &[String],
) -> Option<(Vec<String>, String)> {
    if replacement_tokens.is_empty() {
        return None;
    }
    for idx in 0..replacement_tokens.len() {
        if original_tokens.get(idx) != replacement_tokens.get(idx) {
            if idx == 0 {
                return None;
            }
            return Some((
                replacement_tokens[..idx].to_vec(),
                replacement_tokens[idx].clone(),
            ));
        }
    }
    if replacement_tokens.len() > original_tokens.len() && !original_tokens.is_empty() {
        return Some((
            replacement_tokens[..original_tokens.len()].to_vec(),
            replacement_tokens[original_tokens.len()].clone(),
        ));
    }
    None
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
        assert_eq!(report.reason, "l3_phrase_memory_support");
        assert!(report.score >= 0.42);
        assert!(report.support >= 2);
    }

    #[test]
    fn suppresses_candidate_when_strong_phrase_memory_points_elsewhere() {
        let memory = LlmWaveMemory::from_text(
            "на улице опять идёт дождь\nсегодня на улице опять идёт дождь\nвечером на улице опять идёт дождь\nзавтра на улице опять идёт дождь",
        );
        let report = evaluate_candidate_with_memory(
            "на улице опять идёт д ",
            "на улице опять идёт дом ",
            &memory,
        )
        .expect("phrase report");

        assert_eq!(report.decision, L3PhraseGateDecision::Suppress);
        assert_eq!(report.reason, "l3_phrase_memory_conflict");
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
        assert_eq!(report.support, 0);
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
