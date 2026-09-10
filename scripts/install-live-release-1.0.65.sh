#!/usr/bin/env bash
set -u -o pipefail

EXPECTED_VERSION=1.0.65
ROLLBACK_VERSION=1.0.64
EXTENSION_UUID=lay@radislabus-star.github.io
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${HOME}/.local/lib/lay/bin"
LINK_DIR="${HOME}/.local/bin"
EXTENSION_SOURCE="$ROOT/extension/$EXTENSION_UUID"
EXTENSION_DIR="${HOME}/.local/share/gnome-shell/extensions/$EXTENSION_UUID"
L2_DIR="${HOME}/.local/share/lay/nanda_wave/l2"
L3_UNIT_SOURCE="$ROOT/systemd/lay-l3-online.service"
L3_UNIT_PATH="${HOME}/.config/systemd/user/lay-l3-online.service"
L11_BINARY_NAME=lay-l1.1-serve
L11_PROC_ROOT=/proc
L11_GUARD="$ROOT/scripts/lay-release-l11-guard.py"
L11_UNVERIFIED_RC=70
L11_TERM_TIMEOUT_MS=5000
L11_HEALTH_TIMEOUT_MS=2000
L11_READY_TIMEOUT_MS=60000
L11_CAPTURED_PID=
L11_CAPTURED_MEMORY=
L11_CAPTURED_SOCKET=
L11_CAPTURED_HASH=
L11_MEMORY_HASH=
L11_INSTALLED_HASH=
L11_ROLLBACK_HASH=
L11_RELEASE_HASH=
L11_CURRENT_PID=
L11_CURRENT_EXE=
L11_CURRENT_HASH=
L11_STARTED_PID=
L11_STARTED_UNIT=
L11_INSTALLED_BIN_TREE_FINGERPRINT=
L11_INSTALLED_EXTENSION_TREE_FINGERPRINT=
L11_INSTALLED_L2_TREE_FINGERPRINT=
L11_SNAPSHOT_BIN_TREE_FINGERPRINT=
L11_SNAPSHOT_EXTENSION_TREE_FINGERPRINT=
L11_SNAPSHOT_L2_TREE_FINGERPRINT=
declare -a L11_CAPTURED_ARGV=()

usage() {
    echo "usage: $0 --snapshot ABSOLUTE_SNAPSHOT_DIR | --repair-l11 --snapshot ABSOLUTE_SNAPSHOT_DIR | --self-test" >&2
    exit 2
}

file_count() {
    find "$1" -type f -printf '.' | wc -c
}

tree_modes() {
    find "$1" -type f -printf '%P\t%m\n' | LC_ALL=C sort
}

tree_fingerprint() {
    local root="${1:?tree root required}"
    local listing pipeline_rc

    [[ -d "$root" && ! -L "$root" ]] || return 1
    listing="$(mktemp "${TMPDIR:-/tmp}/lay-release-tree.XXXXXX")" || return 1
    if ! find "$root" -mindepth 1 -print0 >"$listing" \
        || ! LC_ALL=C sort -z -o "$listing" "$listing"; then
        rm -f -- "$listing"
        return 1
    fi
    (
        local entry relative mode size digest target

        mode="$(stat -c '%a' -- "$root")" || exit 1
        printf 'root\0%s\0' "$mode"
        while IFS= read -r -d '' entry; do
            relative="${entry#"$root"/}"
            mode="$(stat -c '%a' -- "$entry")" || exit 1
            if [[ -L "$entry" ]]; then
                target="$(readlink -- "$entry")" || exit 1
                printf 'link\0%s\0%s\0%s\0' "$relative" "$mode" "$target"
            elif [[ -f "$entry" ]]; then
                size="$(stat -c '%s' -- "$entry")" || exit 1
                digest="$(sha256sum -- "$entry" | awk '{print $1}')" || exit 1
                printf 'file\0%s\0%s\0%s\0%s\0' \
                    "$relative" "$mode" "$size" "$digest"
            elif [[ -d "$entry" ]]; then
                printf 'dir\0%s\0%s\0' "$relative" "$mode"
            else
                exit 1
            fi
        done <"$listing"
    ) | sha256sum | awk '{print $1}'
    pipeline_rc=$?
    rm -f -- "$listing"
    return "$pipeline_rc"
}

verify_tree_parity() {
    local source_tree="$1"
    local destination_tree="$2"

    [[ -d "$source_tree" && -d "$destination_tree" ]] || return 1
    diff -qr -- "$source_tree" "$destination_tree" >/dev/null || return 1
    cmp -s <(tree_modes "$source_tree") <(tree_modes "$destination_tree")
}

verify_file_bytes_mode() {
    local source_file="${1:?source file required}"
    local destination_file="${2:?destination file required}"
    local expected_mode="${3:?expected mode required}"

    [[ -f "$source_file" && ! -L "$source_file" ]] || return 1
    [[ -f "$destination_file" && ! -L "$destination_file" ]] || return 1
    cmp -s -- "$source_file" "$destination_file" || return 1
    [[ "$(stat -c '%a' -- "$destination_file")" == "$expected_mode" ]]
}

install_file_atomic() {
    local source_file="${1:?source file required}"
    local destination_file="${2:?destination file required}"
    local install_mode="${3:?install mode required}"
    local destination_dir temporary_file

    [[ -f "$source_file" && ! -L "$source_file" ]] || return 1
    [[ ! -L "$destination_file" ]] || return 1
    destination_dir="$(dirname -- "$destination_file")" || return 1
    mkdir -p -- "$destination_dir" || return 1
    [[ -d "$destination_dir" && ! -L "$destination_dir" ]] || return 1
    temporary_file="$destination_dir/.$(basename -- "$destination_file").install.$$.$RANDOM"
    if ! install -m "$install_mode" -- "$source_file" "$temporary_file"; then
        rm -f -- "$temporary_file"
        return 1
    fi
    if ! mv -f -- "$temporary_file" "$destination_file"; then
        rm -f -- "$temporary_file"
        return 1
    fi
    verify_file_bytes_mode "$source_file" "$destination_file" "${install_mode#0}"
}

restore_tree_atomic() {
    local source_tree="$1"
    local destination_tree="$2"
    local source_file relative destination_file temporary_file mode

    [[ -d "$source_tree" && -d "$destination_tree" ]] || return 1
    while IFS= read -r -d '' source_file; do
        relative="${source_file#"$source_tree"/}"
        destination_file="$destination_tree/$relative"
        temporary_file="$(dirname "$destination_file")/.$(basename "$destination_file").restore.$$.$RANDOM"
        mode="$(stat -c '%a' -- "$source_file")" || return 1
        mkdir -p -- "$(dirname "$destination_file")" || return 1
        if ! install -m "$mode" -- "$source_file" "$temporary_file"; then
            rm -f -- "$temporary_file"
            return 1
        fi
        if ! mv -f -- "$temporary_file" "$destination_file"; then
            rm -f -- "$temporary_file"
            return 1
        fi
    done < <(find "$source_tree" -type f -print0 | LC_ALL=C sort -z)

    verify_tree_parity "$source_tree" "$destination_tree"
}

rollback_files() {
    local snapshot_dir="$1"
    local install_dir="$2"
    local extension_dir="$3"
    local l2_dir="$4"
    local l3_unit_path="$5"

    restore_tree_atomic "$snapshot_dir/bin" "$install_dir" || return 1
    restore_tree_atomic "$snapshot_dir/extension" "$extension_dir" || return 1
    restore_tree_atomic "$snapshot_dir/l2" "$l2_dir" || return 1
    install_file_atomic \
        "$snapshot_dir/systemd/lay-l3-online.service" "$l3_unit_path" 0644 \
        || return 1
}

