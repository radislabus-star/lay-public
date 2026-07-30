#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

INSTALL=0
INCLUDE_LIVE_FEEDBACK=0
MAX_PHRASES="${LAY_SELF_TEACHER_L3_MAX_PHRASES:-256}"
MAX_PAIRS="${LAY_SELF_TEACHER_L3_MAX_PAIRS:-2000}"
WORK_ROOT="${LAY_SELF_TEACHER_L3_PROMOTION_WORK:-$HOME/.local/share/lay/self_teacher/l3/promotions}"
BASE="${LAY_L3_BASE_CONTEXT_PHASE:-$ROOT/data/lexicon/l3_context_phase_v1.nwpc}"
USAGE_EVENTS="${LAY_SELF_TEACHER_L3_USAGE_EVENTS:-}"
BIN_DIR="${LAY_SELF_TEACHER_L3_BIN_DIR:-}"
RUNTIME_MANIFEST="${LAY_L3_CONTEXT_MANIFEST:-$HOME/.local/share/lay/nanda_wave/l3_context_phase.runtime.json}"
FULL_PROOF_CORPUS="${LAY_L3_FULL_PROOF_CORPUS:-}"
FULL_PROOF_SURFACE="${LAY_L3_FULL_PROOF_SURFACE:-}"

usage() {
  cat >&2 <<'EOF'
usage: scripts/l3-self-teacher-promotion-gate.sh [--install] [--include-live-feedback] [--usage-events PATH] [--use-runtime-base] [--max-phrases N] [--max-pairs N] [--work DIR] [--base PATH] [--runtime-manifest PATH] [--full-proof-corpus PATH] [--full-proof-surface PATH]

Build a local L3 self-teacher delta, prove it against the current append-only
runtime manifest and a frozen full differential corpus, and produce a promotion
receipt. Runtime install happens only with --install and only after all gates
pass. The immutable L3 base is never rewritten.

Default training is clean/self-generated only. Local live usage feedback is
included only with --include-live-feedback.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install) INSTALL=1; shift ;;
    --include-live-feedback) INCLUDE_LIVE_FEEDBACK=1; shift ;;
    --usage-events) USAGE_EVENTS="${2:?missing value for --usage-events}"; shift 2 ;;
    --use-runtime-base) BASE="$HOME/.local/share/lay/nanda_wave/l3_context_phase.nwpc"; shift ;;
    --max-phrases) MAX_PHRASES="${2:?missing value for --max-phrases}"; shift 2 ;;
    --max-pairs) MAX_PAIRS="${2:?missing value for --max-pairs}"; shift 2 ;;
    --work) WORK_ROOT="${2:?missing value for --work}"; shift 2 ;;
    --base) BASE="${2:?missing value for --base}"; shift 2 ;;
    --runtime-manifest) RUNTIME_MANIFEST="${2:?missing value for --runtime-manifest}"; shift 2 ;;
    --full-proof-corpus) FULL_PROOF_CORPUS="${2:?missing value for --full-proof-corpus}"; shift 2 ;;
    --full-proof-surface) FULL_PROOF_SURFACE="${2:?missing value for --full-proof-surface}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
done

if [[ "$INCLUDE_LIVE_FEEDBACK" != "1" && -n "$USAGE_EVENTS" ]]; then
  echo "--usage-events requires --include-live-feedback" >&2
  exit 2
fi

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
  if [[ -n "$BIN_DIR" && -x "$BIN_DIR/$bin" ]]; then
    "$BIN_DIR/$bin" "$@"
  elif [[ -x "$ROOT/scripts/cargo-guard.sh" ]]; then
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
DELTA="$WORK/l3_self_teacher_delta_${RUN_ID}.nwpc"
DELTA_CASES="$SELF_DIR/delta_gate_cases.tsv"
TARGETED_RECEIPT="$WORK/targeted-proof.json"
BASELINE_COMPACT="$WORK/l3_context_phase_baseline.nwpc"
CANDIDATE_MANIFEST="$WORK/candidate.runtime.json"
CANDIDATE_COMPACT="$WORK/l3_context_phase_candidate.nwpc"
FULL_RECEIPT="$WORK/full-differential-proof.json"
RUNTIME_STATUS="$WORK/runtime-status.json"
CANDIDATE_STATUS="$WORK/candidate-status.json"
TRANSITION_REPLAY="$WORK/transition-replay.json"
UNSAFE_GATE="$WORK/unsafe-gate.json"
RECEIPT="$WORK/promotion-receipt.json"

