# L1.1 Phase 8H: Exact Decoder-Subtree Posting Cover

Status: projection complete; correctness PASS; representation and work topology REJECTED
Date: 2026-08-14
Owner: proof-only `DecoderSubtreeCoverProjection` under `L1PeakSearch`

## 1. Decision

Phase 8H tests one representation change before any new search executor:

```text
complete atom posting states
-> maximal homogeneous decoder-trie subtrees
-> query updates on subtree roots
-> sparse activated-trie closure
-> exact symbolic score cohorts
-> exact K-th mass and equality closure
```

The representation does not create candidates. It rewrites a repeated exact
relation state over many lexical centers as one update to their common decoder
subtree. Word centers remain the output domain, and every center remains
addressable.

Phase 8H-0 is projection only. It may measure the complete field and fixed
queries, but it may not add a runtime route, mutate a package, implement a hot
executor, or use the dense oracle as an input to future execution.

## 2. Why This Follows From Phase 8G

Phase 8G chose one modal state for an atom over all `852,582` centers. That
scope was too coarse:

```text
atoms                                  218,763
global non-absent modal atoms                 4
global absent modal atoms               218,759
original relations                  110,928,005
global-modal residual events         107,517,677
```

The field is globally sparse but locally correlated. Inflected forms and words
with a shared prefix occupy common decoder-trie regions and often share exact
atom relation states. The next legal question is therefore not whether another
global scheduler can skip events. It is whether a static lexical region can own
one exact relation event on behalf of all its terminals.

The existing package already contains the required independent topology:

```text
primary terminals                       852,582
decoder nodes                          1,584,307
decoder edges                          1,584,306
terminal collisions                            0
maximum terminals per decoder node             1
maximum surface length                        31
forward-index derived RSS             28,759,240 B
forward-index build and validation          250 ms
package load                                  696 ms
```

These are baseline facts, not evidence that subtree cover will be small.

## 3. Frozen State Algebra

For atom `i` and center `w`, retain the Phase 8G state exactly:

```text
s_i(w) = absent
       | relation(strength, position_mode)
```

For observed query atom `q_i`:

```text
A(q_i, absent)   = (0, 0, 0, 0)
A(q_i, relation) = (exact mass contribution,
                     1,
                     surface-channel hit,
                     keyboard-channel hit)

A_Q(w) = sum over query atoms i of A(q_i, s_i(w))
```

All four coordinates of `ForwardActivation` are part of identity. A cover that
preserves only mass is invalid.

## 4. Decoder Tree Domain

Let `T` be the rooted decoder trie. A center is attached to its exact decoder
terminal node. Define `W(v)` as all centers attached to node `v` or any
descendant of `v`.

The proof derives forward child edges, subtree terminal counts, preorder and
postorder from the immutable decoder parent links. It must prove:

```text
one root
acyclic parent topology
one unique child symbol per parent
every primary center attached exactly once
every center appears in exactly one root subtree
subtree terminal counts exact
preorder intervals properly nested or disjoint
package hash unchanged
```

The current corpus happens to have no terminal collisions, but implementation
must preserve the general case where a decoder node owns more than one center.

## 5. Homogeneous Cover

A token `(v, r)` is valid for atom `i` only when every center in `W(v)` has the
same non-absent relation state `r`:

```text
for all w in W(v): s_i(w) = r != absent
```

Tokens for one atom must be disjoint. Their represented terminal union must be
exactly the non-absent posting domain for that atom.

Build the maximal cover recursively:

```text
cover(i, v):
  if W(v) is empty
    emit nothing

  if every center in W(v) has one identical non-absent state r
    emit (v, r)

  otherwise
    encode any center attached directly to v as terminal singleton state
    recurse into every child containing a non-absent state
```

An absent center may never be included in a non-absent token. Different
strength or position states may never share a token.

Two token kinds are required:

```text
Subtree(node_id, relation_state)
  -> applies to every center in W(node_id)

Terminal(terminal_rank, relation_state)
  -> applies only to the center attached directly to one decoder node
```

`Terminal` is necessary when a word is itself a prefix of longer words. A
subtree update at that decoder node would incorrectly update its descendants.

### Minimality for the frozen trie

The recursive maximal cover uses the minimum number of valid subtree tokens for
one atom on this fixed tree.

Proof by induction:

1. If `W(v)` is homogeneous and non-absent, one token is feasible and no cover
   can use fewer than one token.
2. If `W(v)` is not homogeneous, no valid token may cover `v`. Every valid
   cover must partition the directly attached terminals and child subtrees.
3. Applying the induction hypothesis to each independent child gives a minimal
   union.

This proves minimality only for the frozen decoder trie. It does not claim that
the decoder trie is the globally optimal learned partition.

### Bounded projection construction

The projection must not scan all `1,584,307` decoder nodes for every atom. Build
one DFS terminal order and record for every decoder node:

```text
first terminal rank
subtree terminal count
parent
children
direct terminal ranks
```

For one atom, map each posting terminal to DFS rank, sort by rank, and compute
maximal contiguous runs of one exact relation state. A decoder subtree is a
valid token exactly when its terminal interval is wholly contained in one such
run.

At a current run position only decoder nodes whose first terminal rank equals
that position can start a cover token. Each decoder node contributes to exactly
one such start list, so all start lists contain `decoder_nodes` entries in
total, not `decoder_nodes * atoms`. Test candidate nodes largest-first, emit the
largest valid subtree, advance by its terminal count, and otherwise emit one
terminal token.

This is the same maximal recursive cover expressed as an interval sweep. The
projection reports mapping, sorting, interval-test, and token counts separately.
It may not use corpus lexical order as if it were decoder DFS order.

### Event bound

Every token represents at least one original relation, and tokens for an atom
are disjoint. Therefore:

```text
cover_events_i <= original_relations_i
sum cover_events <= complete_forward_relations
```

This event bound does not prove package size or latency. Both are measured
separately.

## 6. Exact Query Semantics

For each observed atom, decode every cover token. The token contributes the
complete activation vector to its subtree root:

```text
subtree_lazy(v) += A(q_i, relation_state_of_subtree_token)
terminal_lazy(t) += A(q_i, relation_state_of_terminal_token)
```

For center `w` attached to decoder node `t(w)`:

```text
cover_activation_Q(w)
  = terminal_lazy(w)
  + sum subtree_lazy(v) over ancestors v of t(w)
```

Because each atom covers exactly the centers in its original posting with the
same state, every atom contributes exactly once when present and zero when
absent. Hence:

```text
cover_activation_Q(w) = dense_complete_posting_activation_Q(w)
```

This equality includes mass, hits, surface hits, and keyboard hits.

## 7. Sparse Activated-Trie Readout

Scanning all decoder nodes or all centers is forbidden in a future hot query.
The projection therefore measures the exact sparse structure needed by a legal
executor.

Let `U_Q` be nodes receiving at least one token update. Let `C_Q` be `U_Q` plus
all ancestors needed to connect those nodes to the root. `C_Q` is the activated
ancestor closure.

Within `C_Q`, propagate path activation. At each closure node, two kinds of
regions exist:

```text
active child region
  -> recurse because a deeper update changes some center

complement region
  -> direct terminals plus inactive child subtrees
  -> every center has exactly the current path activation
  -> one symbolic score cohort with an exact multiplicity
```

The complete score field is therefore a multiset of symbolic uniform cohorts,
not a vector scan. Select the exact K-th mass from cohort masses and
multiplicities. Expand terminal IDs only for cohorts with mass at or above the
K-th boundary. Equality is retained exactly.

