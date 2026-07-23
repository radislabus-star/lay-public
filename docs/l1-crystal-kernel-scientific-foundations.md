# L1 Crystal Kernel: Scientific Foundations and Scaling Contract

Status: normative architecture reference for the experimental shadow kernel.

Snapshot date: 2026-07-23.

This document records the scientific results, mathematical guarantees and
engineering hypotheses used to evolve the L1 crystal kernel. It is the default
reference for L1 capacity, retrieval and proof decisions. A newer experiment
may replace a parameter, but it must not silently weaken the contracts below.

## 1. Problem Statement

L1 must restore a lexical center from a damaged surface without scanning the
whole vocabulary and without allowing vocabulary growth to delete learned
relations.

```text
damaged surface
-> typed n-gram atoms
-> complete learned Atom -> WordCenter relations
-> sparse candidate retrieval
-> forward/reconstruction/phase/anti interference
-> unique lexical center or ABSTAIN
```

The target scale is one bilingual field containing 300,000 Russian and 300,000
English word forms. The working quality target is strictly above 95% top-1 for
every unique damage class. Clean preservation and candidate-set coverage have
stricter gates because a missing candidate cannot be recovered by later L1
ranking.

## 2. Evidence Levels

Every architecture statement belongs to one of three evidence levels:

```text
THEOREM       follows from stated mathematical assumptions
LITERATURE    supported by a cited published result
HYPOTHESIS    a Lay-specific design choice that still requires ablation
```

Numbers such as 256 phase cells, 22 stored components, a 32-relation posting
block or a 1,024-center runtime ceiling are `HYPOTHESIS`, not universal
constants. They are promoted only by fixed heldout proof.

## 3. Scientific Basis

### 3.1 Sparse associative memory

Willshaw, Buneman and Longuet-Higgins showed that sparse binary associative
connections can achieve high information capacity without a dense copy of
every pattern. Tsodyks and Feigel'man showed that low activity increases
associative-memory storage capacity.

Lay consequence:

```text
large stored vocabulary
does not imply
large simultaneously active lexical field
```

The runtime should activate a sparse candidate set while preserving all learned
relations in cold storage.

### 3.2 Dense and modern Hopfield memory

Krotov and Hopfield demonstrated higher-order associative energies that store
more patterns than classical pairwise Hopfield memory. Ramsauer et al. showed
that modern continuous Hopfield networks can have exponential capacity in the
associative dimension under pattern-separation assumptions, and related the
update to attention.

Lay consequence: field dimension and separation matter. Vocabulary size alone
does not determine capacity. Closely packed lexical centers can still form
metastable mixed states, so an exponential-capacity headline is not permission
to accept an unseparated winner.

### 3.3 Sparse distributed memory and attention

Bricken and Pehlevan related attention to Kanerva-style sparse distributed
memory under explicit data conditions.

Lay consequence: content-addressed sparse retrieval is compatible with a wave
readout, but retrieval and authority remain separate. Retrieval may expose a
center to interference; it may not authorize a correction.

### 3.4 Exact dynamic pruning

WAND and Block-Max WAND evaluate top-k retrieval through sound score upper
bounds. A posting or block is skipped only when it cannot beat the current
threshold.

Lay consequence: `Atom -> WordCenter` relations are stored as complete,
compressed postings. A fixed compile-time `truncate(256)` is not a scalable
retrieval policy. Runtime work is bounded by upper-bound pruning, not by
deleting learned links.

Recent sparse-retrieval work continues to use efficient inverted indexes over
learned sparse representations. Approximate retrieval can be evaluated as an
ablation, but the L1 safety baseline uses exact upper bounds.

### 3.5 Adaptive prediction sets

Romano, Sesia and Candes introduced classification sets with valid adaptive
coverage. Angelopoulos et al. added regularization that keeps conformal sets
small while retaining a finite-sample marginal coverage guarantee.

Lay consequence: the candidate frontier should have variable size. A clear
surface may require only a few centers; a damaged or ambiguous surface may
require hundreds. Frontier size is calibrated from heldout evidence instead of
being inferred directly from vocabulary size.

### 3.6 Learned spelling-error channels

Brill and Moore showed that spelling correction improves when the noisy channel
learns multi-character edit probabilities. Schulz and Mihov showed that
Levenshtein automata can search large dictionaries without materializing every
damaged spelling.

Lay consequence: corruption episodes are cold training/proof evidence, not hot
runtime records. Damage classes must test structural coverage, while learned
couplings and reconstruction carry the runtime knowledge.

## 4. Mathematical Contracts

### 4.1 Fixed-frontier impossibility

For every fixed frontier size `K`, a vocabulary can contain `K + 1` centers
that are indistinguishable under the currently observed atoms. A correct center
may therefore occupy rank `K + 1` and be deleted before interference.

```text
THEOREM:
no vocabulary-independent fixed K guarantees candidate recall
for an unrestricted growing dictionary
```

This does not require every query to use an unbounded frontier. It requires the
system to adapt, take a sound slow path, or abstain when evidence is too dense.

### 4.2 Sound-pruning preservation

Let `score(c)` be a candidate's exact retrieval score and `UB(c)` a sound upper
bound such that `score(c) <= UB(c)`. Let `tau` be the current kth exact score.
Discarding a candidate or posting block only when `UB < tau` cannot change the
exact top-k result.

For query atom `a`, a posting contribution is:

```text
contribution(a, c)
  = coupling_strength(a, c)
  * observed_atom_weight(a)
  * position_coherence(a, c)
```

A block bound is the sum of its maximum still-possible contributions for all
active query atoms. The implementation must test that pruned and exhaustive
retrieval return identical ordered top-k results.

### 4.3 Conformal coverage

