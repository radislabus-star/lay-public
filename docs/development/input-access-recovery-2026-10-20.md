# Startup input-access recovery — 2026-10-20

Status: SOURCE_VERIFIED_NOT_INSTALLED; bounded review accepted. Owner: daemon startup and the existing
TD-146 installer helper. Accepted checkout: `lay-browser-final-fixes-20261018`,
HEAD `fd3c19fa11da8618b4ed4f684f5e0c6252014bcc`, source version 1.0.93.
This task does not publish or install a local release.

## Observed failure and proof boundary

Read-only SSH checks on 192.168.3.94 found the KDE session on X11 :71, reached
by Remmina through the existing VNC forward. IBus selects `lay-ime-ru` and the
running engine hash is the previously tested `955fdd5070bb83a1eec30ccf6d875bfff2e5c4b7faa6abe7dd1cfad09e8bd134`.
CLI reports 1.0.93. This is not an old-version diagnosis.

The daemon cannot discover a readable keyboard. User e has no input-group
membership and event nodes are not readable. More importantly, sysfs key
capabilities show only one typing-capable device, `lay-virtual-keyboard`, which
is correctly excluded to prevent feedback. Granting device permissions alone
cannot create a keyboard or deliver VNC events to evdev. First failed transition:
device discovery -> physical gesture listener, before input ownership/ranking.
The obsolete error suggesting input-group membership contradicts TD-146's
session-scoped uaccess provisioning and must be replaced.

Tab is a separate IME path. The user's already-open Kitty lacks
`GLFW_IM_MODULE=ibus`; the successful owned Kitty supplied it. This task must
not call keyboard-access repair a fix for Tab, all KDE clients, or remote keys.

## Options

1. **9/10, selected:** launch a single diagnostic window on the existing
   daemon's first device failure. A small shared tool probes capabilities and
   actual open access, explains missing hardware versus missing permission,
   and invokes the existing setup helper through explicit graphical admin
   authorization when appropriate. Add an explicit menu entry on both desktops.
   Existing late-device recovery resumes automatically when access is restored.
2. **6/10:** add only a menu diagnostic. Smaller change, but a user with the
   reported startup failure still receives no visible explanation automatically.
3. **2/10, rejected:** silently add the user to input or chmod device nodes.
   This revives retired persistent access and does not fix the observed VNC
   topology. A remote-event bridge is a separate input-architecture decision.

## Consequences and invariants before code

- C01/C04/C07/C10 remain unchanged: daemon owns physical gestures, IME owns
  its input contexts, and the existing installer owns privileged provisioning.
  No detector, input owner, replay route, ranking authority or success bypass
  is added. The diagnostic probe never selects runtime devices.
- Candidate/lattice retention, ranking, false authority, package identities,
  online reloads, feedback and learning are untouched. No new user-text logs.
- A failed discovery may spawn one nonblocking helper per daemon invocation;
  repeated one-second discovery must not reopen it. Dialog singleton scope is
  the current display/session. No new retry timer is added to the daemon.
- Probe work occurs once at failure or on explicit user action, never in the
  typing hot path. Device opens are nonblocking and closed immediately without
  reading input. GUI and privileged authorization run outside the daemon.
  Closing/dismissing the window leaves normal input and late recovery intact.
- GUI dependency is the already used GTK/Python desktop stack; install and
  update must ship the diagnostic and the existing provisioning resources.
  Missing GUI/Polkit/setup support produces an actionable stderr error, not
  success. Headless runs must not start a graphical authorization loop.
- Privileged repair requires a deliberate button and the normal Polkit window.
  It invokes only `setup-input-access.sh`; no usermod, device chmod, service or
  IBus restart, alternate transport or global environment mutation is introduced.
  Unknown custom udev files remain protected by that helper.
- After any repair attempt, probe again. An installer exit zero alone never
  becomes READY; inactive/remote seats and missing hardware remain explicit.
  Root cancellation/failure must remain visible and retryable by the user.
- No synchronous child wait on the daemon thread. GUI handles its own bounded
  command completion; errors cannot kill or block the input monitor. Late
  device recovery remains the existing owner and no stale input is replayed.
