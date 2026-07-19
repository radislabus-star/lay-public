# Lay L1-L4 Runtime Architecture

Status: canonical live architecture map and forward contract.

Last source audit: 2026-07-19.

This document is the source of truth for the current L1-L4 runtime, its proven
boundaries, and the next architectural work. Historical plans may explain why
the code exists, but they do not override this map.

The document deliberately separates three things:

```text
LIVE       implemented and on the runtime route
PROVEN     backed by a focused test, graph receipt, or measured artifact
OPEN       desired direction, not a current product claim
```

## 1. Product Objective

Lay is a typing transition processor, not a collection of independent
autocorrect rules:

```text
observed typing state
-> surface evidence
-> candidate lattice
-> phrase and experience evidence
-> one selected typed transition
-> immutable snapshot lease
-> independent structural verification
-> sealed AuthorizedEdit
-> one backend
-> observed-state receipt
-> signed feedback or censoring
```

The intended intelligence boundary is:

```text
L1 observes damaged form.
L2 births lexical hypotheses.
L3 evaluates phrase compatibility.
L4 carries temporal and accepted/rejected experience.
Bayes supplies local priors.
The joint transition field combines their pressure.
TransitionDecisionCore is the only chooser.
The verifier is the only path to AuthorizedEdit.
```

No L1-L4 layer is allowed to mutate text.

### Authority Closure Checkpoint

The committed-tail IBus route now carries the winner-owned `EditAction` and
its `VisibleTailSnapshot` through the adapter boundary. It cannot reconstruct
an unrelated correction from strings. `SnapshotIdentity` binds the observable
source, focus, revision, optional caret/selection/composition/layout epochs,
and a stable visible-tail hash. Adapters that cannot observe a coordinate leave
it absent; they may not invent it.

The split is deliberate:

```text
TransitionDecisionCore
  chooses from the L2 lattice under L3/L4/Bayes phase pressure

StructuralTransitionVerifier
  checks the selected transition against the immutable visible snapshot,
  surrounding text, selection and a minimal typed EditPlan

AuthorizedEdit
  is issued only after the structural verifier admits that exact plan
```

The structural verifier does not call L2 or rerank candidates. A dispatched
system edit is censored until IBus observes the expected suffix. Observation
records positive outcome evidence. A visible mismatch quarantines the execution
lease because it can be an IBus/backend synchronization failure, not semantic
evidence against the selected candidate. Explicit undo and reject routes are
the only sources of negative semantic memory; stale epoch, timeout and
unavailable observation are censored.

## 2. Non-Negotiable Laws

1. Candidate producers never apply their own output.
2. Production behavior must not contain word-specific fixes from chat logs.
3. Test labels, expected answers, and verifier output never become runtime
   features.
4. A live applied edit is an observation, not automatically a true label.
5. Corpus text is cold compiler input. Runtime artifacts must be bounded and
   versioned.
6. `TransitionDecisionCore` is the sole candidate chooser.
7. `AuthorizedEdit` is the sole text mutation capability.
8. Daemon, IME, GNOME, tray, and replay are adapters or executors, not separate
   correction brains.
9. Normal correction changes only the active token.
10. Left-context or word-boundary changes require a typed boundary transition
    and independent proof.
11. Stale focus, revision, caret, surrounding text, or backend state causes
    abstention.
12. Unknown or conflicting evidence causes `SuggestOnly`, `Keep`, `Abstain`, or
    `Veto`, never a destructive guess.
13. Phase, Bayes, L3, and L4 can redistribute evidence but cannot bypass the
    verifier.
14. Faster runtime is not accepted if it removes candidate sources or weakens
    quality.
15. A release claim needs a fixed denominator and clean labels. A rotating live
    log is diagnostic evidence only.

## 3. Current Runtime Tree