Let a calibration set be exchangeable with future queries and let `S(x, y)` be
the chosen nonconformity score. With the finite-sample conformal quantile
`q_alpha`, the prediction set

```text
C_alpha(x) = { y : S(x, y) <= q_alpha }
```

has marginal coverage of at least `1 - alpha`, subject to the conformal
assumptions and tie handling used by the implementation.

For the L1 candidate-birth target:

```text
alpha = 0.001
target marginal candidate coverage = 99.9%
```

This guarantee applies to candidate-set inclusion, not top-1 correctness and
not arbitrary distribution shift. Language, length and damage-density buckets
may use Mondrian calibration only when every bucket has enough calibration
evidence.

### 4.4 Authority boundary

Neither associative-memory capacity, WAND retrieval nor conformal coverage
proves that the first center is correct. L1 authority still requires a unique
settled peak with sufficient margin. Otherwise the result is `ABSTAIN` or a
lattice for later context layers.

## 5. Canonical 600k Kernel Hypothesis

```text
lexical terminals                 600,000
Russian shard                     300,000
English shard                     300,000
shared keyboard-layout bridge     enabled
phase dimension                   256 complex cells
WordCenter64                      64 bytes
stored word components            22
AtomWaveCode                      16 bytes
WaveCoupling                       8 bytes
posting block                      32 relations
forward postings                  complete; no fixed per-atom truncation
reverse reconstruction            96 lexical + all ordered char anchors
anti modes                         adaptive 4..32 clustered modes
candidate set                      conformal, coverage target 99.9%
runtime hard ceiling               1,024 centers, then ABSTAIN/slow path
settling iterations                3
```

The RU and EN shards reduce irrelevant cross-script collisions. They do not own
the final decision. Keyboard atoms can retrieve the opposite shard for layout
repair, after which all retrieved centers enter one calibrated interference
field.

The 256-cell dimension, posting block size, anti range and hard runtime ceiling
must each survive an independent ablation. They must not be changed together in
a way that hides which mechanism caused an improvement.

The first isolated format proof accepted 32-relation blocks on the existing 10k
field:

```text
forward relations                 3,229,043
raw forward bytes                25,832,344
compressed forward bytes         11,751,841
compression ratio                    2.198x
average bytes per relation            3.639
whole package bytes        41,550,288 -> 27,469,785
coupling roundtrip parity              PASS
exact top-64 parity                  500/500
```

This proves the v3 representation candidate, not the 600k quality target and
not Block-Max WAND runtime pruning. The receipt is
`docs/structural_gates/receipts/L1_FORWARD_POSTING_CODEC_V3_10K_2026-07-23.json`.

A full fixed-corpus proof then compared v3 against the existing v24 baseline on
the same `data/lexicon/l2_surface_foundation_ru_100k.txt` 10k slice. Aggregate,
per-class, coverage and false-certainty metrics are identical. The package
changed from 42,201,208 to 28,120,705 bytes. Full class metrics are stored in
`docs/structural_gates/receipts/L1_CRYSTAL_FORMAT_V3_10K_FIXED_2026-07-23.json`.

### 5.1 Evidence scope of the v3 codec result

The following are measured facts:

```text
block codec roundtrips every forward relation
v2 input is readable by the v3 implementation
500 sampled clean top-64 readouts are byte-for-byte equivalent
forward representation is 2.198x smaller on the real 10k package
```

The codec experiment did not measure:

```text
top-1 accuracy by corruption class
full heldout quality parity
600k compile or runtime behavior
Block-Max WAND pruning
live daemon or IME behavior
```

The subsequent fixed-corpus proof measured the complete current 10k quality
matrix and established exact v24-to-v3 quality parity. It did not establish the
600k target or improve current class accuracy. A format or compression PASS
must never be reported as a lexical-quality PASS.

### 5.3 Complete forward-field ablation

The fixed 10k v26 proof removed the compile-time `truncate(256)` while keeping
the corpus, damage split, phase field, reconstruction and proof unchanged.

```text
relations before policy             5,321,204
relations retained by baseline      3,229,043
relations dropped by cap            2,092,161
atoms above cap                         4,574
maximum atom degree                    10,000

aggregate top-1              91.823% -> 96.293%
clean preservation           98.980% -> 100.000%
top-8                        99.145% -> 99.905%
top-64                       99.633% -> 99.993%
classes above 95%                 4/13 -> 9/13
package bytes             28,120,705 -> 35,070,906
hot p50 / p99                   3.503 / 4.182 ms
```

Every damage class improved:

```text
class                           baseline   complete    delta
adjacent transposition           95.756%    99.248%   +3.492 pp
double substitution              75.961%    86.777%  +10.815 pp
extra letter                     93.552%    98.849%   +5.298 pp
layout projection                94.722%    98.425%   +3.702 pp
letter substitution              90.538%    95.816%   +5.278 pp
missing letter                   96.296%    99.386%   +3.090 pp
non-adjacent transposition       81.500%    91.738%  +10.238 pp
omission + transposition         84.508%    92.566%   +8.058 pp
prefix truncation                93.265%    98.252%   +4.987 pp
punctuation suffix               97.863%    99.980%   +2.118 pp
repeated fragment                86.609%    93.737%   +7.127 pp
sparse multi-omission            90.409%    95.037%   +4.628 pp
suffix truncation                96.245%    99.374%   +3.129 pp
```

Complete postings are therefore accepted as a causal architecture improvement,
but the package remains `WATCH_shadow`. Four classes remain below the strict
working gate: double substitution, non-adjacent transposition, omission plus
transposition and repeated fragment. False certainty is not zero. Current anti
interference is also slightly harmful on this field: 5 improved, 8 worsened,
`anti_ablation_drop = -3`.

