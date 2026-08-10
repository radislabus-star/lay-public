# L4 Causal Transition Memory Plan

Status: canonical forward plan for L4.

Execution authority remains in
`docs/phase-word-recovery-canonical-cutover.md`. This plan describes how L4
must evolve without becoming a second candidate generator, a second chooser,
or a collection of application- and word-specific rules.

## 1. Objective

L4 must become compact, local, transferable memory of typing-transition
outcomes:

```text
same kind of semantic scene
+ same typed operator
+ similar candidate relation
-> expected accepted / rejected / reverted outcome
```

The target runtime is:

```text
immutable TypingStateSnapshot
-> L1 surface observations
-> L2 bounded candidate lattice
-> L3 phrase relation field
-> L4 causal transition memory
-> one joint phase competition
-> TransitionDecisionCore
-> independent verifier
-> AuthorizedEdit or no edit
-> observed-state receipt
-> causal outcome routed back to L4
```

L4 does not generate words, project a keyboard layout, or decide grammar. It
remembers whether a proposed transition succeeded in a semantically equivalent
scene and exposes attraction, repulsion, ambiguity, or absence of evidence.

## 2. Current State And Proof Boundary

Already live:

- `TypingMemoryEvent` records typed, accepted, confirmed and rejected events;
- accepted layout projection is represented in feedback;
- usage memory supplies accepted/rejected counts and context priors;
- L4 computes a signed Bayesian signal;
- state-specific transition evidence reaches `TransitionDecisionCore`;
- hidden-state classes combine predicted state, operator, L3 relation,
  verifier evidence, exact witness and phase witness evidence;
- a valid negative or ambiguous certificate can block automatic application;
- compact positive and anti-phase centers exist;
- unknown evidence is not treated as rejection;
- L4 cannot create `AuthorizedEdit`.

Current proof boundary:

```text
exact historical feedback closure: PASS-shadow
negative false apply with memory:  0
negative false apply without L4:   228
cross-scene equivalent-scene proof: PASS-shadow
organic cross-scene promotion:      WATCH
whole-lattice scene boost:          REJECTED
```

Typed representation checkpoint (`0.2.345`):

```text
real V1 journal rows:             2 456
read-only V2 typed rows:          2 456
invalid typed rows:                   0
word states:                        711
transition states:               14 698
signed transition states:         9 416
dirty replay baseline/candidate:  byte-identical JSON
negative false apply:                 0 -> 0
runtime decision authority:       unchanged
```

The live-domain event now uses typed evidence source, interaction operation,
transition operator, layout direction/scope and outcome. Stable `u8` codes are
written in the V2 storage envelope beside lossless labels. The storage adapter
still reads V1; a V2 code/label mismatch is rejected before projection. The
receipt chain is structurally guarded as
`VisibleTailSnapshot -> DecisionTransitionReceipt -> VerifiedTransitionReceipt
-> AuthorizedEdit -> BackendDispatchReceipt -> PendingVisiblePostcondition ->
typed observed outcome`.

The rejected whole-lattice experiment raised support but increased false
top-1 from 1 to 5. L4 therefore must not become an unconstrained score over
every candidate. It must learn causal, typed and candidate-relative evidence.

### 2.1 Cross-scene V1 checkpoint (`0.2.346`)

The first bounded candidate-relative cross-scene field is implemented in
`src/nanda_wave/l4_cross_scene/`. It uses a 64-cell typed phase vector and
stores at most 4 positive, 4 negative, 2 hard-negative and 4 ambiguity centers
per profile. Directed pair memory is bounded to 1,024 pair profiles. The
runtime package contains typed operator, layout direction/scope, relation IDs,
compact signed phase cells, support and calibrated thresholds; it contains no
raw word, phrase, application or backend names.

Runtime ownership remains deliberately narrow:

```text
L1/L2 candidate already exists
-> L3 relation and context evidence
-> L4 cross-scene readout
-> Supported | Repelled | Ambiguous | Unknown
-> SuggestOnly | Keep
-> diagnostic signal inside TransitionDecisionCore
```

`SuggestOnly` cannot become `AuthorizedEdit`, does not add rank energy, does
not change candidate birth and cannot bypass the verifier. The read-only
runtime reloads an atomically replaced package at most once per second without
restarting IBus or the daemon.

Measured full-source proof on `e@192.168.3.94`:

```text
RU eligible source tokens:             2 096 840
EN eligible source tokens:               269 648
deterministic sample:                  512 + 512
train / heldout per language:          410 / 102
train-heldout word overlap:                    0
training causal observations:              1 716
heldout candidate-relative cases:             436
whole-token RU->EN positive/negative: 102/102 at 100%
whole-token EN->RU positive/negative: 102/102 at 100%
grapheme RU->EN positive/negative:         7/7 at 100%
grapheme EN->RU positive/negative:         7/7 at 100%
false automatic projection:                         0
without anti false supports:                 218/218
with anti false supports:                      0/218
no-context positive supports:                   8/218
shuffled direction/sign positive supports:          0
candidate readout order parity:                  PASS
compact package roundtrip:                       PASS
runtime/evaluator parity after i8 quantization:  PASS
package:                                      3 652 B
proof peak RSS:                           63 040 KiB
proof elapsed:                                9.03 s
```

The learner now calibrates the already quantized runtime centers. This removes
the measured defect where the high-precision learner passed all classes while
the compact package produced different numeric readouts.

The fixed 2,466-case dirty replay is semantically byte-identical to the typed
L4 baseline after removing only the input path field. Normalized SHA-256 is
`cd197d597144aeb4b52af5cefdc5f9872b9eef0aefdca7a3685ce8abba7fb43b`;
`negative_false_apply` remains `0`. Release replay took `57.69 s`, used
`158,556 KiB` peak RSS and performed no swaps.

Organic evidence is not promoted. A read-only snapshot of the current live
journal contained 2,437 rows but only 9 complete causal positives in 5 scenes,
45 orphan semantic labels and no negative or reverted causal receipts. Its
1,400-byte package is therefore a coverage receipt, not model authority. The
synthetic heldout package is proof material and is not installed as live
memory. Release binaries `0.2.346` are installed atomically without restarting
global IBus, the managed engine or the daemon.

Exact evidence:
`/home/e/build/lay-l1-shadow/artifacts/l4-cross-scene-v1-2026-08-01/report.json`
and
`/home/ubu/projects/lay/docs/structural_gates/receipts/L4_CROSS_SCENE_V1_SHADOW_2026-08-01.json`.

## 3. Architectural Defects To Remove

### 3.1 Typed identity baseline closed

The live domain no longer carries raw `source/operation` identity. Legacy labels
remain only in the backwards-compatible persistence envelope. The following
distinctions are now independently encoded:

```text
RU -> EN versus EN -> RU
whole-token versus single-grapheme projection
manual toggle versus automatic projection
IME acceptance versus observed automatic result
```

Typed V2 by itself closes representation parity only. Cross-scene V1 now proves
bounded `SuggestOnly` transfer on heldout equivalent scenes; organic promotion
still requires independent causal negative and reverted receipts.

### 3.2 Semantic state and execution state are mixed conceptually

Application, focus, caret, composition generation, backend and layout epoch
are needed to prove that an edit can be executed. They are not evidence that a
candidate is linguistically correct.

```text
SemanticSceneState
  L1/L2/L3/operator evidence used for candidate comparison

ExecutionLease
  focus/revision/caret/composition/backend proof used only by verifier
```

Telegram, WeChat, Firefox and Kitty may require different execution behavior,
but their names must never become language truth.

### 3.3 Exact history is not cross-scene transfer

Remembering one accepted transition closes that exact state. It does not prove
that the same operator transfers to new words, lengths, contexts or layout
directions. Transfer requires heldout phase evidence and ablation.

### 3.4 Outcome causality is incomplete

An applied edit is not automatically correct. A later user change can belong
to another focus, caret position, composition generation or word. L4 training
must join one continuous receipt chain from proposal to observed final state.

### 3.5 Single-character projection is ambiguous

Latin `b` projects to Cyrillic `и`, and Cyrillic `и` projects to Latin `b`.
The projection is deterministic; the intention is contextual. Existence of a
projection cannot grant automatic authority.

## 4. Non-Negotiable Laws

1. L2 births candidates; L4 never invents candidate text.
2. L3 owns phrase compatibility; L4 owns temporal outcome memory.
3. `TransitionDecisionCore` remains the only chooser.
4. Verifier remains the only path to `AuthorizedEdit`.
5. Unobserved is not negative.
6. Stale or mismatched receipts are censored, not rejected.
7. App/backend identity cannot become semantic authority.
8. Exact word history cannot prove transfer alone.
9. A learned anti-center may veto its matching transition class.
10. Positive L4 evidence cannot bypass structural or L3 rejection.
11. Multiword and left-context changes keep separate typed operators.
12. No production rule may name a concrete word or the pair `b/и`.
13. No speed optimisation may remove warm L2/L3/L4 candidate sources.

