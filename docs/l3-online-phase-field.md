# L3 Online Context Phase Field

This document is the canonical learning contract for the Lay L3 context field.
It replaces the old multipass corpus compiler. It does not change text-edit
authority: L3 can emit only `Support`, `Suppress`, `Neutral`, or `Unavailable`.

## Signal Route

```text
bounded token stream
-> cold learned surface-transition field
-> hash token IDs immediately
-> stable L2 semantic anchors
-> candidate relation wave + L2-state signature scene wave
-> independent positive / negative / hard-negative banks
-> learned pairwise L2 lattice field
-> bounded subcenter split
-> deterministic ContextPhasePackage snapshot
-> fixed heldout proof
-> publish on PASS or keep the installed package on WATCH
```

The implementation owners are:

```text
stream reader             src/nanda_wave/context_phase/stream.rs
online learner            src/nanda_wave/context_phase/online.rs
package and hot readout   src/nanda_wave/context_phase/mod.rs
deterministic format      src/nanda_wave/context_phase/format.rs
heldout proof             src/nanda_wave/context_phase/proof.rs
CLI orchestration         src/nanda_wave/context_phase/compiler.rs
surface evidence field    src/nanda_wave/context_phase/surface_field.rs
```

## Learned Surface-Transition Field

L3 must observe the same type of damaged surface that reaches L2. The former
compiler invented deletions at fixed character positions and one middle swap.
That was a synthetic rule-shaped source and could diverge from live typing.
It has been removed from the L3 train/proof route.

The cold compiler now reads verified `from -> to` correction receipts and
reduces each eligible receipt to a compact mutation geometry:

```text
single typed token + single corrected token
-> align characters
-> mutation direction
-> relative position phase bucket
-> length bucket
-> repeated mode support
-> bounded SurfaceMutationField
```

The field currently learns only geometry that can be reconstructed without
retaining a character or a word: a missing typed character and an adjacent
transposition. Layout projection remains a separate L2 operator because its
keyboard mapping is already represented by L2. Multiword receipts remain a
typed boundary-transition concern and are excluded here.

At compile and heldout time the field produces bounded damaged surfaces from a
clean teacher target. L2 independently produces the candidate lattice from
each surface. The target never ranks that lattice. L3 therefore learns from
the same candidate competition as before, but with evidence-shaped damage
rather than fixed positions.

```text
cold corrections.jsonl
-> SurfaceMutationField (compact modes only)
-> clean corpus target
-> observed damage geometry
-> real L2 lattice
-> unary / pairwise positive and anti phase field
-> heldout proof
```

The correction JSONL is never serialized into `ContextPhasePackage`; runtime
contains only the previously existing hashes and phase centers. L3's runtime
role remains contextual selection among candidates already born by L2.

The CLI requires this source explicitly for all new L3 compiles:

```text
lay-nanda-wave-train --build-and-prove-l3-context-phase CORPUS \
  --surface-evidence corrections.jsonl --out PACKAGE
```

The first 100k-fragment shadow on 2026-07-21 admitted 68 eligible receipts
into 10 repeated modes at minimum support two. It reached 11.43% global
support coverage, 98.74% support precision, 63 pairwise improvements and zero
pairwise regressions, but still had 53 false top-1 cases. Its verdict was
`WATCH`; no runtime package was published. Lowering mode support to one
admitted 23 modes and raised false top-1 to 118, so that variant was rejected.

## Online State

The learner keeps no corpus sequences and no raw corpus strings. A raw token is
present only inside the current bounded fragment or the current L2 probe batch.
Persistent learning state is keyed by token hashes and contains phase cells,
support counters, bounded reservoirs, and deterministic admission sketches.

