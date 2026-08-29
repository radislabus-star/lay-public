# TD-002 Runtime Smoke Isolation Evidence

Date: 2026-08-29
Verdict: `TD_002_RUNTIME_SMOKE_ISOLATED`
Runtime authority changed: `false`

## Positive Isolation Proof

All four receipts use `run_id=td002-isolation-v6` and candidate binaries from
this checkout:

| Receipt | Cases | Result |
|---|---|---|
| `ISOLATION_V6_SINGLE_A` | `ru_p_enter` | PASS: `п` |
| `ISOLATION_V6_SINGLE_B` | `no_ne_ty_enter` | PASS: `но не ты` |
| `ISOLATION_V6_FORWARD` | A, B | PASS |
| `ISOLATION_V6_REVERSED` | B, A | PASS |

Forward and reversed batches have the same normalized case projection:

```text
verdict          RUNTIME_SMOKE_ORDER_ISOLATION_PASS
SHA-256          ffe69c98cef8049618dce771ce9b3613060a126996699465962dc3528ac20f4d
trace records    no_ne_ty_enter=99, ru_p_enter=17
manual toggles   0, 0
malformed        0, 0
read errors      none
desktop restored true
```

`ISOLATION_V6_VERDICT.json` independently binds both single receipts and both
actual execution orders. Raw trace SHA/counts and kind counts are retained.
The normalized projection excludes only `ibus_cursor`, whose count is retained
as an explicit volatile partition rather than silently discarded.

## Preserved Diagnostic Failures

No failed receipt was deleted or rewritten.

- `SINGLE_A` captured an early restoration failure while the service snapshot
  was transiently ambiguous. The bounded service restoration implementation was
  corrected before subsequent runs.
- `ORDER_REVERSED` and `ORDER_REVERSED_REPEAT` exposed a pre-existing
  manual-toggle race. In each run one direction reached
  `delete_commit_dispatched_waiting_exact_final` but remained at
  `visible_postcondition=pending`, so the original token was committed. The
  other direction passed. This is tracked as `TD-009`; no sleep or expected text
  was weakened in TD-002.
- `ISOLATION_*` and `ISOLATION_V2_*` show two historical autocorrect expected
  values that fail identically alone and in both orders. Their current outputs
  are order-independent; expectation ownership remains assigned to TD-007.
- `ISOLATION_V3_REVERSED` is another instance of the same manual-toggle pending
  postcondition, this time on a one-character conversion.
- `ISOLATION_V5_SINGLE_A` captured an incorrect first signal-window repair: its
  inherited POSIX mask forced the owned test daemon to `SIGKILL`. V6 uses a
  deferred Python handler, leaves child masks unchanged, and treats any daemon
  exit outside `0/-SIGTERM` as case failure.

Every diagnostic receipt after initial setup records
`desktop_restoration_verified=true`. The final live state was independently
checked as active/running/enabled `lay-daemon`, active `lay-ime-ru`, one global
`ibus-daemon`, and one installed managed Lay IBus engine.

## Binary Identity

Candidate and installed bytes were identical during the proof:

```text
lay-daemon       bab99d5258a442b9345d56feac39a209d600c607ec28fe92b6d0fdcdf5db116b
lay-ibus-engine  2232277072cd7176c6465875e31d2e4f3abf3977b51d3344d22a871a106336c0
lay-test-input   af9398773c5841e0aaaeca7c5f94336ae430f58296a02613b0259da75ff1ef0a
```

No build, install, global IBus restart, or runtime source change was performed
by the live proof.

## Scope

This verdict proves harness ownership, per-case isolation, persistent evidence,
order independence for the positive pair, bounded cleanup, and exact desktop
restoration. It does not promote stale semantic expectations or treat the
manual-toggle race as passing behavior.
