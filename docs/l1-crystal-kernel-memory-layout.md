# L1 Crystal Kernel: Canonical Memory and Runtime

Status: implemented experimental shadow kernel; quality gate remains `WATCH_shadow`.

Snapshot date: 2026-07-23.

Scientific basis and the normative 600k scaling contract:
`docs/l1-crystal-kernel-scientific-foundations.md`.

## 1. Purpose

L1 crystallizes typed n-gram evidence into lexical centers. The corpus is a
cold training reference. The hot runtime stores the learned field and decoder,
not the corpus and not a table of damaged-to-correct strings.

```text
reference words + generated heldout damage
-> reversible typed NGramGraph
-> dense AtomId
-> learned AtomWaveCode
-> bidirectional AtomWordCoupling
-> WordCenter64 + candidate-specific anti-centers
-> forward SurfaceWave + backward ReconstructionWave
-> stable terminal ID or ABSTAIN
-> DecoderGraph materializes UTF-8
```

L1 produces lexical centers. It is not semantic L2. Candidate ranking inside
the L1 lattice must not be renamed L2 to hide an L1 reconstruction deficit.

## 2. Source Tree

```text
file              lines  purpose
atoms.rs            310  normalization, typed atoms and character anchors
ngram_graph.rs      132  reversible NGramKey -> dense AtomId
crystal.rs          249  fixed records and WordCenter64
wave_basis.rs       287  shared basis, compression and coherence
model.rs            114  package tables, couplings and anchor flags
compiler.rs        1481  cold crystallization and anti-centers
runtime.rs         1636  frontier, reconstruction and interference
format.rs          1103  deterministic versioned binary package
posting_codec.rs    303  compressed forward blocks and parity proof
corruption.rs       301  proof-only damage generation
proof.rs           1760  heldout metrics, ambiguity and ablations
restoration.rs      500  typed Winner/Tied/ABSTAIN authority
pairwise.rs         322  bounded lattice crystallization
tests.rs            593  focused invariants
mod.rs               31  shadow facade
                   ----
total              9122
```

## 3. Typed Atom Memory

The graph recognizes these bounded channels:

```text
byte 4-gram
character 2/3-gram
keyboard 2/3-gram
character/keyboard bag 3-gram
character/keyboard skip-gram
character reconstruction anchor
boundary and relative-position atoms
```

```text
symbols or key events
-> NGramGraph arcs
-> terminal AtomId
```

`AtomId` is a dense array index. A conventional stable hash may seed a compact
random projection or identify a cold artifact, but it is never the n-gram
identity, lexical answer or runtime authority. `NGramGraph` owns identity.

Each hot atom currently stores a 16-byte `AtomWaveCode` plus a bounded forward
coupling range. The wave code is compiled from the atom's learned word
couplings. It is not a stored source n-gram string. The fixed per-atom bound is
a known scale blocker: the normative 600k design stores complete compressed
postings and bounds runtime work with sound block upper bounds.

## 4. Exact WordCenter64 Layout

Every primary learned lexical center occupies exactly 64 bytes:

```text
wave_code             44 B   22 x (u8 basis, i8 coefficient)
coupling_start         4 B   reverse AtomWordCoupling range
anti_start             4 B   candidate-specific anti-center range
decoder_terminal       4 B   terminal in the UTF-8 decoder graph
coupling_count         2 B
crystal_support        2 B
anti_count             1 B
stability              1 B
surface_len            1 B
flags                  1 B
                      ----
                       64 B
```

The dense array index is the `WordCenterId`, so it is not repeated inside the
record.

```text
100,000 WordCenter64 x 64 B
= 6,400,000 bytes
= about 6.10 MiB
```

This 6.4 MB is only the learned primary word-center bank. The complete model is
larger because the n-gram graph, atom codes, bidirectional couplings,
anti-centers and decoder graph are shared package tables. The claim must never
be presented as the total size of a 100k model.

## 5. Shared Complex Basis

The current implemented field dimension is 128 complex cells. A center does not
store all 128 `re/im` pairs. Its 22 signed components reference a shared Fourier
basis. The 600k hypothesis raises the shared field to 256 cells without changing
the 64-byte center and requires a causal 128-versus-256 ablation.

