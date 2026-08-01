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

## Append-only self-teacher promotion gate, 2026-07-30

The self-teacher promotion route is now append-only. It snapshots the current
composite field for proof, compiles a small delta, runs targeted and full
differential proofs, and only then may append the delta to the runtime
manifest. The immutable base is not rewritten.

Two implementation defects were closed while proving this route:

1. proof snapshot used the destructive compaction command and could flip the
   supplied manifest;
2. composite pair banks allowed 64 centers while the binary format accepted
   only 16 normal or 4 hard centers.

The new `--snapshot-l3-context-composite` command is read-only. Composite
runtime and serialized package limits are now identical. Incoming evidence is
phase-merged into the closest bounded center instead of producing an unreadable
package or being silently discarded.

The first self-teacher candidate contained the broad 964-case shadow package.
Targeted proof passed, but full proof rejected it:

```text
delta bytes                              288,076
targeted improve                           37/37
targeted safety                              5/5
lost supports                                 107
lost top-1                                    103
full verdict                                WATCH
```

The teacher was then split into a broad discovery shadow and a publishable
delta compiled only from the 37 selected improvement scenes:

```text
filtered delta bytes                       21,192
candidate profiles                             29
pair profiles / centers                    53 / 61
positive centers                                36
training transitions                            88

targeted improve                           37 / 37
targeted false supports                          0
targeted safety                              5 / 5
targeted verdict                              PASS

full lattice transitions                    50,592
full compared transitions                   41,065
base -> candidate supports            1,665 -> 1,635
gained / lost supports                     24 / 54
base -> candidate top-1              1,616 -> 1,591
gained / lost top-1                        28 / 53
new false supports / top-1                   0 / 0
full verdict                                WATCH
```

This is a useful negative result. A targeted improvement is not sufficient
evidence for online admission: positive and pairwise centers can perturb
previously correct basins without creating an obvious false winner. The full
transition-by-transition differential remains a mandatory conjunctive gate.

What was tested:

- clean/self-generated discovery without live feedback;
- broad shadow scoring over 964 cases;
- filtered one-pass delta compile;
- 37 targeted improvements and five safety sentinels;
- full fixed 80k differential proof;
- transition replay and unsafe-edit scoreboard;
- immutable manifest snapshot;
- bounded pair-bank serialization.

What was not tested:

- a greedily selected subset of the 37 scenes that passes the full proof;
- automatic promotion from live feedback;
- multi-day behavior after self-teacher admission.

Verdict scope:

- append-only infrastructure: `PASS`;
- current self-teacher delta: `WATCH`, rejected;
- runtime authority changed: `false`;
- no daemon, IBus or runtime manifest was reloaded.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_SELF_TEACHER_APPEND_ONLY_DELTA_2026-07-30.json
```

## Causal live-feedback reducer and mandatory full gate, 2026-07-31

Release `0.2.335` closes the missing route between actual IME decisions and
append-only L3 deltas. The journal is discovery evidence, not authority. No
word pair, product name, correction result, or phrase replacement is encoded
in the runtime.

The accepted route is:

```text
word_usage_events.jsonl append
-> bounded causal reducer
   -> direct accepted_fix with one changed tail token
   -> or rejected_ime followed by a confirmed IME choice
      with the exact same normalized context within 16 events
-> rejected -> expected candidate relation
-> at least 2 distinct scenes
-> one-pass mini-delta compile
-> targeted improvement + safety proof
-> frozen 80k full transition differential
-> append to runtime manifest only when all gates PASS
-> immutable base remains byte-identical
```

The reducer is deliberately bounded:

```text
recent rejected IME events                    32
maximum event gap                             16
pending candidate relations                  128
distinct scenes per relation                   8
first compile threshold                        2
later retry thresholds                         4 / 8
full proof corpus                         80,000 lines
full proof minimum surface support              2
```

Eviction is by oldest observed relation, not lexical order. Replaying old
traffic is an explicit one-time operation:

```text
--watch-l3-context-online --once --replay-existing-feedback
```

The number of source bytes already replayed is persisted. A normal fresh
worker starts at the current end of the journal and does not silently train on
historical input.

The admission receipt chain is now stored in the manifest:

```text
delta
+ targeted_proof_receipt
+ full_proof_receipt
```

The full receipt is bound to the composite manifest, canonical delta path, and
exact delta byte count. Admission requires:

```text
targeted verdict                         PASS
targeted false supports                     0
full differential verdict                PASS
lost target profiles                        0
lost supports                               0
lost top-1                                  0
new false supports                          0
new false top-1                             0
```

If targeted proof returns `WATCH`, full proof is not run and the report
contains `full_proof_receipt: null`. If frozen proof sources are unavailable,
the worker writes a bound `WATCH` receipt and cannot admit the delta.

Ownership was split so the online worker is no longer a monolithic file:

```text
src/bin/lay_nanda_wave_train/l3_online.rs
    orchestration, paths, journal offset and state I/O             325 lines

