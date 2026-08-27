use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(path)).expect("source file")
}

#[test]
fn visible_tail_decision_delegates_to_transition_core() {
    let facade = read("src/text_edit/transition.rs");
    let core = read("src/typing_transition/decision.rs");
    let verifier = read("src/text_edit/structural_verifier.rs");

    assert!(
        facade.contains("TransitionDecisionCore::decide_visible_text_transition"),
        "text_edit transition must be an adapter into the shared decision core"
    );
    assert!(
        core.contains("fn decide_visible_text_transition")
            && verifier.contains("TextTransitionDecision::Apply"),
        "TransitionDecisionCore must own the visible-tail entrypoint and structural verifier must build apply/reject"
    );
}

#[test]
fn visible_tail_bridge_carries_focus_and_epoch_to_the_transition_core() {
    let bridge = read("src/bin/lay_ibus_engine/bridge.rs");
    let bridge_actions = read("src/bin/lay_ibus_engine/bridge_actions.rs");
    let daemon_bridge = read("src/bin/lay_daemon/layout_controller/ime_bridge.rs");
    let visible_tail = read("src/text_edit/visible_tail.rs");
    let transition = read("src/text_edit/transition.rs");

    assert!(
        bridge.contains("VisibleTailV2") && bridge.contains("ReplaceTailV4"),
        "the live IME bridge must expose a revisioned snapshot and replacement call"
    );
    assert!(
        daemon_bridge.contains("visible_tail_v2()?")
            && daemon_bridge.contains("\"ReplaceTailV4\"")
            && daemon_bridge.contains("expected_epoch")
            && daemon_bridge.contains("expected_focus"),
        "daemon replacement must preflight and forward the observed epoch/focus"
    );
    assert!(
        bridge_actions.contains("unwrap_or_else(|| (engine.tail_epoch, path.clone()))"),
        "legacy bridge callers must inherit the current epoch instead of manufacturing revision zero"
    );
    assert!(
        visible_tail.contains("matches_source_focus_and_epoch")
            && transition.contains("StaleVisibleRevision"),
        "visible-tail admission must reject stale revisions"
    );
}

#[test]
fn double_shift_exact_auto_undo_is_a_protected_first_priority_contract() {
    let shift = read("src/bin/lay_ibus_engine/shift.rs");
    let verifier = read("src/text_edit/structural_verifier.rs");
    let transition = read("src/text_edit/transition.rs");

    let undo = shift
        .find("undo_last_ime_autocorrect(emitter)")
        .expect("double Shift exact undo route");
    let manual = shift
        .find("self.manual_toggle_authority()")
        .expect("manual toggle fallback");

    assert!(
        undo < manual && shift.contains("PROTECTED USER CONTRACT"),
        "double Shift must attempt exact autocorrect undo before any manual/layout toggle"
    );
    assert!(
        verifier.contains("plan_recorded_undo_edit")
            && verifier.contains("TextTransitionIntent::ImeAutoUndo")
            && transition.contains("Self::ImeAutoUndo => TransitionOperator::Undo")
            && transition.contains("Self::ImeAutoUndo => TransitionProof::UndoRecord"),
        "exact autocorrect undo must retain RecordedUndo authority and UndoRecord proof"
    );
}

#[test]
fn physical_double_shift_has_one_daemon_to_ime_event_bridge() {
    let event = read("src/bin/lay_daemon/manual_trigger_runtime/event.rs");
    let fire = read("src/bin/lay_daemon/manual_trigger_runtime/fire.rs");
    let dispatch = read("src/bin/lay_daemon/manual_trigger_runtime/ime.rs");
    let controller = read("src/bin/lay_daemon/layout_controller/ime_manual_toggle.rs");
    let daemon_bridge = read("src/bin/lay_daemon/layout_controller/ime_bridge.rs");
    let ime_bridge = read("src/bin/lay_ibus_engine/bridge.rs");

    assert!(
        event.contains("handle_confirmed_double_shift(ctx, now)")
            && event.contains("fire_configured_manual_trigger(ctx.fire_context())")
            && fire.contains("dispatch_ime_manual_toggle(ctx.buffer)"),
        "the physical p-r-p-r sequence must enter the single IME dispatch adapter"
    );
    assert!(
        dispatch.contains("run_ime_manual_toggle()")
            && controller.contains("ime_bridge::manual_toggle()")
            && daemon_bridge.contains("call_ime_noarg(\"ManualToggleV3\")")
            && ime_bridge.contains("async fn manual_toggle_v3"),
        "the daemon trigger must reach the focused IME through one ManualToggleV3 call"
    );
}

