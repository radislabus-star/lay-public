# W1 Fused-Minimum M2R1 Build-Closure Repair V1

Date: 2026-08-27

## Observed Terminal State

The original M2 namespace is terminal and immutable:

```text
task_id              slice8b-v10-e1-traversal-w1-fused-minimum-m2-v1-20260826
terminal verdict     BLOCKED_BUILD
terminal receipt     21c2098f7c457c4939d6aef2b36e13ed895f7e8f77e040d8b99950f5db2cf85c
Cargo invocations    1
build marker         consumed before Cargo
subject executions   0
perf stat / record   0 / 0
parity                not executed
physical routes       not executed
retry permitted       false
runtime changed       false
```

The sealed Cargo log identifies one exact build-closure defect:

```text
error[E0277]
V13TypedPeak: serde::Serialize is not satisfied
source: m2_result_signature()
cause: serde_json::json! attempted to serialize Vec<V13TypedPeak>
```

This terminal result does not test the fused-minimum mechanism. It establishes
neither mechanism acceptance nor rejection.

## Repair Scope

M2R1 creates a new namespace. It does not modify, resume, or recreate any M2
marker or artifact.

```text
task_id
  slice8b-v10-e1-traversal-w1-fused-minimum-m2r1-v1-20260827

transaction_id
  2dae728a39aecd422995828674d12e311ab6362ebab4013c4f2520b3f6933c5f
```

The only semantic source change is inside test-only parity evidence encoding:

```text
old:
  peaks = &observed.result.peaks

new:
  peaks = observed.result.peaks.iter().map(|peak| {
      { form_ref, certificate_keys }
  })
```

The repair must not derive `Serialize` for `V13TypedPeak`, modify its layout,
change the production prefix, change any candidate transition, or alter the
measured physical route.

Pinned repaired source identities:

```text
V10 source SHA              f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c
D1 fragment prefix SHA      bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665
production prefix SHA       ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26
M2R1 fragment size          156122
M2R1 fragment SHA           a6ea388d5d76f8223511fd4822cff2df9fd0c3394fc200f4fc52db956522ce5b
assembled source size       247640
assembled source SHA        b2b44ed1b42e79dcdb9b7dcfb2753eec547fa7d1e26b5fb63772a706f786ccd5
```

## Pre-Marker Build Closure

Before M2R1 marker creation, the exact repaired assembly must pass a disposable
compile-only proof against the sealed surviving source closure and exact
`Cargo.toml`, `Cargo.lock`, and `cargo-guard.sh`. That proof is not scientific
evidence, does not create or consume M2R1 markers, and its ELF is not admitted
for parity or measurement.

The completed local proof used the exact repaired assembly and returned Cargo
exit `0`. The disposable ELF must not be promoted into M2R1.

## Preserved Scientific Contract

M2R1 inherits the effective M2 paper and corrections without scientific
changes:

```text
route order
  B0-ITERATOR
  G0-M1-GUARDED
  I0-INTERLEAVED
  I1-INTERLEAVED
  G1-M1-GUARDED
  B1-ITERATOR

candidate mechanisms
  ITERATOR_BASELINE
  M1_GUARDED_CHAIN
  INTERLEAVED_RUNNING_MIN

physical context
  single pinned P-core worker
  exact frozen schedule and inputs
  perf stat only
  instructions,cycles,branches,branch-misses,task-clock
```

Build, parity, six physical routes, terminal dispatch, perturbation gates,
semantic gates, route ordering, one-shot marker semantics, failure priorities,
and final mechanism verdicts remain byte-for-byte or value-for-value equivalent
to effective M2 V3 except for namespace and source identities required by this
repair.

No intermediate result may be interpreted before the terminal audit.

## One-Shot State

M2R1 owns fresh markers only:

```text
BUILD
PARITY
B0-ITERATOR
G0-M1-GUARDED
I0-INTERLEAVED
I1-INTERLEAVED
G1-M1-GUARDED
B1-ITERATOR
```

Every marker is consumed by atomic rename before its Cargo or subject action.
Any failure leaves it consumed and forbids retry. The old M2 tree remains
untouched and is a required predecessor.

## Decision Boundary

Positive terminal verdicts remain:

```text
W1_FUSED_MINIMUM_MECHANISM_PASS
W1_FUSED_MINIMUM_MECHANISM_REJECTED
```

Failure verdicts remain the effective M2 taxonomy. A build failure remains
`BLOCKED_BUILD`; semantic, provenance, capability, thermal, measurement, parity,
or perturbation failures retain their distinct classes.

M2R1 cannot admit production edits, installation, restart, deployment, or a
change to runtime authority. It only answers whether the fused-minimum mechanism
removes enough W1 machine cost under the frozen experiment.

## Effective Verdict

```text
M2R1_BUILD_CLOSURE_REPAIR_READY_FOR_IMPLEMENTATION_PREFLIGHT
```