src/bin/lay_nanda_wave_train/l3_online/feedback.rs
    causal event reducer, bounds and candidate relations           401 lines

src/bin/lay_nanda_wave_train/l3_online/proof_chain.rs
    mini-delta compile, targeted/full proof and admission           300 lines
```

### Isolated journal snapshot replay

An isolated replay of the pre-install journal snapshot produced:

```text
source bytes                            511,829
parsed events                             2,670
causal IME choice observations               62
unique pending relations                     61
relations with 2 independent scenes           1
attempted relation               как -> контейнер
targeted verdict                           WATCH
admitted deltas                                0
runtime manifest SHA before/after       identical
runtime authority changed                  false
```

This is a quality result, not a failure of the mechanism. The reducer found
possible feedback, but the field did not demonstrate the requested
improvement, so the relation remained pending and could not change the live
model.

The installed `0.2.335` worker then replayed the live journal after its normal
tail compaction:

```text
source bytes                            511,948
parsed events                             2,614
causal IME choice observations               82
unique pending relations                     82
relations with 2 independent scenes           0
attempted relations                            0
admitted deltas                                0
runtime manifest SHA before/after       identical
IBus engine PID before/after     2,989,683 / 2,989,683
```

The differing event and relation counts are separate snapshots of the bounded
journal, not contradictory denominators. The live replay is the installed
state of record.

### Frozen full-proof control

The frozen sources installed outside the repository are:

```text
/home/ubu/.local/share/lay/nanda_wave/l3-proof/fixed-base-corpus-80k.txt
    80,000 lines
    5,638,191 bytes
    SHA-256 56243b510c93930632c069d440d49c49a5ec58422d622523a0d5130dd085eac7

/home/ubu/.local/share/lay/nanda_wave/l3-proof/surface-geometry-exact.jsonl
    2,349 rows
    181,668 bytes
    SHA-256 9aef85b94831ea72e5027816f9a8258ef3039d218bfc20459a664259cd120673
```

A release-mode zero control using `baseline == candidate` measured:

```text
lattice transitions                       50,592
compared transitions                      41,064
all five regression counters                   0
verdict                                      PASS
wall time                                  26.77 s
average CPU                                1,421%
peak RSS                              219,248 KiB
```

This full cost is paid only after a relation has accumulated enough
independent evidence and passed targeted proof. Ordinary journal appends only
run the bounded reducer.

Tested:

- direct user-correction extraction;
- causal rejected-IME to confirmed-choice pairing;
- accepted-IME prefix removal from sentence context;
- unrelated-context rejection;
- bounded historical replay;
- targeted WATCH with no manifest mutation;
- zero-regression full-receipt validation;
- regression receipt rejection;
- storage of both proof receipts in the manifest;
- frozen 80k release-mode differential control;
- remote test execution and release build with 20 Cargo jobs.

Not tested:

- a real live-feedback relation that passes both gates;
- multi-day accumulation and automatic admission;
- runtime reload after an automatically admitted live-feedback delta.

Verdict scope:

- causal feedback reducer: `PASS`;
- append-only proof chain: `PASS`;
- current journal candidate quality: `WATCH`, no admission;
- runtime authority changed: `false`;
- L3 base rewritten: `false`;
- installed CLI and GNOME extension: `0.2.335`;
- model services reloaded: `true`;
- global IBus restarted: `false`, PID unchanged.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_ONLINE_CAUSAL_FEEDBACK_FULL_GATE_2026-07-31.json
```

## Same-inode journal compaction cursor, 2026-07-31

Release `0.2.335` proved the causal reducer and admission gates, but its byte
offset cursor was not compatible with the actual bounded journal writer.
`usage_prior` caps `word_usage_events.jsonl` at `500 * 1024` bytes by opening
the same file with `truncate(true)` and writing the retained complete-line
tail. The inode therefore stays unchanged while both content and size move.

