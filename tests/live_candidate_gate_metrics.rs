use lay::nanda_wave::candidate_gate::{
    live_candidate_gate_stats_json, live_completion_candidates, LiveCompletionRequest,
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
    assert!(stats["l4_signed_outcome"]["attract"].is_u64());
    assert!(stats["l4_signed_outcome"]["neutral"].is_u64());
    assert!(stats["l4_signed_outcome"]["repel"].is_u64());

    let text = serde_json::to_string(&stats).unwrap();
    assert!(!text.contains("улице"));
    assert!(!text.contains("до"));
}