if [[ -z "$FULL_PROOF_CORPUS" || ! -s "$FULL_PROOF_CORPUS" ]]; then
  echo "missing frozen full L3 proof corpus; use --full-proof-corpus PATH" >&2
  exit 1
fi
if [[ -z "$FULL_PROOF_SURFACE" || ! -s "$FULL_PROOF_SURFACE" ]]; then
  echo "missing frozen full L3 surface evidence; use --full-proof-surface PATH" >&2
  exit 1
fi
if [[ ! -s "$RUNTIME_MANIFEST" ]]; then
  RUNTIME_MANIFEST="$WORK/baseline.runtime.json"
  run_bin lay-nanda-wave-train --init-l3-context-composite \
    --manifest "$RUNTIME_MANIFEST" \
    --base "$BASE" >/dev/null
fi

SELF_ARGS=(
  --lay-self-teacher-l3
  --max-phrases "$MAX_PHRASES"
  --max-pairs "$MAX_PAIRS"
  --out-dir "$SELF_DIR"
  --runtime-manifest "$RUNTIME_MANIFEST"
)
if [[ "$INCLUDE_LIVE_FEEDBACK" == "1" ]]; then
  if [[ -n "$USAGE_EVENTS" ]]; then
    SELF_ARGS+=(--usage-events "$USAGE_EVENTS")
  fi
else
  SELF_ARGS+=(--no-live-feedback)
fi

run_bin lay-nanda-wave-eval "${SELF_ARGS[@]}" > "$SELF_REPORT"

SELF_PACKAGE="$(jq -r '.artifacts.delta_package // .artifacts.shadow_package' "$SELF_REPORT")"
if [[ ! -s "$SELF_PACKAGE" ]]; then
  echo "self-teacher package missing: $SELF_PACKAGE" >&2
  exit 1
fi
if [[ "$(jq -r '.delta_gate.verdict // "WATCH"' "$SELF_REPORT")" != "READY" \
  || ! -s "$DELTA_CASES" ]]; then
  echo "self-teacher produced no manifest-scoped delta improvements" >&2
  exit 1
fi
install -m 0600 "$SELF_PACKAGE" "$DELTA"

run_bin lay-nanda-wave-train --snapshot-l3-context-composite \
  --manifest "$RUNTIME_MANIFEST" \
  --out "$BASELINE_COMPACT" >/dev/null

targeted_rc=0
run_bin lay-nanda-wave-train --prove-l3-context-delta \
  --manifest "$RUNTIME_MANIFEST" \
  --delta "$DELTA" \
  --cases "$DELTA_CASES" \
  --out-receipt "$TARGETED_RECEIPT" >/dev/null || targeted_rc=$?

run_bin lay-nanda-wave-train --init-l3-context-composite \
  --manifest "$CANDIDATE_MANIFEST" \
  --base "$BASELINE_COMPACT" >/dev/null
if [[ "$targeted_rc" == "0" ]]; then
  run_bin lay-nanda-wave-train --admit-l3-context-delta \
    --manifest "$CANDIDATE_MANIFEST" \
    --delta "$DELTA" \
    --proof-receipt "$TARGETED_RECEIPT" \
    --scope "self-teacher-$RUN_ID" >/dev/null
  run_bin lay-nanda-wave-train --compact-l3-context-composite \
    --manifest "$CANDIDATE_MANIFEST" \
    --out "$CANDIDATE_COMPACT" >/dev/null
