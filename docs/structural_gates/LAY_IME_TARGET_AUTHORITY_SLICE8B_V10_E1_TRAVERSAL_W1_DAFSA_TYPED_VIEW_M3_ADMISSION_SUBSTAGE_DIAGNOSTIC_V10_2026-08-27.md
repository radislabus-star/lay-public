# DAFSA Typed View M3 Admission Substage Diagnostic V10

Date: 2026-08-27

Status: `READY_FOR_STRUCTURAL_REVIEW`

## Question

Which existing predicate or post-admission authority operation accounts for the
aggregate candidate-admission tail localized by V9?

V10 is a test-only diagnostic decomposition. It is not an optimization trial,
latency rerun, production source promotion or authority change.

## Namespace

```text
task_id
  slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10-20260827
transaction_id
  8058be994305226e9af3fbdee2e6b29bd9111ffbf8203ef20db9feeb1ca56a22

routes
  BUILD
  TRACE

markers
  build.available -> build.consumed-before-exec
  trace.available -> trace.consumed-before-exec
```

The fresh remote evidence and state parents must be absent before bootstrap.
`BUILD` and `TRACE` are each one-shot. A consumed marker is never recreated,
even after controller, transport, build, subject, parser or audit failure.

## Immutable Predecessors

```text
V8R3 terminal receipt SHA-256
  2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc
V8R3 terminal verdict
  BLOCKED_LATENCY
V9 authoritative terminal receipt SHA-256
  7105b503ce7a0079e441fe736c3e40717ab4f77bd7289c2d67536abb8507f6a8
V9 authoritative verdict
  FINAL_MATERIALIZATION_DECOMPOSED
V9 mechanism decision SHA-256
  3c2e2792d7e6f62abe016c89c132a8df5c171bb9d3c529b1419c39013789f8fe
V9 mechanism structural receipt SHA-256
  b83eb3096ebc676d8373a76960c61db57c081ca869f2d0779df080c87da95602
V9 mechanism status
  CANDIDATE_ADMISSION_AGGREGATE_DOMINANT_SUBMECHANISM_UNKNOWN
```

The V8R3 and V9 namespaces, markers, receipts, journals, logs and scientific
evidence remain immutable. V10 does not execute either old ELF or reuse an old
marker.

## Baseline Source Closure

Before V10 instrumentation, the three directly relevant source identities are:

```text
src/nanda_wave/l2_field/productive_v1/live.rs
  70,181 B
  87180990b6883641483a46886074e5350f35e351454d734f0c3c9da723d758bd
src/typing_transition/decision.rs
  ad3c6d450c01811844a49e9c714d0eb9ff80f7de7d2f03a2e8b3e290deda3691
src/typing_transition/proposal_admission.rs
  dd4a37a8c0430c9ff145f9ae9cbbbc735164ece833a143a19af644ac7ad835ca
```

The implementation may edit only `live.rs` and `proposal_admission.rs` for the
test-only observer. `decision.rs`, the V8 test owner, all candidate producers,
ranking, SafetyGate, edit plans, package code, runtime bridge, daemon and IBus
sources remain byte-identical. The complete build source closure receives a
canonical manifest before bootstrap and is verified again immediately before
marker consumption.

## Observer Design

The selected design instruments the existing code path exactly once. It does
not copy or reimplement `candidate_admission`.

```text
materialize_live_candidates
  -> begin one request-local trace session
  -> for each retained surface
       -> time the existing admit_candidate_proposal call
            -> time each existing short-circuit predicate in place
       -> time the existing live-authority override
       -> record final action and reason
  -> finish fixed-array trace session
  -> emit one deterministic aggregate line for the request
```

All observer machinery in `proposal_admission.rs` is compiled only under
`#[cfg(test)]`. Production builds expand each predicate wrapper directly to the
original expression and contain no clock read, thread-local state, registry,
counter, environment lookup or trace output from V10. V10 uses fixed stage
indices and fixed-size integer arrays; it adds no per-candidate heap or map.

The observer records nanoseconds because many individual predicates are below
one microsecond. Every stage has:

```text
calls
hits
elapsed_ns
```

`hit` means that the predicate selected its short-circuit branch. Value- and
option-producing stages define hit as the exact branch condition already used
by the decision chain. The observer also records:

```text
admission calls / elapsed_ns
post-admission override calls / hits / elapsed_ns
final CandidateGateAction counts
final reason counts
```

The fixed leaf registry is:

