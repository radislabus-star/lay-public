# Wave Transition CPU Plan

> Execution authority: `docs/phase-word-recovery-canonical-cutover.md`.
> This document is retained as supporting transition-memory design history.
> Where wording or implementation order differs, the canonical cutover wins.

## Goal

Replace remaining "candidate got a score and applied" behavior with portable
phase circuits for typed text transitions:

```text
TypingState
-> proposed typed transition
-> relation atoms
-> L2 phase circuit
-> L4 surface frontier
-> verifier
-> AuthorizedEdit | SuggestOnly | ABSTAIN
```

L1 sees the surface. L2 learns transition form. L3 supplies phrase context.
L4 keeps the map of surfaces, experience, and transferability. None of these
layers may mutate text directly.

## 0. Freeze Baseline

Record the current version, dirty-log evaluation, candidate coverage, false
applies, unsafe multiword edits, p50/p99/max latency, and RSS.

Build a transition corpus:

```text
state_before
action proposal
state_after
operator family
verifier result
accepted / rejected / undo
surface_id
route
```

Concrete words are observations for training and debugging, not authority.

Required regression gate: typing, Tab, IME, double Shift, layout sync, and
Space must remain correct.

## 1. Typed Transition Vocabulary

Keep a closed set of executable operators:

```text
ReplaceCurrentToken
LayoutProjection
AdjacentTransposition
MissingLetterRepair
RepeatedLetterRepair
BoundarySplit
BoundaryMerge
AcceptCompletion
ManualToggle
UndoPreviousEdit
Keep
SuggestOnly
ABSTAIN
```

Every operator declares its input shape, allowed and forbidden state delta,
relation atoms, verifier binding, and backend capability.

Ordinary operators may change only the current token. Boundary changes require
their own proof. `AuthorizedEdit` cannot exist without a typed operator and a
passing verifier.

## 2. Relation Atom Encoder

Encode properties of a transition instead of a word identity:

```text
word_count: same / changed
left_context: preserved / touched
script: ru / en / mixed
layout: same / projected
edit distance bucket
prefix/suffix preservation
insert/delete/transposition shape
cursor and revision consistency
context compatibility
operator proposal
verifier evidence
```

The atom encoder does not encode a ready answer. It describes what was
preserved, changed, and proven. Role swaps, foreign left context, multiword
drift, and stale surrounding text must contribute negative atoms.

## 3. L2 Phase-Center Transition Memory

Each operator family has compact phase memory:

```text
positive phase centers
negative / anti-centers
surface coverage
margin threshold
support count
cleanup eligibility
```

Verified accepted transitions train positive centers. Rejected candidates,
undo/revert evidence, verifier rejections, role swaps, boundary violations,
wrong layout direction, and unsafe near misses train anti-centers.

Readout is:

```text
transition atoms
-> phase vector
-> coherence with positive center
-> coherence with anti-center
-> phase margin
-> L2 phase verdict
```

L2 may return support, repel, or unknown with a margin and coverage evidence;
it may not execute an edit.

## 4. L4 Adaptive Surface Frontier

L4 is a map of transferability, not a second candidate scorer. Per operator it
keeps:

```text
known surfaces
covered surfaces
heldout surfaces
accepted transitions
rejected transitions
undo transitions
phase margin distribution
verifier outcomes
desync risk
```

A surface is a new expression of the same transition: another word, length,
layout, error position, context, or boundary shape. It is not a word identity.

For an unknown surface:

```text
unknown surface
-> shadow only
-> verifier labels result
-> covered / still unknown
-> phase circuit may be recomputed
```

There is no production rule such as "three surfaces allow apply". Promotion
uses margin, anti-center separation, heldout evidence, and verifier statistics.

## 5. L3 Context Constraint

L3 answers whether a typed transition is allowed in the current phrase scene.
It contributes phrase-role, previous-word, language-scene, technical/prose,
sentence-continuation, and phrase-pressure atoms.

A strong L2 typo circuit that breaks the phrase becomes `SuggestOnly` or
`ABSTAIN`. L3 cannot override a negative phase center.

## 6. Transition Decision Core

The single decision order is:

```text
candidate producers
-> typed transition proposals
-> L1 relation atoms
-> L2 phase readout
-> L3 context readout
-> L4 surface/readiness readout
-> verifier
-> decision
```

Results are `Apply`, `SuggestOnly`, `Keep`, `ABSTAIN`, or `Veto`.

`Apply` requires a valid typed operator, adequate L2 support, no dominant
anti-center, no L3 rejection, covered or explicitly safe L4 surface, passing
verifier, local revision-safe EditPlan, and backend support. Unknown cases
prefer `SuggestOnly` or `ABSTAIN`.

## 7. Exact-Memory Demotion

Exact counts and dictionaries remain training material, fallback material,
debug evidence, and shadow baselines. They cease to be final authority.

For each operator family prove:

```text
1. train with exact traces
2. measure unseen surfaces
3. compile phase circuit
4. remove exact support for measured cases
5. rerun heldout
6. compare phase ablations
```

Promotion requires cleanup-preserved transfer, phase dependence, anti-center
value on near misses, and `wrong accepts = 0`.

## 8. Shadow Replay

Before any live apply, replay dirty logs:

```text
old decision
new decision
verifier outcome
would_apply
would_abstain
would_suggest
unsafe edit blocked
missed good edit
```

Report coverage, accepted/rejected/undo, false apply, false abstain, phase
margin, surface count, heldout surfaces, and latency for every operator.

Mandatory danger cases are multiword deletion, left-context mutation, cursor
jump, word glue, long replacement, wrong layout flip, and IME tail mismatch.

