# L2 Productive Morphology: Post-V63 Pre-Build Review

Status: `V64_EXECUTED_AFTER_USER_APPROVAL`. The approved surface-basin
diagnostic is complete; V64 promotion is rejected and no later version is
assigned. This document remains the paper gate for any successor work.

## 1. Decision

Retain these V63 mechanisms:

- complete target-independent `LexicalLemmaObservationV1`;
- complete execution-based compatibility;
- set-valued `ColdLemmaBindingV1`;
- compact mmap package, `17,309,944 B`;
- fail-closed `ABSTAIN` authority.

Reject V63 promotion. Do not change coefficients or authority thresholds. The
next work must first make `B` observable, then remove representational crowding
without deleting compatible paradigms, and only then measure generation and
latency again.

## 2. Exact Failure Decomposition

For the `1,300` `LEMMA_HELDOUT` cases:

```text
cases                                            1,300
target lemma candidate born                      1,249
target slot candidate born                       1,198
exact target surface born                        1,197
exact target inside candidate top-16              1,175
exact target retained by final readout            1,197
```

First-loss buckets:

```text
no candidate from the target lemma                  51
lemma born, target slot absent                       51
slot born, exact target surface absent                1
exact target born but below top-16                   22
exact target lost by Winner/Tied/ABSTAIN readout      0
```

Per-class first-loss counts are:

| Class | No L | No slot after L | No exact after slot | Exact below top-16 |
|---|---:|---:|---:|---:|
| adjacent transposition | 4 | 4 | 0 | 3 |
| double substitution | 3 | 2 | 0 | 1 |
| extra letter | 4 | 2 | 0 | 0 |
| layout projection | 5 | 3 | 0 | 0 |
| letter substitution | 3 | 3 | 0 | 0 |
| missing letter | 4 | 9 | 0 | 4 |
| non-adjacent transposition | 4 | 1 | 0 | 0 |
| omission + transposition | 4 | 1 | 0 | 0 |
| prefix truncation | 3 | 3 | 0 | 0 |
| punctuation suffix | 3 | 2 | 0 | 0 |
| repeated fragment | 3 | 2 | 0 | 1 |
| sparse multi-omission | 4 | 3 | 0 | 0 |
| suffix truncation | 7 | 16 | 1 | 13 |

This falsifies a ranking-only explanation: `103 / 1,300` cases lose the target
before exact-surface ranking. It also falsifies a readout-erasure explanation:
once the exact target is born, final `ABSTAIN` suggestions retain it in all
`1,197 / 1,197` cases.

## 3. Proven Top-16 Crowding Mechanism

The runtime currently keeps identities, not surface equivalence classes:

```text
BindingCandidateFrontierV1
  key: target_slot_id per binding
  bound: 16 slot leaders

global RankedCandidateV1 heap
  key: ProductiveCandidateIdentityV1
       = lemma + paradigm + program + slot + surface hash + variant
  bound: 32 + overflow sentinel

calibrated readout
  dedup: exact identity only
```

Therefore several compatible paradigms that generate the same
`(lemma, slot, surface)` occupy several physical positions. Deduplication in
`retain_ranked_candidate()` compares the complete identity, so changing
`paradigm_id` or `program_id` prevents coalescing. The global heap truncates
before readout.

The capped failure receipt contains ten `exact born, below top-16` examples.
Every one has duplicate surfaces in the first eight candidates; duplicate
occupancy ranges from `4 / 8` to `7 / 8`. Examples include eight copies of
`пран`, eight copies of `перелезавши`, and eight copies of `подклеиваю`.

This is enough to prove representational crowding in the sampled failures. It
does not prove that all 22 top-16 losses have only this cause because the
receipt stores at most 64 failure examples.

Source owners:

```text
src/nanda_wave/l2_field/productive_v1/packaged_runtime.rs
  BindingCandidateFrontierV1::into_geometry_selected
  PackagedProductiveRuntimeV1::evaluate_checked
  retain_ranked_candidate

src/nanda_wave/l2_field/productive_v1/calibrate.rs
  calibrated_readout_selected
```

