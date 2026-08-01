#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=l2-package-contract.sh
source "$ROOT/scripts/l2-package-contract.sh"

PACKAGE_NAME="${LAY_L2_PACKAGE_NAME:-$LAY_CANONICAL_L2_PACKAGE_NAME}"
EXPECTED_BYTES="${LAY_L2_PACKAGE_BYTES:-$LAY_CANONICAL_L2_PACKAGE_BYTES}"
EXPECTED_SHA256="${LAY_L2_PACKAGE_SHA256:-$LAY_CANONICAL_L2_PACKAGE_SHA256}"
PACKAGE_URL="${LAY_L2_PACKAGE_URL:-$LAY_CANONICAL_L2_PACKAGE_URL}"
PACKAGE_SOURCE="${LAY_L2_PACKAGE_SOURCE:-$ROOT/data/l2/$PACKAGE_NAME}"
PACKAGE_DIR="${LAY_L2_MODEL_DIR:-$HOME/.local/share/lay/nanda_wave/l2}"
CACHE_DIR="${LAY_L2_PACKAGE_CACHE_DIR:-$HOME/.cache/lay/models}"
CACHE_PATH="$CACHE_DIR/$PACKAGE_NAME"
INSTALLED_PATH="$PACKAGE_DIR/$PACKAGE_NAME"

package_matches_contract() {
    local path="$1"
    local actual_bytes actual_sha256
    [[ -f "$path" ]] || return 1
    actual_bytes="$(stat -c %s -- "$path")"
    [[ "$actual_bytes" == "$EXPECTED_BYTES" ]] || return 1
    actual_sha256="$(sha256sum -- "$path" | awk '{print $1}')"
    [[ "$actual_sha256" == "$EXPECTED_SHA256" ]]
}

describe_mismatch() {
    local path="$1"
    local actual_bytes actual_sha256
    actual_bytes="$(stat -c %s -- "$path" 2>/dev/null || printf 'unreadable')"
    actual_sha256="$(sha256sum -- "$path" 2>/dev/null | awk '{print $1}')"
    printf 'canonical L2 package failed verification: %s\n' "$path" >&2
    printf 'expected bytes=%s sha256=%s\n' "$EXPECTED_BYTES" "$EXPECTED_SHA256" >&2
    printf 'actual   bytes=%s sha256=%s\n' "$actual_bytes" "${actual_sha256:-unreadable}" >&2
}

for candidate in "$PACKAGE_SOURCE" "$INSTALLED_PATH" "$CACHE_PATH"; do
    if package_matches_contract "$candidate"; then
        printf 'canonical L2 package verified: %s\n' "$candidate" >&2
        printf '%s\n' "$candidate"
        exit 0
    fi
done

if [[ -n "${LAY_L2_PACKAGE_SOURCE:-}" && -f "$PACKAGE_SOURCE" ]]; then
    describe_mismatch "$PACKAGE_SOURCE"
    exit 1
fi

if [[ "${LAY_L2_OFFLINE:-0}" == "1" ]]; then
    printf 'canonical L2 package is not available offline: %s\n' "$PACKAGE_NAME" >&2
    printf 'expected a verified copy in %s, %s, or %s\n' \
        "$PACKAGE_SOURCE" "$INSTALLED_PATH" "$CACHE_PATH" >&2
    exit 1
fi

mkdir -p "$CACHE_DIR"
temporary="$CACHE_DIR/.${PACKAGE_NAME}.part.$$"
trap 'rm -f "$temporary"' EXIT

printf 'downloading canonical L2 package (%s bytes)...\n' "$EXPECTED_BYTES" >&2
case "$PACKAGE_URL" in
    file://*)
        cp -- "${PACKAGE_URL#file://}" "$temporary"
        ;;
    https://*)
        if command -v curl >/dev/null 2>&1; then
            curl --proto '=https' --tlsv1.2 --fail --location --show-error \
                --progress-bar --retry 5 --retry-delay 2 \
                "$PACKAGE_URL" --output "$temporary"
        elif command -v wget >/dev/null 2>&1; then
            wget --https-only --output-document="$temporary" "$PACKAGE_URL"
        else
            echo "cannot download canonical L2 package: install curl or wget" >&2
            exit 1
        fi
        ;;
    *)
        printf 'canonical L2 package URL must use HTTPS: %s\n' "$PACKAGE_URL" >&2
        exit 1
        ;;
esac

if ! package_matches_contract "$temporary"; then
    describe_mismatch "$temporary"
    exit 1
fi

mv -f "$temporary" "$CACHE_PATH"
trap - EXIT
printf 'canonical L2 package downloaded and verified: %s\n' "$CACHE_PATH" >&2
printf '%s\n' "$CACHE_PATH"