```text
INPUT ROUTES
|
+-- daemon/evdev committed text
+-- IBus composition and accepted completion
+-- manual double Shift
+-- Enter/Space autocorrect
+-- replay and evaluation
|
v
input_gate
|   one entry to live correction decisions
v
correction_pipeline / correction_core facade
|   candidate production only
v
L1 SURFACE EVIDENCE
|
+-- lexical_surface_atoms::SurfaceFieldEncoder
+-- nanda_wave::l1 per-symbol UTF8/script/keyboard/boundary packets
+-- transition_relation structural delta atoms
|
v
L2 CANDIDATE LATTICE
|
+-- lexical phase readout (LAYLPH02)
+-- layout, boundary, typo, morphology, completion adapters
+-- local usage/context priors
+-- transition phase readout (LAYPC005)
|
v
L3 CONTEXT RELATION PHASE
|   compact semantic states + positive centers + anti-centers
|   support / suppress / neutral
v
L4 HIDDEN TYPING STATE
|   semantic quotient of predicted states
|   accepted / rejected / reverted transition witnesses
v
JOINT TRANSITION INTERFERENCE
|   constructive and destructive evidence within one ranking budget
v
typing_transition::TransitionDecisionCore
|   Apply / SuggestOnly / Keep / Abstain / Veto
v
text_edit actor + verifier
|   focus / epoch / visible tail / boundary / left context / backend proof
v
AuthorizedEdit
|
+-- daemon output backend
+-- IBus commit/delete backend
+-- GNOME layout synchronization backend
+-- undo backend
```

The generated architecture receipt currently proves the authority path. It
does not prove language quality.

## 4. Layer Truth

### 4.1 L1: surface observation

LIVE:

- `lexical_surface_atoms::SurfaceFieldEncoder` creates compact surface atoms;
- `nanda_wave::l1` emits UTF-8, script, keyboard, and boundary packets per
  symbol;
- lexical phase compilation uses position-sensitive and relaxed atom-center
  keys;
- `transition_relation` separately encodes the structural delta of a proposed
  transition.

Important correction to older plans: there is not one universal L1 object in
the code. There are two valid projections of the same event:

```text
lexical projection     what surface basin can reconstruct this token?
transition projection  what typed change does this candidate propose?
```

They share normalized surface utilities but serve different mathematical
objects. Forcing them into one struct would mix lexical retrieval with action
proof. Their schemas must remain compatible, not identical.

L1 has no edit authority.

### 4.2 L2: candidate birth and lexical field

L2 currently has two distinct hot memories.

#### LAYLPH02 lexical artifact

LIVE:

```text
surface atoms
-> center postings
-> terminal hypotheses
-> grapheme decoder path
-> candidate surfaces
```

The artifact contains trie/graph nodes, arcs, terminals, quantized phase
vectors, center postings, decoder states, and decoder arcs. It does not keep a
`Vec<String>` or a `word_id -> String` table.

Truthful boundary: it is still a compact reversible encoding of the compiled
vocabulary. Its graph contains enough Unicode arc information to reconstruct
known surfaces. Therefore it is not yet a purely emergent phase generator that
can invent every valid unseen word from centers alone.

#### LAYPC005 transition phase artifact

LIVE and PROVEN:

```text
transition relation atoms
-> operator profile
-> structural positive / anti-centers
-> lexical candidate positive / anti-centers
-> learned margin thresholds
-> Support / Repel / Unknown
```

The package stores quantized centers, anti-centers, promotion state, support
counts, and thresholds without raw words.

The installed package is compiled from the same fixed dataset used by the
release proof. Live actions and applied edits are not silently appended to
this L2 package. Explicit user feedback is compiled into the separate L4
feedback snapshot described below, so product experience cannot mutate the
heldout L2 proof artifact.

Current production-artifact proof:

```text
promoted operators                 11 / 11
heldout positive support           72 / 72
heldout negative false accepts      0 / 169
positive support without phase      0 / 72
negative false accepts without anti 169 / 169
same-operator negatives observed    396
local lexical candidate negatives   222
nonlocal negatives deferred to L3   174
lexical heldout false supports         0 / 48
false supports without lexical anti   48 / 48
paired candidate top-1                48 / 48
paired top-1 without lexical anti      46 / 48
```

