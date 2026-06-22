use super::score::score_candidate;
use super::ScoredCandidate;
use std::collections::HashSet;

pub fn rank_candidates(
    typed: &str,
    candidates: impl IntoIterator<Item = String>,
) -> Vec<ScoredCandidate> {
    rank_candidates_with_language_weight(typed, candidates, 1.0)
}

pub fn rank_candidates_with_language_weight(
    typed: &str,
    candidates: impl IntoIterator<Item = String>,
    language_weight: f64,
) -> Vec<ScoredCandidate> {
    let mut seen = HashSet::new();
    let mut ranked = Vec::new();
    let language_weight = language_weight.clamp(0.0, 2.0);
    for candidate in candidates {
        let key = candidate.trim().to_string();
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        ranked.push(score_candidate(typed, candidate).with_language_weight(language_weight));
    }
    ranked.sort_by(|a, b| b.total.total_cmp(&a.total));
    ranked
}

pub fn best_candidate(
    typed: &str,
    candidates: impl IntoIterator<Item = String>,
) -> Option<ScoredCandidate> {
    rank_candidates(typed, candidates).into_iter().next()
}

#[cfg(test)]
#[path = "rank_tests.rs"]
mod tests;
