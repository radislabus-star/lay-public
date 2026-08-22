#!/usr/bin/env bash
# uninstall.sh - remove the lay runtime; --purge also removes user data and a clean source checkout.
set -euo pipefail

PURGE=0
KEEP_SOURCE=0
TEST_MODE="${LAY_UNINSTALL_TEST_MODE:-0}"
SYSTEM_ROOT="${LAY_UNINSTALL_SYSTEM_ROOT:-}"
MANAGED_RUNTIME_DIR="${LAY_INSTALL_LIBEXEC_DIR:-$HOME/.local/lib/lay/bin}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)"
SOURCE_DIR="${LAY_INSTALL_DIR:-$SCRIPT_DIR}"
if [ ! -f "$SOURCE_DIR/Cargo.toml" ]; then
    SOURCE_DIR="${LAY_INSTALL_DIR:-$HOME/projects/lay}"
fi

usage() {
    cat <<'EOF'
Использование: bash uninstall.sh [--purge] [--keep-source]

Без флагов удаляется runtime lay, но сохраняются настройки, память и исходники.
--purge       удалить также ~/.config/lay, данные, кэши, логи и чистую git-копию lay
--keep-source не удалять исходники даже вместе с --purge
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --purge) PURGE=1 ;;
        --keep-source) KEEP_SOURCE=1 ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 2 ;;
    esac
    shift
done

run_runtime_cleanup() {
    if [ "$TEST_MODE" = "1" ]; then
        return
    fi
    for unit in lay-daemon.service lay-l3-online.service lay-kde-tray.service lay-host-vm-guard.service lay-ibus-engine.service; do
        systemctl --user disable --now "$unit" >/dev/null 2>&1 || true
    done
    pkill -x lay-daemon >/dev/null 2>&1 || true
    pkill -x lay-ibus-engine >/dev/null 2>&1 || true
    pkill -f "$HOME/.local/bin/lay-kde-tray" >/dev/null 2>&1 || true
    pkill -f "$HOME/.local/bin/lay-host-vm-guard" >/dev/null 2>&1 || true
    gnome-extensions disable lay@radislabus-star.github.io >/dev/null 2>&1 || true
    if command -v ibus >/dev/null 2>&1; then
        case "$(ibus engine 2>/dev/null || true)" in
            lay-ime-ru|lay-ime-us) ibus engine xkb:us::eng >/dev/null 2>&1 || true ;;
        esac
    fi
}

remove_gnome_input_sources() {
    if [ "$TEST_MODE" = "1" ] || ! command -v gsettings >/dev/null 2>&1; then
        return
    fi
    sources="$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null || true)"
    filtered="$(python3 - "$sources" <<'PY' 2>/dev/null || true
import ast
import sys

try:
    sources = ast.literal_eval(sys.argv[1])
except Exception:
    raise SystemExit(0)

filtered = [tuple(item) for item in sources if tuple(item) not in {
    ("ibus", "lay-ime-ru"),
    ("ibus", "lay-ime-us"),
}]
print(repr(filtered) if filtered else "@a(ss) []")
PY
)"
    if [ -n "$filtered" ] && [ "$filtered" != "$sources" ]; then
        gsettings set org.gnome.desktop.input-sources sources "$filtered" >/dev/null 2>&1 || true
        gsettings set org.gnome.desktop.input-sources current 0 >/dev/null 2>&1 || true
    fi
}

remove_system_file() {
    path="$1"
    if [ -n "$SYSTEM_ROOT" ]; then
        rm -f "$SYSTEM_ROOT$path"
    elif command -v sudo >/dev/null 2>&1; then
        sudo rm -f "$path"
    fi
}

