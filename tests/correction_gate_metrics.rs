use lay::config::{default_typing_assist_pipeline, CorrectionSafety};
use lay::correction_core::{correction_gate_stats_json, CorrectionMode};
use lay::input_gate::{decide_input_gate, InputGateRequest, InputGateTrigger};

#[test]
fn correction_gate_metrics_are_status_only() {
    let pipeline = default_typing_assist_pipeline();
    let _ = decide_input_gate(InputGateRequest {
        trigger: InputGateTrigger::Space,
        text_tail: "lfdfq ",
        auto_replace: true,
        typing_assist: true,
        auto_switch_layout: true,
        correction_safety: CorrectionSafety::Normal,
        typing_assist_pipeline: &pipeline,
        nanda_autocorrect: false,
        nanda_candidate_route: lay::correction_core::CandidateReadoutRoute::FullWave,
        nanda_wave_options: lay::nanda_wave::WaveOptions::default(),
        correction_mode: CorrectionMode::DeterministicOnly,
    });

    let stats = correction_gate_stats_json();
    assert!(stats["requests"].as_u64().unwrap_or_default() >= 1);
    assert!(stats["total_candidates"].is_u64());
    assert!(stats["apply_candidates"].is_u64());
    assert!(stats["selected_apply"].is_u64());
    assert!(stats["avg_us"].is_u64());
    assert!(stats["max_us"].is_u64());

    let text = serde_json::to_string(&stats).unwrap();
    assert!(!text.contains("lfdfq"));
    assert!(!text.contains("давай"));
}