The structural lane proves that the typed operator is admissible. The lexical
lane redistributes candidate energy inside that operator. Its compact
projection contains position-sensitive 4-gram trits and authority buckets, not
raw words. The 174 nonlocal rows are deliberately excluded from lexical
training because they change phrase context or multiple tokens and belong to
L3/L4.

#### Joint transition interference

LIVE:

`typing_transition::decision::interference` combines the released ranking
energies and settles phase competition inside the existing L2 energy budget.

```text
surface L2 energy
+ relative promoted phase margin
-> settled L2 energy

settled L2
+ L3 context relation energy
+ L4 Bayesian signed experience
-> joint transition energy
```

The relative phase pass compares promoted `Support` hypotheses in the same
candidate batch. The strongest margin keeps its L2 energy. Weaker margins are
destructively attenuated. No extra ranking budget is created, so adding a new
scorer cannot silently add new authority.

This is a candidate arbitration mechanism, not a verifier. `Repel` and
`Unknown` still follow phase admission policy.

OPEN:

- expand lexical competitor anti-center coverage from clean accepted/rejected
  current-token pairs;
- replace remaining hand-calibrated L2 peak components with learned package
  calibration after a fixed heldout proves parity;
- raise candidate coverage without loading source corpora into the hot path.

### 4.3 L3: compact context relation phase

LIVE and PROVEN:

```text
cold clean corpus
-> token semantic states
-> context + candidate semantic binding
-> candidate relation vector
-> positive centers / candidate-specific anti-centers
-> learned threshold and batch competition
-> Support / Suppress / Neutral
```

The hot `LAYL3P01` package stores token hashes, quantized phase vectors,
support counts, positive centers, anti-centers, and learned thresholds. It
stores no corpus sentences and no raw word table. The legacy phrase-memory
packet remains diagnostic input only and has no production ranking authority.

Current package:

```text
artifact bytes                  3,246,372
corpus fragments                    8,525
compiled transitions               32,249
semantic states                      4,115
candidate profiles                   9,467
positive centers                    14,968
anti-centers                         3,415
heldout evaluated transitions        3,588
heldout supports                      1,779
heldout false supports                    8
support precision                  99.5523%
supports without phase                    0
false supports without anti              42
top-1 without semantic state            469
```

Causal interpretation:

- removing phase destroys all 1,779 supported heldout transitions;
- removing semantic state drops top-1 by 1,310;
- removing anti-centers increases false supports from 8 to 42.

L3 contributes energy and a relation class. It cannot construct
`AuthorizedEdit` and cannot bypass transition verification.

OPEN:

- widen clean sentence-domain coverage beyond the current public-domain
  Russian corpus;
- add English context-phase compilation with the same package contract;
- keep false apply at zero while moving more neutral heldout rows to support;
- learn cross-surface relation classes without adding raw phrase lookup.

### 4.4 L4: hidden typing state and transition witnesses

LIVE:

- groups extensionally identical predicted states into a semantic quotient;
- binds every predicted state to `state_before + operator + state_after`, so
  equal output words in different typing scenes are not one hidden transition;
- consumes L2 state evidence and L3 relation classes instead of a hand-written
  scene classifier;
- commits a deterministic witness plan before reading candidate-specific
  observations;
- resolves a class through at most four target-independent probes:
  transition history, context relation, verifier result, and phase relation;
- records before/after hypothesis classes in witness receipts and independently
  replays them before the resolution can influence admission;
- gives an exact rejected/reverted witness destructive authority;
- loads a compact local feedback snapshot compiled from the newest known state
  of accepted/rejected dirty-log transitions;
- compiles structural accepted/rejected surfaces into a bounded hot bank of
  four positive and four anti-centers with 24 phase cells each;
