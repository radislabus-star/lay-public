use super::feedback::OnlineState;

/// Selects one relation for an impact probe. One relation is the smallest
/// publishable delta; targeted proof measures useful movement first and the
/// frozen differential proof then requires zero old-basin loss.
pub(super) fn select_impact_probe(state: &OnlineState) -> Option<String> {
    state
        .pending
        .iter()
        .filter(|(_, relation)| relation.ready_for_impact_probe())
        .max_by(|(left_key, left), (right_key, right)| {
            left.distinct_scenes()
                .cmp(&right.distinct_scenes())
                .then_with(|| {
                    left.independent_episodes()
                        .cmp(&right.independent_episodes())
                })
                .then_with(|| left.last_observed_ordinal.cmp(&right.last_observed_ordinal))
                // A lexical tie-break is deterministic but never outranks
                // causal evidence or recency.
                .then_with(|| right_key.cmp(left_key))
        })
        .map(|(key, _)| key.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l3_online::feedback::PendingRelation;

    fn relation(
        rejected: &str,
        scenes: &[&str],
        episodes: &[&str],
        ordinal: u64,
    ) -> PendingRelation {
        PendingRelation {
            rejected: rejected.to_string(),
            expected: "target".to_string(),
            scenes: scenes.iter().map(|scene| (*scene).to_string()).collect(),
            episode_ids: episodes.iter().map(|id| (*id).to_string()).collect(),
            last_attempted_episodes: 0,
            last_observed_ordinal: ordinal,
        }
    }

    #[test]
    fn selector_requires_independent_episodes_and_scene_diversity() {
        let mut state = OnlineState::default();
        state.pending.insert(
            "same-scene".to_string(),
            relation("same", &["one scene"], &["e1", "e2"], 9),
        );
        state.pending.insert(
            "one-episode".to_string(),
            relation("one", &["scene one", "scene two"], &["e1"], 10),
        );

        assert_eq!(select_impact_probe(&state), None);
    }

    #[test]
    fn selector_prefers_more_diverse_then_more_recent_evidence() {
        let mut state = OnlineState::default();
        state.pending.insert(
            "older".to_string(),
            relation("older", &["scene one", "scene two"], &["e1", "e2"], 8),
        );
        state.pending.insert(
            "newer".to_string(),
            relation("newer", &["scene three", "scene four"], &["e3", "e4"], 9),
        );

        assert_eq!(select_impact_probe(&state).as_deref(), Some("newer"));
    }
}
