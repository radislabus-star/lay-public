# L1.1 Exact Typed Peak Search: Detailed Refactoring Plan

Status: Phases 0-6 complete; Phase 7 is the next admitted boundary
Date: 2026-08-13  
Scope: `src/nanda_wave/lexical_grokking/` only, plus proof tooling and owned documentation

## 1. Decision

The most promising L1.1 route is:

```text
typed positional atom field Q
-> one L1PeakSearch owner
   -> complete observed-atom posting streams
   -> typed edit-graph states
   -> one evidence accumulator keyed by WordCenterId
   -> exact nonlinear settlement over the proven competitive closure
   -> completeness certificate
-> Winner | Tied | ABSTAIN
```

The central change is not "remove words from the output". L1.1 must still return
`WordCenterId`, because a decoder needs an addressable lexical surface. The change
is:

> A bounded candidate list must no longer be an heuristic input to wave
> settlement. The bounded lattice must be the result of an exact, certified
> search for local peaks in the crystallized field.

Posting expansion and typed edit traversal are two search-node kinds owned by
one `L1PeakSearch`. They are not two independent candidate producers, scorers,
or authority routes.

Implementation must not begin until the measured baseline, target semantics,
and implementation preflight all pass their paper gates.

## 2. Scope

### 2.1 Included

- Freeze the current L1.1 package, source, proof denominator, output, and latency.
- Split the 4,565-line `runtime.rs` by ownership without behavior changes.
- Introduce one internal `L1PeakSearch` contract with the current route as
  `LegacyBirthSearch`.
- Build a proof-only dense oracle over all `WordCenterId` values.
- Build a typed edit-product graph that traverses states, not generated strings.
- Search complete observed-atom postings with sound block bounds.
- Merge all evidence in one accumulator keyed by `WordCenterId`.
- Preserve the existing nonlinear settlement and restoration classifier first.
- Prove optimized search parity with the dense oracle.
- Promote only after all quality, safety, package, RSS, and latency gates pass.
- Remove the legacy birth route after the owner flip.

### 2.2 Excluded

- No word-, suffix-, phrase-, language-example-, source-ID-, or test-name-specific
  runtime branches.
- No changes to L2 morphology, L3 context, L4 feedback, `DecisionCore`,
  `SafetyGate`, verifier, IBus replay, Space handling, or double-Shift rollback.
- No attempt to generate a word form that has no addressable `WordCenterId`.
  Productive unseen morphology remains an L2 contract.
- No ANN index, embedding nearest-neighbor route, full live dictionary scan, or
  materialization of thousands of corrected strings per query.
- No package-format change before an in-memory proof demonstrates exactness and
  useful pruning.
- No daemon shadow worker and no second per-keystroke decision route.
- No version bump for a failed paper or shadow experiment.

## 3. Current Measured Facts

These are historical measured points, not yet the new canonical baseline. Phase
0 must select one immutable package and rerun the required projections on the
exact source snapshot.

| Fact | Measured value | Evidence |
|---|---:|---|
| `runtime.rs` | 4,565 lines | current source |
| `proof.rs` | 3,303 lines | current source |
| wave dimension | 128 real + 128 imaginary cells | current source |
| birth atoms per channel | default 4, maximum 32 | current source |
| posting budget | 131,072 relations | current source |
| reconstruction scan | 8,192 centers | current source |
| geometry scan | 1,024 centers | current source |
| main phase frontier | 128 centers | current source |
| settling iterations | 3 | current source |
| full `birth=32` | 34.531 ms p99, sparse omission 94.2635%, 12/13 | memory-layout document |
| operator-aware V7 | 762,314 centers, 66.11 MiB, 3.146 ms p99, 13/13 | V7 final receipt |
| V8 full posting field | 108,156,559 forward relations | first-touch receipt |
| V8 package used by first-touch proof | 189.05 MiB | first-touch receipt |
| V8 accepted fresh-process hot p99 | 4.397 / 4.917 / 4.865 ms | first-touch receipt |

Important evidence paths:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_OPERATOR_AWARE_V7_FINAL_762314_2026-07-26.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_FIXED_13X20000_2026-08-02.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_FIRST_TOUCH_PHASE6_2026-08-02.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_V8_GROUPED_CACHE_RUNTIME_2026-07-30.json
```

The current route truncates before final interference:

```text
Q
-> select at most 4 atoms per lexical channel
-> stop after at most 131,072 posting relations
-> rank touched centers by forward activation
-> 8,192 reconstruction scan
-> 1,024 geometry scan
-> 128 main phase frontier plus bounded reserves
-> reverse wave and nonlinear settlement
-> geometry / position / ambiguity / pairwise effects
-> bounded lattice
```

This architecture can lose the correct center before the mechanisms designed to
recognize it are executed. Raising 4 to 32 does not repair the architecture: it
raises work sharply, still retains later truncation boundaries, and has already
failed the strict quality and latency contract.

## 4. Target Semantics

### 4.1 Identity

The following identities remain distinct:

```text
NGramKey       identity of a typed atom
AtomId         dense package address of that atom
WordCenterId   identity of an addressable lexical attractor
DecoderNodeId  identity of a node in the lexical surface trie
SearchStateId  transient query-local edit-product state
```

A hash is address material only. A score, wave vector, trie node, or proof label
must never become word identity.

### 4.2 Query field

`L1QueryField` is built exactly once:

```text
L1QueryField
+-- normalized surface metadata
+-- typed positional atom observations
+-- character anchor sequence
+-- physical keyboard sequence
+-- script flags
+-- 128D complex surface wave
`-- query fingerprint for diagnostics/certificates
```

The current atom channels and weights remain unchanged during the refactor. A
scoring change cannot be hidden inside query extraction.

### 4.3 Product topology

The edit geometry is a sparse layered product graph, not a flat metric and not a
single 128D vector:

```text
state = (
  DecoderNodeId,
  input_position,
  typed_operator_state,
  position_certificate,
  accumulated_geometry
)
```

Edges represent typed transitions:

```text
identity
insertion / extra input symbol
deletion / missing target symbol
character substitution
keyboard substitution
keyboard-layout projection
adjacent transposition
non-adjacent transposition state
prefix truncation
suffix truncation
repeated-fragment state
punctuation suffix
omission + transposition
sparse multi-omission
```

The topology preserves operator identity and position. Two states may be merged
only when a proven dominance relation says that every future continuation from
one is no better and carries no distinct authority certificate. Equal scalar
edit distance is not sufficient for merging.

No corrected surface strings are created during traversal. A terminal state
emits `WordCenterId` plus typed evidence.

### 4.4 Competitive closure

Let `W` be all package word centers and `Q` the query field.

For every `w in W`, define independent evidence `I(Q,w)` containing forward,
reverse, positive, sequence, length, position, script, and typed geometry
components before candidate-relative interference.

Define index-independent score bounds for every possible completed competitive
set `C` that contains `w`:

```text
L(Q,w) <= FinalScalar(Q,w,C) <= U(Q,w)
```

`U` includes every still-possible constructive term at its conservative maximum.
`L` includes every bounded destructive term at its conservative maximum pressure.
Geometry, exact reconstruction, and position-based reorderings are not collapsed
into this scalar interval; they are handled by the separate typed set `D(Q)`.
The dense oracle computes `I`, `L`, and `U` directly for every center. These are
semantic bounds; posting-block metadata must conservatively dominate them but
does not define them.

Let `K` initially remain the frozen current phase capacity
`max(MAX_PHASE_FRONTIER, requested_limit)`. For any evaluated set `E` containing
at least `K` centers, let `beta_K(E)` be its `K`-th largest `L(Q,w)`. An unseen
node can be pruned only when every center below it has `U(Q,w) < beta_K(E)`.
Equality must be expanded. At certified termination, `beta_K(E)` is also the
global `K`-th lower-bound value because every unseen center has
`L <= U < beta_K(E)`.

Define:

```text
A(Q) = every center whose U(Q,w) can reach or tie the certified global
       beta_K; this is the complete scalar competitive superset

D(Q) = every terminal that can enter or alter readout through a non-scalar
       exact-reconstruction, geometry, position, layout, operator, or required
       ambiguity-shell certificate; this includes the complete minimum typed
       geometry basin and every lease-eligible neighboring basin

S(Q) = A(Q) UNION D(Q) UNION exact-surface collisions

R(Q) = typed evidence dependencies reachable from S(Q): ambiguity, anti, and
       pairwise records needed to settle or classify members of S(Q)
```

Relation records in `R(Q)` do not birth candidates. Anti and pairwise relations
act only when both word-center endpoints were independently admitted to `S(Q)`.
An ambiguity relation may mark an active owner as unresolved when its competitor
is not independently admitted, matching the current safety behavior, but it does
not promote that competitor into `S(Q)` merely because an edge names it.

The existing nonlinear settlement is run exactly over `S(Q)` while reading the
required records from `R(Q)`. The optimized search is correct only when it
returns the same `S(Q)`, the same typed dependency state, and the same final
readout as a dense oracle that evaluates the definition directly.

### 4.5 Why scalar WAND is insufficient

Final behavior is lattice-relative:

- `forward_milli` is normalized by a maximum;
- structural interference uses maximum surface and keyboard hit counts;
- geometry and position certificates can reorder candidates;
- ambiguity can turn a singleton into a basin;
- pairwise evidence depends on which centers are present;
- equality at a cutoff can change `Tied`, not only top-1.

Therefore a bound only on current `settled_energy` is unsound. Posting/edit
search metadata must upper-bound the index-independent `U(Q,w)` of every center
under the node. Retained centers must also expose valid lower bounds; otherwise
there is no threshold against which an unseen upper bound can be pruned. Every
search node must expose a conservative envelope:

```text
SearchEnvelope
+-- maximum raw forward mass
+-- maximum possible base energy
+-- maximum surface and keyboard hits
+-- minimum possible typed geometry
+-- possible reconstruction/operator modes
+-- can_be_exact_reconstruction
+-- can_change_normalization
+-- can_enter_pairwise_top8
+-- can_join_or_lower_the_geometric_basin
`-- can_hide_a_required_typed_dependency
```

Negative anti and pairwise pressure may be treated as zero in an upper bound,
because they only reduce energy. Positive, phase, sequence, structural, position,
and geometry effects must be included at their conservative maxima or expanded
exactly.

A node may be pruned only when it cannot do any of the following:

1. enter the required independent frontier;
2. tie the cutoff;
3. lower or join the geometric basin and shell;
4. change a normalization maximum;
5. obtain an exact/operator certificate that changes ordering;
6. enter pairwise top-8;
7. hide a relation record required to settle or classify an admitted endpoint.

Equality never permits pruning. It must be expanded to preserve ties.

### 4.6 Completeness result

```rust
enum SearchCompleteness {
    Certified(CompletenessCertificate),
    Unresolved(UnresolvedReason),
}
```

`Unresolved` may produce diagnostic suggestions, but it cannot produce edit
authority. The authoritative result is `ABSTAIN`. There is no silent fallback to
the legacy route.

The certificate must bind at least:

```text
package SHA-256
query fingerprint
search semantics version
expanded/skipped posting nodes
expanded/dominated edit states
maximum unseen independent envelope
certified K-th retained lower bound
minimum unseen geometry lower bound
normalization maxima completeness
typed relation dependency completeness
cutoff and tie completeness
final lattice fingerprint
```

## 5. Ownership And Dependency Tree

Target source tree after the move-only and algorithmic phases:

```text
src/nanda_wave/lexical_grokking/
+-- runtime.rs                       stable facade, no algorithm body
+-- runtime/
|   +-- contract.rs                  query/result/internal evidence types
|   +-- diagnostics.rs               query/benchmark JSON and traces
|   +-- relations.rs                 V7/V8 relation access and caches
|   +-- legacy.rs                    frozen current birth/frontier route
|   +-- settlement.rs                existing exact nonlinear settlement
|   +-- geometry.rs                  typed distances and certificates
|   +-- host.rs                      package/composite host and overlays
|   `-- tests.rs                     moved runtime invariants
+-- peak_search/
|   +-- mod.rs                       sole L1PeakSearch owner/orchestrator
|   +-- postings.rs                  complete posting streams and bounds
|   +-- edit_graph.rs                typed product-graph traversal
|   `-- certificate.rs               dependency and completeness proof
+-- proof.rs                         existing proof facade
`-- proof/
    +-- peak_oracle.rs               dense proof-only oracle
    `-- peak_matrix.rs               stratified position/class comparison
```

This is a target ownership map, not a command to create every file immediately.
A file is created only when its boundary owns distinct state or invariants. No
`utils.rs`, `helpers.rs`, or `common.rs` is allowed.

Dependency direction:

```text
diagnostics / host facade
        |
        v
L1PeakSearch owner
   |          |
   v          v
postings   edit_graph
   \          /
    v        v
 WordCenter accumulator
          |
          v
 frozen settlement + geometry
          |
          v
 Winner | Tied | ABSTAIN + completeness

proof oracle -> may call internal pure kernels
runtime core -X-> must never call proof fixtures or oracle labels
```

Only `L1PeakSearch` owns search completion. `postings.rs` and `edit_graph.rs`
emit evidence; they do not rank, classify, authorize, or fall back.

## 6. Internal Contracts

The exact names may change during paper preflight, but the ownership must not.

```rust
struct L1QueryField {
    normalized: NormalizedSurface,
    atoms: Box<[ObservedAtom]>,
    character_anchors: AnchorSequence,
    physical_keys: Box<[u32]>,
    script_flags: u8,
    surface_wave: ComplexSurfaceWave,
    fingerprint: u64,
}

struct TerminalEvidence {
    terminal_id: u32,
    activation: ForwardActivation,
    geometry: TypedGeometryEvidence,
    source_mask: EvidenceSourceMask,
}

struct PeakSearchResult {
    candidates: Vec<GrokkingCandidate>,
    restoration: RestorationReadout,
    completeness: SearchCompleteness,
    metrics: SearchMetrics,
}

trait L1PeakSearch {
    fn search(
        &self,
        field: &L1FieldView<'_>,
        query: &L1QueryField,
        request: ReadoutRequest,
    ) -> PeakSearchResult;
}
```

The accumulator should retain the existing dense epoch-scratch pattern:

```text
Vec<TerminalEvidence> indexed by WordCenterId
+ epoch array
+ touched WordCenterId list
```

It must not allocate a `HashMap` per input. Evidence combination must be
commutative, deterministic, and permutation-invariant. Operator certificates are
unions/minima/maxima with explicit rules, not last-writer-wins flags.

## 7. Detailed Work Plan

Every phase has an input, work product, proof gate, rollback boundary, and stop
condition. A phase cannot borrow a later gate.

### Phase 0 - Park live authority and pin the baseline

**Input**

- current dirty source tree;
- currently installed binary and package manifest;
- historical V7/V8 receipts;
- fixed heldout generator and latency surfaces.

**Work**

1. Do not edit or restart the installed daemon/IBus route.
2. Create an isolated implementation worktree only after identifying which dirty
   changes belong to the required L1.1 baseline.
3. Record `HEAD`, tracked-file content root, binary hash, package path/hash/size,
   corpus fingerprint, terminal count, relation counts, and package format.
4. Export and hash the fixed `13 x 20,000` heldout denominator.
5. Export and hash the fixed class-balanced latency set.
6. Record runtime workers, affinity, posting cache, reverse cache, warm profile,
   CPU model, and kernel version.
7. Capture exact result projections for `Full` and all existing ablation modes.
8. Keep time fields outside equality projections.

**Output**

```text
docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_BASELINE_<date>/
+-- baseline.json
+-- source-files.sha256
+-- package.sha256
+-- heldout.sha256
+-- latency-surfaces.sha256
`-- candidate-projection.sha256
```

**Gate**

- Every byte-bearing input is pinned.
- Installed authority is explicitly `unchanged`.
- Combined RU+EN and split-language denominators are separately identified.
- No historical metric is substituted for a missing current measurement.

**Rollback**

No runtime mutation occurs. Delete only the isolated worktree if the baseline is
invalid; preserve the failed receipt.

**Stop**

- unknown active package;
- source/package mismatch;
- heldout hash drift;
- unresolved dirty-tree ownership;
- conflicting RSS or latency contract not resolved in the receipt.

### Phase 1 - Freeze observable behavior

**Input**

The Phase 0 baseline.

**Work**

1. Define a stable candidate projection containing every semantic candidate
   field, order, restoration verdict, and safety effect.
2. Define stable JSON projections for query, restore, benchmark, service lattice,
   and host stats outputs.
3. Add a deterministic replay harness over clean, damaged, ambiguous, collision,
   RU, EN, and cross-layout inputs.
4. Add candidate-order permutation tests.
5. Pin allocation, RSS, and latency measurements separately from semantic hashes.

**Output**

- behavior fingerprint command;
- baseline projection hashes;
- explicit ignored dynamic fields;
- targeted test list for each later move.

**Gate**

The unchanged code reproduces its own Phase 0 fingerprints twice.

**Rollback**

Remove only the proof harness change. Package and live runtime remain untouched.

**Stop**

Any nondeterministic semantic field must be explained and removed from decision
authority before code movement starts.

### Phase 2 - Move-only split of `runtime.rs`

No score, constant, call order, allocation strategy, visibility, output schema,
or runtime owner may change in this phase.

Cuts are performed one at a time:

```text
2A diagnostics and environment/config readers
2B internal evidence/result types
2C relation store, V8 access, posting and reverse caches
2D L1RestorationHost and composite overlay route
2E pure geometry/operator kernels
2F legacy prepare/discovery/frontier logic
2G settlement, interference, finalization, and classifier bridge
2H inline runtime tests
```

After every cut:

```text
scripts/cargo-guard.sh --status
scripts/check-lay-changed.sh
targeted lexical_grokking tests through scripts/cargo-guard.sh
behavior fingerprint comparison
git diff --check
```

**Code budget**

- Public exports changed: `0`.
- Runtime semantic LOC growth: at most `5%` for visibility and adapters.
- No new source file without one named owner from Section 5.
- No new per-query allocation.
- No module above roughly 1,200 lines after the split unless a measured ownership
  reason is documented.

**Gate**

