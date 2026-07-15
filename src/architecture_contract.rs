//! Runtime architecture contract for the current lay input pipeline.
//!
//! The status is compiled from the deterministic Graphify/AST receipt produced
//! by `scripts/architecture_graph_gate.py`; runtime code does not infer PASS by
//! searching its own source text.

use serde::Deserialize;
use std::sync::OnceLock;

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
    pub proof: &'static str,
    pub debt: &'static str,
}

const LINES: [ArchitectureLine; 8] = [
    ArchitectureLine {
        id: "decision-authority",
        layer: "Transition Decision Core",
        owner: "typing_transition::decision + text_edit::transition",
        proof: "candidate apply and visible-tail edits pass through transition authority",
        debt: "keep old correction rules as candidate producers only",
    },
    ArchitectureLine {
        id: "ime-backend-only",
        layer: "IME",
        owner: "text_edit::executor::TextEditBackend::Ime",
        proof: "IME may execute verified actions but cannot decide apply truth",
        debt: "daemon boundary worker owns correction truth; IME only executes authorized edits",
    },
    ArchitectureLine {
        id: "edit-plan-verifier",
        layer: "Text Edit Gate",
        owner: "text_edit::safety + text_edit::gate",
        proof: "multiword, boundary, middle-tail, and stale-tail edits require proof",
        debt: "all future text mutation paths must carry an EditAction",
    },
    ArchitectureLine {
        id: "typed-transition-capability",
        layer: "Transition Proof Capability",
        owner: "text_edit::mutation + text_edit::gate",
        proof: "generic proof construction is crate-private; adapters receive narrow typed edit plans",
        debt: "new output routes must not construct TransitionAudit or generic transition plans",
    },
    ArchitectureLine {
        id: "hot-field-memory",
        layer: "Hot Runtime Memory",
        owner: "hot_field + l2_candidate_phase + usage_prior",
        proof: "LAYPC004 stores quantized centers, anti-centers and promotion bits without words",
        debt: "keep exact text in cold training/debug evidence only",
    },
    ArchitectureLine {
        id: "l2-candidate-field",
        layer: "L2",
        owner: "nanda_wave::l2 + l2_candidate_phase",
        proof: "L2 proposes candidates and emits support/repel/unknown; it cannot execute",
        debt: "raise candidate coverage without bypassing per-operator promotion",
    },
    ArchitectureLine {
        id: "l3-l4-learning",
        layer: "L3/L4",
        owner: "usage_prior + l4_signed_memory + typing_memory",
        proof: "state-specific accepted/rejected usage overrides broad popularity and preserves anti-wave",
        debt: "expand organic surface coverage while latest-state feedback remains authoritative",
    },
    ArchitectureLine {
        id: "fast-verifiable",
        layer: "Verification",
        owner: "architecture report + focused tests + latency probes",
        proof: "architecture checks are cheap and do not warm heavy candidate memory",
        debt: "keep final checks focused on modified routes",
    },
];

const TREE: [&str; 16] = [
    "LAY TYPING TRANSITION CPU",
    "|",
    "+-- Input snapshots: source, visible tail, focus and epoch; surrounding cursor/selection when available",
    "+-- L1 Relation Encoder: surface delta, changed region, proof and verifier atoms",
    "+-- L2 Candidate Lattice: candidate producers without apply authority",
    "+-- L2 Phase Memory: promoted centers / anti-centers / learned margin",
    "+-- L3 Context Constraint: phrase admissibility, never text mutation",
    "+-- L4 Surface Frontier: exact-state signed accepted/rejected experience",
    "+-- Transition Decision Core: Apply / SuggestOnly / Keep / ABSTAIN / Veto",
    "+-- Transition Verifier: revision, boundary and left context; backend lifecycle is dispatched/observed/indeterminate",
    "+-- AuthorizedEdit: sealed sole mutation capability",
    "+-- Executor Backends",
    "    |",
    "    +-- daemon: execute verified edits",
    "    +-- IME: display and execute verified IME accepts",
    "    +-- tray: status/config only",
];

const RECEIPT_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/src/generated/architecture_graph_receipt.json"
));

#[derive(Debug, Deserialize)]
struct ArchitectureReceipt {
    schema: String,
    verdict: String,
    checks: Vec<ReceiptCheck>,
}

#[derive(Debug, Deserialize)]
struct ReceiptCheck {
    id: String,
    status: String,
}

fn receipt() -> &'static ArchitectureReceipt {
    static RECEIPT: OnceLock<ArchitectureReceipt> = OnceLock::new();
    RECEIPT.get_or_init(|| {
        serde_json::from_str(RECEIPT_JSON).expect("valid generated architecture graph receipt")
    })
}

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

/// Route evidence compiled from Graphify AST nodes and dependency edges.
///
/// The generated receipt replaces the former source-substring checks. The
/// release architecture gate verifies source/graph freshness before this
/// embedded report is accepted.
pub fn observed_contract_status(id: &str) -> ContractStatus {
    receipt()
        .checks
        .iter()
        .find(|item| item.id == id)
        .filter(|item| item.status == "PASS")
        .map_or(ContractStatus::Watch, |_| ContractStatus::Pass)
}

pub fn architecture_tree() -> &'static [&'static str] {
    &TREE
}

pub fn debt_queue() -> &'static [&'static str] {
    &DEBT
}

pub fn all_contract_lines_pass() -> bool {
    receipt().schema == "lay.architecture-graph-receipt.v1"
        && receipt().verdict == "PASS"
        && LINES
            .iter()
            .all(|line| matches!(observed_contract_status(line.id), ContractStatus::Pass))
}

#[cfg(test)]
mod tests {
    use super::{
        all_contract_lines_pass, architecture_lines, observed_contract_status, ContractStatus,
    };

    #[test]
    fn architecture_contract_has_eight_pass_lines() {
        let lines = architecture_lines();
        assert_eq!(lines.len(), 8);
        assert!(all_contract_lines_pass());
        assert!(lines.iter().any(|line| line.id == "ime-backend-only"));
        assert!(lines.iter().any(|line| line.id == "edit-plan-verifier"));
        assert!(lines
            .iter()
            .any(|line| line.id == "typed-transition-capability"));
        assert!(lines.iter().any(|line| line.id == "l3-l4-learning"));
        assert_eq!(
            observed_contract_status("edit-plan-verifier"),
            ContractStatus::Pass
        );
    }
}