self_test() {
    local temporary_root snapshot_dir live_dir running_pid bash_binary true_binary
    temporary_root="$(mktemp -d "${TMPDIR:-/tmp}/lay-release-controller.XXXXXX")" || return 1
    snapshot_dir="$temporary_root/snapshot"
    live_dir="$temporary_root/live"
    running_pid=

    # shellcheck disable=SC2329
    cleanup_self_test() {
        if [[ -n "$running_pid" ]]; then
            kill -TERM "$running_pid" 2>/dev/null || true
            wait "$running_pid" 2>/dev/null || true
        fi
        case "$temporary_root" in
            "${TMPDIR:-/tmp}"/lay-release-controller.*)
                rm -rf -- "$temporary_root"
                ;;
        esac
    }
    trap cleanup_self_test RETURN

    bash_binary="$(type -P bash)" || return 1
    true_binary="$(type -P true)" || return 1
    mkdir -p \
        "$snapshot_dir/bin" "$snapshot_dir/extension" "$snapshot_dir/l2" \
        "$snapshot_dir/systemd" \
        "$live_dir/bin" "$live_dir/extension" "$live_dir/l2" \
        "$live_dir/systemd" || return 1

    install -m 0755 "$true_binary" "$snapshot_dir/bin/lay-daemon" || return 1
    install -m 0755 "$bash_binary" "$live_dir/bin/lay-daemon" || return 1
    printf '%s\n' rollback-extension >"$snapshot_dir/extension/metadata.json" || return 1
    printf '%s\n' forward-extension >"$live_dir/extension/metadata.json" || return 1
    chmod 0640 "$snapshot_dir/extension/metadata.json" || return 1
    chmod 0600 "$live_dir/extension/metadata.json" || return 1
    printf '%s\n' rollback-l2 >"$snapshot_dir/l2/model.bin" || return 1
    printf '%s\n' forward-l2 >"$live_dir/l2/model.bin" || return 1
    chmod 0644 "$snapshot_dir/l2/model.bin" "$live_dir/l2/model.bin" || return 1
    printf '%s\n' rollback-unit \
        >"$snapshot_dir/systemd/lay-l3-online.service" || return 1
    printf '%s\n' forward-unit \
        >"$live_dir/systemd/lay-l3-online.service" || return 1
    chmod 0644 "$snapshot_dir/systemd/lay-l3-online.service" || return 1
    chmod 0600 "$live_dir/systemd/lay-l3-online.service" || return 1

    "$live_dir/bin/lay-daemon" -c 'sleep 10' &
    running_pid=$!
    kill -0 "$running_pid" || return 1

    rollback_files \
        "$snapshot_dir" "$live_dir/bin" "$live_dir/extension" "$live_dir/l2" \
        "$live_dir/systemd/lay-l3-online.service" \
        || return 1

    kill -0 "$running_pid" || return 1
    verify_tree_parity "$snapshot_dir/bin" "$live_dir/bin" || return 1
    verify_tree_parity "$snapshot_dir/extension" "$live_dir/extension" || return 1
    verify_tree_parity "$snapshot_dir/l2" "$live_dir/l2" || return 1
    verify_file_bytes_mode \
        "$snapshot_dir/systemd/lay-l3-online.service" \
        "$live_dir/systemd/lay-l3-online.service" 644 || return 1
    [[ "$(stat -c '%a' "$live_dir/extension/metadata.json")" == 640 ]] || return 1
    cmp -s "$snapshot_dir/bin/lay-daemon" "$true_binary" || return 1

    echo "release live install controller self-test: PASS"
}

if [[ "${1:-}" == --self-test ]]; then
    [[ "$#" == 1 ]] || usage
    self_test
    exit $?
fi

RELEASE_MODE=
snapshot_argument=
if [[ "$#" == 2 && "$1" == --snapshot ]]; then
    RELEASE_MODE=forward
    snapshot_argument="$2"
elif [[ "$#" == 3 && "$1" == --repair-l11 && "$2" == --snapshot ]]; then
    RELEASE_MODE=installed-recovery
    snapshot_argument="$3"
else
    usage
fi
SNAPSHOT_DIR="$(readlink -f -- "$snapshot_argument")"
case "$SNAPSHOT_DIR" in
    "${HOME}/.local/state/lay/release-backups/1.0.65-preinstall-"*) ;;
    *)
        echo "refusing unexpected snapshot path: $SNAPSHOT_DIR" >&2
        exit 2
        ;;
esac

l11_normalize_exe_path() {
    local value="${1:?executable path required}"
    if [[ "$value" == *" (deleted)" && ! -e "$value" ]]; then
        printf '%s\n' "${value% (deleted)}"
    else
        printf '%s\n' "$value"
    fi
}

