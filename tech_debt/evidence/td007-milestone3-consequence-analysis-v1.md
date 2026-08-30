# TD-007 Milestone 3 Consequence Analysis V1

Status: `READY_FOR_IMPLEMENTATION`

## Scope

Milestone 3 reconciles the eleven remaining Nanda L2/L3 contract failures.
All eleven failures are stale or non-hermetic proofs. Runtime authority,
candidate birth, and ranking remain unchanged.

No fixture word, phrase, test name, or source ID will enter runtime branching.

## Baseline And Alternatives

Baseline:

- candidate birth, ranking, and automatic authority are currently distinct;
- `L2FieldShadow` is the live canonical candidate owner;
- legacy `BoundaryCell32` and `layout_then_l2_word_center` assertions do not
  define current producer identity;
- package-backed canonical readouts cannot be unit-tested hermetically by
  assuming an installed package or L1.1 service;
- known dictionary surfaces remain protected from unframed L3 rewrites.

Considered designs:

1. Restore every historical candidate and producer expectation.
   Rejected because it would add a second mutation step to exact layout,
   revive retired owners, and weaken known-surface safety.
2. Update expected strings to whatever the current runtime emits.
   Rejected because observed output alone is not authority and would preserve
   package-dependent tests.
3. Selected: assert active origin, retention, and safety effects; extract one
   behavior-preserving sparse-reserve helper and prove its unified candidate
   output; construct reference-backed ambiguity from the hermetic fuzzy frontier
   and prove downstream demotion.

## Consequences

- Candidate/lattice retention: unchanged.
- Ranking and false authority: no score, ordering, limit, DecisionCore,
  SafetyGate, edit-plan, or verifier rule changes. The boundary route still
  requires exact target reconstruction and `L2ImeTargetEvidence::Boundary`.
- Latency and tails: sparse-reserve extraction preserves the existing
  iteration, allocation, and limits.
- CPU/RSS: no package, cache, index, thread, or retained allocation changes.
- Cache identity and invalidation: unchanged.
- Package/delta reload: tests no longer depend on ambient package/service state;
  runtime package lookup and generation identity are unchanged.
- Learning/feedback: unchanged; no producer or feedback event is added.
- Concurrency/stale results: unchanged; no mutable state or new owner exists.
- Failure/rollback: the helper extraction is behavior-preserving and can be
  reverted independently. Test-only proofs have no installed effect.
- IME/daemon compatibility: unchanged.
- Maintenance/removal: stale producer assertions are replaced by semantic
  owner/effect assertions, reducing coupling to future producer renames.

## Expected Regressions And Proof

Potential regression: a replacement proof could preserve a candidate while
silently losing its gate or error class. The hermetic reserve and ambiguity
tests therefore exercise the unified candidate projection and gate postcondition.

Proof denominators:

- the eleven focused milestone tests;
- all correctness and package lanes against the exact remaining-failure set;
- the final fixed heldout proof after milestone 4.

## Authority Boundary

Installed/live runtime and source runtime semantics are unchanged.