```text
compact WordCenter64 wave_code
-> SharedWaveBasis
-> 128-cell complex vector in bounded CPU scratch
```

Cold accumulation uses wide integers. Published coefficients use signed
quantization. Internal phase and amplitude are therefore richer than ternary
logic.

`-1 / 0 / +1` is reserved for the final interpretation:

```text
+1  coherent constructive result
 0  unresolved / Neutral / ABSTAIN
-1  destructive contradiction
```

It is not the stored internal geometry and there is no 255 peak cap.

## 6. Two-Wave Runtime

```text
damaged surface
-> NGramGraph
-> observed AtomId + positions
-> AtomWave accumulation
-> forward candidate frontier (main max 128)
-> bounded geometry reserve (max 32, damaged surfaces only)

candidate WordCenter
-> reverse AtomWordCouplings
-> expected atoms and positions
-> backward ReconstructionWave

SurfaceWave + ReconstructionWave + candidate AntiWave
-> three bounded settling iterations
-> ordered lexical lattice
-> stable terminal or ABSTAIN
```

Candidate-specific anti-centers are learned only from competitors that really
occur in the L1 lattice. They activate only when their competing winner is
present. The anti vector enters the same interference calculation before the
final ordering.

The fixed main frontier, bounded geometry reserve and fixed top-4 anti bank
describe the current shadow implementation, not the scale-complete
architecture. The normative replacement is complete postings, exact
upper-bound pruning, conformal candidate coverage and adaptive clustered anti
modes.

## 7. Decoder Boundary

The package does not contain a `Vec<String>` vocabulary. A compact reverse
decoder graph stores shared UTF-8 prefixes:

```text
selected WordCenterId
-> decoder_terminal
-> parent-linked DecoderGraph
-> exact UTF-8 surface
```

The query command now returns both terminal IDs and decoded surfaces. Decoding
does not participate in candidate selection.

## 8. Cold and Hot Separation

```text
REMOTE COLD
corpus -> damage split -> graph/coupling learning -> crystallizer -> proof

LOCAL HOT
read-only package -> bounded frontier -> integer complex interference -> decode
```

The package stores neither the raw corpus nor exact damage episodes. Current
shadow loading still decodes package tables into owned vectors. A zero-copy
mapped view is a separate runtime optimization and must not be claimed until
its actual process RSS and page-sharing proof pass.

## 9. Current Proof Baseline

### 9.1 Normative L1.0 baseline: V55

V55 is the immutable measured baseline for L1.1 work. Later V56--V68 receipts
diagnose the same V55 field; they do not silently replace its selection
configuration. L1.1 must report every quality, coverage, memory and latency
delta against this baseline.

Implemented kernel configuration:

```text
configuration identity                  L1/V55
configuration status                    frozen L1.0 baseline

surface normalization                   trim outer whitespace
surface punctuation ignored             ! , . ? ; :
surface case                            lowercase

byte n-gram length                       4
byte n-gram budget                      24
byte n-gram channel weight               2

character n-gram lengths                 2, 3
character bigram budget                 16
character trigram budget                24
character bigram / trigram weights       1 / 3

keyboard n-gram lengths                  2, 3
keyboard bigram budget                  16
keyboard trigram budget                 24
keyboard bigram / trigram weights        1 / 3

character and keyboard bag length        3
bag budget per channel                  24
bag channel weight                       3

character and keyboard skip distance     2..4
skip budget per channel                 32
skip channel weight                      2

boundary prefix/suffix lengths           1..3
boundary channel weight                  3
character anchors                        one per character
character anchor weight                  1
relative position buckets               16

complex wave dimension                 128 re + 128 im cells
stored WordCenter wave components      22
WordCenter64 record                    64 bytes
stored AtomWaveCode components          4
AtomWaveCode record                    16 bytes
maximum phase frontier                128 centers
ordered output lattice                 64 centers
settling iterations                     3
maximum anchor sequence                32 atoms
forward posting policy                  complete, no truncation
measured maximum forward degree         10,000
maximum reverse lexical couplings      96
```

This block is the reproducibility contract for the measured V55 package. A
change to any listed value creates a new configuration identity; it must not be
reported as V55 even if the binary format and corpus are unchanged.

