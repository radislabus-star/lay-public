#!/usr/bin/env bash
set -euo pipefail

ibus_pids_before="$(pgrep -x lay-ibus-engine 2>/dev/null || true)"

systemctl --user restart lay-daemon.service
if systemctl --user is-active --quiet lay-l3-online.service; then
  systemctl --user restart lay-l3-online.service
fi

ibus_pids_after="$(pgrep -x lay-ibus-engine 2>/dev/null || true)"
if [[ "$ibus_pids_after" != "$ibus_pids_before" ]]; then
  echo "IBus engine changed during model-service reload" >&2
  echo "before=${ibus_pids_before:-none} after=${ibus_pids_after:-none}" >&2
  exit 1
fi

systemctl --user is-active --quiet lay-daemon.service
printf 'Lay model services reloaded; IBus engine unchanged: %s\n' \
  "${ibus_pids_after:-not-running}"
