# Canonical Phase Word Recovery Cutover

Status: authoritative execution plan.

This document is the single source of truth for the L1-L4 refactor. Older
architecture notes remain useful as history and supporting contracts, but they
must not define a competing runtime route or a different meaning of L1-L4.

## 1. Final Objective

Convert lay from a hybrid of stored-word retrieval, rule-owned candidate
selection, duplicated L1 paths, and phase admission into one canonical typing
processor:

```text
observed typing state
-> canonical L1 surface field
-> canonical L2 lexical phase field
-> phase-conditioned surface reconstruction
-> typed candidate lattice
-> L3 phrase field
-> L4 temporal and signed experience field
-> one TransitionDecisionCore
-> independent verifier
-> sealed AuthorizedEdit
-> one selected backend
```

The defining product claim is narrow and testable:

```text
L1 encodes the damaged surface.
L2 settles it toward lexical basins and reconstructs candidate surfaces.
L3 decides which lexical reading is coherent in the phrase.
L4 estimates the current temporal state and supplies accepted/rejected history.
Bayes supplies local priors, but never becomes decision authority.
Only TransitionDecisionCore can choose a transition.
Only a verifier can create AuthorizedEdit.
```

The refactor is complete only when the production hot path no longer depends on
`word_id -> String` retrieval as lexical authority and no legacy candidate path
can bypass the canonical lattice.

## 2. Non-Negotiable Laws

1. No production module may directly apply a correction it generated.
2. No word-specific production hardcode is allowed.
3. No test fixture, expected answer, or verifier label may become runtime input.
4. No `source_words: Vec<String>` may exist in the hot lexical artifact.
5. Raw corpus words are cold compiler input, not live candidate authority.
6. A grapheme alphabet is allowed because reconstruction must be reversible;
   whole-word storage is not required for that reversibility.
7. L1, L2, L3, L4, Bayes, and phase memory may only produce typed evidence.
8. `TransitionDecisionCore` is the only chooser.
9. `AuthorizedEdit` is the only mutation capability.
10. IME, daemon, GNOME, and replay modules are backends, not correction brains.
11. A normal typo transition may touch only the active token.
12. A boundary transition requires a separate typed proof.
13. Unknown, stale, low-margin, or conflicting states produce `SuggestOnly` or
    `Abstain`, never a guessed destructive edit.
14. Temporary shadow code must be deleted at cutover. Rollback is a Git
    checkpoint, not a permanently retained second architecture.

## 3. Canonical Layer Definitions

### L1: Surface Field

L1 owns one representation of observable form:

```text
Unicode scalar/grapheme IDs
script and keyboard projection
4-gram plus position atoms
token and phrase boundaries
cursor-relative position
surface anomaly and residual atoms
```

L1 does not know words, meanings, candidate strings, usage counts, or edit
policy. The existing per-character packet route and 4-gram center route must be
merged into this one encoder and one artifact schema.

### L2: Lexical Phase Field

L2 owns lexical form and candidate birth:

```text
L1 center sequence
-> motif/lexical basin activation
-> positive and anti-center interference
-> top-k latent surface hypotheses
-> phase-conditioned grapheme decoder
-> candidate surfaces
```

L2 must generate surfaces from compact field state. It may not select a word ID
and clone a stored full word. Generic operators such as layout projection,
boundary shift, keep, and completion may seed the hypothesis field, but they do
not decide the winner.

Error classes such as transposition, missing letter, repeated letter, and
substitution are descriptions of the observed delta after reconstruction. They
are not independent rule-owned correction brains.

### L3: Phrase Field

L3 owns phrase compatibility:

```text
candidate lexical state
+ left phrase state
+ language scene
+ grammatical and semantic relations
-> context energy / anti-energy / unknown
```

L3 must compare all serious L2 candidates. It may suppress a locally plausible
word that breaks the phrase and strengthen a noisy candidate that restores a
coherent phrase. It cannot create an edit or override a verifier veto.

### L4: Temporal State and Experience Field

L4 owns the estimated hidden typing state:

```text
previous state
+ key/IME/daemon observation
+ accepted/rejected/undo outcome
-> estimated current state
-> signed transition evidence: positive / neutral / negative
```

