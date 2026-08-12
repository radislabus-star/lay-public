# L2 Productive Paradigm Field: Canonical Design

Status: `DESIGN_ONLY`, implementation and runtime promotion are not complete.

Date: 2026-08-10.

Owning layer: canonical L2 above the immutable L1.1 restoration lattice.

Executable paper specification:
`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-implementation.md`.

Paper completeness review:
`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-review-2026-08-10.md`.

This document is the design authority for the next productive Russian
morphology kernel. It replaces scheduler-level experimentation as the active
architectural direction. It does not replace measured receipts, does not grant
runtime authority, and does not claim that the proposed quality or latency
gates have passed.

## 1. Executive Verdict

The current productive L2 has proved one important `BANK_UNSEEN` capability: it
can find a lemma and materialize a Russian form that is absent from the exact
L1.1 terminal bank. The sampled lemmas were not excluded from productive
sidecar training, so this is not a `LEMMA_HELDOUT` generalization proof. It has
not proved safe standalone selection or accepted hot-path latency.

The remaining failures have one shared architectural shape:

```text
active LemmaCenter set
-> enumerate target morphology slots
-> materialize generated UTF-8 surfaces independently
-> rebuild character and keyboard geometry independently per surface
-> run Damerau-Levenshtein independently per surface
-> truncate the materialized field
-> apply sparse directional evidence late
```

This shape repeats work shared by forms of the same lemma and allows weakly
licensed suffix transforms to enter the same readout as paradigm-compatible
forms. Scheduler, cache, allocation, and local Damerau changes can move an
individual measurement, but they do not remove the repeated representation.

The canonical replacement is:

```text
L1.1 bounded lemma lattice
-> LemmaParadigmBinding
-> learned ParadigmCenter
-> context-conditioned MorphologySlot phase field
-> implicit productive prefix trie
-> one shared exact character + keyboard geometry traversal
-> evidence-calibrated energy reduce
-> Winner | bounded Tied lattice | ABSTAIN
-> L3 sentence field
-> verifier
```

The form graph is storage and traversal geometry. Learned positive, anti, and
ambiguity phase centers remain the decision field. This is therefore a
structural correction to the wave kernel, not a replacement of it with a
literal rule engine.

## 2. Claim Discipline

The statements in this document use four scopes:

```text
MEASURED       present in an exact receipt listed in section 19
CANONICAL      accepted architecture contract for the next implementation
HYPOTHESIS     expected consequence that still requires measurement
OPEN           unresolved design decision that blocks implementation authority
```

No `HYPOTHESIS` or `CANONICAL` statement is a runtime quality claim. Runtime
authority remains unchanged until every promotion gate in section 17 passes.

## 3. Measured Baseline

### 3.1 Bank-unseen mechanism capability

The V38/V39 source-reference family preserves the same fixed quality and safety
denominators:

```text
evaluated cases                                  1,300
generated top-16, worst                            94%
generated readout target retention, worst          91%
generated unique top-1, worst                      61%
false authority / false singleton                 0 / 0
directional same-lemma comparisons                3,483
directional pair coverage                       42.463%
directional target wins / reverse false            69 / 0
productive sidecar bytes                    81,688,382
admitted productive rules                    1,268,215
directional pair relations                        7,191
```

The exact sidecar is mmap-backed. Generated forms remain `SuggestOnly`.

This denominator is `BANK_UNSEEN`: exact target annotation is disabled during
generated birth, but sampled lemmas were not excluded from sidecar training.
The receipt explicitly lists leave-lemma-out generalization as not tested.

### 3.2 Accepted speed baseline

V38 established the best single measured reference before V39:

```text
configuration                         p50       p99       peak RSS
V38 request-level slot map          2.328     5.885 ms   336,692 KiB
gate                                            <=5.000 ms
```

V39 preserved exact class and directional summaries and consistently improved
p50, but did not close p99:

```text
run                                  p50       p99       peak RSS
1                                  2.207     6.092 ms   336,852 KiB
2                                  2.292     7.027 ms   336,648 KiB
3                                  2.244     5.711 ms   337,012 KiB
4                                  2.236     5.819 ms   337,016 KiB
median p99                                    approximately 5.96 ms
twenty-worker 13 x 10 p99                         59.952 ms
```

V39 remains `PASS_reference_parity_FAIL_quality_gate_FAIL_latency`. It is the
source baseline to restore before implementing this design; parity with a
failing reference is not a quality pass.