#[test]
fn ime_target_continuity_and_bridge_replay_share_state_transition_contracts() {
    let preedit = read("src/bin/lay_ibus_engine/preedit.rs");
    let engine = read("src/bin/lay_ibus_engine/engine.rs");
    let transition = read("src/text_edit/transition.rs");
    let state = read("src/bin/lay_ibus_engine/state.rs");

    assert!(
        preedit.contains("target_surface: Option<String>")
            && preedit.contains("stable_candidate_index")
            && !preedit.contains("retarget_blocked_partial")
            && !preedit.contains("block_retarget_for")
            && !engine.contains("preedit_target_surface"),
        "temporal candidate continuity must be private fast-state and may not create a blank retarget frame"
    );
    assert!(
        transition.contains("TextTransitionDecision::AlreadyApplied")
            && state.contains("target_state_already_observed"),
        "a bridge replay of an already visible target must converge without backend output"
    );
}

#[test]
fn candidate_lattice_merges_typed_evidence_without_owner_priority() {
    let lattice = read("src/typing_transition/candidate.rs");
    let candidate = read("src/correction_core.rs");
    let sources = read("src/correction_core/candidate_sources.rs");

    assert!(
        lattice.contains("existing.merge_evidence(candidate)")
            && !lattice.contains("source_owner_priority"),
        "equal candidate surfaces must merge evidence instead of replacing one owner with another"
    );
    assert!(
        candidate.contains("pub(crate) origin: CandidateOrigin")
            && candidate.contains("pub(crate) evidence: Vec<CandidateEvidence>")
            && candidate.contains("fn merge_evidence"),
        "candidate authority must carry typed origin and merged evidence"
    );
    assert!(
        sources.contains("deterministic_composite_text_candidates")
            && sources.contains("lattice.extend_source")
            && !sources.contains("layout_then_typo_candidate(req, pipeline)\n        .or_else"),
        "operator families must reach the lattice together instead of choosing by source order"
    );
}

#[test]
fn transition_core_uses_typed_origin_for_verifier_and_memory() {
    let transition = read("src/typing_transition/mod.rs");
    let decision = read("src/typing_transition/decision.rs");
    let decision_signals = read("src/typing_transition/decision_signals.rs");

    assert!(
        transition.contains("origin: CandidateOrigin")
            && transition.contains("Decision and verifier authority use `origin`"),
        "source_id must be diagnostic provenance, while typed origin carries authority"
    );
    assert!(
        decision.contains("candidate.origin")
            && decision_signals.contains("candidate.origin.memory_key()"),
        "decision and signed memory must read typed origin rather than source-id strings"
    );
    let bayes = read("src/correction_bayes.rs");
    assert!(
        bayes.contains("origin: CandidateOrigin") && bayes.contains("source: origin.memory_key()"),
        "Bayes and signed-memory readout must consume typed candidate origin"
    );
}

