#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVAL_BIN="${LAY_NANDA_WAVE_EVAL_BIN:-$ROOT/target/release/lay-nanda-wave-eval}"
OUTPUT="${LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS:-$HOME/.local/share/lay/nanda_wave/word_usage_feedback_counts.json}"
LIMIT="${LAY_L4_FEEDBACK_LIMIT:-100000}"

if [[ ! -x "$EVAL_BIN" ]]; then
  echo "L4 feedback rebuild skipped: eval binary missing: $EVAL_BIN" >&2
  exit 0
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/lay-l4-feedback.XXXXXX")"
trap 'rm -rf "$work"' EXIT

"$EVAL_BIN" --dirty-log-collect --out "$work/corpus.jsonl" --limit "$LIMIT" \
  > "$work/collect.json"
"$EVAL_BIN" --dirty-log-pack-usage --input "$work/corpus.jsonl" \
  --out "$work/usage.jsonl" --limit "$LIMIT" --latest-state-only \
  > "$work/pack.json"

events="$(jq -r '.events // 0' "$work/pack.json")"
if [[ "$events" == "0" ]]; then
  echo "L4 feedback rebuild skipped: no typed feedback events"
  exit 0
fi

mkdir -p "$(dirname "$OUTPUT")"
"$EVAL_BIN" --compile-usage-feedback --input "$work/usage.jsonl" --out "$OUTPUT"