fi

full_rc=1
if [[ "$targeted_rc" == "0" && -s "$CANDIDATE_COMPACT" ]]; then
  full_rc=0
  run_bin lay-nanda-wave-train --prove-l3-context-phase-delta-full \
    "$FULL_PROOF_CORPUS" \
    --baseline-memory "$BASELINE_COMPACT" \
    --memory "$CANDIDATE_COMPACT" \
    --surface-evidence "$FULL_PROOF_SURFACE" \
    --min-surface-support 2 \
    --max-fragments 80000 \
    --out-receipt "$FULL_RECEIPT" >/dev/null || full_rc=$?
fi

run_bin lay-nanda-wave-train --l3-context-phase-status \
  --memory "$BASELINE_COMPACT" > "$RUNTIME_STATUS"
if [[ -s "$CANDIDATE_COMPACT" ]]; then
  run_bin lay-nanda-wave-train --l3-context-phase-status \
    --memory "$CANDIDATE_COMPACT" > "$CANDIDATE_STATUS"
else
  printf '{}\n' > "$CANDIDATE_STATUS"
fi

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
targeted_pass=false
if [[ "$targeted_rc" == "0" ]] \
  && jq -e '.verdict == "PASS" and .target_failures == 0 and .false_supports == 0' "$TARGETED_RECEIPT" >/dev/null; then
  targeted_pass=true
fi
full_pass=false
if [[ "$full_rc" == "0" ]] \
  && jq -e '.verdict == "PASS" and .lost_target_profiles == 0 and .lost_supports == 0 and .lost_top1 == 0 and .new_false_supports == 0 and .new_false_top1 == 0' "$FULL_RECEIPT" >/dev/null; then
  full_pass=true
fi

gate_pass="false"
if [[ "$self_pass" == "true" \
  && "$targeted_pass" == "true" \
  && "$full_pass" == "true" \
  && "$transition_pass" == "true" \
  && "$unsafe_pass" == "true" ]]; then
  gate_pass="true"
fi
install_requested="false"
if [[ "$INSTALL" == "1" ]]; then
  install_requested="true"
fi
include_live_feedback="false"
if [[ "$INCLUDE_LIVE_FEEDBACK" == "1" ]]; then
  include_live_feedback="true"
fi
[[ -s "$TARGETED_RECEIPT" ]] || printf '{}\n' > "$TARGETED_RECEIPT"
[[ -s "$FULL_RECEIPT" ]] || printf '{}\n' > "$FULL_RECEIPT"

