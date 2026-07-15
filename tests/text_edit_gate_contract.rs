use lay::text_edit::{
    authorize_backend_edit, plan_committed_tail_full_token_replacement, plan_manual_edit,
    EditActionKind, TextEditBackend,
};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn public_text_edit_gate_keeps_manual_replacement_behind_backend_capability() {
    let plan = plan_committed_tail_full_token_replacement("ошипка ", "ошибка ").expect("plan");
    let action = plan_manual_edit("contract-test", 1000, "ошипка ", "ошибка ", plan, 1);

    assert_eq!(action.kind(), EditActionKind::ReplaceLastToken);
    assert!(action.allow_apply());
    let authorization = authorize_backend_edit(TextEditBackend::Daemon, action);
    assert!(authorization.allow_execute);
    assert!(authorization.into_authorized().is_some());
}

#[test]
fn transition_proof_constructor_and_fields_are_not_public_capabilities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mutation = fs::read_to_string(root.join("src/text_edit/mutation.rs")).expect("mutation");
    let gate = fs::read_to_string(root.join("src/text_edit/gate.rs")).expect("gate");
    let facade = fs::read_to_string(root.join("src/text_edit.rs")).expect("facade");

    let action = fs::read_to_string(root.join("src/text_edit/action.rs")).expect("action");
    assert!(mutation.contains("pub(crate) fn proven("));
    assert!(mutation.contains("operator: Option<TransitionOperator>"));
    assert!(!mutation.contains("pub(crate) operator: Option<TransitionOperator>"));
    assert!(gate.contains("struct VerifiedTransitionReceipt"));
    assert!(gate.contains("fn issue(action: &EditAction)"));
    assert!(gate.contains("pub(crate) fn plan_decision_transition_edit("));
    assert!(action.contains("verification: Option<VerifiedTransitionReceipt>"));
    assert!(action.contains("verified_transition_receipt_missing"));
    assert!(!facade.contains("pub use gate::{authorize_replacement"));
}

#[test]
fn runtime_text_edits_use_narrow_typed_plans_not_generic_proof_construction() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rust_files(&root.join("src"), &mut files);

    let mut violations = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.starts_with("src/bin/") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read rust source");
        for (line_idx, line) in text.lines().enumerate() {
            if line.contains("plan_decision_transition_edit(")
                || line.contains("TransitionAudit::proven(")
                || line.contains("VerifiedTransitionReceipt::issue(")
            {
                violations.push(format!("{}:{}: {}", rel, line_idx + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime adapters must use narrow typed plan APIs:\n{}",
        violations.join("\n")
    );
}

#[test]
fn only_text_edit_gate_can_issue_or_attach_execution_receipts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_rust_files(&root.join("src"), &mut files);
    let mut violations = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&path).expect("read rust source");
        if rel != "src/text_edit/gate.rs"
            && (text.contains("VerifiedTransitionReceipt::issue(")
                || text.contains(".attach_verification("))
        {
            violations.push(rel);
        }
    }
    assert!(
        violations.is_empty(),
        "only text_edit/gate.rs may mint execution authority: {}",
        violations.join(", ")
    );
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read source dir") {
        let entry = entry.expect("source dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
