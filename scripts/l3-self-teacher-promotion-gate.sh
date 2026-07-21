#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

INSTALL=0
MAX_PHRASES="${LAY_SELF_TEACHER_L3_MAX_PHRASES:-256}"
MAX_PAIRS="${LAY_SELF_TEACHER_L3_MAX_PAIRS:-2000}"
WORK_ROOT="${LAY_SELF_TEACHER_L3_PROMOTION_WORK:-$HOME/.local/share/lay/self_teacher/l3/promotions}"
BASE="${LAY_L3_BASE_CONTEXT_PHASE:-$HOME/.local/share/lay/nanda_wave/l3_context_phase.nwpc}"

usage() {
  cat >&2 <<'EOF'
usage: scripts/l3-self-teacher-promotion-gate.sh [--install] [--max-phrases N] [--max-pairs N] [--work DIR] [--base PATH]

Build a local L3 self-teacher shard, merge it with the current L3 context phase
package, and produce a promotion receipt. Runtime install happens only with
--install and only after all gates pass.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install) INSTALL=1; shift ;;
    --max-phrases) MAX_PHRASES="${2:?missing value for --max-phrases}"; shift 2 ;;
    --max-pairs) MAX_PAIRS="${2:?missing value for --max-pairs}"; shift 2 ;;
    --work) WORK_ROOT="${2:?missing value for --work}"; shift 2 ;;
    --base) BASE="${2:?missing value for --base}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
done

if [[ ! -s "$BASE" ]]; then
  BASE="$ROOT/data/lexicon/l3_context_phase_v1.nwpc"
fi
if [[ ! -s "$BASE" ]]; then
  echo "missing base L3 context phase package" >&2
  exit 1
fi

run_bin() {
  local bin="$1"
  shift
  if [[ -x "$ROOT/scripts/cargo-guard.sh" ]]; then
    "$ROOT/scripts/cargo-guard.sh" run --quiet --bin "$bin" -- "$@"
  elif command -v "$bin" >/dev/null 2>&1; then
    "$bin" "$@"
  else
    echo "missing binary: $bin" >&2
    exit 127
  fi
}

RUN_ID="$(date +%Y%m%d-%H%M%S)"
WORK="$WORK_ROOT/$RUN_ID"
SELF_DIR="$WORK/self-teacher"
mkdir -p "$SELF_DIR"

SELF_REPORT="$WORK/self-teacher-report.json"
MERGED="$WORK/l3_context_phase_candidate.nwpc"
MERGED_MANIFEST="${MERGED%.nwpc}.manifest.json"
MERGE_REPORT="$WORK/merge-report.json"
BASE_STATUS="$WORK/base-status.json"
MERGED_STATUS="$WORK/merged-status.json"
TRANSITION_REPLAY="$WORK/transition-replay.json"
UNSAFE_GATE="$WORK/unsafe-gate.json"
RECEIPT="$WORK/promotion-receipt.json"

run_bin lay-nanda-wave-eval --lay-self-teacher-l3 \
  --max-phrases "$MAX_PHRASES" \
  --max-pairs "$MAX_PAIRS" \
  --out-dir "$SELF_DIR" > "$SELF_REPORT"

SELF_PACKAGE="$(jq -r '.artifacts.shadow_package' "$SELF_REPORT")"
if [[ ! -s "$SELF_PACKAGE" ]]; then
  echo "self-teacher package missing: $SELF_PACKAGE" >&2
  exit 1
fi

run_bin lay-nanda-wave-train --merge-l3-context-phase-shards \
  --input "$BASE" \
  --input "$SELF_PACKAGE" \
  --out "$MERGED" \
  --min-surface-support 1 > "$MERGE_REPORT"

run_bin lay-nanda-wave-train --l3-context-phase-status --memory "$BASE" > "$BASE_STATUS"
run_bin lay-nanda-wave-train --l3-context-phase-status --memory "$MERGED" > "$MERGED_STATUS"

transition_rc=0
run_bin lay-debug-actions --transition-replay > "$TRANSITION_REPLAY" || transition_rc=$?
unsafe_rc=0
run_bin lay-debug-actions --unsafe-gate > "$UNSAFE_GATE" || unsafe_rc=$?

self_pass=false
if jq -e '.shadow.verdict == "PASS_shadow" and .shadow.false_authority == 0 and .shadow.false_top1 == 0 and .shadow.candidate_order_changed == 0' "$SELF_REPORT" >/dev/null; then
  self_pass=true
fi
transition_pass=false
if jq -e '.verdict == "PASS-shadow" and .transition.false_apply_candidates == 0 and .transition.unverified_transitions == 0 and .transition.unverified_left_context_mutations == 0' "$TRANSITION_REPLAY" >/dev/null; then
  transition_pass=true
fi
unsafe_pass=false
if jq -e '.verdict == "PASS" and .records.gate_failures == 0' "$UNSAFE_GATE" >/dev/null; then
  unsafe_pass=true
