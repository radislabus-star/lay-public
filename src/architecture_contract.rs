//! Runtime architecture contract for the current lay input pipeline.
//!
//! This is intentionally small and cheap: it gives agents and smoke checks one
//! place to inspect the seven non-negotiable ownership boundaries without
//! loading candidate memory or touching a live input backend.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractStatus {
    Pass,
    Watch,
}

impl ContractStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Watch => "WATCH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchitectureLine {
    pub id: &'static str,
    pub layer: &'static str,
    pub owner: &'static str,
    pub status: ContractStatus,
    pub proof: &'static str,
    pub debt: &'static str,
}

const LINES: [ArchitectureLine; 7] = [
    ArchitectureLine {
        id: "decision-authority",
        layer: "Transition Decision Core",
        owner: "typing_transition::decision + text_edit::transition",
        status: ContractStatus::Pass,
        proof: "candidate apply and visible-tail edits pass through transition authority",
        debt: "keep old correction rules as candidate producers only",
    },
    ArchitectureLine {
        id: "ime-backend-only",
        layer: "IME",
        owner: "text_edit::executor::TextEditBackend::Ime",
        status: ContractStatus::Pass,
        proof: "IME may execute verified actions but cannot decide apply truth",
        debt: "daemon boundary worker owns correction truth; IME only executes authorized edits",
    },
    ArchitectureLine {
        id: "edit-plan-verifier",
        layer: "Text Edit Gate",
        owner: "text_edit::safety + text_edit::gate",
        status: ContractStatus::Pass,
        proof: "multiword, boundary, middle-tail, and stale-tail edits require proof",
        debt: "all future text mutation paths must carry an EditAction",
    },
    ArchitectureLine {
        id: "hot-field-memory",
        layer: "Hot Runtime Memory",
        owner: "hot_field + l2_candidate_phase + usage_prior",
        status: ContractStatus::Pass,
        proof: "LAYPC004 stores quantized centers, anti-centers and promotion bits without words",
        debt: "keep exact text in cold training/debug evidence only",
    },
    ArchitectureLine {
        id: "l2-candidate-field",
        layer: "L2",
        owner: "nanda_wave::l2 + l2_candidate_phase",
        status: ContractStatus::Pass,
        proof: "L2 proposes candidates and emits support/repel/unknown; it cannot execute",
        debt: "raise candidate coverage without bypassing per-operator promotion",
    },
    ArchitectureLine {
        id: "l3-l4-learning",
        layer: "L3/L4",
        owner: "usage_prior + l4_signed_memory + typing_memory",
        status: ContractStatus::Pass,
        proof: "state-specific accepted/rejected usage overrides broad popularity and preserves anti-wave",
        debt: "expand organic surface coverage while latest-state feedback remains authoritative",
    },
    ArchitectureLine {
        id: "fast-verifiable",
        layer: "Verification",
        owner: "architecture report + focused tests + latency probes",
        status: ContractStatus::Pass,
        proof: "architecture checks are cheap and do not warm heavy candidate memory",
        debt: "keep final checks focused on modified routes",
    },
];

const TREE: [&str; 16] = [
    "LAY TYPING TRANSITION CPU",
    "|",
    "+-- Input snapshots: daemon/IME state, focus, revision, caret, layout",
    "+-- L1 Relation Encoder: surface delta, changed region, proof and verifier atoms",
    "+-- L2 Candidate Lattice: candidate producers without apply authority",
    "+-- L2 Phase Memory: promoted centers / anti-centers / learned margin",
    "+-- L3 Context Constraint: phrase admissibility, never text mutation",
    "+-- L4 Surface Frontier: exact-state signed accepted/rejected experience",
    "+-- Transition Decision Core: Apply / SuggestOnly / Keep / ABSTAIN / Veto",
    "+-- Transition Verifier: revision, boundary, left context and backend postconditions",
    "+-- AuthorizedEdit: sealed sole mutation capability",
    "+-- Executor Backends",
    "    |",
    "    +-- daemon: execute verified edits",
    "    +-- IME: display and execute verified IME accepts",
    "    +-- tray: status/config only",
];