### 3.3 V42 bounded lemma chunks

V42 replaced up to 256 indexed Rayon lemma tasks with deterministic bounded
chunks. It changed no package, limit, score, transform, or authority rule.
Quality and directional hashes remained identical to V39 and false authority /
singleton remained `0 / 0`.

The release measurements rejected the change:

```text
run                                  p50       p99       peak RSS
workers=1, 13 x 100 run 1          2.507     6.602 ms   337,016 KiB
workers=1, 13 x 100 run 2          2.518     6.343 ms   336,976 KiB
workers=1, 13 x 100 run 3          2.936     7.735 ms   336,640 KiB
workers=1, 13 x 100 run 4          3.034     8.179 ms   337,016 KiB
workers=20, 13 x 10               14.956    70.105 ms   375,708 KiB
gate                                            <=5.000 ms
```

Verdict: `REJECT_bounded_lemma_chunks_no_tail_gain`.

The V42 code is not canonical and must be removed before the next
implementation. The V39 exact geometry and small-range dedup changes remain
the source baseline.

### 3.4 Profile evidence

The unstripped V39 profile identified these self-costs:

```text
damerau_levenshtein_rows                         19.78%
Unicode conversion lookup                         9.52%
compact decoder block cache                       4.81%
generate_forms_prepared                           4.21%
bounded Damerau rows                              4.11%
UTF-8 conversion                                  3.85%
RU key mapping                                    3.60%
allocator family                         approximately 15%
crossbeam steal                                    3.64%
```

The profile does not prove that any isolated local edit will improve release
p99. V40 and V41 already disproved that interpretation for lowercase keyboard
units and common-edge trimming.

## 4. Current Failure Model

### 4.1 Birth is no longer the only problem

The productive route has separately proved:

- target lemma retention can reach 100% on the micro denominator;
- a bounded `prefix + retained stem + suffix` operator can materialize unseen
  forms;
- cross-lemma domination must remain prohibited inside L2;
- exact observed competitor evidence can create directional support without
  reverse false support.

The current larger proof still loses quality at three later boundaries:

```text
generated target outside top-16                  up to 6%
target lost between top-16 and readout            up to 3% more
target retained but not unique top-1             up to 30% more
```

These are distinct denominators and must not be collapsed into aggregate
top-1.

### 4.2 Sparse pair evidence is not a complete field

The current directional package contains 7,191 exact pair relations. The fixed
directional proof finds evidence for only 42.463% of same-lemma comparisons.
Exact relations are safe because they produce zero reverse false supports, but
missing pair records leave 2,004 comparisons without evidence.

Completing a quadratic table for every form pair is rejected. It would:

- grow with the square of each lemma paradigm;
- memorize observed surface pairs instead of learning morphology slots;
- remain sparse for unseen forms;
- make append-only updates expensive;
- duplicate evidence already factorable by number, case, person, tense, mood,
  gender, and form kind.

Exact pair relations remain useful as residual evidence for irregular or
context-specific competitions. They are not the primary morphology field.

### 4.3 Free transform mixing is underconstrained

Global suffix or edge transforms can be supported across the corpus while
remaining incompatible with a particular lemma paradigm. This creates
orthographically plausible but morphologically invalid surfaces.

The runtime must distinguish:

```text
transform exists globally
transform belongs to this paradigm
this lemma is compatible with this paradigm
this context supports the target morphology slot
the damaged surface geometrically supports the generated form
```

These are independent evidence factors. A global transform is not sufficient
authority for a lemma-specific form.

### 4.4 Nested parallelism is not the algorithm

The proof `--workers 1` means one outer proof caller, not one physical CPU. The
inner Rayon expansion still uses the shared pool. With many outer callers, the
same global pool services several independent per-request lemma graphs.

V42 demonstrates that changing task granularity trades scheduling overhead for
load imbalance. It does not remove repeated surface construction or geometry.
The canonical hot path therefore cannot depend on one Rayon task per lemma as
its core performance mechanism.

## 5. Fixed Constraints

The redesign must preserve all of these constraints:

```text
L1.1 package                         immutable baseline
canonical L2 package                immutable baseline
broad lemma frontier                256
active lemma frontier               256
features per lemma                   16
productive form lane                 32
composite identity bound       32 grounded + 32 productive
atom relation budget             196,608
grounded L1.1 candidate loss          0
grounded L1.1 Winner downgrade        contradiction certificate only
runtime word-specific branches        0
generated runtime authority      disabled until promotion
```