The incorrect rule was:

```text
new length < old offset
-> set offset to zero
-> read the entire retained journal again
```

Live logs exposed the result before any delta was admitted:

```text
poll interval                                  5 s
parsed events after repeated scans          49,406
causal observations after repeated scans     1,569
pending relations                                87
admitted deltas                                   0
runtime manifest SHA before/after          identical
```

The contaminated counters and pending set were not retained. Their state was
archived, and the worker was rebuilt from the clean state saved before the
first live replay.

Release `0.2.336` replaces the byte-only cursor with:

```text
device + inode
+ complete-line byte offset
+ hashes of the last 32 complete JSONL lines
+ stable snapshot check:
   size + mtime before read
   size + mtime after read
   5 ms settle
   up to 5 attempts
```

Every poll reads at most the bounded `500 KiB` journal. At the default
five-second interval this is about `100 KiB/s` of sequential local reads.
After append or same-inode truncate, the worker finds the longest retained
suffix of its 32-line cursor and parses only lines after that overlap.

If no overlap exists, the worker reanchors without training. This can lose a
feedback event under a hostile multi-writer race, but it cannot replay old
events or grant false authority. An unstable snapshot returns `WouldBlock`;
the service keeps the old cursor and retries on the next poll.

Current ownership:

```text
src/bin/lay_nanda_wave_train/l3_online.rs
    orchestration and persisted state I/O                         321 lines

src/bin/lay_nanda_wave_train/l3_online/feedback.rs
    causal reducer and bounded candidate relations                412 lines

src/bin/lay_nanda_wave_train/l3_online/journal.rs
    stable snapshot, overlap cursor and compaction handling        313 lines

src/bin/lay_nanda_wave_train/l3_online/proof_chain.rs
    mini-delta compile, targeted/full proof and admission           300 lines
```

The controlled same-inode smoke used a copy of the real bounded journal:

```text
append events                                      1
same-inode truncate + append events                1
total parsed events                                2
compactions                                        1
overlap lines                                     32
reanchors without overlap                          0
next empty-cycle output bytes                      0
```

The final installed replay and 25-second live observation measured:

```text
replay source bytes                          511,876
replay parsed events                           2,480
causal observations                               84
pending relations                                 84
ready relations                                    0
admitted deltas                                    0
events parsed after replay                        45
same-inode compactions                             2
reanchors without overlap                          0
runtime manifest SHA before/after          identical
IBus engine PID before/after         2,989,683 / 2,989,683
```

Tested:

- first append after an empty journal;
- normal append with an incomplete trailing line;
- atomic rename compaction;
- actual same-inode truncate compaction;
- fail-closed rotation with no overlap;
- controlled copy of the real `500 KiB` journal;
- final live replay and two compaction cycles;
- remote `11/11` online tests.

Not tested:

- multi-day cursor stability;
- recovery of feedback when concurrent writers produce no shared tail;
- an automatically admitted real-feedback delta.

Verdict scope:

- `0.2.335` byte-offset cursor: `FAIL`, superseded;
- `0.2.336` bounded overlap cursor: `PASS`;
- L3 quality field changed: `false`;
- runtime authority changed: `false`;
- L3 base or manifest changed: `false`;
- global IBus restarted: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_ONLINE_JOURNAL_CURSOR_2026-07-31.json
```

## Partial IME completion edits, 2026-07-31

Release `0.2.337` separates an exactly accepted completion from a completion
that the user accepted as a useful scaffold and then edited. The first
Backspace after Tab no longer emits an immediate unary `rejected_ime`.
The IME keeps a bounded in-memory trajectory until the next word boundary:

```text
typed prefix
+ suggested word
+ context before the prefix
+ editing flag
```

The canonical example is:

```text
прек[расный]
-> Tab
-> прекрасный
-> Backspace removes the accepted trailing space
-> Backspace, Backspace remove "ый"
-> "о"
-> прекрасно
-> next word boundary
```

The emitted typed-memory event is one causal `edited_ime` relation:

```text
kind                         edited_ime
source                       ime
outcome                      confirmed_positive
prefix                       прек
from                         прекрасный
to / word                    прекрасно
accepted suffix chars        6
preserved suffix chars       4
deleted chars                2
inserted chars               1
```

The ownership and learning route is:

```text
IBus provisional completion state
-> Backspace changes provisional -> editing
-> no learning event yet
-> boundary observes the final committed word
-> TypingMemoryEvent::EditedIme
-> usage hot state:
   -> positive evidence for the final word
   -> positive exact transition suggested -> final
   -> no global rejection of the still-valid suggested word
