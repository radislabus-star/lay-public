#!/usr/bin/env bash
set -euo pipefail

run_root=${RUN_ROOT:-/home/e/build/lay-ime-target-authority-slice1-20260820-v1}
slice0_root=/home/e/build/lay-ime-target-authority-slice0-20260820
baseline_root="$run_root/baseline"
candidate_root="$run_root/candidate"
target_root="$run_root/target-candidate"
output="$run_root/output/candidate-v4"
plan="$slice0_root/execution-plan.json"
manifest="$slice0_root/source-at-execution.sha256-manifest.json"

die() {
  printf 'SLICE1_FATAL %s\n' "$*" >&2
  exit 1
}

verify_sha() {
  local path=$1 expected=$2 actual
  [[ -f "$path" ]] || die "missing candidate file: $path"
  actual=$(sha256sum "$path" | cut -d' ' -f1)
  [[ "$actual" == "$expected" ]] || die "candidate SHA mismatch: $path"
}

verify_candidate_source() {
  local rel expected actual changed=0 outside=0 missing=0
  verify_sha "$candidate_root/src/typing_transition/target_evidence.rs" c01a7fb8ebfa140e114632fa4d4ed35acfc587422b692866d3e7cc921f1e50ed
  verify_sha "$candidate_root/src/typing_transition/mod.rs" 351c8c35db7475c6b9f4851e89229d1636275a5f692f94556f8dab2a508fe246
  verify_sha "$candidate_root/src/typing_transition/live_candidate.rs" d55a04a0521a76cc78f43898e34285d7e58b8c0452ea88ebed3a042cf4fbcfa6
  verify_sha "$candidate_root/src/nanda_wave/l2.rs" 951deb7afd26ce2f7b9c1b2c1d99b9546887c66cc385f7e68a63c5fc1b002cf6
  verify_sha "$candidate_root/src/nanda_wave/l2_field/productive_v1/live.rs" f302e0177b87c95f2d3be56163680501c20526b51d1dbf95dff72f681fea9596
  verify_sha "$candidate_root/src/nanda_wave/l2_field/productive_v1/composite.rs" 2cd6eb3a843d202080053d739e7a201f64d8866f8e594caa05d6af32b03eef4a
  verify_sha "$candidate_root/src/nanda_wave/l2_field/productive_v1/scene.rs" 60a92a723d8dd96ce5e5a1ba21e2c215fb34cee44c437dffd1ac31e56f22ff4e
  verify_sha "$candidate_root/src/correction_core.rs" 9da88d899c4c41486e72b6b84ab11a852b086d4ea549b6c8ab17dd7a6a650120

  while IFS=$'\t' read -r rel expected; do
    if [[ ! -f "$candidate_root/$rel" ]]; then
      printf 'MISSING\t%s\n' "$rel"
      missing=$((missing + 1))
      continue
    fi
    actual=$(sha256sum "$candidate_root/$rel" | cut -d' ' -f1)
    if [[ "$actual" != "$expected" ]]; then
      changed=$((changed + 1))
      case "$rel" in
        src/typing_transition/mod.rs|src/typing_transition/live_candidate.rs|src/typing_transition/candidate.rs|src/nanda_wave/l2.rs|src/nanda_wave/l2/ime_readout.rs|src/nanda_wave/l2_field/bridge.rs|src/nanda_wave/l2_field/productive_v1/live.rs|src/nanda_wave/l2_field/productive_v1/composite.rs|src/nanda_wave/l2_field/productive_v1/scene.rs|src/correction_core.rs|src/correction_core/candidate_sources.rs)
          printf 'ALLOWLIST_CHANGED\t%s\n' "$rel"
          ;;
        *)
          printf 'OUTSIDE_CHANGED\t%s\n' "$rel"
          outside=$((outside + 1))
          ;;
      esac
    fi
  done < <(jq -r '.source.files[] | select(.type == "regular file" and (.path | startswith("src/"))) | [.path,.sha256] | @tsv' "$manifest")
  printf 'ALLOWLIST_CREATED\tsrc/typing_transition/target_evidence.rs\n'
  printf 'SUMMARY\tsource_changed=%d\toutside=%d\tmissing=%d\n' "$changed" "$outside" "$missing"
  [[ "$changed" -eq 7 && "$outside" -eq 0 && "$missing" -eq 0 ]]
} >"$output/source-allowlist.tsv"

