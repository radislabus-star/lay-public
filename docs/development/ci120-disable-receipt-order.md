# 120 gate — Disable positive-fixture receipt order

Preflight 2026-10-31T07:33:09.712761+00:00. Full-v3 first pass3617/3617,
second canonical pass3616/3617; original missing/repeated Disable fixture left
atomic.active=true at residuals.rs5036. This failure is already process-isolated,
so the earlier modifier test isolation is not a sufficient explanation.

The actual_disable helper uses future::zip(engine.disable, observer.process_next).
It permits a consumer-before-receipt order. Runtime observe_lifecycle correctly
refuses an unreceived callback before local Disable cleanup. Adjacent helpers
actual_focus_out and legacy_key_at, and owned_preedit_reset_source, already
observe the real P2P stamp before calling the handler. Reuse that positive
fixture precondition. Dedicated negative before-receipt/late-ingress cases
remain byte-identical and are included in focused checks. No runtime fix or
physical Disable acceptance is claimed from this fixture correction.

Chosen9/10: order receipt then real handler in the existing cfg(test) helper;
no synthetic proof, extra timer/poll, callback budget or output-owner change.
3/10: fixture cleanup retries/cancellation; rejected previous lifetime A/B
addressed atomic-unsubmitted input, a different path (2/32 vs5/32); do not repeat.
1/10: extend the production1ms rendezvous or suppress the final assertion;
rejected. Keep every test body/expected output and the1second outer fixture
bound unchanged. Two bounded existing operations remain inside that same outer
bound. The three helper callers and independent negative controls are selected.

Before final change, controlled RED forces callback-before-observer in the
same helper and runs the original failing case; record this as a controlled
ordering violation, not the unchanged old zip implementation. GREEN reuses
existing observer-before-consumer order and all six selected original tests.
Only the helper span changes; all other bytes in residuals.rs, all test bodies,
production handlers and308other integration files remain hash-frozen. Legacy119
frozen-source checks must normalize ONLY this explicitly approved helper back
to its immutable original body; never rewrite an old ADR or drop a guard.

Consequences: no candidate/rank/package/delta/learning/cache/runtime authority
or runtime CPU/RSS/latency changes; this file is cfg(test). Callback races still
have explicit negative coverage. Failure/rollback is one helper span; raw
full-v1/v2/v3 and controlled RED remain. Normal config and native routes stay
unchanged. Require fresh complete canonical0/gates/build/native/CI/updaters.

The three test-manifest/zero-ledger/first-pass observation files were synced
from full-v3 with strict preimage/archive hashes in a separate metadata-only
audit. They record a real complete first pass and expected zero failures;
full-v3 final failure is retained and NO source acceptance was imported.
Fresh complete all-green execution is still required after this change.

Controlled RED reproduced the same !engine.atomic.active assertion,exit101,in0.043s. GREEN passes all6original selected tests (0.044–0.424s each). All3callers and independent late-ingress/bad-sender/FocusOut controls checked. Full-v4 remains required; no physical-runtime fix claim. Production rendezvous constant verified unchanged at context_admission/rendezvous.rs14:Duration::from_micros(1_000).
