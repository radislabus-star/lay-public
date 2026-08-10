#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="${LAY_NANDA_L3_CONTEXT_MANIFEST:-$HOME/.local/share/lay/nanda_wave/l3_context_phase.runtime.json}"
STATUS_DIR="${LAY_NANDA_L3_RUNTIME_STATUS_DIR:-${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/lay/l3-context}"
RECEIPT="${1:-$ROOT/docs/structural_gates/receipts/L3_PROCESS_LOCAL_REFRESH_2026-08-10.json}"
TIMEOUT_SECONDS="${LAY_L3_REFRESH_PROOF_TIMEOUT_SECONDS:-8}"

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command is unavailable: $1" >&2
    exit 2
  }
}

for command in jq sha256sum systemctl pgrep stat; do
  require_command "$command"
done

[[ -f "$MANIFEST" ]] || {
  echo "L3 runtime manifest is missing: $MANIFEST" >&2
  exit 2
}

daemon_pid="$(systemctl --user show -p MainPID --value lay-daemon.service)"
ibus_pid="$(pgrep -n -f '/lay-ibus-engine( |$).*--managed' || true)"
global_ibus_pid="$(pgrep -o -x ibus-daemon || true)"

[[ "$daemon_pid" =~ ^[1-9][0-9]*$ && -d "/proc/$daemon_pid" ]] || {
  echo "lay-daemon is not running" >&2
  exit 2
}
[[ "$ibus_pid" =~ ^[1-9][0-9]*$ && -d "/proc/$ibus_pid" ]] || {
  echo "managed lay-ibus-engine is not running" >&2
  exit 2
}

status_for_pid() {
  local pid="$1"
  find "$STATUS_DIR" -maxdepth 1 -type f -name "*-$pid.json" -print -quit 2>/dev/null
}

wait_for_status() {
  local pid="$1"
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  local path=""
  while (( SECONDS <= deadline )); do
    path="$(status_for_pid "$pid")"
    if [[ -n "$path" ]] && jq -e --argjson pid "$pid" '.pid == $pid and .memory_warm == true' "$path" >/dev/null; then
      printf '%s\n' "$path"
      return 0
    fi
    sleep 0.1
  done
  echo "no L3 process status for PID $pid under $STATUS_DIR" >&2
  return 1
}

daemon_status="$(wait_for_status "$daemon_pid")"
ibus_status="$(wait_for_status "$ibus_pid")"

daemon_generation_before="$(jq -r '.load_generation' "$daemon_status")"
ibus_generation_before="$(jq -r '.load_generation' "$ibus_status")"
daemon_success_before="$(jq -r '.refresh_successes' "$daemon_status")"
ibus_success_before="$(jq -r '.refresh_successes' "$ibus_status")"
daemon_stamp_before="$(jq -r '.memory.manifest_stamp' "$daemon_status")"
ibus_stamp_before="$(jq -r '.memory.manifest_stamp' "$ibus_status")"

for value in \
  "$daemon_generation_before" "$ibus_generation_before" \
  "$daemon_success_before" "$ibus_success_before" \
  "$daemon_stamp_before" "$ibus_stamp_before"; do
  [[ "$value" =~ ^[0-9]+$ ]] || {
    echo "invalid process status counter: $value" >&2
    exit 2
  }
done