L4 is not a workflow rule table. It is a state estimator and transferable
memory of transition outcomes. Bayes usage priors are an input to L4/readout,
not a second winner selector.

## 4. Canonical Runtime Tree

```text
INPUT ADAPTERS
|
+-- daemon event adapter
+-- IBus/IME adapter
+-- manual toggle adapter
+-- replay/eval adapter
|
v
TypingStateSnapshot
|
v
nanda_wave::l1
|   canonical SurfaceFieldState
v
nanda_wave::l2
|   lexical basins + phase decoder + CandidateProposal list
v
nanda_wave::l3
|   ContextReadout per candidate
v
nanda_wave::l4
|   StateReadout + SignedExperience per candidate
v
typing_transition::TransitionDecisionCore
|   Apply | SuggestOnly | Keep | Abstain | Veto
v
typing_transition::TransitionVerifier
|   revision, focus, caret, boundary, layout, backend postconditions
v
text_edit::AuthorizedEdit
|
+-- daemon/uinput backend
+-- IBus CommitText/DeleteSurroundingText backend
+-- GNOME ReplaceText/layout backend
+-- undo backend
```

Dependency direction is one-way. A lower line may not import policy from a
higher line. In particular, `nanda_wave` must not call
`TransitionDecisionCore`, and adapters must not call candidate generators
directly.

## 5. Canonical Data Contracts

The migration introduces one versioned type for every boundary:

```text
TypingStateSnapshot
  focus_id
  revision
  caret
  visible_tail
  active_token_span
  composition_state
  observed_layout
  backend_capabilities

SurfaceFieldState
  grapheme_ids
  l1_center_refs
  residual_atoms
  script_projection
  layout_projection
  boundary_state

LatentLexicalHypothesis
  lexical_center_id
  motif_path
  decoder_state
  positive_energy
  anti_energy
  support
  novelty

CandidateProposal
  rendered_surface
  active_span
  operator_hint
  l1_readout
  l2_readout
  provenance

ContextReadout
  phrase_energy
  anti_energy
  relation_support
  unknown

StateReadout
  estimated_state_id
  signed_experience
  usage_prior
  context_prior
  uncertainty

TransitionProposal
  state_before
  candidate
  predicted_state_after
  typed_operator
  all_readouts

TransitionProof
  preserved_invariants
  changed_invariants
  independent_verifier_evidence
  backend_postconditions

AuthorizedEdit
  private constructor
  verified proposal
  exact edit plan
  expected revision/focus
  layout postcondition
```

Stringly typed `source_id`, `operation`, and support strings must be replaced by
enums and structured evidence before legacy deletion.

## 6. Hot and Cold Memory Contract

### Cold compiler inputs

```text
corpus words and books
morphological forms
clean phrase corpora
accepted/rejected user events
synthetic corruption generators
training labels
debug/explain metadata
```

### Hot read-only artifact

```text
grapheme ID table
L1 center records
L1-to-L2 motif references
lexical phase centers
anti-centers
phase-conditioned decoder transitions
terminal/continuation probabilities
morpheme and boundary transition centers
quantized usage priors
artifact header, offsets, hashes, version
```

The artifact must be a deterministic binary file, memory-mapped read-only by
daemon and IME. Shared pages must remain shared. Runtime must not rebuild it and
must not hold duplicate `HashSet<String>` or `Vec<String>` copies.

### Ephemeral runtime data

Only the observed input and the final top-k rendered candidates may exist as
strings. Candidate strings are bounded, short-lived, and never become the
primary index.

## 7. Physical Module Ownership

Target modules:

```text
src/nanda_wave/
  phase_math.rs                 one phase/vector/hash implementation
  artifact.rs                   binary schema and mmap reader
  compiler/                     cold-only training/compilation
  l1/
    mod.rs                      facade
    encoder.rs                  canonical live encoder
    centers.rs                  4-gram/position center readout
    types.rs
  l2/
    mod.rs                      facade
    field.rs                    lexical basin settling
    decoder.rs                  center-conditioned grapheme generation
    lattice.rs                  top-k typed proposals
    morphology.rs               compact form transitions
    types.rs
  l3/
    mod.rs
    context_field.rs
    phrase_memory.rs
    types.rs
  l4/
    mod.rs
    state_estimator.rs
    signed_memory.rs
    usage_prior.rs
    types.rs

src/typing_transition/
  engine.rs                     one pipeline orchestrator
  decision.rs                   one chooser
  verifier.rs                   independent proof
  state.rs
  operator.rs
  proof.rs

src/text_edit/
  executor.rs                   sealed AuthorizedEdit
  plan.rs
  safety.rs
  backends/                     physical output only
```

Files are split by owner and runtime route, not merely by line count. Facades
should remain small. Proof fixtures and cold compiler code must never be in the
runtime dependency closure.

## 8. Duplicate Elimination Map

Each concept receives one owner:

```text
surface normalization        -> nanda_wave::l1
token/segment boundaries     -> word_reader
script/layout projection     -> nanda_wave::l1
phase vector math            -> nanda_wave::phase_math
surface distance             -> text_metrics
candidate identity/dedup     -> nanda_wave::l2::lattice
candidate ranking            -> TransitionDecisionCore
phrase compatibility         -> nanda_wave::l3
usage and signed experience  -> nanda_wave::l4
operator typing              -> typing_transition::operator
transition verification      -> typing_transition::verifier
edit planning                -> text_edit::plan
physical mutation            -> text_edit::backends
```

The old implementation is deleted after callers move. It is not wrapped and
left behind. AST/graph guards must reject:

```text
production `if word == ...`
hot `Vec<String>`/`HashSet<String>` lexical authority
test fixture `include_str!` in runtime compiler input
direct CommitText/delete/uinput outside backend modules
candidate source importing DecisionCore
IME importing Bayes, L2 training, or phrase policy
string-based operator/source dispatch
duplicate normalize/split/script/hash/phase helpers
```

## 9. Full Implementation Route

### Stage 0: Freeze the truthful baseline

- Commit and tag the current installable state.
- Record version, Git hash, runtime binary hashes, config, daemon/IME PSS,
  startup time, candidate p50/p99/max, decision/output latency, and dirty-log
  quality.
- Archive current unsafe-edit report and real-suite per-class scoreboard.
- Record the known baseline: Wave 879/1181, deterministic 869/1181, 89 Wave
  regressions, and current surface-L2 ablation behavior.
- Build a fixed compatibility corpus for Tab, Space, first-word IME, double
  Shift, layout synchronization, Telegram, Firefox, WeChat, and terminal paths.

Exit: a reproducible baseline artifact and rollback tag exist.

### Stage 1: Make the plan enforceable

- Add architecture tests for the dependency direction and single authority.
- Replace substring-based architecture PASS with AST/graph-derived checks.
- Add a duplicate-symbol report for normalization, token splitting, phase math,
  language classification, candidate selection, and text mutation.
- Add forbidden-import and forbidden-call guards.
- Fix the Lay live-transition gate profile so the project gate runs on Lay,
  rather than a wrapper-selected foreign profile.

Exit: architecture violations fail before implementation begins.

### Stage 2: Purify proof and data ownership

- Split data into `train`, `heldout`, `adversarial`, `runtime-user`, and
  `explain-only` domains.
- Remove synthetic expected outputs and seed phrases from live runtime sources.
- Remove `verified:true/false` and all outcome labels from model input atoms.
- Keep verifier outcome only as the training target.
- Create contamination checks based on normalized words, n-grams, phrase
  windows, and generated corruption ancestry.
- Invalidate the stale causal receipt. Do not refresh it until the new proof
  passes without leakage.

Exit: runtime cannot read proof fixtures and heldout cannot overlap training.

### Stage 3: Unify L1

- Merge `l1.rs` packet emission and `l1_center_memory.rs` into one
  `SurfaceFieldEncoder`.
- Use one normalization path and one position encoding.
- Preserve short-token behavior with padded/boundary atoms rather than a second
  special L1.
- Make daemon, IME, eval, and cold compiler consume the same encoder.
- Prove debug/runtime parity byte-for-byte for the same input snapshot.