- semantic projection parity: exact;
- candidate order and all fields: exact;
- package bytes: unchanged;
- full proof denominator: unchanged;
- three-run p99 regression: no more than 5%;
- runtime authority: unchanged.

**Rollback**

Each cut is a separate commit or patch. Revert only the failing cut inside the
isolated worktree.

**Stop**

- any behavior difference;
- algorithm change hidden as cleanup;
- a new cyclic dependency;
- proof code imported by runtime;
- old and new implementations both executed on the hot path.

### Phase 3 - Introduce one internal owner contract

**Work**

1. Add `L1QueryField`, `ReadoutRequest`, `PeakSearchResult`, and
   `SearchCompleteness` internal types.
2. Add one `L1PeakSearch` interface.
3. Wrap the existing route as `LegacyBirthSearch` without altering it.
4. Make `LexicalGrokkingMemory::readout` delegate once to that owner.
5. Keep all external methods and JSON outputs unchanged.

There must be exactly one production call per readout:

```text
LexicalGrokkingMemory::readout
-> selected L1PeakSearch owner
-> result
```

**Gate**

- Phase 2 parity remains exact;
- route-gate reports one execution owner and one authority owner;
- no fallback, fan-out, double scoring, or duplicate normalization.

**Rollback**

Remove the internal interface and reconnect the unchanged legacy function.

**Stop**

Any design in which postings, edit traversal, settlement, and fallback can each
return an independent winner.

### Phase 4 - Define and implement the dense oracle

The oracle is proof-only and cannot be compiled into normal runtime authority.

**Work**

1. Decode or address every `WordCenterId` in the selected package.
2. Compute exact independent evidence for every center.
3. Compute complete typed geometry for the declared operator budget.
4. Compute exact `L/U` intervals and the global `beta_K`, then construct `A(Q)`,
   `D(Q)`, `S(Q)`, and typed dependency set `R(Q)` directly.
5. Run the frozen nonlinear settlement and restoration classifier.
6. Emit a deterministic oracle result and component trace.
7. Verify the oracle on tiny lexica where every intermediate value can be fully
   enumerated.

**Oracle output**

```text
query
target, if proof-only heldout evaluation supplies one
all independent maxima
all exact per-center lower/upper bounds
global beta_K and equality set
minimum typed geometry
competitive set WordCenterIds
typed dependency records and endpoint admission roles
final candidate vector
Winner | Tied | ABSTAIN
result fingerprint
```

Heldout targets are used only after readout to score the result. They never
enter oracle search, bounds, closure, or settlement.

**Gate**

- exhaustive enumeration confirmed on tiny packages;
- candidate iteration permutation parity exact;
- heldout target removal does not change oracle bytes;
- oracle is unreachable from daemon/service production entrypoints.

**Rollback**

Delete the proof-only module. No package or runtime behavior changed.

**Stop**

- target label influences retrieval;
- dense oracle semantics still depend on legacy frontier truncation;
- candidate-relative normalization or typed dependency behavior remains undefined.

### Phase 5 - Build the positional and class matrix

The matrix diagnoses mechanisms, not individual words.

Dimensions:

```text
language       RU | EN
length         2-4 | 5-8 | 9-16 | 17-32 symbols
frequency      head | middle | tail
position       0% | 25% | 50% | 75% | 100%
pair position  every ordered populated pair of the five bins
class          all fixed 13 damage classes
ambiguity      objectively unique | objectively tied
```

For every populated stratum, use a deterministic quota and record empty strata
rather than silently changing the denominator. Training and heldout seeds remain
disjoint.

Every failure is assigned to its first loss boundary:

```text
query encoding
posting evidence availability
typed edit reachability
independent frontier eligibility
typed dependency resolution
nonlinear settlement rank
restoration authority
```

**Gate**

- target generation and scoring are separated;
- all 13 classes and both languages are visible separately;
- no case-specific runtime fix is permitted from this matrix;
- failures are grouped by first shared mechanism.

**Rollback**

Matrix generation is proof-only.

**Stop**

Denominator changes after seeing results, literal-word runtime branches, or
reporting aggregate accuracy without per-class values.

### Phase 6 - Build the proof-only forward lexical index

The current decoder stores `(parent, symbol)` and supports terminal-to-root
decoding. It does not expose a ready child-transition index.

**Work**

1. Build child counts from every `DecoderNode.parent`.
2. Prefix-sum counts into child offsets.
3. Fill child IDs and sort each parent's children by symbol and child ID.
4. Build terminal lists keyed by `DecoderNodeId` from primary centers only.
5. Validate parent bounds, acyclicity, unique transition symbols, and terminal
   round-trip parity.
6. Measure node count, build time, and RSS before selecting any package format.

Do not confuse `WordCenter64.decoder_terminal` meanings in different banks:
primary centers refer to decoder nodes, while relation centers may use the same
field to refer to peer word centers. The index builder accepts only primary
centers.

**Gate**

- every primary terminal round-trips to the identical UTF-8 surface;
- every trie transition is deterministic;
- no package bytes changed;
- proof index is not loaded by normal runtime.

**Rollback**

Remove the proof index.

**Stop**

Invalid parent topology, terminal collisions not represented explicitly, or
unbounded startup/RSS projection.

### Phase 7 - Implement typed edit traversal in shadow proof

Implement by operator mechanisms, not example lists:

```text
7A identity, punctuation, prefix and suffix boundaries
7B one insertion, deletion, character substitution, keyboard/layout projection
7C adjacent and non-adjacent transposition, repeated fragment
7D double substitution, omission+transposition, sparse multi-omission
```

For each family:

1. Define the automaton states and allowed transitions on paper.
2. Define dominance and state deduplication.
3. Compare terminal set and typed certificate with the dense oracle.
4. Rerun the entire class/position matrix, not selected examples.
5. Record state expansions, queue peak, terminal emissions, and missed terminals.

**Gate**

- terminal-set recall against oracle: 100% within the declared geometry budget;
- operator certificate parity: exact;
- state-order permutation parity: exact;
- generated runtime strings: 0;
- literal test-word branches: 0.

**Rollback**

Remove only the failing operator-family patch. Earlier proved mechanisms remain
shadow-only.

**Stop**

- scalar-cost state merging loses an operator identity;
- queue bound drops a reachable terminal;
- language-specific examples become runtime conditions;
- traversal is only fast because completeness was silently reduced.

### Phase 8 - Design and prove complete posting bounds

Start with the current V8 complete posting field. Do not rebuild the corpus.

**Work**

1. Create a proof adapter that can enumerate every current compressed posting
   block and compute its exact observed maximum.
2. Define conservative block envelopes for the current query atom, weight,
   channel, and position range.
3. Evaluate at least these physical index choices before changing format:

```text
A. in-memory superblock directory over current V8 bytes
B. package-neutral terminal-range grouping during repack
C. compact skip directory every 128 or 256 posting relations
D. decoder-node breadth-first reindex for child-contiguous traversal
```

4. Run best-first search over both node kinds:

```text
PostingBlockNode | TypedEditStateNode
```

5. Compare every skipped node's predicted envelope with the oracle's exact
   maximum.
6. Report bound tightness before attempting hot-path integration.

Required diagnostics:

```text
posting relations total / decoded / skipped
posting blocks total / expanded / pruned
edit states generated / dominated / expanded
WordCenterIds touched / exactly settled
typed dependency records resolved / unresolved
certificate success rate
p50 / p95 / p99 by language, length, frequency, class and position
```

**Gate A: soundness**

- upper-bound violations: 0;
- oracle closure misses: 0;
- false completeness certificates: 0;
- tie-boundary losses: 0.

**Gate B: feasibility**

- the measured search projects to the `<=5 ms` hot p99 contract;
- package projection remains `<=195 MiB`;
- RSS does not exceed the Phase 0 accepted ceiling;
- no full-center live scan is required in the p99 tail.

The number of exactly settled centers, for example 128 or 8,192, is a diagnostic,
not a correctness gate. A smaller number is useful only when Gate A remains
exact.

**Rollback**

Reject the bound/index design. Keep the oracle and matrix. Do not tune scoring to
make a weak bound appear useful.

**Stop**

- one bound violation;
- bounds are sound but so loose that p99 requires near-full scan;
- a proposed index exceeds package/RSS budgets;
- pruning depends on the heldout target or expected answer.

### Phase 9 - Add the shared WordCenter accumulator

**Work**

1. Feed posting and edit terminal events into one epoch-based dense accumulator.
2. Deduplicate by `WordCenterId`.
3. Merge forward mass, hit counts, typed geometry, exact reconstruction, script,
   and source masks with explicit commutative operators.
4. Preserve raw values required to recompute all normalization maxima exactly.
5. Resolve typed relation dependencies before final authority readout without
   turning evidence-only relation endpoints into candidates.

**Gate**

- posting-first and edit-first event orders produce byte-identical evidence;
- duplicate events are idempotent where the source contract requires it;
- all oracle `S(Q)` centers and required `R(Q)` records are present;
- no second candidate vector is scored independently.

**Rollback**

Keep proof nodes separate and remove the accumulator adapter.

**Stop**

Order-dependent evidence, per-query hash-map allocation, or conflicting owner
state between the two node kinds.

### Phase 10 - Reuse exact nonlinear settlement

