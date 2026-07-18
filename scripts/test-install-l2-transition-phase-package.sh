#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PACKAGE="$TMP/package.nwpc"
DESTINATION="$TMP/runtime/l2_candidate_phase.nwpc"
EVALUATOR="$TMP/lay-nanda-wave-eval"
printf 'new-phase-package' >"$PACKAGE"

printf '%s\n' \
  '#!/usr/bin/env bash' \
  'printf '\''%s\n'\'' '\''{"loaded":true,"raw_words_stored":false,"profile_count":2,"promoted_profiles":1,"anti_centers":3,"lexical_anti_centers":2,"hot_bytes":17}'\''' \
  >"$EVALUATOR"
chmod +x "$EVALUATOR"

LAY_NANDA_WAVE_EVAL_BIN="$EVALUATOR" \
LAY_NANDA_L2_PHASE_MEMORY="$DESTINATION" \
  "$ROOT/scripts/install-l2-transition-phase-package.sh" "$PACKAGE" >/dev/null
cmp -s "$PACKAGE" "$DESTINATION"

# The failure path is the important release invariant: validation may fail,
# but it may never truncate or replace the package used by running processes.
printf 'preserved-package' >"$DESTINATION"
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'printf '\''%s\n'\'' '\''{"loaded":false,"raw_words_stored":false,"profile_count":0,"promoted_profiles":0,"anti_centers":0,"lexical_anti_centers":0,"hot_bytes":0}'\''' \
  >"$EVALUATOR"
chmod +x "$EVALUATOR"

if LAY_NANDA_WAVE_EVAL_BIN="$EVALUATOR" \
  LAY_NANDA_L2_PHASE_MEMORY="$DESTINATION" \
  "$ROOT/scripts/install-l2-transition-phase-package.sh" "$PACKAGE" >/dev/null 2>&1; then
  echo "invalid L2 phase package was accepted" >&2
  exit 1
fi
grep -qx 'preserved-package' "$DESTINATION"

echo "L2 transition phase package installer: PASS"
