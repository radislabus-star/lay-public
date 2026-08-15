#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PACKAGE="$TMP/package.v9.bin"
PROOF="$TMP/proof.json"
MANIFEST="$TMP/manifest.json"
MODEL_DIR="$TMP/model"
PACKAGE_ID="LAY-L11-INSTALL-FIXTURE"
printf 'deterministic-v9-fixture\n' >"$PACKAGE"
PACKAGE_BYTES="$(stat -c '%s' "$PACKAGE")"
PACKAGE_SHA256="$(sha256sum "$PACKAGE" | awk '{print $1}')"

write_proof() {
  local verdict="$1"
  local lattice_limit="${2:-32}"
  python3 - "$PROOF" "$verdict" "$PACKAGE_BYTES" "$PACKAGE_SHA256" "$lattice_limit" <<'PY'
import json
import sys

path, verdict, package_bytes, package_sha256, lattice_limit = sys.argv[1:]
proof = {
    "schema": "lay.l11.typed-basin-quality-proof.v3",
    "verdict": verdict,
    "artifact": {
        "format": "V9",
        "package_bytes": int(package_bytes),
        "package_sha256_before": package_sha256,
        "package_sha256_after": package_sha256,
        "package_bytes_unchanged": True,
        "primary_centers": 1_000,
    },
    "configuration": {
        "heldout_per_class": 20_000,
        "fixed_damage_classes": 13,
        "selected_damage_classes": 13,
        "damage_class_filter": None,
        "expected_damaged_cases": 260_000,
        "clean_limit": 0,
        "lattice_projection_limit": int(lattice_limit),
    },
    "gates": {
        "artifact_prerequisite_pass": True,
        "direct_v9_artifact": True,
        "v9_checksum_valid": True,
        "stored_exact_support_matches_rebuild": True,
        "package_dependencies_resolved": True,
        "package_isolation": True,
        "fixed_damaged_denominator_complete": True,
        "clean_denominator_complete": True,
        "full_fixed_denominator": True,
        "target_retention_complete": True,
        "unique_top1_every_class_strictly_gt_95_percent": True,
        "lattice_coverage_every_class_ge_99_percent": True,
        "clean_preservation_ge_99_9_percent": True,
        "false_authority_zero": True,
        "false_singleton_zero": True,
        "grounded_legacy_candidate_loss_zero": True,
        "conjunctive_full_quality_pass": True,
    },
    "damaged_quality": {
        "classes": {
            name: {
                "cases": 20_000,
                "objective_unique_cases": 20_000,
                "target_retained_complete_field": 20_000,
                "target_in_bounded_lattice": 19_800,
                "unique_top1": 19_200,
                "false_authority": 0,
                "false_singleton": 0,
                "gates": {
                    "unique_top1_strictly_gt_95_percent": True,
                    "lattice_coverage_ge_99_percent": True,
                },
            }
            for name in (
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
        },
        "aggregate": {
            "cases": 260_000,
            "target_retained_complete_field": 260_000,
            "false_authority": 0,
            "false_singleton": 0,
            "runtime_observer": {
                "grounded_candidate_losses_from_exact_field": 0,
            },
        },
    },
    "clean_quality": {
        "cases": 1_000,
        "preserved": 999,
    },
    "claim_boundary": {
        "full_quality_matrix_tested": True,
        "full_quality_claimed": True,
    },
}
with open(path, "w", encoding="utf-8") as output:
    json.dump(proof, output, sort_keys=True)
    output.write("\n")
PY
}

write_manifest() {
  local verdict="$1"
  python3 - "$MANIFEST" "$PACKAGE_ID" "$PACKAGE_BYTES" "$PACKAGE_SHA256" "$PROOF" "$verdict" <<'PY'
import json
import sys

path, package_id, package_bytes, package_sha256, proof, verdict = sys.argv[1:]
manifest = {
    "package_id": package_id,
    "crystal_bytes": int(package_bytes),
    "crystal_sha256": package_sha256,
    "crystal_format": "V9 fixture",
    "proof_receipt": proof,
    "proof_verdict": verdict,
    "runtime_authority": False,
    "release_shadow_installable": True,
}
with open(path, "w", encoding="utf-8") as output:
    json.dump(manifest, output, sort_keys=True)
    output.write("\n")
PY
}

write_proof "PASS_C_QUALITY"
write_manifest "PASS_C_QUALITY"
LAY_L11_MODEL_DIR="$MODEL_DIR" \
  "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null

DESTINATION="$MODEL_DIR/$PACKAGE_ID.v9.bin"
ACTIVE_RECEIPT="$MODEL_DIR/active.installed.json"
cmp -s "$PACKAGE" "$DESTINATION"
FIRST_INODE="$(stat -c '%i' "$DESTINATION")"
LAY_L11_MODEL_DIR="$MODEL_DIR" \
  "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null
test "$(stat -c '%i' "$DESTINATION")" = "$FIRST_INODE"

python3 - "$ACTIVE_RECEIPT" "$DESTINATION" "$PACKAGE_BYTES" "$PACKAGE_SHA256" <<'PY'
import hashlib
import json
import os
import sys

receipt_path, package_path, package_bytes, package_sha256 = sys.argv[1:]
receipt = json.load(open(receipt_path, encoding="utf-8"))
assert receipt["schema"] == "lay.l11.installed-package.v1"
assert receipt["installed_artifact"] == os.path.abspath(package_path)
assert receipt["artifact_bytes"] == int(package_bytes)
assert receipt["artifact_sha256"] == package_sha256
assert receipt["proof_verdict"] == "PASS_C_QUALITY"
assert receipt["runtime_authority"] is False
assert receipt["runtime_admitted"] is True
proof_path = receipt["proof_receipt"]
assert os.path.isabs(proof_path) and os.path.isfile(proof_path)
with open(proof_path, "rb") as proof:
    assert hashlib.sha256(proof.read()).hexdigest() == receipt["proof_sha256"]
PY

write_proof "PASS_C_SMOKE"
write_manifest "PASS_C_SMOKE"
if LAY_L11_MODEL_DIR="$TMP/rejected-verdict" \
    "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null 2>&1; then
  echo "non-final L1.1 proof verdict was accepted" >&2
  exit 1
fi

write_proof "PASS_C_QUALITY" 64
write_manifest "PASS_C_QUALITY"
if LAY_L11_MODEL_DIR="$TMP/rejected-limit" \
    "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null 2>&1; then
  echo "non-live L1.1 lattice proof was accepted" >&2
  exit 1
fi

reject_quality_mutation() {
  local label="$1"
  local pointer="$2"
  local value="$3"
  write_proof "PASS_C_QUALITY"
  python3 - "$PROOF" "$pointer" "$value" <<'PY'
import json
import sys

path, pointer, encoded = sys.argv[1:]
proof = json.load(open(path, encoding="utf-8"))
parts = pointer.strip("/").split("/")
owner = proof
for part in parts[:-1]:
    owner = owner[part]
owner[parts[-1]] = json.loads(encoded)
with open(path, "w", encoding="utf-8") as output:
    json.dump(proof, output, sort_keys=True)
    output.write("\n")
PY
  write_manifest "PASS_C_QUALITY"
  if LAY_L11_MODEL_DIR="$TMP/rejected-$label" \
      "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null 2>&1; then
    echo "invalid L1.1 quality proof mutation was accepted: $label" >&2
    exit 1
  fi
}

reject_quality_mutation "class-top1" "/damaged_quality/classes/missing_letter/unique_top1" "19000"
reject_quality_mutation "class-lattice" "/damaged_quality/classes/missing_letter/target_in_bounded_lattice" "19799"
reject_quality_mutation "aggregate-authority" "/damaged_quality/aggregate/false_authority" "1"
reject_quality_mutation "clean-preservation" "/clean_quality/preserved" "998"
reject_quality_mutation "artifact-format" "/artifact/format" '"V8"'
reject_quality_mutation "artifact-prerequisite" "/gates/artifact_prerequisite_pass" "false"
reject_quality_mutation "artifact-support" "/gates/stored_exact_support_matches_rebuild" "false"

python3 - "$PROOF" "$PACKAGE_BYTES" "$PACKAGE_SHA256" <<'PY'
import json
import sys

path, package_bytes, package_sha256 = sys.argv[1:]
with open(path, "w", encoding="utf-8") as output:
    json.dump({
        "fixed_proof": {"verdict": "PASS_shadow"},
        "package": {"bytes": int(package_bytes), "sha256": package_sha256},
    }, output, sort_keys=True)
    output.write("\n")
PY
write_manifest "PASS_shadow"
if LAY_L11_MODEL_DIR="$TMP/rejected-v9-shadow" \
    "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null 2>&1; then
  echo "V9 package was accepted with a legacy shadow proof" >&2
  exit 1
fi

printf 'different-package-bytes\n' >"$PACKAGE"
PACKAGE_BYTES="$(stat -c '%s' "$PACKAGE")"
PACKAGE_SHA256="$(sha256sum "$PACKAGE" | awk '{print $1}')"
write_proof "PASS_C_QUALITY"
write_manifest "PASS_C_QUALITY"
if LAY_L11_MODEL_DIR="$MODEL_DIR" \
    "$ROOT/scripts/install-l11-shadow-package.sh" "$PACKAGE" "$MANIFEST" >/dev/null 2>&1; then
  echo "immutable L1.1 package_id was replaced with different bytes" >&2
  exit 1
fi

echo "L1.1 package installer integrity regressions: PASS"
