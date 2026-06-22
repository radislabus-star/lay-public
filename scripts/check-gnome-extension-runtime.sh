#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UUID="lay@radislabus-star.github.io"
SRC="$ROOT/extension/$UUID"
DST="$HOME/.local/share/gnome-shell/extensions/$UUID"
CACHE="$HOME/.cache/lay"
FIX=0
RELOAD=0

for arg in "$@"; do
  case "$arg" in
    --fix) FIX=1 ;;
    --reload) RELOAD=1 ;;
    *)
      echo "usage: $0 [--fix] [--reload]" >&2
      exit 2
      ;;
  esac
done

source_version() {
  python3 - "$SRC/metadata.json" <<'PY'
import json, sys
print(json.load(open(sys.argv[1], encoding="utf-8")).get("version-name", ""))
PY
}

loaded_version() {
  gdbus call --session \
    --dest org.gnome.Shell \
    --object-path /io/github/radislabus_star/LayDaemon \
    --method io.github.radislabus_star.LayDaemon.Version 2>/dev/null \
    | sed -n "s/.*'\([^']*\)'.*/\1/p" || true
}

sync_files() {
  mkdir -p "$DST" "$CACHE"
  cp -f "$SRC/metadata.json" "$DST/metadata.json"
  for js in "$SRC"/*.js; do
    name="$(basename "$js")"
    cp -f "$js" "$DST/$name"
    if [[ "$name" != "extension.js" && "$name" != "lay-impl.js" ]]; then
      cp -f "$js" "$CACHE/$name" 2>/dev/null || true
    fi
  done
}

reload_extension() {
  if gnome-extensions help reload >/dev/null 2>&1; then
    gnome-extensions reload "$UUID"
  else
    gnome-extensions disable "$UUID" 2>/dev/null || true
    sleep 1
    gnome-extensions enable "$UUID" 2>/dev/null || true
  fi
}

if [[ "$FIX" == "1" ]]; then
  sync_files
fi

missing=0
if [[ ! -d "$DST" ]]; then
  echo "runtime extension missing: $DST" >&2
  missing=1
fi

drift=0
if [[ "$missing" == "0" ]]; then
  for file in "$SRC"/metadata.json "$SRC"/*.js; do
    name="$(basename "$file")"
    if ! cmp -s "$file" "$DST/$name"; then
      echo "runtime drift: $name" >&2
      drift=1
    fi
  done
fi

if [[ "$RELOAD" == "1" ]]; then
  reload_extension
fi

src_ver="$(source_version)"
loaded_ver="$(loaded_version)"
if [[ -n "$loaded_ver" && "$loaded_ver" != "$src_ver" ]]; then
  echo "loaded extension version drift: loaded=$loaded_ver source=$src_ver" >&2
  drift=1
fi

if [[ "$missing" == "1" || "$drift" == "1" ]]; then
  echo "GNOME extension runtime: DRIFT"
  exit 1
fi

echo "GNOME extension runtime: OK version=$src_ver"