l11_read_exact_argv() {
    local pid="${1:?PID required}"
    local output_name="${2:?output array name required}"
    local -n output="$output_name"

    output=()
    [[ "$pid" =~ ^[1-9][0-9]*$ && -r "$L11_PROC_ROOT/$pid/cmdline" ]] || return 1
    mapfile -d '' -t output <"$L11_PROC_ROOT/$pid/cmdline" || return 1
    [[ "${#output[@]}" == 6 ]] || return 1
    [[ -n "${output[0]}" && "${output[1]}" == run ]] || return 1
    [[ "${output[2]}" == --memory && "${output[3]}" == /* ]] || return 1
    [[ "${output[4]}" == --socket && "${output[5]}" == /* ]] || return 1
}

l11_read_process_exe() {
    local pid="${1:?PID required}"
    local raw_exe

    raw_exe="$(readlink -- "$L11_PROC_ROOT/$pid/exe")" || return 1
    L11_OBSERVED_EXE="$(l11_normalize_exe_path "$raw_exe")" || return 1
    [[ -n "$L11_OBSERVED_EXE" ]]
}

l11_process_identity_matches() {
    local pid="${1:?PID required}"
    local expected_exe="${2:?expected executable required}"
    local expected_hash="${3:?expected executable hash required}"
    local process_hash index
    local -a observed_argv

    l11_read_exact_argv "$pid" observed_argv || return 1
    [[ "${#L11_CAPTURED_ARGV[@]}" == 6 ]] || return 1
    for index in "${!L11_CAPTURED_ARGV[@]}"; do
        [[ "${observed_argv[$index]}" == "${L11_CAPTURED_ARGV[$index]}" ]] || return 1
    done
    l11_read_process_exe "$pid" || return 1
    [[ "$L11_OBSERVED_EXE" == "$expected_exe" ]] || return 1
    process_hash="$(sha256sum "$L11_PROC_ROOT/$pid/exe" | awk '{print $1}')" \
        || return 1
    [[ "$process_hash" == "$expected_hash" ]]
}

l11_binary_hash_matches() {
    local binary="${1:?binary required}"
    local expected_hash="${2:?expected hash required}"
    local actual_hash

    [[ -f "$binary" && ! -L "$binary" ]] || return 1
    actual_hash="$(sha256sum "$binary" | awk '{print $1}')" || return 1
    [[ "$actual_hash" == "$expected_hash" ]]
}

l11_health_ready() {
    local pid="${1:?PID required}"
    local expected_exe="${2:?expected executable required}"
    local expected_hash="${3:?expected executable hash required}"
    local socket="${4:?socket required}"
    local memory="${5:?memory path required}"
    local timeout_ms="${6:-$L11_HEALTH_TIMEOUT_MS}"

    python3 "$L11_GUARD" health \
        --pid "$pid" \
        --expected-exe "$expected_exe" \
        --expected-sha256 "$expected_hash" \
        --timeout-ms "$timeout_ms" \
        --socket "$socket" \
        --package "$memory" \
        -- "${L11_CAPTURED_ARGV[@]}"
}

l11_package_unchanged() {
    local current_hash

    [[ -f "$L11_CAPTURED_MEMORY" && ! -L "$L11_CAPTURED_MEMORY" ]] || return 1
    current_hash="$(sha256sum "$L11_CAPTURED_MEMORY" | awk '{print $1}')" || return 1
    [[ "$current_hash" == "$L11_MEMORY_HASH" ]]
}

l11_capture_process() {
    local installed_exe="$INSTALL_DIR/$L11_BINARY_NAME"
    local rollback_exe="$SNAPSHOT_DIR/bin/$L11_BINARY_NAME"
    local proc_dir pid process_hash count index candidate_exe
    local -a observed_argv candidate_argv

    command -v python3 >/dev/null 2>&1 || return 1
    [[ -f "$L11_GUARD" && ! -L "$L11_GUARD" ]] || return 1
    l11_binary_hash_matches "$installed_exe" \
        "$(sha256sum "$installed_exe" | awk '{print $1}')" || return 1
    l11_binary_hash_matches "$rollback_exe" \
        "$(sha256sum "$rollback_exe" | awk '{print $1}')" || return 1
    L11_INSTALLED_HASH="$(sha256sum "$installed_exe" | awk '{print $1}')" || return 1
    L11_ROLLBACK_HASH="$(sha256sum "$rollback_exe" | awk '{print $1}')" || return 1

    count=0
    for proc_dir in "$L11_PROC_ROOT"/[0-9]*; do
        [[ -d "$proc_dir" ]] || continue
        pid="${proc_dir##*/}"
        l11_read_exact_argv "$pid" observed_argv || continue
        [[ "${observed_argv[0]}" == "$INSTALL_DIR/$L11_BINARY_NAME" \
            || "${observed_argv[0]}" == "$LINK_DIR/$L11_BINARY_NAME" ]] || continue
        l11_read_process_exe "$pid" || continue
        [[ "$L11_OBSERVED_EXE" == "$installed_exe" \
            || "$L11_OBSERVED_EXE" == "$rollback_exe" ]] || continue
        process_hash="$(sha256sum "$L11_PROC_ROOT/$pid/exe" | awk '{print $1}')" \
            || return 1
        count=$((count + 1))
        L11_CAPTURED_PID="$pid"
        L11_CAPTURED_HASH="$process_hash"
        candidate_exe="$L11_OBSERVED_EXE"
        candidate_argv=("${observed_argv[@]}")
    done
    [[ "$count" == 1 ]] || return 1
    [[ "$L11_CAPTURED_HASH" == "$L11_INSTALLED_HASH" \
        || "$L11_CAPTURED_HASH" == "$L11_ROLLBACK_HASH" ]] || return 1

    L11_CAPTURED_ARGV=("${candidate_argv[@]}")
    [[ "${#L11_CAPTURED_ARGV[@]}" == 6 ]] || return 1
    L11_CAPTURED_MEMORY="${L11_CAPTURED_ARGV[3]}"
    L11_CAPTURED_SOCKET="${L11_CAPTURED_ARGV[5]}"
    [[ -f "$L11_CAPTURED_MEMORY" && ! -L "$L11_CAPTURED_MEMORY" ]] || return 1
    L11_MEMORY_HASH="$(sha256sum "$L11_CAPTURED_MEMORY" | awk '{print $1}')" \
        || return 1
    l11_process_identity_matches \
        "$L11_CAPTURED_PID" "$candidate_exe" "$L11_CAPTURED_HASH" || return 1
    l11_health_ready \
        "$L11_CAPTURED_PID" "$candidate_exe" "$L11_CAPTURED_HASH" \
        "$L11_CAPTURED_SOCKET" "$L11_CAPTURED_MEMORY" || return 1
    l11_package_unchanged || return 1

    for index in "${!L11_CAPTURED_ARGV[@]}"; do
        [[ -n "${L11_CAPTURED_ARGV[$index]}" ]] || return 1
    done
}

l11_find_current_process() {
    local installed_exe="$INSTALL_DIR/$L11_BINARY_NAME"
    local rollback_exe="$SNAPSHOT_DIR/bin/$L11_BINARY_NAME"
    local proc_dir pid count index process_hash argv_matches
    local -a observed_argv

    L11_CURRENT_PID=
    L11_CURRENT_EXE=
    L11_CURRENT_HASH=
    count=0
    for proc_dir in "$L11_PROC_ROOT"/[0-9]*; do
        [[ -d "$proc_dir" ]] || continue
        pid="${proc_dir##*/}"
        l11_read_exact_argv "$pid" observed_argv || continue
        l11_read_process_exe "$pid" || continue
        [[ "$L11_OBSERVED_EXE" == "$installed_exe" \
            || "$L11_OBSERVED_EXE" == "$rollback_exe" ]] || continue
        [[ "${#L11_CAPTURED_ARGV[@]}" == 6 ]] || return 2
        argv_matches=1
        for index in "${!L11_CAPTURED_ARGV[@]}"; do
            if [[ "${observed_argv[$index]}" != "${L11_CAPTURED_ARGV[$index]}" ]]; then
                argv_matches=0
                break
            fi
        done
        [[ "$argv_matches" == 1 ]] || return 2
        process_hash="$(sha256sum "$L11_PROC_ROOT/$pid/exe" | awk '{print $1}')" \
            || return 2
        count=$((count + 1))
        L11_CURRENT_PID="$pid"
        L11_CURRENT_EXE="$L11_OBSERVED_EXE"
        L11_CURRENT_HASH="$process_hash"
    done
    [[ "$count" -le 1 ]] || return 2
    [[ "$count" == 1 ]]
}

l11_hash_is_known() {
    local candidate="${1:?hash required}"
    [[ "$candidate" == "$L11_CAPTURED_HASH" \
        || "$candidate" == "$L11_ROLLBACK_HASH" \
        || "$candidate" == "$L11_RELEASE_HASH" ]]
}

l11_stop_current_process() {
    local find_rc

    global_ibus_unchanged || return "$L11_UNVERIFIED_RC"
    l11_package_unchanged || return "$L11_UNVERIFIED_RC"
    if l11_find_current_process; then
        l11_hash_is_known "$L11_CURRENT_HASH" || return "$L11_UNVERIFIED_RC"
        l11_stop_process \
            "$L11_CURRENT_PID" "$L11_CURRENT_EXE" "$L11_CURRENT_HASH" \
            || return "$?"
    else
        find_rc=$?
        [[ "$find_rc" == 1 ]] || return "$L11_UNVERIFIED_RC"
    fi
    global_ibus_unchanged || return "$L11_UNVERIFIED_RC"
    l11_package_unchanged || return "$L11_UNVERIFIED_RC"
}

l11_stop_process() {
    local pid="${1:?PID required}"
    local expected_exe="${2:?expected executable required}"
    local expected_hash="${3:?expected executable hash required}"

    l11_package_unchanged || return "$L11_UNVERIFIED_RC"
    if ! python3 "$L11_GUARD" terminate \
        --pid "$pid" \
        --expected-exe "$expected_exe" \
        --expected-sha256 "$expected_hash" \
        --timeout-ms "$L11_TERM_TIMEOUT_MS" \
        -- "${L11_CAPTURED_ARGV[@]}"; then
        return "$L11_UNVERIFIED_RC"
    fi
    l11_package_unchanged || return "$L11_UNVERIFIED_RC"
}

l11_spawn_process() {
    local binary="${1:?binary required}"
    local unit pid _attempt

    unit="lay-l11-release-${BASHPID}-${RANDOM}.service"
    L11_STARTED_PID=
    L11_STARTED_UNIT=
    # $1..$7 belong to the detached bash, not this controller shell.
    # shellcheck disable=SC2016
    systemd-run --user --quiet --collect --service-type=exec \
        --unit="$unit" \
        --property=Restart=no \
        --property=TimeoutStopSec=5s \
        --property=SendSIGKILL=no \
        --property=StandardInput=null \
        --property=StandardOutput=null \
        --property=StandardError=journal \
        /bin/bash -c \
        'exec -a "$1" "$2" "$3" "$4" "$5" "$6" "$7"' _ \
        "${L11_CAPTURED_ARGV[0]}" "$binary" \
        "${L11_CAPTURED_ARGV[1]}" \
        "${L11_CAPTURED_ARGV[2]}" "${L11_CAPTURED_ARGV[3]}" \
        "${L11_CAPTURED_ARGV[4]}" "${L11_CAPTURED_ARGV[5]}" \
        || return 1
    L11_STARTED_UNIT="$unit"
    for _attempt in {1..40}; do
        if ! pid="$(systemctl --user show "$unit" -p MainPID --value 2>/dev/null)"; then
            break
        fi
        if [[ "$pid" =~ ^[1-9][0-9]*$ && -d "$L11_PROC_ROOT/$pid" ]]; then
            L11_STARTED_PID="$pid"
            return 0
        fi
        sleep 0.05
    done
    l11_cleanup_unbound_started_unit "$unit" \
        || return "$L11_UNVERIFIED_RC"
    return 1
}

l11_cleanup_unbound_started_unit() {
    local unit="${1:?started unit required}"
    local state

    [[ "$unit" =~ ^lay-l11-release-[1-9][0-9]*-[0-9]+[.]service$ ]] || return 1
    timeout 7s systemctl --user stop "$unit" >/dev/null 2>&1 || true
    state="$(timeout 2s systemctl --user is-active "$unit" 2>/dev/null || true)"
    [[ "$state" == inactive ]]
}

l11_started_unit_owns_process() {
    local pid="${1:?PID required}"
    local unit="${L11_STARTED_UNIT:?started unit required}"
    local main_pid

    [[ "$unit" =~ ^lay-l11-release-[1-9][0-9]*-[0-9]+[.]service$ ]] || return 1
    systemctl --user is-active --quiet "$unit" || return 1
    main_pid="$(systemctl --user show "$unit" -p MainPID --value)" || return 1
    [[ "$main_pid" == "$pid" ]]
}

l11_monotonic_millis() {
    local uptime seconds fraction

    read -r uptime _ </proc/uptime || return 1
    seconds="${uptime%%.*}"
    fraction="${uptime#*.}000"
    fraction="${fraction:0:3}"
    [[ "$seconds" =~ ^[0-9]+$ && "$fraction" =~ ^[0-9]{3}$ ]] || return 1
    printf '%s\n' "$((10#$seconds * 1000 + 10#$fraction))"
}

l11_wait_process_ready() {
    local pid="${1:?PID required}"
    local expected_exe="${2:?expected executable required}"
    local expected_hash="${3:?expected executable hash required}"
    local _attempt now deadline remaining attempt_timeout

    now="$(l11_monotonic_millis)" || return 1
    deadline=$((now + L11_READY_TIMEOUT_MS))
    for _attempt in {1..120}; do
        if l11_process_identity_matches "$pid" "$expected_exe" "$expected_hash"; then
            now="$(l11_monotonic_millis)" || return 1
            remaining=$((deadline - now))
            ((remaining > 0)) || return 1
            attempt_timeout="$L11_HEALTH_TIMEOUT_MS"
            ((attempt_timeout <= remaining)) || attempt_timeout="$remaining"
            if l11_health_ready \
                "$pid" "$expected_exe" "$expected_hash" \
                "$L11_CAPTURED_SOCKET" "$L11_CAPTURED_MEMORY" "$attempt_timeout" \
                && l11_package_unchanged; then
                now="$(l11_monotonic_millis)" || return 1
                ((now <= deadline)) && return 0
            fi
        fi
        [[ -d "$L11_PROC_ROOT/$pid" ]] || return 1
        now="$(l11_monotonic_millis)" || return 1
        remaining=$((deadline - now))
        ((remaining > 250)) || return 1
        sleep 0.25
    done
    return 1
}

l11_start_process() {
    local binary="${1:?binary required}"
    local expected_exe="${2:?expected executable required}"
    local expected_hash="${3:?expected executable hash required}"
    local pid spawn_rc=0

    l11_package_unchanged || return 1
    l11_binary_hash_matches "$binary" "$expected_hash" || return 1
    l11_spawn_process "$binary" || spawn_rc=$?
    [[ "$spawn_rc" == 0 ]] || return "$spawn_rc"
    pid="$L11_STARTED_PID"
    if l11_wait_process_ready "$pid" "$expected_exe" "$expected_hash" \
        && l11_started_unit_owns_process "$pid"; then
        return 0
    fi
    l11_stop_process "$pid" "$expected_exe" "$expected_hash" || return "$?"
    return 1
}

l11_transition_to_binary() {
    local binary="${1:?binary required}"
    local expected_exe="$binary"
    local expected_hash="${2:?expected executable hash required}"
    local find_rc

    global_ibus_unchanged || return 1
    l11_package_unchanged || return 1
    l11_binary_hash_matches "$binary" "$expected_hash" || return 1
    if l11_find_current_process; then
        l11_hash_is_known "$L11_CURRENT_HASH" || return "$L11_UNVERIFIED_RC"
        if [[ "$L11_CURRENT_EXE" == "$expected_exe" \
            && "$L11_CURRENT_HASH" == "$expected_hash" ]] \
            && l11_health_ready \
                "$L11_CURRENT_PID" "$expected_exe" "$expected_hash" \
                "$L11_CAPTURED_SOCKET" "$L11_CAPTURED_MEMORY"; then
            global_ibus_unchanged && l11_package_unchanged
            return
        fi
        l11_stop_process \
            "$L11_CURRENT_PID" "$L11_CURRENT_EXE" "$L11_CURRENT_HASH" || return "$?"
    else
        find_rc=$?
        [[ "$find_rc" == 1 ]] || return "$L11_UNVERIFIED_RC"
    fi
    global_ibus_unchanged || return 1
    l11_start_process "$binary" "$expected_exe" "$expected_hash" || return "$?"
    global_ibus_unchanged && l11_package_unchanged
}

# End release-local L1.1 lifecycle

dbus_version() {
    gdbus call --session \
        --dest org.gnome.Shell \
        --object-path /io/github/radislabus_star/LayDaemon \
        --method io.github.radislabus_star.LayDaemon.Version 2>/dev/null \
        | sed -n "s/.*'\([^']*\)'.*/\1/p" || true
}

dbus_ping() {
    gdbus call --session \
        --dest org.gnome.Shell \
        --object-path /io/github/radislabus_star/LayDaemon \
        --method io.github.radislabus_star.LayDaemon.Ping 2>/dev/null || true
}

managed_ime_pids() {
    pgrep -f '(^|/)lay-ibus-engine --ibus( --managed)?$' 2>/dev/null || true
}

global_ibus_pids() {
    pgrep -x ibus-daemon 2>/dev/null || true
}

global_ibus_unchanged() {
    local -a current_pids
    mapfile -t current_pids < <(global_ibus_pids)
    [[ "${#current_pids[@]}" == 1 && "${current_pids[0]}" == "$ibus_before" ]]
}

is_supported_lay_engine() {
    [[ "$1" == lay-ime-us || "$1" == lay-ime-ru ]]
}

rollback_snapshot_matches_live() {
    verify_tree_parity "$SNAPSHOT_DIR/bin" "$INSTALL_DIR" \
        && verify_tree_parity "$SNAPSHOT_DIR/extension" "$EXTENSION_DIR" \
        && verify_tree_parity "$SNAPSHOT_DIR/l2" "$L2_DIR"
}

rollback_engine_mutation_is_safe() {
    global_ibus_unchanged && rollback_snapshot_matches_live
}

current_ibus_engine() {
    local current_engine

    global_ibus_unchanged || return 1
    current_engine="$(timeout 1s ibus engine 2>/dev/null)" || return 1
    global_ibus_unchanged || return 1
    printf '%s\n' "$current_engine"
}

select_verified_xkb() {
    local candidate current_engine

    for candidate in xkb:ru::rus xkb:us::eng; do
        global_ibus_unchanged || return 1
        timeout 2s ibus engine "$candidate" >/dev/null 2>&1 || continue
        global_ibus_unchanged || return 1
        current_engine="$(current_ibus_engine)" || return 1
        [[ "$current_engine" == "$candidate" ]] && return 0
    done
    return 1
}

restore_captured_engine_if_safe() {
    local _attempt current_engine

    is_supported_lay_engine "$engine_before" || return 1
    rollback_engine_mutation_is_safe || return 1
    # The shared stop helper deliberately selects XKB before terminating the
    # managed IME. Never reactivate Lay from mixed bytes: exact snapshot parity
    # is the prerequisite for restoring the captured engine on a failure path.

    for _attempt in {1..5}; do
        rollback_engine_mutation_is_safe || return 1
        gdbus call --session \
            --dest org.gnome.Shell \
            --object-path /io/github/radislabus_star/LayDaemon \
            --method io.github.radislabus_star.LayDaemon.ActivateLayout \
            "$engine_before" >/dev/null 2>&1 || true
        rollback_engine_mutation_is_safe || return 1
        timeout 2s ibus engine "$engine_before" >/dev/null 2>&1 || true
        rollback_engine_mutation_is_safe || return 1
        current_engine="$(current_ibus_engine)" || return 1
        if [[ "$current_engine" == "$engine_before" ]] \
            && rollback_engine_mutation_is_safe; then
            return 0
        fi
        sleep 0.15
    done
    return 1
}

rollback_abort() {
    local message="${1:?rollback failure message required}"
    local exit_code="${2:-2}"
    local current_engine

    if restore_captured_engine_if_safe; then
        echo "$message; captured engine restored from verified rollback bytes" >&2
    elif select_verified_xkb \
        && current_engine="$(current_ibus_engine)" \
        && [[ "$current_engine" == xkb:ru::rus || "$current_engine" == xkb:us::eng ]]; then
        echo "$message; engine verified fail-closed on $current_engine" >&2
    else
        echo "$message; engine state UNVERIFIED" >&2
    fi
    exit "$exit_code"
}

reload_lay_extension() {
    if gnome-extensions help reload >/dev/null 2>&1; then
        gnome-extensions reload "$EXTENSION_UUID"
    else
        gnome-extensions disable "$EXTENSION_UUID" >/dev/null 2>&1 || true
        sleep 0.2
        gnome-extensions enable "$EXTENSION_UUID"
    fi
}

wait_runtime_version() {
    local expected="$1"
    local _attempt
    local -a ime_pids

    for _attempt in {1..40}; do
        mapfile -t ime_pids < <(managed_ime_pids)
        if [[ "$(dbus_version)" == "$expected" ]] \
            && [[ "$(timeout 2s ibus engine 2>/dev/null || true)" == "$engine_before" ]] \
            && [[ "${#ime_pids[@]}" == 1 ]] \
            && global_ibus_unchanged; then
            return 0
        fi
        sleep 0.25
    done
    return 1
}

capture_installed_recovery_fingerprints() {
    L11_INSTALLED_BIN_TREE_FINGERPRINT="$(tree_fingerprint "$INSTALL_DIR")" \
        || return 1
    L11_INSTALLED_EXTENSION_TREE_FINGERPRINT="$(tree_fingerprint "$EXTENSION_DIR")" \
        || return 1
    L11_INSTALLED_L2_TREE_FINGERPRINT="$(tree_fingerprint "$L2_DIR")" \
        || return 1
    L11_SNAPSHOT_BIN_TREE_FINGERPRINT="$(tree_fingerprint "$SNAPSHOT_DIR/bin")" \
        || return 1
    L11_SNAPSHOT_EXTENSION_TREE_FINGERPRINT="$(tree_fingerprint "$SNAPSHOT_DIR/extension")" \
        || return 1
    L11_SNAPSHOT_L2_TREE_FINGERPRINT="$(tree_fingerprint "$SNAPSHOT_DIR/l2")" \
        || return 1
}

installed_recovery_trees_unchanged() {
    [[ -n "$L11_INSTALLED_BIN_TREE_FINGERPRINT" \
        && "$(tree_fingerprint "$INSTALL_DIR")" == "$L11_INSTALLED_BIN_TREE_FINGERPRINT" ]] \
        || return 1
    [[ -n "$L11_INSTALLED_EXTENSION_TREE_FINGERPRINT" \
        && "$(tree_fingerprint "$EXTENSION_DIR")" == "$L11_INSTALLED_EXTENSION_TREE_FINGERPRINT" ]] \
        || return 1
    [[ -n "$L11_INSTALLED_L2_TREE_FINGERPRINT" \
        && "$(tree_fingerprint "$L2_DIR")" == "$L11_INSTALLED_L2_TREE_FINGERPRINT" ]] \
        || return 1
    [[ -n "$L11_SNAPSHOT_BIN_TREE_FINGERPRINT" \
        && "$(tree_fingerprint "$SNAPSHOT_DIR/bin")" == "$L11_SNAPSHOT_BIN_TREE_FINGERPRINT" ]] \
        || return 1
    [[ -n "$L11_SNAPSHOT_EXTENSION_TREE_FINGERPRINT" \
        && "$(tree_fingerprint "$SNAPSHOT_DIR/extension")" == "$L11_SNAPSHOT_EXTENSION_TREE_FINGERPRINT" ]] \
        || return 1
    [[ -n "$L11_SNAPSHOT_L2_TREE_FINGERPRINT" \
        && "$(tree_fingerprint "$SNAPSHOT_DIR/l2")" == "$L11_SNAPSHOT_L2_TREE_FINGERPRINT" ]]
}

installed_recovery_projection_unchanged() {
    local daemon_pid l3_pid current_engine
    local -a ime_pids

    global_ibus_unchanged || return 1
    installed_recovery_trees_unchanged || return 1
    l11_package_unchanged || return 1
    daemon_pid="$(systemctl --user show lay-daemon.service -p MainPID --value)"
    l3_pid="$(systemctl --user show lay-l3-online.service -p MainPID --value)"
    mapfile -t ime_pids < <(managed_ime_pids)
    current_engine="$(current_ibus_engine)" || return 1
    [[ "$daemon_pid" == "$daemon_before" ]] || return 1
    [[ "$l3_pid" == "$l3_before" ]] || return 1
    [[ "${#ime_pids[@]}" == 1 && "${ime_pids[0]}" == "$ime_before" ]] || return 1
    [[ "$current_engine" == "$engine_before" ]] || return 1
    [[ "$(dbus_version)" == "$EXPECTED_VERSION" ]] || return 1
    [[ "$("$LINK_DIR/lay" --version)" == "lay $EXPECTED_VERSION" ]]
}

l11_run_installed_recovery() {
    local release_transition_rc=0 snapshot_transition_rc=0

    l11_transition_to_binary \
        "$INSTALL_DIR/$L11_BINARY_NAME" "$L11_RELEASE_HASH" \
        || release_transition_rc=$?
    if [[ "$release_transition_rc" -eq 0 ]]; then
        if ! installed_recovery_projection_unchanged; then
            echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; protected projection UNVERIFIED, fallback forbidden" >&2
            return "$L11_UNVERIFIED_RC"
        fi
        if verify_process_parity; then
            if ! installed_recovery_projection_unchanged; then
                echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; post-verifier projection UNVERIFIED, fallback forbidden" >&2
                return "$L11_UNVERIFIED_RC"
            fi
            echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=PASS"
            return 0
        fi
    elif [[ "$release_transition_rc" -eq "$L11_UNVERIFIED_RC" ]]; then
        echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; L1.1 cleanup UNVERIFIED, fallback forbidden" >&2
        return "$L11_UNVERIFIED_RC"
    fi

    if ! installed_recovery_projection_unchanged; then
        echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; protected projection UNVERIFIED, fallback forbidden" >&2
        return "$L11_UNVERIFIED_RC"
    fi
    l11_transition_to_binary \
        "$SNAPSHOT_DIR/bin/$L11_BINARY_NAME" "$L11_ROLLBACK_HASH" \
        || snapshot_transition_rc=$?
    if [[ "$snapshot_transition_rc" -eq "$L11_UNVERIFIED_RC" ]]; then
        echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; L1.1 cleanup UNVERIFIED" >&2
        return "$L11_UNVERIFIED_RC"
    fi
    if ! installed_recovery_projection_unchanged; then
        echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; protected projection UNVERIFIED" >&2
        return "$L11_UNVERIFIED_RC"
    fi
    if [[ "$snapshot_transition_rc" -eq 0 ]]; then
        echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; captured service restored" >&2
    else
        echo "L11_INSTALLED_RECOVERY_${EXPECTED_VERSION//./_}=FAIL; clean no-service state" >&2
    fi
    return 2
}

verify_process_parity() {
    local daemon_pid l3_pid ime_pid l11_pid process_hash installed_hash
    local -a ime_pids

    daemon_pid="$(systemctl --user show lay-daemon.service -p MainPID --value)"
    l3_pid="$(systemctl --user show lay-l3-online.service -p MainPID --value)"
    mapfile -t ime_pids < <(managed_ime_pids)
    [[ "$daemon_pid" =~ ^[1-9][0-9]*$ ]] || return 1
    [[ "$l3_pid" =~ ^[1-9][0-9]*$ ]] || return 1
    [[ "${#ime_pids[@]}" == 1 ]] || return 1
    ime_pid="${ime_pids[0]}"
    l11_find_current_process || return 1
    l11_pid="$L11_CURRENT_PID"
    [[ "$L11_CURRENT_EXE" == "$INSTALL_DIR/$L11_BINARY_NAME" ]] || return 1
    l11_process_identity_matches \
        "$l11_pid" "$INSTALL_DIR/$L11_BINARY_NAME" "$L11_CURRENT_HASH" || return 1
    l11_health_ready \
        "$l11_pid" "$INSTALL_DIR/$L11_BINARY_NAME" "$L11_CURRENT_HASH" \
        "$L11_CAPTURED_SOCKET" "$L11_CAPTURED_MEMORY" || return 1
    l11_package_unchanged || return 1

    local specs=(
        "$daemon_pid:lay-daemon:daemon"
        "$l3_pid:lay-nanda-wave-train:l3"
        "$ime_pid:lay-ibus-engine:ime"
        "$l11_pid:lay-l1.1-serve:l11"
    )
    local spec pid installed_name label
    for spec in "${specs[@]}"; do
        IFS=: read -r pid installed_name label <<<"$spec"
        [[ "$(readlink -f "/proc/$pid/exe")" == "$INSTALL_DIR/$installed_name" ]] \
            || return 1
        process_hash="$(sha256sum "/proc/$pid/exe" | awk '{print $1}')" || return 1
        installed_hash="$(sha256sum "$INSTALL_DIR/$installed_name" | awk '{print $1}')" \
            || return 1
        [[ "$process_hash" == "$installed_hash" ]] || return 1
        printf 'process=%s pid=%s sha256=%s\n' "$label" "$pid" "$process_hash"
    done
}

verify_l3_forward_resource_contract() {
    local quota environment fragment l3_pid

    verify_file_bytes_mode "$L3_UNIT_SOURCE" "$L3_UNIT_PATH" 644 || return 1
    quota="$(systemctl --user show lay-l3-online.service \
        -p CPUQuotaPerSecUSec --value)" || return 1
    [[ "$quota" == 1.500000s ]] || return 1
    environment="$(systemctl --user show lay-l3-online.service \
        -p Environment --value)" || return 1
    [[ " $environment " == *" LAY_L3_PROOF_WORKERS=2 "* ]] || return 1
    fragment="$(systemctl --user show lay-l3-online.service \
        -p FragmentPath --value)" || return 1
    [[ "$(readlink -f -- "$fragment")" == "$(readlink -f -- "$L3_UNIT_PATH")" ]] \
        || return 1
    l3_pid="$(systemctl --user show lay-l3-online.service -p MainPID --value)" \
        || return 1
    [[ "$l3_pid" =~ ^[1-9][0-9]*$ && -r "/proc/$l3_pid/environ" ]] || return 1
    tr '\0' '\n' <"/proc/$l3_pid/environ" \
        | grep -Fqx 'LAY_L3_PROOF_WORKERS=2'
}

verify_forward() {
    local version ping spec source_name installed_name source_file installed_file
    local source_hash installed_hash installed_mode resolved_link
    local package_file package_hash package_mode sidecar_file sidecar_mode
    local sidecar_verify_dir sidecar_expected sidecar_hash
    local specs=(
        "lay:lay"
        "lay-daemon:lay-daemon"
        "lay-nanda-wave-eval:lay-nanda-wave-eval"
        "lay-nanda-wave-train:lay-nanda-wave-train"
        "lay-test-input:lay-test-input"
        "lay-ngram-corpus:lay-ngram-corpus"
        "lay-ibus-engine:lay-ibus-engine"
        "lay-memory-report:lay-memory-report"
        "lay-l11-restore:lay-l1.1-restore"
        "lay-l11-serve:lay-l1.1-serve"
    )

    global_ibus_unchanged || return 1
    systemctl --user is-active --quiet lay-daemon.service || return 1
    systemctl --user is-active --quiet lay-l3-online.service || return 1
    version="$(dbus_version)"
    [[ "$version" == "$EXPECTED_VERSION" ]] || return 1
    ping="$(dbus_ping)"
    [[ "$ping" == *"pong from lay-extension"* ]] || return 1
    [[ "$("$LINK_DIR/lay" --version)" == "lay $EXPECTED_VERSION" ]] || return 1
    [[ "$(timeout 2s ibus engine)" == "$engine_before" ]] || return 1

    for spec in "${specs[@]}"; do
        source_name="${spec%%:*}"
        installed_name="${spec#*:}"
        source_file="$ROOT/target/release/$source_name"
        installed_file="$INSTALL_DIR/$installed_name"
        source_hash="$(sha256sum "$source_file" | awk '{print $1}')" || return 1
        installed_hash="$(sha256sum "$installed_file" | awk '{print $1}')" || return 1
        installed_mode="$(stat -c '%a' "$installed_file")" || return 1
        resolved_link="$(readlink -f "$LINK_DIR/$installed_name")" || return 1
        [[ "$source_hash" == "$installed_hash" ]] || return 1
        [[ "$installed_mode" == 755 ]] || return 1
        [[ "$resolved_link" == "$installed_file" ]] || return 1
        printf 'binary=%s installed=%s sha256=%s mode=%s\n' \
            "$source_name" "$installed_name" "$installed_hash" "$installed_mode"
    done

    [[ "$(file_count "$EXTENSION_SOURCE")" == 9 ]] || return 1
    [[ "$(file_count "$EXTENSION_DIR")" == 9 ]] || return 1
    diff -qr -- "$EXTENSION_SOURCE" "$EXTENSION_DIR" >/dev/null || return 1

    # shellcheck source=scripts/l2-package-contract.sh
    # shellcheck disable=SC1091
    source "$ROOT/scripts/l2-package-contract.sh"
    package_file="$L2_DIR/$LAY_CANONICAL_L2_PACKAGE_NAME"
    package_hash="$(sha256sum "$package_file" | awk '{print $1}')" || return 1
    package_mode="$(stat -c '%a' "$package_file")" || return 1
    [[ "$package_hash" == "$LAY_CANONICAL_L2_PACKAGE_SHA256" ]] || return 1
    [[ "$(stat -c '%s' "$package_file")" == "$LAY_CANONICAL_L2_PACKAGE_BYTES" ]] \
        || return 1
    [[ "$package_mode" == 644 ]] || return 1

    sidecar_file="$L2_DIR/LAY-L2-RU-FULL-v13.dafsa"
    sidecar_mode="$(stat -c '%a' "$sidecar_file")" || return 1
    [[ -s "$sidecar_file" && "$sidecar_mode" == 644 ]] || return 1
    sidecar_verify_dir="$(mktemp -d "${TMPDIR:-/tmp}/lay-v13-sidecar-verify.XXXXXX")" \
        || return 1
    sidecar_expected="$sidecar_verify_dir/LAY-L2-RU-FULL-v13.dafsa"
    if ! "$ROOT/target/release/lay-nanda-wave-train" \
        --compile-v13-exact-sidecar "$package_file" \
        --out "$sidecar_expected" \
        || ! cmp -s "$sidecar_expected" "$sidecar_file"; then
        rm -rf -- "$sidecar_verify_dir"
        return 1
    fi
    sidecar_hash="$(sha256sum "$sidecar_file" | awk '{print $1}')" || {
        rm -rf -- "$sidecar_verify_dir"
        return 1
    }
    rm -rf -- "$sidecar_verify_dir"
    printf 'l2_package_sha256=%s sidecar_sha256=%s\n' "$package_hash" "$sidecar_hash"

    verify_l3_forward_resource_contract || return 1
    verify_process_parity || return 1
    printf 'runtime_version=%s ibus_pid=%s engine=%s ping=%s\n' \
        "$version" "$ibus_before" "$(timeout 2s ibus engine)" "$ping"
}

verify_rollback_runtime() {
    [[ "$("$LINK_DIR/lay" --version)" == "lay $ROLLBACK_VERSION" ]] || return 1
    [[ "$(dbus_version)" == "$ROLLBACK_VERSION" ]] || return 1
    [[ "$(timeout 2s ibus engine 2>/dev/null || true)" == "$engine_before" ]] || return 1
    global_ibus_unchanged || return 1
    systemctl --user is-active --quiet lay-daemon.service || return 1
    systemctl --user is-active --quiet lay-l3-online.service || return 1
    verify_file_bytes_mode \
        "$SNAPSHOT_DIR/systemd/lay-l3-online.service" "$L3_UNIT_PATH" 644 \
        || return 1
    verify_process_parity
}

run_forward_rollback() {
    local failed_forward_rc="${1:?forward failure status required}"
    local rollback_prepare_rc=0 l11_stop_rc=0 l11_start_rc=0 find_rc

    echo "FORWARD_INSTALL_${EXPECTED_VERSION//./_}=FAIL rc=$failed_forward_rc; starting atomic rollback" >&2
    systemctl --user stop lay-l3-online.service || rollback_prepare_rc=1
    "$ROOT/scripts/lay-runtime-control.sh" stop || rollback_prepare_rc=1
    select_verified_xkb || rollback_prepare_rc=1
    systemctl --user stop lay-daemon.service || rollback_prepare_rc=1
    if [[ "$rollback_prepare_rc" -ne 0 ]]; then
        rollback_abort "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL before L1.1 stop; release bytes unchanged"
    fi

    l11_stop_current_process || l11_stop_rc=$?
    if [[ "$l11_stop_rc" -eq "$L11_UNVERIFIED_RC" ]]; then
        rollback_abort \
            "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL; L1.1 stop UNVERIFIED, byte rollback forbidden" \
            "$L11_UNVERIFIED_RC"
    elif [[ "$l11_stop_rc" -ne 0 ]]; then
        rollback_abort \
            "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL before byte rollback; L1.1 stop not proved"
    fi

    rollback_files "$SNAPSHOT_DIR" "$INSTALL_DIR" "$EXTENSION_DIR" "$L2_DIR" "$L3_UNIT_PATH" \
        || rollback_prepare_rc=1
    verify_tree_parity "$SNAPSHOT_DIR/bin" "$INSTALL_DIR" || rollback_prepare_rc=1
    verify_tree_parity "$SNAPSHOT_DIR/extension" "$EXTENSION_DIR" \
        || rollback_prepare_rc=1
    verify_tree_parity "$SNAPSHOT_DIR/l2" "$L2_DIR" || rollback_prepare_rc=1
    verify_file_bytes_mode "$SNAPSHOT_DIR/systemd/lay-l3-online.service" "$L3_UNIT_PATH" 644 \
        || rollback_prepare_rc=1
    systemctl --user daemon-reload || rollback_prepare_rc=1
    global_ibus_unchanged || rollback_prepare_rc=1
    l11_package_unchanged || rollback_prepare_rc=1
    if [[ "$rollback_prepare_rc" -ne 0 ]]; then
        rollback_abort \
            "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL after L1.1 stop; bytes or projection require inspection"
    fi

    if l11_find_current_process; then
        l11_start_rc="$L11_UNVERIFIED_RC"
    else
        find_rc=$?
        [[ "$find_rc" == 1 ]] || l11_start_rc="$L11_UNVERIFIED_RC"
    fi
    if [[ "$l11_start_rc" -eq 0 ]]; then
        l11_start_process \
            "$INSTALL_DIR/$L11_BINARY_NAME" \
            "$INSTALL_DIR/$L11_BINARY_NAME" \
            "$L11_ROLLBACK_HASH" \
            || l11_start_rc=$?
    fi
    if [[ "$l11_start_rc" -eq "$L11_UNVERIFIED_RC" ]]; then
        rollback_abort \
            "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL; L1.1 snapshot state UNVERIFIED" \
            "$L11_UNVERIFIED_RC"
    elif [[ "$l11_start_rc" -ne 0 ]]; then
        rollback_abort \
            "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL; clean no-L1.1-service state"
    fi
    if ! global_ibus_unchanged || ! l11_package_unchanged; then
        rollback_abort \
            "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL; post-start projection UNVERIFIED" \
            "$L11_UNVERIFIED_RC"
    fi

    if ! reload_lay_extension; then
        rollback_abort "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during extension reload; Lay runtime requires inspection"
    fi
    if ! systemctl --user start lay-l3-online.service; then
        rollback_abort "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during L3 activation; Lay runtime requires inspection"
    fi
    if ! "$ROOT/scripts/lay-runtime-control.sh" start; then
        systemctl --user stop lay-l3-online.service || true
        rollback_abort "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during Lay activation; Lay runtime requires inspection"
    fi
    if ! restore_captured_engine_if_safe; then
        "$ROOT/scripts/lay-runtime-control.sh" stop || true
        systemctl --user stop lay-l3-online.service || true
        rollback_abort "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL restoring captured engine; Lay runtime requires inspection"
    fi
    if ! wait_runtime_version "$ROLLBACK_VERSION" || ! verify_rollback_runtime; then
        "$ROOT/scripts/lay-runtime-control.sh" stop || true
        systemctl --user stop lay-l3-online.service || true
        rollback_abort "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during runtime verification; Lay runtime requires inspection"
    fi

    echo "ROLLBACK_${ROLLBACK_VERSION//./_}=PASS" >&2
    return "$failed_forward_rc"
}

[[ -d "$SNAPSHOT_DIR" && ! -L "$SNAPSHOT_DIR" ]] || {
    echo "snapshot directory is missing or is a symlink: $SNAPSHOT_DIR" >&2
    exit 2
}
for tree in bin extension l2; do
    [[ -d "$SNAPSHOT_DIR/$tree" ]] || {
        echo "snapshot tree missing: $tree" >&2
        exit 2
    }
done
[[ -d "$SNAPSHOT_DIR/systemd" && ! -L "$SNAPSHOT_DIR/systemd" ]] || {
    echo "snapshot systemd directory is missing or is a symlink" >&2
    exit 2
}
[[ -f "$SNAPSHOT_DIR/systemd/lay-l3-online.service" \
    && ! -L "$SNAPSHOT_DIR/systemd/lay-l3-online.service" ]] || {
    echo "snapshot L3 unit is missing or is a symlink" >&2
    exit 2
}
[[ -f "$L3_UNIT_PATH" && ! -L "$L3_UNIT_PATH" ]] || {
    echo "installed L3 unit is missing or is a symlink" >&2
    exit 2
}

[[ "$(file_count "$INSTALL_DIR")" == 19 ]] || {
    echo "unexpected installed binary-tree file count" >&2
    exit 2
}
[[ "$(file_count "$EXTENSION_DIR")" == 9 ]] || {
    echo "unexpected installed extension file count" >&2
    exit 2
}
[[ "$(file_count "$L2_DIR")" == 7 ]] || {
    echo "unexpected installed L2 file count" >&2
    exit 2
}
if [[ "$RELEASE_MODE" == forward ]]; then
    verify_tree_parity "$SNAPSHOT_DIR/bin" "$INSTALL_DIR" || {
        echo "binary snapshot does not match live baseline" >&2
        exit 2
    }
    verify_tree_parity "$SNAPSHOT_DIR/extension" "$EXTENSION_DIR" || {
        echo "extension snapshot does not match live baseline" >&2
        exit 2
    }
    verify_tree_parity "$SNAPSHOT_DIR/l2" "$L2_DIR" || {
        echo "L2 snapshot does not match live baseline" >&2
        exit 2
    }
    verify_file_bytes_mode "$SNAPSHOT_DIR/systemd/lay-l3-online.service" \
        "$L3_UNIT_PATH" 644 || {
        echo "L3 unit snapshot does not match live baseline" >&2
        exit 2
    }
    [[ "$("$ROOT/target/release/lay" --version)" == "lay $EXPECTED_VERSION" ]] || {
        echo "release build version is not $EXPECTED_VERSION" >&2
        exit 2
    }
    [[ "$("$LINK_DIR/lay" --version)" == "lay $ROLLBACK_VERSION" ]] || {
        echo "installed baseline is not $ROLLBACK_VERSION" >&2
        exit 2
    }
    [[ "$(dbus_version)" == "$ROLLBACK_VERSION" ]] || {
        echo "loaded extension baseline is not $ROLLBACK_VERSION" >&2
        exit 2
    }
    L11_RELEASE_HASH="$(sha256sum "$ROOT/target/release/lay-l11-serve" | awk '{print $1}')" \
        || exit 2
    l11_binary_hash_matches "$ROOT/target/release/lay-l11-serve" "$L11_RELEASE_HASH" \
        || {
            echo "release L1.1 binary is missing or invalid" >&2
            exit 2
        }
else
    [[ "$("$SNAPSHOT_DIR/bin/lay" --version)" == "lay $ROLLBACK_VERSION" ]] || {
        echo "snapshot baseline is not $ROLLBACK_VERSION" >&2
        exit 2
    }
    [[ "$("$LINK_DIR/lay" --version)" == "lay $EXPECTED_VERSION" ]] || {
        echo "installed recovery baseline is not $EXPECTED_VERSION" >&2
        exit 2
    }
    [[ "$(dbus_version)" == "$EXPECTED_VERSION" ]] || {
        echo "loaded recovery baseline is not $EXPECTED_VERSION" >&2
        exit 2
    }
    L11_RELEASE_HASH="$(sha256sum "$INSTALL_DIR/$L11_BINARY_NAME" | awk '{print $1}')" \
        || exit 2
fi
systemctl --user is-active --quiet lay-daemon.service || {
    echo "Lay daemon must be active before release" >&2
    exit 2
}
systemctl --user is-active --quiet lay-l3-online.service || {
    echo "Lay L3 service must be active before release" >&2
    exit 2
}

mapfile -t ibus_pids < <(global_ibus_pids)
[[ "${#ibus_pids[@]}" == 1 ]] || {
    echo "exactly one live global ibus-daemon is required" >&2
    exit 2
}
ibus_before="${ibus_pids[0]}"
[[ "$ibus_before" =~ ^[1-9][0-9]*$ && -d "/proc/$ibus_before" ]] || {
    echo "global ibus-daemon PID is invalid" >&2
    exit 2
}
engine_before="$(timeout 2s ibus engine 2>/dev/null || true)"
is_supported_lay_engine "$engine_before" || {
    echo "selected engine must be lay-ime-us or lay-ime-ru before release: ${engine_before:-<none>}" >&2
    exit 2
}

l11_capture_process || {
    echo "L1.1 process identity, health, or rollback binding is not exact" >&2
    exit 2
}

if [[ "$RELEASE_MODE" == installed-recovery ]]; then
    capture_installed_recovery_fingerprints || {
        echo "installed recovery could not capture immutable tree fingerprints" >&2
        exit "$L11_UNVERIFIED_RC"
    }
    daemon_before="$(systemctl --user show lay-daemon.service -p MainPID --value)"
    l3_before="$(systemctl --user show lay-l3-online.service -p MainPID --value)"
    mapfile -t ime_pids_before < <(managed_ime_pids)
    [[ "$daemon_before" =~ ^[1-9][0-9]*$ \
        && "$l3_before" =~ ^[1-9][0-9]*$ \
        && "${#ime_pids_before[@]}" == 1 ]] || {
        echo "installed recovery requires exact daemon, L3, and managed IME PIDs" >&2
        exit 2
    }
    ime_before="${ime_pids_before[0]}"
    l11_run_installed_recovery
    exit $?
fi

(
    set -euo pipefail
    cd "$ROOT"
    scripts/install-release-binaries.sh
    l11_transition_to_binary "$INSTALL_DIR/$L11_BINARY_NAME" "$L11_RELEASE_HASH"
    scripts/check-gnome-extension-runtime.sh --fix --reload
    install_file_atomic "$L3_UNIT_SOURCE" "$L3_UNIT_PATH" 0644
    systemctl --user daemon-reload
    systemctl --user restart lay-l3-online.service
    scripts/lay-runtime-control.sh restart
    wait_runtime_version "$EXPECTED_VERSION"
    verify_forward
)
forward_rc=$?

if [[ "$forward_rc" -eq 0 ]]; then
    echo "FORWARD_INSTALL_${EXPECTED_VERSION//./_}=PASS"
    exit 0
fi

if [[ "$forward_rc" -eq "$L11_UNVERIFIED_RC" ]]; then
    echo "FORWARD_INSTALL_${EXPECTED_VERSION//./_}=FAIL; L1.1 cleanup UNVERIFIED, rollback mutation forbidden" >&2
    exit "$L11_UNVERIFIED_RC"
fi

run_forward_rollback "$forward_rc"
exit $?