`256` is the historical `bounded_256` ablation limit, not the V55
configuration. A V55 training proof must use
`--l1-complete-forward-postings`; otherwise it measures a different lattice.

The `64 bytes` describe one primary learned center, not the whole model. Shared
atom, coupling, anti, pairwise, basis and decoder banks remain separate dense
package tables.

Measured V55 10k package:

```text
source lexical centers                 10,000
WordCenter64 bank                     640,000 bytes
typed atoms                           123,109
forward couplings                   8,052,768
reverse couplings                   1,026,342
anti centers                           13,734
pair profiles                          16,707
pair centers                           17,227
complete package                   44,888,171 bytes
training surfaces                     622,417
heldout surfaces                       88,463
proof workers                              20
proof time                            482.677 s
hot readout p50 / p99             5.082 / 5.886 ms
heldout top-1                          97.169%
heldout top-64                         99.999%
clean preservation                     99.990%
verdict                           WATCH_shadow
```

Measured V55 target-label top-1 by damage class:

```text
adjacent transposition                 99.582%
double substitution                    90.152%
extra letter                           99.645%
layout projection                      98.109%
letter substitution                    97.976%
missing letter                         99.512%
non-adjacent transposition              95.360%
omission + transposition                94.407%
prefix truncation                       98.032%
punctuation suffix                      99.940%
repeated fragment                       97.365%
sparse multi-omission                   96.100%
suffix truncation                       99.237%
```

V68 reinterpreted double substitution as signal restoration rather than forced
agreement with one synthetic target label:

```text
target uniquely nearest              1,812 / 1,834 = 98.800%
winner belongs to nearest set        2,290 / 2,366 = 96.788%
tied nearest-center cases              518
true remaining geometry failures         76
```

This is why V55 is retained as the L1.0 substrate. Candidate birth, clean
preservation and most restoration classes are already strong. L1.1 changes
crystallization and readout rather than discarding the proven atom graph and
candidate lattice:

```text
V55 single collapsed settled_energy
-> bounded coherent positive subcenters
-> independent anti and hard-negative subcenters
-> calibrated UniqueWinner | TiedLattice | ABSTAIN
```

Normative receipts:

- `docs/structural_gates/receipts/L1_WINNER_OWNED_SEQUENCE_V55_10K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_DOUBLE_GEOMETRY_DENOMINATOR_V67_10K_2026-07-23.json`
- `docs/structural_gates/receipts/L1_DOUBLE_FUNCTIONAL_ACCURACY_V68_10K_2026-07-23.json`

### 9.2 L1.1 multimodal crystallization: E2 PASS

L1.1 keeps the V55 atom graph, complete forward lattice and primary
`settled_energy` unchanged. It adds a separate bounded decision extension:

```text
positive subcenters per primary center       <= 4
anti subcenters per primary center           <= 4
hard-negative subcenters per primary center  <= 2
fit / calibration split                      80% / 20%
readout                                      Winner | Tied | ABSTAIN
maximum returned evidence candidates        32
```

Character geometry is calculated from reverse-only character anchors. Keyboard
geometry is not stored as a typed reverse coupling and cannot alter V55
authority. The clean keyboard atom sequence is stored in the L1.1 extension and
is consulted only when the input and candidate have proven disjoint script
flags. This prevents layout recovery from changing ordinary same-script word
geometry.

The deterministic E2 binary extension is:

```text
magic                                        LAYL11E2
extension header                             40 bytes
CenterPhaseProfile                           24 bytes
keyboard geometry atom                       u32 AtomId
V1--V4 read compatibility                    preserved
trailing bytes                               forbidden
```

Final complete-postings 10k proof:

```text
source centers                               10,000
training / heldout surfaces                  622,417 / 88,463
complete forward couplings                   8,052,768
V55 top-1 / top-64                           97.169438% / 99.998870%
clean preservation                           99.990%

L1.1 geometry unique cases                   79,619
L1.1 unique-basin winner                     99.881938%
L1.1 geometry tied cases                     8,844
L1.1 tied-basin safety                       100.000000%
L1.1 false singleton                        0
L1.1 nearest-set functional                  99.413315%
L1.1 target retained                         98.666109%
L1.1 winner / tied / abstain                 79,525 / 8,419 / 519
L1.1 hot p50 / p99                           3.106 / 3.357 ms
L1.1 verdict                                 PASS_shadow
```