- treats missing evidence as `Unobserved`, not as negative evidence;
- keeps unresolved learned conflicts as `Ambiguous` and blocks automatic
  application while still allowing suggestion display.

The signed lane is a Beta/Bayesian posterior over positive and negative
observations. Word/context priors are weak pseudo-counts; they are not a fixed
table of action rules. The removed `l4_signed_outcome` and live
`derive_l4_scene_state` paths no longer contribute ranking authority.

```text
predicted candidate states
-> semantic quotient classes
-> committed target-independent witness plan
-> L2/L3 field, verifier, exact-history and phase observations
-> independently replayed resolution certificate
-> Resolved / Witnessed / Ambiguous / Rejected / Unobserved
```

Important law:

```text
unknown != negative
```

L4 may veto on real learned conflict or negative witness. Absence of L4
experience does not suppress an otherwise verified local transition.

The release workflow rebuilds
`~/.local/share/lay/nanda_wave/word_usage_feedback_counts.json` through
`scripts/rebuild-l4-feedback-memory.sh`. Raw replay rows and intermediate usage
events stay in a temporary directory and are deleted after compilation. The
runtime merges this bounded snapshot with new live usage events; the snapshot
is evidence for L4, never an edit capability.

Fixed latest-state replay at the introduction of this route:

```text
rows                                      before   feedback memory
positive transitions                        117               117
negative transitions                        748               748
rejected target applied                      228                 0
positive top-1/apply                      24.79%            47.01%
positive candidate coverage               69.23%            73.50%
compiled rejected relation surfaces            0               301
```

This proves exact replay of confirmed local feedback. Cross-scene/heldout
generalization from phase witnesses remains a separate `WATCH` claim.

OPEN:

- bind focus, epoch, composition and caret receipts at the executor boundary;
  these remain backend observations and must not become candidate truth;
- learn transition-class margins from clean replay rather than only observing
  current batch separation;
- widen phase-witness transfer across equivalent typing scenes while preserving
  zero wrong-state admission;
- measure organic certificate coverage after the new runtime has accumulated
  live receipts.

Current cutover evidence (2026-07-17):

- active witness plan/certificate tests: 5/5;
- hidden-state tests: 4/4;
- bounded phase/anti bank test, including anti ablation: 1/1;
- typing-transition route tests: 43/43;
- mutation monopoly / transition authority / input-gate contracts: 15/15,
  17/17, 2/2;
- release IME candidate generation: p50 2 us, p90 6 us, p99 7 us, max
  12 us over 120 warmed samples;
- latest dirty replay remains `WATCH-negative-false-apply`: the candidate
  architecture is safe to cut over, but learned quality is not claimed PASS.

### 4.5 Bayes

LIVE:

- local word usage prior;
- context-word prior;
- rejected word and context prior;
- accepted/rejected counts from this installation.

Bayes is a signal, not an independent chooser. Personal frequency remains
local to the machine. Corpus frequency is cold initialization, not personal
truth.

## 5. Decision and Mutation Authority

### Candidate proposal

Candidate producers may emit:

```text
surface
origin
source role
typed error class
evidence count
suggest/apply eligibility request
```

They cannot authorize output.

### Candidate evaluation

For each candidate, `TransitionDecisionCore` derives:

```text
Bayes posterior and risk
structural explanation
typed action operator
transition relation atoms
L2 lexical peak
L2 positive/anti phase margin
L3 phrase disposition
L4 hidden-state quotient and Bayesian signed memory
joint transition field energy
predicted typed transition
```

The batch then settles relative phase competition before selecting a candidate.

### Verification

Selection is still not permission to mutate. The verifier checks:

- active token and changed token count;
- left-context preservation;
- word-boundary proof;
- focus and epoch freshness;
- visible/surrounding suffix agreement;
- backend capability and postcondition;
- layout synchronization where required.

Only successful verification can issue `AuthorizedEdit`.

## 6. Physical Ownership