Exit: one L1 type, one implementation, one test suite, no parallel encoder.

### Stage 4: Build the cold field compiler

- Read corpora only in an offline compiler binary.
- Compile L1 centers, lexical motif centers, anti-centers, morphology/boundary
  transitions, decoder transitions, and priors into one versioned artifact.
- Make compilation deterministic and bit-reproducible.
- Store source hashes and compiler version in the manifest.
- Reject stale or partially written artifacts atomically.

Exit: the same corpus and config produce the same artifact hash.

### Stage 5: Replace hot word storage

- Remove `source_words` from `L2CenterMemory`.
- Remove `runtime_l2_surface_word_set` and every duplicate hot word set.
- Remove word-ID posting lists that require full strings for final readout.
- Keep only compact center, motif, transition, terminal, and prior arrays.
- Load the artifact through read-only mmap shared by daemon and IME.
- Make memory accounting include mapped bytes, resident shared pages, private
  allocations, capacities, and ephemeral candidate strings.

Exit: production hot memory contains no full-word lexical authority.

### Stage 6: Implement true phase-conditioned surface reconstruction

- Encode the damaged token into canonical L1 state.
- Settle activation against lexical positive and anti-centers.
- Produce top-k latent lexical hypotheses without looking up full words.
- Decode grapheme IDs conditioned on query state, lexical center, motif path,
  position, and context feedback.
- Emit stop/terminal probability and uncertainty.
- Render grapheme IDs to UTF-8 only for bounded final candidates.
- Replace the current `prev2 + prev1 + position` decoder as sole authority. It
  may survive temporarily only as an explicit baseline ablation.

Exit: heldout words absent from all source-word tables are reconstructed, and
disabling phase centers causes a large measured drop.

### Stage 7: Collapse candidate rules into one lattice

- Candidate producers emit only typed latent proposals.
- Keep generic operators: `Keep`, `SurfaceFieldDecode`, `LayoutProjection`,
  `Completion`, `BoundaryShift`, and explicit `ManualToggle`.
- Infer missing/repeated/transposed/substituted classifications from the final
  surface delta instead of maintaining independent decision-owning rules.
- Deduplicate by predicted surface state, not source string.
- Preserve multiple serious hypotheses through top-k.
- Delete migrated paths from `correction_core`, `ru_typo`, `context_wave`,
  `phrase_reader`, `candidate_gate`, and IME-local helpers.

Exit: every candidate visible in runtime appears in one typed lattice report.

### Stage 8: Rebuild L3 as the phrase authority

- Train phrase state from clean corpora/books and accepted local usage.
- Encode phrase relations and candidate insertion state, not exact phrase
  lookup as authority.
- Return positive, negative, and unknown context energy for each L2 candidate.
- Support long enough context to resolve detached letters, layout ambiguity,
  morphology, and split/glue ambiguity.
- Keep phrase rewriting suggest-only unless a boundary operator is proven.

Exit: context ablation measurably harms ambiguous heldout cases while clean
technical tokens and protected text remain stable.

### Stage 9: Consolidate L4, Bayes, and state estimation

- Move accepted, rejected, undone, and manually selected outcomes into one
  signed state memory.
- Keep local unigram/bigram/trigram priors as compact counts or quantized priors,
  never as source-code word lists.
- Use prediction, observation, correction, and uncertainty in one state
  estimator.
- Make stale focus/revision evidence reset or reduce confidence.
- Remove duplicate usage scoring from L2, IME, correction core, and context
  modules.

Exit: one CLI report explains what L4/Bayes learned and why it changed rank.

### Stage 10: Rebuild the one DecisionCore

- Move all final ranking and admission into `TransitionDecisionCore`.
- Consume structured L1/L2/L3/L4 readouts and typed operator evidence.
- Use one deterministic ordering for `Apply`, `SuggestOnly`, `Keep`, `Abstain`,
  and `Veto`.
- Make uncertainty first-class rather than hiding it in arbitrary score gaps.
- Remove local thresholds and winner selection from candidate sources and IME.
- Replace saturated/ineffective tray weight semantics with one documented
  calibration scale.

