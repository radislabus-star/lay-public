#!/bin/bash
# install.sh — собрать и установить lay + lay-daemon + GNOME extension
set -eu

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

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
echo "=== системные зависимости ==="
need_install=()
for pkg in libxcb1 libxcb-shape0 libxcb-xfixes0 wl-clipboard xclip; do
    if ! dpkg -l "$pkg" 2>/dev/null | grep -q '^ii'; then
        need_install+=("$pkg")
    fi
done
if [ ${#need_install[@]} -gt 0 ]; then
    echo "ставим: ${need_install[*]}"
    sudo apt-get install -y "${need_install[@]}"
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
echo "✓ lay        → ~/.local/bin/lay"
echo "✓ lay-daemon → ~/.local/bin/lay-daemon"
echo "✓ lay-ngram-corpus → ~/.local/bin/lay-ngram-corpus"

echo ""
echo "=== systemd unit для lay-daemon ==="
mkdir -p ~/.config/systemd/user
cp "$DIR/systemd/lay-daemon.service" ~/.config/systemd/user/lay-daemon.service
systemctl --user daemon-reload
systemctl --user enable lay-daemon
echo "✓ lay-daemon.service установлен и включён"

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
echo "║  Перелогинься в GNOME чтобы:             ║"
echo "║  • extension загрузился (EN/RU в трее)   ║"
echo "║  • lay-daemon запустился автоматически   ║"
echo "║                                          ║"
echo "║  Двойной Shift = конвертировать слово    ║"
echo "║                                          ║"
echo "║  Обновление:                             ║"
echo "║  cd ~/projects/lay && bash update.sh     ║"
echo "╚══════════════════════════════════════════╝"
