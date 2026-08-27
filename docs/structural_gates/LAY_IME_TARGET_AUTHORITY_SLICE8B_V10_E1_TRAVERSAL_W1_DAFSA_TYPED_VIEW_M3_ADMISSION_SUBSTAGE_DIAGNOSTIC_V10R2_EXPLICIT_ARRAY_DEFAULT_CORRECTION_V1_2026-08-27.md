# DAFSA Typed View M3 Admission-Substage V10R2 Explicit-Array Default Correction V1

Date: 2026-08-27

## Scope

This correction repairs one frozen-toolchain compatibility defect in the
test-only V10 admission observer. It changes neither production behavior nor
the scientific experiment. The only admitted Rust edit replaces the derived
`Default` implementation for `AdmissionTraceCounters` with an explicit
test-only implementation that initializes every scalar and fixed-size array to
zero.

## Immutable V10R1 History

V10R1 is terminal and remains immutable:

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r1-20260827

transaction_id
  7d067078f66e4c64724e5f6568304c0d6f6ab1c1ada92e09a98e05a19f0e1f17

controller implementation receipt
  d9b91168dbaa850ba5901c1b2f65f7119b3591be7918cd6a7126fad6f06451d4

execution journal SHA256SUMS
  ff91735880941eea9c382d10d953e2e2fee1f9cd2a13b0db366716a96a58be7f

build audit receipt
  73ca7d3c5c678d3056cb74048e005d6edde81b72386324fdcbd08ca2ac0e39d9

terminal verdict
  BLOCKED_BUILD
```

Its `build.available` marker was atomically consumed before its sole Cargo
invocation. Its unconsumed `trace.available` marker is retired with the
terminal namespace. V10R1 is never retried, its marker is never recreated, and
its remote or local evidence is never edited or deleted.

## Exact Defect

The frozen compiler returned `E0277` before creating a test ELF:

```text
[u64; 36]: Default is not satisfied
[u64; 43]: Default is not satisfied
```

The failing source is test-only:

```rust
#[cfg(test)]
#[derive(Default)]
struct AdmissionTraceCounters { ... }
```

The observer registries require arrays of 36 stages and 43 reasons. Their
lengths and contents are frozen. Only the construction spelling is incompatible
with the frozen toolchain.

## Admitted Source Repair

The sole Rust change is:

```text
remove #[derive(Default)] from AdmissionTraceCounters
add #[cfg(test)] impl Default for AdmissionTraceCounters
initialize all 3 stage arrays with [0; ADMISSION_TRACE_STAGE_COUNT]
initialize action_counts with [0; 4]
initialize reason_counts with [0; ADMISSION_TRACE_REASON_NAMES.len()]
initialize all 6 scalar counters with 0
```

Forbidden source changes include:

```text
stage/reason/action registry edits
timer boundary edits
candidate or admission logic edits
scientific subject or schedule edits
production authority or runtime edits
unsafe code or representation changes
Cargo.toml / Cargo.lock / cargo-guard edits
```

The explicit implementation must be byte-reviewed after formatting. A local
Cargo or rustc invocation remains forbidden; compatibility is tested only by
the new one-shot remote build after controller implementation and live
admission.

## Fresh Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r2-20260827

transaction_id
  fe5b4741c5d5711b48f356569f3be32a87142edd78979edf0aeb72a9616de7e6
```

V10R2 receives new controller identities, evidence roots, remote namespace,
and canonical BUILD/TRACE markers. No V10 or V10R1 marker or path is reused.

## Reused Scientific Contract

V10R2 preserves the effective V10 and V10R1 contract without change:

```text
routes                        BUILD, TRACE
markers                       build.available, trace.available
scientific subject            m3_end_to_end_physical_proof
CPU                           0
warmup                        382 requests
measured schedule             F/R/F/R, 1,528 requests
trace rows                    exactly 1,910 V9 + 1,910 V10
substage registry             36
action registry               4
reason registry               43
Cargo invocations             at most 1
subject executions            at most 1
perf / PMU                    forbidden
production authority          not admitted
runtime mutation              forbidden
```

The V10R1 UID-context repair also remains effective: root host observation
contains no toolchain query; controlled Cargo/rustc provenance uses the exact
future build environment; non-zero auditor responses retain bounded stdout and
stderr.

## State Machine

```text
V10R1_BLOCKED_BUILD (immutable)
        |
        v
V10R2 correction structural PASS
        |
        v
V10R2 implementation preflight READY_TO_IMPLEMENT
        |
        v
one explicit-Default source edit + fresh controller set
        |
        v
static self-check and fault injection
        |
        v
live admission (no namespace on failure)
        |
        v
bootstrap -> independent audit -> two fresh markers
        |
        v
consume BUILD -> ONE Cargo build -> independent build audit
        |
        v
quiet-host audit -> consume TRACE -> ONE scientific subject
        |
        v
independent terminal audit
        |
        +-- PASS -> ADMISSION_SUBSTAGES_DECOMPOSED
        +-- FAIL -> exact frozen BLOCKED_* verdict
```

Every failure is terminal for V10R2. It does not admit a retry, marker
recreation, second build, second subject, daemon restart, package install,
runtime edit, production promotion, optimization edit, or latency claim.

## Admission Verdict

This paper admits only the V10R2 structural route and implementation preflight.
Rust and controller edits may begin only after both pass. Remote execution is a
separate authority gate after implementation sealing and clean live prestate.
