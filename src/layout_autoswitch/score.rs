//! Layout-candidate scoring.

const LEM_LAYOUT_AUTOSWITCH_MARGIN: f64 = 0.25;

pub(super) fn lem_prefers_layout_candidate(typed: &str, candidate: &str) -> bool {
    let ranked = crate::lem::rank_candidates(typed, [typed.to_string(), candidate.to_string()]);
    let Some(best) = ranked.first() else {
        return false;
    };
    if best.text != candidate {
        return false;
    }

    let margin = ranked
        .get(1)
        .map(|second| best.total - second.total)
        .unwrap_or(f64::INFINITY);
    margin >= LEM_LAYOUT_AUTOSWITCH_MARGIN
}
