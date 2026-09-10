#!/usr/bin/env bash
set -euo pipefail

RESOURCE_EXIT=75
SCRIPT_PATH="$(readlink -f "${BASH_SOURCE[0]}")"

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

require_unsigned_integer() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+$ ]] || resource_failure invalid_configuration "$name=$value"
}

require_nonnegative_number() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]] \
    || resource_failure invalid_configuration "$name=$value"
}

meminfo_kib() {
  local key="$1"
  local path="$2"
  awk -v key="$key:" '$1 == key { print $2; found=1; exit } END { if (!found) exit 1 }' "$path"
}

pressure_full_avg10() {
  local path="$1"
  awk '
    $1 == "full" {
      for (field_idx = 2; field_idx <= NF; field_idx++) {
        if ($field_idx ~ /^avg10=/) {
          sub(/^avg10=/, "", $field_idx)
          print $field_idx
          found=1
          exit
        }
      }
    }
    END { if (!found) exit 1 }
  ' "$path"
}

float_greater_than() {
  awk -v actual="$1" -v limit="$2" 'BEGIN { exit !(actual > limit) }'
}

group_alive() {
  local group_id="$1"
  kill -0 -- "-$group_id" 2>/dev/null
}

terminate_group() {
  local group_id="$1"
  local ticks="${LAY_RESOURCE_TERM_GRACE_TICKS:-20}"
  local tick_sleep="${LAY_RESOURCE_TERM_GRACE_SLEEP:-0.1}"
  require_unsigned_integer LAY_RESOURCE_TERM_GRACE_TICKS "$ticks"

  if ! group_alive "$group_id"; then
    return
  fi
  kill -TERM -- "-$group_id" 2>/dev/null || true
  local tick
  for ((tick = 0; tick < ticks; tick++)); do
    if ! group_alive "$group_id"; then
      return
    fi
    sleep "$tick_sleep"
  done
  kill -KILL -- "-$group_id" 2>/dev/null || true
  for ((tick = 0; tick < ticks; tick++)); do
    if ! group_alive "$group_id"; then
      return
    fi
    sleep "$tick_sleep"
  done
  return 1
}

run_inside_scope() {
  [[ "${LAY_RESOURCE_GUARD_ACTIVE:-0}" == "1" ]] \
    || resource_failure invalid_reentry "active_marker=missing"
  [[ "${LAY_RESOURCE_LEASE_HELD:-0}" == "1" ]] \
    || resource_failure invalid_reentry "lease_marker=missing"
  [[ "$#" -gt 0 ]] || resource_failure invalid_command "argc=0"

  local nice_increment="${LAY_RESOURCE_NICE_INCREMENT:-10}"
  local io_priority="${LAY_RESOURCE_IO_PRIORITY:-7}"
  require_unsigned_integer LAY_RESOURCE_NICE_INCREMENT "$nice_increment"
  require_unsigned_integer LAY_RESOURCE_IO_PRIORITY "$io_priority"

  local child_pid=""
  # shellcheck disable=SC2329 # Called by the signal traps below.
  handle_signal() {
    local status="$1"
    trap - INT TERM HUP
    if [[ -n "$child_pid" ]]; then
      if ! terminate_group "$child_pid"; then
        printf 'lay_resource_guard=BLOCKED_RESOURCE reason=descendant_cleanup group=%s\n' \
          "$child_pid" >&2
        exit "$RESOURCE_EXIT"
      fi
      wait "$child_pid" 2>/dev/null || true
    fi
    exit "$status"
  }
  trap 'handle_signal 130' INT
  trap 'handle_signal 143' TERM
  trap 'handle_signal 129' HUP

  setsid nice -n "$nice_increment" ionice -c 2 -n "$io_priority" "$@" 9>&- &
  child_pid=$!
  set +e
  wait "$child_pid"
  local status=$?
  set -e

  if ! terminate_group "$child_pid"; then
    resource_failure descendant_cleanup "group=$child_pid"
  fi
  child_pid=""
  trap - INT TERM HUP
  return "$status"
}

if [[ "${1:-}" == "--inside" ]]; then
  shift
  run_inside_scope "$@"
  exit $?
fi

if [[ "${1:-}" != "--" || "$#" -lt 2 ]]; then
  echo "usage: scripts/lay-resource-guard.sh -- <command> [args...]" >&2
  exit 2
fi
shift

for command in flock setsid nice ionice nproc timeout; do
  command -v "$command" >/dev/null 2>&1 \
    || resource_failure missing_tool "tool=$command"