## 4. Unknown B/S Boundary

V63 does not emit an independent `B=true paradigm retained` event. The current
metrics cannot distinguish:

1. the true heldout paradigm is outside the train-learned hypothesis class;
2. it is in the class but removed by compatibility binding;
3. it is bound but its target slot is outside the per-binding 16-slot frontier;
4. its slot survives locally but all candidates are outside the global 32;
5. its slot survives but no program emits the target bytes.

The next proof must create a read-only oracle after the train split and after
the package is frozen. It may read excluded heldout forms only in proof code.
It must never enter compilation, fitting, calibration, runtime candidate birth,
or authority.

Required proof stages:

```text
H  true heldout paradigm is representable by train-learned P
B  at least one oracle-compatible paradigm survives C(O)
S0 target slot exists in a retained compatible paradigm
S1 target surface is executed before any frontier bound
S2 target surface survives per-binding slot selection
S3 target surface survives surface-basin/global selection
R  target survives Winner | Tied | ABSTAIN readout
```

`H` is needed to test theorem assumption 1. It is not a runtime stage and must
not be multiplied into product accuracy as if the runtime could repair an
out-of-hypothesis paradigm.

## 5. Paper Candidate: Surface-Equivalence Basin

The compatible paradigm set and the physical candidate frontier must become
two separate objects:

```text
complete C(O) bindings
-> execute compatible programs
-> coalesce equivalent outputs by
   (lemma_id, target_slot_id, normalized_surface)
-> retain one physical SurfaceBasinV1 per distinct output
-> preserve member count, ambiguity union, support envelope,
   deterministic representative and provenance digest
-> bounded 32-basin lattice
```

Coalescing after the global top-32 is too late. Coalescing only by surface is
also wrong because identical text in different morphology slots is syncretism
and must preserve distinct slot identities.

This representation is compatible with the identifiability theorem. Multiple
paradigms that produce the same target surface do not increase `|G_t(O)|`.
Distinct surfaces remain distinct basins and therefore remain `Tied` until L3
context resolves them. The complete compatible paradigm set is preserved as
proof/provenance; only repeated physical output nodes are coalesced.

## 6. Slot-Lattice Consequence

Surface-basin coalescing can recover physical capacity, but it must not be
claimed to solve the `51` missing-slot cases before `S0-S3` instrumentation.
If loss is at the hard 16-slot geometry frontier, increasing 16 is rejected.
The paper-consistent route is an implicit tied slot lattice:

```text
compatible paradigm/slot relation
-> compact unresolved slot basin
-> L3 contextual slot evidence
-> expand selected slot surfaces
```

L3 cannot repair a slot that L2 erased. Conversely, L2 must not force a slot
when morphology alone is non-identifiable.

## 7. Calibration And Authority

V63 calibration measured:

```text
source groups             1,348
target retained groups      121
target lost groups        1,227
fitted groups               121
```

All `2,600` proof cases returned `ABSTAIN`. This is fail-closed and must remain
so. The calibration table cannot be repaired by assigning weights manually or
lowering `MINIMUM_AUTHORITY_GROUPS`. Candidate-equivalence semantics must be
fixed first, then calibration must be replayed from the frozen context spool.

## 8. Latency Owner

The `LEMMA_HELDOUT` maximum class p99 is `144.976 ms`. Cold binding currently
scans compatibility postings and executes complete programs over exposed forms
inside request evaluation. The following are candidates for measurement, not
accepted optimizations:

- intersect observed-slot postings before program execution;
- avoid re-executing an already accepted paradigm from another source anchor;
- cache a validated cold binding by package generation plus lexical-observation
  digest;
- separate first-touch proof from warm repeated-query proof;
- move context-independent binding derivation out of candidate scoring.

Cache-only work cannot fix first-touch and cannot fix missing targets. Any cache
must preserve byte-identical candidate basins and invalidate atomically with
the package generation.

## 9. Rejected Routes

