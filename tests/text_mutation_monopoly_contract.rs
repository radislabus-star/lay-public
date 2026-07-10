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
    let transition_decision = read("src/typing_transition/decision.rs");

    assert!(
        ime_correction.contains("decide_input_gate(InputGateRequest")
            || ime_correction.contains("resolve_text_correction(CorrectionRequest"),
        "ime_correction.rs must enter the shared correction pipeline"
    );
    assert!(
        !correction_core.contains("mod decision_core;")
            && correction_core.contains("TransitionDecisionCore::authorize_gate"),
        "correction_core must delegate apply authority to typing_transition"
    );
    assert!(
        transition_decision.contains("struct TransitionDecisionCore")
            && transition_decision.contains("select_apply_candidate")
            && transition_decision.contains("authorize_gate"),
        "TransitionDecisionCore must own final apply-candidate authority"
    );
}

#[test]
fn hidden_typing_state_is_apply_admission_authority() {
    let transition_mod = read("src/typing_transition/mod.rs");
    let transition_state = read("src/typing_transition/state.rs");
    let transition_decision = read("src/typing_transition/decision.rs");

    assert!(
        transition_mod.contains("state::LatentTypingState")
            && transition_mod.contains("state_before: LatentTypingState")
            && transition_mod.contains("state_after_predicted: LatentTypingState"),
        "TypingTransition must carry latent state before/after the candidate action"
    );
    assert!(
        transition_state.contains("struct LatentTypingState")
            && transition_state.contains("context_words")
            && transition_state.contains("known_word_drift_to")
            && transition_state.contains("candidate_imported_left_context"),
        "LatentTypingState must expose context, drift, and left-context import invariants"
    );
    assert!(
        transition_decision.contains("fn candidate_has_apply_authority")
            && transition_decision.contains("admit_hidden_transition(")
            && transition_decision.contains("latent_known_word_drift_needs_state_proof")
            && transition_decision.contains("latent_context_import"),
        "TransitionDecisionCore must gate apply through hidden-state admission"
    );
}

