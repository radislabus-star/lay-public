#!/usr/bin/env bash
# update.sh — проверить обновления lay, скачать их и переустановить при необходимости.
set -euo pipefail

SOURCE_ONLY=0
INTERACTIVE=0
CHECK_ONLY=0
if [ "$#" -gt 1 ]; then
    echo "Использование: lay-update [--menu|--check|--source-only]" >&2
    exit 2
elif [ "${1:-}" = "--source-only" ]; then
    SOURCE_ONLY=1
elif [ "${1:-}" = "--check" ]; then
    CHECK_ONLY=1
elif [ "${1:-}" = "--interactive" ]; then
    INTERACTIVE=1
elif [ "${1:-}" = "--menu" ]; then
    : # Handled below, after resolving the installed symlink.
elif [ "$#" -gt 0 ]; then
    echo "Использование: lay-update [--menu|--check|--source-only]" >&2
    exit 2
fi

SCRIPT="$(readlink -f -- "${BASH_SOURCE[0]}")"
DIR="$(dirname "$SCRIPT")"
if [ "${1:-}" = "--menu" ]; then
    command='bash "$1" --interactive; result=$?; if [ "$result" != 0 ]; then printf "\nОшибка обновления (код %s).\n" "$result"; fi; printf "\nНажми Enter, чтобы закрыть окно..."; read -r; exit "$result"'
    terminals=(gnome-terminal kgx konsole wezterm kitty xterm)
    if [[ "${XDG_CURRENT_DESKTOP:-}" =~ [Kk][Dd][Ee]|[Pp]lasma ]]; then
        terminals=(konsole gnome-terminal kgx wezterm kitty xterm)
    fi
    for terminal in "${terminals[@]}"; do
        if ! command -v "$terminal" >/dev/null 2>&1; then continue; fi
        case "$terminal" in
            gnome-terminal) exec "$terminal" --wait -- bash -c "$command" lay-update "$SCRIPT" ;;
            kgx) exec "$terminal" -- bash -c "$command" lay-update "$SCRIPT" ;;
            wezterm) exec "$terminal" start -- bash -c "$command" lay-update "$SCRIPT" ;;
            *) exec "$terminal" -e bash -c "$command" lay-update "$SCRIPT" ;;
        esac
    done
    echo "Не найден терминал для обновления Lay. Запусти lay-update в своём терминале." >&2
    exit 1
fi
cd "$DIR"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "update.sh работает только из git-копии lay." >&2
    exit 1
fi

state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/lay"
mkdir -p "$state_dir"
exec 9>"$state_dir/update.lock"
if ! flock -n 9; then
    echo "Обновление Lay уже выполняется в другом окне." >&2
    exit 75
fi

confirm_install() {
    if [ "$INTERACTIVE" != 1 ]; then return; fi
    echo "Для обновления потребуется сборка и установка Lay; возможен запрос sudo."
    printf 'Обновить сейчас? [y/N]: '
    read -r answer || answer=''
    case "$answer" in y|Y|д|Д|да|Да) ;; *) echo 'Обновление отменено.'; exit 0 ;; esac
}

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
    desktop_hint="${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}"
    if [[ "$desktop_hint" =~ [Gg][Nn][Oo][Mm][Ee]|[Uu]nity ]]; then
        if [ -x ./dev-reload.sh ]; then
            bash ./dev-reload.sh
        else
            ./scripts/check-gnome-extension-runtime.sh --fix --reload
            systemctl --user restart lay-daemon
        fi
    else
        systemctl --user restart lay-daemon
    fi
    if [[ "$desktop_hint" =~ [Kk][Dd][Ee]|[Pp]lasma ]] && [ -x "$HOME/.local/bin/lay-kde-tray" ]; then
        pkill -f "$HOME/.local/bin/lay-kde-tray" 2>/dev/null || true
        lay_state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/lay"
        mkdir -p "$lay_state_dir"
        # A long-lived tray must not inherit the updater lock.
        nohup "$HOME/.local/bin/lay-kde-tray" 9>&- >"$lay_state_dir/kde-tray.log" 2>&1 &
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

if [ "$CHECK_ONLY" = 1 ]; then
    echo "source: $(source_version)"
    echo "Проверка завершена; установка и перезапуск не выполнялись."
    exit 0
fi
if [ "$behind" != 0 ] && [ "$ahead" != 0 ]; then
    echo "Ветки разошлись. Обновление остановлено; локальные изменения сохранены на месте." >&2
    exit 1
fi

if [ "$behind" = "0" ]; then
    if [ "$SOURCE_ONLY" = 1 ]; then
        echo "✓ исходники актуальны; установка пропущена (--source-only)"
        exit 0
    fi
    src_ver="$(source_version)"
    installed_ver="$(installed_version)"
    live_ver="$(live_extension_version)"
    echo "source: ${src_ver:-unknown}"
    echo "установлено: ${installed_ver:-unknown}"
    echo "трей/DBus: ${live_ver:-unknown}"

    if [ -n "$src_ver" ] && [ "$installed_ver" != "$src_ver" ]; then
        echo ""
        echo "⚠ установленная версия отстала от рабочей копии, переустанавливаю..."
        confirm_install
        LAY_KEEP_OLLAMA=1 bash install.sh
        refresh_runtime
        echo "✓ runtime обновлён до $(current_version)"
        exit 0
    fi

    if [ -n "$src_ver" ] && [ -n "$live_ver" ] && [ "$live_ver" != "$src_ver" ]; then
        echo ""
        echo "⚠ трей/DBus держит старую версию, перезагружаю runtime..."
        confirm_install
        refresh_runtime
        echo "✓ трей/DBus: $(live_extension_version)"
        exit 0
    fi

    echo ""
    echo "✓ обновлений нет, текущая версия актуальна"
    exit 0
fi

echo ""
if [ "$SOURCE_ONLY" != 1 ]; then confirm_install; fi
preserve_local_changes
echo "=== git pull ==="
git merge --ff-only "$upstream"

if [ "$SOURCE_ONLY" = "1" ]; then
    echo "✓ исходники обновлены; установка пропущена (--source-only)"
    exit 0
fi

echo ""
echo "=== install ==="
LAY_KEEP_OLLAMA=1 bash install.sh

echo ""
echo "=== reload runtime ==="
refresh_runtime
if systemctl --user is-enabled --quiet lay-host-vm-guard.service 2>/dev/null; then
    systemctl --user restart lay-host-vm-guard.service || true
fi

echo ""
echo "✓ lay обновлён"
echo "Если это первая установка или поменялась группа input — выйди из сессии и зайди снова."