The concrete retained-ID expansion is part of query work, not free metadata.
The current consumer merges concrete posting terminal IDs with Phase 7D typed
terminal IDs, so every center in every retained equality cohort must be charged
once. A future consumer may avoid that expansion only after a separate typed
symbolic-cohort contract proves equivalent union and readout semantics.

A decoder node with a direct terminal and descendants is handled explicitly:
the direct terminal has `path_activation + terminal_lazy`, while inactive child
subtrees have `path_activation`. They may share one cohort only when all four
activation coordinates are equal.

The projection builds the full ancestor closure for correctness accounting. A
future executor may replace repeated parent climbs with an Euler-order virtual
tree, but it must produce the same closure, cohort multiplicities, K-th mass,
and retained terminal IDs.

If the K-th mass is zero, the implicit zero cohort remains symbolic until the
consumer explicitly requests terminal IDs. A future runtime must map this case
to its declared abstention/empty-evidence contract; silently dropping zero
ties is forbidden.

## 8. Why Fixed Blocks Are Not The First Experiment

A fixed disjoint block modal field is exact, but it forces one block scale on
every atom. The decoder subtree cover strictly includes variable lexical
scales: one atom may use a large prefix subtree while another falls back to
small subtrees or terminal singletons.

For any partition whose blocks are decoder subtrees, the maximal homogeneous
cover uses no more atom events than that partition. Testing fixed decoder blocks
first would therefore test a weaker member of the same family.

An arbitrary learned clustering tree could outperform the decoder trie, but it
adds a second topology, training objective, mapping, package bytes, and failure
surface. It is not admitted until the existing topology is measured and found
insufficient.

## 9. Projection Metrics

### Deterministic physical projection

The projected replacement package reuses the immutable compact base and
decoder parent links. It replaces only the complete forward posting payload:

```text
outer header                         128 B
compact base                         exact V8 base bytes
atom index                           16 B per atom
32-atom shard index                  16 B per shard
token payload                        zstd level 19

token record
  kind                               Subtree | Terminal
  address                            decoder node ID | DFS terminal rank
  relation state                     strength + position_mode
```

Token lanes are deterministic and independently decodable per atom. The
projection must include kind tags, addresses, state bytes, indices, alignment,
and all compressed shards. It must not assume that the proof-side forward index
is free: derived startup RSS and build time are recorded separately. The
current measured forward decoder index costs `28,759,240 B` resident and
`250 ms` after package load.

### Complete package projection

```text
decoder nodes and edges
subtree terminal-count parity
original relation events
cover token events
tokens by subtree cardinality bucket
terminal singleton tokens
maximum and p50/p95/p99 token cardinality
per-channel original and cover events
per-atom original and cover events
state omissions / duplicates / overlaps
raw token bytes
compressed token bytes at fixed zstd-19
atom index bytes
tree metadata bytes not already in compact base
complete replacement package bytes
deterministic payload SHA-256
build time and peak RSS
```

The byte projection must encode the complete replacement field, not selected
query atoms and not a sampled extrapolation.

### Per-query projection

```text
query postings
original relation events
cover token events
unique updated nodes
naive ancestor insertions
unique activated ancestor-closure nodes
virtual-tree edges
symbolic uniform cohorts
maximum symbolic cohort multiplicity
retained terminal-ID expansions at or above K/equality
retained terminal IDs at K/equality boundary
dense versus symbolic retained-ID symmetric difference
dense activation mismatches
K-th mass mismatch
equality closure mismatch
typed-union target loss
projection time, reported as proof cost only
```

### Distribution requirement

Report aggregate maxima and every fixed damage class separately. A small
aggregate cannot hide one scattered channel or damage class.

## 10. Projection Admission Gates

### Gate A: topology and exactness

```text
decoder topology violations                         0
posting-state omissions                             0
posting-state duplicates                            0
cover overlaps                                      0
cover events > original relations                   0
dense activation field mismatches                   0
K-th mass mismatches                                0
equality closure mismatches                         0
typed target losses                                 0
typed-union schedule mismatches                     0
```