## 5. Target Ownership Tree

```text
L4 CAUSAL TRANSITION MEMORY
|
+-- event/
|   +-- ProposalReceipt
|   +-- DecisionReceipt
|   +-- DispatchReceipt
|   +-- ObservedStateReceipt
|   +-- OutcomeLabel
|
+-- scene/
|   +-- SemanticSceneState
|   +-- CandidateRelation
|   +-- OperatorIdentity
|   +-- ScenePhaseVector
|
+-- exact/
|   +-- latest accepted/rejected state
|   +-- exact transition witness
|
+-- phase/
|   +-- positive centers
|   +-- anti-centers
|   +-- bounded subcenters
|   +-- directed pair relations
|
+-- compiler/
|   +-- causal receipt join
|   +-- latest-state consolidation
|   +-- heldout split
|   +-- deterministic package build
|
+-- runtime/
|   +-- exact readout
|   +-- transferable phase readout
|   +-- pairwise certificate
|   +-- ambiguity certificate
|
+-- proof/
    +-- replay
    +-- ablations
    +-- permutation tests
    +-- latency/RSS/package budgets
```

Proof fixtures and raw logs must not be dependencies of `runtime/`.

## 6. Typed Event Contract

Introduce a versioned record:

```text
TransitionObservationV2
  event_id
  snapshot_identity
  proposal_id
  operator
  operator_direction
  projection_scope
  candidate_relation_id
  semantic_scene_id
  predicted_state_id
  decision
  verifier_receipt_id
  dispatch_receipt_id?
  observed_state_id?
  outcome
  confidence_class
```

Operator identity remains a closed enum. Layout projection adds:

```text
LayoutDirection = RuToEn | EnToRu
ProjectionScope = Grapheme | CurrentToken
```

No concrete character pair is encoded in the operator. `b -> и` is produced
by the universal keyboard projector and represented as `EnToRu + Grapheme`.
Old events remain readable through a compatibility decoder, but new runtime
code must not infer authority from source strings.

## 7. Outcome Model

Use five outcomes instead of overloading `+1 / 0 / -1`:

```text
ConfirmedPositive
  explicit accept, exact manual completion, stable observed result

ConfirmedNegative
  explicit reject, immediate correction away from candidate

Reverted
  transition returned to original state in the same causal lease

Ambiguous
  conflicting valid observations

Censored
  focus/caret changed, snapshot stale, backend failed, timeout, broken chain
```

Only positive, negative and reverted outcomes train signed semantic memory.
Censored outcomes affect reliability metrics only. Repeated identical events
are consolidated before training so duplicated logs cannot create confidence.

## 8. Causal Receipt Chain

```text
ProposalReceipt
-> DecisionReceipt
-> VerifierReceipt
-> DispatchReceipt
-> ObservedStateReceipt
-> OutcomeRouter
-> positive / negative / reverted / ambiguous / censored
```

Every link carries compatible snapshot identity and monotonic revision and
composition epochs. A broken link cannot become a training row.

For an IME prediction not accepted with Tab:

```text
candidate was visible
+ user manually completed exactly that candidate
+ word boundary committed
+ next token started in same focus/revision chain
-> weak ConfirmedPositive
```

This follows the real interaction without learning unfinished words.

## 9. Semantic Scene State

Allowed answer-independent inputs:

```text
operator class, direction and scope
L1 script topology and damage geometry
L2 candidate relation signature
L3 relation class and phase bucket
left-context phase signature
sentence-position and length/edit-shape buckets
Keep-versus-transition relation
accepted/rejected witness state
```

Forbidden semantic inputs:

```text
candidate list position
target label
raw application/backend name
focus id or caret coordinate
exact expected output as a feature
proof fixture id
```

Execution coordinates live only in `ExecutionLease`.

## 10. Compact Phase Memory

For every typed operator relation compile:

```text
positive scene centers
negative scene anti-centers
reverted hard anti-centers
bounded conflict subcenters
support and recency statistics
calibrated margin distribution
```

The hot package stores hashes, ternary/phase vectors, compact center sums,
thresholds and counts. It stores no source corpus or raw phrase.

```text
scene vector
-> positive coherence
-> negative coherence
-> destructive anti-wave
-> signed margin
-> Supported | Repelled | Ambiguous | Unknown
```

Cold compilation may read privacy-controlled local events temporarily. Joined
raw rows are removed after deterministic package creation.

