# KDE Firefox after autocorrection — comparison with GNOME

Status: read-only comparison; no implementation, installation, browser restart
or automatic native input. User manual mode remains active. Observation recorded
2026-10-28T16:54:35Z. One current task: first ordinary input on the standard
Firefox page, Double Shift fails after Lay autocorrection. Exact original and
corrected word and trailing-space detail requested asynchronously, not supplied.
This is a user-reported FAIL; no complete matrix recount or new acceptance.

## Verified baseline and limits

Canonical checkout /home/ubu/projects/lay-dependency-updates-20261027,
HEAD b835ca478198d6cb765a2246329d5646966c118c. Both earlier daemon experiments
were rejected and restored; source1301files/SHA
2ce6b24786bab558599a46345c9ec3583f419847420a1f37ebf5a08e93933c5e.
No functional Rust diff. Preserve all rejected receipts and do not reuse them
as accepted fixes. Last full KDE matrix:Firefox127P55F38B/220,
Chrome179P0F41B/220;306P55F79B/440.

Local installed updater source is1.0.111/22e9fead9744918b99c175454362d9e705caf608.
Its source differs from current HEAD only in the three Chrome observation files
within daemon/IME Rust paths; shared manual-toggle and GNOME source are equal.
This does not identify the exact source of the loaded local GNOME IME:
PID3037150/SHA1095c2648bb59e2f4363a3f58394013e6b6598eb9d8678f5b0fb98a01cd90ebc
is still an older deleted executable, also documented in ime-impact-map-2026-10-27.md.
Do not restart or replace it to resolve that uncertainty.

## First concrete difference: Firefox client startup

GNOME installation, verified from actual files:
- /home/ubu/.local/share/applications/firefox_firefox.desktop:6 invokes
  /home/ubu/.local/bin/lay-firefox; normal/new/private/profile entries use it.
- /home/ubu/.local/bin/firefox also delegates to lay-firefox.
- lay-firefox:6 starts inside Snap, checks the Firefox-only adapter library,
  removes IBUS_ENABLE_SYNC_MODE, selects GTK IBus for this process and prepends
  the adapter to the existing confined LD_PRELOAD. No global session change.
- Local library /home/ubu/snap/firefox/common/lay/compat/liblay-firefox-ibus-reset-notify.so
  exists, SHA924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22.
  This verifies installed launcher wiring, not mappings in a running GNOME Firefox.

Opened manual KDE Firefox, verified from the running process:
PID520337/commfirefox-bin, executable
/home/e/.local/lib/lay-test/firefox/firefox-bin, Firefox158.0.1, DISPLAY:71,
GTK_IM_MODULE=ibus, synchronous mode absent. No adapter selected in LD_PRELOAD
and no firefox-ibus-reset-notify mapping in /proc/520337/maps.
The three standard remote launcher/library locations are absent:
~/.local/bin/lay-firefox, ~/snap/firefox/common/lay/compat/library,
~/.local/lib/lay/compat/library. Other evidence-only build locations not searched.
The browser was opened directly to avoid the standard Snap startup failure.
That launch did not include the existing GNOME Firefox client adapter.

## Existing implementation to reuse, not rewrite

scripts/compat/firefox-ibus-reset-notify.c:35 preserves actual IBus capabilities
and advertises LAY_EXACT_SURROUNDING_REFRESH (bit1<<30).
:65 emits GTK retrieve-surrounding; Firefox's own handler supplies actual text.
:97 preserves the original key filter, requesting one refresh only after an
outer handled printable release without command modifiers (:124).
:131 preserves the original Reset, requesting one refresh after it (:149).
Thread-local depth guards bound nested calls. No synthetic text/keys, polling,
projected snapshot or mutation authority. Existing C03/C05 checks still apply.

Shared window_interaction/observation.rs:5075 reads the capability at:5086.
Common committed_tail.rs:367 performs Space autocorrection. Common
window_interaction/authority.rs:929/975 classifies and admits the exact manual
target. KDE and GNOME layout handoff diverge later in daemon
layout_controller.rs:112/124 (KDE kde_exact.rs vs GNOME gnome_dbus.rs).
No evidence yet that this later divergence causes this manual refusal.

See scripts/compat/README.md:3/10/18/38 for the measured missing asynchronous
commit/reset snapshots, adapter contract and per-Firefox installation.
Existing measured ABI scope is Firefox155.0.1/GTK3; the opened158.0.1 must be
checked rather than assumed equivalent.

## Verdict and next bounded check

FACT: working GNOME launcher integration includes this client adapter; the
opened KDE browser lacks it. This is the first proved startup-contract difference.
HYPOTHESIS: missing exact post-Reset/client receipt explains refusal of Double
Shift after autocorrection. It is not yet a proven cause or a closed bug.

Next minimum, only after user command: reuse this exact C source for a KDE
Firefox-only build/launcher; no daemon/IME/Chrome/GNOME/runtime-route rewrite.
Declare that startup integration scope before edits. A normal restart of the
manual Firefox is required to load the library; preserve user text/profile and
do not close the current window without authorization. Verify the mapped library,
GTK IBus and absent sync mode, then let user repeat the same word/autocorrection/
Double Shift in field1, reverse toggle and ordinary held Shift. Keep the original
without-adapter FAIL. No full matrix until a short manual proof.