**Work**

1. Move, do not rewrite, current reverse-wave scoring and interference kernels.
2. Feed the oracle-equivalent competitive closure to those kernels.
3. Compute normalization maxima from complete proven evidence.
4. Preserve anti, pairwise, sequence, geometry, position, and restoration
   classifier ordering.
5. Run all existing `ReadoutMode` ablations.

Two parity claims stay separate:

```text
move-only LegacyBirthSearch parity      must equal old runtime exactly
ExactTypedPeakSearch parity             must equal dense oracle exactly
```

The exact route may intentionally differ from the legacy route. Such differences
are judged by the fixed proof and safety contract, never by selected examples.

**Gate**

- exact-search versus oracle candidate/result fingerprint: 100%;
- candidate permutation parity: 100%;
- all legacy grounded winners remain in the exact lattice unless independent
  contradictory evidence is explicitly present and measured;
- false authority and false singleton: 0.

**Rollback**

Exact route remains proof-only. Legacy owner is still the only live route.

**Stop**

Any attempt to improve exact-search numbers by changing settlement coefficients
in the same phase.

### Phase 11 - Completeness certificate and failure injection

**Work**

1. Produce the certificate from actual exhausted/skipped queues and closure
   state, not from a success flag.
2. Make `Winner` authority require `Certified`.
3. Inject these failures:

```text
understate one posting block maximum
drop one typed edit edge
omit one equal-cutoff terminal
omit one required ambiguity record
omit one required pairwise record
corrupt package/query fingerprint
misclassify one evidence-only endpoint as a candidate
change a normalization maximum
```

4. Verify that each injection either fails parity or returns `Unresolved`; none
   may emit a certified winner.

**Gate**

- false certificate: 0;
- certified matrix cases: 100% oracle parity;
- every authoritative fixed-proof winner is certified;
- unresolved search is fail-closed;
- no hidden legacy fallback.

**Rollback**

Exact search remains non-authoritative.

**Stop**

Certificate logic merely repeats the search's own assumptions without an oracle
or injected counterexample test.

### Phase 12 - Offline shadow A/B

Shadow means an explicit evaluation command, not a daemon worker.

```text
fixed query
+-- LegacyBirthSearch
+-- ExactTypedPeakSearch
`-- DenseOracle, on the bounded oracle sample
```

Report separately:

```text
legacy vs exact target retention
legacy vs exact rank/verdict changes
exact vs oracle closure and result parity
certificate coverage and unresolved reasons
per-class unique top-1 and lattice coverage
clean preservation
false authority / false singleton
latency and work counters
package/RSS projections
```

**Gate**

- oracle matrix parity: 100%;
- every damage class unique top-1: strictly `>95%`;
- every damage class lattice coverage: `>=99%`;
- clean preservation: `>=99.9%`;
- false authority: `0`;
- false singleton: `0`;
- no class regresses against the pinned baseline;
- runtime authority remains unchanged.

**Rollback**

Remove the exact-search evaluation command from normal builds if it carries
unacceptable compile/runtime cost. Preserve receipts and source in the research
branch.

**Stop**

Aggregate improvement with one regressing class, denominator drift, or oracle
parity below 100%.

### Phase 13 - Decide package representation

Only now may package representation change.

The decision compares measured alternatives:

```text
1. Build child/superblock indexes in RAM from unchanged V8 bytes.
2. Reorder primary decoder nodes breadth-first so children are contiguous while
   preserving the 8-byte `(parent, symbol)` record.
3. Add a compact posting superblock skip directory.
4. Add a new bounded package section only if the first three cannot pass.
```

Current risk: a 189.05 MiB package has only about 5.95 MiB below the 195 MiB
ceiling. A naive child-arc table or an extra record for every 32-relation block
will not fit. The package design must be measured, not assumed.

Preferred order:

1. package-neutral decoder reindex;
2. compressed superblock directory at 128/256 relation granularity;
3. startup-derived index only if RSS and cold startup pass;
4. larger format redesign only after a new paper review.

**Gate**

- deterministic build/repack hash across two runs;
- decoder and relation round-trip parity: exact;
- search/oracle parity: exact;
- package: `<=195 MiB`;
- cold startup and first-touch: Phase 0 contract;
- steady and peak RSS: Phase 0 contract;
- raw corpus and damaged strings stored: 0.

**Rollback**

Reject the new format and retain the immutable baseline package. No installed
manifest changes.

**Stop**

Format compression is presented as quality proof, primary/relation
`decoder_terminal` meanings are mixed, or package headroom is exceeded.

### Phase 14 - Full proof and owner flip

**Preconditions**

- structural gate: PASS for the final route;
- implementation preflight: `READY_TO_IMPLEMENT` for deployment;
- all previous phases: PASS;
- exact artifact and source hashes pinned.

**Full proof**

```text
13 classes x 20,000 fixed heldout cases
separate RU and EN class reports
clean audit
objective ambiguity audit
false-authority and false-singleton audit
three fresh-process latency runs
cold startup, first-touch, hot p50/p90/p99/max
package and RSS audit
candidate-order permutation audit
oracle parity matrix
```

**Owner flip**

```text
L1.1 typed field
-> ExactTypedPeakSearch, sole live owner
-> certified bounded lattice
-> L2
-> L3
-> L4
-> DecisionCore
-> verifier
```

At the flip:

- remove the daemon shadow route if any experimental wiring exists;
- remove `LegacyBirthSearch` from production code;
- remove birth/frontier environment controls that no longer own behavior;
- retain dense oracle only behind proof/test compilation;
- do not keep a silent in-process fallback;
- rollback is deployment-level restoration of the previous binary/package pair.

**Physical application gate**

At minimum verify:

```text
ordinary prefix completion on every typed character
damaged-word restoration
RU <-> EN layout correction
Space preservation after autocorrect
Tab acceptance
Backspace over preedit
autocorrect -> double Shift -> exact original restoration
WeChat and one standard GTK/Chromium input
no repeated/stuck key output
```

**Promotion gate**

All conjunctive contracts pass. No metric compensates for another failure.

**Rollback**

Atomically restore the prior installed binary and package manifest, restart only
the owned Lay services, and prove keyboard input health. Never roll back by
enabling a hidden second owner inside the new process.

**Stop**

Any proof, latency, package, RSS, physical-input, verifier, or rollback failure.

## 8. Fixed Promotion Scoreboard

| Dimension | Required gate |
|---|---:|
| unique top-1, each of 13 classes | `>95.0%` |
| lattice coverage, each class | `>=99.0%` |
| clean preservation | `>=99.9%` |
| false authority | `0` |
| false singleton | `0` |
| oracle parity on certified matrix | `100%` |
| false completeness certificate | `0` |
| candidate permutation parity | exact |
| package | `<=195 MiB` |
| hot p99 | `<=5.0 ms` |
| first-touch/cold/RSS | exact Phase 0 deployment contract |
| runtime authority before Phase 14 | unchanged |

The repository contains historical RSS contracts that are not fully consistent
with the latest V8 deployment receipt. Phase 0 must resolve and freeze one
hardware-specific RSS ceiling before implementation. It is not acceptable to
select the favorable historical number after measuring the new route.

## 9. Build And Crystallization Budget

The refactor is explicitly designed to avoid repeated corpus crystallization.

```text
Phase 0-7    use one immutable existing package; 0 corpus crystallizations
Phase 8-12   use proof-only indexes over that package; 0 corpus crystallizations
Phase 13     at most 2 deterministic repacks of one accepted format candidate
             to prove identical bytes; still no corpus retraining
Phase 14     1 release build and 1 full fixed proof after all smaller gates pass
```

If a new index cannot be derived from the existing package and truly requires a
corpus pass, work stops and a separate compiler paper must predict:

```text
input corpus/hash
new section counts and bytes
relation operation count
peak disk/RSS
expected wall time and CPU utilization
rollback artifact
```

There is no automatic V-next loop. A failed experiment returns to the first
failed mechanism and paper model before another expensive build.

All Cargo commands use `scripts/cargo-guard.sh`. A broad unscoped `cargo test` is
forbidden. Full proof runs are not launched until small oracle and matrix gates
pass.

## 10. Risk Register

### R1 - Bound is sound but too loose

**Consequence:** exact search approaches a 762,314-center scan and misses 5 ms.  
**Response:** reject the index/bound design; keep oracle and typed traversal. Do
not reduce completeness or tune quality coefficients.

### R2 - Typed dependency resolution is incomplete

**Consequence:** a required ambiguity or pairwise record is missing, or an
evidence-only endpoint is incorrectly promoted into the candidate set.  
**Response:** require typed dependency completeness, explicit endpoint roles,
and failure injection before a certificate can be issued.

### R3 - Decoder traversal needs too much memory

**Consequence:** package or RSS gate fails.  
**Response:** test package-neutral breadth-first decoder reindex and compact
superblocks before adding tables.

### R4 - Exact semantics improve retention but change safe ties

**Consequence:** top-1 may rise while false authority appears.  
**Response:** oracle readout includes full `Winner | Tied | ABSTAIN`, ambiguity,
and pairwise behavior. False authority remains a hard zero.

### R5 - Move-only refactor changes behavior

**Consequence:** later algorithm evidence is untrustworthy.  
**Response:** reject the individual cut immediately; do not debug it together
with exact search.

### R6 - Dirty worktree contaminates the baseline

**Consequence:** parity cannot be reproduced or safely rolled back.  
**Response:** isolate and hash the exact source snapshot before Phase 1.

### R7 - Two owners remain live

**Consequence:** duplicate CPU work, mixed normalization, unclear authority.  
**Response:** route gate requires one owner. Shadow comparison is offline only;
promotion removes legacy production execution.

### R8 - The refactor is mistaken for productive morphology

**Consequence:** expectations move from restoration to unseen-form generation.  
**Response:** L1.1 searches existing addressable centers. Productive unknown
word forms and contextual ending choice remain L2/L3 responsibilities.

## 11. Evidence Required Per Experiment

Every architecture experiment updates this document or its successor in the
same change and records:

```text
hypothesis
immutable inputs and hashes
what was tested
what was not tested
global and per-class denominators
quality and safety metrics
work counters and latency distribution
package/RSS impact
oracle/baseline parity
verdict scope
runtime authority changed: true | false
exact receipt path
```

Compression parity is not quality proof. A small sample is not the full gate. A
combined bilingual denominator is not a substitute for RU and EN reports. A
shadow result is not live authority.

## 12. Milestones And Definition Of Done

```text
M0 BASELINE_LOCKED
   all bytes, denominators and runtime settings pinned

