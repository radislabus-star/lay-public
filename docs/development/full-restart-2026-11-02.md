# Explicit complete Lay restart

The user accepted the local GNOME 1.0.123 runtime after its complete restart
and requested the same operation from a menu button. This closes the preceding
local browser diagnostic by user acceptance, not by a new automated matrix.

Baseline: checkout `lay-dependency-updates-20261027`, commit
`4fae57d50b031984c1e0ddc8796e78e142b2a595`; installed/live IME 1.0.123,
SHA-256 `2a4c636a3be959ce096754909599a202146b7711157c2563e6980f5c8909d429`.
Preserve the existing GNOME version-metadata fix and working handoff.

## Scope and map before editing

This is an explicitly authorized lifecycle/menu integration task, not a
correction change. The missing transition is a user-accessible command for
the complete restart; existing `restart` resets only daemon/managed IME.

```text
GNOME menu: extension/.../lay-impl.js
  → extension/.../tray_support.js
KDE menu: scripts/lay-kde-tray.py
  → existing scripts/lay-runtime-control.sh full-restart
    → separate user unit: lay-full-restart.service
      → scripts/lay-full-restart.sh
        → existing runtime-controller desktop/layout/IME helpers
        → current L1.1 socket owner, L3, daemon, IBus/managed IME
        → existing GNOME extension reload helper on GNOME
```

Exact source boundary is declared in the owning ADR and active integration
scope. No Rust input/correction/ownership/verifier/executor/rendering file,
model, user config or browser launch setting may change. No new delivery
route, parallel word owner or fragment implementation is part of this task.
The existing quick restart and all automatic configuration reloads are frozen.

The explicit worker must restore the selected RU/EN Lay source, establish an
XKB global engine before creating a fresh Lay engine, propagate failures, and
remain alive when its calling tray reloads. Only the L1.1 process identified
by its existing socket and verified executable/user may be terminated.
Non-IME mode must not restart global IBus. Do not start optional inactive L3
or KDE tray services. No foreground browser activation or synthetic input.

## Verification and measured result

- Nine new isolated lifecycle tests PASS, including four desktop/language
  combinations, fresh XKB bootstrap, retired IBus socket address, failure
  reporting/restoration, optional-service state and foreign socket refusal.
- Existing KDE startup tests: 12 PASS. Canonical remote development entrypoint:
  `dev-check.py check --target test:tray_ui_contract --compact`, 19/19 PASS,
  `/home/ubu/.cache/lay/development/run-h1vdvwzm/RESULT.json` (13.9 seconds).
  Bash/JS syntax, source-scope check and unchanged Rust-source comparison PASS.
- Local Qt menu attempt retained an environment error: eight daemon-shell
  tests PASS, one GUI test ERROR because neither PyQt5 nor PyQt6 is installed;
  the separate persistence GUI test SKIP. No native KDE GUI PASS is claimed.
- Installed only the frozen lifecycle script bundle and the two GNOME menu
  modules; Rust binaries, models and browser settings remain unchanged.
  `lay-runtime-control` points to the frozen bundle, not a mutable checkout.
- The installed GNOME accessibility tree exposes `Полный перезапуск Lay`.
  Its hidden menu item has no AT-SPI action, so a mouse/accessible click was
  NOT executed. One actual cycle invoked the exact installed controller command
  used by that menu callback, without activating a browser or injecting keys.
- Actual cycle PASS, 6819 ms (one observation, including completion polling):
  IBus 1343894→1761828, daemon 1344025→1761913, IME 1345111→1762198,
  L1.1 1344087→1762230, L3 1343949→1761914. Each executable hash is unchanged.
  Live/installed IME 1.0.123 hashes match; GNOME and IBus both Lay RU;
  L1.1 socket peer is the fresh server. GNOME extension reload check PASS.

Receipts: `/home/ubu/.cache/lay/development/full-restart-20261102/`, especially
`live-launch.json`, `live-result.json`, `verification.json`, `installed-menu.json`
and `before-install/`. A failed worker attempts to restore the previously
active daemon/IME but still reports failure; it never turns recovery into PASS.
The worker drops the retired `IBUS_ADDRESS` locally after restarting IBus.

The Graphify static scan and guarded AST update were refused by shared RAM
admission. No retry outside the guard, no graph-freshness claim. No new release
was published. No source change to input/correction/owner/verifier/executor,
no physical typing matrix, native KDE installation or foreground browser work.
Historical KDE browser totals remain 344 PASS / 15 FAIL / 81 BLOCKED of 440;
the user's local 1.0.123 acceptance is separate from those historical rows.

Rollback: restore the preceding runtime-controller symlink and the backed-up
installed GNOME menu modules, then reload only that extension. Runtime Rust
binaries remain the accepted 1.0.123 bytes.

## Next patch

The user confirmed inclusion of this verified feature in the next patch.
Release notes: «Добавлен пункт “Полный перезапуск Lay”: перезапуск компонентов
ввода и моделей с восстановлением выбранной раскладки и уведомлением о результате».
Local GNOME acceptance and source checks remain as recorded above; this is not
a new native KDE acceptance or an instruction to publish immediately.

## KDE menu installation follow-up

The user now requests the missing KDE full-restart menu item. Read-only inspection
on 2026-11-02 found the live tray PID3538867 running the preceding cached script;
its exact source diff is only the new action and callback (eight added lines).
The user service is inactive with stale display-bus settings; the tray was started
separately. Install the same verified three-script bundle and update only the tray
launch to its existing live session environment. Preserve the old links and unit
configuration. Restart only the tray; do not run the full-restart action or change
IME/daemon/model bytes as part of installing the menu. Verify registration and
the actual menu separately from full lifecycle behavior.

Upstream read-only PR check at 2026-11-02T02:38:15+03:00: zero open PRs.

KDE installation completed: the three-script frozen bundle is installed under
~/.local/lib/lay/runtime-control-full-restart-20261102; both canonical links
point to it. Only the tray service launch was rebound to the preceding tray
process's actual KDE display/session environment. New tray PID3962487 is active
and registered as :1.226778/StatusNotifierItem. A direct screenshot of the
opened native menu visibly confirms «Полный перезапуск Lay» under «Lay включён»
and version1.0.123. Installed daemon/IME hashes were preserved. Full-restart
itself was not invoked. Remote receipts: ~/.cache/lay/development/full-restart-20261102/
install-before.json, install-after.json, tray-registration.json, live-menu.png.
Local screenshot: ~/.cache/lay/development/full-restart-20261102/kde-live-menu.png.
AT-SPI did not expose the tray; its failed probe is retained, and visual acceptance
uses the actual screenshot rather than that failed accessibility assertion.

## 2026-11-09 — full-suite proof pin maintenance

All34Python modules found one stale source pin for the authorized additive full-restart command:478tests,465PASS/1FAIL/12SKIP. Receipt remote `/home/e/projects/lay-development-runner/letter-fragment-full-gate-v44-20261109/all-python-unit-tests.log`. Exact diff adds only launch_full_restart helper, source-import gate, explicit dispatch and usage token. Existing dispatch/function bytes are unchanged. Whole-file old92843...→new066d84... pin updated under explicit ADR2026-11-09-full-restart-controller-proof-pin. New reverse projection must recover exact accepted v1.0.123 bytes, and a controlled legacy-dispatch mutation must fail that old pin. Existing lifecycle checks remain, no runtime code change. Focused and complete gates pending; no installation or release.
