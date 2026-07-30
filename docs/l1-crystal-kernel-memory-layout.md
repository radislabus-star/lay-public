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

### 9.6 Final bilingual corpus OOM VETO and bounded compiler

The first final-corpus attempt used the complete `762,314`-form RU+EN source
with 48 training surfaces per word. Linux killed `lay-nanda-wave-train` at
`31,654,220 KiB` anonymous RSS (`32,413,921,280` bytes) and
`44,776,192 KiB` virtual memory. No package or quality proof was produced. A
second attempt again exhausted interactive headroom and required a manual host
reboot before a proof receipt existed. Both attempts are `VETO`; neither
changes runtime authority.

The failure was a lifetime error, not evidence that the final corpus should be
reduced. Packing generated strings into one UTF-8 arena removed millions of
heap objects, but the compiler subsequently retained overlapping full-corpus
states:

```text
packed training arena
+ per-word atom statistics
+ Vec<Vec<forward relation>>
+ Vec<Vec<reverse relation>>
+ unbounded directional anti relations
+ per-node BTreeMap graph builders
+ encoded package copies
```

The replacement cold path preserves complete forward postings while bounding
the intermediate resident set:

```text
packed training corpus
-> one deduplicated NGramKey set
-> compact recursive NGramGraph nodes/arcs
-> atom-support pass
-> per-word statistics with word-local lifetime
-> 16 range-partitioned 12-byte posting spool records on NVMe
-> one posting shard sorted and materialized at a time
-> flat CSR AtomRecord -> forward_couplings
-> flat reverse_couplings + per-word ranges
-> bounded top-16 directional anti bank
-> global decoder transition table
-> L1.1 subcenters and calibration
-> atomic package encoding
```

`training_budget.rs` independently polls real `VmRSS` every 250 ms. Final
training defaults to a `24 GiB` ceiling and the next controlled run uses
`20 GiB`. If the process crosses the configured ceiling, it writes a
`VETO_RSS_BUDGET` marker and exits with code `86` before `sshd` loses memory
headroom. Compiler checkpoints report stage, current RSS, peak RSS and budget.

Measured verification of the replacement so far:

```text
lexical_grokking unit tests                    57 / 57 PASS
cargo check --lib                             PASS
cargo check --bin lay-nanda-wave-train        PASS
clippy --lib -D warnings                      PASS
clippy --bin lay-nanda-wave-train -D warnings PASS
complete-posting byte determinism              PASS on focused fixtures
full 762,314-form RSS                          not measured yet
full per-class quality                         not measured yet
runtime authority                              unchanged, shadow-only
```

Receipt:
`docs/structural_gates/receipts/L1_L11_FINAL_CORPUS_OOM_VETO_2026-07-24.json`.

### 9.7 Packed anti readout on the full candidate field

Cold directional anti discovery no longer walks `WaveCoupling` structs
directly. It builds a compact block index over the complete forward field:

```text
AtomId
-> sorted terminal blocks of 64
-> 64-bit occupancy mask
-> packed u16(strength, position)
-> cached block upper bound
-> exact current-terminal payload read
```

The fixed microbenchmark denominator uses a `1,000`-center prefix of the
bilingual corpus and its `45,016` damaged training surfaces. This result does
not represent the complete `762,314`-center candidate field. The remote host
exposes 20 hardware threads; the process is restricted to CPUs `0-18`, and
`available_parallelism()` therefore creates 19 workers.

```text
implementation                         anti discovery
plain WAND                                  1.401 s
lazy Block-Max                              0.883 s
packed index before hot-loop cache          1.196 s
packed index + block cache runs     0.863 / 0.869 / 0.874 / 0.869 s
packed median                               0.869 s
packed median vs plain WAND                 1.612x
packed median vs lazy Block-Max             1.016x
packed resident bytes                    3,969,364
dense-oracle parity                           5 / 5 PASS
cursor invariants                             1 / 1 PASS
```

The result is a narrow performance `PASS_shadow`: all four packed runs are
below the `0.883 s` comparator, but the median headroom is only `14 ms`.
Caching is essential. It deliberately evaluates more exact candidates than the
uncached packed variant while avoiding repeated scans of the same block payload.

This experiment did not build the complete `762,314`-center candidate field,
did not run complete anti-target crystallization, did not establish the final
per-class restoration matrix and did not publish or install a package. Runtime
authority remains unchanged and shadow-only. Receipt:
`docs/structural_gates/receipts/L1_L11_PACKED_ANTI_READOUT_1K_2026-07-24.json`.

### 9.8 Full-field anti-search baseline and rejected O(1) block bound

The final `762,314`-center bilingual candidate field was compiled successfully
on the remote 20-thread host. This closes the previous RSS blocker but exposes
the next cold-training CPU blocker:

```text
training surfaces prepared                         65.082 s
NGramGraph compiled                                61.135 s
atom support compiled                              51.164 s
posting spool compiled                             77.019 s
complete postings materialized                     61.574 s
full forward relations                         566,183,479
packed anti index resident bytes             3,383,426,106
peak RSS                                      11,835,613,184 B

full-field anti targets                              1,000
damaged surfaces                                    44,994
anti discovery                                     198.105 s
linear full-target estimate                         ~42 h
```

This is the first anti-search timing whose candidate field is the complete
RU462K+EN300K field. It is not comparable to the earlier one-thousand-center
microbenchmark as a scale result.

An experiment added a three-byte cached block summary
`(max_strength, min_position, max_position)` and replaced payload scanning with
an O(1), mathematically conservative block upper bound. Dense-oracle parity and
all-position bound checks passed, but the full-field denominator rejected it:

```text
anti discovery                         198.105 s -> 190.543 s
speedup                                             1.040x
exact candidates                 8,263,792 -> 12,228,425
block checks                    62,615,939 -> 66,474,601
anti index bytes             3,383,426,106 -> 3,804,108,474
```

The looser bound admitted 48% more exact candidates and spent an additional
420,682,368 bytes for only 3.8% wall-clock improvement. The implementation was
removed. Verdict: `REJECTED`; runtime authority and installed packages remain
unchanged. The next experiment must change exact candidate retrieval rather
than add more metadata to Block-Max WAND. Receipt:
`docs/structural_gates/receipts/L1_L11_FULL_FIELD_ANTI_1K_2026-07-25.json`.

An exact threshold-seed retrieval was also tested. It selected rare postings
until the remaining unselected upper bound was below the target score, then
scored only the mathematically complete candidate union. Synthetic
dense-oracle parity passed, but the full-field run had not completed 1,000
targets after more than 230 seconds, already slower than the 198.105-second
WAND baseline. The run was stopped and the implementation removed. The result
shows that exact retrieval alone does not repair the dominant multiplier:
`762,314 centers x up to 47 damaged surfaces`.

The next cold-training architecture must mine anti-centers from observed
false/tied lattice outcomes in a fixed heldout loop. Centers with no observed
counterexample keep an empty anti bank. This preserves the rule that every
learned anti relation comes from a real L1 lattice while scaling work with
actual conflicts rather than all generated corruptions.

### 9.9 Bounded hard-surface anti pass

The positive and reconstruction fields still learn all 48 selected surfaces
per center. Only expensive full-lattice competitor discovery is bounded to one
deterministically rotated damaged surface per center. Rotation is derived from
the terminal identity, not corpus position or proof labels; over the complete
center population it covers every generated damage-mode slot.

On the complete RU462K+EN300K candidate field:

```text
anti targets                                      1,000
full-surface baseline queries                    45,016
bounded hard-surface queries                      1,000
full-surface anti time                          198.105 s
bounded anti time                                 3.659 s
anti-stage speedup                               54.14x
dense fallbacks                                        0
exact candidates                                183,477
linear 762,314-target estimate                   46.49 min
```

This is a cold-training scale `PASS_shadow`, not a restoration-quality PASS.
The complete pass, L1.1 subcenter compilation and fixed heldout per-class proof
are still required. Runtime authority remains unchanged. Receipt:
`docs/structural_gates/receipts/L1_L11_BOUNDED_HARD_SURFACE_1K_2026-07-25.json`.

### 9.10 Causal fit-surface depth ablation

The earlier `2/4/8/16/32/48` run did not answer whether damaged fit surfaces
were necessary at all. A controlled 10,000-center bilingual experiment
therefore compared depths `0/1/2/4`. Every variant used the same sampled words,
deterministic heldout reservoir, calibration procedure and one anti probe per target.
Because `word_surfaces()` yields the clean surface first, the anti probe was the
same clean surface at every depth. The only changed variable was the number of
additional damaged fit surfaces stored per `WordCenter`.

```text
metric                         depth 0   depth 1   depth 2   depth 4
aggregate unique top-1         99.332%   99.324%   99.212%   99.390%
worst class                    98.448%   98.048%   97.197%   98.999%
clean preservation            100.000%  100.000%  100.000%  100.000%
authority target winner         7.721%    7.721%    7.721%    7.721%
evidence target retained       99.954%   99.950%   99.942%   99.954%
false authority                     0          0          0          0
false singleton                     0          0          0          0
compile seconds                 31.742     42.504     53.446     76.827
proof seconds                  171.467    174.661    194.460    208.045
package MiB                     18.030     21.035     24.956     29.821
peak RSS MiB                    93.586    109.605    127.602    153.246
```

Depth `2` is not a necessary architectural minimum. Depth `0` beats it in
aggregate quality and every measured resource dimension. The damaged examples
are not uniformly harmful, however: depth `2` improves sparse multi-omission by
`0.373` percentage points and omission plus transposition by `0.405` points
relative to depth `0`. Depth `4` is the tested quality candidate for a
final-scale crystallization because it has both the best aggregate and the best
worst-class result. Depth `0` remains the resource-minimal Pareto point. The
weakest depth-4 class is double substitution at `98.998999%`.

