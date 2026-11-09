# KDE fresh-focus query resource reuse — bounded experiment

Baseline v42 source eb785fcb6387d0dfea5fc34f6d4e91fefaa06368a77208d50f0478aeda0d0807; accepted123 remains restored. Both same frozen daemon/IME and installed model fixture are pinned by the owning letter-only document. The first failed physical transition remains later gesture acquisition while an earlier replay is unfinished; do not drop input or weaken isolation.

```text
existing daemon focus_guard / IME kde/desktop
  -> same kde/focus_observer.rs::read_with_deadline
     -> CLIENT try-lock; existing single Pending request
     -> existing KWin owner and connection lifetime
     -> uniquely owned short-lived loaded script (code only)
        foreground.js: NextRequest(resource nonce)
        -> fresh request nonce from the existing Pending
        -> current workspace.activeClient on KWin main thread
        -> same authenticated Observe reply
     -> same exact request/deadline/UUID/metadata and owner recheck
  -> unchanged field identity / reducer / decision / verifier / writer
```

Only runtime source scope: `src/bin/lay_daemon/kde/focus_observer.rs`, `focus_observer/foreground.js`, `focus_observer/tests.rs`. Both binaries reuse this KDE module; no GNOME/common caller changes. Scope/ADR/generated evidence follow the existing preflight.

Options: keep one-shot query5/10(safe but repeated parser/resource setup); reuse short-lived code with a new authenticated read8/10(selected experiment); new X11 observer3/10(wider identity/lifetime change); cached window value1/10(rejected, fails fresh-active proof). KWin5.24.7 getWindowInfo provides window metadata without active/PID, so it cannot replace the canonical proof. Its callDBus error path skips callbacks, requiring an independent one-shot resource-cleanup timer. The timer only frees the private script; reads retain their old deadlines. No polling, deferred correction or observation queue.

The resource instance and its one cancellable transport waiter are not a word owner. Old-instance pulls, wrong KWin sender, duplicate/expired replies, owner replacement and failed cleanup must fail closed. Every accepted observation is recomputed after a new nonce; no old RawWindow is retained. A pending cleanup obligation is discharged before replacement; unique plugin names bind late cleanup to its own resource.

Evidence: remote `letter-fragment-models-prefix3-firefox-v42-bus-profile-20261109/RPC_TIMING_ANALYSIS.json`(1284paired calls,147script cycles; counts include inter-gesture/decoder work and are not exclusive critical-path attribution). Same native actor before/after, no added wait between rapid gestures, no enlarged correction deadline, no waived failures. RED/GREEN, current-byte native improvement and complete release gates required before promotion.

## Experiment checkpoint — 2026-11-09 17:57 UTC

Meaningful source RED: two reads under old provider loaded code twice; new assertion requires one resource and distinct authenticated requests. After bounded code reuse, daemon private-bus controls21PASS/0FAIL/1explicit-live ignored. Same sender/resource, nonce, changing-window, duplicate/drop waiter, expiry, cleanup failure and service-owner replacement retain refusal behavior. Receipt: `/home/ubu/.cache/lay/development/letter-fragment-request-20261102/kde-focus-code-reuse-third-v43.log`.

Actual installed KWin5.24.7 read-only benchmark with a newly compiled test artifact:24fresh reads, median0.645ms, p951.331ms, max7.077ms including cold load. Independent connection verified the unique code resource disappears after provider drop within its existing bounded lifetime. No keyboard input, no metadata export, installed binaries/config unchanged. Receipt remote `/home/e/projects/lay-development-runner/kde-live-focus-v43-20261109/RESULT.json` and `cargo.log`. This is observation latency only, not end-to-end word-flip latency.

Not yet tested: same-candidate rapid prefix, full native regression matrix, strict lint/release gates. No installation/promotion; accepted123 remains restored.

## Rejected after actual native comparison — 2026-11-09

v43 frozen source778464bd814fced2f7345ad8df8214bdd86aabc336063c0ea4ca3db0ae5e7e2e; source42 dual-binary focus controlsPASS. Read-only idle-boundary probe alsoPASS (both20ms/250ms budgets16queries each). Nevertheless actual full-stack Firefox prefix3=v431PASS/2FAIL with canonical focus deadline errors, whereas immediate exact v42 comparison2PASS/1FAIL. Ordinary roundtrip166/181ms vs198/221ms is not acceptance when correctness regresses. Receipts remote `letter-fragment-models-prefix3-firefox-v43-20261109/RESULT.json`, `letter-fragment-models-prefix3-firefox-v42-comparison-20261109/RESULT.json`, `kde-live-focus-idle-v43-20261109/RESULT.json`.

Optimization REJECTED. Only its3runtime/test files restored byte-for-byte to independently hashed v42 baseline; no letter-fragment/Space/writer changes reverted. Private rejected source retained at C/rejected-kde-focus-reuse-v43-source. No release/permanent installation. The failed v43 GNOME rerun is retained and not certified; exact baseline is independently accepted v42 only.
