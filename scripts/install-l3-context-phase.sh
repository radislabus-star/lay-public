#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="${LAY_L3_CONTEXT_PHASE_SOURCE:-$ROOT/data/lexicon/l3_context_phase_v1.nwpc}"
MANIFEST="${SOURCE%.nwpc}.manifest.json"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
DEST="${LAY_NANDA_L3_CONTEXT_MEMORY:-$DATA_HOME/lay/nanda_wave/l3_context_phase.nwpc}"
DEST_MANIFEST="${DEST%.nwpc}.manifest.json"

for required in "$SOURCE" "$MANIFEST"; do
  if [[ ! -s "$required" ]]; then
    echo "L3 context phase artifact missing: $required" >&2
    exit 1
  fi
done

declared_bytes="$(jq -er '.artifact_bytes | numbers' "$MANIFEST")"
declared_sha256="$(jq -er '.artifact_sha256 | strings' "$MANIFEST")"
heldout_verdict="$(jq -er '.heldout.verdict | strings' "$MANIFEST")"
actual_bytes="$(stat -c %s "$SOURCE")"
actual_sha256="$(sha256sum "$SOURCE" | cut -d' ' -f1)"
if [[ "$actual_bytes" != "$declared_bytes" || "$actual_sha256" != "$declared_sha256" ]]; then
  echo "L3 context phase artifact does not match its proof manifest" >&2
  echo "declared bytes=$declared_bytes sha256=$declared_sha256" >&2
  echo "actual   bytes=$actual_bytes sha256=$actual_sha256" >&2
  exit 1
fi
if [[ "$heldout_verdict" != "PASS" ]]; then
  echo "L3 context phase manifest is not a heldout PASS: $heldout_verdict" >&2
  exit 1
fi

mkdir -p "$(dirname "$DEST")"
artifact_tmp="${DEST}.tmp.$$"
manifest_tmp="${DEST_MANIFEST}.tmp.$$"
trap 'rm -f "$artifact_tmp" "$manifest_tmp"' EXIT
install -m 0644 "$SOURCE" "$artifact_tmp"
install -m 0644 "$MANIFEST" "$manifest_tmp"
mv -f "$artifact_tmp" "$DEST"
mv -f "$manifest_tmp" "$DEST_MANIFEST"
trap - EXIT

printf 'L3 context phase artifact installed: %s bytes=%s\n' "$DEST" "$(stat -c %s "$DEST")"
