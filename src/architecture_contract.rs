//! Generated architecture receipt gate.
//!
//! Human architecture belongs in `docs/nanda-wave-architecture.md`. This
//! module only verifies the deterministic Graphify receipt compiled into the
//! binary, so prose cannot become runtime authority.

use serde::Deserialize;
use std::sync::OnceLock;

const REQUIRED_CHECKS: [&str; 11] = [
    "decision-authority",
    "ime-backend-only",
    "edit-plan-verifier",
    "typed-transition-capability",
    "snapshot-lease",
    "observed-outcome-feedback",
    "hot-field-memory",
    "l1-surface-field",
    "l2-candidate-field",
    "l3-l4-learning",
    "fast-verifiable",
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

pub fn all_contract_lines_pass() -> bool {
    let receipt = receipt();
    receipt.schema == "lay.architecture-graph-receipt.v1"
        && receipt.verdict == "PASS"
        && REQUIRED_CHECKS.iter().all(|required| {
            receipt
                .checks
                .iter()
                .any(|check| check.id == *required && check.status == "PASS")
        })
}

#[cfg(test)]
mod tests {
    use super::{all_contract_lines_pass, receipt, REQUIRED_CHECKS};

    #[test]
    fn generated_receipt_proves_every_required_architecture_check() {
        assert_eq!(REQUIRED_CHECKS.len(), 11);
        assert_eq!(receipt().checks.len(), REQUIRED_CHECKS.len());
        assert!(all_contract_lines_pass());
    }

    #[test]
    fn l4_live_identity_is_typed_before_the_storage_adapter() {
        let memory = include_str!("typing_memory.rs");
        let event_body = memory
            .split_once("pub(crate) struct TypingMemoryEvent {")
            .and_then(|(_, tail)| tail.split_once("\n}"))
            .map(|(body, _)| body)
            .expect("TypingMemoryEvent body");
        let typing_cpu = include_str!("typing_cpu/runtime.rs");
        let ime = include_str!("bin/lay_ibus_engine/tail_memory.rs");

        assert!(event_body.contains("TypingMemoryEvidenceSource"));
        assert!(event_body.contains("TypingMemoryOperation"));
        assert!(!event_body.contains("source: String"));
        assert!(!event_body.contains("operation: String"));
        assert!(typing_cpu.contains("transition: ObservedSystemTransition"));
        assert!(ime.contains("ObservedSystemTransition::LayoutProjection"));
        assert!(ime.contains("ObservedSystemTransition::Correction"));
    }

    #[test]
    fn l4_causal_chain_keeps_one_typed_authority_path() {
        let snapshot = include_str!("text_edit/visible_tail.rs");
        let decision = include_str!("typing_transition/decision/receipt.rs");
        let verifier = include_str!("text_edit/gate.rs");
        let executor = include_str!("text_edit/executor.rs");
        let postcondition = include_str!("bin/lay_ibus_engine/engine/types.rs");
        let observer = include_str!("bin/lay_ibus_engine/tail_memory.rs");

        assert!(snapshot.contains("pub struct VisibleTailSnapshot"));
        assert!(decision.contains("struct DecisionTransitionReceipt"));
        assert!(verifier.contains("struct VerifiedTransitionReceipt"));
        assert!(executor.contains("pub struct AuthorizedEdit"));
        assert!(executor.contains("enum BackendDispatchReceipt"));
        assert!(postcondition.contains("struct PendingVisiblePostcondition"));
        assert!(postcondition.contains("snapshot: SnapshotIdentity"));
        assert!(observer.contains("observe_visible_postcondition"));
        assert!(observer.contains("record_observed_system_outcome"));
        assert!(observer.contains("ObservedSystemTransition"));
    }
}