- Rollback removes the helper/menu/startup hook, not administrator rules or
  unrelated devices. Packaging/uninstall removes only the newly owned files.
  Maintenance cost is one diagnostic implementation and reused provisioning,
  without a second permission-management protocol.

## Verification to record

Remote-only tests of actual probe outputs using synthetic filesystem/device
fixtures: readable keyboard; denied keyboard; absent keyboard; excluded Lay
virtual keyboard; inaccessible metadata; missing/denied uinput; denied
non-keyboard nodes; repair success without readiness; cancelled/failed repair;
headless and duplicate prompt behavior. Validate packaging and both menu
entrypoints, existing installer access tests and daemon scoped checks. Record
actual test counts rather than inventing a denominator in advance.

Perform one independent Harness2 frozen-diff review (lay2/gpt-6.1-sol/xhigh),
at most two repair passes, followed by owner verification. A fresh isolated
remote GUI check can establish window rendering only; no real privilege change,
user-field input, IBus restart or user-session restart is needed for this task.
Full native browser/Double Shift/Tab acceptance remains NOT_TESTED by these
diagnostics. Existing browser receipts retain their original scope.

## Implemented behavior and measured evidence

The shared `scripts/lay-input-access.py` diagnostic is packaged as
`~/.local/lib/lay/bin/lay-input-access`, linked from `~/.local/bin`, with the
existing setup helper and all three comparison/provisioning resources alongside
it. The existing Rust release-binary list stays Rust-only. GUI and Polkit package
dependencies are included for supported installer families. Both menus expose the
same diagnostic. KDE retains/reaps its helper through the existing status timer;
GNOME uses asynchronous Gio completion and visible errors.

The daemon launches a sibling diagnostic at most once per invocation after its
existing device failure. No launch occurs in detect-only mode. Child status checks
are nonblocking. Closing the GTK window during setup hides it while retaining
child supervision and the session lock; both Close and window-manager close were
exercised. A second attempt cannot start a concurrent privileged operation.
Normal Polkit authorization is confirmed/cancelled by the user. Only the actual
privileged setup after authorization receives TERM after 30 seconds and KILL
after a further five-second grace period; no authorization timeout is promised.
Setup exit zero is followed by a fresh probe.

Independent review exposed two additional cases, both repaired before the final
source proof. KDE now uses the same child-completion handler before a new menu
launch and on its existing status timer, so clicking again cannot discard an
earlier failure. A known keyboard that disappears or fails with an I/O error is
classified as DEVICE_DISCOVERY rather than permission denial. Only an actual
EACCES/EPERM can establish keyboard permission failure. An unavailable uinput
device likewise does not trigger an administrator prompt without an appropriate
missing-device or permission diagnosis.

