#!/bin/sh
# shellcheck disable=SC2029
set -eu

remote=${LAY_PROOF_REMOTE:-e@192.168.3.94}
run_id=${1:-v17-bwrap-full-route}
scope=${2:-route}

case "$run_id" in
    *[!A-Za-z0-9._-]*|'')
        echo "run id must contain only ASCII letters, digits, dot, underscore or dash" >&2
        exit 64
        ;;
esac
case "$scope" in
    route)
        config_name=trace-enabled-config.json
        ;;
    undo)
        config_name=autocorrect-proof-config.json
        ;;
    *)
        echo "proof scope must be route or undo" >&2
        exit 64
        ;;
esac

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
remote_root=/home/e/projects/lay-atomic-full-route-20260821

ssh "$remote" "mkdir -p '$remote_root/runtime/bin' '$remote_root/runtime/config' \
    '$remote_root/runtime/share/gnome-shell/modes' \
    '$remote_root/runtime/engines' '$remote_root/runtime/manifests' \
    '$remote_root/logs' '$remote_root/output'"
scp -q \
    "$script_dir/capture-process-exit.sh" \
    "$script_dir/launch-patched-ibus-daemon.sh" \
    "$script_dir/prelaunch-patched-ibus-daemon.sh" \
    "$script_dir/reuse-prelaunched-ibus-daemon.sh" \
    "$script_dir/verify-atomic-proof-engine.sh" \
    "$remote:$remote_root/runtime/bin/"
scp -q "$script_dir/$config_name" \
    "$remote:$remote_root/runtime/config/trace-enabled.json"
scp -q "$script_dir/lay-proof-mode.json" \
    "$remote:$remote_root/runtime/share/gnome-shell/modes/lay-proof.json"
ssh "$remote" "cp '$remote_root/runtime/bin/reuse-prelaunched-ibus-daemon.sh' \
    '$remote_root/runtime/bin/ibus-daemon' && \
    chmod 0755 '$remote_root/runtime/bin/'*.sh '$remote_root/runtime/bin/ibus-daemon'"

ssh "$remote" "LAY_PROOF_RUN_ID='$run_id' LAY_PROOF_SCOPE='$scope' bash -s" <<'REMOTE'
set -eu

proof=/home/e/projects/lay-atomic-full-route-20260821
mutter=/home/e/lay-proof/mutter-slice3a-20260821
rootfs=$mutter/rootfs
bwrap=/usr/bin/bwrap
shell=/home/e/projects/gnome-shell-lay-atomic-20260821
ibus_build=/home/e/projects/ibus-lay-atomic-epoch-20260821/build-normal-dynamic
models=/home/e/.local/share/lay/nanda_wave
run_id=${LAY_PROOF_RUN_ID:?LAY_PROOF_RUN_ID is required}
scope=${LAY_PROOF_SCOPE:?LAY_PROOF_SCOPE is required}

active_engine=$proof/runtime/manifests/active-engine
[ -f "$active_engine" ] || {
    echo "atomic proof engine has not been staged" >&2
    exit 66
}
engine_id=$(sed -n '1p' "$active_engine")
[ "$(wc -l <"$active_engine")" -eq 1 ] || {
    echo "atomic proof engine selector must contain exactly one line" >&2
    exit 65
}
case "$engine_id" in
    *[!0-9a-f]*|'')
        echo "atomic proof engine selector is invalid" >&2
        exit 65
        ;;
esac
[ "${#engine_id}" -eq 64 ] || {
    echo "atomic proof engine selector has the wrong length" >&2
    exit 65
}
engine_dir=$proof/runtime/engines/$engine_id
engine=$engine_dir/lay-ibus-engine
admitted_engine_sha=$(
    "$proof/runtime/bin/verify-atomic-proof-engine.sh" "$engine_dir" "$engine_id"
)

log=$proof/logs/atomic-full-route-$run_id.log
metrics=$proof/output/atomic-full-route-$run_id.metrics
hashes=$proof/output/atomic-full-route-$run_id.sha256
cleanup=$proof/output/atomic-full-route-$run_id.cleanup
trace=$proof/output/atomic-full-route-$run_id.engine-trace.jsonl
trace_tmp=$(mktemp /dev/shm/lay-atomic-engine-trace.XXXXXX)
socket=$proof/output/ibus-lay-proof.sock
trap 'rm -f -- "$trace_tmp"' EXIT HUP INT TERM

