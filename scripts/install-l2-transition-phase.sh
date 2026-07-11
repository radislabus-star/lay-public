#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

EVAL_BIN="${LAY_NANDA_WAVE_EVAL_BIN:-$ROOT/target/release/lay-nanda-wave-eval}"
TRAIN_BIN="${LAY_NANDA_WAVE_TRAIN_BIN:-$ROOT/target/release/lay-nanda-wave-train}"
DATASET="${LAY_NANDA_PHASE_DATASET:-$ROOT/data/nanda_training/generated_cases.tsv}"

for file in "$EVAL_BIN" "$TRAIN_BIN" "$DATASET"; do
  if [[ ! -e "$file" ]]; then
    echo "L2 transition phase prerequisite missing: $file" >&2
    exit 1
  fi
done

proof="$($EVAL_BIN --l2-transition-phase-proof --dataset "$DATASET")"
python3 -c '
import json, sys
report = json.load(sys.stdin)
verdict = report.get("verdict")
if verdict != "PASS":
    raise SystemExit(f"L2 transition phase proof is {verdict}")
if report.get("full_phase_false_accepts") != 0:
    raise SystemExit("L2 transition phase proof has false accepts")
if not report.get("promoted_operators"):
    raise SystemExit("L2 transition phase proof promoted no operators")
' <<<"$proof"

"$TRAIN_BIN" --phase-only --pack-live --dataset "$DATASET"
status="$($EVAL_BIN --l2-transition-phase-status)"
python3 -c '
import json, sys
report = json.load(sys.stdin)
if not report.get("loaded"):
    raise SystemExit("L2 transition phase package did not load")
if report.get("raw_words_stored") is not False:
    raise SystemExit("L2 transition phase package stores raw words")
if report.get("promoted_profiles", 0) < 1:
    raise SystemExit("L2 transition phase package has no promoted profiles")
profiles = report.get("profile_count", 0)
promoted = report.get("promoted_profiles", 0)
hot_bytes = report.get("hot_bytes", 0)
print(
    "L2 transition phase package OK: "
    f"profiles={profiles} promoted={promoted} bytes={hot_bytes}"
)
' <<<"$status"