#[test]
fn ime_preedit_uses_shared_candidate_readout_for_ranking() {
    let preedit = read("src/bin/lay_ibus_engine/preedit.rs");
    let preedit_readout = read("src/bin/lay_ibus_engine/preedit_readout.rs");
    let readout = read("src/typing_cpu/candidate.rs");
    let candidate_gate = read("src/nanda_wave/candidate_gate.rs");
    let live_core = read("src/typing_transition/live_candidate.rs");

    assert!(
        !preedit.contains("select_ime_candidate_proposals(ImeCandidateReadoutRequest")
            && preedit_readout
                .contains("select_ime_candidate_proposals(ImeCandidateReadoutRequest")
            && preedit_readout.contains("pub(crate) fn materialize_precognition_candidates(")
            && !preedit.contains("preedit_suffix_bayes_score"),
        "IME renderer must leave material acquisition and ranking in its readout adapter"
    );
    assert!(
        readout.contains("pub fn select_ime_candidate_suffixes")
            && readout.contains("TransitionDecisionCore::select_ime_readout")
            && !readout.contains("cached_usage_prior_snapshot"),
        "shared readout must delegate one final choice to TransitionDecisionCore"
    );
    let post_decision_gate = candidate_gate
        .split("TransitionDecisionCore::select_live_completions")
        .nth(1)
        .and_then(|tail| tail.split("fn live_l2_word_candidates").next())
        .unwrap_or_default();
    assert!(
        candidate_gate.contains("TransitionDecisionCore::select_live_completions")
            && !post_decision_gate.contains("sort_by")
            && live_core.contains("fn select_live_completions")
            && live_core.contains("fn select_ime_readout"),
        "live completion admission and final IME ordering must have one decision owner"
    );
    assert!(
        preedit_readout.contains(".with_authority_order(order)")
            && live_core.contains("proposal.authority_order"),
        "IBus must preserve the shared lattice order instead of re-ranking L2 candidates by UI confidence"
    );
}

#[test]
fn daemon_uses_typing_cpu_as_its_nanda_runtime_front_door() {
    let typing_cpu = read("src/typing_cpu/runtime.rs");
    for path in [
        "src/bin/lay_daemon/boundary_runtime/space.rs",
        "src/bin/lay_daemon/config_runtime/nanda.rs",
        "src/bin/lay_daemon/learning_runtime.rs",
        "src/bin/lay_daemon/nanda_precognition_runtime.rs",
        "src/bin/lay_daemon/startup_runtime/warmup.rs",
        "src/bin/lay_daemon/typing_assist_runtime/output/nanda_trace.rs",
        "src/bin/lay_daemon/correction_runtime/memory.rs",
    ] {
        let source = read(path);
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);
        assert!(
            !production_source.contains("lay::nanda_wave::"),
            "production daemon source bypasses TypingCpu: {path}"
        );
    }
    assert!(
        typing_cpu.contains("pub fn record_user_correction")
            && typing_cpu.contains("pub fn record_reverted_system_apply")
            && typing_cpu.contains("pub fn record_precognition_tick")
            && typing_cpu.contains("pub fn record_typing_assist_trace"),
        "TypingCpu must own the daemon-facing feedback and trace boundary"
    );
}

#[test]
fn input_gate_has_no_shadow_pipeline_or_duplicate_trace_types() {
    let gate = read("src/input_gate.rs");
    assert!(
        gate.contains("fn decide_space_autocorrect_observed")
            && gate.contains("correction_request_from_input_gate(req)")
            && gate.contains("resolve_text_correction_observed(correction_request)")
            && gate.contains("type InputGateCandidateScoreTrace = CorrectionCandidateScoreTrace")
            && gate.contains("type InputGateScoreboard = CorrectionScoreboard"),
        "InputGate must reuse the canonical observed correction resolution and traces"
    );
    assert!(
        !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/correction_pipeline.rs")
            .exists(),
        "shadow correction pipeline must not return"
    );
}

