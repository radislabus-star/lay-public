use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(path)).expect("source file")
}

#[test]
fn l3_context_metric_has_an_isolated_causal_ablation() {
    let l3 = read("src/nanda_wave/l3.rs");
    let report = read("src/nanda_wave/l3_context_metrics.rs");

    assert!(
        l3.contains("pub(crate) const L3_CONTEXT_FIELD_CELL")
            && l3.contains("options.is_enabled(L3_CONTEXT_FIELD_CELL)")
            && l3.contains("options.l3_weight() <= f32::EPSILON"),
        "L3 context must have an explicit ablation boundary"
    );
    assert!(
        report.contains("warm_up_l3_phrase_memory")
            && report.contains("run_l3_with_options(&case.original, &full.l2_candidates")
            && !report.contains("recent_actions.jsonl")
            && !report.contains("ibus_engine_debug.jsonl"),
        "the report must warm memory and reuse one L2 lattice without reading live logs"
    );
}

#[test]
fn l3_context_metric_exposes_connection_authority_and_utility_separately() {
    let report = read("src/nanda_wave/l3_context_metrics.rs");
    let cli = read("src/bin/lay_nanda_wave_eval.rs");

    for field in [
        "evidence_hit_cases",
        "authority_cases",
        "output_changed_cases",
        "improved_cases",
        "worsened_cases",
        "candidate_lattice_drift_cases",
    ] {
        assert!(report.contains(field), "missing L3 metric: {field}");
    }
    assert!(
        cli.contains("--l3-context-report"),
        "the causal report must be reachable from the eval CLI"
    );
}