## 11. Candidate-Relative Memory

L4 learns relations between candidates actually present in the L2 lattice:

```text
transition > Keep
candidate A > candidate B in scene family S
```

Relations are directed and antisymmetric. Unknown edges remain unknown. Cycles
and equally coherent conflicts produce `Ambiguous`.

This memory cannot create a candidate. It may suppress a causally rejected
candidate. Positive authority still requires L2 support, L3 compatibility and
verifier proof. This is the safe replacement for the rejected whole-lattice
scene boost.

## 12. Whole-Token RU/EN Projection

```text
typed token
-> L1 script/layout surface
-> L2 Keep candidate
-> L2 universal projected-token candidate
-> lexical phase binding for both surfaces
-> L3 compares phrase compatibility
-> L4 reads direction-specific transition outcomes
-> DecisionCore compares Keep and LayoutProjection
-> verifier proves current-token edit and layout postcondition
```

Examples such as `djn -> вот`, `lfdfq -> давай`, `цусрфе -> wechat` and the
reverse direction are eval rows, not production rules.

`RuToEn` and `EnToRu` have separate proof denominators. Passing one direction
does not authorize the other.

## 13. General Single-Grapheme Projection (`b/и`)

This is `LayoutProjection + Grapheme`, not a letter-specific rule.

L2 proposes both states:

```text
Keep("b")
LayoutProjection(EnToRu, "b" -> "и")
```

or:

```text
Keep("и")
LayoutProjection(RuToEn, "и" -> "b")
```

Authority levels:

```text
unknown/incomplete context
  Keep or SuggestOnly

strong L3 context, no transferable L4 certificate
  SuggestOnly

strong L3 context + positive L4 certificate + no anti-center + verifier pass
  eligible for learned Apply policy
```

Single-grapheme projection is a high-ambiguity operator family. Its policy is
learned and calibrated for the family, never encoded as a `b/и` threshold.
Context-free cases remain `Keep`/`SuggestOnly` unless exact personal evidence
is independently proven sufficient.

Heldout families must include Russian coordination, English technical `b`,
Russian genuine `и`, English contexts where Cyrillic `и` should become `b`,
mixed code/prose, and incomplete scenes that must abstain.

## 14. Joint Interference

L4 contributes one bounded lane:

```text
L2 lexical/transition energy
+ L3 phrase relation energy
+ L4 causal transition energy
-> one conserved joint field
```

L4 energy exposes exact attraction/repulsion, phase attraction, anti-wave,
pairwise margin and uncertainty separately. There is no post-DecisionCore L4
sort and no unbounded additive boost.

## 15. Learning Lifecycle And Package

```text
live events
-> causal receipt join
-> latest-state consolidation
-> quarantine stale/ambiguous rows
-> operator-stratified train/heldout split
-> positive/negative phase compilation
-> exact-memory cleanup experiment
-> replay and ablations
-> shadow package
-> promotion gate
-> bounded hot package
```

Personal learning stays local. Corpus data may initialize general operator
geometry but cannot override a user's confirmed negative transition.

The versioned package contains encoder hash, operator directory, positive and
negative banks, pair relations, calibration, generation id, proof-manifest
hash and checksum. It has bounded centers and profiles, no unbounded maps, no
raw words and no raw phrases.

## 16. Causal Proof Matrix

Every promotion runs:

```text
Full L4
No L4
Exact Witness Only
Phase Transfer Only
No Anti-Centers
Shuffled Outcome Sign
Shuffled Layout Direction
Shuffled Scene Phase
Magnitude Only
Candidate Order Permutation
No L3 Context
Stale Receipt Injection
Backend Failure Injection
```

Required invariants:

- candidate permutation preserves winner and margins;
- shuffled direction destroys direction-specific transfer;
- anti-center ablation exposes the targeted false-winner class;
- exact-memory cleanup preserves claimed heldout transfer;
- stale/backend failures become censored, not negative;
- unknown evidence never becomes rejection;
- L4 does not change L2 candidate birth;
- L4 cannot bypass verifier;
- evaluator and runtime use identical package and encoder versions.

## 17. Scoreboard And Promotion Gates

Report separately by operator and layout direction:

```text
causally joined observations
positive / negative / reverted / ambiguous / censored
exact and phase-transfer coverage
pairwise known / unknown / conflict edges
L2 candidate birth coverage
top-1 before/after L4
Apply / SuggestOnly / Keep / Abstain
false Apply and immediate undo rate
RuToEn / EnToRu precision and recall
single-grapheme precision and recall
organic certificate coverage
package bytes and RSS contribution
p50 / p95 / p99 / max latency
```

