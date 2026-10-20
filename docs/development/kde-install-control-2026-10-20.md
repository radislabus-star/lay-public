# KDE diagnostic installation and daemon controls — 2026-10-20

Source: `/home/ubu/projects/lay-browser-final-fixes-20261018`, branch
`codex/ime-muted-hint-20261019`, base
`fd3c19fa11da8618b4ed4f684f5e0c6252014bcc`, version 1.0.93 plus an
unpublished local patch. Preserve the unrelated dirty work.

The user explicitly authorized immediate remote KDE installation and requested
that future finished fixes be installed there without another installation
confirmation. This applies to the remote KDE machine, not the local GNOME
desktop. Preserve user fields, other desktops, Codex/Hindsight, IBus and Plasma.

The first missing delivery was installation itself: the input-access work had
passed source/isolated UI checks but was not in the KDE runtime. The installed
tray contained `Lay включён`, with no explicit daemon restart. Three new actions
in `Демон Lay` use the existing runtime controller and target only
`lay-daemon.service`; they do not apply input channels or restart IME.

Evidence root:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/kde-install-control-20261020`.
Remote source/build worker:
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.
`REQUEST.json` pins the payload and all 730 nongenerated Rust inputs to the
previous 320-test daemon proof. `final-checks.json` and `BUILD.json` record the new
focused controls/UI/build result. Independent Harness² static review is
limited to the frozen new controls and their tests; it is not runtime proof.

Installation completed on the remote KDE session at 2026-10-20T00:26:25Z,
including the review fixes. Loaded daemon SHA-256 is
`1de1a316968f5ea5b1d1872d306e0341a2bc2b2a43ddf4217df7b4a6754ac8b2`;
daemon PID is 1163855. Final installed tray SHA-256 is
`6e7428877198d9a8961d93862cc959b6c9c43d1eb04bf18114af183f408a9eb0`;
tray PID is 1182223. Only these two services were restarted during deployment;
the final tray repair did not restart the daemon. IBus, the surviving Lay IME,
Plasma/KWin and the user's Kitty kept their process identities and executable
hashes. No local GNOME installation or publication was performed.

`deployment.json` and `menu-review-deployment.json` pin installed files,
running processes and preservation checks. Remote backups remain under
`deployment-attempt-2/deployment-backup/` and
`deployment-menu-review-backup.py`. The first deployment was rolled back by
an erroneous installer assertion that treated the diagnostic's expected exit
1 for NO_KEYBOARD as a deployment failure. That assertion was corrected to
validate exit status against the returned diagnostic status; the failure and
rollback receipts are retained. This was an installation-check error, not a
keyboard recovery.

Harness² project `lay2`, task `20261020T001156Z-ba776735`, completed with
status/core `ok`, eight calls, no failed calls or fallback stages, using
gpt-6.1-sol/xhigh and a task-local 600-second timeout. Frozen input was unchanged.
The selected static review scored the pre-fix controls 7/10. Its master-state,
full-runtime toggle coverage and private-history findings were fixed; earlier
review stages also found the unbounded wait and discarded command error,
which were fixed. A controlled old-source run failed five checks across six
tests. Final remote proof: six controls/controller/actual-Qt tests, seven
existing diagnostic UI tests and 18 Rust tray contract tests PASS. The first
repair check also exposed a test retaining a deleted QAction after a real menu
rebuild; the test now reacquires the rebuilt action. These source/UI results
are not native input or browser acceptance, and there is no new independent
post-fix review score.

The automatic installed GTK diagnostic was observed as PID 1163860, window
`0x2800003`, title `Lay — доступ к клавиатуре`. The owned-window screenshot
`installed-diagnostic.png` shows NO_KEYBOARD guidance, retry/close actions and
disabled privilege setup. No input was injected. The tray's live process and
installed source were verified; its custom popup is not exported as a DBus
menu, so action wiring was verified in isolated actual Qt rather than claimed
as a live click acceptance.

`installed-updater.json` verifies the installed menu uses
`/home/e/.local/bin/lay-update --menu`, resolving to
`/home/e/.local/lib/lay/update.sh`; its check successfully obtained stable
1.0.93 on 2026-10-20T00:26:53Z. The local installed check also returned stable
1.0.93. These unpublished patched bytes are separate from that public release.

The diagnostic's supported-keyboard discovery remains `NO_KEYBOARD`
on the VNC topology; installing its explanation does not manufacture a
physical evdev keyboard. No permissions are changed without the explicit
diagnostic authorization flow. Full native browser coverage, user-visible
physical Double Shift and Tab remain NOT_TESTED. The historical modified-stand
browser table remains 300 PASS / 20 FAIL / 0 BLOCKED out of 320.

Fresh environment evidence: zero USB HID interfaces and zero devices under
`/sys/bus/hid/devices`; event devices contain power/sleep/video/audio and
Lay's intentionally excluded output keyboard. x11vnc PID 3701035 serves
Xvfb :71 on port 5905 without pipeinput. This is not an input-group failure.
The installed x11vnc help documents `-pipeinput tee:...` and a UINPUT mode,
but notes US-keyboard limitations. No VNC reconfiguration, synthetic virtual
keyboard, udev/ACL change or additional input owner was introduced. A remote
input backend must be proved separately; manufacturing an empty event device
would not establish that the user's real keys reach Lay. The user confirmed
**VNC-only input; no keyboard connected to the mini-PC**.

## Confirmed remote-input boundary

The first absent transition is **x11vnc/XTest → a kernel keyboard event
readable by the daemon**. Xvfb receives the remote keys directly; the managed
IBus Shift branch is deliberately observational, and the existing daemon
remains the C04 gesture owner. Moving that gesture into the legacy IME would
change the contract rather than repair device permissions. No such change was
made. Merely connecting an idle USB keyboard would not make VNC keys originate
from that keyboard.

An owned Xvfb :99 / loopback-only x11vnc 0.9.16 experiment tested the stock
`tee:UINPUT:K,direct_key=<private capture file>,nouinput` mode. No kernel input
device was created and no key was sent to a user desktop. Latin A, left/right
Shift, Tab, Space and Backspace produced their expected down/up event records;
legacy and Unicode Cyrillic я produced **zero** keyboard records. Kernel input
inventory was unchanged. Evidence: `x11vnc-direct-mapping/result.json`, raw
capture and server log; the installed help and pinned upstream 0.9.16 sources
are also retained. This rejects stock UINPUT as a complete Russian-input
repair; these eight mapping observations are not Lay/browser PASS counts.

A separate owned GTK Entry in the actual KDE :71 received XTest Cyrillic
input: its actual text was `пров`, then **`проверка `** after Tab, cursor 9.
The existing IBus/IME and engine selection were preserved, and focus was
restored to the existing diagnostic window. Evidence:
`kde-owned-gtk-ime/result.json` and `before-tab.png`. This proves the observed
Tab text effect on this one client without evdev. It does **not** prove the
candidate's visual publication: no GTK preedit signal was observed, and the
window-only screenshot does not establish the separate panel's rendering.
The diagnostic's 641 ms includes deliberate observation waits and is **not a
product latency measurement**. Full native browser protocol, visible hint,
Qt Tab and physical/VNC Double Shift acceptance remain NOT_TESTED.

An initial ad-hoc `ibus address` check omitted the private `XDG_CONFIG_HOME`
and returned null. Repeating with the actual running IBus environment found
the existing bus and `lay-ime-ru`. This was probe setup, not loss of the
installed engine. No IBus restart or environment change was used to repair it.

`runtime-final.json` at 2026-10-20T00:40:14Z confirms daemon/tray PIDs unchanged,
all six preserved process identities/hashes, the original x11vnc process,
Lay RU engine and restored focus. `NO_KEYBOARD` remains expected. Installation
and bounded diagnosis are complete; **VNC gesture compatibility is not fixed**.
The next implementation must provide a compatible remote input acquisition
path to the existing daemon owner, preserve all user sessions and output
contracts, and prove RU/EN/modifier/release behavior before deployment. Do not
replace this missing proof with a dummy device, a permission grant, a second
IME gesture recognizer, or the old browser totals.

The canonical graph refresh completed successfully at 2026-10-20T00:29:07Z:
`graph-source-final-v2/RESULT.json`, six output hashes and 730 nongenerated Rust
source pins verified. Subsequent changes are evidence-only documentation;
no additional executable source or runtime authority changed.

### Follow-up: X11 observation, without runtime integration

An additional owned Xvfb/loopback RFB experiment observed one VNC left-Shift
press/release through XI2 RawKeyPress/RawKeyRelease (keycode 50, source 5).
An independently sent XTest Shift uses the same source identity. Thus XI2 can
observe this remote input, but source ID alone cannot distinguish VNC input
from a program generating XTest keys. This experiment does not implement a
gesture detector, authorize an edit, prove duplicate suppression or establish
input isolation. Evidence: `x11-observation-probe/result.json` and its separate
VNC/XTest logs. Both servers were experiment-owned; the live display and kernel
input inventory were unchanged. Native Lay acceptance remains NOT_TESTED.

The first launch used an incorrect installed-resource path and exited 127
before starting an experiment. The corrected launch was refused by the
resource guard at I/O full avg10 44.54; nothing was run outside the guard.
After pressure subsided, the same guard admitted the experiment at 2.22 and it
completed successfully. Neither launch error is a Lay runtime failure.

The remaining integration boundary is not another permission repair: an X11
acquisition adapter would have to feed the existing daemon gesture owner and
preserve source ordering, focus invalidation, feedback exclusion and required
input isolation. Existing committed-tail/daemon replay paths still require
evdev grab/uinput guarantees; observing XTest cannot silently grant them. No
production implementation, new input listener, service change or installation
was made in this follow-up. The installed VNC Double Shift remains unfixed.

## Installed settings window acceptance

`settings-window-final-v3/result.json` proves the actual tray-resolved fallback
settings entrypoint opens a visible native window, title Настройки Lay, XID
50331652, PID 2624766. Only that newly created window was closed. The tested
settings.js/settings_view.js match source; an older tray_support.js was backed
up and atomically refreshed after syntax validation. Earlier v1/v2 were
preflight path/parity failures, not window failures. No user window was closed,
no global desktop/IBus service restarted. This is UI acceptance, not input proof.