#[test]
fn l3_l4_live_authority_uses_compact_relation_phase_and_hidden_state() {
    let context_phase = read("src/nanda_wave/context_phase/mod.rs");
    let context_format = read("src/nanda_wave/context_phase/format.rs");
    let hidden_state = read("src/nanda_wave/l4_hidden_state.rs");
    let decision = read("src/typing_transition/decision.rs");
    let candidate_gate = read("src/nanda_wave/candidate_gate.rs");
    let module = read("src/nanda_wave/mod.rs");

    assert!(
        context_phase.contains("candidate_relation_vector")
            && context_phase.contains("ContextPhaseDisposition::Support")
            && context_phase.contains("LAYL3P01")
            && context_format.contains("write_vector"),
        "L3 must compile and read compact candidate-bound relation centers"
    );
    assert!(
        hidden_state.contains("estimate_hidden_typing_state")
            && hidden_state.contains("SemanticClass")
            && hidden_state.contains("ambiguity_authoritative")
            && decision.contains("settle_l4_hidden_state"),
        "L4 semantic quotient must be part of TransitionDecisionCore"
    );
    assert!(
        !decision.contains("derive_l4_scene_state")
            && !candidate_gate.contains("derive_l4_scene_state")
            && !candidate_gate.contains("l4_signed_outcome(")
            && !module.contains("mod l4_signed_outcome"),
        "manual scene/outcome rules must not regain live ranking authority"
    );
}

#[test]
fn l2_and_ime_hot_paths_keep_runtime_owners_separate_from_proof_code() {
    let l2 = read("src/nanda_wave/l2.rs");
    let ime_readout = read("src/nanda_wave/l2/ime_readout.rs");
    let layout = read("src/nanda_wave/l2/layout_adapter.rs");
    let tail_scan = read("src/nanda_wave/l2/tail_scan_adapter.rs");
    let preedit = read("src/bin/lay_ibus_engine/preedit.rs");

    assert!(
        l2.contains("mod ime_readout;")
            && l2.contains("mod layout_adapter;")
            && l2.contains("mod tail_scan_adapter;")
            && l2.contains("mod taught_adapter;")
            && l2.contains("pub struct L2ImeWordCandidate")
            && l2.contains("pub fn ime_l2_word_candidates(")
            && ime_readout.contains("pub(super) fn ime_l2_word_candidates_impl(")
            && l2.contains("mod tests;"),
        "L2 must keep its stable facade while runtime producers and proof code have private owners"
    );
    assert!(
        !layout.contains("TransitionDecisionCore")
            && !tail_scan.contains("TransitionDecisionCore")
            && layout.contains("pub(super) fn layout_candidate")
            && tail_scan.contains("pub(super) fn boundary_scan_candidates"),
        "L2 adapters may produce candidates but must not own final transition authority"
    );
    assert!(
        preedit.contains("#[path = \"preedit/tests.rs\"]")
            && preedit.contains("mod tests;")
            && !preedit.contains("mod tests {"),
        "IME proof fixtures must not remain embedded in the hot preedit module"
    );
}

#[test]
fn candidate_admission_only_marks_eligibility_and_core_selects_transition() {
    let correction = read("src/correction_core.rs");
    let candidates = read("src/correction_core/candidate_sources.rs");
    let gate = read("src/typing_transition/proposal_admission.rs");
    let decision = read("src/typing_transition/decision.rs");
    let apply_policy = read("src/typing_transition/decision/apply_policy.rs");

    assert!(
        correction.contains("CandidateGateAction::Eligible")
            && !std::path::Path::new("src/correction_core/gate.rs").exists()
            && correction.contains("pub use crate::typing_transition::proposal_admission")
            && gate.contains("fn candidate_admission(")
            && gate.contains("gate_candidate_with_origin(")
            && candidates.contains("TransitionDecisionCore::admit_candidate_proposal(")
            && !candidates.contains("gate_candidate_with_origin("),
        "Typing Transition CPU must own proposal admission and candidate producers must enter through its facade"
    );
    assert!(
        decision.contains("pub(crate) fn admit_candidate_proposal(")
            && decision.contains("producer_allows_authority_evaluation(")
            && apply_policy.contains("action == CandidateGateAction::Eligible")
            && apply_policy.contains("l3_pairwise_certified: bool")
            && apply_policy.contains("action == CandidateGateAction::SuggestOnly")
            && apply_policy.contains("l3_pairwise_certified || l4_signal.exact_positive()")
            && decision.contains("candidate_has_apply_authority")
            && !decision.contains("fn authorize_gate"),
        "only TransitionDecisionCore may admit proposals and choose an eligible, L3-pair-certified, or exact-L4-attested candidate"
    );
    assert!(
        !correction.contains("CandidateGateAction::Apply")
            && !gate.contains("CandidateGateAction::Apply"),
        "candidate producer types must not expose an Apply state"
    );
}