The sidecar may change because it is the owner of productive morphology. Base
L1.1 and canonical L2 must not be recrystallized to implement this design.

## 6. Canonical Ownership

### 6.1 Input scene encoder

A typed input scene encoder may produce read-only structural features before
L2:

- left and right token boundaries;
- punctuation and order;
- local function-word and agreement features;
- script and layout features;
- typed morphology-axis observations.

This encoder has no candidate readout and no edit authority. It is a sensor,
not an additional correction layer.

### 6.2 L1.1

L1.1 owns restoration evidence from the damaged visible signal:

```text
surface atoms
-> bounded lexical lattice
-> Winner | Tied | ABSTAIN
```

L1.1 does not own morphology-slot selection or sentence meaning.

### 6.3 Productive L2

Productive L2 owns:

- lemma-to-paradigm compatibility;
- generation of licensed forms that are absent from the exact bank;
- same-lemma morphology-slot competition;
- local structural context evidence;
- honest same-lemma tie and abstention;
- preservation of separate cross-lemma basins for L3.

Productive L2 does not own destructive mutation authority.

### 6.4 L3

L3 owns full sentence and semantic competition. It receives the complete
bounded L2 lattice. It may resolve:

```text
посмотреть <-> посмотри
singular <-> plural under sentence agreement
homonymous lemmas under wider phrase context
```

L3 must not invent a morphology surface that was absent from the L2 logical
field. It may add separately attributed semantic candidates only through its
existing candidate contract.

### 6.5 Verifier

The verifier owns structural apply safety. It must not be weakened to
compensate for productive generation or ranking defects.

## 7. Canonical Memory Model

### 7.1 LemmaParadigmBinding

One lemma may be compatible with more than one paradigm. The binding therefore
must be one-to-many:

```text
LemmaParadigmBinding
  lemma_id
  paradigm_id
  canonical_source_form_ref
  observed_slot_mask
  positive_support
  explicit_anti_support
  stability
  flags
```

No field width is fixed by this design document. Record widths become
canonical only after format measurement and roundtrip proof.

### 7.2 ParadigmCenter

`ParadigmCenter` represents a learned inflection family, not a literal word:

```text
ParadigmCenter
  primary POS/domain mask
  compatible source slots
  target slot program range
  stem/allomorph program range
  positive phase profile range
  anti phase profile range
  ambiguity profile range
  calibration reference
  support and stability
```

A paradigm may cover regular declension, conjugation, comparative formation,
participle formation, or another measured productive family. Different
linguistic domains must not be merged only because one suffix is shared.

### 7.3 MorphologySlot

The complete slot identity is factorized into typed axes:

```text
POS
number
case
gender
person
tense
mood
aspect
voice
form kind
degree
animacy where applicable
```

Not every axis applies to every POS. Inapplicable values must have an explicit
typed representation; they must not be encoded as a coincidental numeric zero.

### 7.4 MorphEditProgram

The generator must support bounded structural edits, not only raw suffix
replacement:

```text
Copy source span
Delete bounded source edge span
Insert learned prefix segment
Insert learned infix or stem-alternation segment
Insert learned suffix segment
Reference exact irregular allomorph
Terminate
```

Programs are learned from aligned corpus paradigms and grouped by
`ParadigmCenter`. Runtime code must never branch on a literal lemma, word,
suffix fixture, damage class, or proof case.

### 7.5 Irregular and suppletive forms

Irregular evidence follows this order:

1. an existing exact canonical L2 form remains the strongest materialization;
2. a measured lemma-local allomorph program may reference the exact decoder;
3. an unsupported global analogy remains a tied low-authority candidate;
4. absence of sufficient evidence produces `ABSTAIN`, not a fabricated form.

An irregular exact exception does not become a global productive transform.

### 7.6 Phase banks

Each applicable morphology axis may own:

```text
positive subcenters
explicit anti-subcenters
ambiguity subcenters
hard-negative residual centers
```

The number and record width of subcenters must be selected by measured package,
quality, and ablation results. This document does not assign them manually.

## 8. Productive Form Graph

### 8.1 Logical field

Every licensed combination

```text
LemmaParadigmBinding
x compatible target MorphologySlot
x MorphEditProgram
```

defines one or more terminal form paths. The complete set of terminal paths is
the logical productive lattice.

The implementation may avoid materializing every UTF-8 string. It may not
silently remove a terminal path to meet latency.

### 8.2 Physical representation

The V1 representation is a compact prefix trie with shared:

- canonical source spans;
- retained stems;
- productive prefixes;
- stem alternation segments;
- suffix continuations;
- terminal decoder programs.

The same transform segment is stored once per applicable paradigm family, not
once per generated surface. Suffix-minimized or convergent FST nodes are not
allowed in V1 because OSA and phase state are emitted-prefix-dependent. A later
FST requires product-state traversal and exact parity proof.

### 8.3 One-to-one terminal contract

Every emitted terminal must have stable attribution:

```text
terminal_id
lemma_id
paradigm_id
target morphology slot
edit program id
decoder path
evidence source ids
```

Two terminal paths that decode to the same normalized surface may be retained
as distinct evidence identities until readout dedup. Surface dedup must not
erase lemma or paradigm attribution.

## 9. Shared Exact Geometry

### 9.1 Rejected approximate frontier

The following shape is forbidden:

```text
cheap approximate score
-> truncate forms
-> exact geometry only for survivors
```

Unless the approximate score supplies a formally admissible bound and exact
parity is proved, this shape can discard the correct target before geometry.

### 9.2 Exact OSA state over a trie

Let the observed sequence be `o[1..m]`. For a generated path prefix of length
`i`, the exact optimal-string-alignment row is `D_i[0..m]`:

```text
D_i[0] = i
D_0[j] = j

D_i[j] = min(
  D_(i-1)[j] + 1,
  D_i[j-1] + 1,
  D_(i-1)[j-1] + mismatch(g_i, o_j),
  D_(i-2)[j-2] + 1  when adjacent transposition is valid
)
```

Traversal state at a graph node contains:

```text
current row
previous row
previous generated unit
generated path length
normalization denominator state
phase/atom accumulator state
decoder path reference
```

Children of the same prefix reuse the parent rows. A branch copies bounded row
state, not the complete generated string and not a fresh full DP history.

### 9.3 Character and keyboard lanes

The traversal runs two exact typed lanes:

```text
Unicode scalar units
keyboard keycode + shift units
```

The final geometry remains equivalent to the existing maximum of normalized
character and keyboard similarities. Keyboard units for stored transform
segments should be prepared in the sidecar or updated incrementally during
traversal; they must not be rebuilt from a complete UTF-8 candidate for every
terminal.

### 9.4 Incremental atom and phase evidence

Character n-gram, keyboard n-gram, boundary, and phase features that depend on
the generated sequence should be updated along graph edges. The state may keep
the bounded suffix history required to form the next atom.

Terminal-only boundary features are applied only when a terminal is visited.

### 9.5 Exactness requirement

For an unchanged candidate set, the shared traversal must produce byte-exact
candidate identities and exact score parity with the V39 exhaustive reference.
An optimization is rejected if any fixed or exhaustive micro case differs.

## 10. Learned Evidence Algebra

### 10.1 Candidate energy decomposition

The intended decomposition is:

```text
E(form | observed, scene) =
    E_lemma
  + E_paradigm
  + E_slot
  + E_geometry
  + E_directional_residual
```

Interpretation:

- `E_lemma`: grounded L1.1 atom and wave evidence;
- `E_paradigm`: compatibility of the lemma with the learned paradigm and edit
  program;
- `E_slot`: positive, anti, and ambiguity phase response for the typed local
  scene;
- `E_geometry`: exact character/keyboard damaged-signal evidence;
- `E_directional_residual`: independently observed pair evidence for
  exceptions not explained by the factorized slot field.

This equation defines factors, not manually assigned coefficients. The paper
implementation makes it executable as a constrained train-only linear score
over centered typed evidence, with fixed-point runtime reduction.

### 10.2 Evidence learning

Each factor must be derived from train-only observations. V1 uses centered
smoothed log evidence and a deterministic L2-regularized pairwise logistic
objective. Supportive coefficients are non-negative; explicit anti evidence is
subtracted. Missing observation is neutral, not negative. Ranking coefficients
are frozen before disjoint calibration replay.

The following operation is forbidden:

```text
other slot was positive in the same context
-> therefore mark it anti for this target
```

Russian contexts are multi-label. For example, `они _` can admit several
tenses and form kinds. Anti-evidence must be explicit:

- a corrected competitor observation;
- a rejected or reverted generated form;
- a scene in which the competing slot was directly contradicted;
- a structurally impossible typed slot relation;
- a schema-grounded structurally impossible typed relation.

### 10.3 Factorized slot field