run_build() {
  local id=$1
  shift
  local log="$output/build/$id.log" status
  set +e
  (cd "$candidate_root" && timeout 1800s env \
    CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=20 \
    LAY_L11_SOCKET=/tmp/lay-l11-canonical-layout-metrics-proof-20260816.sock \
    LAY_L11_RECEIPT=/home/e/build/lay-l1-phase8i-integrity-20260815/runtime-model/active.installed.json \
    LAY_L2_PRODUCTIVE_V1_PACKAGE=/home/e/build/lay-productive-v90-active-20260816/out/LAY-L2-PRODUCTIVE-PARADIGM-v90-L11v9-L2v13.p2m \
    "$@") >"$log" 2>&1
  status=$?
  set -e
  jq -n --arg id "$id" --argjson exit_code "$status" \
    --arg sha256 "$(sha256sum "$log" | cut -d' ' -f1)" \
    --argjson size_bytes "$(stat -c %s "$log")" \
    '{id:$id,exit_code:$exit_code,log:{size_bytes:$size_bytes,sha256:$sha256}}' \
    >"$output/build/$id.json"
  [[ "$status" -eq 0 ]] || die "build failed: $id"
}

run_focus() {
  local id=$1 filter=$2 log status
  log="$output/focused/$id.log"
  set +e
  (cd "$candidate_root" && timeout 300s env \
    CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=20 \
    LAY_L11_SOCKET=/tmp/lay-l11-canonical-layout-metrics-proof-20260816.sock \
    LAY_L11_RECEIPT=/home/e/build/lay-l1-phase8i-integrity-20260815/runtime-model/active.installed.json \
    LAY_L2_PRODUCTIVE_V1_PACKAGE=/home/e/build/lay-productive-v90-active-20260816/out/LAY-L2-PRODUCTIVE-PARADIGM-v90-L11v9-L2v13.p2m \
    scripts/cargo-guard.sh test --lib "$filter" -- --nocapture --test-threads=1) >"$log" 2>&1
  status=$?
  set -e
  jq -n --arg id "$id" --arg filter "$filter" --argjson exit_code "$status" \
    --arg sha256 "$(sha256sum "$log" | cut -d' ' -f1)" \
    --argjson size_bytes "$(stat -c %s "$log")" \
    '{id:$id,filter:$filter,exit_code:$exit_code,log:{size_bytes:$size_bytes,sha256:$sha256}}' \
    >"$output/focused/$id.json"
  [[ "$status" -eq 0 ]] || die "focused test failed: $id"
}

