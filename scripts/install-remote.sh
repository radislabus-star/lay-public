#!/usr/bin/env bash
# install-remote.sh — bootstrap installer for lay from GitHub.
set -euo pipefail

REPO_URL="${LAY_REPO_URL:-https://github.com/radislabus-star/lay-public.git}"
INSTALL_DIR="${LAY_INSTALL_DIR:-$HOME/projects/lay}"

install_apt_packages() {
    if ! command -v apt-get >/dev/null 2>&1; then
        return
    fi

    local packages=(
        git
        curl
        ca-certificates
        build-essential
        pkg-config
        libxcb1
        libxcb-shape0
        libxcb-xfixes0
        wl-clipboard
        xclip
    )

    if apt-cache show qdbus-qt6 >/dev/null 2>&1; then
        packages+=(qdbus-qt6)
    elif apt-cache show qdbus6 >/dev/null 2>&1; then
        packages+=(qdbus6)
    elif apt-cache show qdbus >/dev/null 2>&1; then
        packages+=(qdbus)
    fi

    local desktop_hint="${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}"
    if printf '%s' "$desktop_hint" | grep -Eiq 'kde|plasma' || pgrep -x plasmashell >/dev/null 2>&1; then
        packages+=(python3-pyqt6 libxcb-cursor0)
    fi

    echo "=== apt dependencies ==="
    sudo apt-get update
    sudo apt-get install -y "${packages[@]}"
}

install_rust() {
    if command -v cargo >/dev/null 2>&1; then
        echo "✓ cargo: $(cargo --version)"
        return
    fi

    echo "=== rustup ==="
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
    echo "✓ cargo: $(cargo --version)"
}

checkout_repo() {
    echo "=== lay source ==="
    if [ -d "$INSTALL_DIR/.git" ]; then
        git -C "$INSTALL_DIR" pull --ff-only
        return
    fi

    if [ -e "$INSTALL_DIR" ]; then
        echo "$INSTALL_DIR already exists but is not a git checkout." >&2
        echo "Move it away or set LAY_INSTALL_DIR=/another/path." >&2
        exit 1
    fi

    mkdir -p "$(dirname "$INSTALL_DIR")"
    git clone "$REPO_URL" "$INSTALL_DIR"
}

main() {
    install_apt_packages
    install_rust
    checkout_repo

    echo "=== install lay ==="
    cd "$INSTALL_DIR"
    bash install.sh

    echo ""
    echo "=== update command ==="
    echo "To update later:"
    echo "  cd \"$INSTALL_DIR\" && bash update.sh"
}

main "$@"