```text
OnlineContextPhaseLearner
|-- semantic anchors             max 32,768 states
|-- candidate profiles           max 16,384 profiles
|-- positive centers             max 65,536
|-- generic negative centers     max 24,576
|-- hard-negative centers        max 8,192
|-- exact pair profiles          max 65,536, hashes only
|-- generalized L2-state pairs    max 16,384, hashes only
|   |-- ordinary direction banks max 8 centers each
|   `-- hard counter banks       max 4 centers each
|-- pending negative profiles    max 16,384, no support authority
|-- competition calibration      max 2,048 cases
`-- L2 probe batch               max 16 source fragments
```

The three center capacities are physically independent. A negative wave cannot
consume positive capacity, and broad negative evidence cannot consume the
candidate-specific hard-negative budget.

### Two-Surface Profile Admission

A token seen once remains only in the compact frequency and semantic fields.
It does not allocate a full candidate profile. On a second independent
surface, the learner admits a bounded profile; that profile still needs two
coherent phase scenes before it can appear in a runtime snapshot. This makes
the stored profile a settled relation rather than a one-off corpus token.

```text
first surface
-> frequency / semantic anchor only
-> no candidate-profile authority

second surface
-> bounded profile admission

second coherent profile scene
-> snapshot-eligible unary phase evidence
```

Frequency is strictly an admission witness. It never contributes to candidate
ranking, cannot create `Support`, and cannot override phase competition.

## Signature Transfer Field (V4)

Exact word hashes are useful for candidate authority but cannot by themselves
transfer a learned context relation to another lexical form. V4 adds a bounded
profile keyed by `candidate_l2_signature()`: a lossy projection of L2 center
coverage, motif support, residual pressure and compact L1 surface spectrum.
The package stores the signature hash and phase centers only, never a word.

```text
L2 candidate center signature + context-only scene wave
-> signature phase profile
-> may strengthen an already settled exact lexical profile
-> cannot create Support by itself
```

The signature center receives a context-only vector. Candidate identity is
already present in the signature key, so feeding a candidate-rotated vector
would secretly turn transfer memory back into an exact-word table. Runtime
requires both the lexical center and signature center to have support at least
two before the signature can affect the positive phase. `NoSignatureProfile`
is a mandatory heldout ablation.

The first 10k cold proof after this correction was neutral: full and
NoSignature each had 149 correct top-1 and 12 false top-1. This is a safe
`WATCH`, not a promotion: V4 must reduce false top-1 with zero regressions on
the larger heldout corpus before it can be published.

## Pairwise Lattice Field (V3)

Unary centers answer only: "where has this candidate appeared before?" They
cannot preserve the relation that one real L2 candidate must beat another in a
specific scene. V3 stores that relation as a compact directed phase profile:

```text
canonical scene wave
+ PairKey(low_hash, high_hash)
-> low_wins / high_wins
-> hard_low_wins / hard_high_wins
-> directed dominance edge
```

The scene vector contains context only. Candidate identities occur once, in
the canonical pair key, so training and runtime cannot double-encode a word
into the scene. The same `canonical_scene_wave()` function is used by the
online learner and hot package readout.

At runtime, bounded pairwise competition considers at most eight unique L2
candidates, chosen by unary margin and stable hash tie-break. A proven losing
candidate is excluded before unary admission. `Conflict` and directed cycles
remain Neutral. `Tie` remains unknown competition and does not silently veto
both candidates. A hard pair center becomes destructive only after the same
false-winner phase mode repeats; it can suppress but never create `Support`:
unary readiness remains mandatory.

The serialized v4 package keeps V1-V3 compatibility and adds a bounded
signature-profile count after the V3 header. Older packages load with an empty
signature field. Decoder validation rejects unsorted keys, duplicate keys,
oversized banks and trailing bytes.

When an exact `PairKey` is unknown, V3 may consult a generalized relation key
derived from the existing L2 surface-state readout: center coverage, motif
support and residual pressure. This key contains no word text. It is weaker
than an exact pair and can only suppress a contender; it never manufactures
unary `Support`. A generalized direction is serialized only after it has
settled on three independent exact-pair surfaces. That is the minimum circuit
formation gate: before transfer is demonstrated, its readout is `Unknown`.

## Stable Semantic Coordinates

