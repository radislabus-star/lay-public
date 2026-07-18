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
