#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "== Bazzite/rpm-ostree platform route =="
platform="$(LAY_PACKAGE_MANAGER_OVERRIDE=rpm-ostree bash "$ROOT/install.sh" --check-platform)"
grep -q '^package_manager=rpm-ostree$' <<<"$platform"
remote_platform="$(LAY_PACKAGE_MANAGER_OVERRIDE=rpm-ostree bash "$ROOT/scripts/install-remote.sh" --check-platform)"
grep -q '^package_manager=rpm-ostree$' <<<"$remote_platform"

echo "== dirty checkout update preservation =="
git init --bare -q "$TMP/origin.git"
git init -q -b main "$TMP/seed"
cp "$ROOT/update.sh" "$TMP/seed/update.sh"
printf '[package]\nname = "lay-update-fixture"\nversion = "0.0.1"\n' > "$TMP/seed/Cargo.toml"
git -C "$TMP/seed" add update.sh Cargo.toml
git -C "$TMP/seed" -c user.name=lay-test -c user.email=lay@example.invalid commit -qm initial
git -C "$TMP/seed" remote add origin "$TMP/origin.git"
git -C "$TMP/seed" push -qu origin main
git --git-dir="$TMP/origin.git" symbolic-ref HEAD refs/heads/main
git clone -q "$TMP/origin.git" "$TMP/client"
printf '\n# local user change\n' >> "$TMP/client/Cargo.toml"
printf 'new upstream\n' > "$TMP/seed/release.txt"
git -C "$TMP/seed" add release.txt
git -C "$TMP/seed" -c user.name=lay-test -c user.email=lay@example.invalid commit -qm update
git -C "$TMP/seed" push -qu origin main
HOME="$TMP/home-update" bash "$TMP/client/update.sh" --source-only >/dev/null
test -f "$TMP/client/release.txt"
git -C "$TMP/client" stash list | grep -q 'lay-auto-update-'
git -C "$TMP/client" stash show -p | grep -q 'local user change'

echo "== complete uninstall sandbox =="
HOME_DIR="$TMP/home-uninstall"
SYSTEM_ROOT="$TMP/system-root"
mkdir -p \
    "$HOME_DIR/.local/bin" \
    "$HOME_DIR/.config/systemd/user" \
    "$HOME_DIR/.config/lay" \
    "$HOME_DIR/.local/share/lay" \
    "$HOME_DIR/.local/state/lay" \
    "$HOME_DIR/.cache/lay" \
    "$HOME_DIR/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io" \
    "$SYSTEM_ROOT/usr/share/ibus/component" \
    "$SYSTEM_ROOT/etc/udev/rules.d"
touch \
    "$HOME_DIR/.local/bin/lay" \
    "$HOME_DIR/.local/bin/lay-daemon" \
    "$HOME_DIR/.config/systemd/user/lay-daemon.service" \
    "$HOME_DIR/.config/lay/config.json" \
    "$HOME_DIR/.local/share/lay/recent_actions.jsonl" \
    "$HOME_DIR/.local/state/lay/update.log" \
    "$SYSTEM_ROOT/usr/share/ibus/component/lay-ime.xml" \
    "$SYSTEM_ROOT/etc/udev/rules.d/99-lay-uinput.rules"
HOME="$HOME_DIR" \
LAY_UNINSTALL_TEST_MODE=1 \
LAY_UNINSTALL_SYSTEM_ROOT="$SYSTEM_ROOT" \
bash "$ROOT/uninstall.sh" --purge --keep-source >/dev/null
test ! -e "$HOME_DIR/.local/bin/lay"
test ! -e "$HOME_DIR/.config/systemd/user/lay-daemon.service"
test ! -e "$HOME_DIR/.config/lay"
test ! -e "$HOME_DIR/.local/share/lay"
test ! -e "$SYSTEM_ROOT/usr/share/ibus/component/lay-ime.xml"
test ! -e "$SYSTEM_ROOT/etc/udev/rules.d/99-lay-uinput.rules"

echo "== compact wave memory contract =="
! rg -q 'prefix_index\s*:' "$ROOT/src/nanda_wave/context_wave.rs"
rg -q 'warm_up_prepares_wave_memory_without_prefix_indexes' \
    "$ROOT/src/nanda_wave/context_wave.rs"

echo "public issue regressions: PASS"
