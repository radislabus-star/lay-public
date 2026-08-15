#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE="${1:?usage: install-l11-shadow-package.sh PACKAGE [MANIFEST]}"
MANIFEST="${2:-$ROOT/data/lexical_grokking/lay_l11_ru_composite_en300k_shadow_v4.manifest.json}"
INSTALL_DIR="${LAY_L11_MODEL_DIR:-$HOME/.local/share/lay/nanda_wave/l1.1}"

readarray -t metadata < <(
  python3 - "$MANIFEST" "$ROOT" <<'PY'
import json
import os
import re
import sys

manifest = json.load(open(sys.argv[1], encoding="utf-8"))
if manifest.get("runtime_authority") is not False:
    raise SystemExit("L1.1 release installer accepts only shadow packages")
crystal_format = manifest.get("crystal_format", "")
proof_verdict = manifest.get("proof_verdict", "")
admitted_proof_verdicts = {"PASS_shadow", "PASS_C_QUALITY"}
if proof_verdict not in admitted_proof_verdicts:
    raise SystemExit("L1.1 package has no PASS proof")
if crystal_format.startswith("V9") and proof_verdict != "PASS_C_QUALITY":
    raise SystemExit("L1.1 V9 packages require a full PASS_C_QUALITY proof")
split = manifest.get("split_language_proof", {})
if split:
    if split.get("ru_verdict", "") not in admitted_proof_verdicts:
        raise SystemExit("L1.1 package has no RU split-language PASS proof")
    if split.get("en_verdict", "") not in admitted_proof_verdicts:
        raise SystemExit("L1.1 package has no EN split-language PASS proof")
if manifest.get("release_shadow_installable") is not True:
    raise SystemExit("L1.1 package is not admitted for release-shadow installation")
package_id = manifest.get("package_id", "")
if not re.fullmatch(r"[A-Za-z0-9._+-]+", package_id):
    raise SystemExit("L1.1 package_id is empty or unsafe for an installed filename")

expected_bytes = int(manifest["crystal_bytes"])
expected_sha256 = manifest["crystal_sha256"].lower()

def resolve(path):
    if not os.path.isabs(path):
        path = os.path.join(sys.argv[2], path)
    return os.path.abspath(path)

proof_path = manifest.get("proof_receipt", "")
proof_path = resolve(proof_path)
proof = json.load(open(proof_path, encoding="utf-8"))
if not isinstance(proof, dict):
    raise SystemExit("L1.1 proof receipt must be a JSON object")

if proof_verdict == "PASS_C_QUALITY":
    artifact = proof.get("artifact", {})
    configuration = proof.get("configuration", {})
    gates = proof.get("gates", {})
    claim = proof.get("claim_boundary", {})
    if proof.get("schema") != "lay.l11.typed-basin-quality-proof.v3":
        raise SystemExit("L1.1 quality proof schema is not admitted")
    if proof.get("verdict") != proof_verdict:
        raise SystemExit("L1.1 quality proof verdict does not match the manifest")
    if artifact.get("format") != "V9":
        raise SystemExit("L1.1 quality proof is not bound to a direct V9 artifact")
    if artifact.get("package_bytes") != expected_bytes:
        raise SystemExit("L1.1 quality proof package size does not match the manifest")
    if artifact.get("package_sha256_before", "").lower() != expected_sha256:
        raise SystemExit("L1.1 quality proof package SHA-256 does not match the manifest")
    if artifact.get("package_sha256_after", "").lower() != expected_sha256:
        raise SystemExit("L1.1 quality proof did not preserve package bytes")
    if artifact.get("package_bytes_unchanged") is not True:
        raise SystemExit("L1.1 quality proof package isolation did not pass")
    if configuration.get("lattice_projection_limit") != 32:
        raise SystemExit("L1.1 quality proof used a non-live lattice limit")
    expected_configuration = {
        "heldout_per_class": 20_000,
        "fixed_damage_classes": 13,
        "selected_damage_classes": 13,
        "expected_damaged_cases": 260_000,
        "clean_limit": 0,
    }
    for field, expected in expected_configuration.items():
        if configuration.get(field) != expected:
            raise SystemExit(f"L1.1 quality proof has an incomplete {field} denominator")
    if configuration.get("damage_class_filter") is not None:
        raise SystemExit("L1.1 quality proof is a filtered diagnostic")
    required_gates = (
        "artifact_prerequisite_pass",
        "direct_v9_artifact",
        "v9_checksum_valid",
        "stored_exact_support_matches_rebuild",
        "package_dependencies_resolved",
        "package_isolation",
        "fixed_damaged_denominator_complete",
        "clean_denominator_complete",
        "full_fixed_denominator",
        "target_retention_complete",
        "unique_top1_every_class_strictly_gt_95_percent",
        "lattice_coverage_every_class_ge_99_percent",
        "clean_preservation_ge_99_9_percent",
        "false_authority_zero",
        "false_singleton_zero",
        "grounded_legacy_candidate_loss_zero",
        "conjunctive_full_quality_pass",
    )
    if any(gates.get(field) is not True for field in required_gates):
        raise SystemExit("L1.1 quality proof did not pass every conjunctive gate")
    if claim.get("full_quality_matrix_tested") is not True or claim.get("full_quality_claimed") is not True:
        raise SystemExit("L1.1 quality proof is not a full-matrix claim")

    # L11_FIXED_DAMAGE_CLASSES_BEGIN
    fixed_classes = (
        "missing_letter",
        "extra_letter",
        "adjacent_transposition",
        "letter_substitution",
        "sparse_multi_omission",
        "non_adjacent_transposition",
        "double_substitution",
        "omission_transposition",
        "repeated_fragment",
        "prefix_truncation",
        "suffix_truncation",
        "layout_projection",
        "punctuation_suffix",
    )
    # L11_FIXED_DAMAGE_CLASSES_END
    primary_centers = artifact.get("primary_centers")
    damaged = proof.get("damaged_quality", {})
    classes = damaged.get("classes", {})
    aggregate = damaged.get("aggregate", {})
    clean = proof.get("clean_quality", {})
    if not isinstance(primary_centers, int) or isinstance(primary_centers, bool) or primary_centers <= 0:
        raise SystemExit("L1.1 quality proof has no primary-center denominator")
    if not isinstance(classes, dict) or set(classes) != set(fixed_classes):
        raise SystemExit("L1.1 quality proof does not contain the exact fixed 13 classes")

    def integer(value):
        return isinstance(value, int) and not isinstance(value, bool) and value >= 0

    def ratio_at_least(numerator, denominator, required_numerator, required_denominator):
        return (
            integer(numerator)
            and integer(denominator)
            and 0 < denominator
            and numerator <= denominator
            and numerator * required_denominator >= denominator * required_numerator
        )

    def ratio_strictly_above(numerator, denominator, required_numerator, required_denominator):
        return (
            integer(numerator)
            and integer(denominator)
            and 0 < denominator
            and numerator <= denominator
            and numerator * required_denominator > denominator * required_numerator
        )

    for name in fixed_classes:
        report = classes[name]
        if not isinstance(report, dict):
            raise SystemExit(f"L1.1 quality class {name} is not an object")
        cases = report.get("cases")
        objective_cases = report.get("objective_unique_cases")
        class_gates = report.get("gates", {})
        if (
            cases != 20_000
            or report.get("target_retained_complete_field") != cases
            or not integer(objective_cases)
            or objective_cases > cases
            or not ratio_at_least(report.get("target_in_bounded_lattice"), cases, 99, 100)
            or not ratio_strictly_above(report.get("unique_top1"), objective_cases, 95, 100)
            or not integer(report.get("false_authority"))
            or report.get("false_authority") != 0
            or not integer(report.get("false_singleton"))
            or report.get("false_singleton") != 0
            or class_gates.get("unique_top1_strictly_gt_95_percent") is not True
            or class_gates.get("lattice_coverage_ge_99_percent") is not True
        ):
            raise SystemExit(f"L1.1 quality class {name} failed an independently checked gate")

    observer = aggregate.get("runtime_observer", {})
    if (
        aggregate.get("cases") != 260_000
        or aggregate.get("target_retained_complete_field") != 260_000
        or not integer(aggregate.get("false_authority"))
        or aggregate.get("false_authority") != 0
        or not integer(aggregate.get("false_singleton"))
        or aggregate.get("false_singleton") != 0
        or not integer(observer.get("grounded_candidate_losses_from_exact_field"))
        or observer.get("grounded_candidate_losses_from_exact_field") != 0
    ):
        raise SystemExit("L1.1 quality proof aggregate counters are not admitted")
    if (
        clean.get("cases") != primary_centers
        or not ratio_at_least(clean.get("preserved"), primary_centers, 999, 1_000)
    ):
        raise SystemExit("L1.1 quality proof clean denominator or preservation is not admitted")
elif proof_verdict == "PASS_shadow":
    fixed_proof = proof.get("fixed_proof", {})
    package = proof.get("package", {})
    if fixed_proof.get("verdict") != proof_verdict:
        raise SystemExit("L1.1 shadow proof verdict does not match the manifest")
    if package.get("bytes") != expected_bytes:
        raise SystemExit("L1.1 shadow proof package size does not match the manifest")
    if package.get("sha256", "").lower() != expected_sha256:
        raise SystemExit("L1.1 shadow proof package SHA-256 does not match the manifest")

for lane in ("ru", "en"):
    if not split:
        break
    lane_path = resolve(split.get(f"{lane}_receipt", ""))
    lane_proof = json.load(open(lane_path, encoding="utf-8"))
    lane_verdict = split[f"{lane}_verdict"]
    declared = lane_proof.get("status", lane_proof.get("verdict"))
    if declared != lane_verdict:
        raise SystemExit(f"L1.1 {lane.upper()} split proof verdict does not match the manifest")

print(package_id)
print(expected_bytes)
print(expected_sha256)
print(crystal_format)
print(proof_path)
print(proof_verdict)
PY
)

