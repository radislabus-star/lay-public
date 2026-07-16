#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_DIR="${LAY_RELEASE_SOURCE_DIR:-$ROOT/target/release}"
INSTALL_DIR="${LAY_INSTALL_LIBEXEC_DIR:-$HOME/.local/lib/lay/bin}"
LINK_DIR="${LAY_INSTALL_BIN_DIR:-$HOME/.local/bin}"

binaries=(
  lay
  lay-daemon
  lay-nanda-wave-eval
  lay-test-input
  lay-ngram-corpus
  lay-ibus-engine
  lay-memory-report
)

mkdir -p "$INSTALL_DIR" "$LINK_DIR"

for binary in "${binaries[@]}"; do
  source="$SOURCE_DIR/$binary"
  destination="$INSTALL_DIR/$binary"
  temporary="$INSTALL_DIR/.${binary}.tmp.$$"
  if [[ ! -x "$source" ]]; then
    echo "release binary missing: $source" >&2
    exit 1
  fi
  install -m 0755 "$source" "$temporary"
  mv -f "$temporary" "$destination"
  ln -sfn "$destination" "$LINK_DIR/$binary"
done

printf 'Installed %s release binaries in %s\n' "${#binaries[@]}" "$INSTALL_DIR"