manifest_sha_before="$(sha256sum "$MANIFEST" | awk '{print $1}')"
manifest_bytes_before="$(stat -c '%s' "$MANIFEST")"
base_relative="$(jq -er '.base' "$MANIFEST")"
if [[ "$base_relative" = /* ]]; then
  base_path="$base_relative"
else
  base_path="$(dirname "$MANIFEST")/$base_relative"
fi
[[ -f "$base_path" ]] || {
  echo "L3 base package is missing: $base_path" >&2
  exit 2
}
base_sha_before="$(sha256sum "$base_path" | awk '{print $1}')"

daemon_model_before="$(jq -c '.memory | del(.manifest_stamp)' "$daemon_status")"
ibus_model_before="$(jq -c '.memory | del(.manifest_stamp)' "$ibus_status")"

# Ensure that even a filesystem with coarse timestamp resolution publishes a
# different manifest stamp. The bytes and model reference remain identical.
sleep 1.1
temporary="$(dirname "$MANIFEST")/.l3-refresh-proof.$$.tmp"
cleanup() {
  rm -f "$temporary"
}
trap cleanup EXIT
install -m 600 "$MANIFEST" "$temporary"
sync "$temporary"
mv -f "$temporary" "$MANIFEST"

wait_for_generation() {
  local path="$1"
  local previous_generation="$2"
  local previous_successes="$3"
  local previous_stamp="$4"
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while (( SECONDS <= deadline )); do
    if jq -e \
      --argjson generation "$previous_generation" \
      --argjson successes "$previous_successes" \
      --argjson stamp "$previous_stamp" \
      '.event == "manifest_refresh"
       and .load_generation > $generation
       and .refresh_successes > $successes
       and .memory.manifest_stamp != $stamp
       and .refresh_in_flight == false' \
      "$path" >/dev/null; then
      return 0
    fi
    sleep 0.1
  done
  echo "PID $(jq -r '.pid' "$path") did not install the republished L3 manifest" >&2
  return 1
}

wait_for_generation \
  "$daemon_status" "$daemon_generation_before" "$daemon_success_before" "$daemon_stamp_before"
wait_for_generation \
  "$ibus_status" "$ibus_generation_before" "$ibus_success_before" "$ibus_stamp_before"

daemon_generation_after="$(jq -r '.load_generation' "$daemon_status")"
ibus_generation_after="$(jq -r '.load_generation' "$ibus_status")"
daemon_success_after="$(jq -r '.refresh_successes' "$daemon_status")"
ibus_success_after="$(jq -r '.refresh_successes' "$ibus_status")"

[[ -d "/proc/$daemon_pid" && -d "/proc/$ibus_pid" ]] || {
  echo "a live process changed during the refresh proof" >&2
  exit 1
}
[[ "$(systemctl --user show -p MainPID --value lay-daemon.service)" == "$daemon_pid" ]] || {
  echo "lay-daemon PID changed during the refresh proof" >&2
  exit 1
}
[[ "$(pgrep -n -f '/lay-ibus-engine( |$).*--managed' || true)" == "$ibus_pid" ]] || {
  echo "managed IBus engine PID changed during the refresh proof" >&2
  exit 1
}
if [[ -n "$global_ibus_pid" ]]; then
  [[ "$(pgrep -o -x ibus-daemon || true)" == "$global_ibus_pid" ]] || {
    echo "global ibus-daemon PID changed during the refresh proof" >&2
    exit 1
  }
fi

manifest_sha_after="$(sha256sum "$MANIFEST" | awk '{print $1}')"
manifest_bytes_after="$(stat -c '%s' "$MANIFEST")"
base_sha_after="$(sha256sum "$base_path" | awk '{print $1}')"
daemon_model_after="$(jq -c '.memory | del(.manifest_stamp)' "$daemon_status")"
ibus_model_after="$(jq -c '.memory | del(.manifest_stamp)' "$ibus_status")"

[[ "$manifest_sha_after" == "$manifest_sha_before" ]] || {
  echo "manifest bytes changed during equivalent republish" >&2
  exit 1
}
[[ "$manifest_bytes_after" == "$manifest_bytes_before" ]] || {
  echo "manifest size changed during equivalent republish" >&2
  exit 1
}
[[ "$base_sha_after" == "$base_sha_before" ]] || {
  echo "L3 base package changed during refresh proof" >&2
  exit 1
}
[[ "$daemon_model_after" == "$daemon_model_before" ]] || {
  echo "daemon loaded model identity changed" >&2
  exit 1
}
[[ "$ibus_model_after" == "$ibus_model_before" ]] || {
  echo "IBus loaded model identity changed" >&2
  exit 1
}
[[ "$daemon_model_after" == "$ibus_model_after" ]] || {
  echo "daemon and IBus do not expose the same L3 model identity" >&2
  exit 1
}

mkdir -p "$(dirname "$RECEIPT")"
jq -n \
  --arg manifest "$MANIFEST" \
  --arg manifest_sha "$manifest_sha_after" \
  --arg base "$base_path" \
  --arg base_sha "$base_sha_after" \
  --arg daemon_status "$daemon_status" \
  --arg ibus_status "$ibus_status" \
  --argjson manifest_bytes "$manifest_bytes_after" \
  --argjson daemon_pid "$daemon_pid" \
  --argjson ibus_pid "$ibus_pid" \
  --argjson global_ibus_pid "${global_ibus_pid:-0}" \
  --argjson daemon_generation_before "$daemon_generation_before" \
  --argjson daemon_generation_after "$daemon_generation_after" \
  --argjson ibus_generation_before "$ibus_generation_before" \
  --argjson ibus_generation_after "$ibus_generation_after" \
  --argjson daemon_success_before "$daemon_success_before" \
  --argjson daemon_success_after "$daemon_success_after" \
  --argjson ibus_success_before "$ibus_success_before" \
  --argjson ibus_success_after "$ibus_success_after" \
  '{
    gate: "L3_PROCESS_LOCAL_REFRESH",
    date: "2026-08-10",
    verdict: "PASS",
    method: "atomic byte-identical manifest republish with process-local Arc generation telemetry",
    manifest: $manifest,
    manifest_bytes: $manifest_bytes,
    manifest_sha256_before_after_equal: true,
    manifest_sha256: $manifest_sha,
    base: $base,
    base_sha256_before_after_equal: true,
    base_sha256: $base_sha,
    model_report_before_after_equal: true,
    daemon: {
      pid: $daemon_pid,
      status: $daemon_status,
      load_generation_before: $daemon_generation_before,
      load_generation_after: $daemon_generation_after,
      refresh_successes_before: $daemon_success_before,
      refresh_successes_after: $daemon_success_after,
      pid_preserved: true
    },
    managed_ibus: {
      pid: $ibus_pid,
      status: $ibus_status,
      load_generation_before: $ibus_generation_before,
      load_generation_after: $ibus_generation_after,
      refresh_successes_before: $ibus_success_before,
      refresh_successes_after: $ibus_success_after,
      pid_preserved: true
    },
    global_ibus_pid: $global_ibus_pid,
    global_ibus_pid_preserved: true,
    candidate_or_weight_mutation: false,
    direct_edit_authority_changed: false,
    not_tested: [
      "candidate UI rendering after a semantically different admitted delta",
      "multi-day watcher residency"
    ]
  }' > "$RECEIPT"

echo "$RECEIPT"