for required in \
    "$bwrap" \
    "$shell/build/src/gnome-shell-test-tool" \
    "$shell/source/tests/shell/atomicInputMethodRoute.js" \
    "$ibus_build/bus/ibus-daemon" \
    "$engine" \
    "$engine_dir/lay-ibus-engine.sha256" \
    "$engine_dir/source-files.sha256" \
    "$engine_dir/provenance.env" \
    "$proof/runtime/config/trace-enabled.json" \
    "$proof/runtime/share/gnome-shell/modes/lay-proof.json" \
    "$proof/runtime/components/lay.xml" \
    "$rootfs/usr/share/ibus/component/simple.xml" \
    "$rootfs/usr/bin/ibus"; do
    if [ ! -e "$required" ]; then
        printf 'LAY_PROOF_PREREQUISITE_MISSING path=%s\n' "$required" >&2
        exit 66
    fi
done

grep -Fq '<name>lay-ime-ru</name>' "$proof/runtime/components/lay.xml"
grep -Fq '<name>xkb:us::eng</name>' "$rootfs/usr/share/ibus/component/simple.xml"
grep -Fq '"parentMode": "user"' \
    "$proof/runtime/share/gnome-shell/modes/lay-proof.json"
grep -Fq '"components": []' \
    "$proof/runtime/share/gnome-shell/modes/lay-proof.json"

rm -f -- "$socket" "$log" "$metrics" "$hashes" "$cleanup" "$trace"

{
    sha256sum \
        "$engine" \
        "$engine_dir/lay-ibus-engine.sha256" \
        "$engine_dir/source-files.sha256" \
        "$engine_dir/provenance.env" \
        "$ibus_build/bus/.libs/ibus-daemon" \
        "$shell/build/src/gnome-shell" \
        "$shell/source/tests/shell/atomicInputMethodRoute.js" \
        "$proof/runtime/config/trace-enabled.json" \
        "$proof/runtime/share/gnome-shell/modes/lay-proof.json" \
        "$proof/runtime/components/lay.xml" \
        "$rootfs/usr/share/ibus/component/simple.xml" \
        "$rootfs/usr/bin/ibus"
    find "$mutter/build-normal/src" -maxdepth 1 -type f \
        -name 'libmutter-*.so.*' -print | sort | xargs -r sha256sum
    find "$models" -maxdepth 2 -type f -print | sort | xargs -r sha256sum
} >"$hashes"

scan_owned_processes() {
    destination=$1
    : >"$destination"
    for proc in /proc/[0-9]*; do
        pid=${proc##*/}
        [ "$pid" = "$$" ] && continue
        [ -r "$proc/cmdline" ] || continue
        command=$(tr '\000' ' ' <"$proc/cmdline" 2>/dev/null || true)
        owned=false
        case "$command" in
            *'/proof-runtime/'*|*'/proof-output/ibus-lay-proof.sock'*|*'/lay-engine/lay-ibus-engine'*|*'/shell-source/tests/shell/atomicInputMethodRoute.js'*)
                owned=true
                ;;
        esac
        if [ "$owned" = false ] && [ -r "$proc/environ" ] &&
                tr '\000' '\n' <"$proc/environ" 2>/dev/null |
                    grep -Fxq 'MUTTER_TEST_LOG_DIR=/proof-output'; then
            owned=true
        fi
        if [ "$owned" = true ]; then
            printf '%s\t%s\n' "$pid" "$command" >>"$destination"
        fi
    done
}

