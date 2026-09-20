#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/lay/config.json"
LAY_DBUS_DEST="org.gnome.Shell"
LAY_DBUS_PATH="/io/github/radislabus_star/LayDaemon"
LAY_DBUS_IFACE="io.github.radislabus_star.LayDaemon"

hydrate_desktop_env() {
    if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
        export XDG_RUNTIME_DIR="/run/user/$(id -u)"
    fi
    if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
        export DBUS_SESSION_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/bus"
    fi

    local manager_env key value
    manager_env="$(systemctl --user show-environment 2>/dev/null || true)"
    for key in DISPLAY WAYLAND_DISPLAY XAUTHORITY; do
        if [ -n "${!key:-}" ]; then
            continue
        fi
        value="$(printf '%s\n' "$manager_env" | sed -n "s/^${key}=//p" | head -n1)"
        if [ -n "$value" ]; then
            export "$key=$value"
        fi
    done
}

hydrate_desktop_env

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
    local attempt engine
    for attempt in 1 2 3 4 5; do
        # ibus(1) may return non-zero after the global engine has already
        # changed (for example when an XKB helper fails). Trust the observable
        # global-engine readback, not only the setter's exit status.
        timeout 2s ibus engine "$layout" >/dev/null 2>&1 || true
        engine="$(timeout 1s ibus engine 2>/dev/null || true)"
        if [ "$engine" = "$layout" ]; then
            return 0
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
        exe="${exe% (deleted)}"
        case "$exe" in
            */lay-ibus-engine)
                kill -TERM "$pid" 2>/dev/null || true
                ;;
        esac
    done < <(
        pgrep -f '(^|/)lay-ibus-engine --ibus( --managed)?$' 2>/dev/null || true
    )
}

wait_lay_ibus_engine_stopped() {
    local attempt
    for attempt in $(seq 1 40); do
        if ! pgrep -f '(^|/)lay-ibus-engine --ibus( --managed)?$' >/dev/null 2>&1; then
            return 0
        fi
        sleep 0.05
    done
    return 1
}

xkb_fallback_for_lay_ime() {
    case "${1:?Lay IME required}" in
        lay-ime-us) printf '%s\n' xkb:us::eng ;;
        lay-ime-ru) printf '%s\n' xkb:ru::rus ;;
        *) return 1 ;;
    esac
}

select_lay_ime() {
    local layout="${1:?layout required}"
    local attempt current engine
    if activate_gnome_layout "$layout"; then
        # ActivateLayout already owns the global-engine transition. Do not race
        # it with a second setter; wait for GNOME and IBus to converge first.
        for attempt in 1 2 3 4 5; do
            current="$(current_gnome_layout || true)"
            engine="$(timeout 1s ibus engine 2>/dev/null || true)"
            if [ "$current" = "$layout" ] && [ "$engine" = "$layout" ]; then
                return 0
            fi
            sleep 0.10
        done
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
    local preferred fallback safe_xkb current_source source_already_lay
    current_source="$(current_gnome_layout || true)"
    preferred="$(preferred_lay_ime)"
    if [ "$preferred" = lay-ime-us ]; then
        fallback=lay-ime-ru
    else
        fallback=lay-ime-us
    fi
    case "$current_source" in
        lay-ime-us|lay-ime-ru) source_already_lay=true ;;
        *) source_already_lay=false ;;
    esac
    safe_xkb="$(xkb_fallback_for_lay_ime "$preferred")"

    # Never terminate the process that currently owns the global IBus engine.
    # Move clients onto the same-language XKB engine first and verify that the
    # switch settled; otherwise killing Lay can strand existing input contexts.
    sync_ibus_engine "$safe_xkb" || return 1
    sleep 0.10

    systemctl --user stop lay-ibus-engine.service >/dev/null 2>&1 || true
    stop_lay_ibus_engine
    wait_lay_ibus_engine_stopped || return 1

    if [ "$source_already_lay" = true ]; then
        # During a hot restart GNOME already owns the correct input source.
        # Re-select only the global IBus engine; calling ActivateLayout again
        # races GNOME's own setter and produces a cancelled engine transition.
        sync_ibus_engine "$preferred" || return 1
        return 0
    fi

    select_lay_ime "$preferred" \
        || select_lay_ime "$fallback" \
        || return 1
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
        if systemctl --user is-active --quiet lay-daemon.service; then
            systemctl --user restart lay-daemon.service
            apply_channel "$(text_backend)"
        else
            stop_ime
        fi
        ;;
    channel)
        if systemctl --user is-active --quiet lay-daemon.service; then
            apply_channel "${2:-$(text_backend)}"
        else
            stop_ime
        fi
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
