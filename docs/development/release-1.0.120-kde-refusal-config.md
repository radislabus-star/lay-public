# 1.0.120 — KDE refusal diagnostics read published flags

Preflight: 2026-10-31T06:24:46.088662+00:00. Baseline b06a294b / published
1.0.119. This is a scoped diagnostic configuration fix, not a Firefox latency
repair. Current runtime is exact 119 on remote KDE; local GNOME stays untouched.

## First loss and file map

`exact_ime_tail_replay.rs::log_replay_error` (and processed-but-unconfirmed
result reporting) → `kde/exact_replay_diagnostics.rs::record_refusal` → ordinary
`config_runtime.rs::active_layout_backend` → diagnostic `LayConfig::load` →
`src/config/load.rs` → `runtime_flags::publish_runtime_config`.

The backend resolver is the existing owner of periodic configuration reload.
Keep it and its order unchanged. The *second*, diagnostic-only load is the
unwanted publisher. It can observe pending disk bytes after the ordinary
cached backend check and publish process flags without the normal reload.
The source-only adjacent finding was explicitly left open in TD163 / 1.0.117.
Reuse that accepted fix: `runtime_debug_action_log()` reads published flags,
with the existing normal first-use initialization. No new cache or owner.

## Decision and consequences before production editing

9/10: use the existing published flag after ordinary backend resolution.
Extract the existing short-circuit predicate without changing its order so
its actual implementation can be tested with a fixed observed backend. Then
replace only the explicit diagnostic loader expression. 4/10: a new dedicated
configuration cache duplicates invalidation and reload ownership. 1/10:
changing the common loader/backend/authority to repair logging expands scope.

Allowed runtime file: `src/bin/lay_daemon/kde/exact_replay_diagnostics.rs` only.
Allowed process-isolation metadata: `scripts/test_lanes/discovery.py`; canonical
manifest / zero-known-failure ledger must be regenerated through existing tools.
Keep error privacy/length/JSON, KDE dispatch and call order unchanged.

Candidate/lattice retention, rank, false authority, packages and online reload:
no candidate/model/learning code or data changes. Existing normal reload remains
authoritative; pending flags must not leak from diagnostics. Latency/tails:
remove one extra file load on an already exceptional KDE path; no timeout,
wait, RPC, focus, lease, native receipt or replay change. Do not infer browser
speedup. CPU/RSS/allocations: one existing atomic flag read replaces parsing;
no queue/cache/background worker. Cache identity/invalidation: reuse current
runtime flags and loader, do not add state. Concurrency: flags remain the same
atomics; concurrent ordinary reload may publish normally, never use this test
as a claim of a coherent multi-field config transaction. Failure/rollback:
rollback only this leaf and its new test metadata; keep all failed receipts.
IME/daemon compatibility: payload and signatures unchanged; a diagnostic
refusal never grants mutation authority. Maintenance: same idiom as 117, no
parallel fallback or removal migration. Common input, correction, word owner,
verifier, layout controller, executors, rendering, GNOME and other client
integration source files are hash-frozen against this baseline.

## Proof and limits

The daemon's cfg(test) backend resolver intentionally loads config every call,
so calling it with a disk mutation would test that owner rather than the extra
diagnostic load. Test the extracted *actual production predicate* with fixed
observed KDE/GNOME backends; do not claim a full record_refusal entrypoint test.
Both published flag directions must survive opposite pending disk bytes until
explicit normal load; non-KDE must not even touch config. Keep the real refusal
JSON/privacy test. Process-isolate mutable published flags through the canonical
runner. Establish RED on the unchanged predicate expression, then GREEN with
one expression replacement. No test assertion/deadline weakening.

After focused proof: full canonical source/release/MSRV/lints, zero-warning
all-ten binary build, exact installed/loaded byte checks, unchanged native
Chrome/Firefox roundtrip controls, independent CI/security, both tag mirrors
and both actual installed updater entrypoints. Source/native/publication scopes
remain separate. Historical native 344 PASS / 15 FAIL / 81 BLOCKED stays intact.
Firefox rapid latency investigation remains OPEN in its original owning map;
paired timing capture found repeated fresh KWin reads plus a 106ms layout-list
outlier, not permission to remove freshness/verification. No 120 speedup claim.

## Focused result