```text
increase top-16/top-32          rejected: hides duplicate representation and raises budgets
surface dedup after top-32      rejected: already-dropped basins cannot return
manual score/weight changes     rejected: 103 targets are absent before exact ranking
force Winner                    rejected: violates identifiability and false-authority contract
weaken verifier/SafetyGate      rejected: wrong ownership layer
reinduce the 435 MB corpus      rejected: current questions are runtime/proof questions
cache only                      rejected as quality fix; useful only after parity proof
L3 only                         rejected while L2 can erase the target slot
case-specific morphology rules rejected project-wide
```

## 10. Proposed Proof Order And Cost

No step below is authorized until this review is discussed.

```text
1. proof-only H/B/S0/S1/S2/S3/R instrumentation
   package change          none
   reinduction             none
   release build           approximately 3 minutes on remote host
   diagnostic proof        approximately 60 seconds

2. surface-equivalence micro against frozen V63 package
   denominator             all V63 2,600 cases, plus identity/provenance parity
   stop condition          any exact/readout regression, false singleton, integrity error
   reinduction             none

3. replay calibration and deterministic package from existing reduced artifacts
   raw corpus pass         none
   transition induction    none unless wire semantics require it
   expected package resume approximately 60 seconds from measured prior resume

4. repeat 13 x 100 x 2 diagnostic
   strict gate             every required class and cohort, all safety/resource budgets

5. only after diagnostic PASS
   frozen 20,000 per class, L3 handoff, verifier, multi-client and physical matrix
```

## 11. Review Questions

The required decisions before code are:

1. Accept `H/B/S0/S1/S2/S3/R` as the proof decomposition.
2. Accept surface-equivalence basins keyed by lemma, slot, and surface while
   preserving the complete compatible-paradigm provenance.
3. Keep the fixed `16 / 32` physical budgets and represent unresolved overflow
   symbolically instead of increasing them.
4. Require no raw-corpus reinduction for the next diagnostic cycle.
5. Keep V63 package/runtime authority shadow-only until the full conjunctive
   product gate passes.

Exact V63 evidence:

`docs/structural_gates/receipts/L2_PRODUCTIVE_V63_COLD_BINDING_2026-08-11/`.

## 12. V64 Measured Closure

V64 implemented the accepted surface-equivalence basin before global top-32,
kept slot/global limits `16 / 32`, added read-only `H/B/S0/S1/S2/S3/R`
instrumentation, and required exact probed/unprobed parity. It reused all V63
raw, sorted, reduced, induction, and context artifacts.

```text
package bytes      17,309,944
package sha256     9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
determinism        byte-identical across two independent resumes
probe parity       2,600 / 2,600 exact
false singleton   0
integrity errors   0
authority changed false
```

The heldout chain is now directly measured:

```text
1,300 -> H 1,280 -> B 1,219 -> S0 1,219
      -> S1 1,219 -> S2 1,219 -> S3 1,219 -> R 1,219
```

Against V63, exact birth improved `1,197 -> 1,219`, top-16 improved
`1,175 -> 1,218`, and raw top-1 improved `40 -> 267`. This accepts the
representational-crowding diagnosis. It also proves that remaining loss is no
longer owned by execution, the 16-slot frontier, global 32-basin selection, or
readout. The corrected target-POS audit assigns `20` cases outside `H`, `61`
to target-blind compatibility `B`, and `0` to `B -> S0`. Within `H -> B`, `59`
oracle paradigms are absent from the remaining source-slot postings and `2`
fail exact exposed-form reconstruction.

V64 remains `FAIL_measured_shadow_gates`: all outputs are still fail-closed
`ABSTAIN`, required top-1 gates fail, several retention classes are at or below
`95%`, and maximum class p99 is `97.519 ms`. The next paper question is the
systemic canonical-anchor dependency at `H -> B`, not another score, weight,
frontier, or authority change. The owning successor paper is
`docs/l2-productive-post-v64-anchor-recovery-paper.md`.

Exact V64 evidence:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/`.