set +e
timeout --signal=TERM --kill-after=5s 90s "$bwrap" --die-with-parent --new-session \
    --bind "$rootfs" / --dev /dev --proc /proc \
    --bind "$trace_tmp" /dev/shm/lay-atomic-engine-trace.jsonl \
    --ro-bind /etc/passwd /etc/passwd \
    --ro-bind /etc/group /etc/group \
    --ro-bind /etc/nsswitch.conf /etc/nsswitch.conf \
    --ro-bind /etc/hosts /etc/hosts \
    --ro-bind /etc/resolv.conf /etc/resolv.conf \
    --ro-bind "$mutter/source" /workspace \
    --bind "$mutter/build-normal" /build \
    --ro-bind "$shell/source" /shell-source \
    --bind "$shell/build" /shell-build \
    --ro-bind "$shell/pkgconfig" /shell-pc \
    --ro-bind "$ibus_build" /ibus-build \
    --ro-bind "$engine_dir" /lay-engine \
    --ro-bind "$proof/runtime" /proof-runtime \
    --bind "$proof/output" /proof-output \
    --bind "$proof/logs" /proof-logs \
    --ro-bind "$models" /proof-model/nanda_wave \
    --chdir /build \
    --setenv PATH /proof-runtime/bin:/usr/bin:/bin \
    --setenv PYTHONPATH /workspace/src/tests \
    --setenv GSETTINGS_BACKEND memory \
    --setenv GSETTINGS_SCHEMA_DIR /proof-runtime/schemas \
    --setenv G_DEBUG fatal-criticals \
    --setenv G_MESSAGES_DEBUG 'GNOME Shell' \
    --setenv MUTTER_DEBUG 'input,input-events' \
    --setenv GNOME_SHELL_SESSION_MODE lay-proof \
    --setenv GNOME_SHELL_BUILDDIR /shell-build/src \
    --setenv GNOME_SHELL_DATADIR /shell-build/data \
    --setenv XDG_DATA_DIRS /proof-runtime/share:/usr/share \
    --setenv SHELL_BACKGROUND_IMAGE /shell-source/tests/data/background.png \
    --setenv GI_TYPELIB_PATH /shell-build/subprojects/gvc:/shell-build/subprojects/libshew/src:/shell-build/src:/shell-build/src/st:/build/mtk/mtk:/build/clutter/clutter:/build/cogl/cogl:/build/src:/build/src/tests \
    --setenv LD_LIBRARY_PATH /shell-build/src/st:/shell-build/src:/shell-build/subprojects/gvc:/build/mtk/mtk:/build/clutter/clutter:/build/cogl/cogl:/build/src \
    --setenv META_DBUS_RUNNER_DISABLE_UMOCKDEV 1 \
    --setenv META_DBUS_RUNNER_DISABLE_LOGIND_PASSTHROUGH 1 \
    --setenv MUTTER_TEST_LOG_DIR /proof-output \
    --setenv IBUS_ADDRESS unix:path=/proof-output/ibus-lay-proof.sock \
    --setenv IBUS_COMPONENT_PATH /proof-runtime/components:/usr/share/ibus/component \
    --setenv LAY_PROOF_IBUS_DAEMON /ibus-build/bus/.libs/ibus-daemon \
    --setenv LAY_PROOF_IBUS_LIBDIR /ibus-build/src/.libs \
    --setenv LAY_PROOF_READY_SOCKET /proof-output/ibus-lay-proof.sock \
    --setenv LAY_CONFIG_PATH /proof-runtime/config/trace-enabled.json \
    --setenv LAY_ATOMIC_PROOF_SCOPE "$scope" \
    --setenv LAY_IBUS_TRACE_PATH /dev/shm/lay-atomic-engine-trace.jsonl \
    --setenv LAY_L2_LEXICAL_PHASE_MEMORY /proof-model/nanda_wave/l2_lexical_phase_v2.bin \
    --setenv LAY_NANDA_L3_CONTEXT_MEMORY /proof-model/nanda_wave/l3_context_phase.nwpc \
    --setenv LAY_NANDA_L2_PHASE_MEMORY /proof-model/nanda_wave/l2_candidate_phase.nwpc \
    --setenv LAY_L2_MODEL_DIR /proof-model/nanda_wave/l2 \
    /shell-build/tests/gnome-shell-dbus-runner.py \
        --launch /proof-runtime/bin/prelaunch-patched-ibus-daemon.sh -- \
        /shell-build/src/gnome-shell-test-tool --headless \
        --wrap /proof-runtime/bin/capture-process-exit.sh \
        /shell-source/tests/shell/atomicInputMethodRoute.js >"$log" 2>&1
status=$?
set -e

cp "$trace_tmp" "$trace"

rm -f -- "$socket"
sleep 1
scan_owned_processes "$cleanup.before"
leftovers_before=$(wc -l <"$cleanup.before")

if [ "$leftovers_before" -ne 0 ]; then
    while IFS="$(printf '\t')" read -r pid _; do
        kill -TERM "$pid" 2>/dev/null || true
    done <"$cleanup.before"
    sleep 1
    while IFS="$(printf '\t')" read -r pid _; do
        kill -KILL "$pid" 2>/dev/null || true
    done <"$cleanup.before"
fi

scan_owned_processes "$cleanup.after"
leftovers_after=$(wc -l <"$cleanup.after")
{
    printf 'status=%s\n' "$status"
    printf 'leftovers_before_cleanup=%s\n' "$leftovers_before"
    printf 'leftovers_after_cleanup=%s\n' "$leftovers_after"
    printf '%s\n' '-- before cleanup --'
    cat "$cleanup.before"
    printf '%s\n' '-- after cleanup --'
    cat "$cleanup.after"
} >"$cleanup"
rm -f -- "$cleanup.before" "$cleanup.after"

