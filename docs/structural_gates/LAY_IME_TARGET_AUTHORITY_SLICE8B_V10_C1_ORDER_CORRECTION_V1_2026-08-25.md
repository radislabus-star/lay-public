# Slice 8B V10 C1 Order Correction V1

Status: `PAPER_CORRECTION_PENDING_STRUCTURAL_REVIEW`

Date: 2026-08-25

## Decision

The direct steady-state latency decision C1 is moved before the complete B PMU
matrix. This correction changes research order only. It changes no production
bytes, installed Lay runtime, V10/V11 authority, quality denominator, latency
threshold or prior evidence.

```text
P0 executable provenance                         CLOSED
new-executable semantic parity                   REQUIRED
C1 direct steady-state latency                   NEXT AFTER PREFLIGHT
complete B PMU matrix                            HOLD UNTIL C1_FAIL
V12 fused-executor implementation                NOT_ADMITTED
runtime integration                              NOT_ADMITTED
```

The reason for the order change is decision value:

```text
complete B
  -> describes instructions, cycles and memory pressure
  -> does not prove pristine p99

C1
  -> invokes the exact V10 search function
  -> consumes its production-owned latency fields
  -> directly decides whether latency optimization is needed
```

## Preserved facts

The following evidence remains byte-identical and keeps its original scope:

- exact recovered V10 source identity `f6178f3882c1...`;
- production prefix `1..1148`, `39,047 B`, SHA-256 `ce9ea2d29060...`;
- prior diagnostic ELF `f7bcee37d5df...` and Build ID
  `9829fb05f34bd353877fb6d71f1f8523e084af55`;
- dirty aggregate B5/B6 observation;
- loaded semantic parity `382/382 PASS`;
- B1 `BLOCKED_ENVIRONMENT`;
- Clean V2 controller and preparation evidence;
- active V11 source `d9edfe7346b8...`;
- installed Lay `1.0.43`.

No prior `NOT_ADMITTED`, `BLOCKED_ENVIRONMENT`, `WATCH`, dirty measurement or
parity claim is reclassified.

## Clean V2 supersession

Clean V2 currently remains physically runnable because both paths are absent:

```text
/home/e/.local/share/lay/provenance/
  slice8b-v10-clean-speed-v2-20260825             ABSENT

/home/e/.local/state/lay/
  slice8b-v10-clean-speed-v2-20260825             ABSENT
```

The sealed Clean V2 controller rejects execution when its remote state path
exists. C1 therefore requires a separate fail-closed supersession transition:

```text
verified Clean V2 UNRUN state
  -> create exact remote state directory atomically
  -> state existence immediately blocks Clean V2
  -> publish SUPERSEDED_UNRUN.json and SHA256SUMS
  -> seal state tree read-only
  -> publish local immutable index
```

The required remote tombstone path is:

```text
/home/e/.local/state/lay/
slice8b-v10-clean-speed-v2-20260825/
└── SUPERSEDED_UNRUN.json
```

The transition must preserve the original controller, preflight, preparation
receipt and empty final output. It must state:

```text
subject executed                         false
measurement produced                     false
clean result published                   false
old route physically runnable            false
status                                   SUPERSEDED_UNRUN_BY_C1
rollback                                 retain tombstone
```

If the process fails after creating the state directory, Clean V2 remains
blocked. The failure state is `SUPERSESSION_PARTIAL_FAIL_CLOSED`; cleanup may
not remove the state directory and restore the old execution right. Recovery
may only complete or index the tombstone under a new preflight.

The supersession needs its own implementation preflight. Paper correction
alone grants no remote write authority.

## Corrected sequence

```text
order correction paper
  -> structural review
  -> Clean V2 supersession preflight
  -> fail-closed Clean V2 tombstone
  -> C1 implementation preflight
  -> one source-preserving C1 build
  -> seal executable identity
  -> separate C1-PARITY process
  -> clean-host S/T process matrix
  -> C1_PASS | C1_FAIL | BLOCKED_ENVIRONMENT | BLOCKED_PROVENANCE
```

The old B matrix is not deleted. Its next execution right is held until a
future `C1_FAIL` paper decision explicitly admits a targeted diagnosis.

## Claim boundary

This paper correction proves only that the route ordering and ownership are
explicit. It does not prove:

- successful Clean V2 supersession;
- C1 implementation correctness;
- clean-host availability;
- C1 parity or latency;
- full B completion;
- V12 necessity or admissibility;
- runtime integration readiness.

Runtime authority changed: `false`.

## Supersession implementation result

Clean V2 was superseded without executing its diagnostic subject or producing
a measurement. Preflight history remains append-only:

```text
preflight V1                    BLOCKED_BEFORE_CODE
  cause                         six invalid test-kind labels
preflight V2                    READY_TO_IMPLEMENT, superseded before code
  correction                    ready reports consumed; only run rejects
preflight V3                    READY_TO_IMPLEMENT
  baseline checks               16
  forbidden effects             21
  tests                          21
  blockers                        0
```

The V3 controller passed `21` local checks and all `6` preregistered fault
points without SSH. The live precondition check then found both Clean V2 paths
absent. Its first remote mutation created the exact state directory; it
published and sealed the tombstone without creating the old final result.

```text
status                          SUPERSEDED_UNRUN_BY_C1
subject executed               false
measurement produced           false
clean result published         false
old route physically runnable  false
rollback                       RETAIN_TOMBSTONE

remote state mode              0555
remote files                   2, both 0444
remote writable objects        0
remote SHA256SUMS              PASS
old Clean V2 final path        ABSENT
second supersession run        REJECTED, receipt unchanged
```

Identities:

```text
controller SHA                 9f6d6b2c6a6042d98940ec15cb361bae0424c89c4c7e9a8f8093c5e14f45c060
remote tombstone SHA           76c7bc00053e642fc83b089e2bcfd088d1788f99d5e38bbe7aa6ccdd55e9f35d
remote SHA256SUMS SHA          407ad04a066f916077a816f88ef03fc8dca1e8bbfbdb6061abb675134d2023bd
local receipt SHA              c6a82710d7bf4cafda33bcec21efe784605219c62d1a5fde252ff5c004351ffa
```

Evidence:

`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_SUPERSESSION_V1_2026-08-25.json`

The complete B matrix remains on hold. C1 build, parity and latency remain
unrun and not admitted until a separate C1 implementation preflight passes.
V12 and runtime integration remain `NOT_ADMITTED`; installed Lay stayed
`1.0.43`, active V11 stayed `d9edfe7346b8...`, and runtime authority changed:
`false`.
