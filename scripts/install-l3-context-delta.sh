#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRAINER="${LAY_NANDA_WAVE_TRAIN:-$HOME/.local/lib/lay/bin/lay-nanda-wave-train}"
BASE="${LAY_L3_CONTEXT_BASE:-$HOME/.local/share/lay/nanda_wave/l3_context_phase_v1.nwpc}"
MANIFEST="${LAY_L3_CONTEXT_MANIFEST:-$HOME/.local/share/lay/nanda_wave/l3_context_phase.runtime.json}"
DELTA_DIR="${LAY_L3_CONTEXT_DELTA_DIR:-$HOME/.local/share/lay/nanda_wave/l3-context-deltas}"

usage() {
  echo "usage: $0 --delta PACKAGE.nwpc --cases GATE.tsv --scope NAME" >&2
  exit 2
}

delta=""
cases=""
scope=""
while (($#)); do
  case "$1" in
    --delta)
      delta="${2:-}"
      shift 2
      ;;
    --cases)
      cases="${2:-}"
      shift 2
      ;;
    --scope)
      scope="${2:-}"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

[[ -n "$delta" && -n "$cases" && -n "$scope" ]] || usage
[[ -x "$TRAINER" ]] || { echo "trainer not executable: $TRAINER" >&2; exit 1; }
[[ -f "$BASE" ]] || { echo "L3 base not found: $BASE" >&2; exit 1; }
[[ -f "$delta" ]] || { echo "delta not found: $delta" >&2; exit 1; }
[[ -f "$cases" ]] || { echo "gate cases not found: $cases" >&2; exit 1; }

mkdir -p "$DELTA_DIR" "$(dirname "$MANIFEST")"
if [[ ! -f "$MANIFEST" ]]; then
  "$TRAINER" --init-l3-context-composite --manifest "$MANIFEST" --base "$BASE" >/dev/null
fi

destination="$DELTA_DIR/$(basename "$delta")"
receipt="$DELTA_DIR/$(basename "${delta%.nwpc}").proof.json"
if jq -e --arg path "$destination" '.deltas[]? | select(.path == $path)' "$MANIFEST" >/dev/null; then
  echo "L3 delta already admitted: $destination"
  exit 0
fi

temporary="$(mktemp "$DELTA_DIR/.delta.XXXXXX")"
trap 'rm -f "$temporary"' EXIT
install -m 0600 "$delta" "$temporary"
[[ "$(sha256sum "$delta" | cut -d' ' -f1)" == "$(sha256sum "$temporary" | cut -d' ' -f1)" ]]
mv -f "$temporary" "$destination"
trap - EXIT

if ! "$TRAINER" --prove-l3-context-delta \
  --manifest "$MANIFEST" \
  --delta "$destination" \
  --cases "$cases" \
  --out-receipt "$receipt" >/dev/null; then
  rm -f "$destination" "$receipt"
  echo "L3 delta targeted proof failed; manifest unchanged" >&2
  exit 1
fi

if ! "$TRAINER" --admit-l3-context-delta \
  --manifest "$MANIFEST" \
  --delta "$destination" \
  --proof-receipt "$receipt" \
  --scope "$scope"; then
  rm -f "$destination" "$receipt"
  echo "L3 delta admission failed; manifest unchanged" >&2
  exit 1
fi
