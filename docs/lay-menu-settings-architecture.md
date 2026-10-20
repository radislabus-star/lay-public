# Lay menu and settings architecture

Status: accepted for Lay 1.0.27; update/menu revision for 1.0.90 is recorded below.

## Ownership

```text
GNOME tray (daily actions) -----------+
                                       |
KDE tray (daily actions) -------------+--> ~/.config/lay/config.json
                                       |       |
GNOME standalone settings.js ---------+       +--> lay-daemon / lay-ibus-engine
                                       |
GNOME Extension Preferences prefs.js -+
               |
               +--> settings_view.js (the only settings view owner)
                        |
                        +--> tray_support.js (the only JS config/runtime adapter)
```

`settings.js` and `prefs.js` are entrypoints only. They must not define controls,
defaults, config normalization, or runtime commands.

## Accepted tray inventory

| Visible control | Owner | Effect | Verdict |
|---|---|---|---|
| Layout RU/EN | GNOME input source manager | Activates the opposite current layout | KEEP |
| Lay enabled | `lay-runtime-control start/stop` | Starts or stops managed Lay processes and selects the matching input channel | KEEP |
| KDE: Daemon Lay → Start / Stop / Restart | `lay-runtime-control daemon-start/daemon-stop/daemon-restart` | Controls only `lay-daemon.service`; never changes IBus, input source or delivery route | KEEP |
| Input mode | `text_backend`, `nanda_precognition`, `lay-runtime-control channel` | Selects uinput or IME as one coherent route | KEEP |
| Typing assistance | `typing_assist` | Enables live completion/correction assistance | KEEP |
| Autocorrection | `auto_replace` | Enables admitted automatic replacements | KEEP |
| Check updates | Installed `lay-update -> ~/.local/lib/lay/update.sh` | Opens a terminal; checks the public stable release and asks before source build/install | KEEP |
| Auto-update | Same installed updater, systemd user timer | Enables/disables periodic stable updates; never changes input config or restarts input when toggled | KEEP |
| Settings | `settings.js -> settings_view.js` | Opens the single settings surface | KEEP |
| Runtime status | `systemctl --user is-active` | Read-only service observation | MOVE TO DIAGNOSTICS |
| Recent actions | `recent_actions.jsonl` | Read-only recent edit receipt view | MOVE TO DIAGNOSTICS |
| Open logs | `journalctl --user` | Opens bounded service logs | MOVE TO DIAGNOSTICS |

## Accepted settings inventory

| Section | Control | Runtime consumer |
|---|---|---|
| Input | Input mode | daemon and IBus channel selection |
| Input | Typing assistance | daemon/IME candidate route |
| Input | Autocorrection | admitted automatic edit route |
| Input | Correction safety | L2/L3/DecisionCore admission policy |
| Input | Follow correction language | output/layout handoff |
| Input | Remember manual edits | local learning journal |
| Input | Bracket completion tail | IME preedit renderer |
| Keys | Manual correction trigger | daemon trigger dispatcher |
| Keys | Dedicated RU/EN keys | daemon force-layout hotkey owner |
| Compatibility | Layout environment | layout backend resolver |
| Diagnostics | Detailed action journal | action and NANDA trace flags |
| Diagnostics | Service status | read-only `systemctl` observation |
| Diagnostics | Open logs | read-only `journalctl` view |

## Removed surfaces

| Removed control | Reason |
|---|---|
| L2/L3 weights | Model-authority calibration is not a casual user setting |
| NANDA cells/passport | Research visualizer, not a product setting |
| NANDA autocorrection | Duplicate internal authority switch |
| Tap, Shift window, multi-tap timing | Internal compatibility tuning with no daily user contract |
| Multiple trigger taps | Experimental behavior without a stable product workflow |
| Correct before Enter | Not stable across applications and already documented as outside the public contract |
| Window-bound layout (`ptah_alexs`) | No complete rule editor or cross-desktop owner; hidden focus execution was removed too |
| Restart services | Diagnostics must observe; routine recovery belongs to runtime/service tooling |
| Daemon switch under diagnostics | Replaced by the explicit top-level `Lay enabled` product control |
| About submenu | Version and GitHub are available once in Settings |
| KDE auto input mode | Legacy alias of IME, not a distinct runtime route |
| KDE layout backend menu | Compatibility setting belongs to the shared Settings window |
| KDE NANDA dialog | Stale explanation of an older experimental route |

Removed config keys are not deleted from `~/.config/lay/config.json`. The JS
writer merges visible edits into the complete existing object so Rust-owned and
future keys survive a settings change.

## Gates

```text
one shared settings view
AND each visible action has a runtime consumer
AND model-authority keys are absent from user UI
AND diagnostics does not restart services
AND config save preserves unexposed keys
AND an inactive daemon blocks automatic IME source resynchronization
AND changing the saved input mode cannot start IME while Lay is disabled
AND changing any restart-owned setting cannot start a disabled Lay runtime
AND stale asynchronous service status cannot override a newer enable/disable choice
AND global ibus-daemon PID survives installation
```