```text
src/input_gate.rs
  live correction entrypoint

src/correction_core.rs
  facade, candidate collection, trace projection

src/lexical_surface_atoms.rs
  shared surface atom encoding

src/nanda_wave/l1.rs
  per-symbol L1 packets

src/nanda_wave/l2.rs
src/nanda_wave/l2/*
  candidate sources and lexical readout

src/nanda_wave/lexical_phase/{compiler,format,runtime}.rs
  cold lexical compiler and hot LAYLPH02 readout

src/nanda_wave/l2_candidate_phase.rs
  LAYPC005 structural and lexical phase training, package, and readout

src/nanda_wave/l2_wave_peak.rs
  current calibrated lexical peak evidence; still partly hand-calibrated

src/nanda_wave/l3.rs
src/nanda_wave/l3_phrase_gate.rs
  phrase evidence

src/nanda_wave/l4_goal_state.rs
src/nanda_wave/l4_signed_memory.rs
src/typing_memory.rs
  scene and accepted/rejected transition experience

src/typing_transition/decision.rs
src/typing_transition/decision/*
  sole chooser, admission, joint interference, receipt

src/text_edit/*
  actor, verifier, safety, AuthorizedEdit, executor contract

src/bin/lay_daemon/*
src/bin/lay_ibus_engine/*
  adapters and physical backends

src/architecture_contract.rs
src/generated/architecture_graph_receipt.json
  compiled architecture scoreboard
```

## 7. Hot and Cold Memory

### Cold inputs

- dictionaries and corpora;
- generated forms;
- clean accepted/rejected transition examples;
- phrase corpora;
- counterfactual and role-swap negatives;
- fixed heldout truth sets.

Cold input may contain strings. It is not loaded merely because the process
started.

### Hot runtime

- memory-mapped or bounded lexical graph records;
- quantized lexical phase vectors and postings;
- promoted transition centers and anti-centers;
- compact usage/context snapshots;
- bounded L3/L4 state.

The hot runtime must expose bytes, counts, version, checksum, and warmup time.

### Runtime truth

`raw_words_stored=false` for LAYPC005 means the transition package stores no
raw words. It must not be generalized into the false claim that all lexical
runtime state is non-reversible. LAYLPH02 deliberately keeps a reversible
grapheme graph so output text can be produced.

## 8. Scoreboard

### Architecture authority

Current generated graph verdict: `PASS`.

Proven boundaries:

- one live correction entrypoint;
- one candidate chooser;
- IME backend-only authority;
- sealed text mutation capability;
- boundary and stale-state verification;
- no raw words in the transition phase package.

### Safety

Latest pre-change live safety snapshot:

```text
unsafe multiword apply                 0
unverified left-context apply          0
gate failures                          0
```

These are release invariants, not quality metrics.

### Lexical quality

Recent live diagnostics showed high top-k coverage but unstable top-1. The L4
feedback snapshot now prevents the latest confirmed rejected targets from
being re-applied, while clean heldout quality remains the authority for broader
language claims. A raw live `from -> to` row is not accepted until user
accept/reject state classifies it.

Current quality verdict: `WATCH`.

The required fixed report is:

```text
clean heldout rows
candidate coverage
top-1 before/after joint field
false applies
abstains
operator confusion matrix
same-operator lexical confusion matrix
phase ablation
anti-center ablation
latency p50/p95/p99/max
```

### Performance

Microsecond lexical readout and end-to-end output latency are different
denominators. Disk logging, lazy warmup, backend delete/insert, GNOME/IBus
synchronization, and candidate generation must be timed separately.

## 9. Debt Queue

### P0: expand clean truth and lexical anti-centers

Extend the fixed heldout from explicit user correction, immediate undo/reject,
and curated corpus corruption. Do not label every applied edit as true. Only
same-token alternatives may train candidate-specific anti-centers. Multiword
and left-context negatives remain L3/L4 evidence.

Exit gate:

```text
clean top-1 improves
false apply does not increase
same-operator wrong candidate rate decreases
anti-center ablation removes the gain
```

### P1: candidate coverage