fi
not_shrunk=false
if jq -s '.[1].bytes >= .[0].bytes and .[1].candidate_profiles >= .[0].candidate_profiles and .[1].semantic_states >= .[0].semantic_states' "$BASE_STATUS" "$MERGED_STATUS" >/dev/null; then
  not_shrunk=true
fi

gate_pass="false"
if [[ "$self_pass" == "true" && "$transition_pass" == "true" && "$unsafe_pass" == "true" && "$not_shrunk" == "true" ]]; then
  gate_pass="true"
fi
install_requested="false"
if [[ "$INSTALL" == "1" ]]; then
  install_requested="true"
fi

jq -n \
  --arg kind "l3_self_teacher_promotion_receipt" \
  --arg run_id "$RUN_ID" \
  --arg base "$BASE" \
  --arg self_package "$SELF_PACKAGE" \
  --arg merged "$MERGED" \
  --argjson install_requested "$install_requested" \
  --argjson self_pass "$self_pass" \
  --argjson transition_pass "$transition_pass" \
  --argjson unsafe_pass "$unsafe_pass" \
  --argjson not_shrunk "$not_shrunk" \
  --argjson gate_pass "$gate_pass" \
  --slurpfile self "$SELF_REPORT" \
  --slurpfile merge "$MERGE_REPORT" \
  --slurpfile base_status "$BASE_STATUS" \
  --slurpfile merged_status "$MERGED_STATUS" \
  --slurpfile transition "$TRANSITION_REPLAY" \
  --slurpfile unsafe "$UNSAFE_GATE" \
  '{
    kind: $kind,
    run_id: $run_id,
    runtime_authority: false,
    install_requested: $install_requested,
    gate_pass: $gate_pass,
    gates: {
      self_teacher_shadow_pass: $self_pass,
      transition_replay_pass: $transition_pass,
      unsafe_gate_pass: $unsafe_pass,
      merged_package_not_shrunk: $not_shrunk
    },
    artifacts: {
      base_package: $base,
      self_teacher_package: $self_package,
      merged_candidate_package: $merged
    },
    self_teacher_shadow: $self[0].shadow,
    merge: $merge[0],
    base_status: $base_status[0],
    merged_status: $merged_status[0],
    live_shadow: {
      transition_replay: $transition[0],
      unsafe_gate: $unsafe[0]
    },
    promotion_rule: "install only after self shadow PASS, transition replay PASS, unsafe gate PASS, and merged package not smaller than base"
  }' > "$RECEIPT"

jq -n \
  --arg artifact "$(basename "$MERGED")" \
  --arg artifact_sha256 "$(sha256sum "$MERGED" | cut -d' ' -f1)" \
  --arg receipt "$RECEIPT" \
  --argjson receipt_json "$(cat "$RECEIPT")" \
  '{
    format: "LAYL3P01",
    version: 4,
    artifact: $artifact,
    artifact_sha256: $artifact_sha256,
    raw_words_stored: false,
    source: "merged current runtime L3 context package plus local L3 self-teacher shadow shard",
    promotion_receipt: $receipt,
    promotion: $receipt_json
  }' > "$MERGED_MANIFEST"

if [[ "$gate_pass" != "true" ]]; then
  jq '{kind, gate_pass, gates, artifacts, self_teacher_shadow: {verdict: .self_teacher_shadow.verdict, target_top1_percent: .self_teacher_shadow.target_top1_percent, false_top1_percent: .self_teacher_shadow.false_top1_percent, authority_percent: .self_teacher_shadow.authority_percent, false_authority_percent: .self_teacher_shadow.false_authority_percent}, live_shadow: {transition_verdict: .live_shadow.transition_replay.verdict, unsafe_verdict: .live_shadow.unsafe_gate.verdict}}' "$RECEIPT"
  exit 1
fi

if [[ "$INSTALL" == "1" ]]; then
  preserved_layout="$(timeout 1s ibus engine 2>/dev/null || true)"
  case "$preserved_layout" in
    lay-ime-us|lay-ime-ru) ;;
    *) preserved_layout=lay-ime-ru ;;
  esac
  LAY_L3_CONTEXT_PHASE_SOURCE="$MERGED" "$ROOT/scripts/install-l3-context-phase.sh"
  systemctl --user restart lay-daemon.service
  pkill -x lay-ibus-engine 2>/dev/null || true
  ibus restart
  sleep 1
  ibus engine "$preserved_layout" >/dev/null 2>&1 || true
fi

jq '{kind, gate_pass, install_requested, artifacts, self_teacher_shadow: {verdict: .self_teacher_shadow.verdict, target_top1_percent: .self_teacher_shadow.target_top1_percent, false_top1_percent: .self_teacher_shadow.false_top1_percent, authority_percent: .self_teacher_shadow.authority_percent, false_authority_percent: .self_teacher_shadow.false_authority_percent}, live_shadow: {transition_verdict: .live_shadow.transition_replay.verdict, unsafe_verdict: .live_shadow.unsafe_gate.verdict}}' "$RECEIPT"
printf 'receipt=%s\n' "$RECEIPT"