Receipt:
`docs/structural_gates/receipts/L1_COMPLETE_FORWARD_POSTINGS_V26_10K_2026-07-23.json`.

### 5.4 Bidirectional sequence interference V27

V27 kept the fixed 10k corpus and complete forward field, then replaced the
binary omission-only reconstruction signal with one generic character-anchor
field combining ordered LCS coherence, multiset mass and relative length. It
also allowed the field to act when surface and keyboard hit counts were equal.

Measured against V26:

```text
class                              V26        V27       delta
adjacent transposition          99.248%    99.490%   +0.242 pp
double substitution             86.777%    86.608%   -0.169 pp
extra letter                    98.849%    99.719%   +0.870 pp
layout projection               98.425%    99.291%   +0.866 pp
letter substitution             95.816%    96.068%   +0.252 pp
missing letter                  99.386%    98.517%   -0.869 pp
non-adjacent transposition      91.738%    93.803%   +2.066 pp
omission + transposition        92.566%    93.669%   +1.103 pp
prefix truncation               98.252%    97.481%   -0.771 pp
punctuation suffix              99.980%    99.980%    0.000 pp
repeated fragment               93.737%    97.970%   +4.233 pp
sparse multi-omission           95.037%    92.824%   -2.213 pp
suffix truncation               99.374%    99.017%   -0.358 pp
```

```text
aggregate top-1                 96.293% -> 96.509%
clean preservation             100.000% -> 100.000%
sequence improved / worsened               789 / 103
sequence ablation drop                         +686
top-64                                      99.993%
package bytes                             35,070,906
hot p50 / p99                         3.570 / 4.267 ms
classes above 95%                              9/13
```

The generic sequence relation is causally useful, but V27 is rejected as a
promotion candidate. It closes repeated fragment and improves two other hard
classes, while broad equal-lane activation loses the previously passing sparse
multi-omission class and increases false-certainty failures in some ambiguous
classes. Aggregate improvement cannot compensate for that regression.

This experiment did not test 600k scale, live daemon/IME behavior or runtime
authority. Runtime authority did not change. The next ablation must preserve
the V26 ordered-subsequence behavior as an explicit baseline and permit only a
constructive extension under strict surface-lane ownership.

Receipt:
`docs/structural_gates/receipts/L1_SEQUENCE_INTERFERENCE_V27_10K_2026-07-23.json`.

### 5.5 Conservative sequence extension V28

V28 floored partial sequence coherence at the neutral value and restored strict
surface-lane ownership. It measured 655 improvements and 282 worsenings against
an attempted in-process legacy route, with aggregate top-1 96.472%, clean
preservation 100%, top-64 99.993%, package size 35,070,906 bytes and hot p99
4.270 ms. The scoreboard remained 9/13.

V28 is rejected. Its attempted legacy route used surface/keyboard ownership,
but session-history recovery proved that V26 actually activated omission
reconstruction from the structural relation `observed length < center length`.
Therefore the V28 legacy comparison was not a valid reproduction of V26, and
its causal numbers cannot authorize promotion. This is a proof-harness finding,
not evidence against the recovered V26 omission mechanism. Live runtime and
authority did not change.

Receipt:
`docs/structural_gates/receipts/L1_CONSERVATIVE_SEQUENCE_V28_10K_2026-07-23.json`.

### 5.6 Length-partitioned sequence V29

V29 separated omission geometry from equal/longer sequence geometry. Aggregate
top-1 was 96.476%, clean preservation 100%, top-64 99.993%, package size
35,070,906 bytes and hot p99 4.247 ms. Repeated fragment remained above gate at
97.970% and non-adjacent transposition rose to 94.163%, but omission plus
transposition fell to 90.072% and sparse multi-omission remained below gate at
93.830%.

V29 is rejected because its in-process legacy route still failed V26 parity.
Exact source-history recovery found the missing lease: V26 required every raw
input character to resolve to an anchor before omission reconstruction could
act. V29 omitted that all-anchor coverage condition, allowing an incomplete
observed sequence to become false positive evidence. Runtime authority did not
change and no live route was tested.

Receipt:
`docs/structural_gates/receipts/L1_PARTITIONED_SEQUENCE_V29_10K_2026-07-23.json`.

### 5.7 Coverage-leased sequence V30

V30 restored the complete V26 omission contract, including the requirement
that every observed character resolve into the anchor sequence. Raw parity is
exact:

```text
V26 aggregate top-1 count             85,184
V30 LegacySequence top-1 count        85,184
per-class top-1 count parity           13/13
```

This validates the in-process legacy control. The first report labelled its
all-case legacy percentage too generically; subsequent proof output separates
all-case and unique-case legacy percentages.

The V30 extension itself remains rejected:

```text
aggregate top-1                 96.293% -> 96.476%
clean preservation             100.000% -> 100.000%
repeated fragment               93.737% -> 97.970%
non-adjacent transposition      91.738% -> 94.163%
omission + transposition        92.566% -> 90.072%
sparse multi-omission           95.037% -> 93.830%
sequence vs legacy improved/worsened       456 / 294
top-64                                      99.993%
package bytes                             35,070,906
hot p99                                     4.242 ms
```

Candidate-local length partitioning is insufficient because a generalized
sequence boost in one length relation can still overtake the V26 winner from
another relation. The next experiment must choose the winning baseline length
stratum first and permit generalized sequence interference only inside it.
Runtime authority did not change; 600k and live routes were not tested.

Receipt:
`docs/structural_gates/receipts/L1_COVERAGE_LEASED_SEQUENCE_V30_10K_2026-07-23.json`.

### 5.8 Baseline-stratified sequence V31

V31 selected the winning V26 length stratum before permitting generalized
sequence interference. It reduced sequence-vs-legacy worsenings from 294 to 33
and preserved exact V26 unique metrics as an explicit control, but it did not
cross the class gate:

```text
aggregate top-1                 96.293% -> 96.386%
repeated fragment               93.737% -> 94.082%
non-adjacent transposition      91.738% -> 93.309%
omission + transposition        92.566% -> 93.381%
sparse multi-omission           95.037% -> 93.964%
sequence vs legacy improved/worsened       115 / 33
```

V31 is rejected. A length stratum is too coarse: partial sequence evidence can
still suppress an exact ordered-subsequence center inside the same stratum, and
the stratum boundary also blocks useful repeated-fragment corrections. The
next gate is evidence-owned: exact legacy subsequence coherence is a veto over
partial waves regardless of candidate length. Runtime authority did not
change; 600k and live routes were not tested.

Receipt:
`docs/structural_gates/receipts/L1_STRATIFIED_SEQUENCE_V31_10K_2026-07-23.json`.

### 5.9 Exact-evidence certificate V32

V32 replaced length-stratum ownership with an evidence certificate: if any
candidate has exact ordered-subsequence coherence 1000, weaker partial waves
cannot suppress it. Without an exact certificate, partial waves remain active.

```text
aggregate top-1                 96.293% -> 96.583%
clean preservation             100.000% -> 100.000%
classes above 95%                    9/13 -> 10/13
repeated fragment               93.737% -> 97.927%
sparse multi-omission           95.037% -> 95.037%
missing letter                  99.386% -> 99.386%
prefix truncation               98.252% -> 98.252%
suffix truncation               99.374% -> 99.374%
sequence vs legacy improved/worsened       317 / 61
top-64                                      99.993%
package bytes                             35,070,906
hot p99                                     4.267 ms
```

The certificate is accepted as a causal architecture improvement because it
closes one damage class without moving any previously passing class below the
working threshold. Total L1 promotion remains `WATCH_shadow`: double
substitution, non-adjacent transposition and omission plus transposition remain
below 95%, false certainty is nonzero, and anti interference remains slightly
harmful. Runtime authority did not change; 600k and live routes were not tested.

Receipt:
`docs/structural_gates/receipts/L1_SEQUENCE_CERTIFICATE_V32_10K_2026-07-23.json`.

### 5.10 Composite-margin pairwise sequence V33

V33 allowed only one partial-wave winner when its composite sequence score led
the lattice by at least 32 milli-units. It is rejected:

```text
aggregate top-1                 96.583% -> 96.375%
sequence vs legacy improved/worsened       149 / 77
repeated fragment                         95.119%
double substitution                       86.565%
non-adjacent transposition                 92.501%
omission + transposition                   92.662%
```

The composite score is not a valid directed edge because it mixes ordered
coherence with multiset mass. A false candidate can retain similar mass after
multiple substitutions and win the composite margin. The next ablation keeps
the same bounded LCS computation but separates pure ordered coherence for edge
selection. Runtime authority did not change; 600k and live routes were not
tested.

Receipt:
`docs/structural_gates/receipts/L1_PAIRWISE_SEQUENCE_V33_10K_2026-07-23.json`.

### 5.11 Ordered-margin pairwise sequence V34

V34 replaced the V33 composite margin with a pure ordered-sequence margin.
This separates sequence order from multiset mass, but it is still rejected:

```text
aggregate top-1                              96.409%
sequence vs legacy improved/worsened         123 / 21
double substitution                          86.946%
non-adjacent transposition                    91.738%
omission + transposition                      91.751%
repeated fragment                             93.866%
```

Pure ordered similarity recovers some individual cases, but it discards the
repeated-fragment and non-adjacent evidence preserved by the accepted V32
certificate. Neither a composite absolute sequence score nor a pure ordered
absolute score is a learned directed relation between two competing centers.
The working tree therefore returns to V32. The next experiment must learn
`target > real lattice competitor` relations from training surfaces and use
unknown or contradictory relations as Neutral. Runtime authority did not
change; 600k and live routes were not tested.

Receipt:
`docs/structural_gates/receipts/L1_ORDERED_PAIRWISE_SEQUENCE_V34_10K_2026-07-23.json`.

### 5.12 Pairwise and stored-position rejection V35-V47

V35-V39 tested directed pair memory over real lattice competitors. Tightening
the certificate reduced pairwise damage from `44 improved / 3006 worsened` in
V35 to `0 / 0` in V39. The safe configuration was neutral: one-sided evidence
is Unknown and pairwise cannot create Support. This preserved safety but did
not solve the remaining double-substitution failures.

V40-V47 tested storing direct and relative character-position channels,
same-length isolation, length strata, surface-only position gain and dual
readout. The best 2k double-substitution result was `96.171%` in V42, but
omission + transposition fell to `92.941%`. The 10k stored-position run reached
non-adjacent transposition `95.676%` and double substitution `90.786%`, while
omission + transposition regressed to `92.912%`; p99 also exceeded budget at
`6.038 ms`. All stored-position variants were rejected and the additional
atom channel was removed. Position must be derived from the existing forward
and backward anchor sequences rather than persisted as another posting plane.

The V35-V47 receipts are preserved under
`docs/structural_gates/receipts/` with their experiment names and
`_2026-07-23.json` suffix. Runtime authority did not change.

### 5.13 Exact positional certificate V48-V54

V48 removed the `750` floor from one narrowly defined comparison: exact
same-position coherence between the observed `CharacterAnchor` sequence and a
candidate's clean reverse sequence. It added no stored atom channel and no
package memory. At 2k it raised double substitution to `96.396%` and
non-adjacent transposition to `97.179%`, with clean preservation `100%` and hot
p99 `2.705 ms`. It was rejected because an equal-length dictionary neighbor
could cross a length-changing basin: `69` cases improved but `1432` worsened,
including omission + transposition falling to `83.059%`.

