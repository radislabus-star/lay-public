# Lay menu and settings architecture

Status: accepted for Lay 1.0.27.

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
| Check updates | Repository/developer operation, not an installed runtime setting |
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
