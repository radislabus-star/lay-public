#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo() {
    "$ROOT/scripts/cargo-guard.sh" "$@"
}

current_version="$(perl -ne 'print "$1.$2.$3\n" if /^version = "([0-9]+)\.([0-9]+)\.([0-9]+)"/' Cargo.toml | head -n1)"
if [[ -z "${current_version}" ]]; then
    echo "Cannot read Cargo.toml package version" >&2
    exit 1
fi

IFS=. read -r major minor patch <<<"${current_version}"
next_patch="$((patch + 1))"
version="${major}.${minor}.${next_patch}"
release_date="$(date +%F)"

perl -0pi -e 's/^version = "[^"]+"/version = "'"${version}"'"/m' Cargo.toml
perl -0pi -e 's/"version": [0-9]+/"version": '"${next_patch}"'/' \
  extension/lay@radislabus-star.github.io/metadata.json
perl -0pi -e 's/"version-name": "[^"]+"/"version-name": "'"${version}"'"/' \
  extension/lay@radislabus-star.github.io/metadata.json
perl -0pi -e "s/const APP_VERSION = '[^']+';/const APP_VERSION = '${version}';/" \
  extension/lay@radislabus-star.github.io/lay-impl.js
perl -0pi -e "s/export const APP_VERSION = '[^']+';/export const APP_VERSION = '${version}';/" \
  extension/lay@radislabus-star.github.io/tray_support.js
perl -0pi -e "s/const APP_VERSION = '[^']+';/const APP_VERSION = '${version}';/" \
  extension/lay@radislabus-star.github.io/settings.js \
  extension/lay@radislabus-star.github.io/prefs.js
perl -0pi -e "s/((?:export\s+)?const APP_RELEASE_DATE = ')[^']+(')/\${1}${release_date}\${2}/" \
  extension/lay@radislabus-star.github.io/tray_support.js \
  extension/lay@radislabus-star.github.io/settings.js \
  extension/lay@radislabus-star.github.io/prefs.js
perl -0pi -e 's/Current publication branch version:\n\n- `[^`]+`/Current publication branch version:\n\n- `'"${version}"'`/' \
  VERSIONING.md

cargo check --quiet

printf 'lay version: %s -> %s\n' "$current_version" "$version"
