#!/bin/bash
# install-extension.sh — устанавливает GNOME Shell extension lay@radislabus-star.github.io
set -eu

DIR="$(cd "$(dirname "$0")" && pwd)"
UUID="lay@radislabus-star.github.io"
SRC="$DIR/extension/$UUID"
DST="$HOME/.local/share/gnome-shell/extensions/$UUID"
CACHE="$HOME/.cache/lay"

if [ ! -d "$SRC" ]; then
    echo "✗ нет $SRC" >&2
    exit 1
fi

echo "=== копирую extension → $DST ==="
mkdir -p "$DST"
mkdir -p "$CACHE"
cp -v "$SRC/metadata.json" "$DST/"
for js in "$SRC"/*.js; do
    name="$(basename "$js")"
    cp -v "$js" "$DST/$name"
    if [ "$name" != "extension.js" ] && [ "$name" != "lay-impl.js" ]; then
        cp -v "$js" "$CACHE/$name" 2>/dev/null || true
    fi
done

echo ""
echo "=== gnome-extensions enable ==="
gnome-extensions enable "$UUID" || true
gnome-extensions info "$UUID" | head -10

echo ""
echo "=== проверка загрузки ==="
echo "Если индикатор не появился после первой установки, выйди из GNOME и зайди снова."
echo "Для разработки уже загруженного extension можно использовать ./dev-reload.sh."
echo ""
echo "После перелогина проверка:"
echo "  gdbus call --session \\"
echo "    --dest org.gnome.Shell \\"
echo "    --object-path /io/github/radislabus_star/LayDaemon \\"
echo "    --method io.github.radislabus_star.LayDaemon.Ping"
echo ""
echo "Если ответит 'pong' — extension работает. Если 'service unknown' — extension"
echo "не загрузился (смотри 'journalctl --user -b 0 | grep gnome-shell' на ошибки)."