jq -n \
  --arg kind "l3_self_teacher_promotion_receipt" \
  --arg run_id "$RUN_ID" \
  --arg base "$BASE" \
  --arg runtime_manifest "$RUNTIME_MANIFEST" \
  --arg self_package "$SELF_PACKAGE" \
  --arg delta "$DELTA" \
  --arg delta_sha256 "$(sha256sum "$DELTA" | cut -d' ' -f1)" \
  --arg full_proof_corpus "$FULL_PROOF_CORPUS" \
  --arg full_proof_surface "$FULL_PROOF_SURFACE" \
  --arg targeted_receipt "$TARGETED_RECEIPT" \
  --arg full_receipt "$FULL_RECEIPT" \
  --argjson install_requested "$install_requested" \
  --argjson include_live_feedback "$include_live_feedback" \
  --argjson self_pass "$self_pass" \
  --argjson targeted_pass "$targeted_pass" \
  --argjson full_pass "$full_pass" \
  --argjson transition_pass "$transition_pass" \
  --argjson unsafe_pass "$unsafe_pass" \
  --argjson gate_pass "$gate_pass" \
  --slurpfile self "$SELF_REPORT" \
  --slurpfile targeted "$TARGETED_RECEIPT" \
  --slurpfile full "$FULL_RECEIPT" \
  --slurpfile runtime_status "$RUNTIME_STATUS" \
  --slurpfile candidate_status "$CANDIDATE_STATUS" \
  --slurpfile transition "$TRANSITION_REPLAY" \
  --slurpfile unsafe "$UNSAFE_GATE" \
  '{
    kind: $kind,
    run_id: $run_id,
    runtime_authority: false,
    install_requested: $install_requested,
    teacher_input: {
      mode: (if $include_live_feedback then "clean_plus_explicit_live_feedback" else "clean_self_generated_only" end),
      include_live_feedback: $include_live_feedback
    },
    gate_pass: $gate_pass,
    gates: {
      self_teacher_shadow_pass: $self_pass,
      targeted_delta_pass: $targeted_pass,
      full_differential_pass: $full_pass,
      transition_replay_pass: $transition_pass,
      unsafe_gate_pass: $unsafe_pass
    },
    artifacts: {
      base_package: $base,
      runtime_manifest: $runtime_manifest,
      self_teacher_package: $self_package,
      append_only_delta: $delta,
      append_only_delta_sha256: $delta_sha256,
      targeted_cases: $self[0].artifacts.delta_gate_cases,
      targeted_receipt: $targeted_receipt,
      full_differential_receipt: $full_receipt,
      baseline_compact: $runtime_status[0].path,
      candidate_compact: $candidate_status[0].path,
      full_proof_corpus: $full_proof_corpus,
      full_proof_surface: $full_proof_surface
    },
    self_teacher_shadow: $self[0].shadow,
    delta_gate: $self[0].delta_gate,
    targeted_proof: $targeted[0],
    full_differential_proof: $full[0],
    baseline_status: $runtime_status[0],
    candidate_status: $candidate_status[0],
    live_shadow: {
      transition_replay: $transition[0],
      unsafe_gate: $unsafe[0]
    },
    base_rewritten: false,
    promotion_rule: "append delta only after self shadow, targeted delta, full differential, transition replay, and unsafe gates all PASS"
  }' > "$RECEIPT"

if [[ "$gate_pass" != "true" ]]; then
  jq '{kind, gate_pass, gates, artifacts, self_teacher_shadow: {verdict: .self_teacher_shadow.verdict, target_top1_percent: .self_teacher_shadow.target_top1_percent, false_top1_percent: .self_teacher_shadow.false_top1_percent, authority_percent: .self_teacher_shadow.authority_percent, false_authority_percent: .self_teacher_shadow.false_authority_percent}, live_shadow: {transition_verdict: .live_shadow.transition_replay.verdict, unsafe_verdict: .live_shadow.unsafe_gate.verdict}}' "$RECEIPT"
  exit 1
fi

if [[ "$INSTALL" == "1" ]]; then
  trainer="${BIN_DIR:+$BIN_DIR/lay-nanda-wave-train}"
  trainer="${trainer:-$HOME/.local/lib/lay/bin/lay-nanda-wave-train}"
  LAY_NANDA_WAVE_TRAIN="$trainer" \
    LAY_L3_CONTEXT_BASE="$BASE" \
    LAY_L3_CONTEXT_MANIFEST="$RUNTIME_MANIFEST" \
    "$ROOT/scripts/install-l3-context-delta.sh" \
      --delta "$DELTA" \
      --cases "$DELTA_CASES" \
      --scope "self-teacher-$RUN_ID"
  "$ROOT/scripts/reload-lay-model-services.sh"
fi

jq '{kind, gate_pass, install_requested, artifacts, self_teacher_shadow: {verdict: .self_teacher_shadow.verdict, target_top1_percent: .self_teacher_shadow.target_top1_percent, false_top1_percent: .self_teacher_shadow.false_top1_percent, authority_percent: .self_teacher_shadow.authority_percent, false_authority_percent: .self_teacher_shadow.false_authority_percent}, live_shadow: {transition_verdict: .live_shadow.transition_replay.verdict, unsafe_verdict: .live_shadow.unsafe_gate.verdict}}' "$RECEIPT"
printf 'receipt=%s\n' "$RECEIPT"
