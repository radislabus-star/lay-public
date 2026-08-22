#!/usr/bin/env bash
set -euo pipefail

receipt_root=$(cd "$(dirname "$0")" && pwd)
baseline_root=/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/output/logs/baseline49
candidate_root="$receipt_root/remote-proof-v4/baseline49/logs"
normalized_root="$receipt_root/semantic-normalized-v4"
rows="$normalized_root/parity.tsv"

[[ ! -e "$normalized_root" ]] || {
  printf 'semantic normalization output already exists: %s\n' "$normalized_root" >&2
  exit 1
}
mkdir -p "$normalized_root/baseline" "$normalized_root/candidate"
: >"$rows"

normalize() {
  local source=$1 destination=$2
  awk '
    /^running 1 test$/ { inside=1; next }
    inside && /^test result:/ { exit }
    inside { print }
  ' "$source" | sed -E \
    -e "s/\([0-9]+\) panicked/(PID) panicked/g" \
    -e 's/([[:alnum:]_]+_us=)[0-9]+/\1TIME/g' \
    -e 's/finished in [0-9.]+s/finished in TIME/g' \
    >"$destination"
}

for candidate_log in "$candidate_root"/*.log; do
  id=$(basename "$candidate_log" .log)
  baseline_log="$baseline_root/$id.log"
  [[ -f "$baseline_log" ]] || {
    printf 'missing baseline log: %s\n' "$baseline_log" >&2
    exit 1
  }
  baseline_normalized="$normalized_root/baseline/$id.txt"
  candidate_normalized="$normalized_root/candidate/$id.txt"
  normalize "$baseline_log" "$baseline_normalized"
  normalize "$candidate_log" "$candidate_normalized"
  baseline_sha=$(sha256sum "$baseline_normalized" | cut -d' ' -f1)
  candidate_sha=$(sha256sum "$candidate_normalized" | cut -d' ' -f1)
  if [[ "$baseline_sha" == "$candidate_sha" ]]; then verdict=PASS; else verdict=FAIL; fi
  printf '%s\t%s\t%s\t%s\n' "$id" "$verdict" "$baseline_sha" "$candidate_sha" >>"$rows"
done

jq -Rn '
  [inputs | split("\t") | {id:.[0],verdict:.[1],baseline_sha256:.[2],candidate_sha256:.[3]}] as $cases |
  {schema:"lay.ime-target-authority-slice1-semantic-log-parity.v2",normalization:{region:"after exact test starts and before test result summary",ignored_dynamic_fields:["thread numeric id","elapsed seconds","trace fields ending in _us"]},case_count:($cases|length),pass_count:([$cases[]|select(.verdict=="PASS")]|length),failures:[$cases[]|select(.verdict!="PASS")],cases:$cases}
' <"$rows" >"$receipt_root/semantic-output-parity-v4.json"

jq -e '.case_count==49 and .pass_count==49 and (.failures|length)==0' \
  "$receipt_root/semantic-output-parity-v4.json" >/dev/null
jq '{case_count,pass_count,failures}' "$receipt_root/semantic-output-parity-v4.json"