Primary coverage comes from typed morphology axes, not exact surface pairs.
The slot field may combine:

```text
context phase <-> number center
context phase <-> case center
context phase <-> person center
context phase <-> tense center
context phase <-> mood center
context phase <-> form-kind center
bounded learned interactions between applicable axes
```

This allows evidence to transfer to an unseen surface while preserving the
same morphology slot identity.

### 10.4 Directional residuals

Exact pair evidence remains a residual factor. It may settle a candidate pair
only when its source scene and direction satisfy the existing no-reverse-false
contract.

Absence of a pair record is no evidence against either candidate.

### 10.5 Cross-lemma ownership

L2 morphology evidence may settle forms within one lemma basin. It must not
erase another grounded lemma basin without independent lexical or L3 evidence.

Cross-lemma candidates therefore remain separately attributed through the L2
readout and enter L3 as a bounded lattice.

## 11. Readout And Authority

### 11.1 Ranked field

The runtime first produces a deterministic ordered field using the learned
energy factors and stable identity tie-breakers. Stable identity tie-breakers
may order output but may not create semantic authority.

### 11.2 Winner

`Winner` requires all of:

- one candidate has the highest learned energy;
- its margin over the strongest competitor passes a heldout-calibrated bin;
- no explicit anti or contradiction veto applies;
- the candidate retains complete lemma, paradigm, slot, and geometry
  attribution;
- the verifier contract remains satisfiable;
- the applicable class and ambiguity proof gates have passed.

### 11.3 Tied

`Tied` is required when:

- several forms remain compatible with the scene;
- the calibrated margin is insufficient;
- different grounded lemma basins survive;
- the context is genuinely multi-label;
- exact and generated forms are observationally equivalent.

### 11.4 ABSTAIN

`ABSTAIN` is required when:

- no licensed paradigm path survives;
- evidence is contradictory;
- an unknown irregular form lacks independent support;
- the logical lattice cannot be materialized within a proved bounded contract;
- required package or scene evidence is unavailable.

### 11.5 Calibration

The authority margin is learned after candidate energy training. Calibration
must report:

```text
support bin
margin bin
damage class
seen-exact vs unseen-generated
correct winner count
false winner count
tied count
abstain count
```

The threshold is not lowered to satisfy coverage. False authority and false
singleton remain conjunctive zero gates.

Unique top-1 is evaluated only on cases frozen as uniquely resolvable under the
available L2 scene. Genuine multi-label cases are owned by tied-set retention;
unsupported cases are owned by abstain and false-authority gates.

## 12. One-Pass Crystallization

### 12.1 Raw corpus contract

The compiler reads the morphology corpus once and emits typed events:

```text
lemma identity
surface identity
POS and complete morphology slot
scene features
neighbor observations
support and provenance
train / heldout split identity
```

If the corpus is not sorted by lemma, events enter a bounded sharded spool. The
raw corpus is still read once.

### 12.2 Train reduce and calibration replay

One deterministic train reduce builds:

```text
lemma form groups
paradigm signatures
lemma-to-paradigm bindings
edit programs
shared transform segment tries
slot positive centers
explicit anti centers
ambiguity centers
directional residuals
ranking coefficients
```

After coefficients are frozen, calibration examples are replayed from the typed
calibration spool to build authority and tie tables. The stages may use sorted
shards and bounded external storage. They must not rescan raw corpora separately
for each bank.

### 12.3 Paradigm induction

For each lemma group:

1. choose a deterministic canonical anchor form from evidence;
2. align observed forms to the anchor;
3. derive bounded typed edit programs;
4. create a paradigm signature over observed target slots;
5. group compatible signatures without merging conflicting POS domains;
6. retain multiple compatible paradigms when evidence is incomplete;
7. isolate irregular allomorphs as lemma-local evidence.

Heldout lemma names and all their exact surfaces must be excluded from the
training side of the corresponding generalization proof.

### 12.4 Determinism

The same input bytes, split seed, and compiler version must produce the same
package bytes. Determinism proof includes at least two clean builds and a
SHA-256 comparison.

## 13. Incremental Modification

The immutable base sidecar may be extended by append-only deltas:

```text
new lemma binding
new paradigm support
new explicit anti observation
new ambiguity observation
new directional residual
new calibration evidence generation
```

A delta must contain:

- base package fingerprint;
- generation number;
- typed record counts;
- source receipt range;
- deterministic payload hash;
- proof status;
- authority scope.