done

lock_path="$(resource_lock_path)"
exec 9>"$lock_path"
if ! flock -n 9; then
  resource_failure lock_busy "lock=$lock_path"
fi

profile="${LAY_RESOURCE_PROFILE:-workstation}"
case "$profile" in
  workstation)
    default_jobs=2
    maximum_jobs=2
    required_cpus=0
    default_nice=10
    default_io_priority=7
    default_min_available_mib=8192
    default_min_swap_free_mib=4096
    default_cpu_quota="200%"
    default_cpu_weight=10
    default_memory_high="4G"
    default_memory_max="6G"
    default_memory_swap_max="1G"
    default_tasks_max=128
    default_io_weight=10
    ;;
  dedicated-20cpu)
    default_jobs=20
    maximum_jobs=20
    required_cpus=20
    default_nice=0
    default_io_priority=4
    default_min_available_mib=16384
    default_min_swap_free_mib=1024
    default_cpu_quota="2000%"
    default_cpu_weight=100
    default_memory_high="24G"
    default_memory_max="28G"
    default_memory_swap_max="1G"
    default_tasks_max=512
    default_io_weight=100
    ;;
  *) resource_failure invalid_configuration "LAY_RESOURCE_PROFILE=$profile" ;;
esac

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$default_jobs}"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
require_unsigned_integer CARGO_BUILD_JOBS "$CARGO_BUILD_JOBS"
require_unsigned_integer RUST_TEST_THREADS "$RUST_TEST_THREADS"
if (( CARGO_BUILD_JOBS == 0 || CARGO_BUILD_JOBS > maximum_jobs )); then
  resource_failure profile_jobs \
    "profile=$profile jobs=$CARGO_BUILD_JOBS maximum=$maximum_jobs"
fi
if (( RUST_TEST_THREADS != 1 )); then
  resource_failure test_threads "threads=$RUST_TEST_THREADS required=1"
fi
if (( required_cpus > 0 )); then
  available_cpus="$(nproc)"
  require_unsigned_integer available_cpus "$available_cpus"
  if (( available_cpus < required_cpus )); then
    resource_failure cpu_count \
      "profile=$profile available=$available_cpus required=$required_cpus"
  fi
fi

export LAY_RESOURCE_PROFILE="$profile"
export LAY_RESOURCE_NICE_INCREMENT="${LAY_RESOURCE_NICE_INCREMENT:-$default_nice}"
export LAY_RESOURCE_IO_PRIORITY="${LAY_RESOURCE_IO_PRIORITY:-$default_io_priority}"
export LAY_RESOURCE_GUARD_ACTIVE=1
export LAY_RESOURCE_LEASE_HELD=1

mode="${LAY_RESOURCE_GUARD_MODE:-scope}"
if [[ "$profile" == "dedicated-20cpu" && "$mode" != "scope" ]]; then
  resource_failure profile_requires_scope "profile=$profile mode=$mode"
fi

case "$mode" in
  direct)
    printf 'lay_resource_guard=ACTIVE mode=direct profile=%s jobs=%s test_threads=%s lease=%s\n' \
      "$profile" "$CARGO_BUILD_JOBS" "$RUST_TEST_THREADS" "$lock_path" >&2
    run_inside_scope "$@"
    exit $?
    ;;
  scope) ;;
  *) resource_failure invalid_configuration "LAY_RESOURCE_GUARD_MODE=$mode" ;;
esac

meminfo_path="${LAY_RESOURCE_MEMINFO_PATH:-/proc/meminfo}"
memory_pressure_path="${LAY_RESOURCE_MEMORY_PRESSURE_PATH:-/proc/pressure/memory}"
io_pressure_path="${LAY_RESOURCE_IO_PRESSURE_PATH:-/proc/pressure/io}"
min_available_mib="${LAY_RESOURCE_MIN_AVAILABLE_MIB:-$default_min_available_mib}"
min_swap_free_mib="${LAY_RESOURCE_MIN_SWAP_FREE_MIB:-$default_min_swap_free_mib}"
max_memory_full_avg10="${LAY_RESOURCE_MAX_MEMORY_FULL_AVG10:-2.0}"
max_io_full_avg10="${LAY_RESOURCE_MAX_IO_FULL_AVG10:-10.0}"

