# DAFSA Typed View M3 V9 Mechanism Decision V1

Date: 2026-08-27

Status: `CANDIDATE_ADMISSION_AGGREGATE_DOMINANT_SUBMECHANISM_UNKNOWN`

## Scope

This paper consumes the authoritative offline-corrected V9 terminal evidence.
It identifies the exact source span represented by the measured dominant
`gate_us` stage and decides the only admissible successor. It does not amend the
immutable V8R3 latency failure, choose an admission predicate to optimize,
change source, compile, execute a subject, or grant production authority.

Immutable evidence:

```text
V9 paper SHA-256
  98000de5d6a502d4bf1b2005deca476bfdd12539f02528a4b450f240f3d9ed27
V9 correction SHA-256
  0af397c9969077cb6348cbba84f27837f19554d2cb06fda752fb7682cb249204
V9 correction structural receipt SHA-256
  a8d6e20966421dd4caa256d5491fd4d5f986484ba4f06b741cfd0cd6af01a299
V9 offline implementation preflight receipt SHA-256
  496b779d5df4b9fd398472304632749c752e64e85d6e38b26539e1ad8f3a7860
V9 authoritative terminal receipt SHA-256
  7105b503ce7a0079e441fe736c3e40717ab4f77bd7289c2d67536abb8507f6a8
V9 authoritative SHA256SUMS SHA-256
  e6b9c7ef059e49686ce4f0eaca12c0c64e66fce306e79d3c63bf93e8559b7a39
V9 trace rows SHA-256
  4d97d55d8b3f32aeca843cf1d44f4018dcfabd1686f6be9e3fd64a60ababcd2b
V9 trace summary SHA-256
  dc037e8899f85b5a7c9f01aeabf57f8b79c5cd18e8f306b5ddb552cd2a5ca027
V9 authoritative verdict
  FINAL_MATERIALIZATION_DECOMPOSED
```

The historical V9 V1 `BLOCKED_PROVENANCE` receipt remains immutable. Its only
failure was the documented environment-parser defect. The V2 offline audit
performed no subject, network, marker, Cargo, rustc, perf or PMU action and
verified the retained V1 evidence byte-for-byte before publishing the corrected
interpretation.

## Measured Facts

V9 parsed exactly `1,910` trace rows. The `382` warmup rows were excluded from
scientific distributions; `1,528` measured rows remained.

```text
stage                    p50 us      p99 us      max us       sum us
setup                          0           1           2           31
projection                     0           0           3            9
classification                 5         153         182       18,370
candidate gate               425       6,439      22,652    1,178,683
evidence                       0           0           0            0
traced total                 429       6,477      22,826    1,197,093
```

The frozen top-one-percent tail contains `16` rows. `gate_us` is the largest
stage in all `16`, contributes `226,939 / 228,632 us`, and therefore accounts
for `99.2595087301865%` of the traced tail aggregate.

The same case ordinals recur across all four measured rounds:

```text
case ordinal   surfaces/emitted   repeated gate_us range
375            53 / 53            22,280..22,652 us
371            30 / 30            15,434..15,542 us
223            28 / 28            11,968..12,703 us
366            12 / 12             6,439.. 6,475 us
```

The measured denominator contains `4,984` surface groups and `1,178,683 us` of
aggregate gate time, an aggregate ratio of `236.49337881219904 us` per surface.
That ratio is descriptive, not an independent per-candidate latency estimator.
Candidate count is not sufficient to predict cost: other fixed cases with
`48` or `51` surfaces repeatedly complete `gate_us` in about `2.1..2.3 ms`.
The input/candidate-specific admission path therefore matters in addition to
call cardinality.

## Exact Timed Span

The current source identities are:

```text
src/nanda_wave/l2_field/productive_v1/live.rs
  87180990b6883641483a46886074e5350f35e351454d734f0c3c9da723d758bd
src/typing_transition/decision.rs
  ad3c6d450c01811844a49e9c714d0eb9ff80f7de7d2f03a2e8b3e290deda3691
src/typing_transition/proposal_admission.rs
  dd4a37a8c0430c9ff145f9ae9cbbbc735164ece833a143a19af644ac7ad835ca
```