metric_count=$(grep -c '^.*LAY_ATOMIC_INTEGRATED ' "$log" || true)
paired_contract_count=$(grep -c 'LAY_ATOMIC_INTEGRATED .*paired_releases=514 release_rpcs=0 native_releases=0 ledger_entries=0' "$log" || true)
double_shift_contract_count=$(grep -c 'LAY_ATOMIC_DOUBLE_SHIFT_COMPOSITE input=ghbdtn autocorrect=привет restored=ghbdtn paired_releases=7 release_rpcs=2 native_releases=1 ledger_entries=0 status=PASS' "$log" || true)
service_log_guest=$(sed -n 's/.*log file: \(\/proof-output\/session-[^)]*prelaunch-patched-ibus-daemon.sh.log\)).*/\1/p' "$log" | head -1)
service_log=$proof/output/$(basename "$service_log_guest")
registry_count=0
if [ -f "$service_log" ]; then
    registry_count=$(grep -c 'LAY_PROOF_IBUS_REGISTRY_READY ' "$service_log" || true)
fi
reuse_count=$(grep -c 'LAY_PROOF_IBUS_REUSED ' "$log" || true)
process_exit_zero=$(grep -c 'LAY_PROOF_PROCESS_EXIT status=0' "$log" || true)
forbidden_count=$(grep -Ec 'Script failed|Cannot find engine|current session already has an ibus-daemon|LAY_PROOF_IBUS_REGISTRY_REFUSED' "$log" || true)
metric=$(grep 'LAY_ATOMIC_INTEGRATED ' "$log" | tail -1 || true)
trace_lines=$(wc -l <"$trace" 2>/dev/null || printf '0')
legacy_key_calls=$(grep -c '"kind":"ibus_legacy_key_blocked"' "$trace" 2>/dev/null || true)
engine_key_events=$(grep -c '"kind":"ibus_key"' "$trace" 2>/dev/null || true)

case "$scope" in
    route)
        expected_metric_count=1
        expected_paired_contract_count=1
        expected_double_shift_contract_count=0
        minimum_engine_key_events=514
        ;;
    undo)
        expected_metric_count=0
        expected_paired_contract_count=0
        expected_double_shift_contract_count=1
        minimum_engine_key_events=7
        ;;
esac

{
    printf 'proof_scope=%s\n' "$scope"
    printf 'process_status=%s\n' "$status"
    printf 'integrated_marker_count=%s\n' "$metric_count"
    printf 'paired_release_contract_count=%s\n' "$paired_contract_count"
    printf 'double_shift_contract_count=%s\n' "$double_shift_contract_count"
    printf 'registry_ready_count=%s\n' "$registry_count"
    printf 'reuse_count=%s\n' "$reuse_count"
    printf 'process_exit_zero_count=%s\n' "$process_exit_zero"
    printf 'forbidden_marker_count=%s\n' "$forbidden_count"
    printf 'engine_trace_lines=%s\n' "$trace_lines"
    printf 'engine_key_events=%s\n' "$engine_key_events"
    printf 'legacy_key_calls=%s\n' "$legacy_key_calls"
    printf 'proof_engine_sha256=%s\n' "$admitted_engine_sha"
    printf 'proof_engine_source_manifest_sha256=%s\n' \
        "$(sha256sum "$engine_dir/source-files.sha256" | awk '{print $1}')"
    printf 'leftovers_before_cleanup=%s\n' "$leftovers_before"
    printf 'leftovers_after_cleanup=%s\n' "$leftovers_after"
    printf '%s\n' "$metric"
} >"$metrics"

cat "$metrics"
printf 'log=%s\nmetrics=%s\nhashes=%s\ncleanup=%s\ntrace=%s\nservice_log=%s\n' \
    "$log" "$metrics" "$hashes" "$cleanup" "$trace" "$service_log"

if [ "$status" -ne 0 ] || \
        [ "$metric_count" -ne "$expected_metric_count" ] || \
        [ "$paired_contract_count" -ne "$expected_paired_contract_count" ] || \
        [ "$double_shift_contract_count" -ne "$expected_double_shift_contract_count" ] || \
        [ "$registry_count" -lt 1 ] || [ "$reuse_count" -lt 1 ] || \
        [ "$process_exit_zero" -ne 1 ] || [ "$forbidden_count" -ne 0 ] || \
        [ "$trace_lines" -lt 1 ] || \
        [ "$engine_key_events" -lt "$minimum_engine_key_events" ] || \
        [ "$legacy_key_calls" -ne 0 ] || \
        [ "$leftovers_before" -ne 0 ] || [ "$leftovers_after" -ne 0 ]; then
    tail -160 "$log" >&2
    exit 1
fi
REMOTE
