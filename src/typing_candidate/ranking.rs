use crate::candidate_ranker::rank_best_two;

use super::types::{TypingCandidate, TypingCandidateDecision};

#[cfg(test)]
pub fn choose_typing_candidate<I>(candidates: I) -> Option<TypingCandidate>
where
    I: IntoIterator<Item = TypingCandidate>,
{
    rank_typing_candidates(candidates).map(|decision| decision.best)
}

pub fn rank_typing_candidates<I>(candidates: I) -> Option<TypingCandidateDecision>
where
    I: IntoIterator<Item = TypingCandidate>,
{
    let ranked = rank_best_two(
        candidates,
        |candidate| (!candidate.replacement.trim().is_empty()).then_some(candidate.score.total),
        candidate_tie_breaks_current,
    )?;

    Some(TypingCandidateDecision {
        best: ranked.best,
        second: ranked.second,
        margin: ranked.margin,
    })
}

fn candidate_tie_breaks_current(left: &TypingCandidate, right: &TypingCandidate) -> bool {
    if left.priority != right.priority {
        return left.priority < right.priority;
    }
    if left.rule_id != right.rule_id {
        return left.rule_id < right.rule_id;
    }
    left.replacement < right.replacement
}
