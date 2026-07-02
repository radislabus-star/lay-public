#!/usr/bin/env bash
# install-remote.sh — bootstrap installer for lay from GitHub.
set -euo pipefail

REPO_URL="${LAY_REPO_URL:-https://github.com/radislabus-star/lay-public.git}"
INSTALL_DIR="${LAY_INSTALL_DIR:-$HOME/projects/lay}"

detect_package_manager() {
    if command -v rpm-ostree >/dev/null 2>&1 && [ -d /run/ostree-booted ]; then
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

kde_available() {
    local desktop_hint="${XDG_CURRENT_DESKTOP:-}:${XDG_SESSION_DESKTOP:-}:${DESKTOP_SESSION:-}"
    printf '%s' "$desktop_hint" | grep -Eiq 'kde|plasma' \
        || pgrep -x plasmashell >/dev/null 2>&1 \
        || command -v plasmashell >/dev/null 2>&1 \
        || [ -d /usr/share/plasma ]
}

apt_qdbus_package() {
    if apt-cache show qdbus-qt6 >/dev/null 2>&1; then
        echo qdbus-qt6
    elif apt-cache show qdbus6 >/dev/null 2>&1; then
        echo qdbus6
    else
        echo qdbus
    fi
}

rpm_ostree_dependencies_available() {
    command -v git >/dev/null 2>&1 \
        && command -v curl >/dev/null 2>&1 \
        && command -v gcc >/dev/null 2>&1 \
        && command -v make >/dev/null 2>&1 \
        && { command -v pkg-config >/dev/null 2>&1 || command -v pkgconf >/dev/null 2>&1; } \
        && command -v wl-copy >/dev/null 2>&1 \
        && command -v xclip >/dev/null 2>&1
}

install_system_packages() {
    local pm
    pm="$(detect_package_manager)"
    if [ "$pm" = none ]; then
        echo "No supported package manager found. Install git, curl, build tools and XCB deps manually." >&2
        return
    fi

    local packages=()
    case "$pm" in
        apt)
            packages=(git curl ca-certificates build-essential pkg-config libxcb1 libxcb-shape0 libxcb-xfixes0 wl-clipboard xclip)
            if kde_available; then
                packages+=("$(apt_qdbus_package)" python3-pyqt6 libxcb-cursor0)
            fi
            ;;
        pacman)
            packages=(git curl base-devel pkgconf libxcb wl-clipboard xclip)
            if kde_available; then
                packages+=(qt6-tools python-pyqt6 xcb-util-cursor)
            fi
            ;;
        rpm-ostree)
            packages=(git curl gcc gcc-c++ make pkgconf-pkg-config libxcb wl-clipboard xclip)
            if kde_available; then
                packages+=(qt6-qttools python3-qt6 xcb-util-cursor)
            fi
            ;;
        dnf|yum)
            packages=(git curl gcc gcc-c++ make pkgconf-pkg-config libxcb wl-clipboard xclip)
            if kde_available; then
                packages+=(qt6-qttools python3-qt6 xcb-util-cursor)
            fi
            ;;
    esac

    echo "=== system dependencies ($pm) ==="
    case "$pm" in
        apt)
            sudo apt-get update
            sudo apt-get install -y "${packages[@]}"
            ;;
        pacman)
            sudo pacman -Sy --needed --noconfirm "${packages[@]}"
            ;;
        rpm-ostree)
            if rpm_ostree_dependencies_available; then
                echo "=== system dependencies (rpm-ostree) ==="
                echo "✓ required build/runtime commands are already available"
                return
            fi
            echo "Detected an rpm-ostree based system, such as Bazzite/Fedora Atomic."
            echo "System packages must be layered into the next deployment."
            sudo rpm-ostree install --idempotent -y "${packages[@]}"
            echo ""
            echo "rpm-ostree package layering is prepared."
            echo "Reboot, then run this installer again:"
            echo "  curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash"
            exit 0
            ;;
        dnf|yum)
            sudo "$pm" install -y "${packages[@]}"
            ;;
    esac
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
    install_system_packages
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
