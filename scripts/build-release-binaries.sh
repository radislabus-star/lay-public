#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
if [[ "$#" != 0 ]]; then
  echo "usage: scripts/build-release-binaries.sh" >&2
  exit 2
fi

binary_names="$("$ROOT/scripts/install-release-binaries.sh" --list-source-binaries)"
build_args=(build --release --locked --features research-tools)
while IFS= read -r binary; do
  [[ -n "$binary" ]] || { echo "empty release binary name" >&2; exit 1; }
  build_args+=(--bin "$binary")
done <<<"$binary_names"

state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/lay"
mkdir -p "$state_dir"
build_log="$(mktemp "$state_dir/build-release-XXXXXXXX.log")"
printf '=== сборка устанавливаемых release-бинарников ===\n'
printf 'Первая сборка может занять несколько минут, особенно на ARM.\n'
printf 'Полный журнал: %s\n' "$build_log"

build_pid=""
logger_pid=""
build_fd=""
# The existing guard remains the sole owner of Cargo's process group and its
# termination deadline. This wrapper only forwards cancellation and drains tee.
# shellcheck disable=SC2329 # Called by signal traps.
cancel_build() {
  local status="$1"
  trap '' INT TERM HUP
  if [[ -n "$build_pid" ]]; then
    kill -TERM "$build_pid" 2>/dev/null || true
    wait "$build_pid" 2>/dev/null || true
  fi
  if [[ -n "$build_fd" ]]; then
    exec {build_fd}>&-
  fi
  if [[ -n "$logger_pid" ]]; then
    wait "$logger_pid" 2>/dev/null || true
  fi
  printf 'Сборка прервана (код %s). Журнал: %s\n' "$status" "$build_log" >&2
  exit "$status"
}
trap 'cancel_build 130' INT
trap 'cancel_build 143' TERM
trap 'cancel_build 129' HUP

exec {build_fd}> >(tee "$build_log")
logger_pid=$!
"$ROOT/scripts/cargo-guard.sh" "${build_args[@]}" >&"$build_fd" 2>&1 &
build_pid=$!
set +e
wait "$build_pid"
build_status=$?
build_pid=""
exec {build_fd}>&-
build_fd=""
wait "$logger_pid"
logger_status=$?
logger_pid=""
set -e
trap - INT TERM HUP
if (( build_status != 0 )); then
  printf 'Сборка не завершена (код %s). Журнал: %s\n' \
    "$build_status" "$build_log" >&2
  exit "$build_status"
fi
if (( logger_status != 0 )); then
  printf 'Не удалось сохранить журнал сборки: %s\n' "$build_log" >&2
  exit "$logger_status"
fi
printf '✓ release-сборка завершена. Журнал: %s\n' "$build_log"
