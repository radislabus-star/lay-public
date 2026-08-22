#!/usr/bin/env bash
set -euo pipefail

run_root=${RUN_ROOT:-/home/e/build/lay-ime-target-authority-slice1-20260820-v1}
candidate_root="$run_root/candidate"
target_root="$run_root/target-candidate"
output="$run_root/output/fault-injection-v3"
missing_root="$run_root/fault-missing-adapter-v3"
lossy_root="$run_root/fault-lossy-adapter-v3"

[[ ! -e "$output" && ! -e "$missing_root" && ! -e "$lossy_root" ]]
mkdir -p "$output"
cp -al "$candidate_root" "$missing_root"
cp -al "$candidate_root" "$lossy_root"

sed -i '/^pub(crate) mod target_evidence;$/d' "$missing_root/src/typing_transition/mod.rs"
set +e
(cd "$missing_root" && timeout 300s env CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=20 \
  scripts/cargo-guard.sh check --lib) >"$output/missing-adapter.log" 2>&1
missing_exit=$?
set -e
[[ "$missing_exit" -ne 0 ]]
grep -Eq 'target_evidence|unresolved import|could not find' "$output/missing-adapter.log"

sed -i 's/self\.replacement_target_evidence\.to_common()/super::target_evidence::TargetEvidenceSetV1::complete_empty()/' \
  "$lossy_root/src/typing_transition/live_candidate.rs"
set +e
(cd "$lossy_root" && timeout 300s env CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=20 \
  scripts/cargo-guard.sh test --lib live_candidate_projects_the_exact_replacement_evidence_value -- --nocapture --test-threads=1) \
  >"$output/lossy-adapter.log" 2>&1
lossy_exit=$?
set -e
[[ "$lossy_exit" -ne 0 ]]
grep -Eq 'assertion.*failed|test result: FAILED' "$output/lossy-adapter.log"

(cd "$candidate_root" && timeout 300s env CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=20 \
  scripts/cargo-guard.sh test --lib live_candidate_projects_the_exact_replacement_evidence_value -- --nocapture --test-threads=1) \
  >"$output/candidate-restored.log" 2>&1

jq -n --argjson missing_adapter_exit "$missing_exit" --argjson lossy_adapter_exit "$lossy_exit" \
  --arg missing_sha256 "$(sha256sum "$output/missing-adapter.log" | cut -d' ' -f1)" \
  --arg lossy_sha256 "$(sha256sum "$output/lossy-adapter.log" | cut -d' ' -f1)" \
  --arg restored_sha256 "$(sha256sum "$output/candidate-restored.log" | cut -d' ' -f1)" \
  '{schema:"lay.ime-target-authority-slice1-adapter-fault-injection.v1",verdict:"PASS",missing_adapter:{exit_code:$missing_adapter_exit,compile_rejected:true,log_sha256:$missing_sha256},lossy_adapter:{exit_code:$lossy_adapter_exit,parity_test_rejected:true,log_sha256:$lossy_sha256},candidate_after_faults:{parity_test_passed:true,log_sha256:$restored_sha256},production_runtime_touched:false}' \
  >"$output/summary.json"
jq '.' "$output/summary.json"
