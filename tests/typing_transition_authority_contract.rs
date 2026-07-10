use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(path)).expect("source file")
}

#[test]
fn visible_tail_decision_delegates_to_transition_core() {
    let facade = read("src/text_edit/transition.rs");
    let core = read("src/typing_transition/decision.rs");

    assert!(
        facade.contains("TransitionDecisionCore::decide_visible_text_transition"),
        "text_edit transition must be an adapter into the shared decision core"
    );
    assert!(
        core.contains("fn decide_visible_text_transition")
            && core.contains("TextTransitionDecision::Apply"),
        "TransitionDecisionCore must own visible-tail apply/reject choice"
    );
}

#[test]
fn candidate_lattice_merges_typed_evidence_without_owner_priority() {
    let lattice = read("src/typing_transition/candidate.rs");
    let candidate = read("src/correction_core.rs");

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
}

#[test]
fn transition_core_uses_typed_origin_for_verifier_and_memory() {
    let transition = read("src/typing_transition/mod.rs");
    let decision = read("src/typing_transition/decision.rs");

    assert!(
        transition.contains("origin: CandidateOrigin")
            && transition.contains("Decision and verifier authority use `origin`"),
        "source_id must be diagnostic provenance, while typed origin carries authority"
    );
    assert!(
        decision.contains("candidate.origin") && decision.contains("candidate.origin.memory_key()"),
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
    let readout = read("src/ime_candidate_readout.rs");

    assert!(
        preedit.contains("rank_ime_candidate_suffixes(ImeCandidateReadoutRequest")
            && !preedit.contains("preedit_suffix_bayes_score"),
        "IME adapter must delegate suffix ranking to shared candidate readout"
    );
    assert!(
        readout.contains("pub fn rank_ime_candidate_suffixes")
            && readout.contains("cached_usage_prior_snapshot"),
        "shared readout must own Bayes/usage candidate ranking"
    );
}

#[test]
fn legacy_gate_only_marks_eligibility_and_core_grants_apply() {
    let correction = read("src/correction_core.rs");
    let gate = read("src/correction_core/gate.rs");
    let decision = read("src/typing_transition/decision.rs");

    assert!(
        correction.contains("CandidateGateAction::Eligible")
            && correction.contains("mod gate;")
            && gate.contains("fn legacy_gate_candidate_with_source"),
        "legacy candidate checks must expose eligibility rather than direct execution"
    );
    assert!(
        decision.contains("CandidateGateAction::Eligible | CandidateGateAction::Apply")
            && decision.contains("reason: \"transition_core_authorized\""),
        "only TransitionDecisionCore may promote an eligible candidate to Apply"
    );
}
