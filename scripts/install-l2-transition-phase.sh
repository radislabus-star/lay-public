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
lexical = report.get("lexical_competition", {}).get("full_lexical_phase", {})
if int(lexical.get("negative_cases", 0)) < 1:
    raise SystemExit("L2 lexical phase proof has no heldout negative cases")
if int(lexical.get("negative_support_false_accepts", -1)) != 0:
    raise SystemExit("L2 lexical phase proof has false supports")
if int(report.get("lexical_anti_center_false_support_prevention", 0)) < 1:
    raise SystemExit("L2 lexical anti-center has no causal ablation value")
lexical_pairs = report.get("lexical_pair_competition", {}).get("full_lexical_phase", {})
if int(lexical_pairs.get("cases", 0)) < 1:
    raise SystemExit("L2 lexical phase proof has no paired candidate heldout")
if int(lexical_pairs.get("wrong_top1", -1)) != 0:
    raise SystemExit("L2 lexical phase proof selects a wrong heldout center")
if int(report.get("lexical_anti_center_top1_gain", 0)) < 1:
    raise SystemExit("L2 lexical anti-center has no paired top-1 ablation value")
if int(report.get("lexical_negative_rows_deferred_to_l2_word_center", -1)) != 0:
    raise SystemExit("L2 lexical negatives remain outside candidate anti-centers")
if not report.get("promoted_operators"):
    raise SystemExit("L2 transition phase proof promoted no operators")
' <<<"$proof"

"$TRAIN_BIN" --phase-only --dataset "$DATASET"
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
if report.get("lexical_anti_centers", 0) < 1:
    raise SystemExit("L2 transition phase package has no lexical anti-centers")
profiles = report.get("profile_count", 0)
promoted = report.get("promoted_profiles", 0)
hot_bytes = report.get("hot_bytes", 0)
lexical_anti = report.get("lexical_anti_centers", 0)
print(
    "L2 transition phase package OK: "
    f"profiles={profiles} promoted={promoted} "
    f"lexical_anti_centers={lexical_anti} bytes={hot_bytes}"
)
' <<<"$status"