run_case() {
  local entry=$1 id selector log receipt status observed_count selector_mentions pass_summaries fail_summaries observed_status
  id=$(jq -r '.id' <<<"$entry")
  selector=$(jq -r '.compiled_selector' <<<"$entry")
  mapfile -t command < <(jq -r '.command[]' <<<"$entry")
  mapfile -t environment < <(jq -r '.environment | to_entries[] | "\(.key)=\(.value)"' <<<"$entry")
  log="$output/baseline49/logs/$id.log"
  receipt="$output/baseline49/receipts/$id.json"
  set +e
  (cd "$candidate_root" && timeout 300s env "${environment[@]}" \
    CARGO_TARGET_DIR="$target_root" CARGO_BUILD_JOBS=20 "${command[@]}") >"$log" 2>&1
  status=$?
  set -e
  selector_mentions=$(grep -Foc "test $selector ..." "$log" || true)
  pass_summaries=$(grep -Ec '^test result: ok\. 1 passed; 0 failed;' "$log" || true)
  fail_summaries=$(grep -Ec '^test result: FAILED\. 0 passed; 1 failed;' "$log" || true)
  observed_count=$((pass_summaries + fail_summaries))
  if [[ "$selector_mentions" -eq 1 && "$observed_count" -eq 1 ]]; then
    if [[ "$pass_summaries" -eq 1 && "$status" -eq 0 ]]; then
      observed_status=PASS
    elif [[ "$fail_summaries" -eq 1 && "$status" -ne 0 ]]; then
      observed_status=FAIL
    else
      observed_status=INVALID_EXECUTION
    fi
  else
    observed_status=INVALID_EXECUTION
  fi
  jq -n --argjson planned "$entry" --arg observed_status "$observed_status" \
    --argjson exit_code "$status" --argjson observed_test_count "$observed_count" \
    --arg sha256 "$(sha256sum "$log" | cut -d' ' -f1)" \
    --argjson size_bytes "$(stat -c %s "$log")" \
    '{planned:$planned,exit_code:$exit_code,observed_test_count:$observed_test_count,observed_status:$observed_status,valid_execution:($observed_test_count==1 and ($observed_status=="PASS" or $observed_status=="FAIL")),log:{size_bytes:$size_bytes,sha256:$sha256}}' \
    >"$receipt"
}