A continuously rotating semantic state is invalid for one-pass learning: old
profile vectors would be compared with a different final semantic phase.
Therefore the first two observations establish one immutable semantic anchor.
Later observations raise its support but do not rotate it. Incompatible phrase
modes belong in bounded candidate subcenters, not in a drifting coordinate
system.

Surface and semantic channels are additive:

```text
surface relation weight          1.00
context semantic relation        0.80 after anchor support >= 2
candidate semantic relation      0.85 after anchor support >= 2
```

The surface channel is never erased by token frequency.

## Positive And Destructive Learning

For every observed target transition:

1. Measure the target margin before the teacher update.
2. Reinforce a coherent positive center or split a new bounded mode.
3. Probe the actual L2 lattice at powers of two and at every new context mode.
4. Write every real L2 competitor into the directed pairwise competition
   field. It describes `target > competitor` for this scene, not a global
   defect of the competitor.
5. If a competitor beat the pre-update target field, write a hard counter-wave
   into that same `PairKey(target, competitor)`. The clean corpus target is a
   teacher label only; it never reorders the L2 lattice.

Generic L2 competition is never unary destructive authority: a candidate may
lose one scene and be correct in another. Pairwise phase memory holds that
relation with both candidates present. Corpus false-winner evidence is also
pairwise because its target is known. The unary hard-negative bank is reserved
for a future causal feedback receipt: rejected candidate plus observed final
target, or an explicit undo. A dismissed IME candidate alone is censored: it
does not say which candidate should have won. This prevents a nearby valid
completion from being erased by a generic reject event.

Negative evidence may arrive before a candidate has positive support. It is
kept in `pending_negative` and transferred when a positive profile is later
admitted. Pending negative state is never serialized as a runtime profile and
cannot manufacture `Support`.

Repeated coherent evidence reinforces one center. An incompatible relation
creates a new subcenter. A full bank rejects the new mode instead of averaging
it into an existing center.

## Determinism And Snapshots

- L2 probes execute in parallel but receipts are applied in source order.
- Hash-map state is sorted before serialization.
- Reservoir replacement and admission use stable hash tie-breaks.
- Snapshots are written to a private temporary file and atomically renamed.
- Package reload reconstructs each center sum from `center * support`.
- Progress reports fragments, rate, ETA, profiles, separate center banks,
  calibration cases, estimated learner bytes, and RSS.

## Incremental Runtime Memory

Small L3 changes must not recompile or rewrite the immutable corpus package.
The canonical mutable contour is:

```text
immutable base.nwpc
+ delta-000001.nwpc
+ delta-000002.nwpc
+ ordered runtime manifest
-> deterministic composite memory
-> one unchanged ContextPhasePackage scoring implementation
```

Ownership:

```text
delta compiler and targeted proof   src/nanda_wave/mod.rs
manifest and composite loader       src/nanda_wave/context_phase/composite.rs
swappable runtime owner             src/nanda_wave/context_phase/mod.rs
CLI                                 src/bin/lay_nanda_wave_train.rs
```

An ordinary update performs only these operations:

```text
new independently supported scenes
-> read the manifest and base package to inherit its signature schema
-> compile one small delta without replaying the base corpus
-> targeted replay of changed scenes and fixed safety sentinels
-> PASS receipt with false_supports = 0
-> atomic append to manifest
-> atomic in-process memory swap or daemon reload
```

The delta compiler opens the installed composite only to inherit its exact
signature schema and calibration contract. It does not replay the base corpus
and does not rewrite the base package. Admission also leaves the base
unchanged. Runtime loads the manifest, validates every delta's schema and exact
byte size, and merges the bounded banks once at load time. Hot scoring still
uses one implementation, so base-only and composite runtime cannot drift into
different phase mathematics.

Commands:

```bash
lay-nanda-wave-train --init-l3-context-composite \
  --base base.nwpc --manifest manifest.json

lay-nanda-wave-train --compile-l3-context-delta scenes.txt \
  --surface-evidence corrections.jsonl \
  --out delta-000001.nwpc

lay-nanda-wave-train --prove-l3-context-delta \
  --manifest manifest.json \
  --delta delta-000001.nwpc \
  --cases targeted-cases.tsv \
  --out-receipt delta-000001.proof.json

lay-nanda-wave-train --admit-l3-context-delta \
  --manifest manifest.json \
  --delta delta-000001.nwpc \
  --proof-receipt delta-000001.proof.json \
  --scope local
```

Targeted case rows are:

```text
improve<TAB>sentence context<TAB>candidate-a|candidate-b<TAB>expected
safety<TAB>sentence context<TAB>candidate-a|candidate-b<TAB>allowed-or-
```

One live observation is not immediate global authority. The delta compiler
keeps the existing independent-support gates, and admission additionally
requires a matching targeted PASS receipt. Local deltas remain separately
identified by scope.

The immutable base owns global, competition, pairwise, and existing-profile
calibration. A delta contributes phase evidence but cannot replace those
energy scales. New delta-only profiles start at the base global threshold.
This prevents a narrow shard from globally recalibrating the broad corpus.

Delta pair centers remain independent runtime subcenters. They are not forced
back through the cold package's 16-center bank, because a full base bank would
silently discard a newly learned directional mode. Composite runtime permits
up to 64 centers per pair direction; reaching that bound is a compaction
signal, not permission to overwrite the immutable base during ordinary
learning.

Compaction is not part of ordinary learning. It is recommended only when:

```text
delta_count >= 32
or total_delta_bytes >= 16 MiB
```

It writes a new base path first and flips the manifest afterward. It refuses
to overwrite the current immutable base in place.

### Incremental Contour Checkpoint: 2026-07-28

Tested:

```text
library and lay-nanda-wave-train compile check    PASS
base package rewritten during admission           no, unit contract
delta schema and byte-size validation             implemented
matching targeted PASS receipt required           implemented
runtime memory swap                               implemented
full base corpus recompiled                       no
```

Not tested in this checkpoint:

```text
L3 quality on the 80k fixed heldout corpus
daemon live reload wiring
```

Verdict scope: implementation checkpoint only. Runtime text-edit authority did
not change, and no delta was promoted. The quality gate remains separate and
must not be inferred from package/manifest parity.

### Persistent Online Worker Checkpoint: 2026-07-28

The online owner is a separate low-priority user service:

```text
double-Shift visible undo succeeds
-> record complete accepted contextual transition
-> append word_usage_events.jsonl
-> lay-l3-online.service polls appended bytes only
-> require exactly one changed tail token
-> require 2 independent scenes for rejected -> expected
-> one-pass targeted delta compile
-> changed-scene proof + 5 safety sentinels
-> PASS and false_supports=0: atomically append manifest
-> WATCH: retain bounded pending evidence, leave runtime unchanged
```

Ownership and bounds:

```text
worker                         src/bin/lay_nanda_wave_train/l3_online.rs
double-Shift receipt           src/bin/lay_ibus_engine/committed_tail.rs
service                        systemd/lay-l3-online.service
poll interval                  5,000 ms
minimum independent scenes     2
maximum scenes per relation    8
maximum pending relations      128
retry scene counts             2, 4, 8
service Nice                   10
service CPUWeight              20
service IOWeight               20
```

Persistent runtime paths:

```text
~/.local/share/lay/nanda_wave/word_usage_events.jsonl
~/.local/share/lay/nanda_wave/l3-online/state.json
~/.local/share/lay/nanda_wave/l3-online/delta-*.nwpc
~/.local/share/lay/nanda_wave/l3_context_phase.runtime.json
```

Measured isolated schema-1 smoke:

```text
installed immutable base bytes             12,939,828
installed base SHA-256              d7ca1280e41424c058f5395f9da871d193429d536fa1f4bc146dfb38306326ba
installed signature schema                          1
historical events replayed                      false
independent scenes                                  4
corpus passes                                       1
delta bytes                                      2,912
wall                                             5.83 s
peak RSS                                      148,052 KiB
worker CPU during small compile                    99%
false supports                                       0
verdict                                          WATCH
manifest byte identity before/after                true
admitted deltas                                       0
base rewritten                                    false
```

