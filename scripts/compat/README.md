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

The adapter also preserves Firefox's real IBus capability mask and adds private
bit `1 << 30`, `LAY_EXACT_SURROUNDING_REFRESH`. That bit declares only that the
adapter forces a post-release and post-Reset surrounding-text refresh. It does
not select whole-word preedit, grant edit authority or alter the client's
advertised preedit, focus or surrounding-text bits. Lay still requires an
unselected boundary-exact snapshot, live owner and complete observed suffix
before preparing a Space correction. The interposer first resolves the real
capability setter with
`RTLD_NEXT`. Firefox loads `libibus-1.0.so.5` into a local dynamic-loader scope,
so the adapter falls back to `RTLD_NOLOAD` on that already-loaded library. It
exits before input starts if neither route resolves the real setter.

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

## Native Firefox on KDE

For an already installed native Firefox on KDE, use the separate
`lay-firefox-kde-native /absolute/path/to/firefox [arguments...]` launcher.
It loads the same unchanged adapter from
`~/.local/lib/lay/compat/liblay-firefox-ibus-reset-notify.so`, preserves inherited
preloads, selects GTK IBus only for Firefox, and removes synchronous IBus mode.
It also sets the GTK process flag `GNOME_ACCESSIBILITY=1` for this Firefox only.
This exposes actual character bounds to the read-only KDE hint renderer; it
does not select the GNOME desktop, change global accessibility settings or
change the Firefox profile. KDE owned preedit hints must use those bounds:
Firefox's raw IBus cursor rectangle can describe the start of the composition.
It requires KDE and refuses a missing browser or library. The existing Snap
launcher and GNOME entries remain unchanged. A KDE desktop entry may use this
command with `OnlyShowIn=KDE`; it must not change the default browser or MIME
associations. Restart the browser normally and verify its actual library mapping
before accepting a manual result; compilation alone is not client acceptance.

## KDE IBus candidate-panel focus

On the measured KDE Wayland/direct GTK IBus route, IBus can map its candidate
popup as a normal window and receive keyboard focus during a layout handoff.
`data/kde/ibus-candidate-focus.kwinrules` uses KWin's existing acceptfocus rule
for the exact normal-window class and caption `ibus-ui-gtk3`. The candidate
remains visible; application/owner guards and IME delivery remain unchanged.

The KDE installer calls `lay-kde-panel-focus.py`, which uses the installed
KConfig tools on a temporary copy of `kwinrulesrc`, preserves existing named
and legacy numbered rules, and adds only its own registration. It replaces
the configuration only if the original has not changed. Removal unregisters
the Lay rule and removes its unchanged keys; user edits and unrelated keys stay.
KWin reconfiguration needs no IBus or application restart.
KWin 5 can rewrite the registered group to a number on startup. Reinstallation
and removal track that group by the rule's exact description and popup match;
they also remove unchanged keys from the original orphaned group on uninstall.
The regression fixture is a configuration saved by the actual private KWin run.

Native private KWin5.24.7 with KWaylandServer5.24.6 acceptance:
Firefox/Chrome fields7,8,10, all24 original
roundtrip, short, burst5 and space_burst5 cases PASS. Seven mapped panels all
have wantsInput=false, with no panel activation. This does not certify Plasma6
or every IBus dialog. See the owning architecture document and the
`2026-10-19-kde-ibus-panel-focus.json` decision for exact bytes and receipts.

## External layout publication experiment

`kwin-input-method-layout-modifiers.patch` publishes pending modifiers through
the existing InputMethod keyboard grab after a valid external layout switch.
Its companion test patch covers both group directions with and without a queued
old-group echo. Run the four data rows in an isolated test process: previous
modifier activity in the full suite can hide the original missing publication.

The remote KWin5.24.7 controlled protocol comparison is RED4/GREEN4; the full
KeyboardLayout suite passes. Two upstream InputMethod failures are retained,
including an intermittent client-count failure. This is a private diagnostic
experiment, not an installed or release-qualified dependency. See
`2026-10-19-kde-external-layout-modifier-publication.json` and the owning document
for the exact source, binary, failed full-suite receipts and native qualification.

## IBus Reset surrounding delivery cache

`ibus-reset-surrounding-cache.patch` applies after
`ibus-fresh-focus-surrounding.patch` to the measured private IBus1.5.34-rc2
source. Reset retires the recipient Lay snapshot, but the proxy's equal-tuple
deduplication previously retained its delivery cache. The patch clears only
that proxy's cached `IBusText` before forwarding the existing Reset. It sends
no cached text; the next actual client callback still passes through ordinary
Unicode/cursor/selection handling and Lay admission/verifier checks.

The existing extracted-source test has twelve cases, including identical
actual text after Reset, normal deduplication after delivery, repeated Reset,
Unicode selection and another proxy's independent cache. The separate real
GObject lifetime control confirms that Reset releases its owned text.
Baseline is RED9/3; candidate is GREEN12/0 plus lifetime PASS. Only the private
`ibus-daemon` changes. The original native112 pass improves89/19/4 to90/18/4;
Qt RichTextEdit focus-return recovers. Matching browser320 improves291/29/0
to292/28/0; only Chrome field2 moved-boundary recovers. The same complete
dependency-bundle432 comparison improves380/48/4 to382/46/4, with no other
status changes. Remaining failures and private KWin qualification gaps
prevent acceptance; GNOME is NOT_TESTED. This is not an installed or
release-qualified dependency. See
`2026-10-19-ibus-reset-surrounding-delivery-cache.json` and the owning input
stability document for exact bytes, receipts and scope.
