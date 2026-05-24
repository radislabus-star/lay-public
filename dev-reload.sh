#!/bin/bash
# dev-reload.sh — перезагрузить extension без logout
# Если gnome-extensions поддерживает reload — используем его.
# Иначе disable + enable; двухфайловый loader обходит кэш GJS.

UUID="lay@radislabus-star.github.io"
SRC="$(cd "$(dirname "$0")" && pwd)/extension/$UUID"
DST="$HOME/.local/share/gnome-shell/extensions/$UUID"

GNOME_VER=$(gnome-shell --version 2>/dev/null | grep -oP '\d+' | head -1)
echo "GNOME Shell $GNOME_VER"

# Обновляем файлы (на случай если не симлинк)
mkdir -p "$DST"
cp -f "$SRC/extension.js" "$DST/extension.js" 2>/dev/null || \
  ln -sf "$SRC/extension.js" "$DST/extension.js"
cp -f "$SRC/lay-impl.js" "$DST/lay-impl.js" 2>/dev/null || \
  ln -sf "$SRC/lay-impl.js" "$DST/lay-impl.js"
cp -f "$SRC/metadata.json" "$DST/metadata.json" 2>/dev/null || true

if gnome-extensions help reload >/dev/null 2>&1; then
    echo "→ gnome-extensions reload"
    gnome-extensions reload "$UUID"
else
    echo "→ gnome-extensions reload недоступен: disable + enable"
    gnome-extensions disable "$UUID"
    sleep 1
    gnome-extensions enable "$UUID"
fi

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
echo "✓ готово"
