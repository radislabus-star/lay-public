# Stable releases and menu auto-update

Date: 2026-10-19. Owner: `update.sh`; installation owner: `install.sh`.
User request: a switch in both desktop menus, enabled for this user.

## First failed transition and alternatives

Installed `~/.local/bin/lay-update` resolves into an old development checkout.
Its `@{u}` has no commits ahead, although public stable 1.0.92 exists and the
installed application is 1.0.90. The failure is release discovery, before any
input or model decision. A development branch is not a release authority.

Compared designs (suitability out of 10):

* Keep tracking a development branch, adding a timer: 3/10. It preserves the
  demonstrated false-current result and can install unpublished development.
* Discover stable releases but run the full initial installer: 6/10. Viable for
  an attended fresh install, but repeats privileged dependencies, input access,
  IBus registration and legacy cleanup on every background update.
* Stable release discovery, an owned source cache, and a restricted mode in the
  existing installer: 9/10, selected. No second installation owner, no updates
  to a user's working tree, no background sudo, no repeated input setup.

GitHub's latest stable public release is the version authority. Fetch the exact
validated stable tag into a marked updater-owned checkout only when applying.
The installed updater is a regular file under `~/.local/lib/lay`, independent
of any development checkout. Network/API/tag failures are errors, never current.
An installed version newer than stable is retained; automatic downgrade is
forbidden. `--source-only` remains an explicitly requested developer operation.

The systemd user timer is the single auto-update state. Both frontends call the
same installed updater to observe/enable/disable it. No Rust config field or
input-service restart belongs to this switch. The timer invokes the same updater
and lock as the manual action; there is no second download/install pipeline.
Initial installation leaves the choice off; this user explicitly enables it.

## Consequence analysis before production edits

Candidate/lattice retention, rank and false authority: no input-core change.
Release authority changes from arbitrary Git upstream to public stable metadata;
the tag must validate before fetching/executing its installer.

Latency/CPU/RSS/allocation: menu state reads and writes must finish quickly;
GNOME uses asynchronous subprocesses. Builds run in a low-priority bounded user
service and Cargo's existing disk guard; they never run in a menu callback.
Background work can still contend for disk/CPU, so use a long interval, randomized
delay and a shared nonblocking lock. No speed claim follows from source tests.

Cache identity/invalidation: exactly one marked source cache, fetched by tag;
refuse a pre-existing unmarked or modified tree rather than overwriting it.
No user's repository is stashed/reset. Release metadata is not cached as current
after a network error. A successful application records the exact installed tag.

Package/delta reloads: retain the existing verified model-package installers.
The update mode skips initial setup, legacy learning/cache rebuilds and IBus
registration. Binaries use the existing temporary-file/rename installer. No
new package format, fallback or model authority is introduced.

Learning/feedback: preserve personal corrections, learning journals and config.
Never regenerate legacy live memory as an updater prerequisite.

Concurrency/staleness: one apply lock covers manual and automatic installation.
UI writes are serialized while pending; opening menus re-reads the timer state.
A disabled timer stops future scheduling, not an already running installation.
Disabling must not kill a binary replacement halfway through.

Failure/rollback: discovery/build failures leave installed files untouched.
Existing installer is not a full multi-file transaction; an installation I/O
failure may leave mixed binaries and must remain an error for retry, not PASS.
Retain the previous source tag for diagnosis. User can disable the timer and
perform an attended retry. Do not add an unproven automatic rollback framework.

IME/daemon compatibility: settings toggle never restarts input. Update-mode
installation neither restarts global IBus nor changes input sources, XML,
backend/config or compositor rules. Existing application refresh happens after
successful installation, preserving whether the daemon was running. A running
IBus engine may keep its old mapped executable until its natural lifecycle;
report that distinction rather than forcibly replacing the user's input session.

Maintenance: reuse `update.sh`, `install.sh`, the existing tray adapters, and two
systemd unit files. Remove the implicit development-upstream release discovery;
keep explicit `--source-only` isolated. No scheduler inside either tray.

