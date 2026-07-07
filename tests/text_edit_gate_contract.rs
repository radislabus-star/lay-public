use lay::text_edit::{
    authorize_replacement, authorize_replacement_with_transition,
    plan_committed_tail_full_token_replacement, plan_text_replacement, EditActionKind,
    TransitionAudit,
};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn public_text_edit_gate_keeps_destructive_replacement_behind_safety() {
    let plan = plan_committed_tail_full_token_replacement("ошипка ", "ошибка ").expect("plan");
    let action = authorize_replacement(
        "contract-test",
        720,
        "ошипка ",
        "ошибка ",
        plan,
        Some("missing-letter"),
        Some("missing-letter"),
    );

    assert_eq!(action.kind, EditActionKind::ReplaceLastToken);
    assert!(action.allow_apply());
}

#[test]
fn public_text_mutation_gate_blocks_unverified_left_context_transition() {
    let plan = plan_text_replacement("одно два ", "однотри ").expect("plan");
    let action = authorize_replacement_with_transition(
        "contract-test",
        720,
        "одно два ",
        "однотри ",
        plan,
        Some("nanda"),
        Some("glued-words"),
        TransitionAudit::proven(
            "boundary_transition",
            "left_context_changed_without_boundary_proof",
            false,
            true,
            2,
        ),
    );

    assert_eq!(action.kind, EditActionKind::BlockUnsafe);
    assert!(!action.allow_apply());
    assert_eq!(action.safety_reason(), "edit_transition_not_verified");
    assert_eq!(action.transition.changed_tokens, Some(2));
}

#[test]
fn runtime_text_edits_use_transition_gate_not_plain_authorize_replacement() {
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
        if rel == "src/text_edit/gate.rs" {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read rust source");
        for (line_idx, line) in text.lines().enumerate() {
            if line.contains("authorize_replacement(") {
                violations.push(format!("{}:{}: {}", rel, line_idx + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "runtime text mutation must use authorize_replacement_with_transition:\n{}",
        violations.join("\n")
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