```text
unchanged
explain_candidate
replacement_glues_separate_words
boundary_glues_short_function_tail
boundary_eats_known_current_word
boundary_changes_non_whitespace_surface
multiword_last_vowel_completion
adjacent_transposition_boundary_competition
boundary_splits_known_word
boundary_splits_weak_tail
reflexive_suffix_requires_grammar
known_current_surface_drift
verify_action_operator
surface_changes_left_context
l2_surface_stem_truncation
structural_over_compress
structural_function_prefix_drop
structural_phrase_part_growth
structural_short_initial_growth
structural_short_case_vowel
structural_soft_sign_vowel
structural_short_internal_consonant
structural_short_same_length_multi_edit
structural_same_tail_consonant
structural_infinitive_overreach
structural_protected_context_authority
structural_known_word_different_known
structural_short_layout_context
structural_short_cyrillic_ascii
structural_short_nanda_shrink
structural_short_nanda_internal_vowel
structural_nanda_unknown_word
unproven_stable_surface_shape
semantic_surface_authority
completion_only
final_class_dispatch
```

The post-admission live-authority override is reported separately and is not a
leaf inside `candidate_admission`.

## Trace And Analysis Contract

V10 executes the existing ignored V8 physical proof once on CPU 0, under UID
`e`, with exactly one libtest thread. The environment enables both:

```text
LAY_L2_FIELD_TRACE=1
LAY_PROPOSAL_ADMISSION_TRACE=1
```

The subject retains the same six fixed input identities, `382` cases, one
forward warmup and four measured rounds in order:

```text
FORWARD, REVERSED, FORWARD, REVERSED
```

Exactly `1,910` V9 aggregate lines and `1,910` V10 substage lines are required.
Each V10 line binds its surface/emitted/admission cardinality and fixed registry
schema. The auditor assigns phase, round, schedule and case ordinal only from
the sealed sequential loop. It does not invent a source case identifier or join
to the immutable V8R3 per-request latency.

For every fixed case, the four measured rounds must reproduce identical:

```text
surface and emitted counts
admission call count
final action histogram
final reason histogram
predicate call and hit counts
```

The existing subject receipt must retain zero candidate, certificate,
structured-certificate, schedule, completeness, lattice, emitted-surface and
gate mismatches. Any mismatch is `BLOCKED_SEMANTIC`.

The auditor reports leaf call/hit/time distributions and totals for:

```text
all 1,528 measured requests
the preregistered V9 tail cohort:
  case ordinals 375, 371, 223 and 366 in all four measured rounds
the top 16 V10 requests by admission elapsed_ns
```

No threshold is used to manufacture a winning predicate. The positive V10
verdict means only that the complete substage decomposition is valid. A
separate mechanism decision may later select one optimization candidate or
conclude that cost is distributed.

Nested total and leaf time are not summed as independent work. The auditor
publishes:

```text
admission total
sum of non-overlapping leaf elapsed_ns
unmeasured residual including control and observer overhead
post-admission override total
```

Small leaf values may be timer-overhead limited and cannot support an
optimization claim by themselves. Instrumented V10 wall time cannot pass,
fail, replace or reinterpret V8R3 latency.

## Build And Execution Contract

V10 has one target-host build in a fresh isolated workspace. The exact build
environment remains:

```text
CARGO_BUILD_JOBS=20
CARGO_INCREMENTAL=0
CARGO_NET_OFFLINE=true
CARGO_PROFILE_RELEASE_DEBUG=2
CARGO_PROFILE_RELEASE_STRIP=none
RUSTFLAGS=""
CARGO_TARGET_DIR=<fresh V10 workspace>/target
```

The only Cargo argv is:

```text
scripts/cargo-guard.sh
test
--offline
--locked
--release
--lib
--no-run
m3_v8
```

The controller writes and fsyncs prebuild provenance, atomically renames
`build.available` to `build.consumed-before-exec`, then invokes Cargo once.
Build failure retains the consumed marker, source closure and complete log and
cannot be retried. A successful candidate ELF is sealed and independently
audited for SHA, Build ID, ET_DYN, executable PT_LOAD, symbols and DWARF before
`TRACE` can run.

`trace.available` is consumed before the sole direct ELF execution. No perf,
PMU, attach, signal shutdown, daemon, IBus, install, restart, live package
replacement or generated traffic is reachable.

## Consequence Analysis

Candidate and lattice retention: the observer wraps expressions in place and
must reproduce the exact existing candidate, certificate, lattice and gate
denominators. It cannot filter, reorder, cap or add a candidate.

Ranking and false authority: V10 ends at the same `CandidateGateDecision`; it
does not evaluate or modify ranking, final mutation authority, SafetyGate or
verifier behavior. Action and reason closure is evidence, not a new owner.

Latency and tail behavior: clock reads and synchronous trace output perturb the
test ELF. V10 measures only relative diagnostic substage time and preserves
V8R3 `BLOCKED_LATENCY`. No end-to-end p99 claim follows.

