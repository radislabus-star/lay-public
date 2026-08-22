#!/bin/sh
set -u

child_pid=

terminate_child() {
    trap - HUP INT TERM
    if [ -n "$child_pid" ]; then
        kill -TERM "$child_pid" 2>/dev/null || true
        wait "$child_pid" 2>/dev/null || true
    fi
    exit 143
}

trap terminate_child HUP INT TERM

if [ -n "${LAY_PROOF_READY_SOCKET:-}" ]; then
    attempts=0
    while [ ! -S "$LAY_PROOF_READY_SOCKET" ]; do
        attempts=$((attempts + 1))
        if [ "$attempts" -ge 500 ]; then
            printf 'LAY_PROOF_READY_TIMEOUT socket=%s\n' \
                "$LAY_PROOF_READY_SOCKET" >&2
            exit 70
        fi
        sleep 0.01
    done
    printf 'LAY_PROOF_READY socket=%s attempts=%s\n' \
        "$LAY_PROOF_READY_SOCKET" "$attempts" >&2
fi

printf 'LAY_PROOF_PROCESS_START cwd=%s command=%s\n' "$PWD" "$1" >&2
"$@" &
child_pid=$!
wait "$child_pid"
status=$?
child_pid=
printf 'LAY_PROOF_PROCESS_EXIT status=%s\n' "$status" >&2
exit "$status"