M1 MOVE_ONLY_PARITY
   runtime split, exact legacy behavior preserved

M2 ORACLE_DEFINED
   dense semantics executable and independent of legacy birth

M3 EDIT_GRAPH_COMPLETE
   all 13 operator classes and position strata match oracle terminals

M4 BOUNDS_SOUND
   zero bound violations and zero false certificates

M5 SEARCH_EXACT
   optimized exact search matches dense oracle 100%

M6 FEASIBLE
   package, RSS, startup, first-touch and hot latency pass

M7 FULL_PROOF_PASS
   all conjunctive 13-class quality and safety gates pass

M8 LIVE_OWNER
   one exact L1.1 owner, legacy route removed, physical input verified

M9 RELEASED
   version, canonical docs, graphify, receipts, commit, push and atomic deploy
```

The project is not done at `M3` because edit reachability alone does not prove
ranking or authority. It is not done at `M5` because exactness may be too slow or
large. It is not done at `M7` because deployment and physical input remain
separate gates.

## 13. Current Verdict And Next Action

Current verdict: `PHASE_2_COMPLETE_MOVE_ONLY_PARITY_PASS`; implementation preflight
verdict remains `READY_TO_IMPLEMENT` with `safe_to_implement = true` and no
blockers.

Phase 0 is complete. The immutable source, reader, package, corpus, heldout,
latency, and semantic projection inputs are pinned. The installed reader was
reproduced byte-for-byte from the isolated source snapshot. The fixed current
proof passes all 13 quality and safety class gates, while current diverse hot
`p99 = 21.1-22.2 ms` fails the future `<=5 ms` promotion gate. Runtime authority,
installed package bytes, daemon, and IBus remain unchanged.

The design-only route gate still proves only route separation: two producers
converge on one evidence owner, then one rank owner and one restoration
authority owner. The implementation preflight admits code work against the
pinned baseline; neither result proves the future search mathematics,
implementation correctness, artifact feasibility, or deployment readiness.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_EXACT_PEAK_SEARCH_PLAN_2026-08-13/route-design.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_EXACT_PEAK_SEARCH_PLAN_2026-08-13/route-gate.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_BASELINE_2026-08-13/baseline.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_BASELINE_2026-08-13/implementation-preflight-receipt.json
```

Phase 0 measured facts:

1. Package: `190,139,182 B`; package gate passes with `14,333,138 B` headroom.
2. Fixed proof: `260,000` damaged cases plus `852,582` clean cases; all 13
   classes are strictly above `95%` unique top-1 and at or above `99%` lattice
   coverage; false certainty, false authority, and false singleton are zero.
3. Candidate semantic projection reproduced twice with SHA-256
   `2d99e87b685625d0791c857aecbfce022e6353d58aa5bc359c9d490b1a7c4a96`.
4. Fresh-process startup is `0.92-1.18 s` with peak RSS
   `165,812-166,248 KiB`; both baseline deployment gates pass.
5. Current diverse hot p99 is `21.1-22.2 ms`; the final latency gate fails.

Remaining unproved boundaries:

1. Dense competitive-set semantics exist in this paper but are not yet an
   executable independent oracle.
2. Sound multidimensional bounds have not been measured for tightness.
3. The package has little size headroom, and decoder/posting index representation
   is not yet selected by evidence.
4. Exact-search behavior, completeness certificates, full fixed proof, and live
   deployment have not been tested.

Phase 1 is complete. The streaming behavior fingerprint reproduced byte-for-byte
twice over 616 deterministic cases, all eight `ReadoutMode` values, query,
restore, host lattice, service lattice, stable benchmark/stats projections, and
candidate-order permutation. The semantic SHA-256 is
`c6159bf499146c21a96723435ac4112496eca10bf8c1dd961964d789333267d7`;
the old Phase 0 candidate SHA-256 remains
`2d99e87b685625d0791c857aecbfce022e6353d58aa5bc359c9d490b1a7c4a96`.

The first materialized implementation was rejected despite semantic PASS because
it wrote `646,980,277 B` and reached `5,688,412 KiB` peak RSS. The accepted
length-framed streaming digest writes `19,961 B` and measured
`1,074,120-1,077,756 KiB` peak RSS. This is proof-tool residency, not live L1.1
runtime residency.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_BEHAVIOR_2026-08-13/behavior-fingerprint.json
```

Phase 2A is complete. Diagnostics and runtime configuration readers moved from
`runtime.rs` into named owners without changing the public API, allocation
strategy, `LexicalGrokkingMemory::readout`, package bytes, or runtime authority.
The runtime facade fell from `4,565` to `4,217` lines; the two extracted modules
contain `397` lines, and total source growth is `1.07%`, below the Phase 2 `5%`
budget.

The frozen 616-case fingerprint is byte-identical to Phase 1. All six route
SHA-256 values, the semantic SHA-256
`c6159bf499146c21a96723435ac4112496eca10bf8c1dd961964d789333267d7`,
the projection SHA-256
`85cf72483cf68c2f93e262569571cf1169b9feb90b8a1767536133afaef7b036`,
and the candidate SHA-256
`2d99e87b685625d0791c857aecbfce022e6353d58aa5bc359c9d490b1a7c4a96`
match exactly. Candidate permutation failures remain zero.

Three sequential diverse latency runs measured p99 `15.861`, `18.442`, and
`19.923 ms`, all below the pinned baseline range `21.136-22.217 ms`. This proves
that the move-only cut did not cause the allowed `5%` regression; it does not
prove the future `<=5 ms` promotion gate. The package remains `190,139,182 B`
with SHA-256
`47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9`.

Tested: the Phase 2A move boundary, all frozen semantic routes, candidate order,
three-run latency, changed-file checks, lexical fingerprint tests, package and
fixed-input identity. Not tested: Phase 2B-2H, dense oracle, typed edit graph,
posting bounds, exact-search quality, package redesign, or deployment. Runtime
authority, installed package, daemon, and IBus remain unchanged; crystallization
runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2A_2026-08-13/phase-2a.json
```

Phase 2B is complete. Eleven internal evidence/result types moved into
`runtime/contract.rs`; the runtime algorithm, `readout` call graph, external
exports, and allocation behavior did not change. `runtime.rs` fell from `4,217`
to `4,095` lines. The complete runtime tree is `4,627` lines, cumulative growth
`1.36%` from the `4,565`-line Phase 0 monolith and below the Phase 2 `5%` budget.

The frozen semantic and route hashes match Phase 1 and Phase 2A exactly over all
616 cases and eight modes; candidate permutation failures remain zero. Three
sequential p99 measurements were `16.360`, `15.475`, and `17.758 ms`, with the
same candidate SHA-256 in every run and no regression against the pinned
baseline. Two intermediate compile-only failures exposed Rust child-module
visibility narrowing; restoring the previous `lexical_grokking` visibility
closed them before any fingerprint or runtime execution.

Tested: the Phase 2B type boundary, previous visibility contract, all frozen
semantic routes, candidate order, three-run latency, and changed-file checks.
Not tested: Phase 2C-2H, dense oracle, typed edit graph, posting bounds,
exact-search quality, package redesign, or deployment. Runtime authority,
installed package, daemon, and IBus remain unchanged; crystallization runs
remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2B_2026-08-13/phase-2b.json
```

Phase 2C is complete. Relation storage, V8 posting access, borrowed/shared
coupling views, and reverse-cache ownership moved into `runtime/relations.rs`.
The scoring-side `refresh_frontier_activations` remains in `runtime.rs`, so the
cut does not mix storage access with frontier behavior. `runtime.rs` fell from
`4,095` to `3,857` lines; the full runtime tree is `4,640` lines, cumulative
growth `1.64%` from Phase 0 and below the `5%` budget.

The accepted source preserves the original iterator placement: eager forward
views use `iter`, while batched reverse reconstruction uses `par_iter` inside
the existing runtime pool. Manual diff review rejected one intermediate state
that accidentally swapped this placement even though its semantic fingerprint
matched. None of that rejected run's latency evidence is used for acceptance.

On the corrected source, all six frozen route hashes, semantic SHA, projection
SHA, candidate SHA, and candidate order match exactly. Three sequential p99
measurements were `13.214`, `13.036`, and `15.223 ms`; no Phase 2 regression is
present, while the future `<=5 ms` gate remains unmet and unclaimed.

Tested: the Phase 2C relation boundary, exact iteration placement, all frozen
semantic routes, candidate order, three-run latency on corrected source, and
changed-file checks. Not tested: Phase 2D-2H, dense oracle, typed edit graph,
posting bounds, exact-search quality, package redesign, or deployment. Runtime
authority, installed package, daemon, and IBus remain unchanged;
crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2C_2026-08-13/phase-2c.json
```

