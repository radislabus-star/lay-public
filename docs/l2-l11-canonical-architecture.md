# L2 Canonical Architecture Above L1.1

Status: target internal architecture for the real `L1.1 -> L2 -> L3 ->
verifier` route.

Last source audit: 2026-07-26.

Runtime authority: disabled.

This document records the canonical internal shape of `L2` above `L1.1`. It is
the owner contract for the local candidate field, not a runtime promotion by
itself.

## 1. Why This Document Exists

The current live route is still structurally mixed:

```text
deterministic candidates
+ CompactL2 shared field
+ L1.1 hot restore sidecar
+ shadow same-lemma morphology donor
+ shadow transition-phase teacher
-> one correction lattice
```

That is good enough for continued development, but it is not a clean final
architecture. Right now:

- `/home/ubu/projects/lay/src/nanda_wave/lexical_grokking/restoration.rs`
  owns true `L1.1` lexical restoration readout;
- `/home/ubu/projects/lay/src/correction_core/candidate_sources.rs`
  still owns the live candidate merge and still routes through
  `CandidateReadoutRoute::CompactL2`;
- `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/field.rs`
  is a shadow morphology teacher, not the live canonical `L2`;
- `/home/ubu/projects/lay/src/nanda_wave/l2_candidate_phase.rs`
  is a transition-phase package, not the final `L2` above `L1.1`.

The new `L2` must become one explicit owner:

```text
L1.1
  restores damaged token signal

L2
  owns local competition between restored forms

L3
  owns broader phrase / semantic pressure

verifier
  owns destructive edit authority
```

## 2. Current Live Reality

The factual live route on 2026-07-26 is:

```text
CorrectionRequest
-> deterministic_text_candidates()
-> nanda_text_candidates()
   -> hot_l2_text_candidates()
   -> optional hot_l11_restore_candidate()
-> unified candidate lattice
-> TransitionDecisionCore
-> verifier
```

Important consequences:

1. `L1.1` is already real as a restoration sidecar, but it is not yet the sole
   lexical owner.
2. `CompactL2` is still the live candidate-field route.
3. Morphology and transition-phase learning exist, but they are still
   side-teachers rather than the canonical owner above `L1.1`.

This document defines how that mixed route must close into one real `L2`.

Implementation status on 2026-07-26:

- `CandidateReadoutRoute::L2FieldShadow` now exists as a separate shadow route;
- the new route lives under
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/`;
- `L2FieldShadow` no longer requires an injected
  `L2CorrectionPeakContext` for candidate birth;
- `L2FieldShadow` now self-prepares its local lexical / boundary / `L1.1`
  sidecar contour directly from the input text;
- `L2FieldShadow` now also contains one narrow internal morphology donor for
  same-lemma / morphology-slot competition;
- that donor is shadow-only, limited to already-born Cyrillic local surface
  candidates, and only activates when exactly one same-lemma cohort exists
  inside the bounded shadow field;
- the donor is backed by the existing 462k-form morphology package through
  `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/runtime.rs`;
- it is still donor-based and currently reuses the existing lexical-phase,
  boundary, layout, and `L1.1` donor packages rather than a standalone
  canonical `L2` package;
- runtime authority did not change.

What was tested for this code step:

- `scripts/cargo-guard.sh check --lib`: passed;
- `scripts/cargo-guard.sh check --bin lay`: passed;
- `scripts/cargo-guard.sh check --bin lay-nanda-wave-eval`: passed;
- `correction_core::candidate_sources_tests::l2_field_shadow_route_uses_shadow_surface_source_ids`:
  passed;
- `correction_core::candidate_sources_tests::l2_field_shadow_route_self_prepares_l11_candidate_without_peak_context`:
  passed;
- `correction_core::candidate_sources_tests::l11_sidecar_candidate_enters_shared_surface_route`:
  passed;
- `correction_core::candidate_sources_tests::l11_sidecar_candidate_skips_tied_restore_without_authority`:
  passed;
- `scripts/cargo-guard.sh run --bin lay -- --help`: shows
  `--candidate-route <compact-l2|l2-field-shadow|full-wave>`;
- `scripts/cargo-guard.sh run --bin lay -- --help`: shows
  `--compare-candidate-routes`;
- `scripts/cargo-guard.sh run --bin lay -- --restore-word --candidate-route l2-field-shadow --verbose врмея`:
  returned `время`;
- `target/debug/lay --compare-candidate-routes` on the 4-line smoke set
  `{врмея , руддщ , я думаю допусти мнабираю , посмотри }`:
  `selected_surface_diverged = 0 / 4`,
  `selected_gate_diverged = 0 / 4`,
  `selected_provenance_diverged = 3 / 4`.
- `target/debug/lay --compare-candidate-routes 'врмея ' --candidate-route l2-field-shadow`:
  `selected_surface_diverged = 0 / 1`,
  `selected_gate_diverged = 0 / 1`,
  `selected_provenance_diverged = 1 / 1`,
  with `candidate_count = 7` on both routes.
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2938`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 27 / 134`,
  `compact_apply = 37 / 134`,
  `shadow_apply = 37 / 134`,
  `user_target_match.compact = 6 / 134`,
  `user_target_match.shadow = 6 / 134`,
  `user_target_match.both = 6 / 134`.

What was not tested in this step:

- fixed heldout `L2` proof;
- live IME/daemon authority flip;
- latency, RSS, and cold-load budget of a real standalone `L2` package.
- formal batch-time / RSS receipt for the self-owned replay path.

Verdict scope:

- the new route compiles and is wired as a separate shadow owner contour;
- `L2FieldShadow` now owns its own candidate-birth input contour instead of
  consuming a prebuilt `CompactL2` peak context;
- on the measured 134 real correction-log inputs, that ownership split kept
  selected surface parity and selected gate parity with `CompactL2`;
- it is not yet evidence of a finished standalone canonical `L2` package.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_SMOKE_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_SELF_OWNED_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_SELF_OWNED_EXAMPLES_2026-07-26.json`