The first two-scene attempt was also `WATCH`: one pass, `1,488 B` delta,
schema 1, zero false supports, and byte-identical manifest. This is expected
safe behavior, not a quality PASS. The pending relation remains durable and is
retried only when its independent scene count reaches the next power of two.

Tested:

```text
trainer build                                             PASS
worker acceptance and bounded targeted-case tests          3/3
legacy schema inheritance                                  PASS
double-Shift full-context feedback                         PASS
WATCH admission gate                                       PASS
schema-1 end-to-end isolated smoke                         PASS
one-pass compile                                            yes
immutable base rewritten                                    no
runtime authority changed                                   no
```

Not tested in this checkpoint:

```text
automatic PASS admission from real user traffic
long-running service stability over multiple days
80k fixed heldout quality after a promoted delta
in-process reload latency after PASS
```

Exact receipt:

```text
docs/structural_gates/receipts/L3_ONLINE_SCHEMA1_WORKER_2026-07-28.json
```

This mismatch was resolved on 2026-07-30. The schema-4 artifact and manifest now
both declare `30,698,796 B` and SHA-256
`a71d58a0a01f9c5f8fae4328e1e5011043f3e95ac1d5ee760a0dc56b81cd9ad7`.
The manifest carries the exact 80k heldout `PASS` evidence from the v20 build.
The installer still verifies size, SHA-256 and heldout verdict before creating
any runtime temporary file.

### Payment Relation Delta Experiment: 2026-07-28

Remote host: `e@192.168.3.94`, 20 hardware threads. Immutable base:
`l3_context_phase_v1.nwpc`. The delta corpus contained 64 payment relation
scenes with varied entities and no `Apple` training row.

Measured compile:

```text
corpus fragments                         64
corpus passes                             1
L2 probe workers                         20
wall                                  0.14 s
artifact                            68,516 B
peak RSS                            80,504 KiB
base loaded                            false
base rewritten                         false
full 80k corpus replayed                false
```

Targeted proof used one requested payment scene and five safety sentinels:

```text
target: ... оплатить Apple b -> и
safety: wave / a / GitHub / compiler ... / Apple
false supports                              0
target unary margin before             100,893
target unary margin after              401,151
target threshold                        87,277
target competition margin              294,782
pair high candidate                          и
pair high score                        401,151
pair threshold                         412,246
pair local / bank support               5 / 29
pair outcome                               Tie
target disposition                     Neutral
verdict                                  WATCH
```

The experiment proved that incremental compilation is operational and cheap,
but it did not prove the requested contextual correction. The shard was not
admitted. A direct admission attempt rejected the WATCH receipt with
`InvalidData`; the runtime manifest remained at zero deltas.

The first implementation defect found by this experiment was fixed: a narrow
delta had been able to replace base calibration during generic merge. Base now
owns all global energy scales. The second defect was also fixed: a full
16-center cold pair bank had discarded new delta modes. Composite runtime now
retains delta pair subcenters independently. Neither fix changed text-edit
authority.

Remaining measured limitation: the generalized relation scene still contains
strong exact-token energy. Across unseen entity names, the payment direction
improves unary evidence but does not yet cross the existing pairwise
directional certificate. Relaxing that certificate would trade away tied-basin
safety and is not accepted by this experiment.

## Proof And Publication

### Pairwise Full-Winner Certificate

Pairwise evidence may promote an existing unary profile only when one candidate
has a complete winner certificate across the bounded L2 lattice: it wins every
known edge and has no loss, tie, conflict, or unknown edge. This is still L3
ranking evidence, never direct Apply authority.

The heldout proof reports certificate supports separately from ordinary unary
supports, including correct and false certificate counts. A certificate path is
eligible for publication only when false certificates remain zero and the
candidate-order permutation check remains exact.