remove_managed_runtime() {
    local link raw_target canonical_target canonical_managed_dir
    canonical_managed_dir="$(readlink -f -- "$MANAGED_RUNTIME_DIR" 2>/dev/null || printf '%s' "$MANAGED_RUNTIME_DIR")"

    if [ -d "$HOME/.local/bin" ]; then
        while IFS= read -r -d '' link; do
            raw_target="$(readlink -- "$link" 2>/dev/null || true)"
            canonical_target="$(readlink -f -- "$link" 2>/dev/null || true)"
            case "$raw_target" in
                "$MANAGED_RUNTIME_DIR"/*) rm -f -- "$link"; continue ;;
            esac
            case "$canonical_target" in
                "$canonical_managed_dir"/*) rm -f -- "$link" ;;
            esac
        done < <(find "$HOME/.local/bin" -maxdepth 1 -type l -print0)
    fi

    rm -rf -- "$MANAGED_RUNTIME_DIR"
    rmdir -- "$HOME/.local/lib/lay" 2>/dev/null || true
}

echo "=== остановка lay ==="
run_runtime_cleanup

echo "=== удаление пользовательского runtime ==="
rm -f \
    "$HOME/.local/bin/lay" \
    "$HOME/.local/bin/lay-daemon" \
    "$HOME/.local/bin/lay-nanda-wave-eval" \
    "$HOME/.local/bin/lay-nanda-wave-train" \
    "$HOME/.local/bin/lay-test-input" \
    "$HOME/.local/bin/lay-ngram-corpus" \
    "$HOME/.local/bin/lay-ibus-engine" \
    "$HOME/.local/bin/lay-memory-report" \
    "$HOME/.local/bin/lay-l1.1-restore" \
    "$HOME/.local/bin/lay-l1.1-serve" \
    "$HOME/.local/bin/lay-runtime-control" \
    "$HOME/.local/bin/lay-kde-tray" \
    "$HOME/.local/bin/lay-host-vm-guard" \
    "$HOME/.local/bin/lay-nanda-train" \
    "$HOME/.local/bin/lay-nanda-eval" \
    "$HOME/.local/bin/lay-nanda-loop"
remove_managed_runtime
rm -f \
    "$HOME/.config/systemd/user/lay-daemon.service" \
    "$HOME/.config/systemd/user/lay-l3-online.service" \
    "$HOME/.config/systemd/user/lay-kde-tray.service" \
    "$HOME/.config/systemd/user/lay-host-vm-guard.service" \
    "$HOME/.config/systemd/user/lay-ibus-engine.service" \
    "$HOME/.config/autostart/lay-kde-tray.desktop" \
    "$HOME/.local/share/applications/io.github.radislabus_star.LaySettings.desktop" \
    "$HOME/.local/share/ibus/component/lay-ime.xml" \
    "$HOME/.local/share/ibus/component/lay.xml"
rm -rf \
    "$HOME/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io" \
    "$HOME/.cache/lay"
remove_gnome_input_sources
remove_system_file /usr/share/ibus/component/lay-ime.xml
remove_system_file /usr/share/ibus/component/lay.xml
remove_system_file /etc/udev/rules.d/99-lay-uinput.rules

if [ "$TEST_MODE" != "1" ]; then
    systemctl --user daemon-reload >/dev/null 2>&1 || true
    ibus write-cache >/dev/null 2>&1 || true
    if command -v sudo >/dev/null 2>&1; then
        sudo ibus write-cache --system >/dev/null 2>&1 || true
        sudo udevadm control --reload-rules >/dev/null 2>&1 || true
    fi
    update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
fi

if [ "$PURGE" = "1" ]; then
    echo "=== удаление настроек, памяти и логов ==="
    rm -rf \
        "$HOME/.config/lay" \
        "$HOME/.local/lib/lay" \
        "$HOME/.local/share/lay" \
        "$HOME/.local/state/lay"
fi

source_removed=0
if [ "$PURGE" = "1" ] && [ "$KEEP_SOURCE" = "0" ] && [ -d "$SOURCE_DIR/.git" ]; then
    if [ -n "$(git -C "$SOURCE_DIR" status --porcelain --untracked-files=normal 2>/dev/null || true)" ]; then
        echo "⚠ исходники оставлены: в $SOURCE_DIR есть локальные изменения"
    elif git -C "$SOURCE_DIR" remote -v 2>/dev/null | grep -q 'radislabus-star/lay-public'; then
        cd "$HOME"
        rm -rf "$SOURCE_DIR"
        source_removed=1
    else
        echo "⚠ исходники оставлены: $SOURCE_DIR не распознан как официальный lay checkout"
    fi
fi

echo ""
echo "✓ runtime lay удалён"
if [ "$PURGE" = "1" ]; then
    echo "✓ пользовательские настройки, память и логи удалены"
fi
if [ "$source_removed" = "1" ]; then
    echo "✓ исходники удалены: $SOURCE_DIR"
elif [ "$KEEP_SOURCE" = "1" ] || [ "$PURGE" = "0" ]; then
    echo "ℹ исходники сохранены: $SOURCE_DIR"
fi
echo "ℹ системные пакеты и членство в общей группе input не удалялись"