## Proof plan and boundaries

Remote isolated executable updater tests cover latest-stable discovery from an
old tracking branch, unavailable/malformed API, exact-tag apply, no downgrade,
dirty/unowned source refusal, lock contention, menu quoting, timer controls and
unattended invocation. Installer tests verify update-mode skips input setup and
privileged initial-only actions. Frontend tests cover real switch adapters and
errors; source contracts alone do not establish physical menu acceptance.

After source checks: verify the actual installed symlink target, actual stable
release discovery, timer enabled/active state and both installed frontend files.
Record installed bytes separately from a loaded extension or running IME. The
previous 432 native-input receipt remains historical; no input-route change
requires repeating that matrix for this menu feature.

## Measured source evidence

Remote source checks on 2026-10-19: 36 updater/installer/input-access tests,
15 architecture controls, 6 hint lifecycle controls and 18 tray contract tests
passed. GJS executed the actual asynchronous adapter and the cached-loader
regression; neither was skipped. Shell syntax, scoped canon
against b72c50aac6454ec1cc3d30c9bb8f69e0e3332885, extension archive membership/
bytes and systemd directives passed. Unit validation used the source executable
in isolation; actual installed `%h` resolution is a separate installation gate.
The first unit check caught an unsupported directive; `ConditionFileIsExecutable`
passed the repeated check. No input runtime authority changed in these checks.

Receipt: `tech_debt/evidence/stable-auto-update-source-20261019.json`.
Detailed local evidence: `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/stable-auto-update-checks-four/`.
Release build and installed-menu/timer evidence are recorded separately under
`tech_debt/evidence/stable-auto-update-install-20261019.json` when performed.
No fresh physical input, GNOME/KDE rendering or latency claim follows from these
source checks. The accepted browser 150/10 + 150/10 matrix belongs to KDE only.

Installed discovery bootstrap at 2026-10-19T14:58Z: the regular installed updater
correctly reported installed 1.0.90, public stable 1.0.92, and an available
update. The timer remained disabled; this check performed no build or input
restart. The prior development-upstream false-current result was eliminated.

Installation exposed a second, independent failure at 2026-10-19T15:06Z:
GNOME retained a pre-1.0.92 entrypoint in memory. It staged four modules but
omitted the new hint module, so the implementation import failed and no Lay
DBus endpoint existed. The old verification script reported OK from file
equality alone. Keep one hint implementation: the existing staged support
module copies the missing dependency into the current per-enable directory,
then imports that fresh URI. No desktop or global IBus restart is required.
The regression verifies same-generation identity and next-generation freshness.
Runtime verification now waits for asynchronous loading and rejects missing or
stale endpoints. Controlled missing/stale endpoint tests catch the former false
PASS. After installing this fix, the real local GNOME endpoint returned 1.0.93.

Installed verification before publication: all ten installed binary hashes match
the guarded remote 1.0.93 build; installed updater is the regular libexec owner;
installed extension files match source and its loaded endpoint returns 1.0.93.
The actual off/on commands succeed. The timer is enabled and active, with next
run 2026-10-19 21:02:46 EEST. The global IME process remains PID 1166531.
At this point public stable is still 1.0.92, so discovery correctly reports the
installed version as newer. Publication and a real equal-version service run
are verified afterward; their final receipt updates the installation evidence.

Post-publication verification at 2026-10-19T15:19:16Z: public stable tag
`v1.0.93` resolves to `a59bdab4c11507d861ae6f71d0e5314c698d7c22` and was
published at 15:18:29Z. The actual installed `lay-update --check` reports
installed 1.0.93 and stable 1.0.93 with exit status 0. One actual run of the
installed `lay-auto-update.service` completes with `Result=success` and
`ExecMainStatus=0`; its journal records the same equal-version result. The timer
remains enabled. This proves installed stable discovery and the service's
no-update path; applying a future newer stable release remains untested. The
global IME process is unchanged, and no fresh native-input result is implied.
Final receipt: `tech_debt/evidence/stable-auto-update-install-20261019.json`.