Runtime authority changed:

- `false`

## 2.1 First Internal Morphology Donor Inside `L2FieldShadow`

What was tested for this code step:

- `scripts/cargo-guard.sh check --lib`: passed;
- `scripts/cargo-guard.sh test --lib same_lemma_`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2938`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 26 / 134`,
  `compact_apply = 36 / 134`,
  `shadow_apply = 36 / 134`,
  `user_target_match.compact = 6 / 134`,
  `user_target_match.shadow = 6 / 134`,
  `user_target_match.both = 6 / 134`.

Measured implementation facts:

- the morphology donor now lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- it calls
  `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/runtime.rs`
  lazily through `shadow_same_lemma_surface_readout(...)`;
- it only evaluates already-born shadow surface candidates;
- it only runs for Cyrillic local candidates;
- it only acts when exactly one same-lemma cohort is present;
- on a `Winner`, it filters losing surfaces from that cohort and retags the
  promoted shadow candidate with `L2FieldShadowMorphology`.

What was not tested in this step:

- fixed heldout `L2` proof for same-lemma competition;
- real-log examples where the donor emits `Tied` or `Abstain`;
- live IME authority change;
- latency and RSS of the morphology donor under daemon load.

Verdict scope:

- `L2FieldShadow` now contains its first real internal donor above the input
  contour: same-lemma / morphology-slot competition;
- this donor remains shadow-only and did not change runtime authority;
- on the measured 134 real correction-log inputs, it preserved selected surface
  parity and selected gate parity with `CompactL2`;
- this is not yet proof of a full standalone canonical `L2` local field.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_SAME_LEMMA_MORPHOLOGY_2026-07-26.json`

Runtime authority changed:

- `false`

## 2.2 Second Internal Donor: Near-Neighbor Lexical Competition

What was tested for this code step:

- `scripts/cargo-guard.sh check --lib`: passed;
- `scripts/cargo-guard.sh test --lib near_neighbor_`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2938`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 26 / 134`,
  `compact_apply = 36 / 134`,
  `shadow_apply = 36 / 134`,
  `user_target_match.compact = 6 / 134`,
  `user_target_match.shadow = 6 / 134`,
  `user_target_match.both = 6 / 134`.

Measured implementation facts:

- the near-neighbor donor also lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- it runs after same-lemma morphology filtering and before unified candidate
  materialization;
- it only inspects already-born shadow lexical candidates;
- it only runs on Cyrillic local competition;
- it builds one bounded near-neighbor cohort around the current lexical leader;
- it only acts when the current leader also wins the internal near-neighbor
  strength readout by a conservative margin;
- on a `Winner`, it filters weaker near-neighbor competitors from that cohort
  and retags the promoted shadow candidate with `L2FieldShadowNearNeighbor`.

What was not tested in this step:

- fixed heldout `L2` proof for near-neighbor competition;
- replay examples where the donor should return explicit `Tied` or `Abstain`;
- live IME authority change;
- latency and RSS of the near-neighbor donor under daemon load.

Verdict scope:

- `L2FieldShadow` now contains a second real internal donor above the input
  contour: bounded near-neighbor lexical competition;
- this donor remains shadow-only and did not change runtime authority;
- on the measured 134 real correction-log inputs, it preserved selected surface
  parity and selected gate parity with `CompactL2`;
- this is still not yet proof of a full standalone canonical `L2` local field.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_NEAR_NEIGHBOR_LEXICAL_2026-07-26.json`