-> L3 causal reducer:
   -> contextual relation suggested -> final
   -> at least 2 distinct scenes
   -> one-pass mini-delta
   -> targeted proof
   -> frozen 80k differential
   -> append-only admission only after both gates PASS
```

The typed prefix is removed from the context before the event is written.
`это было прек[расный] -> прекрасно` therefore teaches the scene
`это было прекрасно`, not `это было прек прекрасно`.

The event keeps the useful shared surface indirectly: the final word receives
positive evidence, while the suggested word is not placed in the global
rejected-word bank. Only the proof-gated contextual pair may later make the
suggested form lose in matching scenes. This follows the existing rule that a
valid completion may lose one context without becoming globally invalid.

Tested:

- exact Unicode edit geometry for `прекрасный -> прекрасно`;
- Backspace retains the pending completion instead of destroying its causal
  chain;
- the next boundary consumes the edit trajectory;
- usage memory attracts `прекрасно` without globally rejecting
  `прекрасный`;
- L3 online receives one direct contextual relation and no synthetic
  `rejected_ime` pairing;
- cold feedback-corpus reconstruction retains the final selected phrase;
- the existing double-Shift undo test remains PASS;
- the full IBus baseline and changed tree have the same seven unrelated
  environment-sensitive failures; no new failure was introduced.

Not tested at this point:

- a manually observed physical GUI Tab-edit-boundary sequence in a target
  application after installing `0.2.337`;
- admission of this relation from two real independent scenes;
- calibration that uses the stored edit geometry to vary evidence weight.

Verdict scope:

- partial-completion causal capture: `PASS` in unit and integration tests;
- global rejection of a partially useful valid suggestion: `0`;
- L3 package or runtime manifest changed: `false`;
- double-Shift ownership or behavior changed: `false`;
- live observation authority changed: `true`, `edited_ime` is now a typed causal
  input to usage memory and the online L3 reducer;
- correction-decision authority changed at installation: `false`, no L3 delta
  was admitted and the existing model manifest remained byte-identical.

Installed runtime closure:

```text
installed release                                  0.2.337
lay SHA-256                 59655bb6589c569962bd8d576a1d3890a7856018c5dd5c0c21b2c3a47c76b118
lay-daemon SHA-256          32a8929497b12d9a1e2ec78e91b9ffac399d716d2357a353c2bc02b84cc92eee
lay-ibus-engine SHA-256     0810456508d156f496181b5524223f8cab5bd05d2afae763f2fdcbd40a1faed5
lay-nanda-wave-train SHA-256
                            9e61e07febbcfa09b4fe199d1047e5c05202d23abf61548038ec1a8590aa44e2
lay-daemon PID / RSS                         3,816,139 / 242.2 MiB
L3 online PID / RSS                          3,816,143 /   0.4 MiB
managed IBus PID before/after                3,816,776 / 3,818,488
managed IBus RSS                                          139.9 MiB
global ibus-daemon PID before/after                    3,793 / 3,793
selected engine                                          lay-ime-ru
loaded GNOME tray bridge version                            0.2.337
L3 manifest SHA-256          8d28e83b2426b1c18cb5f8edc55a14d0e24f8f799abfc917a5c4675d211a0e9f
L3 package SHA-256           a71d58a0a01f9c5f8fae4328e1e5011043f3e95ac1d5ee760a0dc56b81cd9ad7
global IBus restarted                                      false
```

The GNOME extension's own `Version()` method returned `0.2.337` after its
bounded disable/enable reload. The managed engine and global IBus PIDs did not
change during that reload. `gnome-extensions info` still reports its older
metadata-registry cache (`0.2.324`); this is not the loaded tray module version.

The managed-engine replacement also exposed an unsafe lifecycle helper:
`pkill -f` could match a controlling shell that merely mentioned
`lay-ibus-engine`. The helper now selects the exact managed argv, verifies
`/proc/<pid>/exe`, and sends `TERM` only to the selected PID. A real controlled
restart preserved the global IBus PID and returned `lay-ime-ru`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_PARTIAL_IME_COMPLETION_EDIT_2026-07-31.json
```