For every non-observed `surface_group`, `materialize_live_candidates` starts
the `gate_us` timer immediately before:

```text
TransitionDecisionCore::admit_candidate_proposal(
    original,
    replacement,
    error_class,
    origin,
)
```

The call is a thin route through
`proposal_admission::gate_candidate_with_origin` into `candidate_admission`.
After it returns, the same timed span also includes:

```text
protected-surface equality
candidate_has_live_authority(...)
Eligible -> SuggestOnly authority deferral when required
```

`candidate_admission` is a short-circuit chain of independently meaningful
safety and authority predicates. It includes candidate explanation, boundary
shape checks, suffix and known-surface checks, action-operator verification,
left-context protection, structural-context gating, stable-shape checks,
semantic surface authority and final class dispatch. Several helpers repeat
token extraction, normalization, edit-distance work and lexical authority
queries for the same original/replacement pair.

V9 did not time those predicates separately. It also did not separate the
post-call live-authority override from `candidate_admission`. Therefore V9 does
not prove that any named predicate, lexical lookup, normalization, allocation,
or override is the dominant submechanism.

## Decision

The V9 result is accepted at exactly this level:

> The aggregate per-surface candidate-admission span is the dominant measured
> component of the traced final-materialization tail. Its internal dominant
> predicate or shared computation remains unknown.

No optimization is selected. In particular, this result does not authorize a
gate bypass, predicate removal, result cache, candidate-count cap, lexical
shortcut, authority downgrade, SafetyGate change, or special case for the four
tail ordinals.

The first structural worksheet is retained as an immutable `VETO`: it reused
broad evidence labels across incompatible roles. Effective route V2 separates
the aggregate result, timer boundary, uncertainty, reproducibility, causal
limit, successor, parity and stop boundary into distinct groups and passes.

```text
structural route V1 receipt SHA-256
  981bdb076d87f602c22470e1629d79b704e49bf4d9c07a7427c61bc8b166c01d
structural route V1 verdict             VETO / superseded
structural route V2 receipt SHA-256
  b83eb3096ebc676d8373a76960c61db57c081ca869f2d0779df080c87da95602
structural route V2 verdict             PASS
authority_ready                         false
```

The only admitted successor is a fresh test-only diagnostic paper that times
the existing admission chain below the V9 aggregate boundary while preserving
its exact decisions. The successor must distinguish at least:

```text
candidate explanation
boundary-family predicates
suffix / known-surface predicates
action-operator verification
surface / left-context predicates
each structural-context predicate
stable-shape / semantic predicates
final class dispatch
post-admission live-authority override
```

Per-predicate call counts, elapsed sums, short-circuit outcome and the exact
final `CandidateGateDecision` must be retained. The full fixed forward/reversed
candidate and certificate parity remains mandatory. Diagnostic timing may
identify a submechanism but may not replace the immutable V8R3 latency result.

## Consequence Boundary

The successor may propose test-only instrumentation and one fresh diagnostic
ELF only after its own structural gate, consequence analysis and implementation
preflight. Instrumentation must be compile-time absent from production builds,
must not duplicate admission logic, and must not create a second decision
owner. It may observe the existing path; it may not alter the path's ordering,
short-circuit semantics, candidate set, reasons, or authority.

Until that closure:

```text
V8R3 latency verdict                     BLOCKED_LATENCY, immutable
V9 decomposition verdict                 FINAL_MATERIALIZATION_DECOMPOSED
dominant aggregate                       candidate admission span
dominant internal predicate              UNKNOWN
optimization implementation              not admitted
production source/runtime authority      unchanged / not admitted
Cargo / rustc / subject / perf / PMU      0 / 0 / 0 / 0 / 0 in this decision
```

## Next Tree

```text
V9 FINAL_MATERIALIZATION_DECOMPOSED
        -> CANDIDATE_ADMISSION_AGGREGATE_DOMINANT_SUBMECHANISM_UNKNOWN
             -> fresh admission-substage diagnostic paper
                  -> structural gate
                  -> implementation preflight
                  -> test-only instrumentation and one diagnostic execution
                  -> exact candidate/certificate/gate parity
                  -> one submechanism decision

Any failed provenance or semantic gate
        -> STOP without optimization or production promotion
```
