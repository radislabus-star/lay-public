#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/tests/fixtures/td007-package-set-v1.json"

if [[ $# -ne 1 ]]; then
  echo "usage: $0 RECEIPT.json" >&2
  exit 2
fi

RECEIPT="$1"
if [[ "$RECEIPT" != /* ]]; then
  RECEIPT="$ROOT/$RECEIPT"
fi
if [[ -e "$RECEIPT" ]]; then
  echo "refusing to overwrite package proof receipt: $RECEIPT" >&2
  exit 2
fi

PACKAGE_ROOT="${LAY_TD007_PACKAGE_ROOT:-$(jq -r '.root_default' "$MANIFEST")}"
declare -A PACKAGE_PATHS
while IFS=$'\t' read -r role relative bytes expected_sha; do
  path="$PACKAGE_ROOT/$relative"
  [[ -f "$path" ]] || { echo "missing $role: $path" >&2; exit 1; }
  actual_bytes="$(stat -c %s "$path")"
  actual_sha="$(sha256sum "$path" | awk '{print $1}')"
  [[ "$actual_bytes" == "$bytes" ]] || { echo "$role size mismatch" >&2; exit 1; }
  [[ "$actual_sha" == "$expected_sha" ]] || { echo "$role SHA-256 mismatch" >&2; exit 1; }
  PACKAGE_PATHS["$role"]="$path"
done < <(jq -r '.files[] | [.role,.relative_path,(.bytes|tostring),.sha256] | @tsv' "$MANIFEST")

[[ "$(jq -r '.installed_artifact' "${PACKAGE_PATHS[l11_receipt]}")" == "${PACKAGE_PATHS[l11_package]}" ]]
[[ "$(jq -r '.proof_receipt' "${PACKAGE_PATHS[l11_receipt]}")" == "${PACKAGE_PATHS[l11_proof]}" ]]
historical_case_count="$(jq -r '.historical_cases | length' "$MANIFEST")"
[[ "$historical_case_count" == 8 ]] || { echo "expected eight historical cases" >&2; exit 1; }
jq -e '
  .historical_cases
  | all(
      (.input | type == "string" and length > 0)
      and (.expected | type == "string" and length > 0)
      and (
        .expected_status == "COVERED_BY_L11_PRODUCER"
        or .expected_status == "ABSTAIN_NO_UNVERIFIED_APPLY"
      )
    )
' "$MANIFEST" >/dev/null
[[ "$(jq '[.historical_cases[] | select(.expected_status == "COVERED_BY_L11_PRODUCER")] | length' "$MANIFEST")" == 1 ]]
[[ "$(jq '[.historical_cases[] | select(.expected_status == "ABSTAIN_NO_UNVERIFIED_APPLY")] | length' "$MANIFEST")" == 7 ]]

source_closure_before="$(python3 - <<'PY'
import json
from scripts.test_lanes.cli import source_closure_identity
print(json.dumps(source_closure_identity(), sort_keys=True))
PY
)"

cd "$ROOT"
scripts/cargo-guard.sh build --locked --offline --bin lay-l11-serve >/dev/null

stage="$(mktemp -d "${TMPDIR:-/tmp}/lay-td007-package-proof.XXXXXX")"
socket="$stage/l11.sock"
service_log="$stage/l11-service.log"
service_pid=""
cleanup() {
  if [[ -n "$service_pid" ]] && kill -0 "$service_pid" 2>/dev/null; then
    kill "$service_pid" 2>/dev/null || true
    wait "$service_pid" 2>/dev/null || true
  fi
  rm -rf "$stage"
}
trap cleanup EXIT

env LAY_L11_RECEIPT="${PACKAGE_PATHS[l11_receipt]}" \
  target/debug/lay-l11-serve run \
  --memory "${PACKAGE_PATHS[l11_package]}" \
  --socket "$socket" >"$service_log" 2>&1 &
service_pid=$!

ready=0
for _ in $(seq 1 240); do
  if ! kill -0 "$service_pid" 2>/dev/null; then
    tail -100 "$service_log" >&2
    exit 1
  fi
  health="$(target/debug/lay-l11-serve health --socket "$socket" 2>/dev/null || true)"
  if [[ "$(jq -r '.report.status // empty' <<<"$health" 2>/dev/null || true)" == "ready" ]]; then
    ready=1
    break
  fi
  sleep 0.5
done
[[ "$ready" == 1 ]] || { tail -100 "$service_log" >&2; exit 1; }

lattice_ready=0
for _ in $(seq 1 480); do
  if python3 - "$socket" <<'PY'
import json
import socket
import sys

request = {"type": "lattice", "surface": "врмея", "limit": 32}
with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
    client.settimeout(1.0)
    client.connect(sys.argv[1])
    client.sendall(json.dumps(request, ensure_ascii=False).encode() + b"\n")
    response = b""
    while not response.endswith(b"\n"):
        chunk = client.recv(65536)
        if not chunk:
            break
        response += chunk

payload = json.loads(response)
if payload.get("type") != "lattice":
    raise SystemExit(1)
if not any(seed.get("surface") == "время" for seed in payload.get("seeds", [])):
    raise SystemExit(1)
PY
  then
    lattice_ready=1
    break
  fi
  sleep 0.5
done
[[ "$lattice_ready" == 1 ]] || {
  echo "L1.1 lattice did not expose the pinned время seed after warmup" >&2
  tail -100 "$service_log" >&2
  exit 1
}

run_package_test() {
  local test_id="$1"
  local log="$2"
  local expected_passes="$3"
  if ! env \
    LAY_L11_RECEIPT="${PACKAGE_PATHS[l11_receipt]}" \
    LAY_L11_SOCKET="$socket" \
    LAY_L2_PACKAGE="${PACKAGE_PATHS[canonical_l2]}" \
    LAY_L2_PRODUCTIVE_V1_PACKAGE="${PACKAGE_PATHS[productive_v90]}" \
    LAY_L2_PRODUCTIVE_PACKAGE="${PACKAGE_PATHS[productive_sidecar]}" \
    LAY_L2_V13_DAFSA="${PACKAGE_PATHS[v13_dafsa]}" \
    scripts/cargo-guard.sh test --locked --offline --lib "$test_id" -- \
      --ignored --test-threads=1 >"$log" 2>&1; then
    cat "$log" >&2
    return 1
  fi
  rg -q "test result: ok\\. ${expected_passes} passed; 0 failed; 0 ignored;" "$log"
}

producer_test="correction_core::candidate_sources_tests::td007_pinned_canonical_route_internalizes_authoritative_l11_seed"
historical_cases_test="correction_core::tests::td007_pinned_canonical_route_reconciles_historical_cases"
package_test_filter="td007_pinned_canonical_route"
package_test_log="$stage/package-tests.log"
run_package_test "$package_test_filter" "$package_test_log" 2

kill "$service_pid"
wait "$service_pid" || true
service_pid=""

source_closure_after="$(python3 - <<'PY'
import json
from scripts.test_lanes.cli import source_closure_identity
print(json.dumps(source_closure_identity(), sort_keys=True))
PY
)"
[[ "$source_closure_after" == "$source_closure_before" ]] || {
  echo "source closure changed during package proof" >&2
  exit 1
}

mkdir -p "$(dirname "$RECEIPT")"
receipt_tmp="$RECEIPT.tmp.$$"
jq -n \
  --arg recorded_at_utc "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg source_head "$(git rev-parse HEAD)" \
  --argjson source_closure "$source_closure_before" \
  --arg controller_sha256 "$(sha256sum "$0" | awk '{print $1}')" \
  --arg package_manifest_sha256 "$(sha256sum "$MANIFEST" | awk '{print $1}')" \
  --arg producer_test "$producer_test" \
  --arg historical_cases_test "$historical_cases_test" \
  --arg package_test_filter "$package_test_filter" \
  --arg package_test_log_sha256 "$(sha256sum "$package_test_log" | awk '{print $1}')" \
  --argjson historical_case_count "$historical_case_count" \
  --slurpfile package_set "$MANIFEST" \
  '{
    schema: "lay.td007.package-proof.v2",
    verdict: "TD007_PINNED_PACKAGE_PROOF_PASS",
    recorded_at_utc: $recorded_at_utc,
    source_head: $source_head,
    source_closure: $source_closure,
    source_closure_changed_during_execution: false,
    controller_sha256: $controller_sha256,
    package_manifest_sha256: $package_manifest_sha256,
    package_set: $package_set[0],
    tests: [
      {id: $producer_test, verdict: "PASS", log_sha256: $package_test_log_sha256},
      {id: $historical_cases_test, verdict: "PASS", log_sha256: $package_test_log_sha256}
    ],
    test_filter: $package_test_filter,
    historical_case_contract: $package_set[0].historical_cases,
    historical_cases_covered: $historical_case_count,
    l11_lattice_ready_probe: {
      input: "врмея",
      expected_seed: "время",
      verdict: "PASS"
    },
    cargo_test_processes: 1,
    installed_runtime_mutated: false,
    installed_runtime_authority_changed: false,
    runtime_authority_scope: "installed_live_runtime",
    runtime_authority_changed: false,
    live_desktop_contacted: false
  }' >"$receipt_tmp"
chmod 0444 "$receipt_tmp"
mv "$receipt_tmp" "$RECEIPT"
echo "td007_package_proof=PASS receipt=$RECEIPT"
