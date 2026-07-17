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
  restart lay-daemon/IBus and verify versions.

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

ensure_ibus_daemon() {
  if busctl --user list 2>/dev/null | grep -q 'org.freedesktop.IBus'; then
    return 0
  fi
  ibus-daemon -drx >/dev/null 2>&1 &
  sleep 1
}

current_gnome_layout() {
  timeout 2s gdbus call \
    --session \
    --dest org.gnome.Shell \
    --object-path /io/github/radislabus_star/LayDaemon \
    --method io.github.radislabus_star.LayDaemon.CurrentLayout 2>/dev/null \
    | sed -n "s/^('\\(.*\\)',)$/\\1/p"
}

select_lay_input_source() {
  local layout="${1:?layout required}"
  local sources index attempt current
  if timeout 2s gdbus call \
    --session \
    --dest org.gnome.Shell \
    --object-path /io/github/radislabus_star/LayDaemon \
    --method io.github.radislabus_star.LayDaemon.ActivateLayout \
    "$layout" 2>/dev/null \
    | grep -q '(true,)'; then
    for attempt in 1 2 3 4 5; do
      current="$(current_gnome_layout || true)"
      if [[ "$current" == "$layout" ]]; then
        return 0
      fi
      sleep 0.15
    done
  fi

  sources="$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null || true)"
  index="$(
    python3 - "$sources" "$layout" <<'PY' 2>/dev/null || true
import ast
import sys

try:
    sources = ast.literal_eval(sys.argv[1])
except Exception:
    raise SystemExit(0)

for idx, source in enumerate(sources):
    if tuple(source) == ("ibus", sys.argv[2]):
        print(idx)
        break
PY
  )"
  if [[ -n "$index" ]]; then
    gsettings set org.gnome.desktop.input-sources current "$index" || true
    return 0
  fi
  return 1
}

preserved_layout="$(current_gnome_layout || true)"
case "$preserved_layout" in
  lay-ime-us|lay-ime-ru) ;;
  *)
    preserved_layout="$(timeout 1s ibus engine 2>/dev/null || true)"
    case "$preserved_layout" in
      lay-ime-us|lay-ime-ru) ;;
      *) preserved_layout=lay-ime-ru ;;
    esac
    ;;
esac

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
  systemctl --user restart lay-daemon.service
  pkill -x lay-ibus-engine 2>/dev/null || true
  ibus restart
  ensure_ibus_daemon
  select_lay_input_source "$preserved_layout"
  sleep 1
  ibus engine "$preserved_layout" || {
    ensure_ibus_daemon
    select_lay_input_source "$preserved_layout"
    sleep 1
    ibus engine "$preserved_layout"
  }
  select_lay_input_source "$preserved_layout"
  sleep 0.2
  if [[ "$(current_gnome_layout || true)" != "$preserved_layout" \
      || "$(timeout 1s ibus engine 2>/dev/null || true)" != "$preserved_layout" ]]; then
    echo "layout restore failed: expected=$preserved_layout" >&2
    exit 1
  fi
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
