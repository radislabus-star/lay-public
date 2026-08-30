#![cfg(feature = "research-tools")]

use std::path::PathBuf;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn feedback_overlay_keeps_live_text_out_of_the_compiled_packet() {
    let dir = std::env::temp_dir().join(format!(
        "lay-l3-context-feedback-contract-{}",
        std::process::id()
    ));
    let events = dir.join("events.jsonl");
    let output = dir.join("overlay.nwpc");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        &events,
        concat!(
            r#"{"kind":"accepted_ime","word":"всегда","context":["установи","в"]}"#,
            "\n",
            r#"{"kind":"rejected_ime","word":"словом","context":["я","просил","записать","слово","в"]}"#,
            "\n",
            r#"{"kind":"typed","word":"шум","context":["на","улице"]}"#,
        ),
    )
    .unwrap();
    let base = PathBuf::from(ROOT).join("data/lexicon/l3_context_phase_v1.nwpc");

    let report =
        lay::nanda_wave::compile_l3_context_feedback_overlay_memory(&base, &events, &output)
            .expect("feedback overlay must compile");

    assert_eq!(report["kind"], "l3_context_phase_feedback_overlay");
    assert_eq!(report["raw_words_stored"], false);
    assert_eq!(report["positive_admitted"], 0);
    assert_eq!(report["negative_admitted"], 0);
    assert_eq!(report["positive_censored_pending_surface_support"], 1);
    assert_eq!(report["negative_censored_no_observed_target"], 1);
    assert_eq!(report["source_events"], 2);
    let bytes = std::fs::read(&output).unwrap();
    assert!(!bytes
        .windows("всегда".len())
        .any(|window| window == "всегда".as_bytes()));
    let _ = std::fs::remove_dir_all(dir);
}
