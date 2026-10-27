#!/usr/bin/env bash
# Install the existing session-owned services, preserving disabled upgrade state.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode="${1:-}"
[[ $# == 1 && ( "$mode" == --fresh || "$mode" == --update ) ]] || {
  echo 'usage: install-session-services.sh --fresh|--update' >&2
  exit 2
}
# Use the same manager-owned location as the installer's other user units.
# A private desktop can override XDG_CONFIG_HOME for its application data.
unit_dir="$HOME/.config/systemd/user"
mkdir -p "$unit_dir"
enabled=()
for unit in lay-daemon.service lay-l3-online.service; do
  if [[ "$mode" == --fresh ]] || systemctl --user is-enabled --quiet "$unit"; then
    enabled+=("$unit")
  fi
  install -m 644 "$ROOT/systemd/$unit" "$unit_dir/$unit"
done
systemctl --user daemon-reload
for unit in "${enabled[@]}"; do
  # enable alone leaves legacy default.target.wants links behind. Reenable
  # rebuilds links using the new WantedBy without starting or stopping input.
  systemctl --user reenable "$unit"
done