The corpus partition is fixed by source ordinal: four fragments train and the
fifth is held out. The second stream pass does not update the package. Every
heldout transition uses the same L2 candidate lattice for:

```text
Full
NoPhase
NoAnti
NoSemanticState
NoPairwise
NoHardPairwise
ShuffledPairDirection
ShuffledPairScene
MagnitudeOnlyPairwise
```

Publication requires all of the following:

```text
context evidence > 0
full support > 0
phase improved > phase worsened
anti improved > anti worsened
false support and false top-1 do not grow
full false top-1 = 0
support coverage >= 10%
candidate permutation mismatches = 0
pairwise worsened cases = 0
L2 lattice unchanged
L3 apply authority = false
```

A `WATCH` report never replaces the installed package.

### Fixed Coverage Denominator

Heldout reports expose two denominators. `lattice_transitions` is every
heldout position for which L2 produced a real competing lattice; it is fixed
for all package variants over the same corpus. `target_profile_missing` is the
part of that fixed set for which the package had no target profile.

`global_support_coverage_ppm` uses the fixed lattice denominator and is the
promotion metric. The older `support_coverage_ppm` remains an internal
conditional diagnostic over profile-present transitions only. It must never be
used to compare package variants or justify publication.

## Local Outcome Corpus

Private logs are a second, local source of evidence, but they are not treated
as an unfiltered language corpus or as an immediate general L3 update. The
collector separates them by outcome:

```text
confirmed IME acceptance
-> private corpus / L4 local memory

rejected candidate without final target
-> censored outcome; telemetry only

rejected candidate + observed final target, or explicit undo
-> future target-bound pairwise anti-wave

typed-only event
-> shadow corpus only; no authority until an outcome receipt exists
```

The local feedback receipt stores no raw phrases, log lines, or personal word
list. A clean heldout corpus remains the only source allowed to create or
mutate global L3 profiles. Personal accepted text becomes a separate corpus
surface and requires independent cross-surface support before promotion.

### Confirmed Feedback Corpus

The trainer can also extract a private clean-text corpus from the same outcome
log. This is a corpus source for a later cold build, not a runtime packet and
not an automatic install path:

```bash
lay-nanda-wave-train --build-l3-context-feedback-corpus \
  --usage-events ~/.local/share/lay/nanda_wave/word_usage_events.jsonl \
  --out ~/.local/share/lay/corpora/l3_confirmed_feedback.txt \
  --max-repeat-per-phrase 4
```

Only `accepted_ime` and `confirmed_ime_prediction` events become lines. Each
complete phrase must be lexically attested, and one normalized phrase is capped
at four occurrences. `rejected_ime` and `rejected_candidate` stay out of the
text corpus and are censored until the runtime records a linked final target
or explicit undo receipt.

## Measured Checkpoint: 2026-07-20

Remote machine: 20 hardware threads. Corpus: 100,000 natural Russian sentence
fragments. The old compiler required more than 600 seconds for this scale.

```text
online compile wall                236.70 s
compile speedup lower bound          2.53x
compile peak RSS                 359,388 KiB
estimated bounded learner       142,121,488 bytes
candidate artifact               14,809,340 bytes
full train + four-mode proof         367.67 s
full proof peak RSS              372,560 KiB
raw corpus text stored                    no
corpus passes during compile               1
```

The speed and bounded-state architecture pass. Promotion does not:

```text
heldout evaluated                  60,825
correct full supports               5,433
false top-1                           126
support coverage                    8.93%
phase improved / worsened       5,433 / 0
anti improved / worsened        2,285 / 1,442
semantic ablation drop              1,408
verdict                              WATCH
package published                    false
```

The remaining debt is candidate applicability quality, not corpus replay or
parallelization. Do not lower proof thresholds, add phrase hardcodes, or bring
back full-corpus caches to turn this checkpoint into a PASS.

## No-Oracle Checkpoint: 2026-07-20

