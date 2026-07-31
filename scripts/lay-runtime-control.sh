#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/lay/config.json"
LAY_DBUS_DEST="org.gnome.Shell"
LAY_DBUS_PATH="/io/github/radislabus_star/LayDaemon"
LAY_DBUS_IFACE="io.github.radislabus_star.LayDaemon"

text_backend() {
    python3 - "$CONFIG_PATH" <<'PY'
import json
import sys
try:
    with open(sys.argv[1], "r", encoding="utf-8") as f:
        print(json.load(f).get("text_backend", "uinput"))
except Exception:
    print("uinput")
PY
}

select_xkb() {
    timeout 2s ibus engine xkb:ru::rus \
        || timeout 2s ibus engine xkb:us::eng \
        || true
}

activate_gnome_layout() {
    local layout="${1:?layout required}"
    timeout 2s gdbus call \
        --session \
        --dest "$LAY_DBUS_DEST" \
        --object-path "$LAY_DBUS_PATH" \
        --method "$LAY_DBUS_IFACE.ActivateLayout" \
        "$layout" >/dev/null
}

current_gnome_layout() {
    timeout 2s gdbus call \
        --session \
        --dest "$LAY_DBUS_DEST" \
        --object-path "$LAY_DBUS_PATH" \
        --method "$LAY_DBUS_IFACE.CurrentLayout" \
        2>/dev/null \
        | sed -n "s/^('\\(.*\\)',)$/\\1/p"
}

sync_ibus_engine() {
    local layout="${1:?layout required}"
    local attempt
    for attempt in 1 2 3 4 5; do
        if timeout 2s ibus engine "$layout" >/dev/null 2>&1; then
            local engine
            engine="$(timeout 1s ibus engine 2>/dev/null || true)"
            if [ "$engine" = "$layout" ]; then
                return 0
            fi
        fi
        sleep 0.15
    done
    return 1
}

stop_lay_ibus_engine() {
    local pid exe

    # Linux comm is limited to 15 bytes. Match the exact managed argv, then
    # verify the executable so a parent shell mentioning the name is untouched.
    while IFS= read -r pid; do
        [ -n "$pid" ] || continue
        exe="$(readlink -f "/proc/$pid/exe" 2>/dev/null || true)"
        case "$exe" in
            */lay-ibus-engine)
                kill -TERM "$pid" 2>/dev/null || true
                ;;
        esac
    done < <(
        pgrep -f '(^|/)lay-ibus-engine --ibus( --managed)?$' 2>/dev/null || true
    )
}

select_lay_ime() {
    local layout="${1:?layout required}"
    if activate_gnome_layout "$layout"; then
        sync_ibus_engine "$layout" || true
        local current engine
        current="$(current_gnome_layout || true)"
        engine="$(timeout 1s ibus engine 2>/dev/null || true)"
        if [ "$current" = "$layout" ] && [ "$engine" = "$layout" ]; then
            return 0
        fi
    fi
    sync_ibus_engine "$layout"
}

preferred_lay_ime() {
    local current engine
    current="$(current_gnome_layout || true)"
    case "$current" in
        lay-ime-us|xkb:us*)
            printf '%s\n' lay-ime-us
            return
            ;;
        lay-ime-ru|xkb:ru*)
            printf '%s\n' lay-ime-ru
            return
            ;;
    esac

    engine="$(timeout 1s ibus engine 2>/dev/null || true)"
    case "$engine" in
        lay-ime-us|xkb:us*) printf '%s\n' lay-ime-us ;;
        *) printf '%s\n' lay-ime-ru ;;
    esac
}

start_ime() {
    local preferred fallback
    preferred="$(preferred_lay_ime)"
    if [ "$preferred" = lay-ime-us ]; then
        fallback=lay-ime-ru
    else
        fallback=lay-ime-us
    fi
    systemctl --user stop lay-ibus-engine.service >/dev/null 2>&1 || true
    stop_lay_ibus_engine
    select_lay_ime "$preferred" \
        || select_lay_ime "$fallback" \
        || true
}

stop_ime() {
    select_xkb
    systemctl --user stop lay-ibus-engine.service >/dev/null 2>&1 || true
    stop_lay_ibus_engine
    select_xkb
}

apply_channel() {
    case "${1:-$(text_backend)}" in
        ime)
            start_ime
            ;;
        uinput)
            stop_ime
            ;;
        auto)
            start_ime
            ;;
        *)
            stop_ime
            ;;
    esac
}

case "${1:-status}" in
    start)
        systemctl --user start lay-daemon.service
        apply_channel "$(text_backend)"
        ;;
    stop)
        systemctl --user stop lay-daemon.service || true
        stop_ime
        ;;
    restart)
        systemctl --user restart lay-daemon.service
        apply_channel "$(text_backend)"
        ;;
    channel)
        apply_channel "${2:-$(text_backend)}"
        ;;
    status)
        printf 'daemon='
        systemctl --user is-active lay-daemon.service || true
        printf 'ime_processes='
        pgrep -c -x lay-ibus-engine || true
        printf 'gnome_layout='
        current_gnome_layout || true
        printf 'ibus_engine='
        timeout 1s ibus engine || true
        ;;
    *)
        echo "usage: lay-runtime-control {start|stop|restart|channel [ime|uinput|auto]|status}" >&2
        exit 2
        ;;
esac
