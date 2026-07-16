#!/bin/bash
# install.sh — собрать и установить lay + lay-daemon + GNOME extension
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

is_kde_session() {
    desktop_hint="${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}"
    printf '%s' "$desktop_hint" | grep -Eiq 'kde|plasma' || pgrep -x plasmashell >/dev/null 2>&1
}

is_kde_available() {
    is_kde_session || command -v plasmashell >/dev/null 2>&1 || [ -d /usr/share/plasma ]
}

install_kde_autostart() {
    mkdir -p "$HOME/.config/autostart"
    cat > "$HOME/.config/autostart/lay-kde-tray.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Lay
Comment=RU/EN layout helper tray
Exec=$HOME/.local/bin/lay-kde-tray
Icon=input-keyboard
Terminal=false
X-KDE-autostart-after=panel
X-KDE-autostart-phase=2
OnlyShowIn=KDE;
EOF
}

detect_package_manager() {
    if [ -n "${LAY_PACKAGE_MANAGER_OVERRIDE:-}" ]; then
        echo "$LAY_PACKAGE_MANAGER_OVERRIDE"
    elif command -v rpm-ostree >/dev/null 2>&1 && [ -d "${LAY_OSTREE_BOOTED_DIR:-/run/ostree-booted}" ]; then
        echo rpm-ostree
    elif command -v apt-get >/dev/null 2>&1; then
        echo apt
    elif command -v pacman >/dev/null 2>&1; then
        echo pacman
    elif command -v dnf >/dev/null 2>&1; then
        echo dnf
    elif command -v yum >/dev/null 2>&1; then
        echo yum
    else
        echo none
    fi
}

package_installed() {
    pm="$1"
    pkg="$2"
    case "$pm" in
        apt) dpkg -l "$pkg" 2>/dev/null | grep -q '^ii' ;;
        pacman) pacman -Qi "$pkg" >/dev/null 2>&1 ;;
        rpm-ostree|dnf|yum) rpm -q "$pkg" >/dev/null 2>&1 ;;
        *) return 1 ;;
    esac
}

install_packages() {
    pm="$1"
    shift
    case "$pm" in
        apt)
            sudo apt-get update
            sudo apt-get install -y "$@"
            ;;
        pacman)
            sudo pacman -Sy --needed --noconfirm "$@"
            ;;
        rpm-ostree)
            sudo rpm-ostree install --idempotent -y "$@"
            LAY_REBOOT_REQUIRED=1
            ;;
        dnf|yum)
            sudo "$pm" install -y "$@"
            ;;
        *)
            echo "Не найден поддерживаемый менеджер пакетов: apt, pacman, rpm-ostree, dnf или yum" >&2
            return 1
            ;;
    esac
}

cleanup_legacy_ollama() {
    if [ "${LAY_KEEP_OLLAMA:-0}" = "1" ]; then
        echo "ℹ LAY_KEEP_OLLAMA=1 — legacy Ollama cleanup пропущен"
        return
    fi

    found=0
    if command -v ollama >/dev/null 2>&1; then
        found=1
    fi
    if systemctl list-unit-files 'ollama.service' --no-pager 2>/dev/null | grep -q '^ollama.service'; then
        found=1
    fi
    if [ -d /usr/share/ollama ] || [ -d "$HOME/.ollama" ]; then
        found=1
    fi
    if [ "$found" = "0" ]; then
        echo "✓ legacy Ollama не найдена"
        return
    fi

    echo "→ удаляю legacy Ollama из старых lay-установок..."
    systemctl --user stop ollama.service >/dev/null 2>&1 || true
    systemctl --user disable ollama.service >/dev/null 2>&1 || true
    sudo systemctl stop ollama.service >/dev/null 2>&1 || true
    sudo systemctl disable ollama.service >/dev/null 2>&1 || true

    if [ -f /etc/systemd/system/ollama.service ]; then
        sudo rm -f /etc/systemd/system/ollama.service
    fi
    sudo rm -f /etc/systemd/system/default.target.wants/ollama.service
    sudo systemctl daemon-reload >/dev/null 2>&1 || true

    sudo rm -f /usr/local/bin/ollama /usr/bin/ollama
    sudo rm -rf /usr/share/ollama /var/lib/ollama
    rm -rf "$HOME/.ollama"
    echo "✓ legacy Ollama удалена; lay теперь работает через deterministic/NANDA pipeline"
}