RED: `CROOT/release120-refusal-red-v1/result.json`, one semantic assertion
fails with exit101 on the unchanged extracted loader expression, privacy/JSON
control passes. GREEN: `CROOT/release120-refusal-green-v1/result.json`, 2/2
actual test identities pass after the one expression replacement. Both are
separate hermetic processes; both flag directions and normal reload checked.
Protected live PIDs and configuration unchanged. Frozen integration files:
309. Strict returned file/preimage checks passed. 120 full gates pending.

Full-v1 FAILED: original td125_level_modifier_wire_order_preserves_owned_word_and_space_lease committed ghbdtn instead of привет after Level3/key release; all other3616source checks passed. Runtime/source frozen guard did not report product mutation; no source-pass claim. Next discriminating control 2026-10-31T06:44:09.794455+00:00: run the EXACT unchanged original test once in a fresh canonical hermetic process, from the same failed compiled source, to distinguish cross-test/background state from intrinsic failure. No assertion/deadline/setup changes, no green-until-retry loop.

Isolation control-v1 PASS is NON_EQUIVALENT_FEATURES: its reused focused compiler command selected default features; full canonical uses research-tools/all-targets. Do not infer shared-state root or change isolation from this result. Corrected control-v2 reuses exact canonical build flags before comparing the unchanged case; no native input or source changes. Recorded 2026-10-31T06:47:33.653158+00:00.

Full-v2 stopped before tests because canonical clean discovery correctly enforced its12GiB cache budget (12.007GiB actual, rounded13). No source/test failure from this variant. Empty incremental cleanup freed4KiB only; selective obsolete package-debug executable cleanup under the same guard freed5,480,192,136bytes, target now7,412,222,337bytes,408obsolete ELF executables removed. All54current canonical compiler artifact paths/inodes and loaded119runtime preserved; no release subtree/source/models/logs touched. Receipts CROOT/release120-disposable-debug-cache-cleanup-v1 (incomplete budget relief) and release120-obsolete-debug-executables-cleanup-v1/result.json (verified). Full-v3 continues with unchanged default12GiB canonical budget and original assertions/deadlines.

Full-v3 first pass3617/3617 completed; final canonical repeat found a different pre-existing positive Disable fixture race. New owning decision/analysis ci120-disable-receipt-order.md records actual P2P receive-before-consumer ordering, the already accepted adjacent helper pattern, controlled-order RED and6/6original-case GREEN. Only cfg(test)actual_disable helper changes, all negative tests and308other integration source files frozen; the rest of residuals.rs is byte-frozen. Old119source guards are preserved by exact normalization of this explicitly approved helper to the immutable old body, never by rewriting119decision. Fresh full-v4 mandatory.

## Final source and installed controls — CI/publication pending

Full-v4 PASS:3617Rust/34targets/494isolates,canonicalexit0,430PythonPASS/10existingSKIPof440,all17gates including two Rust1.88profiles. All10compiler-artifact release executables built with0warnings. Source1328/SHA565282efa1867e90966cbd1a9882301a7218700458aaf05c436bc0562be1038a;1332tested source+metadata inventory. Strict imported73files with preimage/hash checks; newer local handoff preserved only because remote copy unchanged.

Remote candidate installed once. Final version assertion in private install script mistakenly retained119; raw FAIL preserved. Separate verification (Python3.10-compatible after correcting an unavailable tomllib import before any mutation) confirmed120/all10installedbytehashes and real loaded daemon/IME; no repeated install/restart. CROOT/release120-refusal-deployment-verified-v1 and release120-final-loaded-identity-v1. Current daemon535954 SHA2a8ff8c0...,IME536040 SHAb51116c3...,tray536089,globalIBus3168326unchanged,config5d02...exact,LayRU/layout0restored.

Frozen r19 native after-Space roundtrip controls2/2PASS/4flips. Chrome84.406/79.622ms;Firefox185.095/140.536ms(n2each,unpaired,not speedup or rapid-five acceptance). No localGNOME install/restart,learning cleanup,model replacement,globalIBus restart or delivery/rendering change. Historical344PASS15FAIL81BLOCKEDof440 NOT rescored. Actual updater entrypoints and4CI/tag publication still pending.

PUBLISHED2026-10-31T08:47:13Z,v1.0.120/a9b9bb9fab96f6b6d4a8fa11d5dceb6ccead065b. All4CI/SecurityPASS;annotated050a7443d52f2678a14a4d42e604649fd9226c01 identicalorigin/public,peelsa9b9. Actualinstalled updater --checkPASSboth:local119sees120available,remote120current;bothresolvedinstalled update.sh SHAaebfb65e.. andstablelatestAPI verified. No localinstall/restart. Queue7of10published;continue121–123.
