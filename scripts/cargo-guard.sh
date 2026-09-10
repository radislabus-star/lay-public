#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
MAX_BYTES="${LAY_CARGO_TARGET_MAX_BYTES:-12884901888}"
POLL_SECONDS="${LAY_CARGO_TARGET_POLL_SECONDS:-5}"
RESOURCE_EXIT=75

resource_failure() {
  local reason="$1"
  shift
  printf 'lay_resource_guard=BLOCKED_RESOURCE reason=%s' "$reason" >&2
  if [[ "$#" -gt 0 ]]; then
    printf ' %s' "$*" >&2
  fi
  printf '\n' >&2
  exit "$RESOURCE_EXIT"
}

resource_lock_path() {
  if [[ -n "${LAY_RESOURCE_LOCK_PATH:-}" ]]; then
    printf '%s\n' "$LAY_RESOURCE_LOCK_PATH"
    return
  fi

  local runtime_dir="${XDG_RUNTIME_DIR:-}"
  if [[ -z "$runtime_dir" || ! -d "$runtime_dir" || ! -w "$runtime_dir" ]]; then
    runtime_dir="/run/user/$(id -u)"
  fi
  if [[ ! -d "$runtime_dir" || ! -w "$runtime_dir" ]]; then
    runtime_dir="${TMPDIR:-/tmp}"
  fi
  printf '%s/lay-verification-heavy-%s.lock\n' "$runtime_dir" "$(id -u)"
}

target_bytes() {
  if [[ -d "$TARGET_DIR" ]]; then
    local bytes
    bytes="$(du -s --block-size=1 "$TARGET_DIR" 2>/dev/null | awk '{print $1}' || true)"
    if [[ "$bytes" =~ ^[0-9]+$ ]]; then
      printf '%s\n' "$bytes"
    else
      return 1
    fi
  else
    printf '0\n'
  fi
}

human_bytes() {
  numfmt --to=iec-i --suffix=B "$1"
}

check_budget() {
  local bytes
  if ! bytes="$(target_bytes)"; then
    echo "Cannot measure Cargo target size: $TARGET_DIR" >&2
    return 1
  fi
  if (( bytes > MAX_BYTES )); then
    printf 'Cargo target budget exceeded: %s > %s (%s)\n' \
      "$(human_bytes "$bytes")" "$(human_bytes "$MAX_BYTES")" "$TARGET_DIR" >&2
    return 1
  fi
}

if [[ "${1:-}" == "--status" ]]; then
  bytes="$(target_bytes)"
  printf 'cargo_target=%s bytes=%s budget=%s\n' \
    "$TARGET_DIR" "$bytes" "$MAX_BYTES"
  check_budget
  exit
fi

if [[ "$#" == "0" ]]; then
  echo "usage: scripts/cargo-guard.sh <cargo arguments> | --status" >&2
  exit 2
fi

if [[ "${LAY_RESOURCE_GUARD_ACTIVE:-0}" != "1" || "${LAY_RESOURCE_LEASE_HELD:-0}" != "1" ]]; then
  command -v flock >/dev/null 2>&1 || {
    echo "lay_resource_guard=BLOCKED_RESOURCE reason=missing_tool tool=flock" >&2
    exit "$RESOURCE_EXIT"
  }
  lock_path="$(resource_lock_path)"
  exec 8>"$lock_path"
  if ! flock -n 8; then
    printf 'lay_resource_guard=BLOCKED_RESOURCE reason=lock_busy lock=%s\n' \
      "$lock_path" >&2
    exit "$RESOURCE_EXIT"
  fi
fi

check_budget
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
if [[ ! "$CARGO_BUILD_JOBS" =~ ^[0-9]+$ || "$CARGO_BUILD_JOBS" == "0" ]]; then
  resource_failure invalid_configuration "CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS"
fi
case "${LAY_RESOURCE_PROFILE:-workstation}" in
  workstation)
    maximum_jobs=2
    ;;
  dedicated-20cpu)
    maximum_jobs=20
    if [[ "${LAY_RESOURCE_GUARD_ACTIVE:-0}" != "1" || "${LAY_RESOURCE_LEASE_HELD:-0}" != "1" ]]; then
      resource_failure profile_requires_scope "profile=dedicated-20cpu"
    fi
    ;;
  *) resource_failure invalid_configuration \
       "LAY_RESOURCE_PROFILE=${LAY_RESOURCE_PROFILE:-}" ;;
