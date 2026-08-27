# DAFSA Typed View M3 V8R3 Latency Failure Decision V1

Date: 2026-08-27

Status: `V8R3_LATENCY_FAILURE_CLOSED_DIAGNOSTIC_DECOMPOSITION_ONLY`

## Scope

This paper consumes the immutable V8R3 terminal evidence. It closes V8R3,
preserves its passing evidence, and decides the only admissible next research
question. It does not retry V8R3, change source, compile, activate production
authority, install, restart or deploy Lay.

Immutable predecessor identities:

```text
V8R3 paper SHA-256
  3d4fcffdb5ae5a9878ae7c35aec243d767963f516193d44ab59303b03a27b7fd
V8R3 effective structural route SHA-256
  fca7659fe1de100feee0ac3c10c7155cf14de1f0ea5c46c4c5451b0df7bed804
V8R3 structural receipt SHA-256
  68fc2f49f5fc7cb2097e5289acad4c9f876dc617244d7b5bd7c09b90f076aa16
V8R3 preflight manifest SHA-256
  299112c9db3db24ef56d8743328bc2fc80defc7860e543c948e16e8eff0ddf2c
V8R3 preflight receipt SHA-256
  ca3745efde0d2c9311b0f2d777c671ae53e4a3146d11c9ce0ef20897b84d4cb3
V8R3 implementation receipt SHA-256
  9ed8dbfbfc98d5f839ad1145c295831b5a8ba9e5ebbc419bcbe92b86d4a2f8d6
V8R3 subject receipt SHA-256
  65cd8a6f08d77c192ae0eb24fa3df106ee5030e7a8bbdfdf44d08429f7d9bfd5
V8R3 terminal receipt SHA-256
  2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc
V8R3 terminal verdict
  BLOCKED_LATENCY
```

## Measured Facts

V8R3 completed one frozen target-host execution over `382` cases, one warmup
round and four measured rounds. All non-latency gates passed.

```text
semantic mismatches                     0
capacity failures                       0
maximum query scratch                6,144 B
typed materializations                  2
per-request materializations            0
mixed-generation observations           0
two-process PSS delta                9,716 KiB
runtime authority changed            false
production authority admitted        false
```

The latency failure was:

```text
maximum round search p99                3,238 us  > 3,000 us
maximum round total-material p99        8,514 us  > 5,000 us

pooled search p99                       3,126 us
pooled owner-prepare p99                   60 us
pooled final-materialize p99            6,467 us
pooled total-material p99               8,380 us
```

`final_materialize_us` measures exactly the call from the V8R3 proof to
`materialize_live_productive_v1_field`. That function calls
`materialize_live_candidates`, which projects, classifies, gates and packages
each retained lattice surface.

## Interpretation

V8R3 proves that final candidate materialization dominates the measured
end-to-end p99 failure. It does not prove which operation inside final
materialization dominates.

The current implementation already contains opt-in diagnostic counters for:

```text
setup
projection
classification
proposal gate
evidence construction
```

Historical traces show that proposal-gate time can dominate this decomposition,
including multi-millisecond totals for small candidate sets. Those traces were
not produced by V8R3 and are hypothesis evidence only. No source-level cause,
specific admission predicate or optimization is selected by this paper.

## Decision

V8R3 is terminal and must not be retried. Its semantic, capacity, RSS and reload
identity evidence remains valid in its exact test-only scope. Its latency gate
failed, so production authority remains forbidden.

The only admitted successor is one new read-only scientific-diagnostic route on
the exact sealed V8R3 ELF and exact fixed input identities. Its sole question is:

> Which already-required stage inside final materialization explains its p99?

The diagnostic may set the existing `LAY_L2_FIELD_TRACE=1` environment variable
and collect the emitted stage totals. It may not rebuild, edit source, run perf
or PMU events, change the sidecar/package, modify runtime authority, or interpret
its instrumented latency as a replacement for V8R3 latency.

## Diagnostic Contract

The successor must use a fresh task ID, transaction ID, namespace and one-shot
marker. It must preserve the exact V8R3 test ELF and all V8R3 fixed input hashes.
The old V8R3 marker remains consumed.

The frozen route is:

```text
independent live admission
  -> bootstrap immutable V8R3 ELF and fixed inputs by identity
  -> independent bootstrap audit
  -> create one diagnostic marker
  -> consume marker before subject execution
  -> execute the exact V8R3 proof once with LAY_L2_FIELD_TRACE=1
  -> retain structured subject receipt and complete stderr
  -> parse exactly 1,910 ordered materialization rows
  -> immutable terminal audit
  -> STOP
```

The unchanged proof materializes candidates once for every request in one
`382`-case warmup round followed by the frozen four measured rounds. The exact
trace cardinality and ordering are therefore:

```text
rows    0..381      warmup / FORWARD
rows  382..763      measured round 1 / FORWARD
rows  764..1145     measured round 2 / REVERSED
rows 1146..1527     measured round 3 / FORWARD
rows 1528..1909     measured round 4 / REVERSED
total rows          1,910
```

Required observations from every trace row:

```text
deterministic phase / round / schedule / case ordinal
surface and emitted candidate counts
setup_us
projection_us
classify_us
gate_us
evidence_us
sum of traced stage time
```

The diagnostic must report aggregate and per-request stage distributions,
including p50, p99 and maximum, and reconcile the parsed row count against the
subject receipt. The existing trace line does not carry a source case identifier
or the outer per-request `final_materialize_us`; phase and case ordinal are
derived only from the sealed sequential loop and fixed schedule. The V8R3
receipt contains only aggregate outer distributions, so no per-request inner to
outer join is claimed. Because tracing adds timing calls and stderr output,
these numbers may attribute stage dominance but may not pass or fail the
original latency thresholds.

Failure dispatch:

```text
identity, row-count or parse mismatch     BLOCKED_PROVENANCE
semantic/candidate/certificate mismatch   BLOCKED_SEMANTIC
subject or trace capability failure       BLOCKED_CAPABILITY
otherwise complete decomposition          FINAL_MATERIALIZATION_DECOMPOSED
```

Any incomplete observation, unknown predicate or tied terminal cause at the
same priority is `BLOCKED_PROVENANCE`. No failure grants a retry.

## Claim Boundary

Allowed after `FINAL_MATERIALIZATION_DECOMPOSED`:

```text
select one proven dominant materialization stage for a separate paper decision
retain V8R3 typed-view semantic/RSS/reload evidence in its original scope
```

Forbidden:

```text
V8R3 retry
new W1 traversal experiment
production source or runtime edit
gate weakening or candidate-specific exception
SafetyGate or proposal-admission bypass
Cargo / rustc / perf / PMU
install / restart / deployment
production authority claim
```

## Next Tree

```text
V8R3 BLOCKED_LATENCY (terminal)
        -> fresh final-materialization diagnostic
             -> BLOCKED_*: STOP, no retry
             -> FINAL_MATERIALIZATION_DECOMPOSED
                    -> one separate mechanism decision
                    -> no implementation until its own structural gate and preflight
```