### Gate P0: representation and work

```text
complete replacement package              <=195 MiB
maximum query cover tokens                   <=100,000
maximum activated ancestor-closure nodes     <=100,000
maximum symbolic cohort records              <=100,000
maximum retained terminal-ID expansions       <=100,000
tokens + closure + cohorts + retained IDs      <=100,000
full-center scans                                    0
full-decoder-node scans                              0
```

The individual and combined `100,000` limits are conservative projection
admission screens, not runtime caps. The combined screen reserves the measured
`2.5 ms` posting allocation for all major sparse operations rather than spending
it entirely on token decode. Nothing may be truncated to meet a screen. If one
fails, the simple decoder-trie executor is rejected before code.

Gate P0 does not prove `<=5 ms`. It only admits an executor microarchitecture
and physical package round-trip.

### Gate B0: future executor

```text
cover decode plus sparse exact readout max        <=2.5 ms
complete typed plus posting contour max           <=5.0 ms
correctness losses                                      0
runtime allocations after warmup                        0
```

### Gate B1: physical runtime

```text
actual package                                  <=195 MiB
cold startup and steady RSS              Phase 0 limits
hot p99 including preparation                    <=5 ms
```

## 11. Stop Conditions

Stop before executor code when any condition is true:

- a token contains an absent center or two relation states;
- cover expansion differs from complete postings;
- fixed-trie cover events or ancestor closure exceed projection gates;
- retained equality cohorts require more than `100,000` concrete terminal IDs;
- replacement bytes exceed `195 MiB`;
- the result is small only after target-aware ordering, atom selection,
  truncation, equality pruning, or a generated candidate list;
- query-local compressed bytes are presented as a package projection;
- proof timing is presented as runtime latency.

If the decoder topology fails, retain the receipt and analyze the first shared
scattering mechanism by atom channel. Do not tune individual heldout words.

## 12. Ownership and Isolation

```text
proof CLI
  -> Phase8H projection orchestrator
       -> immutable complete-posting adapter
       -> immutable decoder topology adapter
       -> maximal homogeneous cover builder
       -> complete package projector
       -> query token projector
       -> activated-closure projector
       -> symbolic cohort readout projector
  -> dense complete accumulator          independent proof owner
  -> typed Phase 7D traversal            independent evidence owner
  -> exact parity/admission gates        proof owner
  -> Phase 8H receipt                    observer

production runtime -X-> Phase 8H projection
dense oracle       -X-> cover construction
heldout target     -X-> cover construction or query readout
projection budgets -X-> token truncation
proof result       -X-> edit authority
```

## 13. Decision Tree

```text
Phase 8G global modal residual                REJECTED
  |
  v
Phase 8H-0 decoder-subtree cover projection   NEXT
  |-- topology/accounting FAIL              -> reject implementation
  |-- activation/K/equality parity FAIL     -> reject theorem implementation
  |-- package >195 MiB                     -> reject physical representation
  |-- cover tokens >100k                   -> reject fixed decoder topology
  |-- ancestor closure >100k               -> reject sparse readout topology
  |-- symbolic cohorts >100k               -> reject sparse readout topology
  |-- retained ID expansions >100k         -> reject sparse readout topology
  `-- Gate A + P0 PASS
        |
        v
Phase 8H-1 exact sparse executor
  |-- correctness FAIL                     -> reject executor
  |-- intrinsic latency >2.5 ms           -> reject executor
  `-- Gate B0 PASS
        |
        v
one deterministic physical package and B1
```

## 14. Baseline Evidence

```text
Phase 8G negative projection
  docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8G_2026-08-14/evidence/phase-8g-13x1.json
  SHA-256 3644e103754ded16a062dc45b0c3180d9a07ee852a9b362c1a8831c78d5b4482

decoder topology baseline
  docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8H_2026-08-14/baseline/decoder-index.json
  SHA-256 d1d0ed88fad2a23fcc830ac6872633f1924d79fb47046b96229bd77ed9c16a49
```