Focused contract: `tests/tray_ui_contract.rs`.

### Explicit KDE daemon controls — 2026-10-20

The user requested direct daemon controls and immediate installation on the
remote KDE machine. The installed tray already contained the full-runtime
`Lay enabled` checkbox but no explicit restart command. Its installed files
also predated the source-verified input-access diagnostic. These are separate
from physical keyboard and IME acceptance.

The new top-level `Демон Lay` submenu calls the existing runtime controller's
three explicit daemon-only operations. Start is available while inactive;
stop and restart while active. The existing status observation refreshes these
states and the master checkmark from one observed state. Daemon-only commands
have a five-second client wait; a timeout does not claim the queued service job
was cancelled. Missing controllers, timeouts and nonzero exits show a warning
with the available error. The original full-runtime checkbox keeps its existing
behavior. The diagnostic submenu remains observational.

`tests/test_kde_daemon_control.py` exercises real shell dispatch, command
failure propagation, and actual Qt actions in an isolated Xvfb session. It
verifies that daemon commands reach only the daemon unit and that the menu
reflects observed state after success, failure and external state changes.
The test isolates recent-action history and asserts the master action still
dispatches full-runtime start/stop. A controlled old-source
run must fail these regressions. Installation must verify deployed hashes,
service processes and preserved IBus/Plasma identities. This work creates no
new input owner and does not certify Double Shift, Tab, or browser typing.

Evidence and final installation outcome are recorded in
`docs/development/kde-install-control-2026-10-20.md`.

Preflight:

- `docs/receipts/menu-settings-1.0.27-preflight.json`
- `docs/receipts/menu-settings-1.0.27-preflight-receipt.json`

## Verdict scope

Measured before implementation:

- global `ibus-daemon` PID: `3702`;
- active engine: `lay-ime-ru`;
- live config SHA-256:
  `0b080219015f3567125b75420319df0156243686445188bc44584b57849d531b`;
- old GNOME/KDE/settings UI implementation: 3,440 lines across the five owners;
- new implementation including the shared settings view: 1,334 lines;
- reduction: 2,106 lines, or 61.2%.

The change does not alter L1-L3 candidate generation, ranking, edit planning,
double-Shift undo, Space handling, or verifier authority. It does remove the
obsolete extension-owned window-focus layout policy. Runtime installation and
post-install PID/config parity are separate release gates.

Post-install receipt:

- `docs/receipts/menu-settings-1.0.27-post-install.json`

## 1.0.90 menu and updater revision — 2026-10-18

The user requested the update command back and a coordinated GNOME/KDE menu
refresh. Both menus group enablement, layout/input mode, assistance/autoreplace,
and settings/update/diagnostics with separators; status stays read-only. Existing
config/runtime owners and IME delivery paths retain their contracts.

`~/.local/bin/lay-update` resolves the installed symlink to the owning checkout.
Both trays call `--menu`; the common updater chooses the terminal, preserves
progress/errors and asks before a source build/install (including possible sudo).
`--check` only fetches and reports; `--source-only` never repairs runtime. The
installer originally required an opt-out from legacy cleanup; that cleanup is
removed by the 2026-10-19 installer repair below. A per-user lock excludes
concurrent updates. Divergence is refused before stashing; the checked upstream
is explicitly merged ff-only. Refresh follows the current desktop without a
global IBus restart. No release downloader, alternate settings owner, daemon RPC,
background polling or input-delivery route is introduced.

Consequences: updating still requires a Git checkout, network, build dependencies
and a terminal. A terminal/install failure is visible; menu-launch success is
not installation proof. GNOME and KDE visual acceptance must be stated separately.
No native input acceptance or latency improvement is claimed by this menu-only
change. Source, installation and physical-client evidence remain distinct.

Verification records (private development receipts, not bundled runtime data):

- `~/.cache/lay/development/menu90-20261018/UI-CHECK.log`: ten executable
  updater checks and seven checks of the actual Qt menu class passed in an
  isolated HOME. The installed-symlink regression fails against the previous
  updater. Qt uses the offscreen platform; an actual Plasma session is untested.
- `~/.cache/lay/development/menu90-20261018/REVIEW.json`: independent static
  review approved the implementation and rollback installer, 9/10, after two
  passes. The receipt pins the reviewed executable inputs and installer hash.
- `~/.cache/lay/development/run-qlkejm0j/RESULT.json`: all 3212 selected tests
  completed with zero failures, but final aggregation refused an obsolete
  known-failure manifest binding. Updating that digest retains the empty
  allowed-failure list; no test or assertion is waived. This failed aggregate
  is retained rather than rewritten as PASS.
- `~/.cache/lay/development/menu90-20261018/full-proof.json` binds the final
  source run and original request to the tested executable inputs.
  `BUILD.json` in the same directory records the remaining release gates,
  locked binaries and extension hashes; `installation/INSTALL.json` records
  the local outcome, backup/rollback, loaded process identities and preservation
  of config, sources, input mode, global IBus, L1.1 and L3. Each receipt's own
  verdict governs its scope; a build or menu-launch result does not establish
  installation or native-input acceptance.

