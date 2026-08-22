#!/bin/sh
set -eu

: "${LAY_PROOF_IBUS_DAEMON:?LAY_PROOF_IBUS_DAEMON is required}"
: "${LAY_PROOF_IBUS_LIBDIR:?LAY_PROOF_IBUS_LIBDIR is required}"
: "${IBUS_ADDRESS:?IBUS_ADDRESS is required}"

case "$IBUS_ADDRESS" in
    unix:path=/*) ;;
    *)
        echo "IBUS_ADDRESS must be an absolute unix:path address" >&2
        exit 64
        ;;
esac

export LD_LIBRARY_PATH="$LAY_PROOF_IBUS_LIBDIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

exec "$LAY_PROOF_IBUS_DAEMON" \
    --address "$IBUS_ADDRESS" \
    "$@" \
    --config disable \
    --emoji-extension disable