## Partial IME edit live GTK closure, 2026-07-31

The first physical GTK proof invalidated the source-only `0.2.337` verdict for
the live route. A committed-tail Backspace is intentionally returned to the
client, and GTK sends an IBus `Reset` after applying it. The old soft-reset
handler discarded `pending_ime_completion_learning` after the first
Backspace. The visible edit continued, but no `edited_ime` relation survived.

Measured `0.2.337` failure:

```text
live GTK input prefix                                  прек
top completion suffix                                расно
physical result after the probe                  прекрасно
edited_ime events                                        0
IBus Reset calls after native Backspace                   3
verdict                                           LIVE_FAIL
```

The probe intentionally used one Backspace too many for that top candidate;
the important failure was not the resulting surface but the missing event.
Trace evidence showed that every native Backspace was followed by `Reset`, and
the pending edit was destroyed before the boundary.

Release `0.2.338` changes only soft-reset ownership:

```text
ordinary pending completion + Reset                 discard
active editing trajectory + soft Reset             preserve
focus change                                        discard
boundary                                            finalize once
```

This is bounded by the existing `editing` flag. The flag is set immediately
before the first committed-tail Backspace. A normal soft reset before editing
still clears the pending completion, and a focus reset always clears it.

The exact isolated live proof selected the real second candidate instead of
assuming a surface:

```text
typed prefix                                          прек
candidate selected with Down                         расный
Tab result                                      прекрасный + space
edit                           Backspace x3 + "о" + Space
visible GTK result                              прекрасно + space
edited_ime event count                                  1
from                                           прекрасный
to                                              прекрасно
accepted / preserved suffix chars                    6 / 4
deleted / inserted chars                             2 / 1
source / outcome                  ime / confirmed_positive
```

The engine wrote the proof event to an isolated temporary usage journal. The
production `word_usage_events.jsonl` contained `0` `edited_ime` events before
and after the smoke, so synthetic evidence did not enter user learning.

Remote and installed evidence:

```text
soft-reset edit test                                  1/1 PASS
existing edit trajectory test                         1/1 PASS
pending Tab acceptance test                           1/1 PASS
check --lib --bins                                         PASS
isolated smoke lifecycle, ru_p_enter                  1/1 PASS
remote release build                                  109.75 s
remote build max RSS                              1,560,004 KiB
remote build swap                                             0
installed release                                       0.2.338
managed IBus PID after all smoke                     4,061,087
managed IBus RSS                                    143,052 KiB
global ibus-daemon PID                                   3,793
loaded tray bridge                                      0.2.338
lay-daemon / L3 online                                active / active
```

The smoke lifecycle was tightened in the same pass. It now selects only exact
managed-engine argv, verifies `/proc/<pid>/exe`, accepts Linux's
` (deleted)` suffix after atomic binary replacement, writes usage evidence to
temporary files, and restores the exact original IBus engine. Global IBus is
never restarted.

Not tested:

- two independent real user scenes reaching online-delta admission;
- a correction-decision change caused by the new observation;
- every GTK, browser, Electron and terminal client reset sequence.

A path-based TSV smoke was not counted because the test-input binary was built
on the remote host and retained that host's compile-time fixture root. The
path-independent `ru_p_enter` case was used for the lifecycle proof and passed;
the exact partial-edit proof used direct physical events and passed separately.

Verdict scope:

- exact physical GTK partial-completion capture: `PASS`;
- synthetic production-learning events: `0`;
- observation authority changed: `true`;
- correction-decision authority changed: `false`;
- L3 package and manifest changed: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_PARTIAL_IME_COMPLETION_EDIT_LIVE_2026-07-31.json
```

## Direct-only IME feedback sanitation, 2026-07-31

The first production journal audit after enabling partial completion edits
found that event transport and learning authority were mixed incorrectly.
There were two independent defects:

```text
raw typed owner                         lay-daemon + lay-ibus-engine
exact duplicate rows in retained log                         345
old L3 rejected-IME pairing       rejection N + later choice N+k
old bounded pending relations                              128
```

The pairing had no interaction identity. A visible suggestion ignored by
typing another word could therefore become anti-evidence, then the unrelated
later word could become its positive target. Examples observed in the retained
journal included `адрес -> апосмотреть`, `без -> браузер`,
`использовать -> ивдешь`, and `по -> перписки`. This is a failed ownership
model, not useful noisy evidence.

Release `0.2.339` defines one producer and one admissible learning route:

```text
word boundary
-> lay-daemon writes the one raw typed event

