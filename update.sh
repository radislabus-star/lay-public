#!/usr/bin/env bash
# update.sh — обновить lay из git и переустановить локальную сборку.
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "update.sh работает только из git-копии lay." >&2
    exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "В рабочей копии есть локальные изменения." >&2
    echo "Сохрани их или откати перед обновлением, чтобы git pull ничего не затёр." >&2
    exit 1
fi

echo "=== git pull ==="
git pull --ff-only

echo ""
echo "=== install ==="
bash install.sh

echo ""
echo "=== reload runtime ==="
if command -v gnome-extensions >/dev/null 2>&1; then
    if gnome-extensions help reload >/dev/null 2>&1; then
        gnome-extensions reload lay@radislabus-star.github.io || true
    else
        gnome-extensions disable lay@radislabus-star.github.io 2>/dev/null || true
        sleep 1
        gnome-extensions enable lay@radislabus-star.github.io 2>/dev/null || true
    fi
fi
systemctl --user restart lay-daemon || true
if systemctl --user list-unit-files 'lay-kde-tray.service' --no-legend 2>/dev/null | grep -q lay-kde-tray; then
    systemctl --user restart lay-kde-tray.service || true
fi
if systemctl --user is-enabled --quiet lay-host-vm-guard.service 2>/dev/null; then
    systemctl --user restart lay-host-vm-guard.service || true
fi

echo ""
echo "✓ lay обновлён"
echo "Если это первая установка или поменялась группа input — выйди из сессии и зайди снова."
