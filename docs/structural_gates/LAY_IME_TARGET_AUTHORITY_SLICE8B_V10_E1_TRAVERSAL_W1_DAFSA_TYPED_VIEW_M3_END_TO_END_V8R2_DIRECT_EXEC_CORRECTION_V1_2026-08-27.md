# DAFSA Typed View M3 End-to-End V8R2 Direct-Exec Correction V1

Date: 2026-08-27

## Decision

V8R1 is terminal `BLOCKED_PROVENANCE`. It built and audited one exact test ELF,
then the first end-to-end subject failed before a complete scientific receipt.
V8R1 is never retried and its two consumed markers remain consumed.

The deterministic failure was in the controller lifecycle, not in typed-view
semantics. The mode-`0444` ELF was launched through `ld-linux`. Inside that
process, `std::env::current_exe()` resolved to `ld-linux`; the PSS producer then
spawned the loader with test-harness flags but no ELF argument. Both helpers
exited nonzero.

The sealed diagnosis receipt is:

```text
LAY_..._V8R1_PSS_HELPER_LIFECYCLE_DIAGNOSIS_2026-08-27.json
SHA-256 9b05af87d83c937dcc1e4eab0e398ab3d93ef49ac3e0bfb8089a58ba3d64bae0
```

V8R2 corrects only that lifecycle boundary. It reuses the exact V8R1 audited
ELF bytes, creates no new build, and executes an audited mode-`0555` copy
directly.

## Immutable Predecessors

```text
V8 scientific paper SHA        c5f1655ce4ab91f0...
V8 source SHA                  28f87a76fc199698...
V8R1 build audit SHA           d7d5e7110171e5c6...
V8R1 terminal audit SHA        04d0e17158a63a49...
V8R1 journal manifest SHA      c6c9d648bdbc02ee...

V8R1 ELF SHA                   0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea
V8R1 ELF size                  320,613,368 B
V8R1 Build ID                  c6ddac7181428a303cbc51be61dd3bb115677562
V8R1 ELF type                  ET_DYN
V8R1 source SHA                28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2
```

All V8R1 evidence and state are read-only predecessors. V8R2 may not edit,
rename, chmod, delete, recreate, or consume anything in the V8R1 namespace.

## V8R2 Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r2-direct-exec-20260827

transaction_id
  59694b7b9f0327d78896b5bc4797671f54478674186558e338e4a1b0d9ef7813

routes
  E2E

marker
  e2e.available
```

There is no `BUILD` route and no `build.available` marker.

## Bootstrap

Before creating the marker, a remote producer must:

1. verify the exact V8R1 build audit and terminal history;
2. verify the live sealed V8R1 ELF path, mode, size, SHA and Build ID;
3. create a fresh V8R2 parent and state tree;
4. copy the exact ELF bytes into `bootstrap-v1/m3-v8r2-test-elf`;
5. set only the copy to mode `0555`;
6. verify copy SHA and size are unchanged;
7. bind the exact immutable V8R1 input paths used by the subject;
8. publish immutable bootstrap evidence with zero Cargo/rustc/subject/perf/PMU;
9. stop for an independent bootstrap audit.

Changing only the copy's execute permission is an execution-envelope operation,
not a change to machine bytes. The build evidence remains mode `0444` and
immutable.

Only after independent bootstrap PASS may the producer create exactly one
canonical mode-`0400` `e2e.available` marker.

## Direct Subject Command

After an independent quiet-host admission, consume the marker atomically before
execution and run exactly:

```text
sudo -n -u e env <frozen V8 scientific environment>
taskset -c 0
<V8R2 bootstrap>/m3-v8r2-test-elf
--ignored
--exact
nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::m3_end_to_end_physical_proof
--nocapture
--test-threads=1
```

Forbidden executable prefixes include `ld-linux`, Cargo, rustc, perf, shell,
daemon, installer, and any V8R1 E2E path. `argv[0]` and `/proc/self/exe` must
both resolve to the exact V8R2 executable copy before the scientific parent
starts. The helper children inherit the same direct executable identity.

## Reused Scientific Gates

The scientific Rust bytes and all V8 gates remain unchanged:

```text
fixed proof                    382 cases, F/R/F/R, 1,528 requests
candidate/certificate parity  exact
capacity/unresolved            zero
scratch                        <= 512 KiB/query
typed materialization          once per generation, never per request
reload identity                exact
two-process PSS delta          <= 40 MiB
search p99                     <= 3,000 us
total-material p99             <= 5,000 us
CPU                            0, no mismatches
thermal throttle drift         none
```

V8R1 partial observations are not combined with V8R2. V8R2 must produce one
complete fresh scientific receipt or terminate blocked.

## Failure Dispatch

```text
predecessor / ELF / namespace / receipt / incomplete observation
  -> BLOCKED_PROVENANCE

semantic or certificate mismatch
  -> BLOCKED_SEMANTIC

capacity / unresolved / scratch
  -> BLOCKED_CAPACITY

generation publication or reader identity
  -> BLOCKED_RELOAD_IDENTITY

PSS or typed-owned-byte gate
  -> BLOCKED_RSS

p99 gate
  -> BLOCKED_LATENCY

CPU / thermal / quiet-host gate
  -> BLOCKED_ENVIRONMENT
```

Failure priority remains provenance, semantic, capacity, reload identity, RSS,
latency, environment. Any incomplete or contradictory observation dispatches to
`BLOCKED_PROVENANCE`.

## State Machine

```text
V8R1 BLOCKED_PROVENANCE (immutable)
        |
        v
V8R2 implementation preflight
        |
        v
controllers verified unrun
        |
        v
live admission: V8R2 absent, V8R1 exact, host quiet
        |
        v
bootstrap exact executable copy -> independent audit
        |
        v
one e2e.available marker
        |
        v
quiet-host audit
        |
        v
consume marker -> one direct subject -> independent terminal audit
        |
        +-- M3_END_TO_END_TEST_OWNER_PASS
        +-- exact terminal BLOCKED_* verdict
```

No failure grants a second marker, second subject, new build, source edit,
production activation, runtime mutation, daemon restart, or deployment.

## Claim Boundary

A V8R2 PASS establishes only the test-only generation-owner end-to-end contract
for the exact fixed evidence. It admits a separate production-authority decision
paper. It does not itself admit production source edits, runtime reload changes,
installation, daemon/IBus testing, or queue-inclusive product latency claims.