Per-class unique-basin winner / tied-basin safety:

```text
adjacent transposition                       100.000% / 100.000%
double substitution                         100.000% / 100.000%
extra letter                                 100.000% / 100.000%
layout projection                             96.226% / 100.000%
letter substitution                          100.000% / 100.000%
missing letter                               100.000% / 100.000%
non-adjacent transposition                   100.000% / 100.000%
omission + transposition                     100.000% / 100.000%
prefix truncation                            100.000% / 100.000%
punctuation suffix                           100.000% / 100.000%
repeated fragment                            100.000% / 100.000%
sparse multi-omission                        100.000% / 100.000%
suffix truncation                            100.000% / 100.000%
```

The original double-substitution blocker is closed on the same denominator:
all `1,852` unique geometry cases produce a winner and all `522` tied geometry
cases remain `Tied` or `ABSTAIN`.

Artifacts:

- package: `data/lexical_grokking/l1_l11_crystal_e2_complete_10k.bin`
- package SHA-256:
  `fe42bd6c029e1d5df059e4cc9dc0f966f9c3c7a0e378a43696a1ff5dd6d17709`
- receipt:
  `docs/structural_gates/receipts/L1_1_MULTIMODAL_CRYSTAL_E2_10K_2026-07-23.json`

The accepted executable contract is shadow-only:

```bash
lay-l1.1-restore \
  --memory data/lexical_grokking/l1_l11_multimodal_restoration_10k.bin \
  врмея
```

It returns decoded JSON evidence for `winner`, `tied`, or `abstain`. This PASS
authorizes the standalone restoration probe. It does not authorize daemon,
IME, `AuthorizedEdit`, L2, L3 or L4 integration.

### 9.3 Preserved nearest lattice

The first E2 readout bounded a `Tied` result to eight candidates and returned no
candidate evidence for `ABSTAIN`. This discarded useful evidence in large
objective basins. The corrected readout has four explicit outcomes:

```text
Winner        one geometrically unique center with calibrated authority
Tied          complete nearest basin, up to 32 returned candidates
TiedOverflow  nearest basin exceeds the returned evidence bound
ABSTAIN       no authority, with up to 32 evidence candidates when available
```

Anti-wave remains an authority veto for a geometrically unique winner. It must
not prune an objective tied basin: anti evidence says that a center must not
receive authority in the observed scene; it does not prove that the center is
absent from the nearest restoration set. The rejected anti-pruning replay
demonstrated the distinction:

```text
damage class                 anti-pruned evidence   preserved evidence
omission + transposition                  87.799%               91.911%
sparse multi-omission                     86.299%               90.273%
layout projection                         99.803%               99.803%
```

Accepted package-reuse proof on the immutable E2/V55 package:

```text
heldout surfaces                              88,463
V55 top-1 / top-64                 97.169438% / 99.998870%
clean preservation                            99.990%

L1.1 authority target winner                  89.262%
L1.1 decision target retained                 99.090%
L1.1 evidence target retained                 99.251%
target present in full nearest basin           99.251%
nearest-set functional                         99.838%
tied-basin safety                             100.000%
false singleton                                     0
winner / tied / tied-overflow / abstain  79,525 / 8,795 / 1 / 142
hot p50 / p99                           3.105 / 3.351 ms
L1.1 verdict                              PASS_shadow
overall V55 target-label verdict         WATCH_shadow
```

Per-class authority / decision retention / evidence retention:

```text
adjacent transposition        97.010% / 99.987% / 99.987%
double substitution           77.254% / 99.073% / 99.073%
extra letter                  98.813% / 99.993% / 99.993%
layout projection             94.250% / 94.250% / 99.803%
letter substitution           94.351% / 99.875% / 99.875%
missing letter                80.812% / 99.341% / 99.341%
non-adjacent transposition    69.736% / 97.720% / 97.720%
omission + transposition      47.763% / 91.866% / 91.911%
prefix truncation             64.446% / 96.850% / 96.850%
punctuation suffix           100.000% / 100.000% / 100.000%
repeated fragment             90.972% / 99.482% / 99.482%
sparse multi-omission         42.586% / 90.273% / 90.273%
suffix truncation             74.883% / 98.428% / 98.428%
```

