use lay::nanda_wave::candidate_gate::{
    live_candidate_gate_stats_json, live_completion_candidates, LiveCompletionRequest,
};
use lay::typing_cpu::{
    select_ime_candidate_proposals, ImeCandidateProposal, ImeCandidateReadoutRequest,
    ImeCandidateSource, TypingCpu,
};

#[test]
fn live_candidate_gate_metrics_are_status_only() {
    let _ = live_completion_candidates(LiveCompletionRequest {
        context_prefix: "на улице опять идет",
        partial: "до",
        max_suffix_chars: 8,
        active_composition: true,
        allow_short_lexical: true,
        limit: 4,
    });

    let stats = live_candidate_gate_stats_json();
    assert!(stats["requests"].as_u64().unwrap_or_default() >= 1);
    assert!(stats["raw_candidates"].is_u64());
    assert!(stats["returned_candidates"].is_u64());
    assert!(stats["avg_us"].is_u64());
    assert!(stats["max_us"].is_u64());
    assert!(stats["l3_evaluated"].is_u64());
    assert!(stats["l3_suppressed"].is_u64());
    assert!(stats["l4_signed_outcome"]["attract"].is_u64());
    assert!(stats["l4_signed_outcome"]["neutral"].is_u64());
    assert!(stats["l4_signed_outcome"]["repel"].is_u64());

    let text = serde_json::to_string(&stats).unwrap();
    assert!(!text.contains("улице"));
    assert!(!text.contains("до"));
}

#[test]
fn ime_readout_keeps_replacement_as_a_non_mutating_typed_proposal() {
    let proposals = [ImeCandidateProposal::replacement(
        "работает",
        0.91,
        ImeCandidateSource::L2Replacement,
    )];

    let selected = select_ime_candidate_proposals(ImeCandidateReadoutRequest {
        proposals: &proposals,
        limit: 4,
    });

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].replacement.as_deref(), Some("работает"));
    assert!(selected[0].suffix.is_empty());
}

#[test]
fn live_l2_surfaces_current_token_repair_before_space() {
    TypingCpu::warm_l2_for_ime();
    let raw = lay::nanda_wave::l2::ime_l2_word_candidates("код", "рабоает", 48);
    let candidates = TypingCpu::live_completion_candidates(LiveCompletionRequest {
        context_prefix: "код",
        partial: "рабоает",
        max_suffix_chars: 16,
        active_composition: true,
        allow_short_lexical: true,
        limit: 12,
    });

    assert!(
        candidates.iter().any(|candidate| {
            candidate.surface == "работает" && candidate.suffix.is_empty()
        }),
        "raw={raw:?}, selected={candidates:?}"
    );
    assert_eq!(
        candidates
            .first()
            .map(|candidate| candidate.surface.as_str()),
        Some("работает"),
        "the strongest L2 center must remain the default visible replacement: {candidates:?}"
    );
}

fn live_candidates(partial: &str) -> Vec<lay::typing_cpu::LiveCompletionCandidate> {
    TypingCpu::live_completion_candidates(LiveCompletionRequest {
        context_prefix: "",
        partial,
        max_suffix_chars: 16,
        active_composition: true,
        allow_short_lexical: true,
        limit: 12,
    })
}

#[test]
fn full_token_replacement_requires_target_evidence_not_only_an_operator_lane() {
    TypingCpu::warm_l2_for_ime();

    let layout = live_candidates("ytn");
    let one_edit = live_candidates("рабоает");
    let repeated_letter = live_candidates("относитться");
    let weak_corrected_prefix = live_candidates("относитт");

    eprintln!("layout={layout:?}");
    eprintln!("one_edit={one_edit:?}");
    eprintln!("repeated_letter={repeated_letter:?}");
    eprintln!("weak_corrected_prefix={weak_corrected_prefix:?}");

    assert!(layout.iter().any(|candidate| {
        candidate.replacement
            && candidate.surface == "нет"
            && candidate.source == "L1ExactLayoutCell32"
    }));
    assert!(
        layout
            .iter()
            .filter(|candidate| candidate.replacement)
            .all(|candidate| candidate.surface == "нет"),
        "exact layout authority must not leak same-script lexical arrows: {layout:?}"
    );
    assert!(one_edit
        .iter()
        .any(|candidate| { candidate.replacement && candidate.surface == "работает" }));
    assert!(repeated_letter
        .iter()
        .any(|candidate| { candidate.replacement && candidate.surface == "относиться" }));
    assert!(repeated_letter
        .iter()
        .all(|candidate| candidate.surface != "относи ться"));
    assert!(weak_corrected_prefix.iter().all(|candidate| {
        !candidate.replacement
            || (candidate.surface.starts_with("относ") && !candidate.surface.starts_with("относим"))
    }));
}

#[test]
fn known_russian_states_do_not_publish_unrelated_full_token_replacements() {
    TypingCpu::warm_l2_for_ime();

    for partial in ["какое", "новая", "точнее", "относится"] {
        let candidates = live_candidates(partial);
        assert!(
            candidates.iter().all(|candidate| !candidate.replacement),
            "known state {partial:?} published a replacement arrow: {candidates:?}"
        );
    }
}

#[test]
fn damaged_russian_states_only_publish_operator_bound_replacements() {
    TypingCpu::warm_l2_for_ime();

    for partial in ["появлт", "предлает"] {
        let candidates = live_candidates(partial);
        assert!(
            candidates
                .iter()
                .filter(|candidate| candidate.replacement)
                .all(|candidate| replacement_is_bound_to_observed_prefix(
                    partial,
                    &candidate.surface
                )),
            "damaged state {partial:?} published an unrelated replacement arrow: {candidates:?}"
        );
    }
}

fn replacement_is_bound_to_observed_prefix(observed: &str, target: &str) -> bool {
    if lay::text_metrics::damerau_levenshtein(observed, target) == 1 {
        return true;
    }
    let observed_len = observed.chars().count();
    let target_chars = target.chars().collect::<Vec<_>>();
    [
        observed_len.saturating_sub(1),
        observed_len,
        observed_len.saturating_add(1),
    ]
    .into_iter()
    .filter(|prefix_len| *prefix_len >= 2 && *prefix_len < target_chars.len())
    .any(|prefix_len| {
        let target_prefix = target_chars[..prefix_len].iter().collect::<String>();
        lay::text_metrics::damerau_levenshtein(observed, &target_prefix) == 1
    })
}