Phase arbitration cannot recover a candidate that L2 never births. Expand
compact lexical coverage and morphology without loading raw corpora into the
hot process.

### P2: L3 phrase intelligence

Train compact phrase relation memory on clean corpora and prove contextual
disambiguation on unseen phrase families.

Latest cold 200k proof is intentionally still `WATCH`:
`docs/structural_gates/receipts/L3_CONTEXT_200K_HARD_NEGATIVE_PROOF_2026-07-19.json`.
The field has causal phase and semantic ablation signal, but 793 heldout false
top-1 candidates remain. It is not a runtime authority until those collisions
are separated without collapsing valid support.

### P3: L4 temporal state

Increase organic witness coverage. The runtime already binds state-before,
operator and predicted state, consumes real accept/reject/undo observations,
uses bounded phase centers, and verifies every active-resolution certificate.
The remaining debt is coverage and cross-scene transfer, not a missing owner.

### P4: IME parity

IME display must consume the same candidate field readout as autocorrect while
remaining backend-only. First word, Space closure, Tab accept, Backspace, and
application-specific delete profiles require live verification.

### P5: latency tail

Eliminate lazy heavy construction from the input path. Keep all candidate
sources and measure generation, decision, deletion, insertion, layout sync,
and logging independently.

### P6: code boundaries

Split modules only when route evidence proves reduced foreign pull. File size
alone is not evidence. Keep the current facade/owner direction and remove dead
duplicate helpers after graph proof shows zero callers.

## 10. Proof and Release Gates

Required for every ranking or memory cutover:

1. Fixed baseline artifact and denominator.
2. Positive heldout.
3. Negative and near-miss heldout.
4. Phase ablation.
5. Anti-center ablation.
6. Old-vs-new shadow replay.
7. Zero unsafe multiword apply.
8. Zero unverified left-context apply.
9. Architecture graph `PASS`.
10. Focused route tests.
11. Candidate coverage not reduced.
12. Latency and RSS not worsened outside the declared budget.
13. Installed daemon, IME, extension, and tray versions agree.
14. Live runtime process and logs confirm the new binary is loaded.

No release may claim “grokking”, “full nonlinear memory”, “full sentence
understanding”, or “no lookup” unless a causal proof uses the exact production
artifact and runtime route.

## 11. Fast Verification Cadence

During a local edit:

```text
cargo fmt --check
cargo check for the changed target
focused unit/integration tests for the changed route
git diff --check
```

At a route checkpoint:

```text
scripts/check-lay-changed.sh
scripts/check-architecture.sh
graphify update .
nanda-guard-diff for the selected route
```

Before installation or release:

```text
fixed quality replay
unsafe edit gate
transition replay
release build
version sync
runtime restart
live version/process verification
```

Broad full-suite checks are release work, not the inner edit loop.

## 12. Maintenance Rule

When a runtime boundary changes, update all three in the same commit:

1. the owning code and focused tests;
2. `src/architecture_contract.rs` and generated graph receipt when applicable;
3. this document's runtime tree, scoreboard, and debt queue.

If code and document disagree, code plus generated graph are current evidence,
and this document must be corrected before release.

## 13. Definition of Done

The long-term architecture is complete when:

```text
one surface encoder family with explicit lexical/transition projections
one compact L2 candidate field
learned lexical attraction and candidate-specific destructive interference
one L3 phrase relation field
one temporal L4 signed state memory
one joint transition chooser
one independent verifier
one sealed AuthorizedEdit route
backend-only daemon/IME/GNOME executors
no candidate-specific production hardcodes
no unlabelled live action treated as truth
no unsafe multiword or left-context mutation
quality, latency, RSS, and causal ablations all pass on fixed artifacts
```

The current code has the authority skeleton and compact phase memories. Learned
same-operator anti-centers now provide local destructive interference. The main
remaining intelligence debt is broader clean lexical coverage plus stronger
L3/L4 phrase and state evidence for context-dependent ambiguity.
