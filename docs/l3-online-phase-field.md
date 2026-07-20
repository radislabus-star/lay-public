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