base_packages_for_pm() {
    pm="$1"
    case "$pm" in
        apt) echo "libxcb1 libxcb-shape0 libxcb-xfixes0 wl-clipboard xclip ibus gir1.2-ibus-1.0 python3-gi" ;;
        pacman) echo "libxcb wl-clipboard xclip ibus python-gobject" ;;
        rpm-ostree|dnf|yum) echo "libxcb wl-clipboard xclip ibus python3-gobject" ;;
        *) echo "" ;;
    esac
}

kde_packages_for_pm() {
    pm="$1"
    case "$pm" in
        apt)
            if apt-cache show qdbus-qt6 >/dev/null 2>&1; then
                echo "qdbus-qt6 python3-pyqt6 libxcb-cursor0"
            elif apt-cache show qdbus6 >/dev/null 2>&1; then
                echo "qdbus6 python3-pyqt6 libxcb-cursor0"
            else
                echo "qdbus python3-pyqt6 libxcb-cursor0"
            fi
            ;;
        pacman) echo "qt6-tools python-pyqt6 xcb-util-cursor" ;;
        rpm-ostree|dnf|yum) echo "qt6-qttools python3-qt6 xcb-util-cursor" ;;
        *) echo "" ;;
    esac
}

if [ "${1:-}" = "--check-platform" ]; then
    pm="$(detect_package_manager)"
    echo "package_manager=$pm"
    echo "base_packages=$(base_packages_for_pm "$pm")"
    echo "kde_packages=$(kde_packages_for_pm "$pm")"
    exit 0
elif [ "$#" -gt 0 ]; then
    echo "Использование: bash install.sh [--check-platform]" >&2
    exit 2
fi

echo "=== проверка cargo ==="
if ! command -v cargo >/dev/null; then
    if [ -f "$HOME/.cargo/env" ]; then
        . "$HOME/.cargo/env"
    fi
fi
if ! command -v cargo >/dev/null; then
    echo "rust не установлен. Поставь:" >&2
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" >&2
    exit 1
fi
echo "✓ $(rustc --version)"

echo ""
echo "=== группа input (нужна для evdev) ==="
if id -nG "$USER" | grep -qw input; then
    echo "✓ уже в группе input"
else
    echo "→ добавляю $USER в группу input..."
    sudo usermod -aG input "$USER"
    echo "⚠ нужен перелогин чтобы группа применилась"
fi

echo ""
echo "=== uinput permissions (нужно для обратной печати) ==="
UINPUT_RULE='/etc/udev/rules.d/99-lay-uinput.rules'
UINPUT_RULE_TEXT='KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"'
if [ -e /dev/uinput ] && [ -w /dev/uinput ]; then
    echo "✓ /dev/uinput доступен"
else
    echo "→ настраиваю /dev/uinput для группы input..."
    printf '%s\n' "$UINPUT_RULE_TEXT" | sudo tee "$UINPUT_RULE" >/dev/null
    sudo modprobe uinput 2>/dev/null || true
    sudo udevadm control --reload-rules 2>/dev/null || true
    sudo udevadm trigger --subsystem-match=misc 2>/dev/null || true
    if [ -e /dev/uinput ]; then
        sudo chgrp input /dev/uinput 2>/dev/null || true
        sudo chmod 0660 /dev/uinput 2>/dev/null || true
    fi
    if [ -e /dev/uinput ] && [ -w /dev/uinput ]; then
        echo "✓ /dev/uinput доступен"
    else
        echo "⚠ /dev/uinput пока недоступен; нужен перелогин или перезагрузка"
    fi
fi

echo ""
echo "=== legacy Ollama cleanup ==="
cleanup_legacy_ollama

echo ""
echo "=== системные зависимости ==="
pm="$(detect_package_manager)"
if [ "$pm" = none ]; then
    echo "⚠ менеджер пакетов не найден; пропускаю автоустановку зависимостей"
    echo "  поддерживаются apt, pacman, rpm-ostree, dnf и yum"
else
    echo "✓ package manager: $pm"
fi
need_install=()
for pkg in $(base_packages_for_pm "$pm"); do
    if ! package_installed "$pm" "$pkg"; then
        need_install+=("$pkg")
    fi
done
if is_kde_available; then
    for pkg in $(kde_packages_for_pm "$pm"); do
        if ! package_installed "$pm" "$pkg"; then
            need_install+=("$pkg")
        fi
    done
