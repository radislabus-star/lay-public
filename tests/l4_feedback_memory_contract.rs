use std::fs;

#[test]
fn rejected_transition_compiles_into_exact_and_phase_memory() {
    let root =
        std::env::temp_dir().join(format!("lay-l4-feedback-contract-{}", std::process::id()));
    let input = root.join("events.jsonl");
    let output = root.join("feedback-counts.json");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        &input,
        r#"{"ts":1,"kind":"rejected_candidate","word":"так","from":"nfr","to":"так","source":"typing-assist","operation":"layout","surface":"op=layout_projection|shape=replace|words=1:1"}
"#,
    )
    .unwrap();

    let report = lay::nanda_wave::compile_usage_feedback_snapshot(&input, &output).unwrap();
    let snapshot: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();

    assert_eq!(report["status"], "ok");
    assert_eq!(report["parsed_events"], 1);
    assert_eq!(report["surface_anti_states"], 1);
    assert!(snapshot["counts"]["transition_repel"]
        .as_object()
        .is_some_and(|states| !states.is_empty()));
    assert!(snapshot["counts"]["surface_repel"]
        .as_object()
        .is_some_and(|states| !states.is_empty()));

    let _ = fs::remove_dir_all(root);
}
