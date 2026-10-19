#!/usr/bin/env bash
# Build a GNOME Shell extension ZIP for local install or extensions.gnome.org upload.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXT_DIR="$ROOT/extension/lay@radislabus-star.github.io"
OUT_DIR="$ROOT/dist/gnome-extension"

if ! command -v zip >/dev/null; then
    echo "zip is required. Install it with: sudo apt-get install zip" >&2
    exit 1
fi
if ! command -v unzip >/dev/null; then
    echo "unzip is required. Install it with: sudo apt-get install unzip" >&2
    exit 1
fi

read -r UUID VERSION_NAME < <(
    python3 - "$EXT_DIR/metadata.json" <<'PY'
import json
import sys
from pathlib import Path

metadata = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(metadata["uuid"], metadata["version-name"])
PY
)

mkdir -p "$OUT_DIR"
ZIP_PATH="$OUT_DIR/${UUID}-${VERSION_NAME}.zip"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
rm -f "$ZIP_PATH"

cp "$EXT_DIR"/*.js "$TMP_DIR/"
python3 - "$EXT_DIR/metadata.json" "$TMP_DIR/metadata.json" <<'PY'
import json
import sys
from pathlib import Path

metadata = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
# extensions.gnome.org owns the numeric submission version. Keep version-name
# visible in the UI, but don't ship the internal numeric field in the upload ZIP.
metadata.pop("version", None)
Path(sys.argv[2]).write_text(
    json.dumps(metadata, ensure_ascii=False, indent=4) + "\n",
    encoding="utf-8",
)
PY

(
    cd "$TMP_DIR"
    zip -X -q "$ZIP_PATH" metadata.json ./*.js
)

echo "Built: $ZIP_PATH"
echo ""
echo "Archive contents:"
unzip -l "$ZIP_PATH"

echo ""
echo "Local extension-only install:"
echo "  gnome-extensions install --force \"$ZIP_PATH\""
echo "  gnome-extensions enable \"$UUID\""
