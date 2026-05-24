use crate::candidate_ranker::rank_best_two;
use crate::keyboard::KeyEvent;
use crate::lem::ScoredCandidate;
use crate::scoped_tail::rank_scoped_tail_lem_candidates;
use crate::typing_assist::ScopedTailOptions;

const MANUAL_SCOPED_TAIL_MIN_MARGIN: f64 = 0.20;

#[derive(Debug, Clone, PartialEq)]
pub struct RankedDecoderCandidate {
    pub text: String,
    pub total: f64,
    pub language: f64,
    pub noise: f64,
    pub edit: f64,
    pub intervention: f64,
}

impl From<ScoredCandidate> for RankedDecoderCandidate {
    fn from(value: ScoredCandidate) -> Self {
        Self {
            text: value.text,
            total: value.total,
            language: value.language,
            noise: value.noise,
            edit: value.edit,
            intervention: value.intervention,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankedDecoderDecision {
    pub original: String,
    pub best: RankedDecoderCandidate,
    pub second: Option<RankedDecoderCandidate>,
    pub margin: f64,
}

impl Eq for RankedDecoderDecision {}

impl Eq for RankedDecoderCandidate {}

pub fn rank_scoped_tail_candidates(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<RankedDecoderDecision> {
    if !options.lem_enabled {
        return None;
    }

    let (original, ranked) = rank_scoped_tail_lem_candidates(events, options)?;
    let decision = rank_best_two(
        ranked.into_iter().map(RankedDecoderCandidate::from),
        |candidate| Some(candidate.total),
        |_, _| false,
    )?;

    Some(RankedDecoderDecision {
        original,
        best: decision.best,
        second: decision.second,
        margin: decision.margin,
    })
}

pub fn choose_ranked_scoped_tail(
    events: &[KeyEvent],
    options: ScopedTailOptions,
) -> Option<RankedDecoderDecision> {
    let decision = rank_scoped_tail_candidates(events, options)?;
    if decision.best.text == decision.original || decision.best.text.trim().is_empty() {
        return None;
    }
    if decision.margin < MANUAL_SCOPED_TAIL_MIN_MARGIN {
        return None;
    }
    Some(decision)
}