#[test]
fn input_gate_logs_candidate_admission_separately_from_final_outcome() {
    let input_gate = read("src/input_gate.rs");
    let action_log = read("src/action_log.rs");
    let debug_actions = read("src/bin/lay_debug_actions.rs");
    let candidate_quality = read("src/bin/lay_nanda_wave_eval/candidate_quality.rs");

    assert!(
        input_gate.contains("pub(crate) enum InputGateOutcome")
            && input_gate.contains("selected_candidate_gate_action")
            && input_gate.contains("outcome: InputGateOutcome"),
        "InputGate must expose candidate admission and the final decision as different typed facts"
    );
    assert!(
        action_log.contains("decision_outcome: Option<String>")
            && action_log.contains("selected_candidate_gate_action: Option<String>")
            && debug_actions.contains("decision_outcome")
            && candidate_quality.contains("decision_outcome"),
        "logs and reports must consume the final outcome without reinterpreting candidate eligibility"
    );
}

#[test]
fn live_canonical_l2_owns_layout_as_a_typed_contour_without_a_side_donor() {
    let l2 = read("src/nanda_wave/l2.rs");
    let bridge = read("src/nanda_wave/l2_field/bridge.rs");
    let layout = read("src/nanda_wave/l2/layout_adapter.rs");

    assert!(
        !l2.contains("pub(crate) fn hot_short_layout_candidates")
            && !bridge.contains("hot_short_layout_candidates")
            && bridge.contains("CanonicalInputTokenKind::PhysicalLayout")
            && bridge.contains("CanonicalContourRelation::ExactLayout")
            && bridge.contains("live_l11_contour_results("),
        "the canonical field must represent layout evidence as a typed contour instead of calling a side donor"
    );
    assert!(
        !layout.contains("TransitionDecisionCore")
            && !bridge.contains("CorrectionDecision {")
            && !bridge.contains("CandidateGateAction::Apply"),
        "the live canonical layout bridge may propose candidates but must not choose or apply edits"
    );
}

#[test]
fn cold_learning_uses_exact_surface_attestation_not_form_settlement() {
    let hot_field = read("src/hot_field.rs");
    let learning_loop = read("src/bin/lay_nanda_wave_eval/learning_loop.rs");

    assert!(
        hot_field.contains("pub fn learning_surface_is_attested")
            && hot_field.contains("l2_surface_foundation_contains(&lower)")
            && learning_loop.contains("HotFieldSnapshot::current().learning_surface_is_attested"),
        "cold learning must use the explicit exact-surface bridge"
    );
    assert!(
        !learning_loop.contains("is_known_russian_word_or_form(token)"),
        "generated morphology may settle a runtime candidate but cannot manufacture observed training evidence"
    );
}

#[test]
fn l4_memory_owns_complete_transition_targets_and_cold_initialization() {
    let memory = read("src/typing_memory.rs");
    let relation = read("src/transition_relation.rs");
    let usage = read("src/nanda_wave/usage_prior.rs");
    let decision_signals = read("src/typing_transition/decision_signals.rs");

    assert!(
        memory.contains("pub(crate) fn transition_target_text")
            && memory.contains("pub(crate) fn transition_context_words")
            && relation.contains("pub(crate) fn transition_target_id"),
        "L4 transition identity must cover the complete changed target, not only its last token"
    );
    assert!(
        decision_signals.contains("candidate_text: &candidate.replacement"),
        "TransitionDecisionCore must address signed memory with the typed target region"
    );
    assert!(
        usage.contains("ensure_usage_cache_initialized(&mut cache, load_usage_counts)")
            && usage.contains("first_hot_readout_initializes_persisted_usage_memory_once"),
        "the first hot readout must load persisted memory without a foreign warmup route"
    );
}