The original L2 competitor helper was found to reorder candidates by edit
distance to the corpus target. That leaked the teacher label into L3 training
and proof. The helper now preserves the real L2 readout order; the target is
used only to label a proposal as correct or wrong.

On 100,000 natural Tatoeba sentences with the no-oracle lattice, the field is
real but is not safe to publish yet:

```text
heldout transitions                 59,361
support coverage                     11.676%
support precision                     99.071%
phase improved / worsened       6,931 / 0
anti improved / worsened        4,086 / 621
anti false-top1 reduction                38
remaining false top-1                    65
verdict                               WATCH
```

This is a stronger result than the older measurement because L3 no longer sees
an answer-shaped ordering. The next debt is not threshold lowering: candidate
profiles must cover the real L2 lattice more completely, so the correct
candidate is present when a competing phase center becomes strong. A WATCH
package is never installed or granted runtime authority.

## V3 Status

V3 currently has its data model, bounded format, common scene encoder, online
pair learner, directed dominance graph and proof controls. It is still a cold
candidate: the package remains unpublished until a remote heldout corpus shows
fewer false top-1 winners than the no-pairwise control, zero worsened cases,
permutation parity and the existing L3 safety gates.

## Schema-4 online delta checkpoint, 2026-07-30

An isolated HOME used the proven schema-4 base and an initially empty usage
journal. This exposed and fixed a generic initialization defect: numeric offset
zero was incorrectly treated as "state absent", so the first appended batch
could be skipped. Initialization now depends on durable state-file existence.

```text
base bytes                              30,698,796
base SHA-256  a71d58a0a01f9c5f8fae4328e1e5011043f3e95ac1d5ee760a0dc56b81cd9ad7
base signature schema                           4
independent contextual scenes                   2
corpus passes                                   1
delta bytes                                 1,488
delta signature schema                          4
wall                                        0.34s
CPU                                            90%
peak RSS                                  197,752 KiB
false supports                                  0
verdict                                     WATCH
admitted deltas                                 0
base rewritten                              false
base SHA before/after                    identical
```

`WATCH` is the correct result: the relation did not meet targeted authority,
therefore the manifest remained empty and runtime behavior did not change.
No product, brand, phrase or word-specific scoring exception was added.

Tested: empty-journal first append, schema inheritance, one-pass compile,
targeted proof, five safety sentinels, immutable base and admission gate.
Not tested: automatic PASS from real user traffic, multi-day service stability,
or full 80k proof after a promoted delta.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_SCHEMA4_ONLINE_DELTA_2026-07-30.json
```

## Pairwise-only layout delta checkpoint, 2026-07-30

The live L2 one-symbol route and the cold L3 teacher now observe the same
bounded lattice. For Latin `b`, the field emits the keyboard projection and
the configured visual alternatives. Candidate identity still comes from L2;
L3 receives only the competing surfaces and the full sentence context.

The first delta attempt was rejected even though its targeted proof passed.
It emitted `5` semantic states and `32` unary candidate profiles. A full
transition-by-transition comparison found `2` lost supports and `2` lost
top-1 outcomes hidden by equal aggregate top-1. This is the reason aggregate
parity is not a sufficient delta gate.

The accepted package uses a typed `pairwise_only` delta:

```text
immutable schema-4 base
-> inherit 18,857 semantic projection states during learning
-> learn both directions of the `и | в` competition from sentence scenes
-> emit no semantic or unary state
-> emit one exact pair profile with 16 bounded phase centers
-> targeted proof
-> full differential proof
-> append-only manifest admission
```

Measured facts:

```text
teacher fragments                             128
corpus passes                                   1
training transitions                          512
delta bytes                                  2,192
semantic states emitted                         0
candidate/signature profiles emitted          0/0
pair profiles / centers                      1/16
compile wall                                  93 ms
compile peak RSS                        156,938,240 B
targeted improve                              1/1
targeted safety cases                         5/5
targeted false supports                         0
full heldout lattice transitions           50,592
full compared transitions                  41,064
base -> candidate top-1                1,616 -> 1,616
base -> candidate supports             1,665 -> 1,665
lost profile / support / top-1                0/0/0
new false support / top-1                       0/0
full differential wall                     25.54 s
full differential CPU                       1437%
full differential peak RSS              219,232 KiB
targeted verdict                              PASS
full differential verdict                     PASS
```

The old base package reports `WATCH` under the current absolute package proof
because the L2 lattice denominator has changed since the base was built. That
historical drift is recorded separately and is not relabeled as PASS. Delta
promotion uses the fixed current lattice once and compares every transition
between the immutable base and candidate package.

This is not an `Apple` rule and not a phrase replacement table. The training
corpus contains balanced conjunction and preposition scenes; the package
contains hashes and phase centers only. Unknown and technical scenes remain
`Tied` or `ABSTAIN`. L3 still has no direct text-apply authority.

The proof-gated installer repeats the targeted proof after copying the delta
to its final runtime path, then admits it to the append-only manifest:

```bash
/home/ubu/projects/lay/scripts/install-l3-context-delta.sh \
  --delta /home/ubu/projects/lay/data/lexicon/l3_context_relation_layout_v1.nwpc \
  --cases /home/ubu/projects/lay/data/test_input/l3_incremental_relation_delta_gate.tsv \
  --scope relation-layout-pairwise-v1