Runtime authority changed:

- `false`

## 3. Ownership

### 3.1 L1.1 Ownership

`L1.1` owns only:

- one damaged token at a time;
- lexical restoration of that token;
- bounded candidate lattice for that token;
- `Winner / Tied / Abstain` over lexical restoration evidence;
- lexical evidence such as geometry, positive phase, backward reconstruction,
  anti evidence, ambiguity shells, and crystallization state.

`L1.1` does not own:

- phrase-local ending choice;
- same-lemma form competition;
- context-driven candidate reordering;
- neighbor-governed morphology slot choice;
- multiword competition;
- destructive edit authority.

### 3.2 L2 Ownership

`L2` owns:

- the first local field above the bounded `L1.1` lattice;
- same-lemma form competition;
- near-neighbor lexical competition;
- local morphology-slot inference;
- preposition / particle / auxiliary / agreement cues;
- phrase-local tied vs winner vs abstain readout;
- candidate evidence attribution for local decisions.

`L2` is the owner of decisions such as:

```text
посмотреть / посмотри / посмотрим
дом / дома / домом
времени / время / временами
посмотри / просмотри / подсмотри
```

These are local-field decisions, not `L1.1` decisions and not `L3` decisions.

### 3.3 L3 Ownership

`L3` owns only broader context pressure:

- wider phrase memory;
- semantic suppression or support;
- longer-range preference shifts;
- maintaining ties when local evidence is not decisive.

`L3` must not become a substitute for the missing real `L2`.

### 3.4 Verifier Ownership

The verifier remains the sole owner of destructive edit authority:

```text
selected local winner
-> structural verification against visible snapshot
-> AuthorizedEdit or no-op
```

No `L1.1`, `L2`, or `L3` object may bypass this boundary.

## 4. Critique Of The Current Live Shape

The current live shape is useful, but architecturally wrong in three ways:

1. lexical restoration and local phrase competition are still split across
   separate candidate producers;
2. morphology knowledge exists, but does not yet sit inside one canonical local
   field above `L1.1`;
3. `CompactL2` remains the runtime owner, while `L1.1` is only injected as an
   extra source.

That means the current runtime still behaves like:

```text
several candidate birth routes
-> merge
-> decide
```

instead of the desired:

```text
L1.1 bounded lexical lattice
-> one real L2 local field
-> one local readout
-> L3 pressure
-> verifier
```

The purpose of the new `L2` is to remove that ownership drift.

## 5. Canonical L2 Memory

The canonical `L2` package should be centered on stable IDs, not raw string
heuristics.

### 5.1 Main Memory Objects

```text
L2 Field Package
|
+-- FormCenterRef
|   stable reference to an existing L1.1 visible form
|
+-- LemmaCenter
|   lexical identity shared by several visible forms
|
+-- MorphBinding
|   FormCenterRef <-> LemmaCenter <-> slot/features
|
+-- LocalContextMode
|   bounded phrase-local scene identity
|
+-- SlotPhaseCenter
|   learned local evidence for a slot or form family
|
+-- NeighborCoupling
|   support/repel relation from nearby token classes or surfaces
|
+-- CompetitionEdge
|   candidate-vs-candidate local suppress/support relation
|
+-- TieCalibration
|   honest thresholds for Winner / Tied / Abstain
|
+-- DecoderRef
    materializes visible UTF-8 output from FormCenterRef
```

### 5.2 FormCenterRef

`L2` must not duplicate the lexical surface memory that already belongs to
`L1.1`. It should reference it.

Minimal shape:

```text
FormCenterRef
  l1_terminal_id
  script_flags
  length_bucket
  decoder_ref
```

Meaning:

- `L1.1` keeps the visible form identity;
- `L2` points at that identity and competes using local context.

### 5.3 LemmaCenter

`LemmaCenter` is the local family owner for several visible forms:

```text
LemmaCenter
  lemma_id
  primary_pos
  form_range
  local_context_profile_range
  competition_edge_range
```

For example one `LemmaCenter` can bind:

```text
посмотреть
посмотри
посмотрим
посмотрел
посмотрела
```

`L2` then chooses between these forms from local scene evidence.

### 5.4 MorphBinding

`MorphBinding` binds visible form to lemma and slot:

```text
MorphBinding
  form_center_ref
  lemma_center_id
  feature_mask
  support
  flags
```

For Russian the slot must encode at least:

- part of speech;
- case;
- number;
- gender where relevant;
- person;
- tense;
- mood;
- aspect;
- infinitive / finite / imperative distinction.

The existing shadow teacher in
`/home/ubu/projects/lay/src/nanda_wave/morphology_phase/field.rs` is the
starting donor for this layer.

### 5.5 LocalContextMode

`LocalContextMode` is a compact identity for phrase-local scene features:

```text
left function token class
+ right function token class
+ punctuation boundary class
+ local position / adjacency mode
+ optional neighboring lexical class anchors
```

This object must stay bounded and cheap. `L2` is not a full sentence semantic
graph.

### 5.6 SlotPhaseCenter

`SlotPhaseCenter` is the learned local scene pressure for a slot or tight form
group:

```text
scene
-> positive subcenters
-> anti subcenters
-> score for slot/form family
```

Examples:

- imperative scene;
- infinitive-governed scene;
- noun after preposition requiring one case;
- adjective-noun agreement scene;
- plural noun scene;
- finite verb after pronoun scene.

### 5.7 NeighborCoupling

`NeighborCoupling` carries short-range local support or repulsion:

```text
neighbor pattern
-> supports candidate family
-> or repels candidate family
```

Examples:

- `в + noun(prepositional)`
- `к + noun(dative)`
- `не + imperative / finite contrast`
- adjective agreement cues;
- stable local two-word motifs.

### 5.8 CompetitionEdge

`CompetitionEdge` is explicit candidate-vs-candidate local pressure:

```text
candidate A
candidate B
scene key
support delta
anti delta
tie-allowed flag
```

This is the core object that prevents the field from collapsing into one global
string score.

### 5.9 TieCalibration

`TieCalibration` must be learned from evidence, not hard-coded around one
example:

```text
minimum positive
minimum margin
tie window
abstain window
false-authority ceiling
```

The important principle is honest local uncertainty:

- if one same-slot candidate family wins clearly, emit `Winner`;
- if several candidates remain locally valid, emit `Tied`;
- if the scene is too weak, emit `Abstain`.

## 6. Canonical L2 Runtime Path

The runtime path must become:

```text
input token
-> L1.1 lexical restoration
-> bounded L1.1 lattice
-> L2 field birth
-> same-lemma expansion
-> near-neighbor expansion
-> local slot scoring
-> pairwise competition
-> Winner | Tied | Abstain
-> L3 broader pressure
-> TransitionDecisionCore
-> verifier
-> AuthorizedEdit or no-op
```

### 6.1 L2 Field Birth

`L2` begins from the full bounded `L1.1` lattice, not only its top-1.

For each `L1.1` candidate:

1. map `terminal_id -> FormCenterRef`;
2. expand to its `LemmaCenter`;
3. add same-lemma alternate forms that are legal for the bounded local scene;
4. add explicit near-neighbor competitors already linked by local competition
   edges;
5. keep source attribution from `L1.1`.

### 6.2 Same-Lemma Expansion

If `L1.1` restores a lexical family correctly but not the local form,
`L2` must be able to walk within that family:

```text
L1.1 surface winner = посмотреть
local scene = imperative
L2 family walk = {посмотреть, посмотри, посмотрим, посмотрел, ...}
L2 local winner = посмотри
```

This is the main reason `L2` cannot be replaced by raw lexical restoration.

### 6.3 Near-Neighbor Expansion

`L2` must also carry local competition between geometrically close but
context-distinct families:

```text
посмотри
просмотри
подсмотри
досмотри
```

These edges must be explicit and bounded. `L2` should not brute-force the full
lexicon at runtime.

### 6.4 Local Readout

Local readout must use:

- `L1.1` evidence floor;
- slot evidence;
- same-lemma pressure;
- neighbor couplings;
- pairwise competition edges;
- tie / abstain calibration.

`L2` readout emits:

```text
ordered local lattice
+ local verdict
+ evidence attribution
+ tie/abstain reason
```

### 6.5 IME And Daemon Readout

IME and daemon must consume the same `L2` readout.

IME remains only:

- display backend;
- commit backend;
- accepted-completion source.

IME must not own a separate lexical or morphology brain.

## 7. Proposed Code Ownership

The target code layout should converge to:

```text
src/nanda_wave/l2_field/
|
+-- model.rs
+-- compiler.rs
+-- runtime.rs
+-- format.rs
+-- proof.rs
+-- teacher.rs
+-- bridge.rs
+-- mod.rs
```

Proposed responsibilities:

- `model.rs`
  core records: `LemmaCenter`, `MorphBinding`, `CompetitionEdge`,
  `LocalContextMode`, `TieCalibration`;
- `compiler.rs`
  build package from corpora, logs, shadow teachers, and calibrated evidence;
- `runtime.rs`
  field birth, bounded expansion, local competition, local readout;
- `format.rs`
  deterministic binary package format;
- `proof.rs`
  fixed heldout, per-class local decision proof, tie honesty, latency, RSS;
- `teacher.rs`
  cold teacher import from morphology/package builders and future corpora;
- `bridge.rs`
  the one adapter from current correction-core route into the new `L2`.

Existing donors:

- `src/nanda_wave/lexical_grokking/restoration.rs`
  donor for the `L1.1 -> lattice` boundary;
- `src/nanda_wave/morphology_phase/field.rs`
  donor for morphology-slot field concepts;
- `src/correction_core/candidate_sources.rs`
  current live merge route that the new bridge must replace;
- `src/nanda_wave/l2_candidate_phase.rs`
  separate transition-phase donor, but not the canonical local `L2`.

## 8. Proof Contract

Promotion of the new `L2` requires a fixed proof that measures the local-field
job directly, not only lexical restoration.

### 8.1 Required Proof Families

The fixed `L2` proof must contain at least:

1. same-lemma form choice;
2. local morphology slot choice;
3. near-neighbor lexical competition;
4. tie honesty on ambiguous scenes;
5. abstain honesty on underdetermined scenes;
6. zero direct mutation authority bypass;
7. hot-path latency and bounded RSS.

### 8.2 Required Scoreboard

Every run must report:

```text
same-lemma top-1
same-lemma tie coverage
same-lemma false authority

morphology-slot top-1
morphology-slot authority
morphology-slot false authority

near-neighbor top-1
near-neighbor tie coverage
near-neighbor false authority

ambiguous tied accuracy
abstain honesty

package bytes
cold load
steady RSS
hot p50 / p99
```

Aggregate winners are not enough. Per-class denominators are required.

### 8.3 Runtime Promotion Gate

The new `L2` may replace `CompactL2` only when:

1. fixed local proof passes;
2. live shadow route shows no unsafe regression;
3. `TransitionDecisionCore` behavior remains verifier-safe;
4. IME and daemon both read the same emitted lattice;
5. evidence attribution remains inspectable.

## 9. Cutover Plan

The cutover should happen in five explicit stages.

### 9.1 Stage A: Package Build

Compile a standalone `L2` package from:

- `L1.1` terminal identities;
- morphology bindings;
- local context scenes;
- competition edges;
- tie calibration.

No runtime authority yet.

### 9.2 Stage B: Shadow Readout

Add a new shadow route beside `CompactL2`:

```text
CandidateReadoutRoute::CompactL2
CandidateReadoutRoute::L2FieldShadow
```

`L2FieldShadow` must consume the same `CorrectionRequest` and emit the same
kind of bounded candidate evidence object.

### 9.3 Stage C: A/B Receipts

On fixed corpora and selected live logs compare:

- current `CompactL2`;
- `CompactL2 + L1.1 sidecar`;
- new canonical `L2FieldShadow`.

The comparison must show where the new route wins, ties, abstains, or regresses.

### 9.4 Stage D: Runtime Flip

Only after passing the fixed local proof:

```text
live route
CompactL2
-> canonical L2Field
```

At that point `L1.1` stops being an injected sidecar and becomes the formal
lexical input to `L2`.

### 9.5 Stage E: Remove Ownership Drift

After promotion:

- remove the old mixed candidate merge path;
- keep `CompactL2` only as a historical adapter if still needed for receipts;
- keep morphology and transition-phase teachers as teachers, not hidden live
  owners.

## 10. Forbidden Behaviors

The canonical `L2` must not:

- re-implement raw lexical restoration already owned by `L1.1`;
- collapse the `L1.1` lattice to one candidate before local competition;
- brute-force the whole lexicon at runtime;
- depend on IME-only state as its main evidence source;
- silently replace `Tied` or `Abstain` with fake certainty;
- bypass `TransitionDecisionCore` or the verifier;
- hide its local winner without evidence attribution.

## 11. Canonical Summary

The clean target is:

```text
damaged token
-> L1.1 restores lexical basin
-> L2 chooses locally valid form and neighbor winner
-> L3 adds broader phrase pressure
-> verifier decides whether edit may happen
```

The main correction to the current runtime is simple:

`L1.1` must stop being only an extra candidate source.
It must become the formal lexical base of one real `L2`.
