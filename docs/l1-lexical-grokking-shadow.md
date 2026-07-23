# L1 Lexical Grokking Shadow Runtime

Status: L1.0 V55 remains `WATCH_shadow` as a forced top-1 selector. L1.1 E2 is
`PASS_shadow` as a typed `Winner | Tied | ABSTAIN` signal restorer. It is not
installed into daemon or IME.

The normative current configuration, proof and artifact hashes are recorded in
`docs/l1-crystal-kernel-memory-layout.md`, section 9. L1.1 preserves the V55
candidate lattice and adds bounded positive, anti and hard-negative subcenters
plus explicit tied-basin readout.

The scientific and scaling contract for the next kernel is recorded in
`docs/l1-crystal-kernel-scientific-foundations.md`. Current fixed caps remain
implementation facts, not accepted 600k architecture.

## Signal Tree

```text
reference word library
-> clean terminal IDs + proof-only damaged surfaces
-> typed byte/character/keyboard/boundary n-grams
-> reversible NGramGraph -> dense AtomId
-> learned AtomWaveCode
-> robust forward Atom->Word couplings from damaged surfaces
-> clean backward Word->Atom reconstruction couplings
-> ordered character-anchor reconstruction wave for missing material
-> bounded WordCenter frontier
-> surface-lane / keyboard-lane structural interference
-> candidate-specific anti-wave
-> three settling iterations
-> terminal ID lattice
-> DecoderGraph -> UTF-8 only after selection
```

The package stores no raw corpus, source vocabulary strings or exact damage
episodes. The decoder graph only materializes the already selected center.

## Runtime Boundary

The shadow runtime returns candidates and evidence. It cannot mutate text and
is not connected to `AuthorizedEdit`. L2/L3/L4, Bayes and context do not
participate in the L1 proof.

## Current Numbers

```text
scale   clean    top-1   top-8   top-64  sparse   package
2k      100.00%  97.02%  99.93%  99.98%  95.91%   12.36 MB
10k      98.98%  91.82%  99.15%  99.63%  90.41%   42.20 MB
```

The 2k sparse multi-omission blocker is closed under the accepted 95% working
threshold. The 10k package remains blocked because both aggregate top-1 and
sparse multi-omission are below 95%, while clean preservation is below the
formal 99.9% gate.

The working gate is `unique top-1 > 95.0%` independently for every damage
class. The fixed 10k v3 baseline currently passes 4 of 13 classes. Aggregate
top-1 cannot compensate for any failing class.

The isolated v26 complete-postings proof improves all 13 classes and moves the
working scoreboard to 9/13. Aggregate top-1 is 96.293%, clean preservation is
100%, top-64 is 99.993%, package size is 35.07 MB and hot p99 is 4.182 ms. Four
classes remain below gate: double substitution 86.777%, non-adjacent
transposition 91.738%, omission plus transposition 92.566%, and repeated
fragment 93.737%. Runtime authority remains unchanged.

V27 generalized reverse character-anchor interference. It raised aggregate
top-1 to 96.509%, produced 789 improvements against 103 worsenings, and raised
repeated fragment to 97.970%. It is still rejected because sparse
multi-omission regressed from 95.037% to 92.824%; the passing-class count stayed
9/13. Receipt: `docs/structural_gates/receipts/L1_SEQUENCE_INTERFERENCE_V27_10K_2026-07-23.json`.

## Commands

```bash
lay-nanda-wave-train \
  --prove-l1-lexical-grokking CORPUS \
  --max-words 2000 \
  --out PACKAGE

lay-nanda-wave-train \
  --query-l1-lexical-grokking PACKAGE \
  --surface врмея \
  --limit 8

lay-nanda-wave-train \
  --bench-l1-lexical-grokking PACKAGE \
  --surface переподлчаю \
  --iterations 2000

lay-l1.1-restore \
  --memory data/lexical_grokking/l1_l11_multimodal_restoration_10k.bin \
  врмея
```

The query output contains `terminal_id`, decoded `surface`, forward/backward
coherence, surface/keyboard structural hits, positive phase, anti phase and
settled energy.

## Accepted Architectural Changes

- `WordCenter64` is exactly 64 bytes with 22 compact phase components.
- NGramGraph owns atom identity; no `HashMap<String, Word>` is runtime authority.
- Forward memory learns damage tolerance; backward memory reconstructs clean form.
- Surface and keyboard lanes interfere independently before their strongest
  coherent lane contributes to the lattice.
- Anti compilation uses 20 target shards and a bounded top-4 merge.
- Full/NoPhase/NoAnti proof evaluation uses all available workers.
- Objective ambiguity is compiled from clean, training and heldout surfaces.
- Character anchors are reverse-only couplings: they cannot birth candidates,
  alter positive centers or leak into anti-center compilation.
- Omission reconstruction is a bounded ordered-subsequence interference lane;
  it is disabled for equal-length layout and transposition candidates.
- Physical-key geometry uses the observed `KeyEvent` sequence for layout
  projection and preserves punctuation keys across script projection.
- Deletion-aware reconstruction can recover two ordered omissions or one
  omission plus one necessary adjacent transposition into the tied basin.
- Reconstruction is bounded to the strongest eight scalar candidates, cannot
  alter V55 ranking and never uses proof target labels at runtime.

## Remaining Gate

Do not install the package into live typing yet. The next quality work must
improve 10k winner crystallization without reducing clean preservation, top-64
coverage or layout projection. A zero-copy mapped runtime and genuinely
incremental surface accumulator also remain unproven runtime optimizations.
