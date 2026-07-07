use std::path::{Path, PathBuf};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn ime_composition_does_not_own_input_gate_decision() {
    let source = read("src/bin/lay_ibus_engine/composition_commit.rs");
    let direct_gate_call = ["decide", "_input_gate("].concat();
    let direct_gate_request = ["InputGate", "Request {"].concat();

    assert!(
        !source.contains(&direct_gate_call) && !source.contains(&direct_gate_request),
        "composition_commit.rs must delegate correction decisions to lay::ime_correction"
    );
}

#[test]
fn ime_correction_route_reaches_common_decision_core() {
    let ime_correction = read("src/ime_correction.rs");
    let correction_core = read("src/correction_core.rs");
    let decision_core = read("src/correction_core/decision_core.rs");

    assert!(
        ime_correction.contains("decide_input_gate(InputGateRequest")
            || ime_correction.contains("resolve_text_correction(CorrectionRequest"),
        "ime_correction.rs must enter the shared correction pipeline"
    );
    assert!(
        correction_core.contains("mod decision_core;"),
        "correction_core must own the DecisionCore module"
    );
    assert!(
        decision_core.contains("struct CorrectionDecisionCore")
            && decision_core.contains("select_apply_candidate"),
        "DecisionCore must own final apply-candidate selection"
    );
}

#[test]
fn ime_preedit_display_does_not_own_correction_or_apply() {
    let source = read("src/bin/lay_ibus_engine/preedit.rs");
    let forbidden = [
        "decide_input_gate",
        "resolve_text_correction",
        "decide_text_transition",
        "authorize_replacement",
        "commit_text(",
        "replace_committed_tail(",
        "record_candidate_edit_action_before_apply(",
    ];

    for needle in forbidden {
        assert!(
            !source.contains(needle),
            "preedit.rs must stay display-only and must not contain {needle}"
        );
    }
}

#[test]
fn dead_ime_pending_space_autocorrect_route_stays_deleted() {
    for path in source_files("src/bin/lay_ibus_engine") {
        let source = std::fs::read_to_string(&path).expect("source file");
        assert!(
            !source.contains("pending_space_committed_tail_replace")
                && !source.contains("PendingSpaceCommittedTailReplace")
                && !source.contains("apply_pending_committed_tail_space_autocorrect"),
            "{} must not revive pending committed-tail Space autocorrect",
            path.display()
        );
    }
}

#[test]
fn candidate_before_apply_logs_use_typed_mutation_routes() {
    let mut files = source_files("src/bin");
    files.push(Path::new(ROOT).join("src/action_log_tests.rs"));
    for path in files {
        let source = std::fs::read_to_string(&path).expect("source file");
        if !source.contains("record_candidate_edit_action_before_apply(") {
            continue;
        }
        for call in source.match_indices("record_candidate_edit_action_before_apply(") {
            let tail = &source[call.0..source.len().min(call.0 + 240)];
            assert!(
                tail.contains("MutationLogRoute::"),
                "{} has candidate_before_apply call without typed MutationLogRoute near byte {}",
                path.display(),
                call.0
            );
        }
    }
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(relative)).expect("source file")
}

fn source_files(relative_dir: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_rs_files(&Path::new(ROOT).join(relative_dir), &mut out);
    out
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read source dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
