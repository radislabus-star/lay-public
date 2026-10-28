#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ "${LAY_RESOURCE_GUARD_ACTIVE:-0}" != "1" && " $* " != *" --list "* ]]; then
    exec "$ROOT/scripts/lay-resource-guard.sh" -- "$ROOT/scripts/check-lay-component.sh" "$@"
fi
cd "$ROOT"
exec python3 -u scripts/check-lay-component.py "$@"
