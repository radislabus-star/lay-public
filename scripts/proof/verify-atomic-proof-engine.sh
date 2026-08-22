#!/bin/sh
set -eu

engine_dir=${1:?engine directory is required}
engine_id=${2:?engine SHA-256 identity is required}

case "$engine_id" in
    *[!0-9a-f]*|'')
        echo "invalid atomic proof engine identity" >&2
        exit 64
        ;;
esac
[ "${#engine_id}" -eq 64 ] || {
    echo "atomic proof engine identity must be 64 lowercase hex characters" >&2
    exit 64
}

engine=$engine_dir/lay-ibus-engine
binary_manifest=$engine_dir/lay-ibus-engine.sha256
source_manifest=$engine_dir/source-files.sha256
provenance=$engine_dir/provenance.env

for required in "$engine" "$binary_manifest" "$source_manifest" "$provenance"; do
    [ -f "$required" ] || {
        printf 'atomic proof engine artifact is incomplete: %s\n' "$required" >&2
        exit 66
    }
done
[ -x "$engine" ] || {
    echo "atomic proof engine is not executable" >&2
    exit 66
}
[ -s "$source_manifest" ] || {
    echo "atomic proof source manifest is empty" >&2
    exit 66
}

(cd "$engine_dir" && sha256sum -c --status lay-ibus-engine.sha256) || {
    echo "atomic proof engine hash mismatch" >&2
    exit 65
}
actual_sha=$(sha256sum "$engine" | awk '{print $1}')
[ "$actual_sha" = "$engine_id" ] || {
    echo "atomic proof engine directory identity mismatch" >&2
    exit 65
}
strings "$engine" | grep -F 'ProcessKeyEventAtomicV1' >/dev/null || {
    echo "atomic proof engine does not expose ProcessKeyEventAtomicV1" >&2
    exit 65
}
grep -Fxq 'schema=lay.atomic-proof-engine.v1' "$provenance"
grep -Fxq "binary_sha256=$engine_id" "$provenance"
grep -Fxq 'required_method=ProcessKeyEventAtomicV1' "$provenance"

printf '%s\n' "$actual_sha"
