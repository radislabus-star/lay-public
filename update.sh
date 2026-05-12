#!/usr/bin/env bash
# update.sh — проверить обновления lay, скачать их и переустановить при необходимости.
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

current_version() {
    if command -v "$HOME/.local/bin/lay" >/dev/null 2>&1; then
        "$HOME/.local/bin/lay" --version 2>/dev/null || true
    elif [ -x target/release/lay ]; then
        target/release/lay --version 2>/dev/null || true
    fi
}

echo "=== проверка обновлений ==="
echo "локально: $(current_version)"

upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>/dev/null || true)"
if [ -z "$upstream" ]; then
    if git remote get-url public >/dev/null 2>&1; then
        upstream="public/main"
    else
        upstream="origin/main"
    fi
fi

remote="${upstream%%/*}"
if [ "$remote" = "$upstream" ]; then
    remote="origin"
fi

git fetch --quiet "$remote"

if ! git rev-parse --verify "$upstream" >/dev/null 2>&1; then
    echo "Не могу проверить upstream: $upstream" >&2
    exit 1
fi

behind="$(git rev-list --count "HEAD..$upstream")"
ahead="$(git rev-list --count "$upstream..HEAD")"
echo "upstream: $upstream"
echo "новых коммитов: $behind"
if [ "$ahead" != "0" ]; then
    echo "локальных неопубликованных коммитов: $ahead"
fi

if [ "$behind" = "0" ]; then
    echo ""
    echo "✓ обновлений нет, текущая версия актуальна"
    exit 0
fi

echo ""
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
if [ -f "$HOME/.config/autostart/lay-kde-tray.desktop" ]; then
    pkill -f "$HOME/.local/bin/lay-kde-tray" 2>/dev/null || true
    desktop_hint="${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}"
    if [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] \
        && { printf '%s' "$desktop_hint" | grep -Eiq 'kde|plasma' || pgrep -x plasmashell >/dev/null 2>&1; }; then
        nohup "$HOME/.local/bin/lay-kde-tray" >/tmp/lay-kde-tray.log 2>&1 &
    else
        echo "ℹ KDE tray обновлён; он стартует при следующем входе в KDE"
    fi
fi
if systemctl --user is-enabled --quiet lay-host-vm-guard.service 2>/dev/null; then
    systemctl --user restart lay-host-vm-guard.service || true
fi

echo ""
echo "✓ lay обновлён"
echo "Если это первая установка или поменялась группа input — выйди из сессии и зайди снова."
