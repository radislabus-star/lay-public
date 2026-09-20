# Firefox IBus surrounding-text notifications

Firefox on native Wayland can omit the final surrounding-text notification
after an asynchronous IBus commit. GTK IBus 1.5.29 can also send Reset without
retrieving the surrounding text afterwards because its initial
RequireSurroundingText callback disconnects itself. Either gap leaves Lay
without the exact client snapshot it requires. See the measured TD-121 evidence
in `tech_debt/121-preserve-word-across-ime-layout-handoff.md`.

`firefox-ibus-reset-notify.c` preserves the real GTK key filter and Reset. It
emits one `retrieve-surrounding` signal after an outer handled printable release
without command modifiers, and one after an outer Reset. Both require an IBus
GtkIMMulticontext. Firefox supplies actual text through its existing handler.
Thread-local depth guards prevent nested requests. The adapter does not inspect
text, synthesize keys or snapshots, poll, or grant edit access. Lay's current
owner, epoch, cursor, selection and exact-snapshot checks remain required.

Build only on the remote worker under `scripts/lay-resource-guard.sh`, with:

```sh
cc -std=c11 -Wall -Wextra -Werror -fPIC -shared \
  scripts/compat/firefox-ibus-reset-notify.c -ldl \
  -o /absolute/evidence/path/liblay-firefox-ibus-reset-notify.so
```

The library's installation path for Snap Firefox is
`~/snap/firefox/common/lay/compat/liblay-firefox-ibus-reset-notify.so`.
Install `lay-firefox` as an executable in `~/.local/bin/` and use it only in the
user's Firefox desktop entries. The launcher selects direct GTK IBus and puts
the Lay adapter before the preload already established inside Snap confinement.
It explicitly removes inherited
`IBUS_ENABLE_SYNC_MODE`; synchronous native Wayland processing can lose a
modifier transition. Never set `LD_PRELOAD`, `GTK_IM_MODULE`, or synchronous
IBus mode in the global session environment.

Before installation, verify the built library hash, mapping and absent
synchronous mode in an owned Firefox process, then run the fixed native text
and modifier controls. Existing Firefox processes need a normal browser restart
to load the adapter. Restore the prior desktop Exec lines and remove the
launcher/library to roll back. This adapter covers the measured Firefox
155.0.1/GTK 3 route; rerun the controls after a Firefox or GTK ABI change.