Receipts:

- accepted:
  `docs/structural_gates/receipts/L1_1_PRESERVED_NEAREST_LATTICE_10K_2026-07-23.json`
- rejected:
  `docs/structural_gates/receipts/L1_1_ANTI_PRUNED_LATTICE_REJECTED_10K_2026-07-23.json`

This closed the evidence-loss P0 without claiming that the two weakest damage
classes were solved. Their full nearest basins retained only `91.911%` and
`90.273%` of targets. The following two package-reuse proofs address that
measured loss without using hidden target labels at runtime.

#### Physical-key geometry

The surface encoder now preserves the observed physical `KeyEvent` sequence
when the package advertises the physical-key profile flag. This keeps
punctuation keys from the wrong layout and compares layout candidates in key
space rather than reconstructing key identity from already decoded text. Old
E2 packages remain readable and V55 ranking is byte-for-byte unchanged.

```text
damage class                         before authority   after authority
layout projection                            94.250%           99.882%

layout evidence                              99.803%          100.000%
layout ABSTAIN                                   142                  3
false singleton                                    0                  0
hot p99                                      3.435 ms
L1.1 verdict                              PASS_shadow
```

Receipt:
`docs/structural_gates/receipts/L1_1_PHYSICAL_KEY_GEOMETRY_10K_2026-07-23.json`.

#### Deletion-aware multimodal reconstruction

When no exact lexical center exists, the runtime forms a bounded union of the
scalar nearest basin and reconstruction hypotheses. Reconstruction admits two
ordered omissions and one omission plus one necessary adjacent transposition.
An exact lexical center closes speculative reconstruction. Expansion is
bounded to the eight strongest scalar candidates and cannot change V55
ranking.

Final 10k package-reuse proof:

```text
source centers                                  10,000
heldout surfaces                                88,463
V55 top-1 / top-64                    97.161525% / 99.998870%
clean preservation                              99.990%

L1.1 authority target winner                    86.763%
L1.1 evidence target retained                   99.633%
scalar nearest-basin retention                  99.256%
reconstruction basin expansions                  4,226
targets recovered / lost                         333 / 0
tied-basin safety                              100.000%
false singleton                                      0
winner / tied / tied-overflow / abstain  77,055 / 11,404 / 1 / 3
hot p50 / p99                           3.127 / 3.375 ms
L1.1 verdict                                PASS_shadow
overall V55 target-label verdict           WATCH_shadow
```

Per-class authority / evidence retention / scalar retention / recovered:

```text
adjacent transposition        91.864% / 99.987% / 99.987% /   0
double substitution           77.254% / 99.073% / 99.073% /   0
extra letter                  98.186% / 99.993% / 99.993% /   0
layout projection             99.882% / 100.000% / 100.000% / 0
letter substitution           93.646% / 99.875% / 99.875% /   0
missing letter                74.238% / 99.341% / 99.341% /   0
non-adjacent transposition    69.736% / 97.720% / 97.720% /   0
omission + transposition      47.763% / 99.593% / 91.911% / 170
prefix truncation             57.651% / 96.850% / 96.850% /   0
punctuation suffix           100.000% / 100.000% / 100.000% / 0
repeated fragment             90.972% / 99.482% / 99.482% /   0
sparse multi-omission         42.586% / 99.941% / 90.273% / 163
suffix truncation             65.363% / 98.428% / 98.428% /   0
```

For every class, tied-basin safety is `100%`, false singleton is `0`, and
reconstruction target loss is `0`. Authority is deliberately lower for
severely incomplete signals because additional plausible centers remain
`Tied`; proof targets are not used to collapse those basins.

Artifacts:

- package:
  `data/lexical_grokking/l1_l11_multimodal_restoration_10k.bin`
- package SHA-256:
  `0823cfe739d4387c92fa11ce2ccadb6b3e1611c76fca34aa474a3aae8c02f385`
- receipt:
  `docs/structural_gates/receipts/L1_1_MULTIMODAL_RECONSTRUCTION_10K_2026-07-23.json`