Exit: graph analysis finds exactly one candidate chooser.

### Stage 11: Make verification independent

- Construct predicted state before verification.
- Verify focus, revision, caret, active span, left context, token count,
  boundary delta, layout postcondition, and backend capability.
- Keep verifier features out of model input.
- Make `AuthorizedEdit` constructor private to the verifier boundary.
- Require explicit boundary proof for every space-count or left-context change.
- Preserve undo as a typed inverse transition.

Exit: graph analysis finds no physical edit without `AuthorizedEdit`.

### Stage 12: Reduce daemon and IME to adapters

- Both adapters create `TypingStateSnapshot` and call one engine.
- IME may display, select, accept, cancel, and report composition state only.
- Daemon may observe keys, maintain its snapshot, and execute authorized output.
- Remove IME-local ranking, Bayes, phrase prediction, and correction logic.
- Keep backend-specific deletion/commit behavior isolated behind capabilities.
- Ensure Space closes composition and Tab commits exactly one candidate plus
  the configured delimiter.

Exit: the same snapshot gives the same decision in daemon replay, IME, and eval.

### Stage 13: Make layout a typed state transition

- Keep manual double Shift deterministic and user-authoritative.
- Auto layout projection becomes a candidate requiring language/context proof.
- Include the expected GNOME/IBus/tray layout in transition postconditions.
- Verify both RU-to-EN and EN-to-RU on heldout real words and technical tokens.
- Delete independent layout-switch decisions from correction and tray code.

Exit: text, IBus source, GNOME source, and tray state converge after every
layout transition.

### Stage 14: Promote boundary operators last

- Represent split, merge, and moved-prefix repairs as explicit boundary-state
  transitions.
- Require separate positive/anti-centers and a higher margin than token-local
  edits.
- Prove zero left-context mutation outside the declared span.
- Keep multiword phrase rewrites suggest-only until independently proven.
- Run dedicated Telegram, Firefox, WeChat, terminal, and long-tail scenarios.

Exit: unsafe multiword apply and unproven left-context mutation remain zero.

### Stage 15: Atomic authority cutover

- Run old and canonical engines only in bounded shadow replay.
- Compare candidate coverage, top-k, top-1, per-class regressions, unsafe edits,
  latency, memory, and layout outcomes.
- When gates pass, switch the single `typing_transition::engine` call site.
- In the same cutover series, delete legacy authority and temporary shadow
  branches.
- Do not ship a permanent old/new mode selector.

Exit: only the canonical route is reachable in the production graph.

### Stage 16: Delete legacy and collapse facades

- Remove obsolete word retrieval, prefix authority, duplicate L1, old scorer,
  rule graph, string source IDs, stale status fields, and unused config.
- Reduce `correction_core` to a compatibility facade, then remove it when no
  public caller remains.
- Split oversized modules only along the ownership map in this document.
- Remove dead tests that only prove deleted internals; preserve behavior and
  safety fixtures through canonical public tests.
- Regenerate graphify and require no forbidden cross-route edges.

Exit: no compatibility code retains a second brain.

### Stage 17: Release proof and live verification

- Run clean heldout, adversarial, causal ablations, dirty-log replay, route
  tests, memory/latency probes, and runtime application checks.
- Refresh causal receipts only from the final source tree.
- Run NANDA structural and live-transition gates. WATCH, VETO, and ERROR block
  promotion.
- Build/install one versioned checkpoint, restart daemon and IME, verify loaded
  binary hashes, then test live windows.
- Bump the version at every installable checkpoint so runtime provenance is
  visible in tray and logs.
- Push/release only after the installed runtime passes.

Exit: release gates below pass and repository/runtime provenance agree.

## 10. Required Causal Ablations

The final claim is rejected unless all controls exist:

```text
full canonical field
without L1 centers
without L2 phase centers
without anti-centers
shuffled phase
magnitude only
random centers
without L3 context
without L4 signed experience
without Bayes prior
without exact/full-word storage
without current Markov baseline
verifier-label atom forbidden
```

Required interpretation:

- Removing full-word storage must not destroy canonical reconstruction.
- Removing phase centers must significantly reduce heldout lexical recovery.
- Shuffled/magnitude/random phase controls must not preserve the result.
- Removing anti-centers must increase false candidates or reduce margin.
- Removing L3 must hurt context-ambiguous cases, not simple layout projection.
- Verifier false accepts must remain zero for promoted destructive transitions.

## 11. Release Gates

### Correctness

```text
unsafe multiword apply                         0
left-context mutation without boundary proof  0
backend execution without AuthorizedEdit      0
verifier label in model input                  0
proof fixture in runtime input                 0
hot full-word lexical authority                0
```

### Lexical product quality

```text
heldout single-edit candidate coverage         >= 90%
heldout layout projection coverage             >= 99%
heldout correct candidate top-3                >= 90%
clean/protected false apply                     0 in release suite
Wave real-suite overall                        strictly above baseline
Wave per-class regression                      none without explicit review
phase-off causal drop                          >= 20 percentage points
```

The denominator and dataset provenance must be printed with every percentage.

### Performance and memory

```text
warm L1-L2 candidate p99                       <= 1 ms
warm L1-L2 candidate max in fixed bench        <= 5 ms
no corpus parsing or artifact build on keypress
no first-word lazy initialization stall
hot artifact                                   mmap read-only
combined private dirty growth after warmup     bounded and reported
candidate strings                              bounded top-k only
```

Output/backend latency is reported separately from field compute latency.
Performance may not be improved by disabling candidate sources or reducing
quality.

### Runtime behavior

```text
first-word IME candidate visible
Space closes preedit
Tab commits one visible candidate and delimiter
double Shift works on first press
manual toggle remains deterministic
auto layout synchronizes GNOME, IBus, and tray
undo reverses exactly one authorized transition
Telegram, Firefox, WeChat, and terminal smoke pass
```

## 12. Fast Verification Cadence

Do not run the full release suite after every edit.

```text
inner loop
  cargo fmt --check
  focused module tests
  focused graphify query/path
  route-specific NANDA guard

checkpoint
  cargo check --all-targets
  focused clippy for changed crates/targets
  relevant replay/ablation subset
  graphify update .

cutover/release only
  full tests and clippy
  full heldout/adversarial/dirty replay
  causal ablation matrix
  live-transition gate
  release build/install/restart/live windows
```

This keeps implementation fast without weakening the final proof.

## 13. Checkpoint and Commit Strategy

Use behavior-complete commits, not arbitrary file batches:

```text
checkpoint 0  baseline and measurements
checkpoint 1  proof/data separation and architecture guards
checkpoint 2  canonical L1
checkpoint 3  compiler and binary artifact
checkpoint 4  true L2 phase reconstruction
checkpoint 5  L3/L4 and one DecisionCore
checkpoint 6  daemon/IME adapter cutover
checkpoint 7  layout and boundary cutover
checkpoint 8  legacy deletion
checkpoint 9  final proof, version, install, release
```

Every checkpoint must be buildable. Only installable checkpoints require a
version bump. No checkpoint may claim behavior preservation if it also changes
the algorithm without a before/after scoreboard.

## 14. Definition of Done

The project is canonical only when all statements are true:

1. A damaged word can be reconstructed on a clean heldout surface without that
   word existing in any hot full-word table.
2. Phase ablation causally destroys a substantial part of that recovery.
3. One L1 encoder is used by runtime, compiler, replay, and eval.
4. One candidate lattice contains every runtime candidate.
5. One DecisionCore chooses every transition.
6. One verifier creates every AuthorizedEdit.
7. IME and daemon are adapters with no independent correction authority.
8. L3 and L4 contribute measurable context/state value without bypassing
   safety.
9. Training, heldout, proof, runtime, and user-memory data are physically
   separated.
10. Hot memory contains compact field state and bounded rendered candidates,
    not a duplicated corpus.
11. Legacy routes are deleted, not merely disabled.
12. Graph, tests, receipts, installed binaries, tray version, and live behavior
    all describe the same release.

Until then the architecture verdict remains `WATCH`, regardless of local demos
or attractive aggregate scores.
