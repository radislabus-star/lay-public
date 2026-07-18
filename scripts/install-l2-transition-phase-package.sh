#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE="${1:-}"
EVAL_BIN="${LAY_NANDA_WAVE_EVAL_BIN:-$ROOT/target/release/lay-nanda-wave-eval}"
DESTINATION="${LAY_NANDA_L2_PHASE_MEMORY:-$HOME/.local/share/lay/nanda_wave/l2_candidate_phase.nwpc}"

if [[ -z "$PACKAGE" || ! -f "$PACKAGE" ]]; then
  echo "usage: $0 PACKAGE.nwpc" >&2
  exit 2
fi
if [[ ! -x "$EVAL_BIN" ]]; then
  echo "L2 transition phase evaluator missing: $EVAL_BIN" >&2
  exit 1
fi

destination_dir="$(dirname "$DESTINATION")"
mkdir -p "$destination_dir"
temporary="$(mktemp "$destination_dir/.l2_candidate_phase.nwpc.XXXXXX")"
trap 'rm -f "$temporary"' EXIT
install -m 0600 "$PACKAGE" "$temporary"

# Validate the exact staged bytes before replacing the live package. A stale
# phase format must leave the currently working runtime memory untouched.
status="$($EVAL_BIN --l2-transition-phase-status --l2-phase-memory "$temporary")"
python3 -c '
import json, sys
report = json.load(sys.stdin)
if not report.get("loaded"):
    raise SystemExit("L2 transition phase package did not load")
if report.get("raw_words_stored") is not False:
    raise SystemExit("L2 transition phase package stores raw words")
if int(report.get("promoted_profiles", 0)) < 1:
    raise SystemExit("L2 transition phase package has no promoted profiles")
if int(report.get("lexical_anti_centers", 0)) < 1:
    raise SystemExit("L2 transition phase package has no lexical anti-centers")
' <<<"$status"

mv -f "$temporary" "$DESTINATION"
trap - EXIT

python3 -c '
import json, sys
report = json.load(sys.stdin)
print(
    "Installed L2 transition phase package: "
    "profiles={profile_count} promoted={promoted_profiles} "
    "anti_centers={anti_centers} "
    "lexical_anti_centers={lexical_anti_centers} bytes={hot_bytes}".format(**report)
)
' <<<"$status"
