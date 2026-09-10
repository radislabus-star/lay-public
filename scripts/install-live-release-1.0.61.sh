#!/usr/bin/env bash
set -u -o pipefail

EXPECTED_VERSION=1.0.61
ROLLBACK_VERSION=1.0.60
EXTENSION_UUID=lay@radislabus-star.github.io
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${HOME}/.local/lib/lay/bin"
LINK_DIR="${HOME}/.local/bin"
EXTENSION_SOURCE="$ROOT/extension/$EXTENSION_UUID"
EXTENSION_DIR="${HOME}/.local/share/gnome-shell/extensions/$EXTENSION_UUID"
L2_DIR="${HOME}/.local/share/lay/nanda_wave/l2"

usage() {
    echo "usage: $0 --snapshot ABSOLUTE_SNAPSHOT_DIR | --self-test" >&2
    exit 2
}

file_count() {
    find "$1" -type f -printf '.' | wc -c
}

tree_modes() {
    find "$1" -type f -printf '%P\t%m\n' | LC_ALL=C sort
}

verify_tree_parity() {
    local source_tree="$1"
    local destination_tree="$2"

    [[ -d "$source_tree" && -d "$destination_tree" ]] || return 1
    diff -qr -- "$source_tree" "$destination_tree" >/dev/null || return 1
    cmp -s <(tree_modes "$source_tree") <(tree_modes "$destination_tree")
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

    restore_tree_atomic "$snapshot_dir/bin" "$install_dir" || return 1
    restore_tree_atomic "$snapshot_dir/extension" "$extension_dir" || return 1
    restore_tree_atomic "$snapshot_dir/l2" "$l2_dir" || return 1
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
        "$live_dir/bin" "$live_dir/extension" "$live_dir/l2" || return 1

    install -m 0755 "$true_binary" "$snapshot_dir/bin/lay-daemon" || return 1
    install -m 0755 "$bash_binary" "$live_dir/bin/lay-daemon" || return 1
    printf '%s\n' rollback-extension >"$snapshot_dir/extension/metadata.json" || return 1
    printf '%s\n' forward-extension >"$live_dir/extension/metadata.json" || return 1
    chmod 0640 "$snapshot_dir/extension/metadata.json" || return 1
    chmod 0600 "$live_dir/extension/metadata.json" || return 1
    printf '%s\n' rollback-l2 >"$snapshot_dir/l2/model.bin" || return 1
    printf '%s\n' forward-l2 >"$live_dir/l2/model.bin" || return 1
    chmod 0644 "$snapshot_dir/l2/model.bin" "$live_dir/l2/model.bin" || return 1

    "$live_dir/bin/lay-daemon" -c 'sleep 10' &
    running_pid=$!
    kill -0 "$running_pid" || return 1

    rollback_files \
        "$snapshot_dir" "$live_dir/bin" "$live_dir/extension" "$live_dir/l2" \
        || return 1

    kill -0 "$running_pid" || return 1
    verify_tree_parity "$snapshot_dir/bin" "$live_dir/bin" || return 1
    verify_tree_parity "$snapshot_dir/extension" "$live_dir/extension" || return 1
    verify_tree_parity "$snapshot_dir/l2" "$live_dir/l2" || return 1
    [[ "$(stat -c '%a' "$live_dir/extension/metadata.json")" == 640 ]] || return 1
    cmp -s "$snapshot_dir/bin/lay-daemon" "$true_binary" || return 1

    echo "release live install controller self-test: PASS"
}

if [[ "${1:-}" == --self-test ]]; then
    [[ "$#" == 1 ]] || usage
    self_test
    exit $?
fi

[[ "$#" == 2 && "$1" == --snapshot ]] || usage
SNAPSHOT_DIR="$(readlink -f -- "$2")"
case "$SNAPSHOT_DIR" in
    "${HOME}/.local/state/lay/release-backups/1.0.61-preinstall-"*) ;;
    *)
        echo "refusing unexpected snapshot path: $SNAPSHOT_DIR" >&2
        exit 2
        ;;
