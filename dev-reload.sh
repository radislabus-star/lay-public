#!/bin/bash
# dev-reload.sh — перезагрузить extension без logout
# Если gnome-extensions поддерживает reload — используем его.
# Иначе disable + enable; двухфайловый loader обходит кэш GJS.

UUID="lay@radislabus-star.github.io"
SRC="$(cd "$(dirname "$0")" && pwd)/extension/$UUID"
DST="$HOME/.local/share/gnome-shell/extensions/$UUID"
LAY_GJS_CACHE="$HOME/.cache/lay"

GNOME_VER=$(gnome-shell --version 2>/dev/null | grep -oP '\d+' | head -1)
echo "GNOME Shell $GNOME_VER"

echo "→ sync GNOME extension runtime"
"$(cd "$(dirname "$0")" && pwd)/scripts/check-gnome-extension-runtime.sh" --fix --reload

sleep 2
systemctl --user restart lay-daemon
LOADED_VERSION="$(
    gdbus call --session \
    --dest org.gnome.Shell \
    --object-path /io/github/radislabus_star/LayDaemon \
    --method io.github.radislabus_star.LayDaemon.Version 2>/dev/null \
    | sed -n "s/.*'\([^']*\)'.*/\1/p" \
    || true
)"
if [ -n "$LOADED_VERSION" ]; then
    echo "✓ загруженная версия extension: $LOADED_VERSION"
else
    echo "⚠ не удалось проверить загруженную версию extension через DBus"
fi
"$(cd "$(dirname "$0")" && pwd)/scripts/check-gnome-extension-runtime.sh"
echo "✓ готово"