V49 made basin ownership structural. A positional certificate may rerank an
equal-length basin, but may not displace the current winner when that winner
belongs to a length-changing basin. At 2k this restored omission +
transposition to `95.294%`, extra letter to `99.849%`, missing letter to
`99.493%`, sparse multi-omission to `98.141%`, clean preservation to `100%`,
and hot p99 to `2.689 ms`. The positional field was then nearly neutral at
`14 improved / 13 worsened`; double substitution remained `93.018%`, so V49
also remains `WATCH_shadow` and is not publishable.

The evidence from V48-V49 is that position is useful but is not an independent
winner owner. The next experiment must compose positional coherence with the
existing reverse-sequence certificate. A weaker positional relation cannot
override a stronger exact reconstruction wave. Runtime authority did not
change.

Receipts:

- `docs/structural_gates/receipts/L1_POSITION_CERTIFICATE_V48_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_BASIN_POSITION_V49_2K_2026-07-23.json`

V50 composed position with reverse-sequence ordering. It improved `42` cases
and worsened `15`; double substitution reached `94.369%`, but omission +
transposition regressed to `94.118%`. V51-V52 therefore added proof-only
transition diagnostics. They store no runtime memory and exposed the incumbent
and selected center's length relation, sequence coherence, positional
coherence and settled energy for changed heldout outcomes.

The diagnostics found two separable authority regimes:

```text
cross-length useful double-substitution energy deficit       <= 836
cross-length harmful length-changing energy deficit          >= 886
harmful omission incumbent sequence coherence              860..935
useful double-substitution incumbent sequence coherence     750..760
```

V53 tested an `850` cross-length energy lease and strict sequence improvement
inside equal-length basins. It reached double substitution `94.820%`, but four
omission + transposition outcomes still worsened because a strong incumbent
sequence was not yet protected.

V54 composes the certificates in authority order:

```text
exact reconstruction
-> strong reverse sequence
-> bounded energy lease
-> exact positional coherence
-> Neutral when evidence cannot cross the lease
```

At 2k, V54 produced `32 improved / 0 worsened`, clean preservation `100%`, hot
p99 `2.641 ms`, and every damage class above the strict `95%` top-1 threshold.
Double substitution reached `95.045%`; omission + transposition remained
`95.294%`. The overall verdict remains `WATCH_shadow` because ambiguous false
certainty is still non-zero in adjacent transposition, letter substitution,
missing letter and prefix truncation. The fixed 10k proof is required before
any promotion claim, and runtime authority remains unchanged.

The fixed 10k V54 proof rejected the 2k calibration. Double substitution was
only `90.279%`, omission + transposition `93.587%`, position changed `96`
correct and `22` incorrect outcomes, clean preservation was `99.99%`, and hot
p99 was `5.965 ms`. The package remained `44,888,171` bytes. Inspection of
the changed outcomes showed that the `850` energy separation disappeared at
10k: useful and harmful cross-length transitions overlap.

The 10k diagnostics also exposed an upstream authority defect. The V32 global
sequence certificate resets every partial reconstruction wave when any lattice
candidate has an exact legacy omission relation. With a denser lexicon, an
accidental longer subsequence candidate can therefore erase the useful reverse
wave of the current correct extra-letter or omission candidate even when that
accidental candidate does not own the winner. The next experiment must make
the sequence certificate winner-owned and measure its ablation before adding
rarity-weighted positional coherence. Fixed energy leases are not accepted as
a vocabulary-independent solution.

V55 made the sequence certificate winner-owned: an exact legacy subsequence
may suppress partial reverse waves only when that exact candidate is itself
the strongest legacy-energy center. At 10k this removed all positional
regressions (`83 improved / 0 worsened`) and raised omission + transposition
from `93.587%` to `94.407%`. Double substitution remained `90.152%`, however,
and the sequence certificate itself measured `192 improved / 80 worsened`.
The verdict remains `WATCH_shadow`; the next unresolved problem is dense
equal-length positional collision, not candidate birth.

V55 also split compile from proof. The package-proof route validates the
terminal count and every decoded terminal against the corpus before reusing a
package. Exact 2k parity passed after excluding timing and path fields. A 10k
runtime experiment can now reuse the immutable `44.9 MB` package with
`compile_ms = 0`; V55 proof took `482.677 s` and about `414 MB RSS` instead of
rebuilding roughly `4 GB` of compiler state. Runtime authority did not change.

V56 tested using stored atom `support` as an inverse-frequency tie-break for
equal raw positional coherence. It was rejected at 2k: rarity improved `1`
outcome and worsened `3`, while double substitution fell from `95.045%` to
`94.595%`. The stored support counts generated training-surface mass and may
saturate at `u16::MAX`; it is not clean lexicon document frequency and cannot
be reinterpreted as IDF. All V56 scoring code was removed. Future collision
evidence must come from the active lattice or from an explicitly trained and
proved memory field.

V57 added bounded training-only double-substitution variants with different
alphabet shifts on the selected position pairs. Exact heldout surfaces stayed
disjoint. At 2k, double substitution did not change (`95.045%`), while the
package grew from `10.65 MB` to `13.05 MB` and layout projection fell from
`99.010%` to `98.812%`. The extra unary surfaces diluted coupling mass without
learning which real competitor must lose. V57 was rejected and all variant
generation code was removed. The next mechanism must learn competition on the
actual lattice rather than add more unary corruption episodes.

V58-V59 revisited one-sided pair memory only inside equal-length candidates
with identical raw positional coherence. V58 remained safely Neutral because
the old unary coherence condition blocked every veto. Removing that unrelated
condition in V59 activated the aggregate pair center, but produced `1 improved
/ 16 worsened` and reduced double substitution to `93.243%`. Both variants
were rejected and removed. A one-sided bank cannot act until incompatible
surface scenes are split into bounded coherent subcenters; the aggregate center
is not a valid relation certificate.