No crystallization, package mutation, daemon restart, IBus restart, version
bump, deployment, or runtime authority change is admitted by this paper.

## 15. Measured Result

The fixed remote `13 x 1` projection completed over the unchanged package and
the fixed Phase 8 manifest.

```text
schema                                  lay.l11.posting-search-phase8-proof.v8
correctness                             PASS
final verdict                           REJECT_REPRESENTATION
runtime authority changed               no
package changed                         no
current package                         190,139,182 B
projected replacement package           317,591,731 B = 302.879 MiB
package limit                           204,472,320 B = 195 MiB
package excess                          113,119,411 B
projection wall                         25.04 s
average CPU                             1,158%
peak RSS                                3,567,448 KiB
```

Exactness and isolation held:

```text
complete relations verified             110,928,005
activation reconstruction mismatches              0
activation histogram mismatches                   0
K/equality mismatches                              0
retained-ID symmetric difference                  0
typed target losses                               0
typed-union schedule mismatches                    0
package SHA before/after                  identical
```

The fixed decoder topology does not factor the field enough:

```text
original relation events                110,928,005
cover token events                        83,525,208
event reduction                               24.703%
singleton tokens                          76,510,830
singleton share                                91.602%
atoms unchanged                           124,970 / 218,763 = 57.126%
compressed token payload                 236,021,443 B

maximum query cover tokens                 4,639,567  FAIL
maximum activated closure nodes              967,193  FAIL
maximum symbolic cohorts                     742,029  FAIL
maximum retained terminal-ID expansions          145  PASS
maximum combined work                      6,288,648  FAIL
projection work limit                        100,000
```

All 13 classes fail the same work mechanism. The smallest combined query is
`2,645,468` work units and the largest is `6,288,648`; this is a `26.5x-62.9x`
miss before an executor exists. The retained-ID accounting amendment does not
cause the rejection: equality output remains only `128-145` IDs in this fixed
screen.

Per-channel cover reduction confirms that lexical prefix topology is the wrong
partition for positional n-gram state:

| Channel family | Cover reduction |
|---|---:|
| boundary position | `49.872%` |
| character/keyboard bigram | `32.464%` |
| character/keyboard trigram and bag | `24.232-24.292%` |
| byte gram | `21.269%` |
| character/keyboard skip gram | `18.560-18.562%` |

An n-gram may occur at many depths and positions below the same lexical prefix;
different lengths also change its normalized position state. Consequently most
decoder subtrees contain absent centers or several exact relation states, so the
minimal legal cover falls back to singleton or tiny tokens. This is the first
shared scattering mechanism, not a scheduler defect.

The physical encoding then loses twice. It removes only `24.703%` of relation
events, while subtree/terminal tags and mixed node/rank addresses compress to
`236,021,443 B`, versus `108,568,894 B` for the current terminal-delta posting
payload. A different scheduler over this cover cannot repair either the package
or work gate.

Tested: complete field construction, exact state cover, all four activation
coordinates, exact K/equality, typed evidence union, deterministic compressed
projection, package isolation, and one fixed case per damage class. Not tested:
an executor, production latency, a physical package round trip, the full
`13 x 20,000` quality matrix, nonlinear settlement, daemon/IBus, installation,
or live authority. Those tests cannot reverse the preregistered P0 rejection and
are intentionally not run.

Evidence:

```text
docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8H_2026-08-14/evidence/phase-8h-13x1.json
  SHA-256 897ea676f018fab4fe37e332c6c87ab662112cc1a492244f4ec123b9f51e5f9d

docs/structural_gates/receipts/L1_L11_PEAK_SEARCH_PHASE_8H_2026-08-14/evidence/phase-8h-13x1.time.txt
  SHA-256 3dece0041e115f96ff64045f15c96a0a0f065812cced77256e8c16536b98a441
```
