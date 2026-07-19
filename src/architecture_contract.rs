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
}