Hard gates:

```text
orphan semantic labels:                 0
stale/backend failures learned negative:0
known rejected/reverted reapply:        0
false automatic layout projection:      0 on fixed danger heldout
context-free grapheme auto apply:        0
negative Keep scenes preserved:         100%
candidate-order parity:                 PASS
phase and anti ablations causal:         PASS
runtime/evaluator parity:               PASS
```

Candidate birth, top-1, suggestion coverage and automatic apply are separate
denominators and must never be collapsed into one percentage.

## 18. Implementation Route

### Stage 0: freeze baseline

Record current version, fixed dirty replay, layout directions, single-grapheme
danger set, false applies, top-1, coverage, latency, RSS and rollback binaries.

Exit: reproducible baseline with fixed denominator.

### Stage 1: typed identities

Add typed operator direction, scope and outcome; version events; retain old
event decoding; remove new runtime inference from source strings.

Exit: old/new event readout parity.

### Stage 2: causal receipts

Bind proposal, decision, verifier, dispatch and observed-state receipts. Route
stale/focus/backend failures to censored. Confirm manual IME completion only
after boundary and next-token start.

Exit: no semantic label without a complete causal chain.

### Stage 3: scene encoder

Build `SemanticSceneState` from L1/L2/L3/operator signals and keep execution
coordinates in `ExecutionLease`.

Exit: deterministic train/runtime parity plus app/backend-name invariance.

### Stage 4: cold compiler

Implement latest-state consolidation, positive/negative/reverted/conflict
banks, bounded subcenters and deterministic package emission.

Exit: package roundtrip, bounded memory and raw-data cleanup proof.

### Stage 5: causal readout

Add exact, transferable phase, anti-center and ambiguity lanes in shadow.

Exit: fixed replay and causal ablations; runtime authority unchanged.

### Stage 6: candidate-relative memory

Learn `transition > Keep` and bounded pair relations; detect cycles; permit
suppression but no candidate birth or independent promotion.

Exit: false top-1 decreases, worsened count remains zero.

### Stage 7: whole-token layout shadow

Ensure universal L2 projection in both directions; train direction-specific L4
outcomes; verify text/layout/tray postcondition.

Exit: recall gain with zero fixed-set false automatic projections.

### Stage 8: single-grapheme shadow

Use the same operator with `Grapheme` scope; compare Keep/projection through L3
and L4; unknown cases remain suggestion-only; calibrate from heldout.

Exit: `b/и` works as general transfer with zero context-free auto applies.

### Stage 9: joint-field integration

Add bounded causal energy to the conserved field, remove duplicate late L4
sorting if found, and expose all L4 lanes in diagnostics.

Exit: one chooser, runtime/eval parity and no authority bypass.

### Stage 10: organic shadow soak

Accumulate causally joined local traffic without changing physical behavior;
measure certificate coverage, false winners, censored outcomes and latency;
compare every package generation on fixed replay.

Exit: sufficient organic coverage and no safety regression.

### Stage 11: staged promotion

```text
IME ranking
-> SuggestOnly visibility
-> exact negative veto
-> exact positive authority
-> whole-token transfer authority
-> single-grapheme transfer authority
```

Every stage has an independent kill switch and rollback package.

### Stage 12: observability

Expose learned operator families, positive/negative outcomes, unknown scenes,
package generation and decision explanation in CLI/tray without raw personal
text. One privacy/debug switch controls raw event logging.

## 19. TREE / SCOREBOARD / DEBT QUEUE

### TREE

```text
surface
-> L2 Keep + typed operator candidates
-> L3 phrase relations
-> L4 exact + phase + anti + pairwise memory
-> conserved joint field
-> DecisionCore
-> verifier / AuthorizedEdit
-> observed outcome
-> causal L4 compiler
```

### SCOREBOARD

```text
exact feedback closure:       PASS-shadow
negative replay closure:      PASS-shadow
typed event V2 replay parity: PASS
typed causal chain:           PASS-structural
hidden-state certificates:    LIVE
phase witness bank:           LIVE-bounded
cross-scene synthetic proof:  PASS-shadow
cross-scene organic package:  WATCH-insufficient-evidence
whole-lattice scene boost:    REJECTED
whole-token layout transfer:  PASS-shadow-SuggestOnly
single-grapheme transfer:     PASS-shadow-SuggestOnly
```

### DEBT QUEUE