require_unsigned_integer LAY_RESOURCE_MIN_AVAILABLE_MIB "$min_available_mib"
require_unsigned_integer LAY_RESOURCE_MIN_SWAP_FREE_MIB "$min_swap_free_mib"
require_nonnegative_number LAY_RESOURCE_MAX_MEMORY_FULL_AVG10 "$max_memory_full_avg10"
require_nonnegative_number LAY_RESOURCE_MAX_IO_FULL_AVG10 "$max_io_full_avg10"

[[ -r "$meminfo_path" ]] || resource_failure host_metrics_unavailable "path=$meminfo_path"
available_kib="$(meminfo_kib MemAvailable "$meminfo_path")" \
  || resource_failure host_metrics_unavailable "metric=MemAvailable path=$meminfo_path"
swap_total_kib="$(meminfo_kib SwapTotal "$meminfo_path")" \
  || resource_failure host_metrics_unavailable "metric=SwapTotal path=$meminfo_path"
swap_free_kib="$(meminfo_kib SwapFree "$meminfo_path")" \
  || resource_failure host_metrics_unavailable "metric=SwapFree path=$meminfo_path"
require_unsigned_integer MemAvailable "$available_kib"
require_unsigned_integer SwapTotal "$swap_total_kib"
require_unsigned_integer SwapFree "$swap_free_kib"

if (( available_kib < min_available_mib * 1024 )); then
  resource_failure memory_available \
    "available_mib=$((available_kib / 1024)) minimum_mib=$min_available_mib"
fi
if (( swap_total_kib > 0 && swap_free_kib < min_swap_free_mib * 1024 )); then
  resource_failure swap_free \
    "free_mib=$((swap_free_kib / 1024)) minimum_mib=$min_swap_free_mib"
fi

[[ -r "$memory_pressure_path" ]] \
  || resource_failure host_metrics_unavailable "path=$memory_pressure_path"
memory_full_avg10="$(pressure_full_avg10 "$memory_pressure_path")" \
  || resource_failure host_metrics_unavailable "metric=memory_full_avg10"
require_nonnegative_number memory_full_avg10 "$memory_full_avg10"
if float_greater_than "$memory_full_avg10" "$max_memory_full_avg10"; then
  resource_failure memory_pressure \
    "full_avg10=$memory_full_avg10 maximum=$max_memory_full_avg10"
fi

[[ -r "$io_pressure_path" ]] \
  || resource_failure host_metrics_unavailable "path=$io_pressure_path"
io_full_avg10="$(pressure_full_avg10 "$io_pressure_path")" \
  || resource_failure host_metrics_unavailable "metric=io_full_avg10"
require_nonnegative_number io_full_avg10 "$io_full_avg10"
if float_greater_than "$io_full_avg10" "$max_io_full_avg10"; then
  resource_failure io_pressure \
    "full_avg10=$io_full_avg10 maximum=$max_io_full_avg10"
fi

systemd_run="${LAY_RESOURCE_SYSTEMD_RUN:-systemd-run}"
command -v "$systemd_run" >/dev/null 2>&1 \
  || resource_failure missing_tool "tool=$systemd_run"
systemctl_command="${LAY_RESOURCE_SYSTEMCTL:-systemctl}"
command -v "$systemctl_command" >/dev/null 2>&1 \
  || resource_failure missing_tool "tool=$systemctl_command"

cpu_quota="${LAY_RESOURCE_CPU_QUOTA:-$default_cpu_quota}"
cpu_weight="${LAY_RESOURCE_CPU_WEIGHT:-$default_cpu_weight}"
memory_high="${LAY_RESOURCE_MEMORY_HIGH:-$default_memory_high}"
memory_max="${LAY_RESOURCE_MEMORY_MAX:-$default_memory_max}"
memory_swap_max="${LAY_RESOURCE_MEMORY_SWAP_MAX:-$default_memory_swap_max}"
tasks_max="${LAY_RESOURCE_TASKS_MAX:-$default_tasks_max}"
io_weight="${LAY_RESOURCE_IO_WEIGHT:-$default_io_weight}"
unit="lay-verify-$(id -u)-${BASHPID:-$$}"

printf 'lay_resource_guard=ACTIVE mode=scope profile=%s jobs=%s test_threads=%s cpu_quota=%s memory_high=%s memory_max=%s swap_max=%s tasks_max=%s memory_full_avg10=%s io_full_avg10=%s lease=%s\n' \
  "$profile" "$CARGO_BUILD_JOBS" "$RUST_TEST_THREADS" "$cpu_quota" "$memory_high" \
  "$memory_max" "$memory_swap_max" "$tasks_max" "$memory_full_avg10" \
  "$io_full_avg10" "$lock_path" >&2

