#!/usr/bin/env bash
# Provision logind-managed access; never change user groups or device modes.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Filesystem override for isolated installer tests, like uninstall.sh's root.
SYSTEM_ROOT="${LAY_INPUT_ACCESS_SYSTEM_ROOT:-}"
if [ -n "$SYSTEM_ROOT" ]; then
    if [[ "$SYSTEM_ROOT" != /* || "$SYSTEM_ROOT" == / || ! -d "$SYSTEM_ROOT" ]]; then
        echo "Некорректный LAY_INPUT_ACCESS_SYSTEM_ROOT" >&2
        exit 2
    fi
    SYSTEM_ROOT="${SYSTEM_ROOT%/}"
fi

check_owned_file() {
    if { [ -e "$2" ] || [ -L "$2" ]; } && ! cmp -s "$1" "$2"; then
        echo "⚠ сохраняю изменённый файл $2; установка прав остановлена" >&2
        exit 1
    fi
}
# Check both before changing either, including repeated installations.
check_owned_file "$ROOT/udev/70-lay-input.rules" \
    "$SYSTEM_ROOT/etc/udev/rules.d/70-lay-input.rules"
check_owned_file "$ROOT/systemd/lay-uinput.conf" \
    "$SYSTEM_ROOT/etc/modules-load.d/lay-uinput.conf"

sudo install -D -m 0644 "$ROOT/udev/70-lay-input.rules" \
    "$SYSTEM_ROOT/etc/udev/rules.d/70-lay-input.rules"
sudo install -D -m 0644 "$ROOT/systemd/lay-uinput.conf" \
    "$SYSTEM_ROOT/etc/modules-load.d/lay-uinput.conf"

legacy="$SYSTEM_ROOT/etc/udev/rules.d/99-lay-uinput.rules"
if [ -e "$legacy" ]; then
    if cmp -s "$ROOT/udev/99-lay-uinput.legacy.rules" "$legacy"; then
        sudo rm -- "$legacy"
    else
        echo "⚠ сохраняю изменённое правило $legacy" >&2
    fi
fi

echo "✓ правила uaccess установлены; состав групп пользователя не изменён"
# Apply both rules even if this kernel cannot load uinput; retain a nonzero
# activation result rather than aborting before evdev provisioning.
module_status=0
sudo modprobe uinput || module_status=$?
sudo udevadm control --reload-rules
sudo udevadm trigger --action=change --subsystem-match=input '--sysname-match=event*'
sudo udevadm trigger --action=change --subsystem-match=misc --sysname-match=uinput
sudo udevadm settle --timeout=10
if [ "$module_status" -ne 0 ]; then
    echo "⚠ правила сохранены, но загрузка uinput не удалась; активация не подтверждена" >&2
    exit "$module_status"
fi

readable_input=0
for node in "$SYSTEM_ROOT"/dev/input/event*; do
    if [ -r "$node" ]; then readable_input=1; break; fi
done
if [ -w "$SYSTEM_ROOT/dev/uinput" ] && [ "$readable_input" = 1 ]; then
    echo "✓ устройства ввода доступны текущему процессу"
else
    echo "⚠ правила применены; доступ зависит от активной локальной logind-сессии" >&2
fi