#[test]
fn usage_learning_keeps_disk_io_out_of_the_hot_path() {
    let usage = read("src/nanda_wave/usage_prior.rs");
    let append_start = usage
        .find("fn append_usage_event(event: UsageEvent)")
        .expect("usage event append owner");
    let append_end = usage[append_start..]
        .find("fn refresh_usage_cache_after_write")
        .map(|offset| append_start + offset)
        .expect("usage cache refresh owner");
    let append_body = &usage[append_start..append_end];

    assert!(
        append_body.contains("refresh_usage_cache_after_write(&event);")
            && append_body.contains("enqueue_usage_persist(path, line);")
            && !append_body.contains("append_private_text"),
        "the typing hot path must update memory and enqueue persistence without disk IO"
    );
    assert!(
        usage.contains(".name(\"lay-usage-persist\".to_string())")
            && usage.contains("fn flush_usage_persist(")
            && usage
                .contains("mpsc::sync_channel::<UsagePersistLine>(USAGE_PERSIST_CHANNEL_CAPACITY)")
            && usage.contains("sender.try_send")
            && usage.contains("append_private_text(&path, &text)"),
        "one named bounded persistence worker must own batched disk writes"
    );
}

#[test]
fn usage_event_semantics_are_shared_without_cold_to_hot_dependency() {
    let usage = read("src/nanda_wave/usage_prior.rs");
    let hot = read("src/nanda_wave/usage_prior/hot.rs");
    let projection = read("src/nanda_wave/usage_prior/projection.rs");

    assert!(
        usage.contains("mod hot;")
            && usage.contains("mod projection;")
            && usage.contains("use projection::{UsageEventProjection, TRANSITION_ANY};"),
        "cold persistence and hot runtime must share a neutral event projection owner"
    );
    assert!(
        hot.contains("use super::projection::{UsageEventProjection, TRANSITION_ANY};")
            && !projection.contains("super::hot")
            && !projection.contains("UsageHotState"),
        "event semantics must not depend on the numeric hot representation"
    );
}

#[test]
fn logs_and_usage_share_hot_runtime_flags_and_bounded_periodic_writers() {
    let flags = read("src/config/runtime_flags.rs");
    let config = read("src/config.rs");
    let debug_log = read("src/debug_log.rs");
    let usage = read("src/nanda_wave/usage_prior.rs");

    assert!(
        flags.contains("AtomicU8")
            && flags.contains("runtime_debug_action_log")
            && flags.contains("runtime_usage_learning_enabled")
            && config.contains("publish_runtime_config"),
        "hot feature flags must have one atomic runtime owner"
    );
    for (name, source) in [("debug", debug_log), ("usage", usage)] {
        assert!(
            source.contains("sync_channel")
                && source.contains("try_send")
                && source.contains("next_flush"),
            "{name} writer must be bounded, nonblocking, and flush on a fixed deadline"
        );
    }
}

#[test]
fn decision_thresholds_and_tie_break_have_one_owner() {
    let decision = read("src/typing_transition/decision.rs");
    let admission = read("src/typing_transition/decision/admission.rs");
    let calibration = read("src/typing_transition/decision/calibration.rs");

    assert!(
        decision.contains("compare_candidate_decision_order")
            && decision.contains(".total_cmp(&right_eval.signals.rank_score)")
            && decision.contains("changed_tokens"),
        "equal-score candidate selection must have a deterministic final order"
    );
    assert!(
        admission.contains("CURRENT.structural_preservation_gain_milli")
            && admission.contains("CURRENT.l2_competitor_gap_milli")
            && admission.contains("CURRENT.phase_competitor_gap_milli")
            && calibration.contains("pub(super) const CURRENT: AdmissionCalibration"),
        "learned admission thresholds must come from one calibration profile"
    );
}
