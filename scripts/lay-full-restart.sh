#!/usr/bin/env bash
# Explicit recovery only. Automatic settings reloads keep using ordinary restart.
set -euo pipefail
source "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")/lay-runtime-control.sh"
restore_daemon=0
preferred=""

finish() {
    local status=$?
    if [ "$status" -eq 0 ]; then
        echo "Lay: полный перезапуск завершён"
        notify-send -a Lay "Lay" "Полный перезапуск завершён" 2>/dev/null || true
    else
        if [ "$restore_daemon" -eq 1 ]; then
            user_systemctl start lay-daemon.service >/dev/null 2>&1 || true
            if [ -n "$preferred" ]; then
                start_ime >/dev/null 2>&1 || true
            fi
        fi
        echo "Lay: полный перезапуск не завершён (код $status); journalctl --user -u lay-full-restart.service" >&2
        notify-send -a Lay "Lay: ошибка перезапуска" \
            "Подробности: journalctl --user -u lay-full-restart.service" 2>/dev/null || true
    fi
    exit "$status"
}
trap finish EXIT

stop_l11_socket_owner() {
    # Socket peer credentials, executable and pidfd prevent broad/pid-reuse kills.
    /usr/bin/python3 - "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/lay-l11.sock" <<'PY'
import os, select, signal, socket, struct, sys
from pathlib import Path
with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
    connection.settimeout(2)
    try:
        connection.connect(sys.argv[1])
    except (FileNotFoundError, ConnectionRefusedError):
        sys.exit(0)
    pid, uid, _ = struct.unpack("3i", connection.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12))
    if uid != os.getuid() or pid <= 1:
        raise RuntimeError("L1.1 socket does not belong to the current user")
    try:
        descriptor = os.pidfd_open(pid)
    except ProcessLookupError:
        sys.exit(0)
    try:
        executable = Path(os.readlink(f"/proc/{pid}/exe").removesuffix(" (deleted)"))
        if executable.name != "lay-l1.1-serve":
            raise RuntimeError("Refusing to stop a foreign L1.1 socket peer")
        signal.pidfd_send_signal(descriptor, signal.SIGTERM)
        poll = select.poll()
        poll.register(descriptor, select.POLLIN)
        if not poll.poll(5000):
            raise RuntimeError("L1.1 did not stop within 5 seconds")
    finally:
        os.close(descriptor)
PY
}

backend="$(text_backend)"
if [[ "$backend" == ime || "$backend" == auto ]]; then
    preferred="$(preferred_lay_ime)"
    safe_xkb="$(xkb_fallback_for_lay_ime "$preferred")"
    if kde_session; then
        prepare_kde_ime
    fi
    if timeout 2s ibus engine >/dev/null 2>&1; then
        sync_ibus_engine "$safe_xkb"
    fi
fi

if user_systemctl is-active --quiet lay-daemon.service; then
    restore_daemon=1
fi
user_systemctl stop lay-daemon.service
if [ -n "$preferred" ]; then
    user_systemctl stop lay-ibus-engine.service >/dev/null 2>&1 || true
    stop_lay_ibus_engine
    wait_lay_ibus_engine_stopped
fi
stop_l11_socket_owner

if [ -n "$preferred" ]; then
    if ! kde_session && [ "$(user_systemctl show --property=LoadState --value org.freedesktop.IBus.session.GNOME.service)" = loaded ]; then
        user_systemctl restart org.freedesktop.IBus.session.GNOME.service
    else
        timeout 15s ibus restart
    fi
    # A restarted bus may have a new socket; never retain its retired address.
    unset IBUS_ADDRESS
    # A real GlobalEngine must exist before a new Lay engine is instantiated.
    sync_ibus_engine "$safe_xkb"
fi

if user_systemctl is-active --quiet lay-l3-online.service \
    || user_systemctl is-enabled --quiet lay-l3-online.service; then
    user_systemctl restart lay-l3-online.service
fi

if ! kde_session; then
    uuid=lay@radislabus-star.github.io
    # Reload only an enabled installed Lay extension, never GNOME Shell itself.
    if gnome-extensions list --enabled | /usr/bin/grep -Fxq "$uuid"; then
        gnome-extensions disable "$uuid"
        gnome-extensions enable "$uuid"
        for attempt in $(seq 1 25); do
            [ -n "$(current_gnome_layout || true)" ] && break
            sleep 0.2
        done
        [ -n "$(current_gnome_layout)" ]
    fi
fi

user_systemctl start lay-daemon.service
if [ -n "$preferred" ]; then
    start_ime
    [ "$(timeout 2s ibus engine)" = "$preferred" ]
    if ! kde_session; then
        [ "$(current_gnome_layout)" = "$preferred" ]
    fi
fi
if kde_session && user_systemctl is-active --quiet lay-kde-tray.service; then
    user_systemctl restart lay-kde-tray.service
fi