```

The installer never restarts IBus and never rewrites the immutable base.

Tested:

- L2 birth of both short layout hypotheses;
- training/runtime one-symbol lattice parity;
- balanced pair directions;
- targeted improvement and five safety sentinels;
- exact 80,000-line corpus and 2,349-row surface field;
- full differential non-regression;
- immutable base and proof-gated append-only admission.

Not tested:

- multi-day daemon stability;
- every possible one-symbol visual ambiguity;
- automatic admission from unreviewed user traffic.

Runtime authority changed during this experiment: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_LAYOUT_PAIRWISE_DELTA_2026-07-30.json
```

## Mixed-case runtime closure, 2026-07-30

The accepted pairwise delta was present in the runtime manifest, but the first
end-to-end probe still abstained. The package proof and the runtime adapter did
not use the same tokenizer at their boundary:

```text
schema-4 context       -> case-preserving tokens
candidate replacement -> lowercased legacy tokens
prefix equality       -> false for mixed-case sentences
L3 report             -> discarded before scoring
```

The schema-aware adapter now tokenizes both sides with the schema-4 relation
tokenizer. The resulting directed pair certificate is carried into L4 as its
own context witness. Broad L2 context support remains separate and cannot
erase the directed L3 winner. The certificate narrows the candidate field; it
does not bypass the transition verifier and does not create a candidate.

Measured release probe:

```text
input       Нужно посмотреть через MTC можно оплатить Apple b
L2 lattice и | в
L3 winner   и
L3 pairwise true
L4 state    witnessed
L4 probe    context_relation
verifier    passed
selected    Нужно посмотреть через MTC можно оплатить Apple и
remote tests context/L4/L2/undo = 75/22/8/1 passed
release build                           110 s
```

Safety probes with no selected candidate:

```text
Apple b
wave b
a b
b
GitHub b
compiler сохранил Quasar b
```

The immutable schema-4 base and the `2,192` byte pairwise delta are unchanged.
No product-specific branch, phrase replacement table, or direct L3 apply
authority was added.

Tested:

- mixed-case schema-4 adapter parity;
- pairwise context witness over two broadly supported L2 states;
- unresolved one-symbol safety scenes;
- context-phase, L4 hidden-state, correction-core and double-Shift regression
  suites;
- installed release smoke without restarting IBus.

Not tested:

- multi-day daemon stability;
- every possible mixed-script sentence;
- automatic admission from unreviewed user traffic.

Runtime authority changed in release `0.2.332`: `true`, only for a verifier-
passed winner carrying a directed pairwise context certificate.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_LAYOUT_RUNTIME_CLOSURE_2026-07-30.json
```