### 9.4 L1.1 crystallization closure: bounded ambiguity shell

The final 10k safety closure does not enlarge the complete settling field and
does not add lexical rules. It preserves the V55 main frontier and admits a
small evidence reserve from the same real forward postings:

```text
typed atom postings
-> complete touched-center set
-> main mass frontier, max 128
-> exact clean center has priority
-> damaged-only geometry reserve, max 32
   -> calibrated nearest basin
   -> one adjacent ambiguity shell (min + 1)
-> positive / anti / hard-negative / pairwise interference
-> ambiguity-shell candidates remain in the restoration basin
-> complete crystallization certificate may authorize one winner
-> otherwise Tied or ABSTAIN
```

The reserve stores no target label and scans no external vocabulary. Runtime
uses the package's predecoded character-anchor sequences only for centers born
from the observed atom postings. The ambiguity shell can preserve evidence and
veto unsupported singleton authority; it cannot choose a word by itself.

Two alternatives were rejected:

```text
precomputed nearest-neighbor map
  startup about 22 s, peak RSS about 500 MiB

global phase frontier 128 -> 256
  pairwise work grows quadratically
  full proof exceeded 12 minutes and was stopped
```

The accepted reserve loads the same package in about `0.2--0.3 s` with about
`124 MiB` process RSS. Isolated hot probes measured `2.820 ms` exact-surface
p99 and `3.415 ms` on the difficult `буноть` ambiguity. The full fixed proof
measured:

```text
source words                                  10,000
training / heldout surfaces         789,936 / 107,544
package bytes                              49,663,268
proof workers                                      20
proof time                                    380.885 s

clean preservation                   10,000 / 10,000 = 100.000%
target-label top-1                              94.521%
target-label top-64                             99.943%

L1.1 authority target winner                    65.699%
L1.1 evidence target retained                   99.752%
objective ambiguity safety         2,783 / 2,783 = 100.000%
false authority                                       0
false singleton                                       0
reconstruction recovered / lost                 217 / 0
winner / tied / abstain              70,886 / 36,105 / 553
hot p50 / p99                         3.782 / 4.754 ms

L1.1 verdict                                PASS_shadow
overall target-label verdict               WATCH_shadow
```

The authority delta from the preceding unsafe reserve was
`71.404% -> 65.699%`. This is intentional: the removed authority was converted
to evidence-preserving `Tied`, not redirected to another candidate. Promotion
still requires solving target-label top-1 and proving the same safety at scale.

Per-class proof includes the actual case denominator. Columns are
`cases / unique top-1 / top-64 / authority / evidence retained / ambiguity
safety`:

```text
adjacent transposition       19,464   98.140%   99.995%   63.882%   99.928%   100%
double substitution           2,472   81.358%   99.757%   75.647%   99.757%   100%
extra letter                 19,513   96.333%   99.995%   71.460%   99.995%   100%
layout projection             2,433   87.752%   99.301%   90.464%   99.301%   100%
letter substitution          19,900   97.034%   99.935%   62.940%   99.653%   100%
missing letter               19,502   97.525%   99.933%   46.282%   99.487%   100%
non-adjacent transposition    2,377   86.890%  100.000%   73.917%  100.000%   100%
omission + transposition      2,384   86.682%   99.832%   58.977%   99.790%   100%
prefix truncation             2,480   98.147%  100.000%   52.702%   99.435%   100%
punctuation suffix           10,017  100.000%  100.000%  100.000%  100.000%   100%
repeated fragment             2,455   89.201%  100.000%   87.373%  100.000%   100%
sparse multi-omission         2,169   86.691%   99.816%   50.622%   99.769%   100%
suffix truncation             2,378   92.243%   99.916%   38.898%   98.486%   100%
```

Accepted artifacts:

- package:
  `data/lexical_grokking/l1_l11_crystallization_10k.bin`
- package SHA-256:
  `af5150ebc57fa01d7faf7b98723e3a0483160bcc51e99371dd11da56045d1f58`
- receipt:
  `docs/structural_gates/receipts/L1_1_CRYSTALLIZATION_AMBIGUITY_SHELL_10K_2026-07-23.json`