Evidence root (outside Git):
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/input-access-recovery-20261020/`.

| Check | Result and scope | Receipt |
|---|---|---|
| Daemon | 320/320 PASS, scoped Rust source checks | `/home/ubu/.cache/lay/development/run-0nlc0wf1/RESULT.json` |
| Device diagnostic and existing provisioner | 29/29 PASS, isolated device/filesystem fixtures | `revision3/checks.json`, `check-0.log` |
| Existing release/menu regressions | 27/27 PASS | `revision3/checks.json`, `check-2.log` |
| GTK/Qt controls | 7/7 PASS: six GTK controls and one real Qt menu regression, private Xvfb/session bus and harmless child fixtures | `revision3/check-3.log` |
| Packaging and public install/update/uninstall | PASS in managed temporary homes, no actual installation | `revision3/check-1.log`, `check-5.log` |
| Shared GNOME adapter | GJS module load PASS, not Shell UI acceptance | `revision3/check-4.log` |
| Earlier Qt menu probe | PASS: real isolated Qt action, duplicate notice and child-failure reporting | `revision2/qt-entrypoint.json` |
| Real device preflight and GTK rendering | NO_KEYBOARD displayed; repair disabled; screenshot retained | `revision2/gtk-real-preflight.json`, `gtk-real-preflight.png` |

The first optional screenshot script failed before opening a window because it
imported Gdk4 before Gtk3. Its failure log is retained. Selecting Gdk3 explicitly
in the probe fixed this tooling error; no product change was made for it. The
separate GTK source tests already passed. Public shell regressions skip GUI
tests without a display; the final dedicated Xvfb invocation executed all seven
GTK/Qt tests without skips.

Fourteen source/document files matched origin and revision3 worker hashes when
the final checks were collected. Subsequent documentation/graph closure does not
change tested program bytes; the final acceptance manifest separates these files.
Twenty read-only real-device probes report zero supported keyboards and available
uinput; median 0.322 ms, p95 0.349 ms. This measures device preflight only, not
typing/Double Shift/Tab latency. `revision2-preflight.json` pins this earlier
measurement; `revision3/regression-proof.json` records the final implementation.

No new native browser matrix was run. Historical browser receipts remain at their
original scope. Real seat ACL changes, actual user-session startup notification,
physical Double Shift and IME Tab acceptance are NOT_TESTED by this change. No
installation, release, IBus restart, learning change or delivery-route change occurred.

## Independent review and regression closure

Harness² is actually used with project `lay2`, `gpt-6.1-sol`, `xhigh`. The first
240-second base call timed out after writing partial findings; that failed run
is retained as failed evidence, not accepted review. A task-local configuration
raises the limit to 600 seconds per model call; global settings are unchanged.
A second attempt accidentally prohibited even read-only source inspection. Its
unread result was rejected and its own workload stopped; no source was accepted
from that run. The corrected task explicitly permits read-only inspection only.

Current task: `20261019T234336Z-393df45a`. Its base review scores the frozen
pre-fix implementation 8/10 and identifies the two P2 cases corrected above.
`revision3/regression-proof.json` proves both regression tests fail against the
frozen old implementations, then `revision3/checks.json` proves the full affected
checks pass after repair. The final aggregate report completed successfully: eight successful calls, zero
failed calls and no fallback stages. It retains score 8/10 on the frozen pre-fix
source and ranks the KDE issue P2 and error classification P3. The owner accepted
this bounded static review after inspecting its final artifact, verifying seven
frozen input hashes and independently proving both repairs. Frozen reviewer
input does not silently become a review of later source bytes. No speed or token
savings are established by this run.

Final read-only device-open preflight: 20 NO_KEYBOARD results, zero supported
keyboards and available uinput; median 0.370 ms, p95 0.427 ms. This isolated
measurement is unrelated to end-to-end typing latency or universal KDE support.

## Graph and installed-updater closure

The canonical remote `scripts/update-architecture-graph.sh` completed PASS at
2026-10-19T23:53:06.888437+00:00. Receipt:
`graph-source-final-v2/RESULT.json` under the evidence root. All six output hashes
were verified on copy-back; all 730 nongenerated Rust source pins still match the
successful daemon gate. This is source graph consistency, not native acceptance.

The actual installed GNOME and KDE menu entrypoints resolve to `lay-update`.
Read-only `lay-update --check` returned success locally and remotely at
23:53:15Z, both identifying installed/latest stable 1.0.93. Receipt:
`installed-updater-checks.json`. No update/install action occurred, and the new
unpublished diagnostic is not implied to exist in that installed release.

Final Harness report:
`/home/ubu/.local/state/harness2-codex/projects/lay2/tasks/20261019T234336Z-393df45a/report.json`.
The selected review is copied to `HARNESS-REVIEW.md`; `acceptance.json` pins the
final source, test receipts, graph, updater and independent acceptance scope.

The review also records a UI reporting limitation: explicitly dismissing the
window while setup runs dismisses its eventual result; the hidden dialog is
released after completion. A cancelled operation remains distinct while visible.
No automatic reopening or desktop notification is promised after dismissal;
users can reopen the shared diagnostic for a fresh device-state check. The close
fixtures prove successful-child lifetime/lock release only, not delivery of a
failed or timed-out result after dismissal. This limitation remains explicit;
this change does not claim that untested result-delivery behavior is accepted.