esac

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
            && [[ "$(timeout 2s ibus engine 2>/dev/null || true)" == lay-ime-ru ]] \
            && [[ "${#ime_pids[@]}" == 1 ]] \
            && global_ibus_unchanged; then
            return 0
        fi
        sleep 0.25
    done
    return 1
}

verify_process_parity() {
    local daemon_pid l3_pid ime_pid process_hash installed_hash
    local -a ime_pids

    daemon_pid="$(systemctl --user show lay-daemon.service -p MainPID --value)"
    l3_pid="$(systemctl --user show lay-l3-online.service -p MainPID --value)"
    mapfile -t ime_pids < <(managed_ime_pids)
    [[ "$daemon_pid" =~ ^[1-9][0-9]*$ ]] || return 1
    [[ "$l3_pid" =~ ^[1-9][0-9]*$ ]] || return 1
    [[ "${#ime_pids[@]}" == 1 ]] || return 1
    ime_pid="${ime_pids[0]}"

    local specs=(
        "$daemon_pid:lay-daemon:daemon"
        "$l3_pid:lay-nanda-wave-train:l3"
        "$ime_pid:lay-ibus-engine:ime"
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
    [[ "$(timeout 2s ibus engine)" == lay-ime-ru ]] || return 1

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
    verify_process_parity
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
[[ "$engine_before" == lay-ime-ru ]] || {
    echo "lay-ime-ru must be selected before release" >&2
    exit 2
}

(
    set -euo pipefail
    cd "$ROOT"
    scripts/install-release-binaries.sh
    scripts/check-gnome-extension-runtime.sh --fix --reload
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

echo "FORWARD_INSTALL_${EXPECTED_VERSION//./_}=FAIL rc=$forward_rc; starting atomic rollback" >&2
rollback_prepare_rc=0
systemctl --user stop lay-l3-online.service || rollback_prepare_rc=1
"$ROOT/scripts/lay-runtime-control.sh" stop || rollback_prepare_rc=1
systemctl --user stop lay-daemon.service || rollback_prepare_rc=1
rollback_files "$SNAPSHOT_DIR" "$INSTALL_DIR" "$EXTENSION_DIR" "$L2_DIR" \
    || rollback_prepare_rc=1
verify_tree_parity "$SNAPSHOT_DIR/bin" "$INSTALL_DIR" || rollback_prepare_rc=1
verify_tree_parity "$SNAPSHOT_DIR/extension" "$EXTENSION_DIR" || rollback_prepare_rc=1
verify_tree_parity "$SNAPSHOT_DIR/l2" "$L2_DIR" || rollback_prepare_rc=1
global_ibus_unchanged || rollback_prepare_rc=1

if [[ "$rollback_prepare_rc" -ne 0 ]]; then
    echo "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL before reactivation; Lay runtime remains stopped or requires inspection" >&2
    exit 2
fi

if ! reload_lay_extension; then
    echo "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during extension reload; Lay runtime remains stopped" >&2
    exit 2
fi
if ! systemctl --user start lay-l3-online.service; then
    echo "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during L3 activation; Lay runtime remains stopped" >&2
    exit 2
fi
if ! "$ROOT/scripts/lay-runtime-control.sh" start; then
    systemctl --user stop lay-l3-online.service || true
    echo "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during Lay activation; managed runtime was stopped again" >&2
    exit 2
fi
if ! wait_runtime_version "$ROLLBACK_VERSION" || ! verify_rollback_runtime; then
    "$ROOT/scripts/lay-runtime-control.sh" stop || true
    systemctl --user stop lay-l3-online.service || true
    echo "ROLLBACK_${ROLLBACK_VERSION//./_}=FAIL during runtime verification; managed runtime was stopped again" >&2
    exit 2
fi

echo "ROLLBACK_${ROLLBACK_VERSION//./_}=PASS" >&2
exit "$forward_rc"