Phase 2D is complete. `L1RestorationHost`, composite overlay loading, terminal
ID translation, tombstones, exact-surface lookup, and host lattice assembly
moved into `runtime/host.rs`. The extraction preserves the public host exports,
the existing readout owner, allocation behavior, and package representation.
`runtime.rs` fell from `3,857` to `3,349` lines; `host.rs` contains `516` lines.
The complete runtime tree is `4,648` lines, cumulative growth `1.82%` from the
Phase 0 monolith and below the Phase 2 `5%` budget.

The frozen 616-case fingerprint, all six route SHA-256 values, semantic SHA,
projection SHA, candidate SHA, and candidate permutation result match Phase 1
exactly. The focused remote lexical gate passed `109/109` tests. Three
sequential fixed-520 runs on `e@192.168.3.94` measured p99 `10.070`, `10.009`,
and `9.572 ms`; every run preserved candidate SHA-256
`2d99e87b685625d0791c857aecbfce022e6353d58aa5bc359c9d490b1a7c4a96`.
Peak RSS was `418,128-418,452 KiB`. These measurements pass the move-only
no-regression gate; they do not satisfy or claim the future `<=5 ms` promotion
target.

The remote build and proof were configured with `CARGO_BUILD_JOBS=20` and
`RAYON_NUM_THREADS=20`. The accepted release profile still measured `99%`
average CPU during final compilation because it intentionally uses
`codegen-units=1`; changing that profile would invalidate latency comparability.
The focused test run measured `669%` average CPU. No training or package
crystallization ran locally or remotely.

Tested: the Phase 2D host/composite boundary, all frozen semantic routes,
candidate order, focused lexical tests, three-run remote latency, package and
fixed-input identity, changed-file checks, and Cargo disk budget. Not tested:
Phase 2E-2H, dense oracle, typed edit graph, posting-bound soundness,
exact-search quality, package redesign, or deployment. Runtime authority,
installed package, daemon, and IBus remain unchanged; crystallization runs
remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2D_2026-08-13/phase-2d.json
```

The next action is Phase 2E: move pure geometry and operator kernels without
changing behavior or authority. No package crystallization, daemon restart,
owner flip, or scoring change is justified in Phase 2.

Phase 2E is complete. The pure edit-geometry kernels moved into
`runtime/geometry.rs`: Damerau distance, reconstruction-mode recognition,
surface substitution/transposition recognition, ordered-subsequence traversal,
deletion-transposition traversal, and the ambiguity geometry-link predicate.
Operator birth, geometry reserve ranking, candidate certificate interference,
settlement, and finalization deliberately remain in `runtime.rs`; moving them
here would have mixed the Phase 2E mathematical boundary with the Phase 2F
frontier and Phase 2G settlement owners.

`runtime.rs` fell from `3,349` to `3,126` lines; `geometry.rs` contains `248`
lines. The complete runtime tree is `4,673` lines, cumulative growth `2.37%`
from the Phase 0 monolith and below the Phase 2 `5%` budget. No public export,
constant value, branch order, allocation path, caller, or runtime owner changed.

The remote exact fingerprint reproduced all `616` cases, all eight readout
modes, semantic SHA, projection SHA, all six route SHA values, candidate order,
and zero permutation failures. Focused lexical tests passed `109/109`; route
contracts passed `20/20` and `15/15`; `cargo check --lib --bins` passed. Three
sequential fixed-520 runs measured p99 `10.005`, `10.975`, and `10.272 ms`, all
with the frozen candidate SHA. Peak RSS was `417,612-418,640 KiB`. This passes
the move-only no-regression gate but does not claim the future `<=5 ms` target.

All Cargo and proof work ran on `e@192.168.3.94` with
`CARGO_BUILD_JOBS=20` and `RAYON_NUM_THREADS=20`. A temporary `bwrap` mount
namespace exposed the immutable inputs at their original `/home/ubu/...` paths,
so path-bearing fingerprint JSON remained byte-comparable without creating
persistent remote aliases. Two format checks stopped before compilation: the
remote `1.97.1` toolchain lacked rustfmt, and the initial moved source needed
three rustfmt-only line wraps. The accepted gates ran after those formatting
issues were resolved; neither failed run executed runtime behavior.

Tested: the Phase 2E geometry boundary, all frozen semantic routes, candidate
order, focused lexical and route-contract tests, all lib/bin compile routes,
three-run remote latency, package/input identity, and Cargo disk budget. Not
tested: Phase 2F-2H, dense oracle, typed edit graph, posting-bound soundness,
exact-search quality, package redesign, or deployment. Runtime authority,
installed package, daemon, and IBus remain unchanged; training and
crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2E_2026-08-13/phase-2e.json
```

Phase 2F is complete. Legacy atom selection, thread-local forward scratch,
readout preparation, exact singleton discovery, exact-anchor lookup, operator
and geometry reserves, surface resolution, terminal activation, and frontier
refresh moved into `runtime/legacy.rs`. Settlement, interference, finalization,
the classifier bridge, and inline tests remain in `runtime.rs` for Phases 2G
and 2H. Every moved symbol exists exactly once; runtime authority, call order,
allocation behavior, package representation, and public exports are unchanged.

`runtime.rs` fell from `3,126` to `2,343` lines; `legacy.rs` contains `792`
lines. The complete runtime tree is `4,682` lines, cumulative growth `2.56%`
from the Phase 0 monolith and below the `5%` budget.

The remote exact fingerprint reproduced all `616` cases, all eight readout
modes, semantic SHA, projection SHA, all six route SHA values, candidate order,
and zero permutation failures. Focused lexical tests passed `109/109`; route
contracts passed `20/20` and `15/15`; `cargo check --lib --bins` passed. Three
sequential fixed-520 runs measured p99 `9.922`, `11.236`, and `13.397 ms`, all
with the frozen candidate SHA. Peak RSS was `417,560-418,668 KiB`. This passes
the move-only no-regression gate but does not claim the future `<=5 ms` target.

All Cargo, proof, and benchmark work ran on `e@192.168.3.94` with
`CARGO_BUILD_JOBS=20`, `RAYON_NUM_THREADS=20`, and 20 test threads. The final
release codegen unit averaged `99%` CPU because the accepted release profile
pins `codegen-units=1`; that profile was not changed. No CPU-heavy project work
ran locally.

One early cut-1 result is explicitly rejected: a mistaken sync wrote the new
parent source as `runtime/runtime.rs`, so the remote compiler used the old
`runtime.rs`. The nested file was deleted, both exact destination SHA-256
values were verified, and every accepted Phase 2F gate was rerun on the correct
source. The rejected `109/109` result is not evidence for this phase.

Tested: the Phase 2F owner boundary, symbol uniqueness, all frozen semantic
routes, candidate order, focused lexical and route-contract tests, all lib/bin
compile routes, three-run latency, package/input identity, and Cargo disk
budget. Not tested: Phases 2G-2H, dense oracle, typed edit graph, posting-bound
soundness, exact-search quality, package redesign, or deployment. Runtime
authority, installed package, daemon, and IBus remain unchanged; training and
crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2F_2026-08-13/phase-2f.json
```

Phase 2G is complete. Candidate settlement, phase evidence, restoration
geometry, ambiguity observations, structural/pairwise/sequence/geometry/
position interference, finalization, and the restoration classifier bridge
moved into `runtime/settlement.rs`. Each moved symbol exists exactly once.
Call order, score constants, allocation behavior, package representation,
public exports, readout ownership, and runtime authority are unchanged.

`runtime.rs` fell from `2,343` to `1,051` lines; `settlement.rs` contains
`1,324` lines. The complete runtime tree is `4,714` lines, cumulative growth
`3.26%` from the `4,565`-line Phase 0 monolith and below the `5%` budget.
`settlement.rs` is slightly above the approximate `1,200`-line guide because
settlement, interference, finalization, and the classifier bridge are one named
ownership boundary. Splitting interference into another owner would weaken
that boundary without reducing runtime behavior or dependency complexity.

The remote exact fingerprint reproduced all `616` cases, all eight readout
modes, semantic SHA, projection SHA, all six route SHA values, candidate order,
and zero permutation failures. Focused lexical tests passed `109/109`; route
contracts passed `20/20` and `15/15`; `cargo check --lib --bins` and
`scripts/check-lay-changed.sh` passed. Three sequential fixed-520 runs measured
p99 `11.520`, `9.864`, and `11.061 ms`, all with the frozen candidate SHA.
Peak RSS was `417,284-418,232 KiB`. This passes the move-only no-regression
gate but does not satisfy or claim the future `<=5 ms` promotion target.

All Cargo, proof, and benchmark work ran on `e@192.168.3.94` with
`CARGO_BUILD_JOBS=20`, `RAYON_NUM_THREADS=20`, and 20 test threads. The release
build took `183.48 s`, peaked at `2,398,456 KiB`, and averaged `100%` CPU
because the pinned release profile uses one final codegen unit. No CPU-heavy
project work ran locally.

One intermediate lexical-test result is rejected: immediately after the move,
four inline parent-module tests could no longer resolve
`truncate_with_reconstruction_tail`. The helper was exposed only to its parent
runtime module and imported only under `cfg(test)`. The accepted `109/109`
result and every behavior/latency/route gate ran after that repair on the final
source SHA values.

Tested: the Phase 2G owner boundary, uniqueness of moved symbols, all frozen
semantic routes, candidate order, focused lexical and route-contract tests,
all lib/bin compile routes, three-run latency, package/input identity, changed
checks, and Cargo disk budget. Not tested: Phase 2H, dense oracle, typed edit
graph, posting-bound soundness, exact-search quality, package redesign, or
deployment. Runtime authority, installed package, daemon, and IBus remain
unchanged; training and crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2G_2026-08-14/phase-2g.json
```

