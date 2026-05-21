//! Generic candidate selection helpers.
//!
//! Correction modules should generate candidates; this module owns the repeated
//! "best candidate must clearly beat second best" mechanics.

pub(crate) fn choose_best_with_gap<I, T, F>(
    candidates: I,
    min_gap: f64,
    mut score_candidate: F,
) -> Option<(T, f64)>
where
    I: IntoIterator<Item = T>,
    F: FnMut(&T) -> Option<f64>,
{
    let mut best: Option<(T, f64)> = None;
    let mut second_best = f64::NEG_INFINITY;

    for candidate in candidates {
        let Some(score) = score_candidate(&candidate) else {
            continue;
        };

        match &best {
            Some((_, best_score)) if score <= *best_score => {
                second_best = second_best.max(score);
            }
            Some((_, best_score)) => {
                second_best = second_best.max(*best_score);
                best = Some((candidate, score));
            }
            None => best = Some((candidate, score)),
        }
    }

    let (candidate, best_score) = best?;
    if best_score - second_best < min_gap {
        return None;
    }
    Some((candidate, best_score))
}
