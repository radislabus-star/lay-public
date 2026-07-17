#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
MAX_BYTES="${LAY_CARGO_TARGET_MAX_BYTES:-12884901888}"
POLL_SECONDS="${LAY_CARGO_TARGET_POLL_SECONDS:-2}"

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

check_budget
export CARGO_INCREMENTAL=0

toolchain="${LAY_RUST_TOOLCHAIN:-}"
if [[ -z "$toolchain" ]] && ! rustc --version >/dev/null 2>&1; then
  toolchain="1.97.0"
fi

cargo_command=(cargo)
if [[ -n "$toolchain" ]]; then
  cargo_command+=("+$toolchain")
fi

child_pid=""
monitor_pid=""
stop_child() {
  if [[ -n "$monitor_pid" ]]; then
    kill "$monitor_pid" 2>/dev/null || true
  fi
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM -- "-$child_pid" 2>/dev/null || true
  fi
}
trap stop_child INT TERM HUP

setsid "${cargo_command[@]}" "$@" &
child_pid=$!

monitor_budget() {
  while kill -0 "$child_pid" 2>/dev/null; do
    sleep "$POLL_SECONDS"
    if ! bytes="$(target_bytes)"; then
      continue
    fi
    if (( bytes > MAX_BYTES )); then
      printf 'Stopping Cargo: target grew to %s; budget is %s.\n' \
        "$(human_bytes "$bytes")" "$(human_bytes "$MAX_BYTES")" >&2
      kill -TERM -- "-$child_pid" 2>/dev/null || true
      sleep 1
      kill -KILL -- "-$child_pid" 2>/dev/null || true
      return
    fi
  done
}

monitor_budget &
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
