# DAFSA Typed View M3 Admission Lexical Fact Reuse V11 Implementation Consequence V1

Date: 2026-08-27

Status: `READY_FOR_IMPLEMENTATION_PREFLIGHT`

## Authority Boundary

This contract implements only the test-build mechanism selected by the V11
decision. It permits one edit surface:

```text
src/typing_transition/proposal_admission.rs
```

All runtime owners, candidate producers, `TransitionDecisionCore`, SafetyGate,
verifier, typed-view owner, sidecar/package code, Cargo inputs, daemon and IBus
sources remain byte-identical. No installed binary or package is changed.

The implementation pass may compile and run focused local non-scientific tests.
It may not contact the target host, create a scientific namespace or marker,
build the future B0/B1 target ELF, or execute the M3 physical route.

## Mechanism

`candidate_admission` creates one call-local lexical-fact owner. The owner is
lazy: no word extraction, normalization or lexical lookup occurs before the
first predicate that already required it. The owner is destroyed when the
single admission call returns.

In test builds an explicit mode selects either:

```text
UNCACHED  every consumer executes the existing helper path
REUSE     the first consumer computes a fact and later consumers reuse it
```

The future experiment controls the mode with
`LAY_PROPOSAL_ADMISSION_FACT_REUSE`. A test-only thread-local override is
allowed solely for deterministic unit parity without mutating a process-global
environment variable. A non-test build contains no environment read, override,
counter or retained cache.

The reusable fact set is bounded to the exact candidate pair:

```text
last text word slices
lowercase original/replacement words
Cyrillic-only flags
character lengths
known_russian_autocorrect_token(original)
known_russian_autocorrect_token(replacement)
protected_current_surface_token(original)
```

No fact may be keyed by case ordinal, source ID, candidate count, error class or
specific text. There is no map, static cache, thread-local result cache,
cross-request storage or eviction policy.

## Semantic Preservation

The call graph and short-circuit order remain exact. Existing predicates still
own every branch and final reason:

```text
known_current_word_gets_unproven_surface_drift
stable_known_form_grows_into_infinitive_overreach
protected_current_surface_rewrite_requires_context_authority
known_russian_word_rewritten_to_different_known_word
unproven_stable_surface_shape_drift
nanda_surface_candidate_outputs_unknown_word
```

The owner supplies facts only where the old helper would have computed the same
fact. It cannot precompute a later predicate, turn a negative observation into
authority, or bypass a current error-class/origin guard.

The local parity proof must compare UNCACHED and REUSE decisions over the
complete existing proposal-admission fixture matrix under both
`FullReferenceAllowed` and `FieldSnapshotOnly`. It compares exact action and
reason. Focused existing typing-transition and correction-core tests remain
mandatory.

## Future Scientific Route

This implementation creates no scientific authority. A separate remote
execution paper and preflight must freeze:

```text
one fresh target-host build
same ELF for B0 and B1
B0 UNCACHED once
B1 REUSE once
same owner, inputs, CPU, cases, warmup and four-round schedule
full candidate/certificate/lattice/emission/gate parity
paired latency, RSS, reload and rollback gates
```

The future wrapper, not the subject receipt, may bind the exact mode to each
one-shot route. No V8, V9 or V10 marker is reused.

## Local State Machine

```text
implementation preflight READY
  -> edit proposal_admission.rs
  -> rustfmt/static checks
  -> focused parity and compile-only checks
  -> immutable implementation receipt
  -> V11_MECHANISM_IMPLEMENTED_UNRUN
  -> STOP before remote execution admission
```

Any source, format, compile, parity or receipt failure retains diagnostics and
publishes no positive implementation verdict. It does not grant a second edit
or scientific run.

## Claim Boundary

```text
runtime authority changed              false
production behavior selected           false
installed Lay changed                  false
remote reads/writes                     0 / 0
scientific markers                      0
scientific subjects                     0
perf / PMU                              0 / 0
future B0/B1                            not run
```

The only positive implementation verdict is:

```text
V11_MECHANISM_IMPLEMENTED_UNRUN
```

It admits only a separate execution preflight.

