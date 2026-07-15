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
    let correction_gate = read("src/correction_core/gate.rs");
    let transition_decision = read("src/typing_transition/decision.rs");

    assert!(
        ime_correction.contains("decide_input_gate(InputGateRequest")
            || ime_correction.contains("resolve_text_correction(CorrectionRequest"),
        "ime_correction.rs must enter the shared correction pipeline"
    );
    assert!(
        !correction_core.contains("mod decision_core;")
            && correction_core.contains("mod gate;")
            && correction_gate.contains("candidate_admission(")
            && correction_gate.contains("gate_candidate_with_origin(")
            && !correction_gate.contains("TransitionDecisionCore"),
        "candidate admission must not own final transition authority"
    );
    assert!(
        transition_decision.contains("struct TransitionDecisionCore")
            && transition_decision.contains("evaluate_candidates")
            && transition_decision.contains("candidate_has_apply_authority")
            && !transition_decision.contains("authorize_gate"),
        "TransitionDecisionCore must own final apply-candidate authority"
    );
}

#[test]
fn hidden_typing_state_is_apply_admission_authority() {
    let transition_mod = read("src/typing_transition/mod.rs");
    let transition_state = read("src/typing_transition/state.rs");
    let transition_decision = read("src/typing_transition/decision.rs");
    let transition_admission = read("src/typing_transition/decision/admission.rs");

    assert!(
        transition_mod.contains("state::LatentTypingState")
            && transition_mod.contains("state_before: LatentTypingState")
            && transition_mod.contains("state_after_predicted: LatentTypingState")
            && transition_mod.contains("l4_signed_signal: L4SignedTransitionSignal")
            && !transition_mod.contains("l4_state_estimate"),
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
        transition_decision.contains("mod admission;")
            && transition_decision.contains("use admission::candidate_has_apply_authority;")
            && transition_admission.contains("fn candidate_has_apply_authority")
            && transition_admission.contains("admit_evaluated_hidden_transition(")
            && transition_admission.contains("latent_known_word_drift_needs_state_proof")
            && transition_admission.contains("latent_context_import")
            && transition_admission.contains("latent_l4_negative_transition_memory")
            && !transition_admission.contains("latent_l4_state_desync_risk"),
        "TransitionDecisionCore must gate apply through latent invariants and signed L4 memory"
    );
}

