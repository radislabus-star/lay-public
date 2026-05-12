#!/bin/bash
# install.sh — собрать и установить lay + lay-daemon + GNOME extension
set -eu

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
    if command -v apt-get >/dev/null 2>&1; then
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
        dnf|yum) rpm -q "$pkg" >/dev/null 2>&1 ;;
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
        dnf|yum)
            sudo "$pm" install -y "$@"
            ;;
        *)
            echo "Не найден поддерживаемый менеджер пакетов: apt, pacman, dnf или yum" >&2
            return 1
            ;;
    esac
}

base_packages_for_pm() {
    pm="$1"
    case "$pm" in
        apt) echo "libxcb1 libxcb-shape0 libxcb-xfixes0 wl-clipboard xclip" ;;
        pacman) echo "libxcb wl-clipboard xclip" ;;
        dnf|yum) echo "libxcb wl-clipboard xclip" ;;
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
        dnf|yum) echo "qt6-qttools python3-qt6 xcb-util-cursor" ;;
        *) echo "" ;;
    esac
}

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
echo "=== системные зависимости ==="
pm="$(detect_package_manager)"
if [ "$pm" = none ]; then
    echo "⚠ менеджер пакетов не найден; пропускаю автоустановку зависимостей"
    echo "  поддерживаются apt, pacman, dnf и yum"
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
    install_packages "$pm" "${need_install[@]}"
else
    echo "✓ все пакеты уже стоят"
fi

echo ""
echo "=== сборка release ==="
cargo build --release --quiet
echo "✓ lay:        $(ls -lh target/release/lay | awk '{print $5}')"
echo "✓ lay-daemon: $(ls -lh target/release/lay-daemon | awk '{print $5}')"

echo ""
echo "=== n-gram cache ==="
target/release/lay-ngram-corpus cache >/tmp/lay-ngram-cache-install.log 2>&1 || {
    cat /tmp/lay-ngram-cache-install.log
    echo "⚠ n-gram cache не собран; daemon соберёт fallback при первом вызове"
}
if [ -f "$HOME/.cache/lay/ngram_ru_v1.json" ]; then
    echo "✓ $(ls -lh "$HOME/.cache/lay/ngram_ru_v1.json" | awk '{print $9 ": " $5}')"
fi

echo ""
echo "=== симлинки в ~/.local/bin/ ==="
mkdir -p ~/.local/bin
ln -sf "$DIR/target/release/lay" ~/.local/bin/lay
ln -sf "$DIR/target/release/lay-daemon" ~/.local/bin/lay-daemon
ln -sf "$DIR/target/release/lay-ngram-corpus" ~/.local/bin/lay-ngram-corpus
ln -sf "$DIR/scripts/lay-kde-tray.py" ~/.local/bin/lay-kde-tray
ln -sf "$DIR/scripts/lay-host-vm-guard.sh" ~/.local/bin/lay-host-vm-guard
echo "✓ lay        → ~/.local/bin/lay"
echo "✓ lay-daemon → ~/.local/bin/lay-daemon"
echo "✓ lay-ngram-corpus → ~/.local/bin/lay-ngram-corpus"
echo "✓ lay-kde-tray → ~/.local/bin/lay-kde-tray"
echo "✓ lay-host-vm-guard → ~/.local/bin/lay-host-vm-guard"

echo ""
echo "=== systemd unit для lay-daemon ==="
mkdir -p ~/.config/systemd/user
cp "$DIR/systemd/lay-daemon.service" ~/.config/systemd/user/lay-daemon.service
cp "$DIR/systemd/lay-kde-tray.service" ~/.config/systemd/user/lay-kde-tray.service
cp "$DIR/systemd/lay-host-vm-guard.service" ~/.config/systemd/user/lay-host-vm-guard.service
systemctl --user daemon-reload
systemctl --user enable lay-daemon
echo "✓ lay-daemon.service установлен и включён"
if is_kde_available; then
    install_kde_autostart
    systemctl --user disable lay-kde-tray.service >/dev/null 2>&1 || true
    if is_kde_session; then
        pgrep -f "$HOME/.local/bin/lay-kde-tray" >/dev/null 2>&1 || {
            nohup "$HOME/.local/bin/lay-kde-tray" >/tmp/lay-kde-tray.log 2>&1 &
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
mkdir -p "$DST"
cp "$DIR/extension/$UUID/metadata.json" "$DST/"
cp "$DIR/extension/$UUID/extension.js" "$DST/"
cp "$DIR/extension/$UUID/lay-impl.js" "$DST/"
gnome-extensions enable "$UUID" 2>/dev/null || true
echo "✓ extension установлен: $DST"

echo ""
echo "=== optional LLM backends ==="
echo "По умолчанию lay не требует Ollama/GGUF и не загружает модель."
if command -v ollama >/dev/null; then
    echo "✓ ollama: $(ollama --version 2>/dev/null | head -1)"
    if ollama list 2>/dev/null | grep -q "smollm:135m"; then
        echo "✓ optional модель smollm:135m уже есть"
    else
        echo "ℹ optional LLM режим не установлен"
        echo "  если нужен эксперимент: ollama pull smollm:135m"
    fi
else
    echo "ℹ ollama не установлен; это нормально для обычного double Shift"
fi

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
echo "╚══════════════════════════════════════════╝"