#[test]
fn decoder_edit_plan_carries_transition_audit_to_outputs() {
    let decoder_edit = read("src/decoder/edit_plan.rs");
    assert!(
        decoder_edit.contains("transition: TransitionAudit")
            && !decoder_edit.contains("pub transition: TransitionAudit")
            && decoder_edit.contains("with_input_gate_trace")
            && decoder_edit.contains("pub fn authorize_verified_replacement")
            && decoder_edit.contains("self.transition.blocks_apply()"),
        "DecoderEditPlan must privately carry transition audit, authorize output edits, and block unverified plans before output"
    );

    let daemon_gate = read("src/bin/lay_daemon/typing_assist_runtime/decoder/gate.rs");
    assert!(
        daemon_gate.contains("edit.with_input_gate_trace")
            && daemon_gate.contains("edit.authorize_verified_replacement("),
        "typing-assist decoder must bind input-gate transition audit into DecoderEditPlan"
    );

    let ime_output = read("src/bin/lay_daemon/typing_assist_runtime/output/ime.rs");
    let minimal_output = read("src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs");
    for (path, source) in [
        (
            "src/bin/lay_daemon/typing_assist_runtime/output/ime.rs",
            ime_output,
        ),
        (
            "src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs",
            minimal_output,
        ),
    ] {
        assert!(
            source.contains("edit.authorize_verified_replacement(")
                && !source.contains("edit.transition.clone()")
                && !source.contains("edit.selected_source_id.as_deref()")
                && !source.contains("edit.selected_error_class.as_deref()"),
            "{path} must execute through DecoderEditPlan without reading transition internals"
        );
    }
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
fn ime_autocorrect_text_helpers_stay_deleted() {
    for path in source_files("src/bin/lay_ibus_engine") {
        let source = std::fs::read_to_string(&path).expect("source file");
        assert!(
            !source.contains("autocorrect_active_composition_text(")
                && !source.contains("autocorrect_committed_tail_text("),
            "{} must use ime_correction decision objects instead of local autocorrect text helpers",
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
            let tail: String = source[call.0..].chars().take(240).collect();
            assert!(
                tail.contains("MutationLogRoute::"),
                "{} has candidate_before_apply call without typed MutationLogRoute near byte {}",
                path.display(),
                call.0
            );
        }
    }
}

#[test]
fn manual_replay_paths_are_edit_action_gated() {
    let replay = read("src/bin/lay_daemon/correction_runtime/output/replay.rs");
    assert!(
        replay.contains("authorize_replacement_with_transition(")
            && replay.contains("MutationLogRoute::MANUAL_TEXT_REPLACE")
            && replay.contains("authorize_backend_edit("),
        "manual replay output must pass through EditAction and ExecutorContract before backspace/replay"
    );

    let native = read("src/bin/lay_daemon/correction_runtime/output/native.rs");
    assert!(
        !native.contains("if is_replay {\n        return true;\n    }"),
        "native replay must not bypass EditAction with a direct true"
    );
    assert!(
        replay.contains("manual_replay_plan_verified")
            && native.contains("MutationLogRoute::MANUAL_NATIVE_REPLACE")
            && native.contains("manual_native_replay_plan_verified"),
        "manual replay output must log through typed manual routes and carry replay transition proof"
    );

    let output = read("src/bin/lay_daemon/correction_runtime/output.rs");
    assert!(
        output.contains("apply_layout_replay(&mut common, kbd, input_gate)"),
        "manual replay must preserve the input_gate trace into the EditAction log"
    );
}

#[test]
fn live_text_mutation_outputs_use_executor_contract() {
    let cases = [
        (
            "src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs",
            "TextEditBackend::Daemon",
        ),
        (
            "src/bin/lay_daemon/typing_assist_runtime/output/ime.rs",
            "TextEditBackend::Ime",
        ),
        (
            "src/bin/lay_daemon/correction_runtime/output/text_replace.rs",
            "TextEditBackend::Daemon",
        ),
        (
            "src/bin/lay_daemon/correction_runtime/output/replay.rs",
            "TextEditBackend::Daemon",
        ),
        (
            "src/bin/lay_daemon/correction_runtime/output/native.rs",
            "TextEditBackend::Ime",
        ),
        (
            "src/bin/lay_daemon/correction_runtime/output/native.rs",
            "TextEditBackend::Daemon",
        ),
        (
            "src/bin/lay_daemon/auto_undo_runtime.rs",
            "TextEditBackend::Daemon",
        ),
        (
            "src/bin/lay_daemon/enter_autocorrect_runtime.rs",
            "TextEditBackend::Daemon",
        ),
        (
            "src/bin/lay_ibus_engine/composition_commit.rs",
            "TextEditBackend::Ime",
        ),
        (
            "src/bin/lay_ibus_engine/committed_tail.rs",
            "TextEditBackend::Ime",
        ),
    ];

    for (path, backend) in cases {
        let source = read(path);
        assert!(
            source.contains("authorize_backend_edit(")
                && source.contains(backend)
                && source.contains(".authorized()"),
            "{path} must hold AuthorizedEdit before physical mutation through ExecutorContract for {backend}"
        );
    }

    let executor_contract = read("src/typing_transition/executor_contract.rs");
    assert!(
        !executor_contract.contains("#![allow(dead_code)]"),
        "ExecutorContract must be live architecture, not an unused declaration"
    );
    assert!(
        read("src/text_edit.rs").contains("authorize_backend_edit"),
        "bin backends must reach ExecutorContract through the shared text_edit API"
    );
    let executor = read("src/text_edit/executor.rs");
    assert!(
        executor.contains("pub struct AuthorizedEdit")
            && executor.contains("authorized: Option<AuthorizedEdit"),
        "executor must issue a sealed AuthorizedEdit capability"
    );

    let daemon_pipeline = read("src/bin/lay_daemon/text_output/replacement.rs");
    assert!(
        daemon_pipeline.contains("pub(crate) fn apply_text_replacement_pipeline")
            && daemon_pipeline.contains("authorized: &AuthorizedEdit"),
        "daemon replacement pipeline must require AuthorizedEdit rather than raw plan/text"
    );
    let layout_controller = read("src/bin/lay_daemon/layout_controller.rs");
    assert!(
        layout_controller.contains("try_ime_replace_tail(\n    authorized: &AuthorizedEdit")
            && layout_controller.contains("call_replace_text(\n    authorized: &AuthorizedEdit"),
        "IME and GNOME bridge mutations must require AuthorizedEdit"
    );
}

#[test]
fn typing_assist_boundary_event_cannot_schedule_a_second_apply() {
    for path in [
        "src/bin/lay_daemon/typing_assist_runtime/output/ime.rs",
        "src/bin/lay_daemon/typing_assist_runtime/output/minimal.rs",
    ] {
        let source = read(path);
        assert!(
            !source.contains("next_correction_after_forwarded_spaces"),
            "{path} must finish one verified transition instead of recursively applying a second correction"
        );
    }

    assert!(
        !Path::new(ROOT)
            .join("src/bin/lay_daemon/typing_assist_runtime/output/queued.rs")
            .exists(),
        "forwarded-space correction queue must not be revived without a new typed transition route"
    );
}

#[test]
fn typing_assist_ime_fallback_requires_a_typed_receipt() {
    let ime = read("src/bin/lay_daemon/typing_assist_runtime/output/ime.rs");
    let output = read("src/bin/lay_daemon/typing_assist_runtime/output.rs");
    assert!(
        ime.contains("enum ImeTypingApplyReceipt")
            && ime.contains("Applied(TypingAssistOutcome)")
            && ime.contains("Unavailable")
            && ime.contains("Blocked"),
        "IME output must report execution state instead of collapsing it into Option"
    );
    assert!(
        output.contains("ImeTypingApplyReceipt::Applied")
            && output.contains("ImeTypingApplyReceipt::Unavailable")
            && output.contains("ImeTypingApplyReceipt::Blocked"),
        "daemon fallback must branch on an explicit IME execution receipt"
    );
}

#[test]
fn changed_check_runs_shadow_replay_release_gate() {
    let script = read("scripts/check-lay-changed.sh");
    assert!(
        script.contains("--transition-replay") && script.contains("--unsafe-gate"),
        "changed check must promote shadow replay and unsafe edit scan to release gates"
    );
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
