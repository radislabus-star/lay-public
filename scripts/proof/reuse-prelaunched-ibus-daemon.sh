#!/bin/sh
set -eu

: "${IBUS_ADDRESS:?IBUS_ADDRESS is required}"

case "$IBUS_ADDRESS" in
    unix:path=/*) ;;
    *)
        echo "IBUS_ADDRESS must be an absolute unix:path address" >&2
        exit 64
        ;;
esac

ibus_socket=${IBUS_ADDRESS#unix:path=}
if [ ! -S "$ibus_socket" ]; then
    printf 'LAY_PROOF_IBUS_REUSE_REFUSED socket=%s\n' "$ibus_socket" >&2
    exit 70
fi

printf 'LAY_PROOF_IBUS_REUSED socket=%s\n' "$ibus_socket" >&2
