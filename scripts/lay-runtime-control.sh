#!/usr/bin/env bash
set -euo pipefail

CONFIG_PATH="${XDG_CONFIG_HOME:-$HOME/.config}/lay/config.json"

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

start_ime() {
    systemctl --user stop lay-ibus-engine.service >/dev/null 2>&1 || true
    pkill -x lay-ibus-engine || true
    timeout 2s ibus engine lay-ime-ru \
        || timeout 2s ibus engine lay-ime-ru \
        || timeout 2s ibus engine lay-ime-us \
        || true
}

stop_ime() {
    select_xkb
    systemctl --user stop lay-ibus-engine.service >/dev/null 2>&1 || true
    pkill -TERM -x lay-ibus-engine || true
    pkill -x lay-ibus-engine || true
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
        printf 'ibus_engine='
        timeout 1s ibus engine || true
        ;;
    *)
        echo "usage: lay-runtime-control {start|stop|restart|channel [ime|uinput|auto]|status}" >&2
        exit 2
        ;;
esac
