#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE="${1:?usage: install-l11-shadow-package.sh PACKAGE [MANIFEST]}"
MANIFEST="${2:-$ROOT/data/lexical_grokking/lay_l11_ru_composite_en300k_shadow_v2.manifest.json}"
INSTALL_DIR="${LAY_L11_MODEL_DIR:-$HOME/.local/share/lay/nanda_wave/l1.1}"

readarray -t metadata < <(
  python3 - "$MANIFEST" <<'PY'
import json
import sys

manifest = json.load(open(sys.argv[1], encoding="utf-8"))
if manifest.get("runtime_authority") is not False:
    raise SystemExit("L1.1 release installer accepts only shadow packages")
if manifest.get("proof_verdict") != "PASS_shadow":
    raise SystemExit("L1.1 package has no PASS_shadow proof")
split = manifest.get("split_language_proof", {})
if split.get("ru_verdict") != "PASS_shadow":
    raise SystemExit("L1.1 package has no RU split-language PASS_shadow proof")
if split.get("en_verdict") != "PASS_shadow":
    raise SystemExit("L1.1 package has no EN split-language PASS_shadow proof")
if manifest.get("release_shadow_installable") is not True:
    raise SystemExit("L1.1 package is not admitted for release-shadow installation")
print(manifest["package_id"])
print(manifest["crystal_bytes"])
print(manifest["crystal_sha256"])
print(manifest["crystal_format"])
PY
)

PACKAGE_ID="${metadata[0]}"
EXPECTED_BYTES="${metadata[1]}"
EXPECTED_SHA256="${metadata[2]}"
ACTUAL_BYTES="$(stat -c '%s' "$PACKAGE")"
ACTUAL_SHA256="$(sha256sum "$PACKAGE" | awk '{print $1}')"
CRYSTAL_FORMAT="${metadata[3]}"

case "$CRYSTAL_FORMAT" in
  V8*) PACKAGE_EXTENSION="v8.bin" ;;
  V7*) PACKAGE_EXTENSION="v7.bin" ;;
  *)
    echo "unsupported L1.1 crystal format: $CRYSTAL_FORMAT" >&2
    exit 1
    ;;
esac

if [[ "$ACTUAL_BYTES" != "$EXPECTED_BYTES" ]]; then
  echo "L1.1 package size mismatch: expected=$EXPECTED_BYTES actual=$ACTUAL_BYTES" >&2
  exit 1
fi
if [[ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]]; then
  echo "L1.1 package SHA-256 mismatch: expected=$EXPECTED_SHA256 actual=$ACTUAL_SHA256" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
DESTINATION="$INSTALL_DIR/$PACKAGE_ID.$PACKAGE_EXTENSION"
RECEIPT="$INSTALL_DIR/$PACKAGE_ID.installed.json"
TEMPORARY="$INSTALL_DIR/.$PACKAGE_ID.$PACKAGE_EXTENSION.tmp.$$"

install -m 0644 "$PACKAGE" "$TEMPORARY"
mv -f "$TEMPORARY" "$DESTINATION"
python3 - "$MANIFEST" "$DESTINATION" "$RECEIPT" "$ACTUAL_BYTES" "$ACTUAL_SHA256" <<'PY'
import json
import os
import sys

manifest_path, package_path, receipt_path, size, digest = sys.argv[1:]
manifest = json.load(open(manifest_path, encoding="utf-8"))
receipt = {
    "package_id": manifest["package_id"],
    "format": manifest["crystal_format"],
    "installed_artifact": os.path.abspath(package_path),
    "artifact_bytes": int(size),
    "artifact_sha256": digest,
    "proof_receipt": manifest["proof_receipt"],
    "proof_verdict": manifest["proof_verdict"],
    "runtime_authority": False,
}
temporary = receipt_path + ".tmp"
with open(temporary, "w", encoding="utf-8") as output:
    json.dump(receipt, output, ensure_ascii=False, indent=2)
    output.write("\n")
os.replace(temporary, receipt_path)
PY

printf 'Installed shadow-only L1.1 package: %s\n' "$DESTINATION"
printf 'Installation receipt: %s\n' "$RECEIPT"