run_baseline49() {
  local active=0 entry
  while IFS= read -r entry; do
    run_case "$entry" &
    active=$((active + 1))
    if [[ "$active" -ge 8 ]]; then
      wait -n
      active=$((active - 1))
    fi
  done < <(jq -c '.series.baseline49[]' "$plan")
  wait
  jq -s '{receipt_count:length,valid_execution_count:([.[]|select(.valid_execution)]|length),pass_count:([.[]|select(.observed_status=="PASS")]|length),fail_count:([.[]|select(.observed_status=="FAIL")]|length),invalid_count:([.[]|select(.valid_execution|not)]|length),receipts:.}' \
    "$output"/baseline49/receipts/*.json >"$output/baseline49/run-summary.json"
  jq -e '.receipt_count==49 and .valid_execution_count==49 and .invalid_count==0' "$output/baseline49/run-summary.json" >/dev/null

  jq -n --slurpfile baseline "$slice0_root/output/run-summary.json" \
    --slurpfile candidate "$output/baseline49/run-summary.json" '
      ($baseline[0].receipts | map(select(.planned.series=="baseline49"))) as $old |
      ($candidate[0].receipts) as $new |
      {baseline_count:($old|length),candidate_count:($new|length),differences:[
        $new[] as $candidate_receipt |
        ($old[] | select(.planned.id==$candidate_receipt.planned.id)) as $baseline_receipt |
        select($baseline_receipt.observed_status != $candidate_receipt.observed_status) |
        {id:$candidate_receipt.planned.id,baseline:$baseline_receipt.observed_status,candidate:$candidate_receipt.observed_status}
      ]}' >"$output/baseline49/status-parity.json"
  jq -e '.baseline_count==49 and .candidate_count==49 and (.differences|length)==0' "$output/baseline49/status-parity.json" >/dev/null
}

find_test_binary() {
  local root=$1 selector=$2 binary
  while IFS= read -r binary; do
    if "$binary" --list 2>/dev/null | grep -Fq "$selector: test"; then
      printf '%s\n' "$binary"
      return 0
    fi
  done < <(find "$root/debug/deps" -maxdepth 1 -type f -name 'lay-*' -perm /111 | sort)
  return 1
}

measure_rss() {
  local selector baseline_binary candidate_binary iteration baseline_median candidate_median baseline_max candidate_max delta
  selector=typing_transition::live_candidate::tests::decision_core_is_the_live_completion_sort_owner
  baseline_binary=$(find_test_binary "$slice0_root/target" "$selector") || die 'baseline lib-test binary not found'
  candidate_binary=$(find_test_binary "$target_root" "$selector") || die 'candidate lib-test binary not found'
  : >"$output/rss-baseline-kib.txt"
  : >"$output/rss-candidate-kib.txt"
  for iteration in $(seq 1 9); do
    /usr/bin/time -f %M -o "$output/rss-baseline-kib.txt" -a \
      "$baseline_binary" "$selector" --exact --test-threads=1 >/dev/null 2>&1
    /usr/bin/time -f %M -o "$output/rss-candidate-kib.txt" -a \
      "$candidate_binary" "$selector" --exact --test-threads=1 >/dev/null 2>&1
  done
  baseline_median=$(sort -n "$output/rss-baseline-kib.txt" | sed -n '5p')
  candidate_median=$(sort -n "$output/rss-candidate-kib.txt" | sed -n '5p')
  baseline_max=$(sort -n "$output/rss-baseline-kib.txt" | tail -1)
  candidate_max=$(sort -n "$output/rss-candidate-kib.txt" | tail -1)
  delta=$((candidate_median - baseline_median))
  jq -n --arg selector "$selector" --argjson runs 9 \
    --argjson baseline_median_kib "$baseline_median" --argjson candidate_median_kib "$candidate_median" \
    --argjson baseline_max_kib "$baseline_max" --argjson candidate_max_kib "$candidate_max" \
    --argjson median_delta_kib "$delta" --argjson limit_kib 5120 \
    '{selector:$selector,runs_per_binary:$runs,baseline_median_kib:$baseline_median_kib,candidate_median_kib:$candidate_median_kib,baseline_max_kib:$baseline_max_kib,candidate_max_kib:$candidate_max_kib,median_delta_kib:$median_delta_kib,limit_kib:$limit_kib,verdict:(if $median_delta_kib <= $limit_kib then "PASS" else "FAIL" end)}' \
    >"$output/rss-delta.json"
  jq -e '.verdict=="PASS"' "$output/rss-delta.json" >/dev/null
}

main() {
  [[ -d "$baseline_root" && -d "$candidate_root" ]] || die 'isolated source roots missing'
  [[ ! -e "$output" ]] || die "output already exists: $output"
  mkdir -p "$output/build" "$output/focused" "$output/baseline49/logs" "$output/baseline49/receipts"
  verify_candidate_source
  run_build build-lib-tests scripts/cargo-guard.sh test --lib --no-run
  run_build build-ibus-tests scripts/cargo-guard.sh test --bin lay-ibus-engine --no-run
  run_build build-typing-authority-test scripts/cargo-guard.sh test --test typing_transition_authority_contract --no-run
  run_build build-mutation-monopoly-test scripts/cargo-guard.sh test --test text_mutation_monopoly_contract --no-run
  run_focus common-target-evidence target_evidence
  run_focus live-candidate-adapter live_candidate_projects_the_exact_replacement_evidence_value
  run_focus productive-completeness common_completeness_preserves_complete_overflow_and_failed_states
  run_focus productive-material immutable_field_materialization_preserves_the_complete_readout
  run_focus scene-profile scene_uses_the_canonical_material_normalization_profile
  run_baseline49
  measure_rss
  jq -n --arg created_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --slurpfile baseline49 "$output/baseline49/run-summary.json" \
    --slurpfile parity "$output/baseline49/status-parity.json" \
    --slurpfile rss "$output/rss-delta.json" \
    '{schema:"lay.ime-target-authority-slice1-remote-proof.v1",created_at:$created_at,verdict:"PASS",source_allowlist:"PASS",builds:"4/4 PASS",focused_contract_groups:"5/5 PASS",baseline49:{valid:$baseline49[0].valid_execution_count,pass:$baseline49[0].pass_count,fail:$baseline49[0].fail_count,status_differences:($parity[0].differences|length)},rss:$rss[0],runtime_authority_changed:false,deployment_actions:0}' \
    >"$output/summary.json"
  sha256sum "$output/summary.json" "$output/baseline49/run-summary.json" "$output/baseline49/status-parity.json" "$output/rss-delta.json" >"$output/top-level.sha256"
  jq '.' "$output/summary.json"
}

main "$@"
