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
existing installer is called with `LAY_KEEP_OLLAMA=1`. A per-user lock excludes
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