const DEBT: [&str; 7] = [
    "P0: raise L2 candidate coverage; phase admission cannot recover a candidate that was never born",
    "P1: accumulate organic L4 surface evidence without mixing stale accept/reject states",
    "P2: keep CompositeTypo split into typed subforms when new evidence reveals distinct circuits",
    "P3: keep IME display aggressive, first-word capable and backend-only",
    "P4: investigate end-to-end output latency separately from microsecond phase readout",
    "P5: preserve zero unsafe multiword and unverified left-context applies in live logs",
    "P6: retrain and re-gate every package after relation encoder or proof source changes",
];

pub fn architecture_lines() -> &'static [ArchitectureLine] {
    &LINES
}

/// Cheap route evidence compiled from the physical mutation modules.
///
/// This is deliberately not a claim of whole-program proof: graphify carries
/// that broader graph role. It prevents the status command from reporting a
/// static PASS after a known execution route stops acquiring AuthorizedEdit.
pub fn observed_contract_status(id: &str) -> ContractStatus {
    match id {
        "decision-authority" => {
            let facade_uses_lattice = source_contains_all(
                include_str!("correction_core.rs"),
                &[
                    "L2CandidateLattice::with_options",
                    "lattice.into_resolution()",
                ],
            );
            let authority_uses_core = source_contains_all(
                include_str!("typing_transition/candidate_resolution.rs"),
                &["TransitionDecisionCore::select_apply_candidate"],
            );
            if matches!(facade_uses_lattice, ContractStatus::Pass)
                && matches!(authority_uses_core, ContractStatus::Pass)
            {
                ContractStatus::Pass
            } else {
                ContractStatus::Watch
            }
        }
        "ime-backend-only" => {
            let routes_hold_capability = mutation_routes_hold_authorized_edit(&[
                include_str!("bin/lay_ibus_engine/composition_commit.rs"),
                include_str!("bin/lay_ibus_engine/committed_tail.rs"),
                include_str!("bin/lay_daemon/typing_assist_runtime/output/ime.rs"),
            ]);
            let ime_sources = [
                include_str!("bin/lay_ibus_engine/managed.rs"),
                include_str!("bin/lay_ibus_engine/committed_tail.rs"),
            ];
            let ime_event_loop_has_no_boundary_decision = ime_sources.iter().all(|source| {
                !source.contains("decide_active_composition_autocorrect(")
                    && !source.contains("decide_input_gate(")
            });
            let daemon_space_uses_worker = source_contains_all(
                include_str!("bin/lay_daemon/boundary_runtime/space.rs"),
                &["typing_assist_worker", "PendingTypingAssist::waiting"],
            );
            let worker_owns_boundary_decision =
                include_str!("bin/lay_daemon/typing_assist_worker.rs")
                    .contains("prepare_typing_assist_after_space");
            let daemon_owns_boundary_decision =
                matches!(daemon_space_uses_worker, ContractStatus::Pass)
                    && worker_owns_boundary_decision
                    && include_str!("bin/lay_daemon/boundary_runtime/deferred.rs")
                        .contains("apply_prepared_typing_assist_after_space(");
            if matches!(routes_hold_capability, ContractStatus::Pass)
                && ime_event_loop_has_no_boundary_decision
                && daemon_owns_boundary_decision
            {
                ContractStatus::Pass
            } else {
                ContractStatus::Watch
            }
        }
        "edit-plan-verifier" => {
            let executor_has_capability = source_contains_all(
                include_str!("text_edit/executor.rs"),
                &[
                    "pub struct AuthorizedEdit",
                    "authorized: Option<AuthorizedEdit",
                ],
            );
            let routes_hold_capability = mutation_routes_hold_authorized_edit(&[
                include_str!("bin/lay_daemon/typing_assist_runtime/output/minimal.rs"),
                include_str!("bin/lay_daemon/correction_runtime/output/text_replace.rs"),
                include_str!("bin/lay_daemon/correction_runtime/output/replay.rs"),
                include_str!("bin/lay_daemon/correction_runtime/output/native.rs"),
                include_str!("bin/lay_daemon/auto_undo_runtime.rs"),
                include_str!("bin/lay_daemon/enter_autocorrect_runtime.rs"),
            ]);
            let physical_executors_require_capability = matches!(
                source_contains_all(
                    include_str!("bin/lay_daemon/text_output/replacement.rs"),
                    &["authorized: &AuthorizedEdit", "authorized.action()"],
                ),
                ContractStatus::Pass
            ) && matches!(
                source_contains_all(
                    include_str!("bin/lay_daemon/layout_controller.rs"),
                    &[
                        "try_ime_replace_tail(\n    authorized: &AuthorizedEdit",
                        "call_replace_text(\n    authorized: &AuthorizedEdit",
                    ],
                ),
                ContractStatus::Pass
            );
            if matches!(executor_has_capability, ContractStatus::Pass)
                && matches!(routes_hold_capability, ContractStatus::Pass)
                && physical_executors_require_capability
            {
                ContractStatus::Pass
            } else {
                ContractStatus::Watch
            }
        }
        "hot-field-memory" => source_contains_all(
            include_str!("nanda_wave/l2_candidate_phase.rs"),
            &[
                "LAYPC004",
                "operator_promoted",
                "raw_words_stored",
                "proven_phase_operators",
            ],
        ),
        "l2-candidate-field" => {
            let lattice = source_contains_all(
                include_str!("typing_transition/candidate_resolution.rs"),
                &[
                    "resolve_l2_lattice",
                    "TransitionDecisionCore::select_apply_candidate",
                ],
            );
            let phase_authority = source_contains_all(
                include_str!("typing_transition/decision.rs"),
                &["l2_transition_phase_readout", "phase_policy_rejection"],
            );
            if matches!(lattice, ContractStatus::Pass)
                && matches!(phase_authority, ContractStatus::Pass)
            {
                ContractStatus::Pass
            } else {
                ContractStatus::Watch
            }
        }
        "l3-l4-learning" => source_contains_all(
            include_str!("typing_transition/decision_signals.rs"),
            &[
                "transition_state_id",
                "l4_signed_memory_signal",
                "transition_state_specific",
            ],
        ),
        "fast-verifiable" => source_contains_all(
            include_str!("text_edit/executor.rs"),
            &["ExecutorContract::backend_only", "authorize_edit"],
        ),
        _ => ContractStatus::Watch,
    }
}