Runtime loading uses immutable base plus ordered read-only overlays. A delta
does not rewrite L1.1 or canonical L2 and does not gain authority before its
own differential proof.

## 14. Package Layout Requirements

The next sidecar format must separate:

```text
header and fingerprints
typed slot dictionary
ParadigmCenter bank
LemmaParadigmBinding bank
MorphEditProgram bank
shared segment/decoder pool
productive graph nodes and arcs
positive phase profiles
anti phase profiles
ambiguity profiles
directional residual profiles
calibration tables
append-only delta manifest references
```

Format requirements:

- mmap-safe fixed headers;
- checked offsets and lengths;
- no process-sized unpacked relation vectors;
- decoder strings materialized only for final readout;
- stable typed IDs;
- backward rejection with explicit version error;
- package fingerprint bound to the canonical L2 package;
- deterministic roundtrip and corruption tests.

No record width or package-size estimate is promoted before a measured package
exists.

## 15. Complexity Contract

### 15.1 Current upper shape

The existing bounded topology can approach:

```text
lemmas x target slots x generated forms x per-surface geometry
256    x 16           x 32              x sequence work
```

Actual runtime work is lower because family ranges and target slots are sparse,
but the representation still repeats shared prefix, conversion, allocation,
and DP operations.

### 15.2 Intended shape

For observed length `m`, active lemma set `A`, and each lemma's visited prefix
edges `E_l`, the shared graph should approach:

```text
observed input length x sum over active lemmas of visited prefix edges
+ bounded slot/phase evaluation
+ final terminal readout
```

This is a structural complexity target, not a measured latency claim.

### 15.3 Concurrency

The canonical first implementation should be deterministic without requiring
one Rayon task per lemma. Request concurrency and graph traversal concurrency
must have one owner.

Nested request-level and lemma-level pools are forbidden unless a later proof
shows bounded scheduling, fairness, candidate parity, and p99 improvement.

## 16. Proof Ladder

### 16.1 Existing-receipt failure audit

Before implementation, partition every existing fixed failure by the first
loss point:

```text
lemma absent
paradigm incompatible
target slot absent
target form not generated
target outside top-16
target lost by readout
target retained but tied
wrong unique top-1
false authority
```

This audit uses existing receipts and changes no runtime authority.

### 16.2 Geometry exactness proof

Compare shared traversal with the V39 exhaustive reference over:

- every pair of small ternary strings through a fixed exhaustive length;
- character and keyboard lanes;
- insertion, omission, substitution, and adjacent transposition;
- real productive candidate sets;
- normalization and boundary behavior;
- reused-prefix branches and irregular decoder segments.

Required result: exact equality for every candidate score and ordering input.

### 16.3 Candidate identity proof

In the speed-only mode, the new graph must enumerate the same logical candidate
identities as V39 before paradigm ranking changes are enabled.

Required result: candidate identity hash parity `100%`.

### 16.4 Paradigm generalization proof

Use disjoint lemma cohorts:

```text
train lemmas
calibration lemmas
heldout lemmas
```

No surface or lemma identity from heldout may enter paradigm or slot training.
Report regular and irregular cohorts separately.

### 16.5 Fixed damage proof

Both seen-exact and truly unseen-generated forms run through all 13 fixed
damage classes. Report every class independently:

```text
unique top-1
top-16 retention
readout retention
tied coverage
abstain
false authority
false singleton
```

### 16.6 Clean and ambiguity proof

Required separate denominators:

- clean exact surfaces;
- clean generated-looking but valid surfaces;
- syncretic morphology slots;
- homonymous lemmas;
- multi-label contexts;
- unsupported irregular forms;
- cross-lemma geometric neighbors.

### 16.7 Ablation proof

Measure at least:

```text
without ParadigmCenter compatibility
without slot positive centers
without explicit anti centers
without ambiguity centers
without directional residuals
without exact geometry
```

An ablation must expose the independent contribution of each bank. A component
with no measured effect is not retained merely because it is architecturally
plausible.

### 16.8 Performance proof

Report separately:

```text
cold load
first request
single active request p50 / p95 / p99 / max
admitted multi-client p50 / p95 / p99 / max
CPU time per request
wall time
steady RSS
peak RSS
major faults
package bytes
```

The benchmark must state both outer request workers and internal traversal
workers. `workers=1` must not be described as one physical CPU unless the
internal pool is also one.

It must additionally record CPU model, governor, affinity, NUMA topology,
arrival pattern, queue depth, warmup, request count, package/binary hashes, and
hot/cold page-cache state.