This is `PASS_shadow` for selecting a depth, not package promotion. It proves
that the architecture does not require depth `2`, and that additional damaged
surfaces are a class-conditioned regularizer rather than a monotonic capacity
control. The result does not prove the complete `762,314`-center package,
installed latency or live authority. Runtime authority remains unchanged.
Receipt:
`docs/structural_gates/receipts/L1_L11_CAUSAL_DEPTH_10K_2026-07-25.json`.

### 9.11 Class-conditioned allocation

The depth ablation exposed an accidental selector policy. For budgets at or
below the number of available classes, the legacy selector consumed a
`BTreeMap` in alphabetical class order. Consequently, depth `2` did not mean
"two representative damaged surfaces"; it usually meant one adjacent
transposition and one double substitution.

The first replacement allocated surfaces only to double substitution, omission
plus transposition and sparse multi-omission. It improved the intended classes,
but removed too much cross-class geometry:

```text
metric                         clean 0    hard 4
aggregate unique top-1          99.332%    99.278%
layout projection               99.549%    97.897%
omission plus transposition      98.887%    99.494%
sparse multi-omission            99.255%    99.894%
authority                         7.721%     7.721%
```

The hard-only policy was rejected and removed. The retained experimental
policy uses this deterministic schedule:

```text
layout projection
double substitution
omission plus transposition
sparse multi-omission
adjacent transposition
extra letter
```

If a generated surface belongs to heldout, the selector skips it rather than
leaking proof evidence into training. Depths `4/5/6` were compared against the
same clean and legacy controls. Hybrid depth `5` is the tested candidate:

```text
metric                         clean 0   legacy 4   hybrid 5
aggregate unique top-1          99.332%     99.390%     99.305%
worst damage class              98.448%     98.999%     97.998%
clean preservation             100.000%    100.000%    100.000%
authority target winner          7.721%       7.721%     87.975%
objective unique winner            7.783%       7.783%     88.687%
evidence target retained        99.954%      99.954%     99.954%
false authority                      0            0           0
false singleton                      0            0           0
package MiB                     18.030       29.821      31.653
```

The standalone 20-thread proof removed concurrent-ablation latency noise:

```text
proof wall time                  88.83 s
peak RSS                        149.14 MiB
hot readout p50                   3.421 ms
hot readout p99                   3.790 ms
overall verdict             PASS_shadow
L1.1 verdict                PASS_shadow
crystallization verdict     WATCH_shadow
```

Hybrid depth `5` crosses the accepted per-class `>95%` gate, keeps clean input
at `100%`, raises certified authority by `80.254` percentage points and creates
no false authority. It is not promoted: the complete `762,314`-center proof has
not run, and crystallization certification still abstains on part of the
objective-unique denominator. The default remains legacy until that proof.
Runtime authority and installed packages are unchanged. Receipt:
`docs/structural_gates/receipts/L1_L11_CLASS_CONDITIONED_10K_2026-07-25.json`.

### 9.12 Normative single-pass streaming crystallizer contract

The current compiler is one command, but it is not a single-pass streaming
crystallizer. This distinction is architectural, not cosmetic.

The current implementation first materializes the word dictionary and all
selected training surfaces in `TrainingCorpus`. It then performs separate
whole-corpus or whole-training-surface stages:

```text
read_to_string(corpus)
-> prepare and retain TrainingCorpus
-> compile_graph()
-> compile_atom_support()
-> compile_posting_spool()
-> discover_anti_centers()
-> compile_l11_subcenters()
-> calibrate_l11()
-> build a temporary runtime memory
-> calibrate_l11_ambiguity_thresholds()
     -> readout(surface, 64, Full) for training surfaces
-> rebuild runtime memory
-> calibrate_l11_tied_energy_margin()
     -> readout(surface, 64, Full) for a calibration subset
-> encode package
-> clean audit
-> latency audit
-> fixed heldout proof
```

This route is wrong as the final cold architecture because:

```text
raw corpus is loaded as one String
selected training surface text remains resident
the same surfaces are encoded repeatedly
calibration executes the hot runtime inside the compiler
threshold learning requires temporary complete packages
CPU cost grows as repeated readout rather than one evidence reduction
stage-local Vec/HashMap state overlaps and raises peak RSS
```

The ongoing final-scale run may still provide valid quality measurements. It
does not prove that the compiler has the required streaming architecture, and
its result must not be described as a one-pass compile.

#### Required whole streaming route

The accepted definition of "single pass" is exactly one read of the raw corpus.
Single-pass does not mean single-threaded. The corpus and evidence streams are
partitioned into deterministic shards and must use all available workers; the
bounded merge restores canonical order and byte-identical output. "One pass"
constrains how often each evidence record enters learning, not how many CPU
cores process independent records.

Global relations still require a deterministic reduce after all atom owners
are known. That reduce may read a compact typed evidence spool once; it may not
read the raw corpus or retained UTF-8 training surfaces again.

```text
BufRead corpus lines                                              CORPUS PASS = 1
|
+-> normalize and deduplicate lexical surface
+-> assign deterministic TerminalId
+-> generate deterministic train / heldout split
+-> encode clean and damaged surfaces once into typed NGramKey events
+-> update decoder builder
+-> emit AtomOccurrence { ngram_key, terminal_id, strength, phase, position }
+-> emit SurfaceEvidence { terminal_id, class, atom-span, geometry summary }
+-> update fixed heldout reservoir
+-> discard source line and generated UTF-8 surfaces
|
v
bounded sharded spools
|
+-> external sort / merge AtomOccurrence by NGramKey
+-> assign dense AtomId
+-> reduce atom support, flags, forward postings and reverse ranges together
+-> expose the complete real candidate field
|
v
single compact SurfaceEvidence reduction                         EVIDENCE PASS = 1
|
+-> join atom spans to dense AtomId
+-> construct the real candidate lattice once
+-> accumulate primary positive centers
+-> accumulate directional anti-centers
+-> accumulate positive / anti / hard-negative subcenters
+-> accumulate ambiguity and tied-basin evidence
+-> update bounded calibration histograms and quantile sketches
+-> discard evidence record
|
v
deterministic bounded merge
|
+-> settle centers and directional banks
+-> derive thresholds from accumulated sufficient statistics
+-> encode the package directly
+-> checksum and decoder/package roundtrip
|
v
read-only package + independent fixed heldout reservoir          PROOF, NOT TRAINING
|
+-> clean preservation
+-> all damage classes
+-> ambiguity / false-certainty gates
+-> p50 / p99
+-> package bytes and peak RSS
`-> PASS_shadow or WATCH_shadow
```

The compact evidence spool is not a second corpus. It contains typed IDs,
bounded geometry summaries and atom spans, not raw lines, words or generated
damage strings. It exists because a surface cannot be evaluated against
competitors that have not yet entered the global posting field. Removing this
deferred join would make learning dependent on corpus order or blind to future
competitors.

#### Mandatory invariants

```text
raw corpus passes                         exactly 1
raw corpus read API                       BufRead, never read_to_string
retained raw corpus                       false
retained generated damage UTF-8           false after event emission
compact evidence reductions              exactly 1
full runtime readouts during training     0
temporary package rebuilds                0
candidate source                          real complete posting field
calibration state                         bounded histograms/sketches
positive/anti/subcenter banks             hard bounded
worker-count parity                       byte-identical package
input-order policy                        explicit and deterministic
heldout use during learning               forbidden
proof                                     separate read-only phase
```

One executable invocation is not sufficient evidence of this contract.
Receipts must report `raw_corpus_passes`, `compact_evidence_passes`,
`training_full_readouts`, spool bytes, peak RSS and stage wall times. Promotion
is forbidden unless the counters are `1`, `1` and `0` respectively and the
conjunctive quality gate in section 12 also passes.

#### Non-regression boundary for the streaming rewrite

The streaming rewrite changes cold dataflow only. It must not redesign or
remove the accepted L1.1 field:

```text
preserve typed atom channels
preserve the complete posting candidate field
preserve WordCenter64 and decoder semantics
preserve positive centers
preserve directional anti-centers
preserve positive / anti / hard-negative subcenters
preserve ambiguity and tied-basin readout
preserve restoration geometry and authority rules
preserve every fixed heldout damage class
```

The current complete staged run is the migration baseline. Its package,
checksum, compile counters, full per-class matrix and system budgets must be
captured before replacing the compiler. The streaming compiler is accepted
only when the same final corpus proves:

```text
all 13 damage classes                    no regression and every class > 95%
clean preservation                       no regression and >= 99.9%
lattice coverage                         no regression and every class >= 99%
false authority / false singleton        remain 0
runtime readout semantics                parity
decoder surfaces                         exact parity
package determinism                      PASS across worker counts
raw corpus passes                        1
compact evidence passes                  1
training full readouts                    0
```

Package bytes need not be byte-identical to the staged package if calibration
statistics are encoded more compactly, but decoded centers, decisions and
proof outcomes must satisfy the parity and non-regression gates above. A speed
improvement cannot compensate for a failed quality class.

#### Final hybrid-5 package VETO

The complete `762,314`-center hybrid-5 run finished and produced a package, but
the package is rejected. This is a measured result, not an estimate:

```text
package bytes                         1,825,863,344 = 1.700 GiB
required package ceiling                204,472,320 = 195 MiB
compile time                           12,330.240 s
fixed heldout proof time                  637.883 s
peak RSS                            7,384,752,128 B = 6.878 GiB
clean preservation                         100.000%
classes passing unique top-1 >95%              1 / 13
false authority / false singleton               0 / 0
hot L1.1 p50 / p99                         5.500 / 8.833 ms
verdict                                     VETO_SIZE
quality verdict                          WATCH_shadow
runtime authority                           unchanged
```

Artifact:
`/home/e/build/lay-l1-shadow/artifacts/l11-final-hybrid5-bounded1-evidence-only-762314-2026-07-25/package.bin`.
SHA-256:
`259f0a528ed64bcb31ed6f57c0c7277e1e92d076594c789a56aa227d38a16e35`.
Receipt:
`docs/structural_gates/receipts/L1_L11_FINAL_HYBRID5_BOUNDED1_762314_2026-07-26.json`.

The exact package-size decomposition is:

```text
compressed forward postings       775.185 MiB
reverse couplings                 600.008 MiB
positive subcenters               185.090 MiB
primary WordCenter64               46.528 MiB
ambiguity subcenters               43.202 MiB
keyboard geometry                  26.556 MiB
phase profiles                     17.448 MiB
hard-negative subcenters           13.702 MiB
atoms                              13.543 MiB
decoder                            10.033 MiB
graph nodes/arcs                    9.952 MiB
```

Forward plus reverse coupling storage accounts for `78.98%` of the file. The
current compiler also feeds damaged hybrid training surfaces through
`word_surfaces()` while compiling the primary graph, atom support, forward
postings and reverse couplings. This violates the intended ownership boundary:
damaged evidence must train bounded residual banks, not expand the clean
primary crystal.

The required correction is architectural:

```text
clean lexical surfaces
-> primary graph + primary postings + WordCenter