V60-V61 implemented bounded phase clustering for pair relation scenes while
keeping unary anti memory on its previous aggregate. Neither `850` nor `950`
coherence produced a repeated additional subcenter: pair center count stayed
at `1151` or fell to `1150`, and pairwise remained `0 improved / 0 worsened`.
The extra modes occurred only once and correctly failed the support threshold.
Both clustering variants were removed. Current synthetic training does not
contain enough repeated pair-specific evidence to authorize this memory.

Additional receipts:

- `docs/structural_gates/receipts/L1_COMPOSED_POSITION_V50_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_POSITION_DIAGNOSTICS_V51_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_POSITION_ENERGY_DIAGNOSTICS_V52_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_ENERGY_LEASED_POSITION_V53_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_COMPOSED_LEASES_V54_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_COMPOSED_LEASES_V54_10K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_WINNER_OWNED_SEQUENCE_V55_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_WINNER_OWNED_SEQUENCE_V55_10K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_REUSED_PACKAGE_PARITY_V55_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_RARITY_POSITION_V56_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_DIVERSE_DOUBLE_TRAINING_V57_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_COLLISION_PAIR_VETO_V58_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_COLLISION_PAIR_VETO_V59_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_PAIR_SUBCENTERS_V60_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_STRICT_PAIR_SUBCENTERS_V61_2K_2026-07-23.json`

### 5.2 Architecture evidence maintenance

Every completed experiment must update this document or its owning architecture
document in the same change. The update must state:

```text
what was tested
measured facts and denominators
what was not tested
verdict and its exact scope
receipt path
whether runtime authority changed
```

Terminal output alone is not durable architecture evidence. Estimates,
hypotheses, measured facts and promotion claims must remain visibly separate.

## 6. Memory Forecast

The fixed primary center bank is exact:

```text
600,000 * 64 bytes = 38,400,000 bytes
```

The complete package forecast depends on measured relation density:

```text
primary WordCenter64 bank          38 MB
forward postings                 1.5-2.5 GB
reverse reconstruction           0.5-0.8 GB
anti memory                      0.3-0.9 GB
graph, atom codes and decoder    0.1-0.3 GB
------------------------------------------------
forecast total                   2.4-4.5 GB before measured compression
```

This forecast is a budget, not a measured result. The compiler receipt must
report actual counts, package bytes, peak RSS, temporary disk and elapsed time.
The v3 10k proof measured 3.639 bytes per forward relation and therefore lowers
the forward-section projection to roughly 0.7-1.3 GB if 600k relation density
remains comparable. This is still a projection. The runtime target is a
read-only mapped package; owned-vector loading is not called zero-copy.

## 7. Training Contract

```text
frequency-ranked RU word forms + frequency-ranked EN word forms
-> Unicode NFC, lowercase and exact deduplication
-> dense stable terminal IDs
-> streaming corruption generation
-> typed atom observations
-> complete forward postings
-> clean reverse reconstruction
-> real-lattice competitor anti modes
-> deterministic package
```

Frequency is learned support, not a rule that can override contradictory
surface evidence. Raw corpora, source strings and generated damaged episodes
are not stored in the runtime package. Only the decoder may materialize the
surface of a selected terminal.

Training and heldout corruption seeds are disjoint. Heldout labels may measure
the result but may not enter candidate birth, phase accumulation, anti lookup
or winner selection.

## 8. Promotion Proof

The 600k package remains shadow-only until all required metrics are reported:

```text
clean preservation                         >= 99.9%
unique top-1 for every damage class          > 95.0%
candidate-set coverage                      >= 99.9%
ambiguous false certainty                    = 0
RU -> EN and EN -> RU layout projection     >= 99.0%
Full better than NoPhase and NoAnti          required
WAND/exhaustive top-k parity                 exact
candidate-order permutation parity           exact
package size                                <= 4.5 GB
hot runtime p99                              <= 5 ms
raw corpus stored                            false
exact damaged episodes stored                0
```

These dimensions form one conjunctive contract. Per-class restoration is the
functional objective, while clean preservation, lattice coverage, false
certainty, package/RSS bounds and latency are simultaneous correctness and
feasibility constraints. None may be dismissed as secondary, and strength in
one dimension cannot compensate for failure in another.

The fixed 10k v3 baseline currently passes this working top-1 gate in 4 of 13
classes. The remaining nine classes are blockers regardless of aggregate
top-1:

```text
class                              top-1     gap to >95
double substitution               75.961%      19.039 pp
non-adjacent transposition        81.500%      13.500 pp
omission + transposition          84.508%      10.492 pp
repeated fragment                 86.609%       8.391 pp
letter substitution              90.538%       4.462 pp
sparse multi-omission             90.409%       4.591 pp
prefix truncation                 93.265%       1.735 pp
extra letter                      93.552%       1.448 pp
layout projection                 94.722%       0.278 pp
```

Ambiguous surfaces are not mislabeled as unique failures. They must retain all
valid centers in the calibrated lattice and produce no false certainty. L2/L3
may later resolve them from context, but L1 proof cannot use that context.

## 9. Rejected Shortcuts

```text
fixed per-atom relation truncation
fixed frontier presented as vocabulary-independent
raising all capacities in one uninformative experiment
using frequency as unconditional winner authority
claiming modern-Hopfield exponential capacity without separation proof
calling top-p mass a conformal guarantee without calibration
using proof target labels during retrieval or interference
installing a WATCH package into daemon or IME
```

### V62 proof-only alignment ceiling