CPU, RSS and allocation: fixed thread-local arrays and counters are test-only.
Trace formatting happens after the measured admission loop. V10 does not amend
the V8R3 RSS proof and grants no production memory budget.

Cache, package and reload identity: the six fixed inputs and typed generation
owner remain exact. V10 creates no result cache, package generation, sidecar,
reload path or second source of truth.

Learning and feedback: the isolated ignored test emits no learning event,
feedback mutation, daemon request or user-visible correction.

Concurrency and stale results: the diagnostic subject is pinned to CPU 0 with
one libtest thread. Existing V8 generation/reload checks remain semantic
predecessor evidence; V10 does not generalize its timings to concurrent
production traffic.

Failure and rollback: build and trace markers are consumed atomically before
their irreversible action. Every failure retains evidence and forbids retry.
No installed or runtime state needs rollback because none may change.

Compatibility and maintenance: instrumentation is isolated behind `cfg(test)`
and one environment variable. It may be removed after the submechanism
decision without changing runtime interfaces or data formats.

## Rejected Designs

1. A duplicated profiled admission function is rejected because it creates a
   second decision implementation that can drift from authority semantics.
2. Perf sampling of the old ELF is rejected for this question because the old
   route has no predicate boundaries and would reopen measured-region and
   inlining attribution ambiguity.
3. Per-candidate stderr is rejected because it multiplies synchronous I/O inside
   the loop. V10 emits one fixed aggregate line after each request.
4. Hoisting, caching or deleting suspected repeated work is deferred. V9 did
   not identify which repeated work owns the tail.

## Failure Dispatch

Priority is fixed:

```text
0 provenance
1 build
2 semantic
3 capability
4 complete decomposition
```

```text
identity, source, marker, registry, row, parse, order or audit drift
  -> BLOCKED_PROVENANCE
Cargo, rustc, ELF or debug-material failure
  -> BLOCKED_BUILD
candidate, certificate, action, reason, call/hit or existing gate mismatch
  -> BLOCKED_SEMANTIC
subject launch, trace emission or complete observation failure
  -> BLOCKED_CAPABILITY
all required observations complete
  -> ADMISSION_SUBSTAGES_DECOMPOSED
```

Unknown predicates, incomplete observations, multiple causes at one priority
or dispatch-schema mismatch are `BLOCKED_PROVENANCE`.

## State Machine

```text
paper + structural PASS
  -> implementation preflight
  -> test-only observer and controllers
  -> static and fault self-checks
  -> independent live admission
  -> bootstrap source and input closure
  -> independent bootstrap audit
  -> create BUILD and TRACE markers
  -> consume BUILD
  -> one Cargo build
  -> independent ELF audit
  -> consume TRACE
  -> one direct diagnostic subject
  -> immutable terminal audit
  -> ADMISSION_SUBSTAGES_DECOMPOSED or BLOCKED_*
  -> STOP before optimization
```

## Claim Boundary

```text
runtime authority changed              false
production authority admitted          false
installed Lay changed                  false
Cargo / rustc                           at most 1 / build-owned only
subject executions                     at most 1 after marker consumption
perf record / perf stat / PMU           0 / 0 / 0
daemon / IBus / install / restart       0 / 0 / 0 / 0
```

The only positive verdict is:

```text
ADMISSION_SUBSTAGES_DECOMPOSED
```

It admits only a separate paper decision about the measured substage evidence.
It does not admit an optimization edit or production promotion.

## Structural Closure

The global worksheet remains an explicit size-only `WATCH` because its ten
independent route groups exceed the checker limit of eight. It has no conflict,
evidence gap, foreign pull, weak triad or repair task. Hierarchical linked-group
verification produced ten digest-valid route receipts, all `PASS`, with no
missing, duplicate, unexpected or truncated route. The typed
`all_routes_pass` claim covers exactly those ten keys and independently passes.

```text
global receipt SHA-256
  081c61ced9a4ac6f705f80eb4794ce9c8c0e744beb77a89fc08b9395c0fc92a8
global verdict                       WATCH / size-only
split packet SHA-256
  4e2b5086b5ef45ea2162d7c4dc5f805489ca17e97d95dadec3fa0fe8891442ba
split receipt SHA-256
  19fde401142400eea62e40f543de5271abd4b89bb36d61cc9ff26857df63bf25
local route PASS                     10 / 10
receipt aggregation                  PASS / complete / all_routes_pass
claim boundary                       PASS / coverage complete
truncated branches                   0
aggregate status
  STRUCTURALLY_ACCEPTED_WITH_SPLIT
```

This is structural coherence only. Code remains blocked until the separate
implementation preflight returns `READY_TO_IMPLEMENT` with
`safe_to_implement=true`.