visible IME prediction ignored
-> bounded debug trace only
-> no positive evidence
-> no negative evidence

prediction exactly matches the completed word
-> exact lexical attestation
-> weak confirmed_ime_prediction evidence

Tab completion edited before the boundary
-> same live edit trajectory
-> exact prefix/from/to geometry
-> exact lexical attestation of final surface
-> one edited_ime relation

accepted_fix or valid edited_ime
-> direct contextual relation
-> bounded pending bank
-> targeted proof
-> frozen full differential proof
-> admission or WATCH
```

The exact-attestation gate accepts a final surface only when it exists in the
English layout lexicon or an exact Russian HotField/L2 decoder bank. Generated
morphology can still help readout, but cannot by itself turn an observed typo
into positive learning authority. Production examples
`зарегестрированы`, `режимем`, `перписки`, `апосмотреть`, `ивдешь`, and `такм`
are rejected. Valid targets `прекрасно`, `хостинге`, `зарегистрированы`,
`режиме`, `видишь`, `переписки`, and `посмотреть` remain admitted. A missing
L2 lexical artifact now yields a conservative rejection instead of panicking
inside feedback admission.

Historical compatibility is deliberately conservative:

```text
rejected_ime without interaction identity           ignored
unattested edited/prediction positive                ignored
identical same-second event payload                  counted once
raw append-only journal                              retained
usage-count schema                                   14 -> 15
L3 online state                         v1 heuristic -> v2 direct
```

The v1-to-v2 migration preserves the generation and admitted-delta count,
clears only derived pending/cursor/counter state, and replays the retained
journal through the direct-only reducer. It does not delete or rewrite the raw
journal.

An isolated replay of a production snapshot measured:

```text
parsed retained events                              2,569
old pending relations                                 128
new direct relation observations                        5
new unique pending relations                            5
ready for admission                                    0
causal cross-event observations                         0
admitted deltas                                         0
runtime authority changed                           false
```

The five retained direct relations were
`дтп -> lng`, `предложения -> предложении`, `русских -> русский`,
`уничтожить -> уничтожил`, and `хостинг -> хостинге`, each with one independent
scene. None can enter the runtime until the existing evidence and proof gates
pass.

Tested:

- production typo and valid-form exact-attestation regression;
- direct completion-edit geometry and forged-prefix rejection;
- separate rejected/predicted events cannot form an L3 relation;
- historical invalid positives are censored during cold rebuild;
- exact duplicate payloads are counted once during cold rebuild;
- v1 state migration preserves generation/admission identity and removes
  heuristic pending state;
- missing L2 decoder artifact returns no attestation instead of aborting;
- usage-prior `34/34`, typing-memory `15/15`, context compiler `7/7`, and L3
  online `16/16` focused suites.

Not tested at this checkpoint:

- two independent post-install user scenes reaching online admission;
- a correction-decision change caused by a new direct relation;
- every supported client lifecycle;
- the unrelated data-dependent legacy IME candidate expectation failures in
  the broad preedit suite. The representative
  `four_letter_russian_prefix_can_use_wave_lookup` failure was reproduced from
  clean commit `afa7ba7` (`0.2.338`) before these changes.

The remote broad `--lib` run is not green: `1,043/1,083` passed and `40`
route/data-dependent tests failed outside the owning suites. Only the
representative IME failure above was independently reproduced on the clean
parent, so the other 39 are recorded as unresolved baseline status rather than
claimed as pre-existing. This release verdict is limited to the focused
feedback ownership, migration, and fail-closed artifact gates listed above.

Verdict scope:

- learning-event ownership: `PASS`;
- historical contamination removal on an isolated production replay: `PASS`;
- correction-decision authority changed: `false`;
- live service migration: `PASS`;
- post-install physical user event capture: `PASS`.

Installed closure:

```text
release                                             0.2.339
L3 online state                 lay-l3-online-v2-direct-relations
pending relations                                128 -> 5
ready relations                                         0
admitted deltas                                         0
usage-count schema                                  14 -> 15
known bad positive surfaces present                      0
remote release build                              110.24 s
remote build max RSS                         1,551,976 KiB
remote build swap                                         0
lay-daemon PID / RSS                       542,957 / 160,828 KiB
L3 online PID / RSS                       542,959 /   4,580 KiB
managed IBus PID / RSS                    542,987 / 139,852 KiB
global ibus-daemon PID                                  3,793
engine before / after                   lay-ime-ru / lay-ime-ru
loaded tray bridge                                    0.2.339
post-restart service warnings                              0
post-install retained journal rows                         34
post-install typed / confirmed rows                   18 / 16
post-install exact duplicate rows                           0
```

The final release binary also replayed a current journal snapshot in an
isolated HOME with no L2 lexical artifact. It processed `2,565` events,
retained two relations supported by the common exact lexicon, admitted no
delta, and exited normally. This is a fail-closed availability proof only; it
does not replace the full-artifact five-relation replay above.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_IME_FEEDBACK_SANITATION_2026-07-31.json
```