fi
if [ ${#need_install[@]} -gt 0 ]; then
    echo "ставим: ${need_install[*]}"
    LAY_REBOOT_REQUIRED=0
    install_packages "$pm" "${need_install[@]}"
    if [ "$LAY_REBOOT_REQUIRED" = "1" ]; then
        echo ""
        echo "✓ rpm-ostree зависимости добавлены в следующий deployment."
        echo "Перезагрузите Bazzite/Fedora Atomic и повторите установку lay."
        exit 0
    fi
else
    echo "✓ все пакеты уже стоят"
fi

echo ""
echo "=== сборка release ==="
scripts/cargo-guard.sh build --release --bins --quiet
echo "✓ lay:        $(ls -lh target/release/lay | awk '{print $5}')"
echo "✓ lay-daemon: $(ls -lh target/release/lay-daemon | awk '{print $5}')"

echo ""
echo "=== L2 transition phase package ==="
scripts/install-l2-transition-phase.sh
scripts/install-l2-lexical-phase.sh

echo ""
echo "=== L3 phrase memory ==="
target/release/lay-nanda-wave-eval --llmwave-pack-live

echo ""
echo "=== n-gram cache ==="
LAY_STATE_DIR="$HOME/.local/state/lay"
mkdir -p "$LAY_STATE_DIR"
LAY_NGRAM_INSTALL_LOG="$LAY_STATE_DIR/ngram-cache-install.log"
LAY_KDE_TRAY_LOG="$LAY_STATE_DIR/kde-tray.log"
target/release/lay-ngram-corpus cache >"$LAY_NGRAM_INSTALL_LOG" 2>&1 || {
    cat "$LAY_NGRAM_INSTALL_LOG"
    echo "⚠ n-gram cache не собран; daemon соберёт fallback при первом вызове"
}
if [ -f "$HOME/.cache/lay/ngram_ru_v1.json" ]; then
    echo "✓ $(ls -lh "$HOME/.cache/lay/ngram_ru_v1.json" | awk '{print $9 ": " $5}')"
fi

echo ""
echo "=== установка release в ~/.local/lib/lay/bin/ ==="
scripts/install-release-binaries.sh
ln -sf "$DIR/scripts/lay-runtime-control.sh" ~/.local/bin/lay-runtime-control
ln -sf "$DIR/scripts/lay-kde-tray.py" ~/.local/bin/lay-kde-tray
ln -sf "$DIR/scripts/lay-host-vm-guard.sh" ~/.local/bin/lay-host-vm-guard
rm -f ~/.local/bin/lay-nanda-train ~/.local/bin/lay-nanda-eval ~/.local/bin/lay-nanda-loop
echo "✓ lay        → ~/.local/bin/lay"
echo "✓ lay-daemon → ~/.local/bin/lay-daemon"
echo "✓ lay-nanda-wave-eval → ~/.local/bin/lay-nanda-wave-eval"
echo "✓ lay-test-input → ~/.local/bin/lay-test-input"
echo "✓ lay-ngram-corpus → ~/.local/bin/lay-ngram-corpus"
echo "✓ lay-memory-report → ~/.local/bin/lay-memory-report"
echo "✓ lay-kde-tray → ~/.local/bin/lay-kde-tray"
echo "✓ lay-ibus-engine → ~/.local/bin/lay-ibus-engine"
echo "✓ lay-runtime-control → ~/.local/bin/lay-runtime-control"
echo "✓ lay-host-vm-guard → ~/.local/bin/lay-host-vm-guard"

echo ""
echo "=== desktop entry для окна настроек ==="
mkdir -p "$HOME/.local/share/applications"
sed "s|/home/ubu|$HOME|g" "$DIR/extension/lay-settings.desktop" \
    > "$HOME/.local/share/applications/io.github.radislabus_star.LaySettings.desktop"
update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
echo "✓ settings launcher: ~/.local/share/applications/io.github.radislabus_star.LaySettings.desktop"

echo ""
echo "=== optional IBus bridge ==="
mkdir -p "$HOME/.local/share/ibus/component"
IBUS_COMPONENT_XML="$HOME/.local/share/ibus/component/lay-ime.xml"
"$HOME/.local/bin/lay-ibus-engine" --xml > "$IBUS_COMPONENT_XML"
rm -f "$HOME/.local/share/ibus/component/lay.xml"
if [ -d /usr/share/ibus/component ] && command -v sudo >/dev/null 2>&1; then
    sudo cp "$IBUS_COMPONENT_XML" /usr/share/ibus/component/lay-ime.xml 2>/dev/null || true
    sudo rm -f /usr/share/ibus/component/lay.xml 2>/dev/null || true
    sudo ibus write-cache --system 2>/dev/null || true
fi
ibus write-cache 2>/dev/null || true
echo "✓ IBus component установлен: ~/.local/share/ibus/component/lay-ime.xml"
echo "✓ старый IBus component lay.xml удалён"
echo "  Экспериментальный режим: выбрать Lay IME RU/Lay IME US в IBus и поставить text_backend=ime"

echo ""
echo "=== systemd unit для lay-daemon ==="
mkdir -p ~/.config/systemd/user
cp "$DIR/systemd/lay-daemon.service" ~/.config/systemd/user/lay-daemon.service
cp "$DIR/systemd/lay-kde-tray.service" ~/.config/systemd/user/lay-kde-tray.service
cp "$DIR/systemd/lay-host-vm-guard.service" ~/.config/systemd/user/lay-host-vm-guard.service
systemctl --user daemon-reload
systemctl --user disable --now lay-ibus-engine.service >/dev/null 2>&1 || true
systemctl --user enable lay-daemon
echo "✓ lay-daemon.service установлен и включён"
echo "✓ старый lay-ibus-engine.service отключён; IME запускает IBus"
if is_kde_available; then
    install_kde_autostart
    systemctl --user disable lay-kde-tray.service >/dev/null 2>&1 || true
    if is_kde_session; then
        pgrep -f "$HOME/.local/bin/lay-kde-tray" >/dev/null 2>&1 || {
            nohup "$HOME/.local/bin/lay-kde-tray" >"$LAY_KDE_TRAY_LOG" 2>&1 &
        }
    fi
    echo "✓ KDE tray autostart установлен: ~/.config/autostart/lay-kde-tray.desktop"
else
    systemctl --user disable lay-kde-tray.service >/dev/null 2>&1 || true
    rm -f "$HOME/.config/autostart/lay-kde-tray.desktop"
    echo "ℹ KDE tray установлен, но autostart отключён вне KDE"
fi
if systemctl --user is-enabled --quiet lay-host-vm-guard.service 2>/dev/null; then
    systemctl --user restart lay-host-vm-guard.service || true
    echo "✓ lay-host-vm-guard.service обновлён и перезапущен"
else
    echo "ℹ VM guard установлен, но не включён автоматически"
    echo "  для тестов VM: systemctl --user enable --now lay-host-vm-guard.service"
fi

echo ""
echo "=== GNOME Shell extension ==="
UUID="lay@radislabus-star.github.io"
DST="$HOME/.local/share/gnome-shell/extensions/$UUID"
LAY_GJS_CACHE="$HOME/.cache/lay"
mkdir -p "$DST"
mkdir -p "$LAY_GJS_CACHE"
cp "$DIR/extension/$UUID/metadata.json" "$DST/"
cp "$DIR/extension/$UUID/"*.js "$DST/"
for js in "$DIR/extension/$UUID/"*.js; do
    name="$(basename "$js")"
    if [ "$name" != "extension.js" ] && [ "$name" != "lay-impl.js" ]; then
        cp "$js" "$LAY_GJS_CACHE/$name"
    fi
done
gnome-extensions enable "$UUID" 2>/dev/null || true
echo "✓ extension установлен: $DST"

echo ""
echo "=== быстрый тест CLI ==="
~/.local/bin/lay "Ye djn ghbvth"
~/.local/bin/lay "руддщ цщкдв"

echo ""
echo "╔══════════════════════════════════════════╗"
echo "║  Установка завершена!                    ║"
echo "║                                          ║"
echo "║  Перелогинься в desktop-сессию чтобы:    ║"
echo "║  • GNOME/KDE tray загрузился             ║"
echo "║  • lay-daemon запустился автоматически   ║"
echo "║                                          ║"
echo "║  Двойной Shift = конвертировать слово    ║"
echo "║                                          ║"
echo "║  Обновление:                             ║"
echo "║  cd ~/projects/lay && bash update.sh     ║"
echo "║  Удаление: bash uninstall.sh --purge     ║"
echo "╚══════════════════════════════════════════╝"
