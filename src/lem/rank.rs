use super::score::score_candidate;
use super::ScoredCandidate;
use std::collections::HashSet;

pub fn rank_candidates<I>(typed: &str, candidates: I) -> Vec<ScoredCandidate>
where
    I: IntoIterator<Item = String>,
{
    let mut seen = HashSet::new();
    let mut ranked = Vec::new();
    for candidate in candidates {
        let key = candidate.trim().to_string();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        ranked.push(score_candidate(typed, candidate));
    }
    ranked.sort_by(|a, b| b.total.total_cmp(&a.total));
    ranked
}

pub fn best_candidate<I>(typed: &str, candidates: I) -> Option<ScoredCandidate>
where
    I: IntoIterator<Item = String>,
{
    rank_candidates(typed, candidates).into_iter().next()
}
