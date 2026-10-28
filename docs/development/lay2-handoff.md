# Lay — active seven-release series, 2026-10-28

User: autonomously implement/test/push/publish seven meaningful releases105–111, isolate integrations, ask no questions. Do not stop after an intermediate report. No Harness2/subagents. No local GNOME installation/restart/synthetic input. All builds/tests/graph updates/native input remote under the existing guard. Preserve C01–C10, common owner/reducer/verifier/executor and every honest FAIL/BLOCKED/NOT_TESTED.

Checkout `/home/ubu/projects/lay-dependency-updates-20261027`, branch `codex/browser-matrix-repair-20261027`. origin is PRIVATE; never touch its divergent main. public/main currentlye178be59563fdce592921402ce9e6634e9b2e9c7. Fast-forward workbranch/public main only, no force.

## Completed and published

-105:4ebd03f3, stable tagv1.0.105, updater PASS. Native7/7 andChrome220180P0F40B. C05 pair/NotReady continuation, exact boundary-successor fence; no new transport/owner.
-106:f88c4ba9, stable tagv1.0.106, updater PASS. Native7/7. KDE preparation entrypoints moved out of common observation, common witness remains sole owner.
-107:e178be59563fdce592921402ce9e6634e9b2e9c7, stable tagv1.0.107, all4CI SUCCESS, both updaters PASS. local installed104 sees107; remote107current. Source708926cec7f4600a831580464ef75679242eb359b9d72c20a2480b058e3f2e7c.3550source/1113IME/18gates/10artifacts/0warnings. KDE read-only resample uses current key token and unchanged field/focus/lease/layout/append witness; strict edit predicate unchanged. Root: old binding token after key-scope settlement blocked second actual read. CausalRED83/85→GREEN85/85; nativeSearch5/5. Chrome policy moved display/chrome.rs→kde/chrome.rs, function bodies unchanged.

107fullChrome220179PASS0FAIL41BLOCKED0NOT_TESTED. Separate realupperomnibox0PASS1FAIL2NOT_TESTED: first own hint absent, native selection present, char geometryzero; previous independent limitation preserved, not a PASS. FreshFirefox220baseline130PASS51FAIL39BLOCKED0NOT_TESTED.50rapid failures generally miss intermediate cadence; one roundtrip refused. Do not claim latency closure. Native timing/full scope receipts are in versioned tech_debt/evidence/release-1.0.107-verification.json.

## Current108 — source frozen, full check running

Baselinee178be59; actual sourceSHA `b31a47181e330d17f46ff2fa95b9a83df9184d72ced4923d86b0d5fa4cf7297f`,1291files. New labels1.0.108 synchronized source-only. All original test identities/classifications preserved, one new semantic daemon test:3551correctness+package=3515+36,1113IME,11performance,41ignored. Zero known-failure declarations. Source files are DIRTY, do not overwrite.

Scope: `src/bin/lay_daemon/layout_controller/ime_bridge.rs` and new `ime_bridge/settlement.rs`. Existing wait loop/strict settled predicate/80ms limit moved unchanged. Missing V4 engine/field identity now rejected as identity unavailable before privacy comparison; actual privacy mismatch still rejected. No input/delivery/IME state change. SemanticRED15/16→GREEN16/16. This is shared daemon bridge work, not Firefox-only. New ADR and ownerdoc: `docs/development/release-1.0.108-exact-tail-settlement-isolation.md`.

Source full run local `/home/ubu/.cache/lay/development/run-060t8122`, remote `/home/e/projects/lay-development-runner/run-Qhnw3W`, active execsession68233. Await final result, then prepared18gates driver `isolated-release108-gates-v1.py` using that exact request/delta. Launch scriptCACHE/release108-gates-v1-launch.sh. Installer `isolated-release108-install-v1.py` is prepared, checks3551 and prior107deploymentV3; not yet launched. Then exact artifacts install, unchanged nativeFirefox20/Chrome controls with honest existingFAILs, commit/pushCI/stable108/updater.

## Diagnostic evidence and remaining work

Three isolated active-word Firefox cases pass; this does not erase cumulative failure. Cumulative20 debug diagnostic13PASS5FAIL2BLOCKED. Journal shows unbound V4 readout mislabelled retention-policy change; this observation supports108diagnostic fix, not a proven latency cause. Temporary daemon debugdrop-in was removed/restored; current daemonPID2355888, same107bytes. Journal filter: use `_SYSTEMD_USER_UNIT=lay-daemon.service`; `journalctl --user -u` returned empty. Raw `isolated-release108-cumulative-diagnostic-v1/result.json`.

109 planned: move existing KDE terminal qualification intoKDEterminal module; isolate existing KDE-specific daemon focus functions, preserve API/testnames and transport/provider.110planned: move existing Chrome-only native presentation underkde/chrome and verify real GTK/Qt editors/ordinary terminal controls.111planned: automatic diff-scope guard and exact component test entrypoints, final release/update checks. Plans may follow stronger fresh evidence; no empty tags. Do not weaken tests or make literal per-word/runtime exceptions.

## Operational pointers

CACHE `/home/ubu/.cache/lay/development/browser-matrix-repair-20261027`; OLDCACHE `/home/ubu/.cache/lay/development/dependency-updates-20261027`; CROOT `/home/e/projects/lay-development-runner/kde-install-control-20261020`; remote workspace its parent/workspace. SSH e@192.168.3.94, key `/home/ubu/.ssh/mega-mini-admin`, BatchMode/IdentitiesOnly/StrictHostKeyChecking=yes. KDEDISPLAY:71, nativeRFBloopback5905. Never manager's GNOME:10.

