#!/usr/bin/env bash
# update.sh — проверить обновления lay, скачать их и переустановить при необходимости.
set -euo pipefail

SOURCE_ONLY=0
if [ "${1:-}" = "--source-only" ]; then
    SOURCE_ONLY=1
elif [ "$#" -gt 0 ]; then
    echo "Использование: bash update.sh [--source-only]" >&2
    exit 2
fi

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "update.sh работает только из git-копии lay." >&2
    exit 1
fi

preserve_local_changes() {
    if [ -z "$(git status --porcelain --untracked-files=normal)" ]; then
        return
    fi
    backup_name="lay-auto-update-$(date +%Y%m%d-%H%M%S)"
    echo "Локальные изменения найдены; сохраняю их в git stash: $backup_name"
    git stash push --include-untracked -m "$backup_name" >/dev/null
    echo "✓ локальные изменения сохранены: $(git stash list -1 --format='%gd %s')"
    echo "  вернуть позже: git stash pop"
}

current_version() {
    if command -v "$HOME/.local/bin/lay" >/dev/null 2>&1; then
        "$HOME/.local/bin/lay" --version 2>/dev/null || true
    elif [ -x target/release/lay ]; then
        target/release/lay --version 2>/dev/null || true
    fi
}

source_version() {
    sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1
}

installed_version() {
    current_version | sed -n 's/^lay //p' | head -1
}

live_extension_version() {
    gdbus call --session \
        --dest org.gnome.Shell \
        --object-path /io/github/radislabus_star/LayDaemon \
        --method io.github.radislabus_star.LayDaemon.Version 2>/dev/null \
        | sed -n "s/.*'\([^']*\)'.*/\1/p" \
        || true
}

refresh_runtime() {
    if [ -x ./dev-reload.sh ]; then
        bash ./dev-reload.sh
        return
    fi

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
    src_ver="$(source_version)"
    installed_ver="$(installed_version)"
    live_ver="$(live_extension_version)"
    echo "source: ${src_ver:-unknown}"
    echo "установлено: ${installed_ver:-unknown}"
    echo "трей/DBus: ${live_ver:-unknown}"

    if [ -n "$src_ver" ] && [ "$installed_ver" != "$src_ver" ]; then
        echo ""
        echo "⚠ установленная версия отстала от рабочей копии, переустанавливаю..."
        bash install.sh
        refresh_runtime
        echo "✓ runtime обновлён до $(current_version)"
        exit 0
    fi

    if [ -n "$src_ver" ] && [ -n "$live_ver" ] && [ "$live_ver" != "$src_ver" ]; then
        echo ""
        echo "⚠ трей/DBus держит старую версию, перезагружаю runtime..."
        refresh_runtime
        echo "✓ трей/DBus: $(live_extension_version)"
        exit 0
    fi

    echo ""
    echo "✓ обновлений нет, текущая версия актуальна"
    exit 0
fi

echo ""
preserve_local_changes
echo "=== git pull ==="
git pull --ff-only

if [ "$SOURCE_ONLY" = "1" ]; then
    echo "✓ исходники обновлены; установка пропущена (--source-only)"
    exit 0
fi

echo ""
echo "=== install ==="
bash install.sh

echo ""
echo "=== reload runtime ==="
refresh_runtime
if [ -f "$HOME/.config/autostart/lay-kde-tray.desktop" ]; then
    pkill -f "$HOME/.local/bin/lay-kde-tray" 2>/dev/null || true
    lay_state_dir="$HOME/.local/state/lay"
    mkdir -p "$lay_state_dir"
    lay_kde_tray_log="$lay_state_dir/kde-tray.log"
    desktop_hint="${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}"
    if [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] \
        && { printf '%s' "$desktop_hint" | grep -Eiq 'kde|plasma' || pgrep -x plasmashell >/dev/null 2>&1; }; then
        nohup "$HOME/.local/bin/lay-kde-tray" >"$lay_kde_tray_log" 2>&1 &
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