- receipt SHA-256:
  `032bd00d8e988c31ab7f8cf60cd1f50a364bccf40b1bef9bdcff7232673e164d`

This authorizes only the standalone shadow command:

```bash
lay-l1.1-restore \
  --memory data/lexical_grokking/l1_l11_crystallization_10k.bin \
  буноть
```

It does not authorize IME, daemon, auto-apply, L2, L3 or L4 integration.

### 9.5 Historical baselines

Accepted 2,000-word v15 baseline after fixing relative-position quantization:

```text
clean preservation            100.000%
heldout top-1                  94.543%
heldout top-8                  99.800%
heldout top-64                 99.981%
anti improved / worsened          31 / 6
phase ablation drop              10539
artifact bytes              12,277,464
WordCenter bank bytes           128,000
```

Class top-1:

```text
adjacent transposition          98.236%
double substitution             82.207%
extra letter                    97.658%
layout projection               98.218%
letter substitution             95.345%
missing letter                  92.580%
non-adjacent transposition      88.235%
omission + transposition        82.339%
prefix truncation               89.376%
punctuation suffix              99.548%
repeated fragment               91.509%
sparse multi-omission           70.979%
suffix truncation               91.775%
```

Candidate birth is effectively solved at this scale because top-64 is nearly
100%. Winner crystallization is not solved. The package remains shadow-only.

Accepted 2,000-word v22 after channel-separated structural interference and
clean-reference backward reconstruction:

```text
clean preservation            100.000%   delta  +0.000 pp
heldout top-1                  96.647%    delta  +2.104 pp
heldout top-8                  99.919%    delta  +0.119 pp
heldout top-64                 99.981%    delta  +0.000 pp
sparse multi-omission          89.668%    delta +18.689 pp
layout projection              98.614%    delta  +0.396 pp
artifact bytes             12,241,112    delta -36,352 B
compile time                     6.63 s   delta -38.91 s
hot p50 / p99               1.065/1.276 ms
```

At 10,000 words, v23 compared with the same pre-change v18 baseline:

```text
clean preservation             99.840%   delta -0.140 pp
heldout top-1                  90.705%    delta +1.449 pp
heldout top-8                  98.996%    delta +0.336 pp
heldout top-64                 99.631%    delta -0.001 pp
artifact bytes             41,550,288    delta -117,192 B
compile time                    59.06 s   delta -415.22 s
hot p50 / p99               1.435/1.744 ms
```

The 2k model clears the user-accepted 95% working threshold. The 10k model is
still `WATCH_shadow`: top-1 is below 95% and clean preservation misses the
formal 99.9% gate by 0.06 percentage points.

V24 adds a character-anchor reconstruction wave. Anchor atoms do not create
candidates, enter the positive phase center or train anti-centers. They only
test whether the observed character order is a complete subsequence of a
longer candidate center. The lane is active only when raw input is shorter than
the center, so equal-length transposition and layout routes retain their old
geometry.

```text
2k v22 -> v24
clean preservation            100.000% -> 100.000%
heldout top-1                  96.647% -> 97.022%
sparse multi-omission          89.668% -> 95.911%  PASS_working
layout projection              98.614% -> 98.614%
missing letter                 97.378% -> 98.843%
artifact bytes             12,241,112 -> 12,360,452
same-binary hot p99             10.009 -> 10.080 ms
```

The same mechanism is not yet scale-complete:

```text
10k v24
clean preservation             98.980%
heldout top-1                  91.823%
heldout top-8                  99.145%
heldout top-64                 99.633%
sparse multi-omission          90.409%
artifact bytes             42,201,208
compile / proof             379.5 / 578.7 s
```

Therefore the accepted 95% criterion is closed at the 2k proving scale, while
10k remains a promotion blocker. Receipts:

```text
docs/structural_gates/receipts/L1_CRYSTAL_OMISSION_RECONSTRUCTION_V24_2K_2026-07-23.json
docs/structural_gates/receipts/L1_CRYSTAL_OMISSION_RECONSTRUCTION_V24_10K_2026-07-23.json
```

The proof now processes Full, NoPhase and NoAnti readouts on all available CPU
workers. On the remote 20-thread host, proof evaluation fell from about 78.6 s
to about 9.9 s. Cold graph compilation is still partly sequential.