PACKAGE_ID="${metadata[0]}"
EXPECTED_BYTES="${metadata[1]}"
EXPECTED_SHA256="${metadata[2]}"
ACTUAL_BYTES="$(stat -c '%s' "$PACKAGE")"
ACTUAL_SHA256="$(sha256sum "$PACKAGE" | awk '{print $1}')"
CRYSTAL_FORMAT="${metadata[3]}"
PROOF_RECEIPT="${metadata[4]}"
PROOF_VERDICT="${metadata[5]}"
PROOF_SHA256="$(sha256sum "$PROOF_RECEIPT" | awk '{print $1}')"

case "$CRYSTAL_FORMAT" in
  V9*) PACKAGE_EXTENSION="v9.bin" ;;
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
exec 9>"$INSTALL_DIR/.install.lock"
flock -x 9
DESTINATION="$INSTALL_DIR/$PACKAGE_ID.$PACKAGE_EXTENSION"
RECEIPT="$INSTALL_DIR/$PACKAGE_ID.installed.json"
ACTIVE_RECEIPT="$INSTALL_DIR/active.installed.json"
TEMPORARY="$INSTALL_DIR/.$PACKAGE_ID.$PACKAGE_EXTENSION.tmp.$$"
PROOF_DESTINATION="$INSTALL_DIR/$PACKAGE_ID.$PROOF_SHA256.proof.json"
PROOF_TEMPORARY="$INSTALL_DIR/.$PACKAGE_ID.proof.tmp.$$"
trap 'rm -f "$TEMPORARY" "$PROOF_TEMPORARY"' EXIT