## WeChat preedit editing and synthetic-input containment, 2026-08-01

Two distinct failures were reported from the WeChat input field. They must not
be represented as one model-quality problem.

The first failure was external synthetic input. The private IBus trace retained
`494` actual `keyval=46`, `keycode=53`, decoded `.` key events. These were real
input events, not a preedit rendering marker. The retained tail reached its
bounded `160` character capacity and the text was subsequently removed with
Backspace. A three-second live observation found no continuing dot events and
no active `lay-test-input`, `xdotool`, `ydotool` client, or runtime-smoke
process. The exact historical producer cannot be proven from the old trace
because it did not record input-device identity.

The containment rule is now:

```text
direct lay-test-input scenario
-> reject before virtual-keyboard creation

isolated run_runtime_smoke.py capture
-> set LAY_TEST_INPUT_ARMED=1 explicitly
-> allow the bounded scenario
```

The long-running `ydotoold` service is not evidence of a producer by itself;
it owns `/dev/uinput` but had no active client during diagnosis. Runtime `lay`
does not depend on that service because `lay-daemon` owns its own virtual
keyboard.

The second failure was a real IME ownership defect. In managed committed-tail
mode the typed prefix is already committed to the application, while the
visible completion is a virtual IBus preedit suffix. The old Backspace route
immediately recomputed and republished that suffix before returning the same
Backspace to the client. A client that treats preedit cancellation as the key's
effect therefore removed the prediction instead of the real prefix character.

The corrected route is:

```text
typed prefix + visible virtual suffix
-> Backspace
-> hide preedit suffix
-> clear candidate tracking
-> update Lay committed-tail memory by one character
-> return Backspace to the client
-> do not republish a suffix in the same key event
```

The adjacent candidate readout was also checked against the current single-owner
contract. Bracket rendering is presentation-only and cannot create a second
candidate gate. A one-character suffix may be shown when the completed surface
is an attested L2 center, including a corpus-backed morphology form such as
`жуть -> жутью`. Geometry score alone cannot authorize an unbound
one-character suffix. Full-token and split/glue replacements remain owned by
the boundary/autocorrect route rather than the completion-suffix route.

Measured code evidence:

```text
composition-edit focused tests                       6/6 PASS
central live-field selector tests                     4/4 PASS
full lay-ibus-engine tests                        155/155 PASS
WeChat-shaped state: прек + расный -> пре          PASS
physical post-install WeChat confirmation              PASS
preedit/candidate/replacement state after Backspace empty
unarmed lay-test-input exit code                         1
virtual keyboard created by rejected invocation         no
lay-test-input compile                                PASS
remote release build, CARGO_BUILD_JOBS=20             PASS
final remote release build elapsed                   1m59s
installed runtime version                          0.2.340
installed lay-daemon PID                            1630127
installed lay-ibus-engine PID                       1630206
global ibus-daemon restarted                          false
active engine after activation                   lay-ime-ru
```

What remains unproven at this checkpoint:

- the exact process that produced the already-finished historical dot stream;
- every Electron, Chromium, GTK, Qt, terminal and native WeChat lifecycle.

Verdict scope:

- committed-prefix Backspace ownership: `PASS_CODE`;
- accidental direct test-input containment: `PASS_CODE`;
- single-character live candidate display gate: `PASS_CODE`;
- live WeChat confirmation: `PASS`;
- trained L1.1, L2 or L3 package authority changed: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/IME_WECHAT_PREEDIT_EDIT_CONTAINMENT_2026-08-01.json
```
