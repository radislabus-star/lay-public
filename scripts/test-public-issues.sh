#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

search_quiet() {
    local pattern="$1"
    local file="$2"
    if command -v rg >/dev/null 2>&1; then
        rg -q "$pattern" "$file"
    else
        grep -Eq "$pattern" "$file"
    fi
}

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

echo "== issue 41 managed runtime ownership =="
release_names=(
    lay
    lay-daemon
    lay-nanda-wave-eval
    lay-nanda-wave-train
    lay-test-input
    lay-ngram-corpus
    lay-ibus-engine
    lay-memory-report
    lay-l1.1-restore
    lay-l1.1-serve
)

NORMAL_HOME="$TMP/home-uninstall-normal"
NORMAL_MANAGED="$NORMAL_HOME/.local/lib/lay/bin"
mkdir -p \
    "$NORMAL_HOME/.local/bin" \
    "$NORMAL_MANAGED" \
    "$NORMAL_HOME/.local/lib/lay/rollback" \
    "$NORMAL_HOME/unrelated"
touch \
    "$NORMAL_HOME/.local/lib/lay/rollback/keep-me" \
    "$NORMAL_HOME/unrelated/lay-helper"
ln -s "$NORMAL_HOME/unrelated/lay-helper" "$NORMAL_HOME/.local/bin/lay-unrelated"
for name in "${release_names[@]}"; do
    touch "$NORMAL_MANAGED/$name"
    ln -s "$NORMAL_MANAGED/$name" "$NORMAL_HOME/.local/bin/$name"
done
touch "$NORMAL_MANAGED/lay-future-release-tool"
ln -s \
    "$NORMAL_MANAGED/lay-future-release-tool" \
    "$NORMAL_HOME/.local/bin/lay-future-release-tool"

HOME="$NORMAL_HOME" \
LAY_UNINSTALL_TEST_MODE=1 \
bash "$ROOT/uninstall.sh" --keep-source >/dev/null
for name in "${release_names[@]}"; do
    test ! -e "$NORMAL_HOME/.local/bin/$name"
    test ! -L "$NORMAL_HOME/.local/bin/$name"
done
test ! -e "$NORMAL_HOME/.local/bin/lay-future-release-tool"
test ! -L "$NORMAL_HOME/.local/bin/lay-future-release-tool"
test ! -e "$NORMAL_MANAGED"
test -e "$NORMAL_HOME/.local/lib/lay/rollback/keep-me"
test -L "$NORMAL_HOME/.local/bin/lay-unrelated"
test "$(readlink "$NORMAL_HOME/.local/bin/lay-unrelated")" = "$NORMAL_HOME/unrelated/lay-helper"

# A repeated uninstall must stay inside the same ownership boundary.
HOME="$NORMAL_HOME" \
LAY_UNINSTALL_TEST_MODE=1 \
bash "$ROOT/uninstall.sh" --keep-source >/dev/null
test -e "$NORMAL_HOME/.local/lib/lay/rollback/keep-me"
test -L "$NORMAL_HOME/.local/bin/lay-unrelated"

CUSTOM_HOME="$TMP/home-uninstall-custom"
CUSTOM_MANAGED="$CUSTOM_HOME/custom/libexec"
mkdir -p "$CUSTOM_HOME/.local/bin" "$CUSTOM_MANAGED"
touch "$CUSTOM_MANAGED/lay-custom-owner"
ln -s "$CUSTOM_MANAGED/lay-custom-owner" "$CUSTOM_HOME/.local/bin/lay-custom-owner"
HOME="$CUSTOM_HOME" \
LAY_INSTALL_LIBEXEC_DIR="$CUSTOM_MANAGED" \
LAY_UNINSTALL_TEST_MODE=1 \
bash "$ROOT/uninstall.sh" --keep-source >/dev/null
test ! -e "$CUSTOM_MANAGED"
test ! -e "$CUSTOM_HOME/.local/bin/lay-custom-owner"
test ! -L "$CUSTOM_HOME/.local/bin/lay-custom-owner"

echo "== complete purge sandbox =="
HOME_DIR="$TMP/home-uninstall"
SYSTEM_ROOT="$TMP/system-root"
mkdir -p \
    "$HOME_DIR/.local/bin" \
    "$HOME_DIR/.local/lib/lay/bin" \
    "$HOME_DIR/.local/lib/lay/rollback" \
    "$HOME_DIR/.local/lib/lay/staging" \
    "$HOME_DIR/.config/systemd/user" \
    "$HOME_DIR/.config/lay" \
    "$HOME_DIR/.local/share/lay" \
    "$HOME_DIR/.local/state/lay" \
    "$HOME_DIR/.cache/lay" \
    "$HOME_DIR/.local/share/gnome-shell/extensions/lay@radislabus-star.github.io" \
    "$SYSTEM_ROOT/usr/share/ibus/component" \
    "$SYSTEM_ROOT/etc/udev/rules.d"
touch \
    "$HOME_DIR/.local/lib/lay/rollback/old-runtime" \
    "$HOME_DIR/.local/lib/lay/staging/staged-runtime" \
    "$HOME_DIR/.config/systemd/user/lay-daemon.service" \
    "$HOME_DIR/.config/lay/config.json" \
    "$HOME_DIR/.local/share/lay/recent_actions.jsonl" \
    "$HOME_DIR/.local/state/lay/update.log" \
    "$SYSTEM_ROOT/usr/share/ibus/component/lay-ime.xml" \
    "$SYSTEM_ROOT/etc/udev/rules.d/99-lay-uinput.rules"
for name in "${release_names[@]}"; do
    touch "$HOME_DIR/.local/lib/lay/bin/$name"
    ln -s "$HOME_DIR/.local/lib/lay/bin/$name" "$HOME_DIR/.local/bin/$name"
done
HOME="$HOME_DIR" \
LAY_UNINSTALL_TEST_MODE=1 \
LAY_UNINSTALL_SYSTEM_ROOT="$SYSTEM_ROOT" \
bash "$ROOT/uninstall.sh" --purge --keep-source >/dev/null
test ! -e "$HOME_DIR/.local/bin/lay"
test ! -e "$HOME_DIR/.local/bin/lay-nanda-wave-train"
test ! -e "$HOME_DIR/.local/bin/lay-l1.1-restore"
test ! -e "$HOME_DIR/.local/bin/lay-l1.1-serve"
test ! -e "$HOME_DIR/.local/lib/lay"
test ! -e "$HOME_DIR/.config/systemd/user/lay-daemon.service"
test ! -e "$HOME_DIR/.config/lay"
test ! -e "$HOME_DIR/.local/share/lay"
test ! -e "$SYSTEM_ROOT/usr/share/ibus/component/lay-ime.xml"
test ! -e "$SYSTEM_ROOT/etc/udev/rules.d/99-lay-uinput.rules"

echo "== compact wave memory contract =="
test ! -e "$ROOT/src/nanda_wave/context_wave.rs"
search_quiet '"raw_word_table": false' \
    "$ROOT/data/lexicon/l2_lexical_phase_v2.manifest.json"
search_quiet 'include_english' \
    "$ROOT/data/lexicon/l2_lexical_phase_v2.manifest.json"

echo "public issue regressions: PASS"
