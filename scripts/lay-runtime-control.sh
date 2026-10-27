#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/lay/config.json"
LAY_DBUS_DEST="org.gnome.Shell"
LAY_DBUS_PATH="/io/github/radislabus_star/LayDaemon"
LAY_DBUS_IFACE="io.github.radislabus_star.LayDaemon"

user_systemctl() {
    # A remote desktop can have its own session bus. Only service-manager
    # commands use the user's systemd bus; IBus/KDE keep the desktop bus.
    local manager_bus="unix:path=${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/bus"
    DBUS_SESSION_BUS_ADDRESS="$manager_bus" systemctl --user "$@"
}

hydrate_desktop_env() {
    if [ -z "${XDG_RUNTIME_DIR:-}" ]; then
        export XDG_RUNTIME_DIR="/run/user/$(id -u)"
    fi
    if [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]; then
        export DBUS_SESSION_BUS_ADDRESS="unix:path=$XDG_RUNTIME_DIR/bus"
    fi

    local manager_env key value
    manager_env="$(user_systemctl show-environment 2>/dev/null || true)"
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

kde_session() {
    case "${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}" in
        *KDE*|*kde*|*Plasma*|*plasma*) return 0 ;;
        *) return 1 ;;
    esac
}

current_kde_ime() {
    local layouts index confirmed profile attempt status
    # Device creation can briefly expose an old list with a new group index.
    # Read a coherent snapshot before choosing any input source.
    for attempt in 1 2 3 4 5 6 7 8; do
        layouts="$(timeout 1s gdbus call --session --dest org.kde.keyboard \
        --object-path /Layouts --method org.kde.KeyboardLayouts.getLayoutsList)" || return 1
        index="$(timeout 1s gdbus call --session --dest org.kde.keyboard \
        --object-path /Layouts --method org.kde.KeyboardLayouts.getLayout)" || return 1
        confirmed="$(timeout 1s gdbus call --session --dest org.kde.keyboard \
        --object-path /Layouts --method org.kde.KeyboardLayouts.getLayoutsList)" || return 1
        if profile="$(python3 - "$layouts" "$index" "$confirmed" <<'PY'
import ast
import re
import sys
try:
    layouts, = ast.literal_eval(sys.argv[1])
    match = re.fullmatch(r"\((?:uint32 )?(\d+),\)", sys.argv[2])
    if not match or not isinstance(layouts, list) or not 1 <= len(layouts) <= 32:
        raise ValueError("invalid KDE layout snapshot")
    if any(not isinstance(row, tuple) or len(row) != 3
           or any(not isinstance(value, str) for value in row) for row in layouts):
        raise ValueError("invalid KDE layout description")
    if sum(len(value) for row in layouts for value in row) > 8192:
        raise ValueError("KDE layout description exceeds bound")
    if ast.literal_eval(sys.argv[3]) != (layouts,) or int(match[1]) >= len(layouts):
        sys.exit(75)
    name = layouts[int(match[1])][0]
    if name not in ("us", "ru") or sum(row[0] == name for row in layouts) != 1:
        raise ValueError("current KDE layout is unsupported or ambiguous")
    print("lay-ime-" + name)
except (ValueError, SyntaxError, TypeError, IndexError) as error:
    print(str(error), file=sys.stderr)
    sys.exit(1)
PY
        )"; then
            printf '%s\n' "$profile"
            return 0
        else
            status=$?
        fi
        [ "$status" -eq 75 ] || return "$status"
        if [ "$attempt" -lt 8 ]; then sleep 0.15; fi
    done
    echo "KDE keyboard layout snapshot did not settle" >&2
    return 1
}

prepare_kde_ime() {
    # KDE owns its map. IBus must not replace it when selecting an engine.
    timeout 1s gsettings set org.freedesktop.ibus.general use-system-keyboard-layout true \
        || return 1
    [ "$(timeout 1s gsettings get org.freedesktop.ibus.general use-system-keyboard-layout)" = true ]
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
    if kde_session; then
        [ "$(current_kde_ime)" = "$layout" ] || return 1
        sync_ibus_engine "$layout" || return 1
        [ "$(current_kde_ime)" = "$layout" ]
        return
    fi
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
    if kde_session; then
        current_kde_ime
        return
    fi
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
    preferred="$(preferred_lay_ime)" || return 1
    if kde_session; then
        prepare_kde_ime || {
            echo "Не удалось сохранить карту клавиатуры KDE в IBus" >&2
            return 1
        }
    fi
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

    user_systemctl stop lay-ibus-engine.service >/dev/null 2>&1 || true
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
    user_systemctl stop lay-ibus-engine.service >/dev/null 2>&1 || true
    stop_lay_ibus_engine
    select_xkb
}

apply_channel() {
    case "${1:-$(text_backend)}" in
        ime|auto)
            local start_status=0
            start_ime || start_status=$?
            if [ -x "$HOME/.local/bin/lay-ime-setup" ]; then
                "$HOME/.local/bin/lay-ime-setup" --show-if-needed >/dev/null 2>&1 &
            fi
            return "$start_status"
            ;;
        uinput)
            stop_ime
            ;;
        *)
            stop_ime
            ;;
    esac
}

case "${1:-status}" in
    start)
        user_systemctl start lay-daemon.service
        apply_channel "$(text_backend)"
        ;;
    stop)
        user_systemctl stop lay-daemon.service || true
        stop_ime
        ;;
    restart)
        if user_systemctl is-active --quiet lay-daemon.service; then
            user_systemctl restart lay-daemon.service
            apply_channel "$(text_backend)"
        else
            stop_ime
        fi
        ;;
    daemon-start|daemon-stop|daemon-restart)
        # Explicit daemon controls must not cycle IBus or change input sources.
        user_systemctl "${1#daemon-}" lay-daemon.service
        ;;
    daemon-is-active)
        user_systemctl is-active --quiet lay-daemon.service
        ;;
    channel)
        if user_systemctl is-active --quiet lay-daemon.service; then
            apply_channel "${2:-$(text_backend)}"
        else
            stop_ime
        fi
        ;;
    status)
        printf 'daemon='
        user_systemctl is-active lay-daemon.service || true
        printf 'ime_processes='
        pgrep -c -x lay-ibus-engine || true
        printf 'gnome_layout='
        current_gnome_layout || true
        printf 'ibus_engine='
        timeout 1s ibus engine || true
        ;;
    *)
        echo "usage: lay-runtime-control {daemon-start|daemon-stop|daemon-restart|daemon-is-active|start|stop|restart|channel [ime|uinput|auto]|status}" >&2
        exit 2
        ;;
esac