damaged training evidence
-> bounded positive / anti / hard-negative / ambiguity residual banks
-> never primary AtomWordCoupling
```

The `195 MiB` package ceiling is a preflight gate. Before any final-scale run,
the compiler must compute a conservative encoded-size upper bound from section
counts and reject the configuration if that bound exceeds `195 MiB`. A run
must not consume hours merely to discover a deterministic size violation.

#### Canonical final L1.1 configuration

The final target is one conjunctive configuration. Passing isolated parts is
not sufficient:

```text
corpus
  RU 462k + EN 300k
  762,314 deterministic WordCenter identities

primary crystal
  depth = 0
  clean lexical surfaces only
  damaged surfaces never enter primary graph/postings/couplings

residual L1.1 learning
  damaged evidence enters bounded residual banks only
  positive + directional anti + hard-negative + ambiguity
  no unbounded or mirrored full relation field

cold dataflow
  raw corpus passes = 1
  compact evidence reductions = 1
  full runtime readouts during training = 0
  all 20 remote hardware threads used
  deterministic shard merge

encoded package
  hard ceiling <= 195 MiB
  preflight upper-bound check before crystallization
  raw corpus stored = false
  exact damage episodes stored = 0

fixed proof
  13 damage classes
  20,000 heldout cases per class
  260,000 heldout cases total
  unique top-1 > 95% in every class
  lattice coverage >= 99% in every class
  clean preservation >= 99.9%
  false authority = 0
  false singleton = 0
  hot p99 <= 5.000 ms
```

The current evidence status is:

```text
depth-0 primary tested separately             yes
195 MiB package                               no
single-pass streaming compiler                no
20-worker execution                           yes
13 x 20,000 fixed heldout proof               yes
all requirements in one package               no
```

No package may be called final L1.1 until every line above is true in the same
receipt.

### Current L1.1 architecture and parameter snapshot, 2026-07-26

```text
broken surface
-> normalization
-> 11 typed atom channels
-> NGramGraph: atom key -> dense AtomId
-> candidate birth through inverted postings
-> bounded candidate frontier
-> forward + reconstruction wave
-> phase / geometry / sequence interference
-> Winner | Tied lattice | ABSTAIN
-> DecoderGraph: WordCenterId -> UTF-8 word
```

#### Atoms per input

| Channel | Size | Weight | Maximum atoms |
|---|---:|---:|---:|
| byte gram | 4 bytes | 2 | 24 |
| character bigram | 2 characters | 1 | 16 |
| character trigram | 3 characters | 3 | 24 |
| keyboard bigram | 2 keys | 1 | 16 |
| keyboard trigram | 3 keys | 3 | 24 |
| character bag trigram | 3 unordered characters | 3 | 24 |
| keyboard bag trigram | 3 unordered keys | 3 | 24 |
| character skip gram | distance 2-4 | 2 | 32 |
| keyboard skip gram | distance 2-4 | 2 | 32 |
| boundary prefix/suffix | length 1-3 | 3 | 6 |
| character anchor | individual characters | 1 | up to 32 |

The encoder can produce at most `222` lexical atoms plus up to `32` anchors.
An anchor does not give birth to candidates. It verifies order and
reconstruction.

#### Main candidate-birth error: 4 -> 32

```text
BEFORE:
each lexical channel
-> sort by rarity
-> take(4)
-> at most 40 birth atoms
-> most of the signal was discarded before interference

AFTER:
each lexical channel
-> sort by degree ASC, weight DESC, AtomId ASC
-> take(32)
-> at most 222 birth atoms
```

Because every individual channel generator is itself bounded by `32`,
`birth=32` means that the complete generated atom field is used without an
additional runtime truncation.

#### Bounded runtime

```text
complete posting activation
-> reconstruction scan             8,192 candidates
-> geometry scan                   1,024 candidates
-> primary phase frontier            128 candidates
-> operator reserve                   64
-> reconstruction reserve             64
-> geometry reserve                   32
-> maximum before settlement          288, with deduplication
-> pairwise lattice                     8
-> tied output                         32
-> settling iterations                  3
```

#### Wave kernel

```text
complex wave dimension              128 re + 128 im
AtomWaveCode                          4 basis components
AtomWaveCode                         16 bytes
WordCenter64                         22 basis components
phase code inside WordCenter         44 bytes
center metadata                      20 bytes
WordCenter total                     64 bytes
position buckets                     16
```

`WordCenter64` stores its wave code, reverse-coupling range, anti range,
decoder terminal, support, stability, surface length and script flags. The
word itself exists only in `DecoderGraph`.

#### Phase banks per center

```text
positive subcenters                 maximum 4
anti subcenters                     maximum 4
hard-negative subcenters            maximum 2
ambiguity subcenters                maximum 8
ambiguity centers per relation      maximum 2
directional anti relations          maximum 16
pairwise candidates                 maximum 8
```

#### Format V6

```text
header                              192 bytes
NGramGraph node                      12 bytes
NGramGraph arc                        8 bytes
AtomRecord                           28 bytes
raw WaveCoupling                      8 bytes
compressed posting block             32 relations
posting block header                  8 bytes
CenterPhaseProfile                   24 bytes
PairPhaseProfile                     24 bytes
DecoderNode                           8 bytes
WordCenter                           64 bytes
```

#### Depth-0 10k PASS

```text
words                               10,000
atoms                               57,831
forward couplings                1,296,290
reverse couplings                1,031,133
training damaged surfaces                0
package                         18,905,577 B = 18.03 MiB
clean preservation                  100.000%
overall top-1                        98.155%
top-64                               99.992%
weakest class                         95.597%
false certainty                            0
hot p50 / p99                   1.725 / 1.990 ms
classes above 95%                       13/13
```

#### Current full depth-0 package

```text
WordCenter                         762,314
atoms                              204,324
forward couplings               98,768,909
reverse couplings               78,644,261
WordCenter bank                48,788,096 B
positive subcenters               762,314
ambiguity subcenters              760,876
anti / hard-negative / pairs            0
training damaged surfaces                0
package                       1,175,182,559 B = 1.094 GiB
required ceiling               204,472,320 B = 195 MiB
```

The full depth-0 package currently stores almost one positive and ambiguity
subcenter for every word and duplicates tens of millions of relations in the
forward and reverse directions. That volume is not functionally justified for
depth-0 and is the next compression target.

#### Depth-0 V7 compaction experiments, 2026-07-26

The first V7 experiment retained at most `32` forward relations per
WordCenter and reconstructed only reverse relations and keyboard geometry.
Its size passed, but the fixed proof rejected it:

| Damage class | Unique top-1 |
|---|---:|
| adjacent transposition | 96.030% |
| double substitution | 96.797% |
| extra letter | 35.536% |
| layout projection | 99.900% |
| letter substitution | 99.499% |
| missing letter | 30.444% |
| non-adjacent transposition | 95.080% |
| omission + transposition | 20.243% |
| prefix truncation | 26.205% |
| punctuation suffix | 100.000% |
| repeated fragment | 32.850% |
| sparse multi-omission | 21.767% |
| suffix truncation | 22.256% |

```text
10k sparse-32 package                 5,721,472 B
overall top-1                            59.682%
clean preservation                     100.000%
top-64                                  99.946%
false authority                               0
false singleton                               0
hot p99                                  7.276 ms
verdict                               WATCH_shadow
```

The causal result is unambiguous: limiting relations per WordCenter compressed
knowledge rather than its representation. The sparse-32 storage design is
rejected even though its lattice coverage and disk size passed.

The corrected V7 ownership rule preserves the complete depth-0 field:

```text
stored:
  NGramGraph
  AtomWaveCode bank
  primary WordCenter64 bank
  DecoderGraph
  restoration calibration

not stored:
  forward coupling bank
  reverse coupling bank
  primary-equivalent positive subcenters
  zero-authority ambiguity subcenters
  per-center keyboard geometry

reconstructed on load from DecoderGraph + NGramGraph:
  exact atom occurrence support
  complete forward coupling field
  complete reverse coupling field
  physical keyboard geometry
  zero-residual CenterPhaseProfile