V62 tested whether ordinary Damerau-Levenshtein alignment could recover a
missing ranking signal inside the existing V55 top-64 lattice. It was a
proof-only readout: it did not alter candidate birth, the package, runtime
authority, IME or the daemon, and it never read the heldout target.

On the fixed V55 2k package it changed aggregate heldout top-1 from 98.277% to
98.558% (`+45` cases), but the required per-class contract failed:

```text
class                         V55 unique top-1   V62 alignment   delta
double substitution                  95.045%          96.396%      +6
letter substitution                  98.221%          99.036%     +22
omission + transposition             95.294%          93.647%      -7
sparse multi-omission                98.141%          94.796%      -9
```

The result is `REJECTED`: scalar edit distance helps same-length corruption
but collapses distinct length-changing reconstruction basins. Aggregate gain
cannot pay for a regression in either omission class. The probe was removed
from runtime and proof code after measurement, and no 10k run was promoted.
The immutable evidence is
`docs/structural_gates/receipts/L1_ALIGNMENT_PROBE_V62_2K_2026-07-23.json`.

### V63 fixed-10k failure decomposition

V63 reran the immutable V55 10k package without changing selection and recorded
which observable waves already support the correct center in every unique
top-1 failure. Exact metric parity held: aggregate `97.169%`, double
substitution `90.152%`, omission plus transposition `94.407%`, and verdict
`WATCH_shadow`.

```text
class                    failures  missing top-64  rank-2  stronger position  stronger structural
double substitution           233               1     126                151                  155
non-adjacent transpose        103               0      74                 63                   73
omission + transpose          116               0      83                  0                   87
```

This proves that the dominant blocker is no longer candidate birth. It is a
cross-center relation problem: several component waves already favor the
target, but scalar settled energy still leaves the false incumbent first. The
existing terminal-pair memory cannot solve this regime: V55 contains 16,707
pair profiles and 17,227 pair centers but changes `0 / 0` outcomes because
word-specific reverse directions are too sparse.

The next accepted experiment is therefore a global learned relation wave. Its
training examples are real lattice comparisons between the target and false
competitors; its runtime input contains only candidate-relative wave evidence,
never a damage-class label, word-specific rule or heldout target. It may veto
or rerank an incumbent but cannot create a candidate or grant apply authority.
Evidence:
`docs/structural_gates/receipts/L1_FAILURE_DECOMPOSITION_V63_10K_2026-07-23.json`.

V64 trained one global positive/negative relation center from at most six
training surfaces per word. Its hard-negative threshold was frozen before the
heldout pass. It remained completely Neutral (`0 improved / 0 worsened`): one
center mixed incompatible length geometries and the negative envelope vetoed
every proposal.

V65 split that memory into nine structural basins keyed only by challenger and
incumbent length relation. It activated, but failed safety at 2k:

```text
relation improved / worsened             9 / 6
double substitution          95.045% -> 94.820%
non-adjacent transpose       97.436% -> 97.949%
```

Both variants are `REJECTED` and their proof authority code was removed. The
experiment establishes that a single centroid per length basin is still too
coarse. Any successor must use bounded coherent subcenters plus independent
hard-negative subcenters; a threshold adjustment cannot substitute for that
missing multimodal memory.

Evidence:

- `docs/structural_gates/receipts/L1_GLOBAL_RELATION_WAVE_V64_2K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_LENGTH_BASIN_RELATION_WAVE_V65_2K_2026-07-23.json`

### V66-V68 energy and ambiguity audit

The fixed V55 double-substitution score of `90.152%` treated one generated
target label as the only correct answer even when another dictionary center was
equally close to the damaged surface. V66-V68 added proof-only edit geometry;
it never changed candidate birth, settled energy, runtime authority or the
package.

On all 2,366 generator-unique double-substitution cases:

```text
geometry                                  cases   selected minimum   rate
target is the unique nearest center       1,834          1,812       98.800%
target shares minimum with competitors      518            466       89.961%
another center is nearer than target         13             12       92.308%
target missing from top-64                    1              0        0.000%
---------------------------------------------------------------------------
winner belongs to nearest-center set       2,366          2,290       96.788%
```

Therefore V55 already exceeds the 95% restoration target for double
substitution when correctness means selecting a geometrically admissible
nearest lexical center. It does not exceed the target-label score because 518
cases require context or abstention to choose among equally damaged words.
Both measurements remain mandatory:

```text
target-label top-1                         diagnostic of synthetic label match
nearest-set functional accuracy            L1 surface-restoration metric
tied-minimum set retention                 handoff contract for L2/L3
```

Edit distance is proof evidence, not new runtime authority. The wave kernel
must learn to expose the tied basin and preserve its members; it must not call a
decoder-side Damerau rule during selection.

The energy audit also found the actual remaining defect among 233 target-label
failures:

```text
both positive phases >= 990                199 / 233
both anti pressures zero                   231 / 233
both pairwise losses zero                  232 / 233
false winner has stronger forward wave     225 / 233
target is unique nearest but loses          22 / 233
```

Positive phase is saturated and does not separate close centers, while anti and
pairwise memories are almost always Neutral. The next L1 ranking work is
limited to the 76 cases where the selected winner is outside the nearest set,
plus explicit tied-basin/ABSTAIN calibration. It is not a 233-case
candidate-birth problem.

The V68 `11.770 ms` p99 is not a hot-runtime regression: latency was measured
after a parallel proof-only decoder audit and is cache/CPU contaminated. The
authoritative unchanged V55 p99 remains `5.886 ms`. Subsequent proofs measure
hot latency before cold geometry diagnostics.

Evidence:

- `docs/structural_gates/receipts/L1_ENERGY_AMBIGUITY_AUDIT_V66_10K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_DOUBLE_GEOMETRY_DENOMINATOR_V67_10K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_DOUBLE_FUNCTIONAL_ACCURACY_V68_10K_2026-07-23.json`