if [[ -e "$DESTINATION" ]]; then
  INSTALLED_BYTES="$(stat -c '%s' "$DESTINATION")"
  INSTALLED_SHA256="$(sha256sum "$DESTINATION" | awk '{print $1}')"
  if [[ "$INSTALLED_BYTES" != "$ACTUAL_BYTES" || "$INSTALLED_SHA256" != "$ACTUAL_SHA256" ]]; then
    echo "refusing to replace immutable L1.1 package_id with different bytes: $PACKAGE_ID" >&2
    exit 1
  fi
else
  install -m 0644 "$PACKAGE" "$TEMPORARY"
  sync -f "$TEMPORARY"
  mv "$TEMPORARY" "$DESTINATION"
fi
if [[ -e "$PROOF_DESTINATION" ]]; then
  INSTALLED_PROOF_SHA256="$(sha256sum "$PROOF_DESTINATION" | awk '{print $1}')"
  if [[ "$INSTALLED_PROOF_SHA256" != "$PROOF_SHA256" ]]; then
    echo "refusing corrupt immutable L1.1 proof receipt: $PROOF_DESTINATION" >&2
    exit 1
  fi
else
  install -m 0644 "$PROOF_RECEIPT" "$PROOF_TEMPORARY"
  sync -f "$PROOF_TEMPORARY"
  mv "$PROOF_TEMPORARY" "$PROOF_DESTINATION"