```

The fixed 10k source V6 contains `1,296,290` forward and `1,031,133` reverse
relations. The corrected implicit-field V7 is `4,148,280 B`; its unit crystal
has exact package parity after encode/decode.

The fixed 13-class proof now confirms that removing the redundant disk banks
does not remove their runtime knowledge:

| Damage class | Unique top-1 | Lattice coverage |
|---|---:|---:|
| adjacent transposition | 99.799% | 100.000% |
| double substitution | 99.299% | 100.000% |
| extra letter | 99.099% | 100.000% |
| layout projection | 99.900% | 99.900% |
| letter substitution | 99.950% | 100.000% |
| missing letter | 98.317% | 100.000% |
| non-adjacent transposition | 99.649% | 100.000% |
| omission + transposition | 95.597% | 100.000% |
| prefix truncation | 96.974% | 100.000% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.300% | 100.000% |
| sparse multi-omission | 96.860% | 100.000% |
| suffix truncation | 97.141% | 100.000% |

```text
10k implicit-field V7 package           4,148,280 B
overall top-1                              98.155%
clean preservation                       100.000%
top-64                                     99.992%
false authority                                  0
false singleton                                  0
hot p50 / p99                         2.890 / 3.977 ms
verdict                              PASS_shadow_10k
```

This proves the representation change at 10k scope. It does not yet prove the
full `762,314`-center package, full `13 x 20,000` matrix, or the single-pass
streaming crystallizer. Runtime authority remains unchanged.

Runtime authority did not change. Exact evidence:

```text
/home/e/build/lay-l1-shadow/artifacts/l11-depth0-sparse32-10k-2026-07-26/proof.json
/home/e/build/lay-l1-shadow/artifacts/l11-depth0-sparse32-10k-2026-07-26/proof.log
/home/ubu/projects/lay/artifacts/l11-depth0-implicit-v7-10k-2026-07-26/package.bin
/home/ubu/projects/lay/artifacts/l11-depth0-implicit-v7-10k-2026-07-26-compaction.json
/home/ubu/projects/lay/artifacts/l11-depth0-implicit-v7-10k-2026-07-26-compaction.time
/home/ubu/projects/lay/artifacts/l11-depth0-implicit-v7-10k-proof-2026-07-26/report.json
/home/ubu/projects/lay/artifacts/l11-depth0-implicit-v7-10k-proof-2026-07-26/run.log
/home/ubu/projects/lay/artifacts/l11-depth0-implicit-v7-10k-proof-2026-07-26/time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_DEPTH0_IMPLICIT_V7_10K_2026-07-26.json
```

#### Immutable proof gate

```text
13 classes x 20,000               260,000 cases
unique top-1 for every class       >95.0%
lattice coverage for every class  >=99.0%
clean preservation                >=99.9%
false authority                          0
false singleton                          0
package                             <=195 MiB
hot p99                             <=5.0 ms
```

#### Full-field V7 selector experiments

The full `762,314`-center fixed proof established that unbounded `birth=32`
does not satisfy the latency gate and does not close every quality class:

```text
cases                              260,000 / 260,000
workers                                           20
wall time                                    1:29:56
peak RSS                                     3.09 GiB
sparse multi-omission                         94.2635%
hot p99                                      34.531 ms
quality classes passing                         12/13
```

The package representation is not the cause: exact V6/V7 runtime-field parity
allows this functional result to transfer to the `66.11 MiB` V7 package. The
runtime selector is the failing component. A bounded selector now uses four
birth atoms per typed channel and at most `131,072` postings per readout.

The first typed inverse probe removed the latency failure and recovered sparse
multi-omission, but an unbounded cross-distance certificate displaced
one-omission and truncation basins:

```text
scope                              13 x 2,000
hot p99                              3.115 ms
sparse multi-omission                  96.252%
missing letter                         70.999%
prefix truncation                      66.127%
suffix truncation                      68.210%
verdict                           REJECTED_shadow
receipt
/home/e/build/lay-l1-shadow/artifacts/l11-v7-typed-inverse-probe-2k-762314-2026-07-26/report.json
```

A single cross-distance energy lease of `1,500` was also rejected. It restored
most one-omission basins but suppressed the intended two-omission inverse:

```text
scope                              13 x 2,000
hot p99                              3.138 ms
sparse multi-omission                  92.716%
missing letter                         96.576%
prefix truncation                      94.985%
suffix truncation                      80.864%
verdict                           REJECTED_shadow
receipt
/home/e/build/lay-l1-shadow/artifacts/l11-v7-typed-lease1500-probe-2k-762314-2026-07-26/report.json
```

This experiment did not test a full `13 x 20,000` matrix and did not alter
runtime authority. The next selector must preserve distinct operator
semantics: exact boundary truncation, one omission and two sparse omissions
cannot share one rank and one energy lease.

The operator-aware selector implements that separation. It keeps the
two-omission inverse strong against an untyped incumbent, prevents it from
displacing a stronger one-omission inverse, and gives exact prefix/suffix
completion its own boundary certificate.

```text
scope                              13 x 2,000
quality classes >95%                    13/13
overall top-1                         85.742%
top-64                               99.715%
clean preservation                  100.000%
false certainty                            0
hot p99                               3.246 ms

missing letter                         99.022%
prefix truncation                      99.151%
sparse multi-omission                  96.535%
suffix truncation                      99.074%

lowest lattice coverage:
omission + transposition                98.900%
verdict                    PASS_probe_top1_only
receipt
/home/e/build/lay-l1-shadow/artifacts/l11-v7-operator-aware-probe-2k-762314-2026-07-26/report.json
```

The short probe proves the strict per-class top-1, clean, false-certainty and
latency gates at 2k scope. It does not prove the lattice gate because one class
is `0.1 pp` below it. The previous fixed full baseline measured `99.500%`
lattice coverage for that class, so promotion is decided only by the fixed
`13 x 20,000` proof rather than by further tuning against this small slice.
Runtime authority remains unchanged.

#### Final full-corpus V7 proof

The operator-aware selector passed the complete fixed proof:

| Damage class | Unique top-1 | Lattice coverage |
|---|---:|---:|
| adjacent transposition | 98.062% | 99.605% |
| double substitution | 95.455% | 99.895% |
| extra letter | 98.912% | 99.990% |
| layout projection | 98.342% | 99.615% |
| letter substitution | 99.945% | 99.640% |
| missing letter | 98.909% | 99.320% |
| non-adjacent transposition | 96.320% | 99.530% |
| omission + transposition | 95.701% | 99.040% |
| prefix truncation | 99.127% | 99.965% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.858% | 100.000% |
| sparse multi-omission | 96.856% | 99.720% |
| suffix truncation | 99.175% | 100.000% |

```text
WordCenter                              762,314
fixed heldout cases                     260,000
workers                                      20
proof wall time                         4m 33.76s
peak RSS                               2.215 GiB
package                           69,325,620 B
package                                66.11 MiB
package SHA-256  232de23384dc1d62977a5c244a2fed14615c03d2818ee79618ade3a618456ab4
overall top-1                            85.422%
top-64                                  99.717%
clean preservation                     100.000%
false authority                               0
false singleton                               0
hot p50 / p99                    2.506 / 3.146 ms
verdict                             PASS_shadow
```

All conjunctive L1.1 gates pass on the combined bilingual heldout. This is a
shadow architecture proof, not a live promotion: daemon, IME and
`AuthorizedEdit` authority remain unchanged. A combined bilingual denominator
is not sufficient for promotion because it can hide a language-specific
failure. The experiment did not prove the future single-pass streaming
crystallizer.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_OPERATOR_AWARE_V7_FINAL_762314_2026-07-26.json
```

#### Split-language V7 proof and promotion veto

The same immutable V7 package was then proved separately over its global
terminal ranges. The proof still loads the complete bilingual dictionary so
candidate IDs and cross-language competitors remain identical to package
construction. Only heldout generation and clean audit are restricted to the
declared language range.

Russian terminal range `0..462314` passes:

| Damage class | Unique top-1 | Lattice coverage |
|---|---:|---:|
| adjacent transposition | 97.935% | 99.970% |
| double substitution | 98.000% | 99.945% |
| extra letter | 98.706% | 100.000% |
| layout projection | 97.210% | 99.480% |
| letter substitution | 99.958% | 99.905% |
| missing letter | 98.764% | 99.985% |
| non-adjacent transposition | 97.368% | 99.880% |
| omission + transposition | 96.641% | 99.555% |
| prefix truncation | 99.107% | 99.975% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.950% | 100.000% |
| sparse multi-omission | 96.908% | 99.690% |
| suffix truncation | 98.612% | 100.000% |

English terminal range `462314..762314` does not pass:

| Damage class | Unique top-1 | Lattice coverage |
|---|---:|---:|
| adjacent transposition | 98.788% | **98.900%** |
| double substitution | **90.824%** | 99.845% |
| extra letter | 99.439% | 99.970% |
| layout projection | 100.000% | 99.825% |
| letter substitution | 99.866% | **98.920%** |
| missing letter | 99.194% | **97.895%** |
| non-adjacent transposition | **94.370%** | **98.960%** |
| omission + transposition | **93.183%** | **98.140%** |
| prefix truncation | 99.481% | 99.875% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.546% | 100.000% |
| sparse multi-omission | 95.564% | 99.740% |
| suffix truncation | 99.468% | 99.990% |

```text
metric                         RU462314          EN300000
verdict                      PASS_shadow       WATCH_shadow
heldout cases                    260,000            260,000
clean preservation               100.000%           100.000%
false authority                         0                   0
false singleton                         0                   0
hot p99                              3.199 ms            3.166 ms
wall time                         4m 11.44s           5m 01.59s
peak RSS                           2.185 GiB           2.132 GiB
```

The combined `PASS_shadow` therefore cannot admit this package for release
installation. The package remains a valid measured research artifact, but its
promotion verdict is `VETO_EN_SPLIT_LANGUAGE`. The installer must require
separate RU and EN `PASS_shadow` receipts in addition to the combined proof.
Runtime authority remains unchanged.

