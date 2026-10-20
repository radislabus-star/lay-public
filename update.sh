#!/usr/bin/env bash
# Stable release discovery, apply and scheduling controls have one owner.
set -euo pipefail
SCRIPT="$(readlink -f -- "${BASH_SOURCE[0]}")"
MODE="${1:---interactive}"
(( $# <= 1 )) || { echo "Ожидается один режим lay-update." >&2; exit 2; }
case "$MODE" in
    --menu)
        terminal_command='bash "$1" --interactive; result=$?; printf "\nНажми Enter, чтобы закрыть окно..."; read -r; exit "$result"'
        terminals=(gnome-terminal kgx konsole wezterm kitty xterm)
        if [[ "${XDG_CURRENT_DESKTOP:-}" =~ [Kk][Dd][Ee]|[Pp]lasma ]]; then
            terminals=(konsole gnome-terminal kgx wezterm kitty xterm)
        fi
        for terminal in "${terminals[@]}"; do
            command -v "$terminal" >/dev/null || continue
            case "$terminal" in
                gnome-terminal) exec "$terminal" --wait -- bash -c "$terminal_command" lay-update-menu "$SCRIPT" ;;
                konsole) exec "$terminal" --separate -e bash -c "$terminal_command" lay-update-menu "$SCRIPT" ;;
                kgx) exec "$terminal" -- bash -c "$terminal_command" lay-update-menu "$SCRIPT" ;;
                wezterm) exec "$terminal" start -- bash -c "$terminal_command" lay-update-menu "$SCRIPT" ;;
                *) exec "$terminal" -e bash -c "$terminal_command" lay-update-menu "$SCRIPT" ;;
            esac
        done
        echo "Не найден терминал для обновления Lay. Запусти lay-update в своём терминале." >&2
        exit 1 ;;
    --auto-status)
        state="$(systemctl --user show lay-auto-update.timer --property=UnitFileState --value)"
        case "$state" in enabled|enabled-runtime) echo enabled ;; *) echo disabled ;; esac
        exit 0 ;;
    --auto-enable)
        systemctl --user enable --now lay-auto-update.timer
        echo enabled; exit 0 ;;
    --auto-disable)
        # Do not cancel an installation already replacing files.
        systemctl --user disable --now lay-auto-update.timer
        echo disabled; exit 0 ;;
    --check|--interactive|--automatic|--source-only) ;;
    *) echo "Неизвестный режим: $MODE" >&2; exit 2 ;;
esac
state_dir="${XDG_STATE_HOME:-$HOME/.local/state}/lay"
mkdir -p "$state_dir"
exec 9>"$state_dir/update.lock"
flock -n 9 || { echo "Другое обновление Lay уже выполняется." >&2; exit 75; }

if [[ "$MODE" == --source-only ]]; then
    # Explicit developer operation, never called by menus or the timer.
    cd "$(dirname "$SCRIPT")"
    git rev-parse --is-inside-work-tree >/dev/null
    upstream="$(git rev-parse --abbrev-ref --symbolic-full-name '@{u}')"
    git fetch --quiet "${upstream%%/*}"
    git merge --ff-only "$upstream"
    exit 0
fi
installed="$("$HOME/.local/bin/lay" --version)"
installed="${installed#lay }"
api="${LAY_UPDATE_RELEASE_API:-https://api.github.com/repos/radislabus-star/lay-public/releases/latest}"
repository="${LAY_UPDATE_REPOSITORY_URL:-https://github.com/radislabus-star/lay-public.git}"
source_dir="${XDG_DATA_HOME:-$HOME/.local/share}/lay/update-source"
release="$(python3 - "$api" "$installed" <<'PY'
import json, re, sys, urllib.request
try:
    request = urllib.request.Request(sys.argv[1], headers={
        "Accept": "application/vnd.github+json", "User-Agent": "Lay-updater"})
    with urllib.request.urlopen(request, timeout=20) as response:
        data = json.load(response)
    tag = data.get("tag_name", "")
    if data.get("draft") is not False or data.get("prerelease") is not False:
        raise ValueError("ответ не является стабильным опубликованным релизом")
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag):
        raise ValueError("некорректный тег стабильного релиза")
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", sys.argv[2]):
        raise ValueError("не удалось определить установленную версию")
    latest = tuple(map(int, tag[1:].split(".")))
    current = tuple(map(int, sys.argv[2].split(".")))
    print(tag, tag[1:], "new" if latest > current else "current" if latest == current else "newer")