esac
if (( CARGO_BUILD_JOBS > maximum_jobs )); then
  resource_failure profile_jobs \
    "profile=${LAY_RESOURCE_PROFILE:-workstation} jobs=$CARGO_BUILD_JOBS maximum=$maximum_jobs"
fi

args=("$@")
for ((arg_index = 0; arg_index < ${#args[@]}; arg_index++)); do
  argument="${args[$arg_index]}"
  requested_jobs=""
  case "$argument" in
    -j|--jobs)
      ((arg_index += 1))
      if (( arg_index >= ${#args[@]} )); then
        resource_failure invalid_configuration "argument=$argument"
      fi
      requested_jobs="${args[$arg_index]}"
      ;;
    -j?*) requested_jobs="${argument#-j}" ;;
    --jobs=*) requested_jobs="${argument#--jobs=}" ;;
    --config)
      if (( arg_index + 1 < ${#args[@]} )) \
        && [[ "${args[$((arg_index + 1))]}" == *build.jobs* ]]; then
        resource_failure jobs_configuration_override "argument=--config"
      fi
      ;;
    --config=*build.jobs*)
      resource_failure jobs_configuration_override "argument=--config"
      ;;
  esac
  if [[ -n "$requested_jobs" ]]; then
    if [[ ! "$requested_jobs" =~ ^[0-9]+$ || "$requested_jobs" == "0" ]]; then
      resource_failure invalid_configuration "jobs_argument=$requested_jobs"
    fi
    if (( requested_jobs > maximum_jobs )); then
      resource_failure profile_jobs \
        "profile=${LAY_RESOURCE_PROFILE:-workstation} jobs=$requested_jobs maximum=$maximum_jobs"
    fi
  fi
done

toolchain="${LAY_RUST_TOOLCHAIN:-}"
if [[ -z "$toolchain" ]] && ! rustc --version >/dev/null 2>&1; then
  toolchain="1.97.1"
fi

cargo_command=(cargo)
if [[ -n "$toolchain" ]]; then
  cargo_command+=("+$toolchain")
fi

child_pid=""
monitor_pid=""
# shellcheck disable=SC2329 # Called by the signal traps below.
stop_child() {
  if [[ -n "$monitor_pid" ]]; then
    kill "$monitor_pid" 2>/dev/null || true
  fi
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM -- "-$child_pid" 2>/dev/null || true
    for _ in {1..20}; do
      if ! kill -0 -- "-$child_pid" 2>/dev/null; then
        return
      fi
      sleep 0.1
    done
    kill -KILL -- "-$child_pid" 2>/dev/null || true
  fi
}
trap stop_child INT TERM HUP

# The guard process owns the host-wide lease. Cargo and anything it launches
# must not retain that lease after the guarded command has returned.
setsid "${cargo_command[@]}" "$@" 8>&- &
child_pid=$!

monitor_budget() {
  local monitor_sleep_pid=""
  stop_monitor_sleep() {
    if [[ -n "$monitor_sleep_pid" ]]; then
      kill "$monitor_sleep_pid" 2>/dev/null || true
      wait "$monitor_sleep_pid" 2>/dev/null || true
    fi
    exit 0
  }
  trap stop_monitor_sleep INT TERM HUP

  while kill -0 "$child_pid" 2>/dev/null; do
    sleep "$POLL_SECONDS" &
    monitor_sleep_pid=$!
    wait "$monitor_sleep_pid" || return
    monitor_sleep_pid=""
    if ! bytes="$(target_bytes)"; then
      continue
    fi
    if (( bytes > MAX_BYTES )); then
      printf 'Stopping Cargo: target grew to %s; budget is %s.\n' \
        "$(human_bytes "$bytes")" "$(human_bytes "$MAX_BYTES")" >&2
      kill -TERM -- "-$child_pid" 2>/dev/null || true
      sleep 1 &
      monitor_sleep_pid=$!
      wait "$monitor_sleep_pid" || return
      monitor_sleep_pid=""
      kill -KILL -- "-$child_pid" 2>/dev/null || true
      return
    fi
  done
}

# The monitor only observes the guarded process and target size. It must not
# keep the host-wide lease alive through its polling sleep after Cargo exits.
monitor_budget 8>&- &
monitor_pid=$!

set +e
wait "$child_pid"
status=$?
set -e
child_pid=""
kill "$monitor_pid" 2>/dev/null || true
wait "$monitor_pid" 2>/dev/null || true
monitor_pid=""

if ! check_budget; then
  exit 75
fi
exit "$status"