Failure decomposition identifies an English operator-separation problem rather
than candidate loss: target-missing top-64 is zero in the three failing top-1
classes. Double substitution has `1,668` unique failures, including `1,121`
edit-distance ties. Non-adjacent transposition has `930` unique failures, of
which `844` select a genuinely closer competing center. Omission plus
transposition has `776` unique failures, with sequence evidence favoring the
target in `728` cases but the current settlement still selecting another
center. The next L1.1 experiment must alter English operator-conditioned
settlement and lattice admission, not widen candidate birth or rebuild the
stored V7 field.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_OPERATOR_AWARE_V7_RU462314_2026-07-26.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_OPERATOR_AWARE_V7_EN300000_2026-07-26.json
```

#### Certificate-preserving bounded tail

The English split failures were reduced without rebuilding or changing the
immutable V7 package. Direct surface operators now outrank generic
cross-script keyboard projection, exact words expand the operator lattice only
when `limit > 1`, and proof diagnostics keep independent quotas for unique and
ambiguous failures.

A broad inverse insertion lane over every lattice readout was measured and
rejected:

```text
missing-letter lattice             98.150% -> 98.150%
omission+transposition lattice     98.650% -> 98.750%
hot p99                              3.199 -> 5.715 ms
verdict                             WATCH_shadow
```

The extra enumeration did not repair missing-letter retention and crossed the
`5 ms` latency gate. It is not part of the accepted runtime.

Terminal trace `636673`, surface `abo`, target `aibo`, exposed the actual
retention defect:

```text
target before bounded truncation       rank 42 / 174
target reconstruction mode             single deletion
target after bounded truncation        absent
```

`truncate_with_reconstruction_tail` had unconditionally removed original
ranks `33..64` whenever it admitted 32 reconstruction candidates from beyond
the limit. The corrected bounded rule may replace only weak in-limit
candidates with no reconstruction certificate. It cannot evict operator
evidence already inside the 64-center lattice.

English `13 x 2,000` proof after that correction:

| Damage class | Unique top-1 | Lattice coverage |
|---|---:|---:|
| adjacent transposition | 100.000% | 99.950% |
| double substitution | 96.198% | 100.000% |
| extra letter | 99.354% | 100.000% |
| layout projection | 98.898% | 99.900% |
| letter substitution | 99.939% | 100.000% |
| missing letter | 99.439% | 99.450% |
| non-adjacent transposition | 95.424% | 100.000% |
| omission + transposition | 95.376% | 99.800% |
| prefix truncation | 98.983% | 100.000% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.349% | 100.000% |
| sparse multi-omission | 96.607% | 100.000% |
| suffix truncation | 98.734% | 100.000% |

```text
clean preservation               100.000%
false authority                          0
false singleton                          0
hot p99                           3.307 ms
verdict                         PASS_shadow
```

Measured artifact:

```text
/home/e/build/lay-l1-shadow/artifacts/l11-v7-en-certificate-preserving-tail-probe-2k-2026-07-26/report.json
```

This probe did not test the final `13 x 20,000` split denominator. Runtime
authority, daemon, IME and `AuthorizedEdit` were not changed.

#### Final certificate-preserving split proof

The same immutable `69,325,620 B` V7 package passed both fixed language
denominators after the bounded-tail correction.

| Damage class | RU unique top-1 | RU lattice | EN unique top-1 | EN lattice |
|---|---:|---:|---:|---:|
| adjacent transposition | 99.954% | 100.000% | 99.945% | 99.990% |
| double substitution | 98.568% | 100.000% | 96.606% | 100.000% |
| extra letter | 98.670% | 100.000% | 99.357% | 99.980% |
| layout projection | 97.007% | 99.470% | 98.776% | 99.905% |
| letter substitution | 99.969% | 100.000% | 99.957% | 99.995% |
| missing letter | 99.244% | 99.985% | 99.339% | 99.550% |
| non-adjacent transposition | 97.862% | 100.000% | 95.847% | 100.000% |
| omission + transposition | 97.443% | 99.670% | 95.247% | 99.585% |
| prefix truncation | 98.400% | 99.975% | 98.804% | 99.990% |
| punctuation suffix | 100.000% | 100.000% | 100.000% | 100.000% |
| repeated fragment | 98.945% | 100.000% | 98.541% | 100.000% |
| sparse multi-omission | 96.876% | 99.690% | 95.537% | 99.990% |
| suffix truncation | 98.612% | 100.000% | 99.371% | 100.000% |

```text
metric                         RU462314          EN300000
verdict                      PASS_shadow       PASS_shadow
heldout cases                    260,000            260,000
overall top-1                    88.313%            81.932%
overall top-64                   99.907%            99.922%
clean preservation              100.000%           100.000%
false authority                        0                   0
false singleton                        0                   0
hot p50 / p99              2.443 / 3.083 ms   2.641 / 3.498 ms
wall time                         4m 12.68s           5m 01.89s
peak RSS                          2.183 GiB           2.132 GiB
```

Every accepted working gate is conjunctively satisfied in each language:
unique top-1 is strictly above `95%` for every damage class, lattice coverage
is at least `99%` for every class, clean preservation is `100%`, and false
authority and false singleton counts are zero. The package is admitted for
shadow installation only. `runtime_authority=false`; daemon, IME and
`AuthorizedEdit` authority remain unchanged.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_OPERATOR_AWARE_V7_RU462314_2026-07-26.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_OPERATOR_AWARE_V7_EN300000_2026-07-26.json
```

Exact proof reports:

```text
/home/e/build/lay-l1-shadow/artifacts/l11-v7-ru-certificate-preserving-tail-full-20k-2026-07-26/report.json
/home/e/build/lay-l1-shadow/artifacts/l11-v7-en-certificate-preserving-tail-full-20k-2026-07-26/report.json
```

The full split proof did not test live authority or the future single-pass
streaming crystallizer.

#### RU lexical coverage veto after smoke

Post-proof smoke exposed a corpus-scope failure that the fixed heldout cannot
measure: `ландо` exists in the RU462k package, but the required target `ладно`
does not exist as a WordCenter. It is present in
`data/lexicon/l2_surface_foundation_ru_100k.txt`.

The cause is structural. The RU462k lexical source was extracted only from
`LAY-RU-NOUN-MORPH-462K-SHADOW-v1`, so it contains noun paradigms rather than a
complete general Russian lexical substrate. The split `PASS_shadow` remains
valid for its declared package dictionary, but it does not prove general RU
lexical coverage.

```text
ландо package center                  present
ладно package center                  absent
ландо -> ладно restoration            impossible
runtime authority                     unchanged / false
promotion verdict                     VETO_RU_LEXICAL_COVERAGE
```

The v1 package may remain installed as a shadow research artifact. It is not
release-installable. The v2 source must be a deterministic union of morphology
and broad lexical sources, and package parity must prove every source surface
before damage-class percentages can promote it.

#### Composite RU lexical v2 closure

The corpus-coverage veto was closed by rebuilding the source as a deterministic
union of noun morphology and broad Russian lexical sources. The English range
was preserved unchanged.

```text
RU surfaces                         535,410
EN surfaces                         300,000
total WordCenter                    835,410
corpus SHA-256                      162f1471f6a1f635aec634e2644123597ae60a46158314d7d9765474c552ee65
```

Depth-0 crystallization now skips directional anti discovery and learned
positive/anti/ambiguity subcenter search. It still materializes the complete
deterministic keyboard geometry required for V6/V7 runtime-field parity.

```text
workers                                      20
crystallization wall time                3m 14.24s
crystallization peak RSS          3,926,888,448 B
V6 bytes                         1,180,567,557 B
V7 compaction wall time                  37.68s
V7 bytes                            76,246,568 B
V7 budget                          204,472,320 B
V7 SHA-256               4bafc58c71e1343aaf3b1edfeb773303d0daf072a77c5ffabda7c182a1c63d8b
exact runtime-field parity                    true
```

The final immutable V7 package passed both fixed `13 x 20,000` language
denominators:

| Damage class | RU unique top-1 | RU lattice | EN unique top-1 | EN lattice |
|---|---:|---:|---:|---:|
| adjacent transposition | 99.972% | 100.000% | 99.945% | 99.990% |
| double substitution | 98.455% | 100.000% | 96.611% | 100.000% |
| extra letter | 98.777% | 100.000% | 99.346% | 99.980% |
| layout projection | 96.912% | 99.175% | 99.277% | 99.685% |
| letter substitution | 99.968% | 100.000% | 99.957% | 99.995% |
| missing letter | 99.286% | 99.975% | 99.366% | 99.540% |
| non-adjacent transposition | 97.651% | 100.000% | 95.853% | 100.000% |
| omission + transposition | 97.367% | 99.560% | 95.414% | 99.585% |
| prefix truncation | 98.343% | 99.985% | 98.815% | 99.995% |
| punctuation suffix | 100.000% | 100.000% | 100.000% | 100.000% |
| repeated fragment | 98.914% | 100.000% | 98.551% | 100.000% |
| sparse multi-omission | 96.813% | 99.575% | 95.307% | 99.975% |
| suffix truncation | 99.222% | 100.000% | 99.323% | 100.000% |

```text
metric                         RU535410          EN300000
verdict                      PASS_shadow       PASS_shadow
heldout cases                    260,000            260,000
clean preservation               100.000%           100.000%
false authority                         0                   0
false singleton                         0                   0
hot p50 / p99              2.497 / 3.056 ms   2.675 / 3.464 ms
wall time                         4m 30.47s           5m 17.09s
peak RSS                           2.349 GiB           2.304 GiB
```

Smoke was performed on V7, not on V6:

```text
врмея -> время                    target rank 1
restoraton -> restoration         target rank 1
ландо -> ладно                    target rank 5
qzxqzxqzx                         ABSTAIN / outside calibrated basin
```

`ландо` itself is also a clean WordCenter and therefore remains rank 1.
L1.1 correctly retains `ладно` in the transposition lattice but cannot grant
authority to replace one valid exact center with another. Contextual collapse
of that collision belongs to L2/L3 and was not tested here.

Local shadow installation was verified separately:

```text
installed package
  /home/ubu/.local/share/lay/nanda_wave/l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-SHADOW-v2.v7.bin
installed CLI
  /home/ubu/.local/lib/lay/bin/lay-l1.1-restore
installed/source SHA-256 parity                    true
врмея candidate rank                                  1
candidate surface                                 время
authority                                          false
verdict                           ABSTAIN / outside_calibrated_basin
cold standalone CLI wall time                     60.05 s
cold standalone CLI peak RSS                1,813,380 KiB
```

The cold standalone number measures package decode and reconstruction of the
runtime views in a new process. It is not the hot readout latency measured by
the fixed proof. This smoke tested one installed-package query; it did not test
daemon integration, a persistent package server or mmap startup optimization.

#### Local L1.1 sidecar route

A local sidecar service has now been implemented so that L1.1 can be loaded
once and reused by the CLI and the daemon's shared correction route without
changing runtime authority. IME preedit authority is unchanged and does not
query this service:

```text
binary                               lay-l1.1-serve
transport                            Unix socket
default socket                       $XDG_RUNTIME_DIR/lay-l11.sock
admin commands                       run / health / stats / reload
client route                         lay-l1.1-restore --socket ...
direct fallback                      lay-l1.1-restore --memory ...
authority                            unchanged / false
```

The service binds its socket immediately and warms the package in a background
thread. While the package is loading, `health` and `stats` report
`status=warming` and `restore` returns a typed error instead of blocking the
caller invisibly. The daemon now discovers the installed shadow package from:

```text
/home/ubu/.local/share/lay/nanda_wave/l1.1/*.installed.json
```

and starts or reloads `lay-l1.1-serve` during startup warmup without waiting
for full package readiness.

Measured socket smoke on the existing 10k package:

```text
package                    data/lexical_grokking/l1_l11_multimodal_restoration_10k.bin
service health             ready
terminal_count             10,000
requests_served            3
socket restore             врмея -> время
restore route              lay-l1.1-restore --socket /tmp/lay-l11-test.sock врмея
```

The live correction hook is now:

```text
typing assist boundary
-> correction_core::nanda_text_candidates_for_route()
-> hot_l2_text_candidates()
-> local L1.1 sidecar winner as extra shared candidate source
-> TransitionDecisionCore
-> AuthorizedEdit or no-op
```

The sidecar candidate is inserted only on the shared `CompactL2` correction
path. It uses:

```text
origin                    CandidateOrigin::L2Surface
source_id                 L11SurfaceRestore
default timeout           12 ms
fallback on missing/
warming/error             skip candidate, keep existing route
runtime authority         unchanged / false
```

Measured facts in this integration change:

```text
cargo-guard check                         OK
lib test l11_sidecar_candidate*          2/2 OK
lib test authority parse                  1/1 OK
lib test installed package discovery      1/1 OK
lay-daemon warmup plan tests              4/4 OK
```

Not tested in this change:

```text
full 835,410-package ready-state warmup time inside the sidecar
live daemon end-to-end typing session against the installed release binary
IME preedit behavior, because IME authority must remain separate
mmap-backed hot-image startup optimization
```

This proves the local sidecar contract, the shared JSON protocol, the
correction_core insertion point and the daemon warmup hook. It does not
promote runtime authority or change the IME route.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_COMPOSITE_V2_V7_FINAL_835410_2026-07-26.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_COMPOSITE_V2_V7_RU535410_2026-07-26.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_COMPOSITE_V2_V7_EN300000_2026-07-26.json
```

Exact remote reports:

```text
/home/e/build/lay-l1-shadow/artifacts/l11-final-depth0-v2-835410-2026-07-26/crystallize-v6.log
/home/e/build/lay-l1-shadow/artifacts/l11-final-depth0-v2-835410-2026-07-26/compact-v7.log
/home/e/build/lay-l1-shadow/artifacts/l11-final-depth0-v2-835410-2026-07-26/proof-ru-20k.log
/home/e/build/lay-l1-shadow/artifacts/l11-final-depth0-v2-835410-2026-07-26/proof-en-20k.log
```

The measured verdict is `PASS_SHADOW_ONLY`. This experiment did not promote or
change daemon authority, IME authority, or the ownership of `AuthorizedEdit`.
It only made L1.1 available as a shadow extra candidate inside the existing
shared correction decision path.

### V8 lossless lazy runtime, 2026-07-29

V8 is a runtime representation of the accepted RU535,410 + EN300,000 depth-0
V7 crystal. It is not a new corpus crystallization and does not remove,
truncate or retrain a relation:

```text
V7 immutable compact base
-> mmap
-> complete forward-posting index
-> independently compressed posting shards
-> bounded posting/shard caches
-> reverse relations reconstructed only for active WordCenter
-> bounded reverse cache
```

Measured full-field identity:

```text
WordCenter                                  835,410
atoms                                       218,387
forward relations                       108,156,559
reverse relations                        86,120,980
relations removed                                  0
V7 compact base                         76,246,568 B
posting index                            3,494,192 B
```

The first measured V8 package used 54 shards with 4,096 atoms per shard:

```text
compressed postings                     99,728,649 B
package                                179,470,401 B = 171.16 MiB
SHA-256                 738c2a5ee25237c5bb5513f108fc9298b38488aa93076491531d9217b294726e
вреям hot p50 / p99                   3.139 / 3.527 ms
steady RSS                                167.95 MiB
```

The V7/V8 exact relation and readout test passed for every fixture relation and
for `время`, `вреям`, `работат` and `downlod`. The initial fixed-proof attempt
did not run because its reuse-package branch still called the V7-only decoder.
That ownership error was fixed by routing reused packages through the common
path-based V7/V8 loader.

A `1,000 centers x 100 cases x 13 classes` smoke then exposed a different
runtime problem. The proof reached the real V8 readout and preserved the
strict unique-class gate in all 13 smoke denominators, but large shards could
be decompressed concurrently by several proof workers:

```text
proof cases                                      1,300
clean preservation                             100.000%
minimum unique top-1                             96.939%
minimum lattice coverage                         99.000%
false authority                                       0
false singleton                                       0
proof peak RSS                                  2.622 GiB
reported diverse-surface p99                 2,229.263 ms
```

This smoke is not the fixed quality proof and its latency is not the accepted
hot latency measurement. Its scope was to expose runtime behavior under
diverse concurrent queries. It found duplicate in-flight shard decompression
and a shard working set too coarse for the 32 MiB cache.

The first corrected V8 layout used 256 atoms per shard and one lock per shard.
Different shards still decompressed concurrently; the same shard was decoded
once and then shared:

```text
atoms per shard                                      256
posting shards                                       854
compressed postings                     102,044,350 B
package                                181,798,902 B = 173.38 MiB
bytes per forward relation                        0.9435
SHA-256                 ddfd42cf85832664c7c89bf15264f3267a7c931d4a3ffffada48697a3f75d82d
repack wall time                                  50.31 s
repack peak RSS                               3,618,812 KiB
repack average CPU                                  462%
cold query wall time                               1.00 s
cold query peak RSS                             192,000 KiB
вреям hot p50 / p99                   3.425 / 4.134 ms
```

Measured gates for this corrected representation:

```text
package <= 195 MiB                                  PASS
cold process peak <= 250 MiB                        PASS
вреям hot p99 <= 5 ms                               PASS
all forward relations retained                      PASS
fixed 13 x 20,000 per-class proof                   NOT TESTED
diverse concurrent proof RSS/latency                FAIL
append-only live delta overlay                      NOT IMPLEMENTED
runtime authority                                   UNCHANGED
```

The 256-atom smoke preserved exactly the same quality counters and measured:

```text
smoke wall time                                  4m 38.31s
smoke peak RSS                               1,693,664 KiB
diverse first-touch p99                         319.194 ms
heldout evaluation                                38.725 s
```

This was a large improvement over 4,096 atoms per shard but still left
first-touch latency too high. The next bounded experiment used 32 atoms per
shard:

```text
atoms per shard                                       32
posting shards                                     6,825
compressed postings                     104,976,781 B
package                                184,826,869 B = 176.27 MiB
bytes per forward relation                        0.9706
SHA-256                 05ff08798cb1ff04db4ce9e233941732f3f859e9db0c69b6b34a42beebdd37d7
repack wall time                                  47.27 s
repack peak RSS                               3,281,704 KiB
repack average CPU                                  483%
cold query wall time                               0.91 s
cold query peak RSS                             152,884 KiB
вреям hot p50 / p99                   3.220 / 3.510 ms
smoke wall time                                  4m 05.80s
smoke peak RSS                               1,256,220 KiB
diverse first-touch p99                         238.719 ms
heldout evaluation                                37.125 s
```

The 32-atom representation is the current V8 candidate because it remains
inside the package and process-RSS gates and dominates the larger shards on
first-touch behavior. It does not yet pass the diverse first-touch latency
gate. The fixed `13 x 20,000` proof is running against this exact artifact.

The reconstruction lane was then changed to obtain V8 character anchors
directly from `DecoderGraph`. The previous lazy path rebuilt complete reverse
byte/character/keyboard relation vectors for up to 8,192 centers merely to
read ordered character anchors. Fixture parity now checks every center:

```text
eager V7 reverse-anchor sequence
==
lazy V8 DecoderGraph-derived anchor sequence              PASS