The next action is Phase 2H: move the inline runtime tests into
`runtime/tests.rs` without changing behavior or authority, then repeat the
complete Phase 2 parity gate.

Phase 2H is complete, which closes the complete Phase 2 move-only split. The
inline `cfg(test)` module moved from `runtime.rs` into `runtime/tests.rs`. The
`484` moved test lines reproduced their pre-move SHA-256 exactly; no test body,
fixture, assertion, name, runtime symbol, visibility, or production call path
changed. The release binary also remained byte-identical to Phase 2G.

`runtime.rs` fell from `1,051` to `566` lines; `runtime/tests.rs` contains
`484` lines. The complete runtime tree is `4,713` lines, cumulative growth
`3.24%` from the `4,565`-line Phase 0 monolith and below the `5%` budget. The
facade now contains package loading, public orchestration, warm/profile entry
points, and surface-index construction; proof fixtures no longer occupy the
runtime facade body.

The remote exact fingerprint reproduced all `616` cases, all eight readout
modes, semantic SHA, projection SHA, all six route SHA values, candidate order,
and zero permutation failures. Focused lexical tests passed `109/109`; route
contracts passed `20/20` and `15/15`; `cargo check --lib --bins` and
`scripts/check-lay-changed.sh` passed. Three sequential fixed-520 runs measured
p99 `10.131`, `10.199`, and `9.709 ms`, all with the frozen candidate SHA.
Peak RSS was `417,584-418,564 KiB`. This closes the Phase 2 no-regression gate;
it does not satisfy or claim the future `<=5 ms` promotion target.

All Cargo, proof, and benchmark work ran on `e@192.168.3.94` with
`CARGO_BUILD_JOBS=20`, `RAYON_NUM_THREADS=20`, and 20 test threads. The release
build took `178.63 s`, peaked at `2,398,432 KiB`, and produced the exact Phase
2G binary SHA-256. No CPU-heavy project work ran locally.

Tested: exact test-byte preservation, the complete Phase 2 module boundary,
the production binary SHA, all frozen semantic routes, candidate order,
focused lexical and route-contract tests, all lib/bin compile routes,
three-run latency, package/input identity, changed checks, and Cargo disk
budget. Not tested: the Phase 3 owner contract, dense oracle, typed edit graph,
posting-bound soundness, exact-search quality, package redesign, or deployment.
Runtime authority, installed package, daemon, and IBus remain unchanged;
training and crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_2H_2026-08-14/phase-2h.json
```

The next action is Phase 3: introduce one internal `L1PeakSearch` owner
contract and wrap the unchanged legacy route behind it. Phase 3 may begin only
as a separate commit and must retain all Phase 2 fingerprints exactly.

Phase 3 is complete. Production `LexicalGrokkingMemory::readout` now delegates
exactly once through the internal `L1PeakSearch` contract to
`LegacyBirthSearch`. That owner preserves the three existing outcomes: empty,
exact singleton, or the bounded legacy lattice followed by the existing
nonlinear settlement. There is no fan-out, shadow execution, fallback, second
winner, scoring change, normalization change, or new query allocation.
The complete runtime/search tree is `4,801` lines, `5.17%` above the Phase 0
monolith. The `5%` limit applied to the move-only Phase 2, which closed at
`3.24%`; Phase 3 is the first separately admitted owner-contract growth.

The observed-source route gate passed with `6` nodes, `12` edges, and `6`
event/kind routes. The fused `readout_modes` function remains a proof-only
eight-mode fingerprint optimization; it does not participate in daemon or
service production authority and is explicitly outside the production owner
cardinality claim.

The `616`-case fingerprint, all eight readout modes, semantic SHA, projection
SHA, all six route SHA values, candidate order, and zero permutation failures
match Phase 2 exactly. Focused lexical tests passed `109/109`; authority and
mutation contracts passed `20/20` and `15/15`; all lib/bin compile routes and
`scripts/check-lay-changed.sh` passed. Three uncontaminated latency runs
measured p99 `10.093`, `9.778`, and `9.412 ms`; readout p99 was `10.888`,
`10.332`, and `10.462 ms`. Peak RSS was `417,744-418,236 KiB`. This proves
Phase 3 no-regression, not the future `<=5 ms` promotion target.

Binary identity was resolved rather than waived. A clean Phase 2 rebuild at a
different absolute path produced a different ELF, while an isolated rebuild
that exposed the same source bytes at the original compiler-visible path
reproduced SHA-256 `86a556df...` exactly. Phase 3 produces SHA-256
`0bbe9a30...` and is `192` bytes larger because the declared owner interface
changes executable and unwind sections. Therefore whole-ELF equality is a
valid reproducibility check for unchanged source, but not a behavior-parity
condition after adding executable code. Preflight v3 was created prospectively
from a restored Phase 2 source state, passed with zero blockers, and the Phase
3 patch was then reapplied byte-for-byte with SHA-256 `388fd5c2...`. Rejected
preflight revisions and failed namespace/toolchain attempts remain preserved.

Tested: production route cardinality, exact frozen behavior, candidate order,
focused lexical and authority gates, all compile routes, three-run latency,
Phase 2 binary reproducibility, package identity, and Cargo disk budget. Not
tested: dense oracle semantics, typed edit traversal, posting bounds, exact
search quality, package redesign, or deployment. Runtime authority, installed
package, daemon, and IBus remain unchanged; training and crystallization runs
remain zero.

The incremental source graph was updated remotely with `20` workers in
`12.36 s`: `14,905` nodes, `27,436` edges, and `2,877` communities. Only the
tracked graph outputs were returned to the local worktree.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_3_2026-08-14/phase-3.json
```

The next action is Phase 4: implement the dense proof-only oracle and prove it
on exhaustively enumerable tiny packages. It must remain unreachable from
daemon and service production entrypoints.

Phase 4 is complete. `peak_oracle.rs` is compiled only under
`cfg(any(test, feature = "lexical-compiler"))`; normal runtime, service, and
`peak_search` sources contain no oracle import, switch, call, fallback, or
authority edge. The proof route resolves the query once, computes activation
for every `WordCenterId`, calls the existing settlement, geometry,
interference, and restoration-classifier kernels, and enumerates every
non-empty competitive subset. It does not duplicate score formulas or accept a
target/expected surface as input.

The accepted tiny package contains five centers. The oracle enumerated all
`31` non-empty subsets and all `16` subsets containing each center. For every
center, reported lower and upper final scalar values matched the independently
replayed minimum and maximum over those subsets. Requested `K=128` correctly
reported `effective_K=5`. Forward, reverse, and deterministic permuted center
iteration produced byte-identical JSON and result fingerprints. External
heldout labels cannot enter the API and did not change report bytes.

Phase 4 deliberately uses a conservative complete typed set `D(Q)` containing
all tiny-package centers. Therefore `S(Q)` is complete but not yet sharp. This
proves the reference semantics required to judge later traversal and pruning;
it does not prove a production-search speedup, a posting bound, fixed-heldout
quality, or the `<=5 ms` promotion target. Sharp typed reachability belongs to
Phase 7 and sharp posting bounds belong to Phase 8.

The first paper route was rejected because proof nodes were placed in an
authority graph. Its `VETO` receipt is preserved. The corrected route has one
execution path ending at frozen settlement, a separate proof path, a separate
observation path, and no authority route. The first implementation preflight
was also preserved as `BLOCKED_BEFORE_CODE`: its test kind was not an admitted
parity kind. The corrected preregistration returned `READY_TO_IMPLEMENT` with
zero blockers before source code was changed.