Installed107enginePID2211235 SHA18a45eb56c73f10ea4b8402210f55336078f214a6a9d8cacb5d3042d32be1b84; daemon2355888; tray2211365. Original protectedIBusparent3168326, unitlay-release97-restored-system-ibus.service. No desktop/IBus parent/model provisioning restart.

Remote guard: source dedicated-resource.env, memoryhigh24G/max28G,20jobs,targetcache candidate-target-cache-20261023,DBUSunix:path=/run/user/1000/bus; scripts/lay-resource-guard.sh. AllCargo throughscripts/cargo-guard.sh. One heavy/input lease; never retry outside guard. Officialsourcecommand: python3 scripts/dev-check.py check --config OLDCACHE/development.json --compact (local transport only, actualtestsremote).

Publishing helpers inCACHE: verify-committed-source.py COMMIT REQUEST EXPECTED_SHA OUTPUT; wait-publish-verified-release.py VERSION COMMIT PREV_TAG waits all4GitHubSUCCESS then publishes/updater --check. Never useHEAD literal as commit identity. Git normalizes group-write; compare contents+executable bits, not raw Git-mode SHA to filesystem proofSHA.107publication watcher18257COMPLETED, do not poll again.

At end of all7, use existing half-volume soundhook `/home/ubu/.local/bin/codex-turn-done-sound` with agent-turn-complete/cwdJSON. Do not end now.

## Latest109 milestone (supersedes current108-running section above)

108commit058a247f0200ad6139fcb2a70abd186f05d1ebff pushed PRIVATEworkbranch+public/main. Allsource3551/18gates/10artifacts/0warnings, exact108installed engine2518551 SHA fddc0fded1632f467bab6a1d03e9edb49cce61c5b346a2425222947f0adeb89c, daemon2518664. Chrome9PASS, Firefox20=12P6F2B; all failures retained, no speedup claim. CI watcher/publication session13231 RUNNING; stable107 until actual publication.

109source implemented DIRTY in checkout: KDE terminal funcs →src/bin/lay_ibus_engine/kde/terminal.rs, fourKDEfocus funcs+oneflag →src/bin/lay_daemon/kde/focus_policy.rs, common facades/re-exports preserve consumers/testnames. No GNOME/provider/executor/input changes. Valid-schema/ID ADR and owningdoc written before code. Sourcefreeze3c5463769bafe6b0ce5adf040f07d5adeaede758787fec99fd37d0549d16de33,1294files;3551/3551sourcePASS,1113IME,oldtestinventoryexact. Localrun-hn_ql_x8/remoterun-uywcKX. Fullsource session21043 COMPLETED; do not poll. Mechanical audit6body-equivalent functions and singleflag savedCACHE/release109-mechanical-review.json.

10918releasegates executing session53154 withCROOT/isolated-release109-gates-v1.py and frozenrun-uywcKX. After pass: collect-release-phase.py109gates-v1 CROOT/isolated-release109-gates-v1 --metadata, then launch CACHE/release109-native-phase-v1-launch.sh (exactinstall109,9terminal suites,Chrome3,Firefox20). Installer correctly binds prior108deploymentV1. Native scripts fully uploaded (upload28150COMPLETED). Native actors extend only launchers/kinds; actualdefault Konsole profile replaces old fallback profile, GTK VTE additional kind, all input/assertions/cadence exact. Never call PTY text PASS a visualhintPASS without screenshot/snapshot inspection.

Disposable debugcache inventory/prune completed:307old executables/ownrlibs,6,322,727,334bytes;retained108artifacts10 unchanged;currenttarget~6.7GB within12GiB. RawCROOT/isolated-release109-cache-prune-v1/result.json. No source/models/config/installed deletion. Sourcefreeze/currentremote workspace109 differs from installed108 as intended.

Remaining110/111 detailed cacheplan: CACHE/remaining-isolation-implementation-plan.json.110moveChrome presentation intokde/chrome/presentation.rs preserving originalRustnamespace;111automaticdiffscopeguard and exact nonzero component test entrypoints, final freshnative matrices. All7stable releases105–111 still required, no end after109. Metadata-only publication receipts commit toprivatebranch and go public with nextsource, no redundantpublicCI. MainCI has no cancel concurrency; Security cancels samebranch, so nextpublicpush waits priorSecuritySUCCESS. wait-publish helper now waits previousstable tag as well as4SUCCESS, preserving stable order.

108publication FINAL:stablev1.0.108 2026-10-28T06:52:55Z at058a247f,4/4CI andupdaterchecksPASS. Publicationwatch13231COMPLETED;do notpoll. Localupdaterinstalled107→108 (automaticuserupdate; agent never manually applied/restarted). Currentremote108 pending109installaftergates. Fourstableversions105–108 nowpublished;109–111stillrequired.

## Current milestone —109 published,110 checking

109 stable tagv1.0.109 is published from96a98674,4/4CI and actual local/remote updater checks PASS. Remote installed109, local107 at check (auto-update remains enabled; no manual local apply).110 full source3553/3553 PASS,1115IME; localrun-dv5qe6cg/remoterun-hN7w0z. Gatev1 retained a controller metadata KeyError head_sha; fixed only its delta schema in v2, four exact-snapshot successful gates reused, no source/test change.110gatesv2 running, native installation/33terminal+25GTKQt+Chrome/Firefox controls pending.111scope/component tool implementation is an unapplied draft in cache; never confuse it with110testedsource. Upstream public pulls/issues checked2026-10-28, both empty. Continue through111; no stop here.