V7/V8 complete forward relation addressability            PASS
V7/V8 readout parity on fixed fixture surfaces             PASS
full-package post-change first-touch p99                   NOT MEASURED
```

The running fixed full proof started before this optimization and therefore
proves package quality with the previous process image. Its latency audit is
not evidence for the new DecoderGraph-anchor path. Latency is measured
separately after that proof completes.

The following cannot yet be claimed:

```text
final V8 quality PASS
full-corpus diverse-query p99 <= 5 ms
bounded RSS during the fixed parallel proof
production-scale delta latency and reload continuity
promotion into daemon, IME or AuthorizedEdit ownership
```

Exact remote evidence:

```text
/home/e/build/lay-l1-shadow/artifacts/l11-v8-full-2026-07-29/package.v8.bin
/home/e/build/lay-l1-shadow/artifacts/l11-v8-full-2026-07-29/proof-smoke/report.json
/home/e/build/lay-l1-shadow/artifacts/l11-v8-full-2026-07-29/proof-smoke/run.log
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard256-v2-2026-07-29/package.v8.bin
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard256-v2-2026-07-29/build-report.json
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard256-v2-2026-07-29/build.log
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/package.v8.bin
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/build-report.json
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/build.log
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/proof-smoke/report.json
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/proof-smoke/run.log
```

### Append-only V8 composite overlay, 2026-07-29

The immutable V8 base no longer has to be rebuilt for every local lexical
addition or removal. The first append-only composite contract is:

```text
lay-l11-composite-v1 manifest
-> immutable V8/V7 base
-> zero or more proof-receipted delta packages
-> normalized surface tombstones
-> generation counter
-> temp file + atomic rename on every manifest change
```

Each delta is a normal small L1.1 package. It may contain new atoms,
WordCenter, positive subcenters, anti subcenters and ambiguity evidence. The
base file is not rewritten. Runtime assigns every delta a disjoint global
terminal-ID range:

```text
base terminal IDs                         0 .. base_count
delta 1 terminal IDs             base_count .. base_count + delta_1_count
delta N terminal IDs             preceding end .. composite terminal_count
```

Readout resolves every package independently, converts local terminal IDs to
global IDs, removes tombstoned surfaces, sorts by evidence score and
deduplicates equal normalized surfaces. This retains new atoms that do not
exist in the base `NGramGraph`; they are resolved by the delta graph rather
than forced into the immutable base graph.

Manifest tombstones use the same lexical normalization function as atom
encoding and runtime readout, including terminal punctuation stripping. There
is no second tombstone-only normalization rule.

The safety contract is deliberately conservative:

```text
base package calibration remains immutable
delta admission requires an existing proof receipt
tombstone admission requires an existing proof receipt
receipt verdict must begin with PASS
composite Winner authority                         disabled
composite output                                   lattice / abstain
runtime authority                                  unchanged
```

This prevents a tiny delta corpus from treating its narrow local calibration
as authority over the 835,410-center base. L2 can consume the merged bounded
lattice immediately, but `AuthorizedEdit` cannot be promoted by the overlay.
The same rule applies before the first delta: a generation-1 manifest with
zero deltas and zero tombstones returns a non-authoritative lattice, not the
standalone base package's winner contract.

Service snapshot replacement was also corrected:

```text
old:
  acquire write lock
  -> load and validate complete replacement
  -> swap
  -> release write lock

new:
  keep serving current immutable host
  -> load and validate replacement outside the lock
  -> acquire write lock
  -> one host-pointer-sized ownership flip
  -> release write lock
```

Manifest writers use a private `flock` sidecar around the complete
read/validate/mutate/rename transaction. Atomic rename protects readers from a
partial JSON file; the writer lock separately prevents two simultaneous
admissions from reading the same generation and losing one update. Before the
rename, the temporary file is flushed with `fsync`; after the rename, the
parent directory is flushed. Thus a successful admission is also bounded
against immediate process or host failure, rather than only being atomically
visible in page cache.

The service health response carries `manifest_generation`. Reusing the same
manifest path after an append is not treated as ready when the daemon still
serves an older generation. `ensure_l11_service_started()` requests a reload
when generations differ. The reload response timeout is 5 seconds; the former
250 ms limit was below the measured 0.91-second V8 cold-load time.

Manifest parsing performs structural, file-size and receipt checks. It does
not checksum/decode the complete V8 and then ask the package loader to repeat
the same work. The package loader remains the single checksum/decode owner on
runtime load. Package/manifest detection reads one byte first; an ordinary
176 MiB binary V8 is not copied into a temporary `Vec` merely to determine
that it is not JSON.

Defensive format bounds:

```text
manifest bytes                              <= 8 MiB
delta packages                              <= 64
compaction recommendation                   >= 32 deltas or >= 32 MiB
tombstones                                  <= 100,000
normalized tombstone length                 <= 128 characters
```

These are storage/runtime safety bounds, not learned field coefficients.

Measured local integration fixture:

```text
base representation                                    V8
base centers                                            13
delta centers                                             1
composite centers                                        14
empty generation-1 composite restore authority         false
empty generation-1 composite verdict                 lattice
new exact surface resolved with global terminal ID      PASS
base file rewritten                                    false
composite restore authority                            false
tombstone removed base exact lookup                     PASS
tombstone removed base decode                           PASS
tombstone removed candidate from lattice                PASS
manifest delta/tombstone counters                       PASS
failed proof receipt rejected                            PASS
manifest generation after delta + tombstone                3
composite benchmark route                               PASS
old snapshot readable during blocked replacement load   1,000 / 1,000
new snapshot visible after ownership flip               PASS
reload during initial warming rejected                  PASS
parallel tombstone admissions retained                  2 / 2
generation after two concurrent admissions              3 -> 5
```

Exact tests:

```text
nanda_wave::lexical_grokking::tests::
append_only_manifest_adds_centers_and_applies_tombstones_without_rewriting_base

lay-l11-serve::
replacement_load_does_not_block_current_snapshot_readers
reload_during_initial_warmup_cannot_race_background_owner

nanda_wave::lexical_grokking::composite::tests::
manifest_writer_lock_serializes_independent_transactions
```

The following has not yet been proved:

```text
full fixed 13 x 20,000 quality proof with a real admitted delta
cross-package pairwise phase interference between base and delta centers
delta compaction into a new immutable V8 base
authority promotion of any composite snapshot
```

Implementation ownership:

```text
/home/ubu/projects/lay/src/nanda_wave/lexical_grokking/composite.rs
/home/ubu/projects/lay/src/nanda_wave/lexical_grokking/runtime.rs
/home/ubu/projects/lay/src/bin/lay_l1_1_serve.rs
/home/ubu/projects/lay/src/bin/lay_nanda_wave_train.rs
```

Operator route:

```bash
lay-nanda-wave-train \
  --init-l11-composite \
  --manifest /absolute/path/l11.runtime.json \
  --base /absolute/path/package.v8.bin

lay-nanda-wave-train \
  --admit-l11-delta \
  --manifest /absolute/path/l11.runtime.json \
  --delta /absolute/path/proved-delta.bin \
  --proof-receipt /absolute/path/delta-proof.json \
  --scope local-lexical-addition

lay-nanda-wave-train \
  --admit-l11-tombstone \
  --manifest /absolute/path/l11.runtime.json \
  --surface obsolete-surface \
  --proof-receipt /absolute/path/tombstone-proof.json \
  --scope local-lexical-removal

lay-l11-serve reload \
  --memory /absolute/path/l11.runtime.json \
  --socket "$XDG_RUNTIME_DIR/lay-l11.sock"
```

`--bench-l1-lexical-grokking` accepts either a single package or the composite
manifest. Single-package measurements retain the original direct readout path,
so historical V8 p50/p99 remains comparable. Composite measurements report
generation, delta count and tombstone count and exercise the merged lattice.

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

An aggregate percentage cannot hide a failing class. The historical fixed 10k
v3 baseline passed this working gate in 4 of 13 classes.

### Final V8 working-gate proof, 2026-07-29

The fixed `20,000 x 13` proof was repeated after two runtime corrections:

```text
DecoderGraph anchor reconstruction per candidate
-> one compact offsets + AtomId anchor table built during load

