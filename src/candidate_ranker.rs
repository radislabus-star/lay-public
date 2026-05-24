//! Generic candidate selection helpers.
//!
//! Correction modules should generate candidates; this module owns the repeated
//! "best candidate must clearly beat second best" mechanics.

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RankedPair<T> {
    pub best: T,
    pub second: Option<T>,
    pub best_score: f64,
    pub margin: f64,
}

pub(crate) fn rank_best_two<I, T, F, B>(
    candidates: I,
    mut score_candidate: F,
    mut tie_breaks_current: B,
) -> Option<RankedPair<T>>
where
    I: IntoIterator<Item = T>,
    F: FnMut(&T) -> Option<f64>,
    B: FnMut(&T, &T) -> bool,
{
    let mut best: Option<(T, f64)> = None;
    let mut second: Option<(T, f64)> = None;

    for candidate in candidates {
        let Some(score) = score_candidate(&candidate) else {
            continue;
        };

        if best
            .as_ref()
            .map(|(current, current_score)| {
                candidate_beats(
                    &candidate,
                    score,
                    current,
                    *current_score,
                    &mut tie_breaks_current,
                )
            })
            .unwrap_or(true)
        {
            second = best;
            best = Some((candidate, score));
        } else if second
            .as_ref()
            .map(|(current, current_score)| {
                candidate_beats(
                    &candidate,
                    score,
                    current,
                    *current_score,
                    &mut tie_breaks_current,
                )
            })
            .unwrap_or(true)
        {
            second = Some((candidate, score));
        }
    }

    let (best, best_score) = best?;
    let (second, second_score) = second
        .map(|(candidate, score)| (Some(candidate), score))
        .unwrap_or((None, f64::NEG_INFINITY));

    Some(RankedPair {
        best,
        second,
        best_score,
        margin: best_score - second_score,
    })
}

pub(crate) fn choose_best_with_gap<I, T, F>(
    candidates: I,
    min_gap: f64,
    mut score_candidate: F,
) -> Option<(T, f64)>
where
    I: IntoIterator<Item = T>,
    F: FnMut(&T) -> Option<f64>,
{
    let ranked = rank_best_two(
        candidates,
        |candidate| score_candidate(candidate),
        |_, _| false,
    )?;
    if ranked.margin < min_gap {
        return None;
    }
    Some((ranked.best, ranked.best_score))
}

fn candidate_beats<T, B>(
    candidate: &T,
    score: f64,
    current: &T,
    current_score: f64,
    tie_breaks_current: &mut B,
) -> bool
where
    B: FnMut(&T, &T) -> bool,
{
    const EPSILON: f64 = 0.000_001;
    let diff = score - current_score;
    if diff.abs() > EPSILON {
        return diff > 0.0;
    }
    tie_breaks_current(candidate, current)
}