fn source_contains_all(source: &str, needles: &[&str]) -> ContractStatus {
    if needles.iter().all(|needle| source.contains(needle)) {
        ContractStatus::Pass
    } else {
        ContractStatus::Watch
    }
}

fn mutation_routes_hold_authorized_edit(routes: &[&str]) -> ContractStatus {
    if routes
        .iter()
        .all(|route| route.contains("authorize_backend_edit(") && route.contains(".authorized()"))
    {
        ContractStatus::Pass
    } else {
        ContractStatus::Watch
    }
}

pub fn architecture_tree() -> &'static [&'static str] {
    &TREE
}

pub fn debt_queue() -> &'static [&'static str] {
    &DEBT
}

pub fn all_contract_lines_pass() -> bool {
    LINES
        .iter()
        .all(|line| matches!(observed_contract_status(line.id), ContractStatus::Pass))
}

#[cfg(test)]
mod tests {
    use super::{
        all_contract_lines_pass, architecture_lines, observed_contract_status, ContractStatus,
    };

    #[test]
    fn architecture_contract_has_seven_pass_lines() {
        let lines = architecture_lines();
        assert_eq!(lines.len(), 7);
        assert!(all_contract_lines_pass());
        assert!(lines.iter().any(|line| line.id == "ime-backend-only"));
        assert!(lines.iter().any(|line| line.id == "edit-plan-verifier"));
        assert!(lines.iter().any(|line| line.id == "l3-l4-learning"));
        assert_eq!(
            observed_contract_status("edit-plan-verifier"),
            ContractStatus::Pass
        );
    }
}