## Authorized implementation preflight — 2026-10-28

User explicitly asked to fix and then continue after interruption. Authorized
scope is the existing adapter plus KDE-only native Firefox startup and one
normal restart of the owned manual browser. Current remote identities refreshed:
daemon374525/eb6098..., engine3556137/8b84bf..., IBus3168326/cb8ccaff...,
tray3228081/a2f33a6e...; manualFirefox520337 remains open without the adapter.
All three new installation targets are absent; no replacement of user files.

Repository change: new scripts/compat/lay-firefox-kde-native with explicit
native binary argument, KDE desktop guard, same process-only preload ordering,
asynchronous GTK IBus and missing-library refusal as the existing Snap launcher.
Existing scripts/compat/lay-firefox and firefox-ibus-reset-notify.c remain
byte-identical. README documents the native entrypoint. Scope guard/ADR declare
these exact two files; no new owner/queue/cache or weakened test.

Remote-only artifacts: ~/.local/lib/lay/compat/liblay-firefox-ibus-reset-notify.so,
~/.local/bin/lay-firefox-kde-native, ~/.local/share/applications/lay-firefox-kde-native.desktop.
The desktop entry is named Firefox (Lay), OnlyShowIn=KDE; no MIME/default-browser
changes or overwrite of existing Snap/GNOME entries. Use the existing native
Firefox binary and normal user profile; no WebDriver or new profile. Per-process
preload is not a session-wide environment change. Protect all Lay services,
binaries, config, GNOME launchers and other windows. Close only the identified
manual Firefox window through its normal window-manager close request. Do not
force-kill or delete profile locks. If it cannot close normally, stop the restart.

Rollback removes only our verified newly installed files; previous Firefox
launch remains available and Lay is untouched. Existing adapter tests and
zero-warning C build are setup evidence, not proof of the user's correction/
Double Shift scenario. Leave native page ready for manual confirmation.

### Effective KDE menu directory

The live private KDE uses XDG_DATA_HOME=/home/e/.cache/lay/kde-live-20261019/data,
not the normal ~/.local/share. Before finalizing the menu, move only our newly
created, hash-matched desktop entry into this session's applications directory;
remove the unused standard-path copy. Preserve all pre-existing entries. Refresh
only the KDE application cache under the existing remote guard. Launcher and
library remain in their declared HOME paths. No default/MIME or GNOME change.

## Installed setup, manual acceptance pending — 2026-10-28T17:46:48.978456+00:00

Unchanged C compiled with Wall/Wextra/Werror,0warnings;2/2 existing semantic/
launcher testsPASS, shell syntax and graph/architecturePASS. Built library
SHA924b59822e26f5ad3e6e0daf2a8f762b15e6905bb0b3683ce56bee55744a6f22
is byte-identical to the installed GNOME adapter. No C/Snap-launcher or functional
Rust edits. Source1302files/d44e72639f97c0a94802308b97319f60f69b61cc982d0c7c426db505c847edf9; six graph metadata updates.

Normal WM_DELETE closed authorized owned Firefox520337. Restarted manual
Firefox719893/window67108885: page opened/active, mapped library and inode verified,
async GTK IBus, same actual user profile. No new profile, WebDriver or automatic
text input. Daemon/engine/IBus/tray/config unchanged, no local GNOME action or release.

Preserve setup-v1: its initial profile assertion counted the program-directory
.parentlock updater lock as a second user profile. Read-only v2 excludes that
exact known installation directory, verifies the one same real profile/prefs.js;
no profile mutation, waiver, second restart or reinstall. Menu entry moved only
from our new unused standard copy to actual private KDE XDG_DATA_HOME. Cache
buildPASS; initial UTF-8-only check was invalid for Qt's binary index. Read-only
v2 verifies UTF-16 entry, without rebuilding cache or altering desktop again.
No MIME/default-browser change; menu Firefox (Lay), OnlyShowIn=KDE.

READY_FOR_MANUAL_CHECK_NOT_FUNCTIONAL_PASS. Initial user after-autocorrection
FAIL retained; after-setup result unknown. Do not update306P55F79B/440 from setup.
Compact receipt: kde-firefox-client-adapter-2026-10-28-verification.json.
CROOT is /home/e/projects/lay-development-runner/kde-install-control-20261020.

## User manual acceptance — 2026-10-28T18:18:58.453828+00:00

User explicitly confirmed: «да это ты починил». Accepted scope: original
first ordinary KDE Firefox input, Double Shift after autocorrection, with the
installed existing Firefox client adapter. Treat this exact setup/case as
finished; do not repeat tests/builds/browser restarts to reconfirm it. No claim
about all ten fields, reverse toggle, rapid bursts or a new complete matrix.
Original user FAIL and setup observations retained. No runtime/service/source
implementation change for this confirmation. Full matrix remains historical
306PASS55FAIL79BLOCKED/440; this manual PASS is separate.
