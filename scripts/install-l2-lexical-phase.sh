#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="${LAY_L2_LEXICAL_PHASE_SOURCE:-$ROOT/data/lexicon/l2_lexical_phase_v2.bin}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
DEST="${LAY_L2_LEXICAL_PHASE_MEMORY:-$DATA_HOME/lay/nanda_wave/l2_lexical_phase_v2.bin}"
SOURCE_MANIFEST="${SOURCE%.bin}.manifest.json"
DEST_MANIFEST="${DEST%.bin}.manifest.json"

if [[ ! -s "$SOURCE" ]]; then
  echo "L2 lexical phase artifact missing: $SOURCE" >&2
  exit 1
fi
if [[ ! -s "$SOURCE_MANIFEST" ]]; then
  echo "L2 lexical phase manifest missing: $SOURCE_MANIFEST" >&2
  exit 1
fi

mkdir -p "$(dirname "$DEST")"
tmp="${DEST}.tmp.$$"
manifest_tmp="${DEST_MANIFEST}.tmp.$$"
trap 'rm -f "$tmp" "$manifest_tmp"' EXIT
install -m 0644 "$SOURCE" "$tmp"
install -m 0644 "$SOURCE_MANIFEST" "$manifest_tmp"
mv -f "$tmp" "$DEST"
mv -f "$manifest_tmp" "$DEST_MANIFEST"
trap - EXIT

printf 'L2 lexical phase artifact installed: %s bytes=%s\n' "$DEST" "$(stat -c %s "$DEST")"
