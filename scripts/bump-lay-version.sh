#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo() {
  "$ROOT/scripts/cargo-guard.sh" "$@"
}

SYNC_ONLY=0
NO_BUILD=0
NO_RELOAD=0

usage() {
  cat >&2 <<'EOF'
usage: scripts/bump-lay-version.sh [--sync-only] [--no-build] [--no-reload]

Default:
  increment patch version, sync GNOME extension runtime, build release binaries,
  restart Lay model services without restarting IBus, and verify versions.

Options:
  --sync-only  do not increment version; only sync installed runtime and reload
  --no-build   skip cargo build --release --bins
  --no-reload  sync files but do not reload GNOME extension / restart daemons
EOF
}

for arg in "$@"; do
  case "$arg" in
    --sync-only) SYNC_ONLY=1 ;;
    --no-build) NO_BUILD=1 ;;
    --no-reload) NO_RELOAD=1 ;;
    -h|--help) usage; exit 0 ;;
    *) usage; exit 2 ;;
  esac
done

version_from_cargo() {
  perl -ne 'print "$1\n" if /^version = "([^"]+)"/' Cargo.toml | head -n1
}

if [[ "$SYNC_ONLY" == "0" ]]; then
  scripts/bump-version-from-git.sh
else
  cargo check --quiet
fi

version="$(version_from_cargo)"
if [[ -z "$version" ]]; then
  echo "Cannot read Cargo.toml package version" >&2
  exit 1
fi

if [[ "$NO_RELOAD" == "1" ]]; then
  scripts/check-gnome-extension-runtime.sh --fix
else
  scripts/check-gnome-extension-runtime.sh --fix --reload
fi

if [[ "$NO_BUILD" == "0" ]]; then
  cargo build --release --bins
  scripts/install-l2-transition-phase.sh
  scripts/install-l2-lexical-phase.sh
  scripts/install-l3-context-phase.sh
  target/release/lay-nanda-wave-eval --llmwave-pack-live
  scripts/rebuild-l4-feedback-memory.sh
  scripts/install-release-binaries.sh
fi

if [[ "$NO_RELOAD" == "0" ]]; then
  scripts/reload-lay-model-services.sh
fi

scripts/check-gnome-extension-runtime.sh

installed_lay="${LAY_INSTALL_LIBEXEC_DIR:-$HOME/.local/lib/lay/bin}/lay"
if [[ -x "$installed_lay" ]]; then
  bin_version="$($installed_lay --version | awk '{print $2}')"
else
  bin_version=""
fi
if [[ "$NO_BUILD" == "0" && "$bin_version" != "$version" ]]; then
  echo "release binary version drift: binary=$bin_version source=$version" >&2
  exit 1
fi

if command -v lay >/dev/null 2>&1; then
  installed_version="$(lay --version | awk '{print $2}')"
  if [[ "$installed_version" != "$version" ]]; then
    echo "installed lay version drift: installed=$installed_version source=$version" >&2
    echo "hint: run scripts/install-release-binaries.sh before publishing" >&2
    exit 1
  fi
fi

printf 'lay version OK: %s\n' "$version"
if [[ "$NO_RELOAD" == "0" ]]; then
  printf 'ibus engine: %s\n' "$(ibus engine)"
  pgrep -af 'lay-daemon|lay-ibus-engine' || true
fi