```text
DONE typed event/operator identity (`0.2.345`)
DONE causal observed-state receipt chain (`0.2.345`, structural)
DONE fixed layout and grapheme danger sets
DONE scene encoder train/runtime parity
DONE causal phase compiler and ablations
DONE direction-specific whole-token layout transfer
DONE single-grapheme SuggestOnly proof
P2 organic shadow coverage
P2 learned apply calibration
P2 tray/CLI learning report
```

## 20. Definition Of Done

```text
feedback identity is fully typed
no semantic learning occurs without a causal receipt chain
execution state cannot become language truth
exact rejected/reverted transitions remain closed
phase transfer improves unseen equivalent scenes
anti-center ablation proves destructive interference
RuToEn and EnToRu pass independently
single-grapheme projection uses the general operator family
context-free grapheme auto apply remains zero
L4 does not change L2 candidate birth
L4 does not bypass L3 rejection or verifier safety
runtime/evaluator package and encoder match
hot package is compact and bounded
false automatic layout projection is zero on fixed heldout
IME, Space, Tab, Backspace, double Shift and app smoke gates pass
```

The end state is a local causal memory that learns which typed transition
succeeds in which semantic scene, transfers the relation through compact phase
centers, and abstains when scene or outcome is not proven.

## 21. Exact User Rollback Feedback And Standard Publication, 2026-08-10

### What was tested

The same causal outcome now owns both live rollback adapters:

```text
successful system apply
-> exact user rollback
-> TypingMemoryOutcome::Reverted
-> typed L4 anti / hard-negative evidence
```

IBus preserves the typed system transition in `PendingImeAutoUndo`; double
Shift records `record_reverted_system_apply` and no longer mislabels the action
as a generic user correction. The daemon writes the distinct receipt kind
`system-apply-reverted`. Historical correction backfill accepts only the exact
structural cycle `lay_from -> lay_to -> lay_from`; an unrelated later edit is
not inferred to be a rollback.

The standard `scripts/rebuild-l4-feedback-memory.sh` now compiles the bounded
cross-scene package from the live usage journal plus exact correction receipts,
validates that the package is `shadow_suggest_only`, and atomically publishes
it. The older bounded usage-count snapshot remains a separate output of the
same rebuild command. The published package was then verified at
`/home/ubu/.local/share/lay/nanda_wave/l4_cross_scene_v1.bin` without granting
automatic edit authority.

### Measured facts

```text
live usage source                    1,634 rows / 511,837 B
correction source                    2,950 rows / 390,502 B
exact rollback receipts                                   172
rollback token observations                               176
live joined positives                                      10
joined observations total                                 186
ignored live observations                               1,624
invalid observations                                        0
conflict scenes                                              4
consolidated scenes                                         61
profiles / pair profiles                               16 / 58
package bytes                                           13,228
package SHA-256       5a32cf50b94105679ec40bec7bd5c46c2937075ede864bd7961203427a6cf1b5
installed package bytes                                  13,228
installed package SHA-256   5a32cf50b94105679ec40bec7bd5c46c2937075ede864bd7961203427a6cf1b5
raw text stored                                         false
runtime authority                         shadow_suggest_only
automatic apply possible                                 false
```

Two independent compile routes, direct CLI and the standard rebuild script,
produced the same package SHA-256. Focused gates passed: L4 cross-scene
`13/13`, typed rollback `1/1`, related auto-undo contracts `12/12`, daemon
receipt `1/1`, IBus auto-undo `4/4`, authority contract `20/20`, mutation
monopoly `15/15`, and the final unsafe-edit scoreboard with `0` gate failures.

### What was not tested

- organic heldout transfer or organic anti-center ablation: only `10` positive
  observations and `4` conflict scenes exist, so a promotion denominator is
  not credible;
- automatic edit promotion;
- physical automatic application behavior, because the installed package is
  deliberately `shadow_suggest_only` and has no apply authority;
- L1.1 or L2 quality, package, RSS, or latency gates.

### Verdict scope

- exact live rollback identity: `PASS_targeted`;
- exact historical rollback backfill: `PASS_targeted`;
- deterministic bounded package compilation: `PASS_shadow`;
- standard atomic package publication: `PASS_installed`;
- organic L4 promotion: `WATCH_insufficient_independent_scenes`;
- runtime decision authority changed: `false`;
- automatic apply authority: `NOT_GRANTED`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_PROCESS_REFRESH_L4_ROLLBACK_FEEDBACK_1_0_19_2026-08-10.json
```