### 16.9 Product proof

Only after offline gates pass:

```text
L1.1 lattice
-> productive L2
-> L3
-> L4 / DecisionCore
-> verifier
-> daemon and IBus shadow
-> physical WeChat / Telegram / Chromium / GTK / Qt / Kitty matrix
```

No global IBus restart occurs before the installed binaries and packages pass
their read-only health gate.

## 17. Promotion Gates

All gates are conjunctive:

```text
generated unique top-1, every damage class       >95.0%
generated top-16, every damage class              >95.0%
generated readout retention, every damage class   >95.0%
clean preservation                                >=99.9%
ambiguity retention                               >=99.0%
grounded L1.1 candidate loss                            0
false authority                                         0
false singleton                                         0
geometry reference parity                           100%
candidate identity parity in speed-only mode        100%
single-request hot p99                             <=5 ms
admitted multi-client p99                         <=5 ms
productive sidecar bytes                    <=81,688,382
steady RSS KiB                                 <=314,888
peak RSS KiB                                   <=337,016
cold package publish                           <=1,000 ms
deterministic package hash parity                     PASS
daemon/IBus physical matrix                           PASS
```

Aggregate top-1 cannot hide a failing class. Compression parity cannot prove
quality. A generated candidate remaining somewhere in top-64 cannot replace
the required top-1, top-16, and readout denominators.

The three top-1/top-16/readout gates apply to `UNIQUE_RESOLVABLE` cases. Genuine
`MULTI_LABEL` cases apply the ambiguity and false-singleton gates instead.

## 18. Seven-Step Delivery Route

### Step 1: restore the measured baseline

- preserve V42 receipts;
- remove only V42 bounded chunk code;
- restore V39 source behavior;
- rerun focused local tests;
- do not build or install a runtime package.

Exit: source matches the documented V39 baseline and V42 remains a receipt-only
rejected experiment.

### Step 2: partition failures

- derive first-loss buckets from existing receipts;
- report counts and classes;
- confirm or reject the paradigm/slot/readout hypotheses.

Exit: every fixed failure has one first-loss owner.

### Step 3: crystallize the paradigm sidecar

- implement one raw corpus pass;
- learn paradigms and edit programs;
- compile factorized slot sufficient statistics;
- prove deterministic package bytes;
- keep authority disabled.

Exit: compact sidecar with format and corpus receipts.

### Step 4: implement shared exact traversal

- add exact graph traversal;
- preserve V39 candidate identities in speed-only mode;
- prove character/keyboard geometry parity;
- remove independent per-surface DP from the canonical path only after parity.

Exit: exact parity and a measured performance result.

### Step 5: train and calibrate slot authority

- train positive, explicit anti, and ambiguity centers;
- add paradigm compatibility;
- retain pair relations as residual evidence;
- calibrate `Winner | Tied | ABSTAIN` on a separate cohort.

Exit: strict fixed quality and zero false-certainty gates pass.

### Step 6: complete offline and integrated proof

- larger fixed denominator;
- clean and ambiguity gates;
- L3 handoff;
- CPU/RSS/package telemetry;
- daemon/IBus shadow parity.

Exit: every gate in section 17 has authoritative evidence.

### Step 7: release

- version synchronization;
- changed and release gates through `scripts/cargo-guard.sh`;
- remote release build;
- package and binary SHA parity;
- atomic installation;
- health/PID verification;
- physical multi-client matrix;
- architecture/receipt/graphify update;
- commit and public push.

Exit: installed runtime, public source, documentation, and receipts describe the
same promoted version.

## 19. Rejected Routes

The following routes are explicitly rejected unless new evidence reopens them:

- manual word, phrase, suffix, application, or fixture conditions;
- fixing proof failures one case at a time;
- reducing `256 / 256 / 16 / 32 / 196608` to meet latency;
- approximate prefilter without an admissible-bound parity proof;
- treating another positive morphology slot as anti-evidence;
- completing every form pair in a quadratic directional table;
- per-lemma nested Rayon as the main runtime architecture;
- increasing caches without a measured reuse owner;
- moving unsafe candidate-generation defects into the verifier;
- promoting generated forms because aggregate top-1 improved;
- claiming one-worker CPU behavior from `--workers 1` while inner Rayon remains
  active;
- recrystallizing L1.1 or canonical L2 for this sidecar redesign.

## 20. Paper Decisions Closed

