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
        debt: "do not add correction scoring inside lay_ibus_engine",
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
        owner: "hot_field + nanda_wave center memories",
        status: ContractStatus::Pass,
        proof: "hot processes use compact field/center readout as authority",
        debt: "dictionary strings remain training/fallback material, not hot authority",
    },
    ArchitectureLine {
        id: "l2-candidate-field",
        layer: "L2",
        owner: "nanda_wave::l2 + surface_motif_memory + center memories",
        status: ContractStatus::Pass,
        proof: "L2 emits word candidates from layout, typo, motif, center, and usage signals",
        debt: "raise coverage without restoring direct dictionary authority",
    },
    ArchitectureLine {
        id: "l3-l4-learning",
        layer: "L3/L4",
        owner: "usage_prior + l4_signed_memory + typing_memory",
        status: ContractStatus::Pass,
        proof: "accepted/rejected usage provides signed context and transition memory",
        debt: "expand clean corpus and user-local feedback while keeping rejected memory active",
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

const TREE: [&str; 13] = [
    "LAY TYPING CPU",
    "|",
    "+-- L1 Surface Encoder: chars, layout, boundaries, speed, route",
    "+-- L2 Candidate Action Factory: layout, typo, transposition, boundary, completion",
    "+-- L3 Context State Encoder: phrase scene and word admissibility",
    "+-- L4 Transition Memory: accepted/rejected signed experience",
    "+-- Transition Decision Core: Apply / SuggestOnly / Keep / Veto",
    "+-- Text Edit Gate: verified edit plan, no unsafe multiword drift",
    "+-- Executor Backends",
    "    |",
    "    +-- daemon: execute verified edits",
    "    +-- IME: display and execute verified IME accepts",
    "    +-- tray: status/config only",
];

const DEBT: [&str; 7] = [
    "P0: keep every text mutation behind EditAction and TransitionAudit",
    "P1: remove any remaining direct apply authority from legacy correction producers",
    "P2: raise L2 field coverage without hot dictionary authority",
    "P3: make L3/L4 signed memory explain learned boosts and vetoes",
    "P4: keep IME display aggressive but backend-only",
    "P5: shadow-eval old vs transition-core ranking on dirty logs",
    "P6: keep p99 candidate readout in microseconds and investigate max spikes",
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
                &["L2CandidateLattice::new", "lattice.into_resolution()"],
            );
            let lattice_uses_core = source_contains_all(
                include_str!("typing_transition/candidate.rs"),
                &["TransitionDecisionCore::select_apply_candidate"],
            );
            if matches!(facade_uses_lattice, ContractStatus::Pass)
                && matches!(lattice_uses_core, ContractStatus::Pass)
            {
                ContractStatus::Pass
            } else {
                ContractStatus::Watch
            }
        }
        "ime-backend-only" => mutation_routes_hold_authorized_edit(&[
            include_str!("bin/lay_ibus_engine/composition_commit.rs"),
            include_str!("bin/lay_daemon/typing_assist_runtime/output/ime.rs"),
        ]),
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
            include_str!("hot_field.rs"),
            &["HotFieldSnapshot", "FieldSnapshotOnly"],
        ),
        "l2-candidate-field" => source_contains_all(
            include_str!("typing_transition/candidate.rs"),
            &[
                "L2CandidateLattice",
                "TransitionDecisionCore::select_apply_candidate",
            ],
        ),
        "l3-l4-learning" => source_contains_all(
            include_str!("typing_transition/mod.rs"),
            &["L4StateEstimator", "TransitionMemory::allows_apply"],
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