Remote gates on `e@192.168.3.94` passed: tiny oracle `5/5`, all focused lexical
tests `114/114`, transition authority `20/20`, mutation monopoly `15/15`, all
library/binary compile routes, and `scripts/check-lay-changed.sh`. The focused
lexical run completed in `5.35 s`, averaged `1657%` CPU, and peaked at
`98,212 KiB`; changed-file checks completed in `20.99 s`, peaked at
`2,336,504 KiB`, and reported no unsafe edits. Cargo target usage was
`3,338,248,192` bytes under the `12 GiB` guard budget. No heavy Cargo, proof,
or graph work ran locally.

The `lexical-compiler` feature compile gate passed separately, proving that the
proof-only oracle is available in its declared compiler build and absent from
default production builds. The incremental graph update ran remotely with 20
workers in `12.58 s`: `14,937` nodes, `27,527` edges, and `2,860`
communities.

Tested: exhaustive center/subset enumeration on a five-center package, exact
subset extrema, target-free API, iteration-order byte parity, proof-only source
reachability, focused lexical and authority contracts, all compile routes,
installed-package identity, and Cargo disk budget. Not tested: the Phase 5
class/position matrix, forward lexical index, typed edit reachability, posting
bounds, exact-search quality/latency, package redesign, or deployment. Runtime
authority, installed package, daemon, and IBus remain unchanged; training and
crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_4_2026-08-14/phase-4.json
```

Phase 5 is complete. The deterministic proof matrix covers exactly `260,000`
fixed heldout cases: `13` damage classes with `20,000` cases each. It records
language, length, frequency, edit position, ordered position pair, damage
class, objective ambiguity, and exactly one first-loss boundary for every
case. All independent axis sums and the first-loss sum equal `260,000`;
unsupported cases are zero. The matrix contains both languages (`181,235` RU
and `78,765` EN), all three frequency buckets, all five position buckets,
`1,444` populated requested strata, and `14,156` explicitly empty strata.

The old semantic proof projection is byte-identical to the Phase 0 baseline:
SHA-256 `f11cd463ce7ed0bb8ff2eec989d1294776d00d54d2f7b8b40cd35cdacd8eb9fa`.
The matrix therefore adds diagnostics without changing candidate order,
quality, safety, or authority. The strict quality contract remains unchanged:
all `13/13` classes exceed `95%` unique top-1, the minimum is sparse
multi-omission at `96.694332%`, minimum lattice coverage is `99.535%`, clean
preservation is `100%`, and false certainty, false authority, and false
singleton remain zero.

The exclusive first-loss decomposition is:

```text
contract satisfied                  48,272
independent frontier eligibility        23
nonlinear settlement rank            6,387
restoration authority              205,318
query/posting/typed/dependency loss       0
total                              260,000
```

This does not say that `205,318` words are restored incorrectly. It says the
frozen L1.1 restoration classifier intentionally does not issue singleton
authority for those cases. The exact-peak-search work must first eliminate the
`23` pre-settlement frontier losses and then prove rank/closure parity; it must
not weaken authority or safety to turn abstentions into winners.

The full run used the immutable V8 package and fixed corpus/heldout bytes. It
completed on the remote 20-logical-CPU host in `21:46.85`, averaged `512%`
CPU, and peaked at `1,255,684 KiB` RSS. The low effective parallelism is a
proof-tool effect: first-loss posting availability repeatedly reconstructs V8
reverse evidence through one shared cache. Production runtime latency was not
changed or re-characterized by this diagnostic run. Later matrix reruns may
replace that check with an equivalent proof-only atom-ID intersection after a
separate parity gate.

Tested: exact fixed denominator, every independent matrix axis, exclusive
first-loss accounting, both languages, all fixed classes, unsupported-case
rejection, semantic projection parity, focused lexical and authority
contracts, all compile routes, installed package identity, and Cargo disk
budget. Not tested: the Phase 6 forward decoder index, typed edit traversal,
posting-bound soundness, optimized exact-search latency, package redesign, or
deployment. Runtime authority, installed package, daemon, and IBus remain
unchanged; training and crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_5_2026-08-14/phase-5.json
/home/ubu/.cache/lay/l1-peak-search-phase5-2026-08-14/full-13x20000.json
```

Phase 6 is complete. `forward_decoder_index.rs` derives one transient CSR
forward view from the package's immutable decoder parent links:

```text
DecoderNode(parent, symbol)
-> child counts
-> prefix-sum child offsets
-> children sorted by (symbol, child node ID)
-> unique transition-symbol validation

package.centers only
-> terminal counts keyed by DecoderNodeId
-> prefix-sum terminal offsets
-> explicit sorted WordCenterId lists
```

The full V8 package contains `1,584,307` decoder nodes, `1,584,306` forward
edges, and `852,582` primary WordCenter terminals. Every terminal decoded to a
UTF-8 surface, traversed forward to the identical decoder node, and appeared
in that node's terminal list: `852,582/852,582`. Parent bounds, unique root,
acyclicity, Unicode symbols, child sorting, and unique transition symbols all
passed. Maximum fan-out is `59`; maximum surface length is `31` characters.
The accepted corpus has zero terminal-collision nodes, while a focused fixture
proves that multiple WordCenterIds at one decoder node are represented
explicitly rather than overwritten.

The index builder reads only `package.decoder_nodes` and `package.centers`.
Fault injection placed invalid `decoder_terminal` values in relation banks;
the index remained valid and contained only the primary center. Invalid root,
out-of-range parent, parent cycle, self-parent, and duplicate child symbol are
rejected before admission.

The derived index occupies `28,759,240` bytes (`27.43 MiB`). On the remote
package proof, package load took `758 ms`, index build plus complete validation
took `270 ms`, fresh-process wall time was `1.32 s`, and process peak RSS was
`187,400 KiB`. Incremental RSS after package load was `29,569,024` bytes. A
second run measured `763 ms` load and the same `270 ms` build; the stable
report projection and index fingerprint were byte-identical. These are
proof-tool measurements, not production startup or hot-path latency.

The package SHA-256 remained
`47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9`
before and after both runs. The module is compiled only under
`cfg(any(test, feature = "lexical-compiler"))`; the proof CLI exists only with
`lexical-compiler`. Normal runtime, service, daemon, and `peak_search` do not
import, load, or call the index. Runtime authority, scoring, package format,
installed package, daemon, and IBus are unchanged.

The first route design was rejected because evidence `proves` edges terminated
at an orchestrator instead of a proof owner. The corrected route has one
execution path, one proof path, one observation path, and zero authority
routes. The first implementation preflight was also retained as
`BLOCKED_BEFORE_CODE` because five declared vetoes lacked static tripwires.
After those tripwires were added, preflight returned `READY_TO_IMPLEMENT` with
zero blockers. No code was written before that result.

Remote gates passed: focused decoder-index tests `7/7`, focused lexical tests
`129/129`, transition authority `20/20`, mutation monopoly `15/15`, default
and `lexical-compiler` lib/bin compile routes, `scripts/check-lay-changed.sh`,
format/diff checks, package-byte parity, and the `12 GiB` Cargo target budget.
The first remote Cargo attempt is rejected tool evidence: non-login SSH selected
system Cargo `1.75.0`, which cannot parse lockfile v4. The accepted gates used
the pinned `$HOME/.cargo/bin/cargo 1.97.1`; no lockfile or source workaround
was made.

Tested: decoder topology, transition determinism, primary-terminal isolation,
complete real-package UTF-8 round-trip, explicit terminal collisions,
deterministic index bytes, resource use, package identity, production-source
isolation, and normal regression contracts. Not tested: Phase 7 typed edit
traversal, terminal-set/certificate parity by damage family, posting bounds,
optimized exact-search quality/latency, package redesign, or deployment.
Training and crystallization runs remain zero.

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_6_2026-08-14/phase-6.json
/home/ubu/.cache/lay/l1-peak-search-phase6-2026-08-14/forward-index.json
```

### Phase 7A accepted result - 2026-08-14

Phase 7A is complete and remains proof-only. One `L1TypedEditTraversal` now
traverses the Phase 6 decoder index with exact identity and one-target-insertion
states, canonicalizing completed insertions to prefix or suffix certificates.
Punctuation suffix evidence is emitted from independently preserved raw
boundary metadata; punctuation remains present in the reserved raw/layout lane.

The independent tiny oracle and forward/reverse/permuted scheduling gate passed
`5/5`. On the unchanged full package, clean identity passed
`852,582/852,582`; prefix, suffix, and punctuation each passed target retention,
certificate recall, and scheduling parity `20,000/20,000`. The full `60,000`
case run took `9.14 s`, averaged `1,027%` CPU, and peaked at `591,208 KiB` RSS.
Package bytes and runtime authority remain unchanged. Focused lexical regression
is now `134/134`; authority `20/20` and mutation `15/15` remain PASS.

Measured evidence and scope boundaries are recorded at:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/l1-l11-typed-edit-product-phase7.md
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_7A_2026-08-14/phase-7a.json
/home/ubu/.cache/lay/l1-peak-search-phase7a-2026-08-14/full-3x20000.json
```

The next action is Phase 7B: extend the same owner with interior target
insertion, input deletion, generic one-symbol substitution, and all-symbol exact
layout projection. It must retain the accepted 7A families and pass complete
tiny terminal/certificate parity before the fixed seven-family gate.