fi
INSTALLED_BYTES="$(stat -c '%s' "$DESTINATION")"
INSTALLED_SHA256="$(sha256sum "$DESTINATION" | awk '{print $1}')"
INSTALLED_PROOF_SHA256="$(sha256sum "$PROOF_DESTINATION" | awk '{print $1}')"
if [[ "$INSTALLED_BYTES" != "$EXPECTED_BYTES" || "$INSTALLED_SHA256" != "$EXPECTED_SHA256" ]]; then
  echo "installed L1.1 package changed between validation and receipt publication" >&2
  exit 1
fi
if [[ "$INSTALLED_PROOF_SHA256" != "$PROOF_SHA256" ]]; then
  echo "installed L1.1 proof changed between validation and receipt publication" >&2
  exit 1
fi
python3 - "$PACKAGE_ID" "$CRYSTAL_FORMAT" "$DESTINATION" "$RECEIPT" "$ACTIVE_RECEIPT" "$INSTALLED_BYTES" "$INSTALLED_SHA256" "$PROOF_DESTINATION" "$PROOF_RECEIPT" "$INSTALLED_PROOF_SHA256" "$PROOF_VERDICT" <<'PY'
import json
import os
import sys

package_id, crystal_format, package_path, receipt_path, active_path, size, digest, proof_path, proof_source, proof_digest, proof_verdict = sys.argv[1:]
receipt = {
    "schema": "lay.l11.installed-package.v1",
    "package_id": package_id,
    "format": crystal_format,
    "installed_artifact": os.path.abspath(package_path),
    "artifact_bytes": int(size),
    "artifact_sha256": digest,
    "proof_receipt": os.path.abspath(proof_path),
    "proof_source": os.path.abspath(proof_source),
    "proof_sha256": proof_digest,
    "proof_verdict": proof_verdict,
    "runtime_authority": False,
    "runtime_admitted": True,
}
for destination in (receipt_path, active_path):
    temporary = destination + ".tmp"
    with open(temporary, "w", encoding="utf-8") as output:
        json.dump(receipt, output, ensure_ascii=False, indent=2)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    os.replace(temporary, destination)
directory_fd = os.open(os.path.dirname(active_path), os.O_RDONLY | os.O_DIRECTORY)
try:
    os.fsync(directory_fd)
finally:
    os.close(directory_fd)
PY
trap - EXIT

printf 'Installed shadow-only L1.1 package: %s\n' "$DESTINATION"
printf 'Installation receipt: %s\n' "$RECEIPT"
printf 'Active admission receipt: %s\n' "$ACTIVE_RECEIPT"
