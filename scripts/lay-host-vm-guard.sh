#!/usr/bin/env bash
# Pause the host lay-daemon while a VM viewer is running.
#
# This is useful when testing lay inside a guest desktop: the host and guest can
# both see the same physical double-Shift and both replay text into the VM
# window. The guard keeps only the guest daemon active while a known viewer runs.
set -euo pipefail

interval="${LAY_VM_GUARD_INTERVAL:-1}"
state_dir="${XDG_RUNTIME_DIR:-/tmp}"
state_file="${state_dir}/lay-host-vm-guard.paused"

viewer_pattern='python3 .*/tmp/lay-kde-spice-viewer-clipboard\.py|lay-kde-spice-viewer|remote-viewer|virt-viewer|spicy|gnome-boxes|VirtualBoxVM'

viewer_running() {
    pgrep -af "${viewer_pattern}" | grep -v -E 'lay-host-vm-guard|pgrep|grep' >/dev/null
}

daemon_active() {
    sudo systemctl is-active --quiet lay-daemon.service
}

pause_daemon() {
    if daemon_active; then
        sudo systemctl stop lay-daemon.service
        printf '%s\n' "paused-by-vm-guard" >"${state_file}"
        echo "lay-host-vm-guard: paused host lay-daemon"
    fi
}

resume_daemon() {
    if [ -f "${state_file}" ]; then
        sudo systemctl start lay-daemon.service
        rm -f "${state_file}"
        echo "lay-host-vm-guard: resumed host lay-daemon"
    fi
}

trap 'resume_daemon' EXIT INT TERM

while true; do
    if viewer_running; then
        pause_daemon
    else
        resume_daemon
    fi
    sleep "${interval}"
done