raw layout surface
-> layout projection before punctuation normalization
-> exact projected candidate enters the geometry shell
-> bounded top-64 preserves geometry-shell evidence
```

The compact anchor table retains every center sequence. It changes no package
bytes and removed repeated DecoderGraph traversal and allocation from the hot
path. Raw projection is required because the physical `,` and `.` keys encode
Russian `б` and `ю`; trimming those keys before projection destroyed valid
layout evidence.

Fixed proof:

| Damage class | Unique top-1 | Lattice |
|---|---:|---:|
| adjacent transposition | 99.964% | 99.995% |
| double substitution | 97.852% | 100.000% |
| extra letter | 98.887% | 100.000% |
| layout projection | 98.306% | 99.905% |
| letter substitution | 99.956% | 99.995% |
| missing letter | 99.262% | 99.850% |
| non-adjacent transposition | 97.195% | 100.000% |
| omission + transposition | 97.032% | 99.550% |
| prefix truncation | 98.318% | 99.990% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.813% | 100.000% |
| sparse multi-omission | 96.628% | 99.705% |
| suffix truncation | 99.058% | 100.000% |

```text
cases                                      260,000
working unique top-1 gate                  PASS 13/13
lattice >= 99% gate                        PASS 13/13
clean preservation                         100.000%
false authority                            0
false singleton                            0
heldout top-1 / top-8 / top-64             86.587 / 96.494 / 99.922%
proof wall                                 13m 07.13s
heldout evaluation                         694.498s
average CPU                                1601%
peak RSS                                   1,709,668 KiB
swap                                       0
package                                    184,826,869 B = 176.27 MiB
package SHA-256                            05ff08798cb1ff04db4ce9e233941732f3f859e9db0c69b6b34a42beebdd37d7
runtime authority changed                  false
```

Standalone final runtime:

```text
cold startup                               0.64s
cold RSS                                   158,344 KiB
hot surface                                вреям
hot iterations                             2,000
hot p50 / p90 / p99                        2.160 / 2.188 / 2.294 ms
hot max                                    3.276 ms
hot RSS                                    169,168 KiB
```

The repeated-surface hot gate passes. The proof's diverse first-touch
restoration p99 is still `24.365 ms`; this is a separate stricter target and
must not be reported as the hot repeated-surface result.

Append-only overlay scaling:

| Deltas | Manifest B | Cold | Cold RSS KiB | Hot RSS KiB | p50 ms | p99 ms |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 186 | 0.71s | 158,580 | 169,380 | 2.149 | 2.262 |
| 1 | 535 | 0.72s | 158,140 | 169,300 | 2.853 | 3.414 |
| 2 | 882 | 0.73s | 158,256 | 169,624 | 2.909 | 3.376 |
| 4 | 1,576 | 0.74s | 158,800 | 169,724 | 3.016 | 3.469 |
| 8 | 2,964 | 0.74s | 158,648 | 169,980 | 3.202 | 3.718 |

Each measured delta is a proved `56,600 B` package. No base relation was
removed or rewritten.

Full-V8 shadow service reload under concurrent Unix-socket traffic:

```text
workers                                     8 request producers / 20 service workers
lattice requests                            800
successful lattice responses                800
errors                                      0
reload wall                                 0.85s
post-reload service state                   ready
live daemon or IBus touched                 false
```

Exact evidence:

```text
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/proof-full-anchor-layout/report.json
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/proof-full-anchor-layout/run.log
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/runtime-final/hot-vreyam-2000.json
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/runtime-final/hot-vreyam-2000.time
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/overlay-scale/
/home/e/build/lay-l1-shadow/artifacts/l11-v8-shard32-2026-07-29/service-reload-v2/
```

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

### Canonical V8 shard-1 shadow runtime, 2026-07-29

The shard-32 package above remains valid historical evidence. A second runtime
experiment changed only the lossless V8 posting representation and readout
implementation. The learned field, terminal IDs, couplings, calibration and
fixed heldout set did not change.

Canonical package and runtime parameters:

```text
WordCenter                               835,410
AtomId                                   218,387
forward relations                       108,156,559, retained losslessly
reverse relations                        86,120,980
atoms per compressed shard                        1
compressed posting shards                  218,387
package bytes                           198,233,790 = 189.05 MiB
package SHA-256                         03e320857cafc7d661a1893f728f8777c5e0b755ee01214502548779fbee6494

runtime workers                                  14 maximum
service request workers                           4 default
posting cache                                    32 MiB
decoded shard cache                               0 MiB
reverse cache                                    16 MiB
```

The runtime executes one posting-cache lookup batch, decodes only misses in the
persistent Rayon pool, and performs one deterministic admission batch. Reverse
reconstruction and exact frontier activation remain fused in one parallel
stage. Splitting those two operations was measured and rejected.

#### Fixed proof

```text
corpus                                   RU 535,410 + EN 300,000
fixed heldout                            13 x 20,000 = 260,000
proof workers                            20
wall                                     6m 44.69s
average CPU                              1323%
peak proof RSS                           1,136,644 KiB
swap                                     0
clean preservation                       100.000%
false authority                          0
false singleton                          0
working unique top-1 gate                PASS 13/13
lattice >=99% gate                       PASS 13/13
```

| Damage class | Unique top-1 | Lattice |
|---|---:|---:|
| adjacent transposition | 99.964% | 99.995% |
| double substitution | 97.852% | 100.000% |
| extra letter | 98.887% | 100.000% |
| layout projection | 98.306% | 99.905% |
| letter substitution | 99.956% | 99.995% |
| missing letter | 99.262% | 99.850% |
| non-adjacent transposition | 97.195% | 100.000% |
| omission + transposition | 97.032% | 99.550% |
| prefix truncation | 98.318% | 99.990% |
| punctuation suffix | 100.000% | 100.000% |
| repeated fragment | 98.813% | 100.000% |
| sparse multi-omission | 96.628% | 99.705% |
| suffix truncation | 99.058% | 100.000% |

Latency from that same full proof:

```text
cold load                                622 ms
readout p50 / p99 / max                  3.077 / 4.547 / 4.986 ms
readout gate <=5 ms                      PASS
readout + restoration p50 / p99 / max    3.369 / 6.265 / 7.624 ms
restoration authority                    WATCH
```

The distinction is mandatory. The bounded L1.1 lattice/readout passes its live
shadow runtime gate. The authority classifier still abstains on most cases and
its complete restoration path exceeds 5 ms at p99. Therefore the V8 package may
replace the heavy V7 package as the shadow lattice source, but it may not become
model authority.

#### Shadow service

The service used four external request workers because each worker owns a dense
scratch array over all terminals. Twenty external workers allocated about
`20 x 17 MiB` of scratch and raised RSS to about `515 MiB`. Four workers share
the 14-thread internal runtime pool and produced:

```text
cold-ready RSS                           125,932 KiB
800 concurrent restore requests          800 success, 0 errors
wall                                     825 ms
throughput                               about 969 requests/s
steady RSS                               214,484 KiB

reload with concurrent requests           4,000 success, 0 errors
reload wall                               0.65s
post-reload RSS                           211,532 KiB
```

Measured rejected variants:

```text
fill-only caches                         p99 8.4-14.4 ms; rejected
CPU-pinned runtime workers               p99 10-11 ms; rejected
separate reverse decode/activation       median readout p99 6.606 ms; rejected
```

Not tested by this experiment:

```text
live IME model authority
full proof with an admitted append-only delta
cross-package pairwise interference
delta compaction into a replacement V8 base
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_V8_SHARD1_BATCH_RUNTIME_2026-07-29.json
```

### Global L1.1 + L2 seed compaction, 2026-07-30

The `17,172` Russian lemmas that had no L1.1 seed were exported by the generic
L2 compiler and appended to the existing bilingual corpus. No damaged spelling
or word-specific correction rule was added. A separate delta was tested first,
but cross-package rank merging remained `WATCH`; the accepted representation is
one deterministic V8 base.

```text
base RU surfaces                         535,410
added RU L2 lemma seeds                   17,172
EN surfaces                              300,000
WordCenter total                         852,582
AtomId total                             218,763
forward relations                    110,928,005
reverse relations                     87,972,901
V8 shard width                                  32 atoms
package                              190,139,182 B = 181.33 MiB
package budget headroom                14,333,138 B
package SHA-256  47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9
raw corpus stored in package                    no
training damaged surfaces                         0
```

The full fixed proof used `20` workers and all `260,000` heldout cases. It took
`17m 07.94s`, averaged `1205%` CPU, peaked at `1,418,204 KiB` RSS and used no
swap.

| Damage class | Unique top-1 | Lattice | False certainty |
|---|---:|---:|---:|
| adjacent transposition | 99.958% | 99.995% | 0 |
| double substitution | 97.889% | 100.000% | 0 |
| extra letter | 98.883% | 100.000% | 0 |
| layout projection | 98.323% | 99.905% | 0 |
| letter substitution | 99.956% | 99.995% | 0 |
| missing letter | 99.257% | 99.850% | 0 |
| non-adjacent transposition | 97.248% | 100.000% | 0 |
| omission + transposition | 97.028% | 99.535% | 0 |
| prefix truncation | 98.266% | 99.990% | 0 |
| punctuation suffix | 100.000% | 100.000% | 0 |
| repeated fragment | 98.808% | 100.000% | 0 |
| sparse multi-omission | 96.694% | 99.670% | 0 |
| suffix truncation | 99.142% | 100.000% | 0 |

```text
unique top-1 >95%                         PASS 13/13
lattice >=99%                             PASS 13/13
clean preservation                        100.000%
false authority                                   0
false singleton                                   0
aggregate top-1                              86.790%
aggregate top-8                              96.560%
aggregate top-64                             99.918%
verdict                                  PASS_shadow
```

#### Grouped cache runtime

The learned package and all scores remained byte-identical. Runtime cache
misses are now grouped by shard, and cached reverse arcs are collected under
one mutex acquisition. The 2k quality projection before and after this change
has the same SHA-256:

```text
79f979fae94806c08fc746372557df03846633ad4c91f028f38cbd6f37ff5cb4
```

Repeated hot benchmark, same package and same surface:

| Runtime | p50 | p90 | p99 | max | RSS |
|---|---:|---:|---:|---:|---:|
| before | 1.876 ms | 2.934 ms | 13.987 ms | 45.017 ms | 177,404 KiB |
| grouped cache | 1.960 ms | 2.670 ms | 3.514 ms | 4.655 ms | 176,208 KiB |

The dedicated hot gate is therefore `PASS`. The diverse first-touch path from
the full quality proof remains slower (`p99 12.314 ms`) and is recorded
separately; it is not represented as hot latency.

Tested:

```text
full fixed quality proof                  yes
all 13 class denominators                 20,000 each
quality parity after runtime-only change  exact
hot repeated readout                      10,000 iterations
L2 seed reachability                      all 93,672 lemmas
```

Not tested by this checkpoint:

```text
L1.1 direct edit authority
an admitted non-empty L1.1 delta in the full 260k proof
long-running cache behavior over multiple days
```

Runtime authority did not change. Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_GLOBAL_COMPACTION_852582_2026-07-30.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L1_L11_V8_GROUPED_CACHE_RUNTIME_2026-07-30.json
```