scope_pid=""
systemctl_timeout="${LAY_RESOURCE_SYSTEMCTL_TIMEOUT_SECONDS:-2}"
require_unsigned_integer LAY_RESOURCE_SYSTEMCTL_TIMEOUT_SECONDS "$systemctl_timeout"
if (( systemctl_timeout == 0 )); then
  resource_failure invalid_configuration \
    "LAY_RESOURCE_SYSTEMCTL_TIMEOUT_SECONDS=$systemctl_timeout"
fi

bounded_systemctl() {
  timeout --foreground --kill-after=1s "${systemctl_timeout}s" \
    "$systemctl_command" --user "$@"
}

scope_active() {
  local status
  if bounded_systemctl is-active --quiet "${unit}.scope" >/dev/null 2>&1; then
    return 0
  else
    status=$?
  fi
  case "$status" in
    3|4) return 1 ;;
    *) return 2 ;;
  esac
}

cleanup_scope() {
  local ticks="${LAY_RESOURCE_TERM_GRACE_TICKS:-20}"
  local tick_sleep="${LAY_RESOURCE_TERM_GRACE_SLEEP:-0.1}"
  require_unsigned_integer LAY_RESOURCE_TERM_GRACE_TICKS "$ticks"

  local scope_status
  if scope_active; then
    :
  else
    scope_status=$?
    [[ "$scope_status" == "1" ]] && return 0
    return 1
  fi
  bounded_systemctl kill --signal=TERM --kill-whom=all \
    "${unit}.scope" >/dev/null 2>&1 || true
  local tick
  for ((tick = 0; tick < ticks; tick++)); do
    if scope_active; then
      :
    else
      scope_status=$?
      [[ "$scope_status" == "1" ]] && return 0
      return 1
    fi
    sleep "$tick_sleep"
  done
  bounded_systemctl kill --signal=KILL --kill-whom=all \
    "${unit}.scope" >/dev/null 2>&1 || true
  for ((tick = 0; tick < ticks; tick++)); do
    if scope_active; then
      :
    else
      scope_status=$?
      [[ "$scope_status" == "1" ]] && return 0
      return 1
    fi
    sleep "$tick_sleep"
  done
  return 1
}

# shellcheck disable=SC2329 # Called by the signal traps below.
stop_scope() {
  local status="$1"
  local ticks="${LAY_RESOURCE_TERM_GRACE_TICKS:-20}"
  local tick_sleep="${LAY_RESOURCE_TERM_GRACE_SLEEP:-0.1}"
  require_unsigned_integer LAY_RESOURCE_TERM_GRACE_TICKS "$ticks"
  trap - INT TERM HUP
  if [[ -n "$scope_pid" ]]; then
    local cleanup_failed=0
    kill -TERM "$scope_pid" 2>/dev/null || true
    cleanup_scope || cleanup_failed=1
    local tick
    for ((tick = 0; tick < ticks; tick++)); do
      if ! kill -0 "$scope_pid" 2>/dev/null; then
        break
      fi
      sleep "$tick_sleep"
    done
    if kill -0 "$scope_pid" 2>/dev/null; then
      kill -KILL "$scope_pid" 2>/dev/null || true
    fi
    wait "$scope_pid" 2>/dev/null || true
    cleanup_scope || cleanup_failed=1
    if (( cleanup_failed != 0 )); then
      printf 'lay_resource_guard=BLOCKED_RESOURCE reason=descendant_cleanup unit=%s\n' \
        "${unit}.scope" >&2
      exit "$RESOURCE_EXIT"
    fi
  fi
  exit "$status"
}
trap 'stop_scope 130' INT
trap 'stop_scope 143' TERM
trap 'stop_scope 129' HUP

"$systemd_run" \
  --user \
  --scope \
  --quiet \
  --collect \
  --unit="$unit" \
  -p "CPUQuota=$cpu_quota" \
  -p "CPUWeight=$cpu_weight" \
  -p "MemoryHigh=$memory_high" \
  -p "MemoryMax=$memory_max" \
  -p "MemorySwapMax=$memory_swap_max" \
  -p "TasksMax=$tasks_max" \
  -p "IOWeight=$io_weight" \
  -p "KillMode=control-group" \
  "$SCRIPT_PATH" --inside "$@" 9>&- &
scope_pid=$!
set +e
wait "$scope_pid"
status=$?
set -e
if ! cleanup_scope; then
  resource_failure descendant_cleanup "unit=${unit}.scope"
fi
scope_pid=""
trap - INT TERM HUP
exit "$status"