The twelve former open decisions are resolved normatively in:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-implementation.md`.

The fixed resolutions are:

1. 16-byte typed slot key with distinct inapplicable and unknown values.
2. Seven bounded scalar edit instructions with package-derived maxima.
3. Exact subset compatibility for incomplete paradigms, retaining all matches.
4. Distinct variant identities inside one complete morphology slot.
5. Prefix trie only for V1; no convergent FST geometry state.
6. Complete-prefix OSA state in a per-request DFS arena.
7. Incremental fixed-point typed atom state with bounded prefix/tail units.
8. Constrained train-only linear score with a stated pairwise objective.
9. Disjoint-lemma calibration with zero-error authority and PAVA tie envelope.
10. Immutable additive deltas, explicit supersedes, atomic model generations.
11. Little-endian package V1 with fixed record widths and checked mmap views.
12. One service-level `MorphExecutor`; nested Rayon is forbidden.

No implementation should guess outside these decisions. A required deviation
must first update the paper specification and state its proof obligation.

## 21. Evidence And Receipts

Canonical architecture and plan:

```text
/home/ubu/projects/lay/docs/l2-l11-canonical-architecture.md
/home/ubu/projects/lay/docs/l2-l11-candidate-field-contract.md
/home/ubu/projects/lay/docs/lay-development-plan-after-0.2.340.md
/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-implementation.md
/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-review-2026-08-10.md
```

Current productive baseline:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SMALL_DEDUP_V39_2026-08-10/
```

Rejected V40/V41:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LOWER_KEY_UNITS_V40_2026-08-10/
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_COMMON_EDGES_V41_2026-08-10/
```

Rejected V42:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_LEMMA_CHUNKS_V42_2026-08-10/
```

Directional evidence:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_RAW_V20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_TWO_LANE_V21_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_EXACT_V22_13X10_2026-08-10.json
```

The exact raw V39 profile remains on the remote proof host:

```text
/home/e/build/lay-l1-shadow/artifacts/l2-productive-v39-small-dedup-2026-08-10/perf.data
```

## 22. Runtime Authority

This document changes no runtime authority.

Current state:

```text
productive generated forms       SuggestOnly
L1.1 package                      unchanged
canonical L2 package              unchanged
daemon                            unchanged
IBus                              unchanged
installed version                 unchanged
public branch                     unchanged
```

The first permissible authority change is after all gates in section 17 pass
and the physical product matrix proves the installed route.

## 23. V64 Surface-Basin Evidence

V64 implemented the paper-approved output equivalence relation before global
top-32:

```text
complete compatible bindings
-> execute selected slots
-> coalesce (lemma_id, target_slot_id, normalized_surface)
-> preserve deterministic representative plus equivalence/support metadata
-> bounded 32-basin lattice
```

Different slots remain different basins under syncretism. Raw corpus,
transition induction, `16 / 32` bounds, learned coefficients, authority gates,
SafetyGate, and verifier were unchanged. The package stayed `17,309,944 B` and
was byte-deterministic across two resumes.

The fixed `13 x 100 x 2` proof measured the previously unknown boundary:

```text
LEMMA_HELDOUT cases  1,300
H                    1,280
B                    1,219
S0                   1,219
S1                   1,219
S2                   1,219
S3                   1,219
R                    1,219
top-16               1,218
raw unique top-1       267
```

V63-to-V64 exact birth increased `1,197 -> 1,219` and top-16 increased
`1,175 -> 1,218`. Therefore the surface-basin mechanism is accepted and global
duplicate crowding is closed for this denominator. A later proof audit bound
oracle identity to `(lemma, POS, paradigm)`, eliminating cross-POS credit. The
authoritative first losses are `20` outside target-POS `H`, `61` at `H -> B`,
and `0` at `B -> S0`; there are zero measured losses across
`S0 -> S1 -> S2 -> S3 -> R`. Of the `61` binding losses, `59` have no oracle
paradigm in the remaining source-slot postings and `2` fail exact exposed-form
reconstruction.

Promotion remains rejected. All `2,600` verdicts are `ABSTAIN`, false singleton
and integrity errors are zero, probed/unprobed parity is `2,600 / 2,600`, but
the strict per-class quality and `5 ms` latency gates fail. Runtime authority,
installed daemon/IBus, and public release are unchanged.

Exact evidence:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/`.

The paper-approved successor micro is the sparse reverse-anchor recovery lane:

`/home/ubu/projects/lay/docs/l2-productive-post-v64-anchor-recovery-paper.md`.