## 9. Cutover Order

Promote one operator family at a time:

```text
1. AdjacentTransposition
2. MissingLetterRepair
3. RepeatedLetterRepair
4. LayoutProjection
5. AcceptCompletion
6. BoundarySplit
7. BoundaryMerge
8. CompositeTypo
```

Each stage is shadow, verifier-backed apply, live statistics, then an
installable rollback-capable checkpoint. Boundary operators are last.

## 10. IME and Daemon

IME remains a backend:

```text
TypingStateSnapshot
-> shared DecisionCore
-> candidate display
-> accepted typed transition
-> AuthorizedEdit
-> CommitText / DeleteSurroundingText
```

IME has no independent Bayes, L2, autocorrect, or decision authority. IME and
daemon may have different snapshots but must carry focus ID, revision, caret,
composition state, visible tail, and layout state. Snapshot mismatch means
`ABSTAIN`, never a guessed restoration.

## 11. Proof Suite

Every promoted circuit requires independent surfaces, heldout transfer,
exact-memory cleanup, positive-center ablation, anti-center ablation, phase
shuffle, magnitude-only ablation, random-center control, and zero verifier
false accepts.

## 12. Production Metrics

Expose facts in the tray and CLI:

```text
operator coverage
surface frontier
phase centers / anti-centers
exact traces eligible for cleanup
accepted / rejected / undone
false applies
ABSTAIN rate
L2/L3/L4 contribution
p50 / p99 / max latency
hot memory bytes
```

Quality is:

```text
good verified applies
- false applies
- unsafe edits
- unexplained latency
```

## Final Architecture

```text
L1: surface and transition atoms
L2: phase-center memory of typed corrections
L3: phrase/context admissibility
L4: adaptive surface frontier and signed experience
DecisionCore: typed selection and ABSTAIN
Verifier: proves predicted state transition
AuthorizedEdit: sole execution capability
IME / daemon: output backends
```

The end state is not an autocorrector that guesses more words. It recognizes a
portable transition type, remembers it as a compact phase circuit, determines
whether the new surface is covered, and executes only a proven transition.

## Implementation Scoreboard

Release checkpoint: `0.2.207`.

| Stage | Status | Evidence |
| --- | --- | --- |
| 0. Baseline | PASS | `0.2.206` runtime snapshot, dirty-log corpus, latency and PSS recorded before cutover |
| 1. Typed vocabulary | PASS | adapter-neutral `TransitionOperatorKind` covers 11 learned executable families; manual/unknown remain non-learned |
| 2. Relation atoms | PASS | no concrete word identity; changed region, current token, boundary, script, proof and verifier shape are encoded |
| 3. L2 phase memory | PASS | `LAYPC004`, 128 cells, 48 positive centers, 139 anti-centers, 11/11 promoted profiles |
| 4. L4 frontier | PASS | exact-state accepted/rejected evidence, latest-state consolidation, signed fallback |
| 5. L3 constraint | PASS | contextual evidence is owned by L3/L4 and cannot train broad L2 anti-centers |
| 6. Decision core | PASS | phase memory has no apply authority; `TransitionDecisionCore` remains the sole chooser |
| 7. Exact-memory demotion | PASS | hot package has zero exact traces and stores no raw words |
| 8. Shadow replay | PASS-safety / WATCH-coverage | zero negative false applies; candidate coverage remains a product-quality debt |
| 9. Cutover | PASS | all 11 proven operator profiles are enabled through fail-closed phase admission |
| 10. IME/daemon | PASS | both consume shared decisions; IME remains display/execution backend only |
| 11. Proof suite | PASS | full phase 72/72, near-negative repel 170/170, false accepts 0; no-phase and magnitude-only support 0 |
| 12. Metrics | PASS | phase package, promotion, L4 state, decisions and latency are exposed in CLI/tray diagnostics |

### Causal Proof

```text
training entries:                 745
heldout entries:                  242
full-phase positive support:      72 / 72
full-phase negative repel:        170 / 170
full-phase false accepts:         0
promoted operators:               11 / 11
no-phase positive support:        0 / 72
magnitude-only positive support:  0 / 72
exact traces after compile:        0
raw words in hot package:         false
```

Removing anti-centers produces 170 false accepts. Destroying phase removes all
heldout positive support. The executable result therefore depends on phase and
anti-wave structure, not on magnitude or exact lookup.

### Runtime Result

```text
phase package:       96,772 bytes
operator coverage:   100%
covered surfaces:    1,697
rejected surfaces:   668

precognition hot path, n=120:
  p50:                23 us
  p90:                50 us
  p99:                63 us
  max:                82 us

lay-daemon:
  PSS:                4,583 kB

lay-ibus-engine:
  PSS:                45,466 kB
```

The dirty-log latest-state replay preserved zero negative false applies with
phase admission both off and on. Its positive candidate coverage was 37.07%,
which is explicitly a candidate-birth debt, not a reason to weaken transition
verification.

### Final Tree

```text
TypingStateSnapshot
-> L1 relation encoder
-> L2 candidate lattice
-> L2 promoted phase centers / anti-centers
-> L3 context constraint
-> L4 exact-state signed memory
-> TransitionDecisionCore
-> TransitionVerifier
-> AuthorizedEdit
-> daemon or IME backend
```

### Remaining Product Debt

1. Raise candidate birth coverage without weakening admission.
2. Accumulate organic L4 evidence without mixing stale accept/reject states.
3. Keep context rejection in L3/L4 instead of broad L2 anti-memory.
4. Measure end-to-end output latency separately from microsecond phase readout.
5. Preserve zero unsafe multiword and unverified left-context applies in live logs.
