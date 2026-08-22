#!/bin/sh
set -eu

: "${LAY_PROOF_IBUS_DAEMON:?LAY_PROOF_IBUS_DAEMON is required}"
: "${LAY_PROOF_IBUS_LIBDIR:?LAY_PROOF_IBUS_LIBDIR is required}"
: "${IBUS_ADDRESS:?IBUS_ADDRESS is required}"
: "${IBUS_COMPONENT_PATH:?IBUS_COMPONENT_PATH is required}"

case "$IBUS_ADDRESS" in
    unix:path=/*) ;;
    *)
        echo "IBUS_ADDRESS must be an absolute unix:path address" >&2
        exit 64
        ;;
esac

ibus_socket=${IBUS_ADDRESS#unix:path=}
rm -f -- "$ibus_socket"

lay_component_found=false
xkb_component_found=false
old_ifs=$IFS
IFS=:
for component_dir in $IBUS_COMPONENT_PATH; do
    if [ -f "$component_dir/lay.xml" ] &&
            grep -Fq '<name>lay-ime-ru</name>' "$component_dir/lay.xml"; then
        lay_component_found=true
    fi
    if [ -f "$component_dir/simple.xml" ] &&
            grep -Fq '<name>xkb:us::eng</name>' "$component_dir/simple.xml"; then
        xkb_component_found=true
    fi
done
IFS=$old_ifs

if [ "$lay_component_found" != true ] || [ "$xkb_component_found" != true ]; then
    printf 'LAY_PROOF_IBUS_REGISTRY_REFUSED lay=%s xkb=%s path=%s\n' \
        "$lay_component_found" "$xkb_component_found" "$IBUS_COMPONENT_PATH" >&2
    exit 66
fi

printf 'LAY_PROOF_IBUS_REGISTRY_READY lay=true xkb=true path=%s\n' \
    "$IBUS_COMPONENT_PATH" >&2

export LD_LIBRARY_PATH="$LAY_PROOF_IBUS_LIBDIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

exec "$LAY_PROOF_IBUS_DAEMON" \
    --address "$IBUS_ADDRESS" \
    --cache none \
    --panel disable \
    --config disable \
    --emoji-extension disable