KDE lifecycle repair: a `QProcess` parented to the tray would be destroyed when
the updater itself restarts that tray, potentially killing a kitty/xterm update
terminal. A detached-session `subprocess.Popen` survives that restart; the
existing status timer observes completion/errors and prevents duplicate launches.
Manual tray close reports an active update instead of silently exiting. This
adds no input owner, model/cache identity or training behavior. No hot-path
latency/allocation changes occur; the only process is the user-requested updater.
The updater lock remains authoritative across tray lifetimes. Failed launches
stay visible; installation backup/rollback and loaded-identity evidence remain
separate from menu activation. A second background updater or delayed restart
service would add ownership and removal cost and is unnecessary.

Decision: [restore updater and unify daily grouping](architecture/decisions/2026-10-18-menu-updates.md).

## KDE installer dependencies and unrelated-data preservation — 2026-10-19

The first failed installation transition on Ubuntu 22.04 was dependency
resolution: both the GitHub bootstrap and direct installer required
`python3-pyqt6`, which is unavailable in that distribution's repositories. The
existing tray imports also required Qt6. Both installers now select Qt6 when
available through apt and otherwise request `python3-pyqt5`; the same tray
implementation supports both bindings. Existing qdbus package-name selection
and the Arch/Fedora dependency routes are retained.

The direct installer also contained unconditional legacy Ollama removal unless
an environment flag was provided. That cleanup had no ownership proof for the
affected services, executables or models. It has been deleted rather than
exposed as another installation option. The documented installation command and
stable updater no longer need the preservation flag.

The KDE tray now honors the existing `LAY_CONFIG_PATH` override used by the
daemon and IME. It reads and writes that same configuration, preserving unknown
keys. No new configuration owner or input-delivery route is introduced. Rust
binaries, correction, learning and verifier authority are unchanged.

Measured evidence, with source identities and test names retained in receipts:

- Worker `~/.cache/lay/kde-live-20261019/installer-check-1/RESULT.json`: **39/39**
  focused installer/updater tests passed in **7.318 seconds**. Three new
  regressions detect the previous cleanup, unavailable Qt dependency and
  ignored config override against the pre-change source. Package installation,
  privileged commands and service actions in these tests are sandboxed.
- The actual Ubuntu 22.04 package probe selects
  `qdbus python3-pyqt5 libxcb-cursor0`. The actual KDE tray runs with PyQt5;
  its registered status icon was visually confirmed in the existing Plasma
  panel. The daemon and tray are active. This establishes GUI startup only.
- Worker `~/.cache/lay/kde-live-20261019/lay-start-receipt.json` pins the installed
  1.0.93 binary hashes and private-session launch environment. The existing IME
  process and user windows were preserved. No global IBus restart occurred.
- The installed menu updater resolves to `~/.local/lib/lay/update.sh`. Its real
  GitHub check initially returned HTTP 500 from the worker and correctly exited
  with an error. After independent API probes confirmed restored access, a
  second installed-updater check passed: installed 1.0.93 equals stable 1.0.93;
  the auto-update timer is enabled. No transport or retry behavior was changed
  to obtain that result. Worker `~/.cache/lay/kde-live-20261019/installer-live-check.json`
  pins the successful commands, unchanged binary hashes and preserved IME PID;
  `installer-live-check-network-failure.json` in that directory retains the
  earlier failure. A real newer-version apply remains untested in this change.

Not tested: a complete fresh download/build/install, other distributions' live
trays, or new GNOME native-input acceptance. The current private Xvfb/VNC desktop
has no evdev keyboard; the daemon reports unavailable keyboards. Double Shift
through that remote desktop is **BLOCKED** and is not certified by the active
service, registered icon or source tests. Existing browser receipts remain
historical. No input latency improvement is claimed.

## Shared input-access diagnostic — 2026-10-20

Both desktop menus add **Диагностика → Доступ к клавиатуре…**, launching the
same installed `~/.local/bin/lay-input-access` owner. GNOME uses asynchronous
Gio completion reporting; KDE retains/reaps the child through its existing status
timer and reports launch/nonzero-exit errors. Repeated KDE activation explains an
already running diagnostic instead of spawning another privileged operation.
The helper also holds a session/display-scoped window lock.

The common GTK window distinguishes missing keyboard, device permissions,
unknown discovery and device preflight readiness. Only an explicit button can
request normal administrator authorization for the existing input-access setup
helper. It rechecks actual access after cancellation, failure or successful setup.
Closing during setup is allowed; the hidden helper continues supervision until
completion. Runtime input ownership, delivery routes, models and settings are not
changed by opening the diagnostic. See TD-146 for provisioning ownership and scope.

Remote proof is limited to isolated GTK controls, real Qt menu launch/error
handling with a harmless fixture, source/packaging checks and read-only device
preflight. It does not certify an installed new menu, stock GNOME/Plasma interaction,
real privilege changes, Double Shift or Tab. Receipts and native exclusions:
`docs/development/input-access-recovery-2026-10-20.md`.