#[test]
fn decoder_edit_plan_carries_transition_audit_to_outputs() {
    let decoder_edit = read("src/decoder/edit_plan.rs");
    assert!(
        decoder_edit.contains("transition: TransitionAudit")
            && !decoder_edit.contains("pub transition: TransitionAudit")
            && decoder_edit.contains("selected_transition: Option<DecisionTransitionReceipt>")
            && decoder_edit.contains("with_text_edit_input_gate_decision")
            && decoder_edit.contains("pub fn authorize_verified_replacement")
            && decoder_edit.contains("self.selected_transition.is_none()"),
        "DecoderEditPlan must privately carry the DecisionCore receipt, authorize output edits, and block missing receipts before output"
    );

    let daemon_gate = read("src/bin/lay_daemon/typing_assist_runtime/decoder/gate.rs");
    assert!(
        daemon_gate.contains(".with_text_edit_input_gate_decision(&decision)")
            && daemon_gate.contains("edit.authorize_verified_replacement("),
        "typing-assist decoder must bind the opaque input-gate decision receipt into DecoderEditPlan"
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
    let replay_action = read("src/bin/lay_daemon/correction_runtime/output/replay/action.rs");
    assert!(
        replay_action.contains("plan_manual_edit(")
            && replay_action.contains("MutationLogRoute::MANUAL_TEXT_REPLACE")
            && replay_action.contains("authorize_backend_edit(")
            && replay_action.contains("Option<AuthorizedEdit>"),
        "manual replay output must pass through EditAction and ExecutorContract before backspace/replay"
    );
    let authorization = replay
        .find("manual_replay_action(ctx, input_gate)?")
        .expect("manual replay must obtain AuthorizedEdit");
    let first_backspace = replay
        .find("let backspace_result =")
        .expect("manual replay must emit backspaces");
    assert!(
        authorization < first_backspace,
        "manual replay must obtain AuthorizedEdit before its first physical mutation"
    );

    let native = read("src/bin/lay_daemon/correction_runtime/output/native.rs");
    assert!(
        !native.contains("if is_replay {\n        return true;\n    }"),
        "native replay must not bypass EditAction with a direct true"
    );
    assert!(
        replay_action.contains("plan_manual_edit(")
            && native.contains("MutationLogRoute::MANUAL_NATIVE_REPLACE")
            && native.contains("plan_native_edit("),
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
            "src/bin/lay_daemon/correction_runtime/output/replay/action.rs",
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
                && source.contains(".into_authorized()"),
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
            && daemon_pipeline.contains("authorized: AuthorizedEdit")
            && daemon_pipeline.contains("IndeterminateAfterDelete"),
        "daemon replacement pipeline must consume AuthorizedEdit and expose partial destructive execution honestly"
    );
    let layout_controller = read("src/bin/lay_daemon/layout_controller.rs");
    let compact_layout_controller = without_whitespace(&layout_controller);
    assert!(
        compact_layout_controller.contains("try_ime_replace_tail(authorized:AuthorizedEdit")
            && compact_layout_controller.contains("call_replace_text(authorized:AuthorizedEdit"),
        "IME and GNOME bridge mutations must consume AuthorizedEdit"
    );

    assert!(
        !executor.contains("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AuthorizedEdit")
            && executor.contains("pub fn into_authorized(self)"),
        "AuthorizedEdit must be a move-only one-shot mutation capability"
    );

    let replay = read("src/bin/lay_daemon/correction_runtime/output/replay.rs");
    let replay_preflight = read("src/bin/lay_daemon/correction_runtime/output/replay/preflight.rs");
    let layout_preflight = read("src/bin/lay_daemon/text_output/layout_preflight.rs");
    assert!(
        daemon_pipeline.contains("prepared_insert.runs.iter().map(|run| run.target_is_ru)")
            && layout_preflight.contains("for target_is_ru in target_layouts")
            && layout_preflight.contains("switch_to_target_layout(target_is_ru)"),
        "daemon replacement must capability-preflight every required insert layout run"
    );
    assert_before(
        &replay_preflight,
        ".validate_current()",
        "LayoutCapabilityPreflight::run(",
        "manual replay must validate current state before layout capability preflight",
    );
    assert_before(
        &replay_preflight,
        "LayoutCapabilityPreflight::run(",
        "mutation_preflight.consume()",
        "manual replay must consume and revalidate after layout capability preflight",
    );
    assert_before(
        &replay,
        "preflight_manual_replay(ctx)",
        "let backspace_started",
        "manual replay must finish layout and mutation preflight before Backspace",
    );
    assert_before(
        &daemon_pipeline,
        ".validate_current()",
        "LayoutCapabilityPreflight::run(",
        "daemon replacement must validate current state before layout capability preflight",
    );
    assert_before(
        &daemon_pipeline,
        "LayoutCapabilityPreflight::run(",
        "mutation_preflight.consume()",
        "daemon replacement must consume and revalidate after layout capability preflight",
    );
    assert_before(
        &daemon_pipeline,
        "mutation_preflight.consume()",
        "apply_text_replacement(dev, plan, fast_output)",
        "daemon replacement must consume the observed lease before cursor/delete side effects",
    );
    assert_eq!(
        daemon_pipeline
            .matches("mutation_preflight.consume()")
            .count(),
        1,
        "daemon replacement must consume its mutation lease exactly once"
    );
    assert_eq!(
        replay_preflight
            .matches("mutation_preflight.consume()")
            .count(),
        1,
        "manual replay must consume its mutation lease exactly once"
    );
    assert!(
        daemon_pipeline.contains("layout_preflight.restore_initial_best_effort(label)")
            && replay_preflight
                .contains("layout_preflight.restore_initial_best_effort(\"manual replay\")"),
        "failed final validation must restore the known initial layout before returning"
    );
    let prepare_start = daemon_pipeline
        .find("fn prepare_text_insert_for_replacement_plan")
        .expect("replacement prepare function exists");
    let delete_start = daemon_pipeline
        .find("fn apply_text_replacement")
        .expect("replacement apply function exists");
    assert!(
        !daemon_pipeline[prepare_start..delete_start].contains("switch_to_target_layout("),
        "replacement prepare phase must be side-effect free before lease consume"
    );

    let composition = read("src/bin/lay_ibus_engine/composition_commit.rs");
    assert!(
        composition.contains("enum ActiveCompositionAuthority")
            && composition.contains("UserInput")
            && composition.contains("VerifiedEdit(Box<AuthorizedEdit>)")
            && !composition.contains("Option<AuthorizedEdit>"),
        "plain IME input and verified model edits must be distinct typed routes"
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
            && ime.contains("NotSelected")
            && ime.contains("Blocked"),
        "IME output must report execution state instead of collapsing it into Option"
    );
    assert!(
        output.contains("ImeTypingApplyReceipt::Applied")
            && output.contains("ImeTypingApplyReceipt::NotSelected")
            && output.contains("ImeTypingApplyReceipt::Blocked"),
        "daemon fallback must branch on an explicit IME execution receipt"
    );
    assert!(
        output.contains("ImeTypingApplyReceipt::Blocked => return")
            && output.contains("ImeTypingApplyReceipt::NotSelected => {}"),
        "only a pre-dispatch NotSelected receipt may reach the daemon backend"
    );
}

#[test]
fn dispatched_text_edit_cannot_fall_through_to_a_second_backend() {
    let executor = read("src/text_edit/executor.rs");
    assert!(
        executor.contains("enum BackendDispatchReceipt")
            && executor.contains("Self::NotDispatched { .. }")
            && executor.contains("permits_backend_reselection"),
        "shared executor must distinguish pre-dispatch selection from terminal backend outcomes"
    );

    for path in [
        "src/bin/lay_daemon/auto_undo_runtime.rs",
        "src/bin/lay_daemon/enter_autocorrect_runtime.rs",
        "src/bin/lay_daemon/correction_runtime/output/native.rs",
        "src/bin/lay_daemon/typing_assist_runtime/output/ime.rs",
    ] {
        let source = read(path);
        assert!(
            source.contains("permits_backend_reselection"),
            "{path} must fail closed after an IME dispatch"
        );
        assert!(
            !source.contains("fallback to uinput"),
            "{path} must not retry a dispatched edit through uinput"
        );
    }

    let layout_controller = read("src/bin/lay_daemon/layout_controller.rs");
    assert!(
        layout_controller.contains("VisibleTailSource::from_bridge_state(&state).is_some()"),
        "active composition and committed-tail states must share typed IME ownership"
    );

    let ibus_interface = read("src/bin/lay_ibus_engine/ibus_interface.rs");
    assert!(
        ibus_interface.contains("name = \"RequireSurroundingText\"")
            && ibus_interface.contains("Self::require_surrounding_text(&emitter)"),
        "IME enable must activate the standard IBus surrounding-text contract"
    );
}

#[test]
fn typed_authority_mints_have_only_named_runtime_callers() {
    let allowed = [
        (
            "plan_manual_edit(",
            &[
                "src/bin/lay_daemon/correction_runtime/output/replay/action.rs",
                "src/bin/lay_daemon/correction_runtime/output/text_replace.rs",
            ][..],
        ),
        (
            "plan_native_edit(",
            &["src/bin/lay_daemon/correction_runtime/output/native.rs"][..],
        ),
        (
            "plan_recorded_undo_edit(",
            &["src/bin/lay_daemon/auto_undo_runtime.rs"][..],
        ),
        (
            "plan_ime_completion_edit(",
            &[
                "src/bin/lay_ibus_engine/committed_tail.rs",
                "src/bin/lay_ibus_engine/composition_commit.rs",
            ][..],
        ),
    ];

    for path in source_files("src/bin") {
        let relative = path
            .strip_prefix(ROOT)
            .expect("runtime source is under repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let source = std::fs::read_to_string(&path).expect("runtime source");
        for (mint, allowed_paths) in allowed {
            if source.contains(mint) {
                assert!(
                    allowed_paths.contains(&relative.as_str()),
                    "{relative} must not mint transition authority through {mint}"
                );
            }
        }
    }
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

fn assert_before(source: &str, earlier: &str, later: &str, message: &str) {
    let earlier_idx = source.find(earlier).expect("earlier source marker exists");
    let later_idx = source.find(later).expect("later source marker exists");
    assert!(earlier_idx < later_idx, "{message}");
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
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