## 10. Architecture Conclusion: What The Experiments Study

Мы фактически изучаем **геометрию и динамику ассоциативного лексического поля
L1**, а через неё проверяем архитектуру Lay.

### Что уже выяснили

- Candidate birth почти решён: правильное слово обычно присутствует в lattice.
- Главная проблема находится в выборе устойчивого пика между близкими центрами.
- Positive phase насыщается около `990..1000` сразу у нескольких слов и
  перестаёт их различать.
- Forward-wave часто чрезмерно усиливает ложный центр.
- Anti-phase и pairwise memory почти не действуют.
- Один scalar `settled_energy` смешивает разные виды доказательств и иногда
  создаёт ложный пик.
- Часть ошибок на самом деле является объективной неоднозначностью:
  `ауала -> акула / атака`.
- Для однозначных double-substitution случаев V55 даёт `98.800%`.
- Общая функциональная точность этого класса равна `96.788%`, а не прежним
  synthetic `90.152%`.

### Архитектурный вывод

```text
повреждённая поверхность
-> sparse lattice центров
-> положительные и отрицательные волны
-> несколько устойчивых бассейнов
-> unique peak либо tied basin
-> L1 winner / ABSTAIN / lattice для L2-L3
```

L1 не должен насильно выбирать одно слово, когда поле содержит несколько равно
допустимых центров. Его ответственность:

1. Родить правильный бассейн.
2. Удалить доказанно ложные центры anti-wave.
3. Выбрать unique peak только при реальном разделении.
4. Передать неоднозначный lattice контекстным L2/L3.

Таким образом, эксперименты проверяют не отдельную формулу и не набор опечаток.
Они выполняют **идентификацию ассоциативного поля**:

- как образуются аттракторы;
- где возникают смешанные состояния;
- как плотность словаря влияет на разделимость;
- какие волны дают реальное доказательство;
- когда система обязана отказаться от выбора.

Следующий прогресс должен прийти не от общего усиления энергии, а от разделения
фазовых подцентров, работающей anti-memory и явного представления tied basin.

### L1.1

`L1.0` обозначает принятый V55 baseline: winner-owned sequence certificate,
bounded positional authority и scalar settled-energy readout.

`L1.1` обозначает следующую конфигурацию crystal kernel:

```text
surface wave
-> candidate lattice
-> bounded multimodal positive subcenters
-> independent anti / hard-negative subcenters
-> interference
-> unique peak | tied basin | ABSTAIN
```

У L1.1 две измеримые задачи:

1. Исправить 76 случаев double substitution, где выбранный центр не принадлежит
   множеству ближайших допустимых центров.
2. Для 518 неоднозначных случаев сохранять tied lattice или возвращать ABSTAIN,
   а не выдавать случайный singleton с ложной уверенностью.

Коэффициенты coherence, competition margin и границы `winner / tied / ABSTAIN`
обучаются и калибруются по evidence. Вручную фиксируются только типы сигналов,
структурные safety-инварианты и ресурсные бюджеты. Damage-class и word-specific
правила запрещены.

Гейт L1.1:

```text
unique-nearest accuracy              > 98.800%
nearest-set functional accuracy      > 96.788%
tied nearest-set retention           strictly improves toward 100%
false singleton on tied cases        0
candidate birth                      no regression
clean preservation                   >= 99.9%
hot runtime p99                      <= 5 ms
```

## 11. References

1. D. J. Willshaw, O. P. Buneman, H. C. Longuet-Higgins. "Non-holographic
   associative memory." Nature 222, 1969.
   https://doi.org/10.1038/222960a0
2. M. V. Tsodyks, M. V. Feigel'man. "The enhanced storage capacity in neural
   networks with low activity level." Europhysics Letters 6(2), 1988.
   https://doi.org/10.1209/0295-5075/6/2/002
3. D. Krotov, J. J. Hopfield. "Dense Associative Memory for Pattern
   Recognition." NeurIPS 2016. https://arxiv.org/abs/1606.01164
4. H. Ramsauer et al. "Hopfield Networks is All You Need." ICLR 2021.
   https://arxiv.org/abs/2008.02217
5. T. Bricken, C. Pehlevan. "Attention Approximates Sparse Distributed
   Memory." NeurIPS 2021. https://arxiv.org/abs/2111.05498
6. A. Z. Broder et al. "Efficient Query Evaluation Using a Two-Level Retrieval
   Process." CIKM 2003. https://doi.org/10.1145/956943.956944
7. S. Ding, T. Suel. "Faster Top-k Document Retrieval Using Block-Max
   Indexes." SIGIR 2011. https://doi.org/10.1145/2009916.2010048
8. S. Bruch, F. M. Nardini, C. Rulli, R. Venturini. "Efficient Inverted
   Indexes for Approximate Retrieval over Learned Sparse Representations."
   SIGIR 2024. https://doi.org/10.1145/3626772.3657769
9. Y. Romano, M. Sesia, E. J. Candes. "Classification with Valid and Adaptive
   Coverage." NeurIPS 2020. https://arxiv.org/abs/2006.02544
10. A. Angelopoulos, S. Bates, J. Malik, M. I. Jordan. "Uncertainty Sets for
    Image Classifiers using Conformal Prediction." ICLR 2021.
    https://arxiv.org/abs/2009.14193
11. E. Brill, R. C. Moore. "An Improved Error Model for Noisy Channel Spelling
    Correction." ACL 2000. https://aclanthology.org/P00-1037/
12. K. U. Schulz, S. Mihov. "Fast String Correction with Levenshtein
    Automata." IJDAR 5, 2002. https://doi.org/10.1007/s10032-002-0082-8