V25 format work introduces a block-compressed v3 forward section while keeping
v2 read compatibility. On the real 10k v23 package, all 3,229,043 relations
roundtrip exactly, 500 sampled top-64 readouts are identical and the complete
package falls from 41,550,288 to 27,469,785 bytes. This is a format PASS only;
the compiler still truncates forward lists and the runtime does not yet execute
Block-Max WAND directly over mapped blocks.

The full fixed-corpus v3 proof preserves every v24 quality metric exactly while
reducing the current v24 package from 42,201,208 to 28,120,705 bytes. The class
matrix remains `WATCH_shadow`; compression neither improves nor degrades it.
Receipt:
`docs/structural_gates/receipts/L1_CRYSTAL_FORMAT_V3_10K_FIXED_2026-07-23.json`.

```text
10k fixed corpus v3                 unique top-1
adjacent transposition                  95.756%
double substitution                     75.961%
extra letter                            93.552%
layout projection                       94.722%
letter substitution                     90.538%
missing letter                          96.296%
non-adjacent transposition              81.500%
omission + transposition                84.508%
prefix truncation                       93.265%
punctuation suffix                      97.863%
repeated fragment                       86.609%
sparse multi-omission                   90.409%
suffix truncation                       96.245%

clean preservation                      98.980%
aggregate heldout top-1                 91.823%
heldout top-8                           99.145%
heldout top-64                          99.633%
```

V26 removes the fixed forward cap as an isolated ablation. The old compiler
dropped 2,092,161 of 5,321,204 learned relations across 4,574 saturated atoms.
Returning the complete field improves every damage class, raises aggregate
top-1 from 91.823% to 96.293%, restores clean preservation to 100%, and moves
the strict working gate from 4/13 to 9/13 passing classes. The compressed v3
package remains 35,070,906 bytes and hot p99 is 4.182 ms.

The remaining class blockers are:

```text
double substitution              86.777%
non-adjacent transposition       91.738%
omission + transposition         92.566%
repeated fragment                93.737%
```

The current anti field is not accepted on the complete lattice because it has
5 improvements and 8 worsenings. Complete postings are a causal architecture
PASS; total L1 promotion remains `WATCH_shadow`. Receipt:
`docs/structural_gates/receipts/L1_COMPLETE_FORWARD_POSTINGS_V26_10K_2026-07-23.json`.

## 10. Ambiguity Contract

Some damaged surfaces genuinely match more than one valid word. For example,
one surface can be a transposition of one center and a substitution of another.
The proof must build ambiguity from clean, training and heldout surfaces, not
only heldout labels.

```text
unique case       target top-1 is measurable
ambiguous case    target lattice coverage is measurable
false certainty   must be zero
```

L1 must return Neutral/ABSTAIN when its own field cannot distinguish the
centers. L2/L3 context may later collapse that lattice; hidden proof labels may
not.

## 11. Rejected Variants

Experiments are removed when their causal proof is worse:

```text
three unlabeled positive subcenters:
  top-1 94.543% -> 94.400%, package +384 KB; rejected

three-round spectral graph diffusion:
  top-1 94.543% -> 78.385%
  clean 100.0% -> 98.5%
  anti improved/worsened 199/2764; rejected
```

This prevents failed research branches from becoming permanent runtime debt.

## 12. Promotion Gate

The accepted working gate is strict and conjunctive:

```text
unique top-1 > 95.0% in every damage class
```

An aggregate percentage cannot hide a failing class. The current fixed 10k v3
baseline passes this working gate in 4 of 13 classes; all nine remaining
classes are blockers.

The stricter formal target remains:

```text
clean preservation             >= 99.9%
unique top-1 by damage class   >= 99.0%
ambiguous lattice coverage     >= 99.0%
false certainty                = 0
top-8/top-64                    reported
phase/anti causal ablations     positive
package size and peak RSS       bounded
hot p50/p99                     reported
raw corpus stored               false
exact damage episodes stored    0
```

No daemon, IME or `AuthorizedEdit` route may load this model until the shadow
gate passes. L1 proof cannot use L2, L3, L4, Bayes or a hidden target label.