except Exception as error:
    print(f"Ошибка проверки стабильного релиза: {error}", file=sys.stderr)
    sys.exit(1)
PY
)"
read -r tag version relation <<<"$release"
printf '=== проверка обновлений ===\nустановлено: %s\nстабильный релиз: %s\nисточник: %s\n' "$installed" "$version" "$api"
if [[ "$relation" == current ]]; then
    echo "✓ обновлений нет, установлена актуальная стабильная версия"; exit 0
elif [[ "$relation" == newer ]]; then
    echo "Установленная версия новее стабильной; понижение не выполняется."; exit 0
fi
echo "Доступно обновление: $installed → $version"
[[ "$MODE" != --check ]] || exit 0
if [[ "$MODE" == --interactive ]]; then
    read -r -p "Собрать и установить стабильный релиз? [y/N] " answer || answer=""
    case "$answer" in y|Y|yes|YES|д|Д|да|Да) ;; *) echo "Обновление отменено."; exit 0 ;; esac
fi
if [[ -e "$source_dir" ]]; then
    [[ -f "$source_dir/.lay-updater-owner" ]] &&
    [[ "$(cat "$source_dir/.lay-updater-owner")" == lay-stable-updater-v1 ]] || {
        echo "Отказ: каталог не принадлежит updater: $source_dir" >&2; exit 1;
    }
    [[ "$(git -C "$source_dir" remote get-url origin)" == "$repository" ]] || {
        echo "Отказ: изменён источник стабильных обновлений." >&2; exit 1;
    }
    git -C "$source_dir" diff --quiet && git -C "$source_dir" diff --cached --quiet || {
        echo "Отказ: исходники updater изменены; изменения сохранены." >&2; exit 1;
    }
else
    mkdir -p "$source_dir"
    git -C "$source_dir" init --quiet
    git -C "$source_dir" remote add origin "$repository"
    printf 'lay-stable-updater-v1\n' > "$source_dir/.lay-updater-owner"
fi
export GIT_TERMINAL_PROMPT=0
timeout 180 git -C "$source_dir" fetch --quiet --depth=1 origin "refs/tags/$tag"
commit="$(git -C "$source_dir" rev-parse --verify 'FETCH_HEAD^{commit}')"
git -C "$source_dir" checkout --quiet --detach "$commit"
source_version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$source_dir/Cargo.toml" | head -1)"
[[ "$source_version" == "$version" ]] || {
    echo "Версия исходников не совпадает со стабильным релизом." >&2; exit 1;
}
(
    cd "$source_dir"
    bash ./install.sh --update
)
actual="$("$HOME/.local/bin/lay" --version)"
[[ "$actual" == "lay $version" ]] || {
    echo "Установленная версия не совпадает с релизом: $actual" >&2; exit 1;
}
printf '%s %s\n' "$tag" "$commit" > "$state_dir/installed-release"
# Respect a stop performed while a long build was running.
systemctl --user try-restart lay-daemon
if [[ "${XDG_CURRENT_DESKTOP:-}" == *GNOME* ]]; then
    bash "$source_dir/scripts/check-gnome-extension-runtime.sh" --reload
elif [[ "${XDG_CURRENT_DESKTOP:-}" == *KDE* ]]; then
    pkill -f "$HOME/.local/bin/lay-kde-tray" 2>/dev/null || true
    nohup "$HOME/.local/bin/lay-kde-tray" 9>&- >"$state_dir/kde-tray.log" 2>&1 &
fi
echo "✓ установлен стабильный Lay $version"
echo "Активный IME продолжит текущую сессию ввода; глобальный IBus не перезапускался."
