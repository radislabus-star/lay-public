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
  lay-nanda-wave-train
  lay-test-input
  lay-ngram-corpus
  lay-ibus-engine
  lay-memory-report
)

mkdir -p "$INSTALL_DIR" "$LINK_DIR"

install_binary() {
  local source_name="$1"
  local installed_name="$2"
  local source="$SOURCE_DIR/$source_name"
  local destination="$INSTALL_DIR/$installed_name"
  local temporary="$INSTALL_DIR/.${installed_name}.tmp.$$"
  if [[ ! -x "$source" ]]; then
    echo "release binary missing: $source" >&2
    exit 1
  fi
  install -m 0755 "$source" "$temporary"
  mv -f "$temporary" "$destination"
  ln -sfn "$destination" "$LINK_DIR/$installed_name"
}

for binary in "${binaries[@]}"; do
  install_binary "$binary" "$binary"
done

install_binary lay-l11-restore lay-l1.1-restore

printf 'Installed %s release binaries in %s\n' "$(( ${#binaries[@]} + 1 ))" "$INSTALL_DIR"
