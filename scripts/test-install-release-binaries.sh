#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

SOURCE_DIR="$TMP/release"
mkdir -p "$SOURCE_DIR"
for binary in \
    lay lay-daemon lay-nanda-wave-eval lay-nanda-wave-train lay-test-input \
    lay-ngram-corpus lay-ibus-engine lay-memory-report lay-l11-restore lay-l11-serve; do
    printf '#!/usr/bin/env sh\nexit 0\n' >"$SOURCE_DIR/$binary"
    chmod +x "$SOURCE_DIR/$binary"
done
cat >"$SOURCE_DIR/lay-nanda-wave-train" <<'EOF'
#!/usr/bin/env sh
set -eu
if [ "${1:-}" = "--compile-v13-exact-sidecar" ]; then
    shift 2
    [ "${1:-}" = "--out" ]
    printf 'verified exact V13 sidecar fixture\n' >"$2"
fi
EOF
chmod +x "$SOURCE_DIR/lay-nanda-wave-train"

FIXTURE="$TMP/LAY-L2-TEST.bin"
printf 'verified canonical L2 fixture\n' >"$FIXTURE"
FIXTURE_BYTES="$(stat -c %s "$FIXTURE")"
FIXTURE_SHA256="$(sha256sum "$FIXTURE" | awk '{print $1}')"
PACKAGE_NAME="LAY-L2-TEST.bin"
SIDECAR_NAME="LAY-L2-TEST.dafsa"

run_install() {
    local home="$1"
    HOME="$home" \
    LAY_RELEASE_SOURCE_DIR="$SOURCE_DIR" \
    LAY_INSTALL_LIBEXEC_DIR="$home/libexec" \
    LAY_INSTALL_BIN_DIR="$home/bin" \
    LAY_L2_MODEL_DIR="$home/models" \
    LAY_L2_PACKAGE_CACHE_DIR="$home/cache" \
    LAY_L2_PACKAGE_NAME="$PACKAGE_NAME" \
    LAY_EXACT_V13_SIDECAR_NAME="$SIDECAR_NAME" \
    LAY_L2_PACKAGE_SOURCE="$home/missing/$PACKAGE_NAME" \
    LAY_L2_PACKAGE_URL="file://$FIXTURE" \
    LAY_L2_PACKAGE_BYTES="$FIXTURE_BYTES" \
    LAY_L2_PACKAGE_SHA256="$FIXTURE_SHA256" \
    bash "$ROOT/scripts/install-release-binaries.sh"
}

echo "== clean checkout downloads and verifies canonical L2 =="
HOME_OK="$TMP/home-ok"
run_install "$HOME_OK" >/dev/null
cmp -s "$FIXTURE" "$HOME_OK/models/$PACKAGE_NAME"
cmp -s "$FIXTURE" "$HOME_OK/cache/$PACKAGE_NAME"
grep -q 'verified exact V13 sidecar fixture' "$HOME_OK/models/$SIDECAR_NAME"
test -x "$HOME_OK/libexec/lay"
test -L "$HOME_OK/bin/lay"

echo "== offline update reuses the verified installed package =="
rm -rf "$HOME_OK/cache"
mv "$FIXTURE" "$TMP/fixture-away"
HOME="$HOME_OK" \
LAY_RELEASE_SOURCE_DIR="$SOURCE_DIR" \
LAY_INSTALL_LIBEXEC_DIR="$HOME_OK/libexec" \
LAY_INSTALL_BIN_DIR="$HOME_OK/bin" \
LAY_L2_MODEL_DIR="$HOME_OK/models" \
LAY_L2_PACKAGE_CACHE_DIR="$HOME_OK/cache" \
LAY_L2_PACKAGE_NAME="$PACKAGE_NAME" \
LAY_EXACT_V13_SIDECAR_NAME="$SIDECAR_NAME" \
LAY_L2_PACKAGE_SOURCE="$HOME_OK/missing/$PACKAGE_NAME" \
LAY_L2_PACKAGE_URL="file://$FIXTURE" \
LAY_L2_PACKAGE_BYTES="$FIXTURE_BYTES" \
LAY_L2_PACKAGE_SHA256="$FIXTURE_SHA256" \
LAY_L2_OFFLINE=1 \
bash "$ROOT/scripts/install-release-binaries.sh" >/dev/null
cmp -s "$TMP/fixture-away" "$HOME_OK/models/$PACKAGE_NAME"
grep -q 'verified exact V13 sidecar fixture' "$HOME_OK/models/$SIDECAR_NAME"
mv "$TMP/fixture-away" "$FIXTURE"

echo "== unavailable package fails before partial binary installation =="
HOME_FAIL="$TMP/home-fail"
if HOME="$HOME_FAIL" \
    LAY_RELEASE_SOURCE_DIR="$SOURCE_DIR" \
    LAY_INSTALL_LIBEXEC_DIR="$HOME_FAIL/libexec" \
    LAY_INSTALL_BIN_DIR="$HOME_FAIL/bin" \
    LAY_L2_MODEL_DIR="$HOME_FAIL/models" \
    LAY_L2_PACKAGE_CACHE_DIR="$HOME_FAIL/cache" \
    LAY_L2_PACKAGE_NAME="$PACKAGE_NAME" \
    LAY_EXACT_V13_SIDECAR_NAME="$SIDECAR_NAME" \
    LAY_L2_PACKAGE_SOURCE="$HOME_FAIL/missing/$PACKAGE_NAME" \
    LAY_L2_PACKAGE_BYTES="$FIXTURE_BYTES" \
    LAY_L2_PACKAGE_SHA256="$FIXTURE_SHA256" \
    LAY_L2_OFFLINE=1 \
    bash "$ROOT/scripts/install-release-binaries.sh" >"$TMP/fail.log" 2>&1; then
    echo "offline install unexpectedly succeeded without the model" >&2
    exit 1
fi
grep -q 'canonical L2 package is not available offline' "$TMP/fail.log"
test ! -e "$HOME_FAIL/libexec/lay"

echo "== corrupt download is rejected before binary installation =="
HOME_BAD="$TMP/home-bad"
printf 'corrupt\n' >"$TMP/corrupt.bin"
if HOME="$HOME_BAD" \
    LAY_RELEASE_SOURCE_DIR="$SOURCE_DIR" \
    LAY_INSTALL_LIBEXEC_DIR="$HOME_BAD/libexec" \
    LAY_INSTALL_BIN_DIR="$HOME_BAD/bin" \
    LAY_L2_MODEL_DIR="$HOME_BAD/models" \
    LAY_L2_PACKAGE_CACHE_DIR="$HOME_BAD/cache" \
    LAY_L2_PACKAGE_NAME="$PACKAGE_NAME" \
    LAY_EXACT_V13_SIDECAR_NAME="$SIDECAR_NAME" \
    LAY_L2_PACKAGE_SOURCE="$HOME_BAD/missing/$PACKAGE_NAME" \
    LAY_L2_PACKAGE_URL="file://$TMP/corrupt.bin" \
    LAY_L2_PACKAGE_BYTES="$FIXTURE_BYTES" \
    LAY_L2_PACKAGE_SHA256="$FIXTURE_SHA256" \
    bash "$ROOT/scripts/install-release-binaries.sh" >"$TMP/bad.log" 2>&1; then
    echo "corrupt canonical L2 download unexpectedly succeeded" >&2
    exit 1
fi
grep -q 'canonical L2 package failed verification' "$TMP/bad.log"
test ! -e "$HOME_BAD/libexec/lay"

echo "release installer canonical L2 regressions: PASS"
