#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVAL_BIN="${LAY_NANDA_WAVE_EVAL_BIN:-$ROOT/target/release/lay-nanda-wave-eval}"
TRAIN_BIN="${LAY_NANDA_WAVE_TRAIN_BIN:-$ROOT/target/release/lay-nanda-wave-train}"
OUTPUT="${LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS:-$HOME/.local/share/lay/nanda_wave/word_usage_feedback_counts.json}"
L4_USAGE_EVENTS="${LAY_L4_USAGE_EVENTS:-$HOME/.local/share/lay/nanda_wave/word_usage_events.jsonl}"
L4_CORRECTIONS="${LAY_L4_CORRECTIONS:-$HOME/.local/share/lay/corrections.jsonl}"
L4_OUTPUT="${LAY_L4_CROSS_SCENE_MEMORY:-$HOME/.local/share/lay/nanda_wave/l4_cross_scene_v1.bin}"
LIMIT="${LAY_L4_FEEDBACK_LIMIT:-100000}"

if [[ ! -x "$EVAL_BIN" && ! -x "$TRAIN_BIN" ]]; then
  echo "L4 feedback rebuild skipped: eval and train binaries are missing" >&2
  exit 0
fi

if [[ ! -x "$EVAL_BIN" ]]; then
  echo "L4 feedback rebuild skipped: eval binary missing: $EVAL_BIN" >&2
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/lay-l4-feedback.XXXXXX")"
trap 'rm -rf "$work"' EXIT

if [[ -x "$EVAL_BIN" ]]; then
  "$EVAL_BIN" --dirty-log-collect --out "$work/corpus.jsonl" --limit "$LIMIT" \
    > "$work/collect.json"
  "$EVAL_BIN" --dirty-log-pack-usage --input "$work/corpus.jsonl" \
    --out "$work/usage.jsonl" --limit "$LIMIT" --latest-state-only \
    > "$work/pack.json"

  events="$(jq -r '.events // 0' "$work/pack.json")"
  if [[ "$events" == "0" ]]; then
    echo "L4 feedback counts rebuild skipped: no typed feedback events"
  else
    mkdir -p "$(dirname "$OUTPUT")"
    "$EVAL_BIN" --compile-usage-feedback --input "$work/usage.jsonl" --out "$OUTPUT"
  fi
fi

if [[ ! -x "$TRAIN_BIN" ]]; then
  echo "L4 cross-scene rebuild skipped: train binary missing: $TRAIN_BIN" >&2
  exit 0
fi

if [[ ! -s "$L4_USAGE_EVENTS" && ! -s "$L4_CORRECTIONS" ]]; then
  echo "L4 cross-scene rebuild skipped: no live usage or correction receipts"
  exit 0
fi

usage_input="$L4_USAGE_EVENTS"
if [[ ! -s "$usage_input" ]]; then
  usage_input="$work/empty-usage-events.jsonl"
  : > "$usage_input"
fi

compile_args=(
  --compile-l4-cross-scene
  --input "$usage_input"
  --out "$work/l4-cross-scene.bin"
)
if [[ -s "$L4_CORRECTIONS" ]]; then
  compile_args+=(--corrections "$L4_CORRECTIONS")
fi

"$TRAIN_BIN" "${compile_args[@]}" > "$work/l4-compile.json"
"$TRAIN_BIN" --l4-cross-scene-status "$work/l4-cross-scene.bin" \
  > "$work/l4-status.json"
jq -e '
  .loaded == true
  and .runtime_authority == "shadow_suggest_only"
  and .automatic_apply_possible == false
' "$work/l4-status.json" >/dev/null

mkdir -p "$(dirname "$L4_OUTPUT")"
temporary="$(mktemp "$(dirname "$L4_OUTPUT")/.l4-cross-scene.XXXXXX")"
install -m 0600 "$work/l4-cross-scene.bin" "$temporary"
mv -f "$temporary" "$L4_OUTPUT"

jq -r '
  "L4 cross-scene rebuilt: live=\(.live_source_observations) "
  + "rollback_receipts=\(.backfilled_revert_receipts) "
  + "rollback_observations=\(.backfilled_revert_observations) "
  + "joined=\(.joined_observations) bytes=\(.logical_center_bytes)"
' "$work/l4-compile.json"
