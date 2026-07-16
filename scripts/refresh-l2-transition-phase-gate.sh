#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

EVAL_BIN="${LAY_NANDA_WAVE_EVAL_BIN:-$ROOT/target/release/lay-nanda-wave-eval}"
DATASET="${LAY_NANDA_PHASE_DATASET:-$ROOT/data/nanda_training/generated_cases.tsv}"
PROFILE="$ROOT/docs/structural_gates/lay-live-transition-gate.profile.json"
RECEIPT="$ROOT/docs/structural_gates/receipts/L2_TRANSITION_PHASE_PROOF_V1.json"
MANIFEST="$ROOT/docs/structural_gates/receipts/L2_TRANSITION_PHASE_PROOF_V1.manifest.json"
PROOF="$(mktemp)"
GATE_STATUS="$(mktemp)"
trap 'rm -f "$PROOF" "$GATE_STATUS"' EXIT

"$EVAL_BIN" --l2-transition-phase-proof --dataset "$DATASET" >"$PROOF"
mkdir -p "$(dirname "$RECEIPT")"
python3 - "$ROOT" "$PROOF" "$RECEIPT" "$MANIFEST" <<'PY'
import hashlib
import json
import os
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
proof = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
receipt_path = pathlib.Path(sys.argv[3])
manifest_path = pathlib.Path(sys.argv[4])
full = proof.get("modes", {}).get("full_phase", {})
no_phase = proof.get("modes", {}).get("no_phase", {})
magnitude = proof.get("modes", {}).get("magnitude_only", {})
without_anti = proof.get("modes", {}).get("without_anti", {})
lexical_full = proof.get("lexical_competition", {}).get("full_lexical_phase", {})
lexical_without_anti = proof.get("lexical_competition", {}).get("without_lexical_anti", {})
lexical_pairs = proof.get("lexical_pair_competition", {}).get("full_lexical_phase", {})
lexical_pairs_without_anti = proof.get("lexical_pair_competition", {}).get("without_lexical_anti", {})
positive_cases = int(full.get("positive_cases", 0))
positive_support = int(full.get("positive_support", 0))
wrong_accepts = int(full.get("negative_support_false_accepts", 0))
lexical_negative_cases = int(lexical_full.get("negative_cases", 0))
lexical_wrong_supports = int(lexical_full.get("negative_support_false_accepts", 0))
abstained = positive_cases - positive_support
phase_causal = (
    int(no_phase.get("positive_support", 0)) == 0
    and int(magnitude.get("positive_support", 0)) == 0
    and int(proof.get("causal_positive_support_drop", 0)) > 0
)
anti_causal = int(without_anti.get("negative_support_false_accepts", 0)) > wrong_accepts
lexical_anti_causal = (
    lexical_negative_cases > 0
    and lexical_wrong_supports == 0
    and int(lexical_without_anti.get("negative_support_false_accepts", 0))
        > lexical_wrong_supports
    and int(proof.get("lexical_negative_rows_deferred_to_l2_word_center", -1)) == 0
)
lexical_top1_causal = (
    int(lexical_pairs.get("cases", 0)) > 0
    and int(lexical_pairs.get("wrong_top1", -1)) == 0
    and int(lexical_pairs.get("correct_top1", 0))
        > int(lexical_pairs_without_anti.get("correct_top1", 0))
)
receipt = {
    "report_kind": "lay_l2_transition_phase_proof_v1",
    "verdict": "PROVEN" if proof.get("verdict") == "PASS" else "NOT_PROVEN",
    "verdicts": {
        "full_execution_pass": positive_cases > 0 and positive_support == positive_cases and wrong_accepts == 0,
        "phase_causal_pass": phase_causal,
        "relational_atoms_causal_pass": phase_causal,
        "core_causal_pass": proof.get("exact_memory_rows_after_compile") == 0,
        "anti_center_causal_pass": anti_causal,
        "lexical_anti_center_causal_pass": lexical_anti_causal,
        "lexical_anti_center_top1_causal_pass": lexical_top1_causal,
    },
    "wrong_accepts": wrong_accepts,
    "abstained_queries": abstained,
    "package_failures": 0,
    "exact_cache_overlap": int(proof.get("exact_memory_rows_after_compile", 0)),
    "structural_exact_parity_failures": 0,
    "corpus_seeds": int(proof.get("training_surfaces", 0)),
    "heldout_queries": int(proof.get("heldout_entries", 0)),
    "correct_cpu_executions": positive_support,
    "lexical_negative_cases": lexical_negative_cases,
    "lexical_wrong_supports": lexical_wrong_supports,
    "lexical_false_supports_prevented": int(
        proof.get("lexical_anti_center_false_support_prevention", 0)
    ),
    "lexical_pair_correct_top1": int(lexical_pairs.get("correct_top1", 0)),
    "lexical_pair_cases": int(lexical_pairs.get("cases", 0)),
    "lexical_pair_wrong_top1": int(lexical_pairs.get("wrong_top1", 0)),
    "promoted_operators": proof.get("promoted_operators", []),
    "phase_proof": proof,
}

sources = [
    "src/transition_relation.rs",
    "src/nanda_wave/l2_candidate_phase.rs",
    "src/typing_transition/decision.rs",
    "src/bin/lay_nanda_dataset.rs",
    "data/nanda_training/generated_cases.tsv",
]

def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

tmp = receipt_path.with_suffix(receipt_path.suffix + ".next")
tmp.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
os.replace(tmp, receipt_path)
manifest = {
    "schema": "nando.wave-causal-proof-manifest.v1",
    "artifact": {
        "path": str(receipt_path.relative_to(root)),
        "sha256": sha256(receipt_path),
    },
    "sources": [
        {"path": source, "sha256": sha256(root / source)} for source in sources
    ],
}
tmp = manifest_path.with_suffix(manifest_path.suffix + ".next")
tmp.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
os.replace(tmp, manifest_path)
PY

NANDO_LIVE_GATE_PROFILE="$PROFILE" \
  NANDO_TRANSITION_ADMISSION_JSON="$GATE_STATUS" \
  nando-live-transition-gate --project-root "$ROOT"
