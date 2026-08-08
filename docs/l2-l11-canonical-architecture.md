# L2 Canonical Architecture Above L1.1

Status: canonical L2 ownership is closed for the live local IME/daemon route:

```text
L1.1 bounded lattice
-> standalone V13 CanonicalL2Field
-> one Winner | Tied | Abstain readout
-> L3
-> TransitionDecisionCore
-> verifier
```

Last source audit: 2026-08-01.

Runtime authority: unchanged by the ownership cleanup. The live default is
`CandidateReadoutRoute::CanonicalL2Field`; `FullWave` remains a compare-only
reference. The immutable V13 package is loaded directly and was not rebuilt.
There is no executable same-lemma or near-neighbor donor fallback in
`src/nanda_wave/l2_field/bridge.rs`.

Sections that describe `L2FieldShadow`, morphology donors, or near-neighbor
donors are retained below as dated implementation history. They do not describe
the current executable owner after 2026-08-01.

## 0. Canonical Live Owner Closure, 2026-08-01

The current code ownership is:

```text
CorrectionRequest
-> CandidateReadoutRoute::CanonicalL2Field
-> canonical_text_readout()
-> bounded L1.1 seed surfaces
-> StandaloneL2Field::readout() over immutable V13
-> CanonicalL2FieldReadout { candidates, authority }
-> one shared candidate lattice
```

Measured facts:

- installed V13 bytes: `135121803` (`128.86 MiB`);
- installed V13 SHA-256:
  `bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e`;
- package SHA before and after the code cutover: identical;
- warmed before/after semantic snapshot: `8 / 8` inputs identical;
- candidate records compared: `86` before and `86` after;
- semantic snapshot SHA-256 before and after:
  `08e25753179ff608ef96ab968f8585803e337afc0d3701337fee69160ae1f418`;
- selected-surface divergence: `0 / 8`;
- selected-gate divergence: `0 / 8`;
- standalone V13 fixed proof remains the quality authority:
  same-lemma false authority `0`, near-neighbor false authority `0`;
- focused `nanda_wave::l2_field` proof: `26 passed / 0 failed` after removal
  of eight test-only donor tests and their dead implementation.
- remote 20-job release build: `110.43 s`, average CPU `203%`, peak RSS
  `1563256 KiB`, swaps `0`;
- remote focused test build/run: `26 / 26` passed in `10.96 s`, average CPU
  `156%`, peak RSS `1412604 KiB`.

The before/after snapshot compares replacement, error class, gate action,
winner/none, scoreboard, candidate count, and candidate ordering. Diagnostic
route names, source IDs, and reason strings are intentionally renamed from
`Shadow` to `Canonical`; they are not field geometry.

What was not tested in this ownership-only change:

- no L1.1 or L2 package was recompiled;
- no new L2 quality training was run;
- no L3/L4 behavior was promoted;
- the pre-existing environment-sensitive tests for `звгрузи` and IME
  transposition authority remain separate test debt; the same failure was
  reproduced from baseline commit `a5188a5`.

Verdict: `PASS_canonical_live_owner`.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_LIVE_OWNER_CUTOVER_2026-08-01.json`

Runtime authority changed: `false`.

## 1. Why This Document Exists

The current live route is now a mixed live-owner flip:

```text
deterministic candidates
+ L2FieldShadow local field
+ internalized L1.1 seeded birth
+ shadow same-lemma morphology donor
+ shadow near-neighbor donor
-> one correction lattice
```

That is good enough for continued development, but it is not a clean final
architecture. Right now:

- `/home/ubu/projects/lay/src/nanda_wave/lexical_grokking/restoration.rs`
  owns true `L1.1` lexical restoration readout;
- `/home/ubu/projects/lay/src/correction_core/candidate_sources.rs`
  still owns the live candidate merge, but the live route now resolves through
  `CandidateReadoutRoute::L2FieldShadow`;
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
   -> shadow_text_candidates()
      -> bounded lexical birth
      -> internalized L1.1 seeded birth
      -> same-lemma donor
      -> near-neighbor donor
      -> one local readout
-> unified candidate lattice
-> TransitionDecisionCore
-> verifier
```

Important consequences:

1. `L1.1` is already real as bounded lexical restoration evidence, but it is
   not yet a standalone fully packaged lexical owner.
2. `L2FieldShadow` is now the live candidate-field route for local IME/daemon
   correction.
3. Morphology and transition-phase learning exist, but they are still
   side-teachers rather than the canonical owner above `L1.1`.

This document defines how that mixed route must close into one real `L2`.

Implementation status on 2026-07-26:

- `CandidateReadoutRoute::live_default()` now resolves to
  `CandidateReadoutRoute::L2FieldShadow`;
- `CandidateReadoutRoute::compare_reference()` now resolves to
  `CandidateReadoutRoute::FullWave`;
- the new route lives under
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/`;
- `L2FieldShadow` no longer requires an injected
  `L2CorrectionPeakContext` for candidate birth;
- `L2FieldShadow` now self-prepares its local lexical / boundary contour
  directly from the input text;
- `L2FieldShadow` now internalizes bounded `L1.1` restore surfaces into that
  same local field instead of emitting a separate shadow-side `L1.1` sidecar
  candidate;
- `L2FieldShadow` now also contains one narrow internal morphology donor for
  same-lemma / morphology-slot competition;
- that donor is shadow-only, limited to already-born Cyrillic local surface
  candidates, and only activates when exactly one same-lemma cohort exists
  inside the bounded shadow field;
- the donor is backed by the existing 462k-form morphology package through
  `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/runtime.rs`;
- short low-entropy tokens of length `<= 3` currently bypass `L1.1` seeded
  birth and stay on the plain lexical field to preserve abstain parity on
  ambiguous local signals;
- IME boundary-owned Space/autocorrect mutations now surface from
  `L2FieldShadowBoundary` on the live route rather than `BoundaryCell32`;
- the local donor winner multiplier is now explicit as
  `SHADOW_DONOR_WINNER_WEIGHT = 5` in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- it is still donor-based and currently reuses the existing lexical-phase,
  boundary, layout, and `L1.1` donor packages rather than a standalone
  canonical `L2` package;
- the public CLI candidate-route surface now exposes only
  `l2-field-shadow` and `full-wave`;
- no `CompactL2`, `compact-l2`, `uses_peak_context`, or
  `L2LexicalPhaseCell32` matches remain in `src/`, `tests/`, or `src/bin/`;
- runtime authority changed for the live local route, but not for standalone
  package promotion.

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib correction_core`: failed
  (`95 passed / 14 failed`);
- `scripts/cargo-guard.sh check --bin lay`: passed;
- `scripts/cargo-guard.sh check --bin lay-nanda-wave-eval`: passed;
- `scripts/cargo-guard.sh check --bin lay-daemon`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_uses_shadow_surface_source_ids`:
  passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_self_prepares_l11_candidate_without_peak_context`:
  passed;
- `scripts/cargo-guard.sh test --lib live_l2_field_shadow_route_births_nanda_candidates_without_full_wave_authority`:
  passed;
- `scripts/cargo-guard.sh test --lib hidden_state_blocks_live_known_form_drifts_from_logs`:
  passed;
- `scripts/cargo-guard.sh test --lib ambiguous_known_to_known_swap_requires_relation_proof`:
  passed;
- `scripts/cargo-guard.sh test --lib gate_authorizes_same_transition_behind_unchanged_right_context`:
  passed;
- `scripts/cargo-guard.sh run --bin lay -- --help`: shows
  `--candidate-route <l2-field-shadow|full-wave>`;
- `scripts/cargo-guard.sh run --bin lay -- --help`: shows
  `--compare-candidate-routes`;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2939`,
  `records_used = 134`,
  `surface_diverged = 18 / 134`,
  `gate_diverged = 18 / 134`,
  `provenance_diverged = 32 / 134`,
  `reference_apply = 27 / 134`,
  `shadow_apply = 29 / 134`,
  `user_target_match.reference = 7 / 134`,
  `user_target_match.shadow = 8 / 134`,
  `user_target_match.both = 5 / 134`.

What was not tested in this step:

- fixed heldout `L2` proof;
- live IME/daemon authority flip;
- latency, RSS, and cold-load budget of a real standalone `L2` package.
- formal batch-time / RSS receipt for the self-owned replay path;
- resolution of the 14 broader `correction_core` failures from the broad lib
  run.

Verdict scope:

- the new route compiles and is wired as the only executable live local-field
  owner contour;
- `L2FieldShadow` now owns its own candidate-birth input contour instead of
  consuming a prebuilt legacy lexical route;
- the old `CompactL2` route and `L2LexicalPhaseCell32` source path are removed
  from executable/public route selection;
- on the measured 134 real correction-log inputs, the current live owner no
  longer has full selected-surface or selected-gate parity with `FullWave`;
- this is not yet evidence of a finished standalone canonical `L2` package.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_LIVE_OWNER_IME_DAEMON_GATE_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_LEGACY_ROUTE_REMOVAL_2026-07-26.json`

Runtime authority changed:

- `true`

Historical note:

- remaining `CompactL2` and `L2LexicalPhaseCell32` mentions below refer only
  to earlier compare baselines and receipts; they do not describe current
  executable route selection.

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

## 17. 2026-08-06 Nonblocking Layout Handoff After Autocorrection

The live log exposed a second synchronous owner on the physical Space route
after the `DecisionCore` prefetch work had already been moved off that route.
The observed sequence was:

```text
Tcnm
-> autocorrection commits "Есть "
-> process-level switch to lay-ime-ru blocks
-> switch command times out after 204 ms, ok=false
-> the switch completes later
-> the first key of the next word is decoded under the old layout
-> nакой
-> the next Space repairs it to "такой "
```

Measured pre-fix facts from
`/home/ubu/.local/share/lay/ibus_engine_debug.jsonl`:

```text
prefetch for Tcnm                         486 us
CommitText for "Есть "                    49 us
replacement state                    204,383 us
replacement total                    204,435 us
physical Space total                 204,588 us
ibus_layout_sync target=ru            ok=false
```

The `1.0.13` runtime contract is now:

```text
authorized layout autocorrection
-> commit corrected surface and one Space
-> immediately set this LayIbusEngine decoder to the target layout
-> publish committed-tail handoff
-> schedule one latest-only background process-level IBus switch
-> return from physical Space
```

The background state is bounded to one worker and one replaceable desired
request. A newer desired layout replaces a request that has not started yet.
The worker emits the final `ibus_layout_sync` result, while the hot path emits
`ibus_layout_sync_requested`. The external IBus command and its timeout no
longer belong to autocorrection's physical Space latency.

Manual double-Shift remains on the blocking layout synchronization route. That
operation explicitly asks for a completed user-visible layout transition and
is not part of this Space-only ownership change.

What was tested:

- release compilation of `lay`, `lay-daemon`, and `lay-ibus-engine`;
- installation of Lay `1.0.13` and GNOME extension runtime `1.0.13`;
- restart of only `lay-daemon` and `lay-ibus-engine`;
- global `ibus-daemon` retained PID `3702`.

What was not tested:

- post-install physical GUI Space latency percentiles;
- a repeated live `Tcnm -> Есть такой` interaction after installation;
- quality impact on the fixed L1.1 or L2 heldout proofs.

Verdict scope:

- the measured `204.588 ms` is a pre-fix fact and is not presented as a
  post-fix result;
- code ownership and the installed runtime changed so that autocorrection no
  longer waits for the process-level IBus switch;
- live behavioral confirmation remains pending user typing.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_AUTOCORRECT_LAYOUT_HANDOFF_NONBLOCKING_2026-08-06.json
```

Runtime authority changed:

- `true`

## 18. 2026-08-08 Shared L3 Scene On The IME Preedit Path

The live IME trace exposed a latency outlier while constructing a display-only
completion:

```text
token                         сдела
returned candidates               8
full precognition             83,652 us
L2 material                     889 us
L3 context                   82,290 us
DecisionCore                     27 us
visible suffix                     ть
```

This was not a Space stall and did not apply an edit. It blocked the printable
key path while L3 constructed the preedit candidate field.

The redundant computation was inside
`ContextPhasePackage::score_candidates_with_mode_and_pair_views`. Before
`1.0.14`, one batch built the same context scene once for the batch and then
rebuilt it again inside `candidate_relation_vector` for every candidate:

```text
old: context scene builds = 1 + frontier size
new: context scene builds = 1
```

The live trace records eight returned candidates, but did not record the raw L3
frontier used before final admission. Therefore this experiment does not claim
an exact old scene-build count for that sample. For any raw frontier size `N`,
the count moves from `1 + N` to `1`. Each candidate still clones that scene and
adds its own semantic relation vector before the existing positive, anti,
signature, pairwise, and DecisionCore readout.

The optimization is result-preserving:

- no L2 or L3 candidate limit changed;
- no positive, anti, hard-negative, signature, semantic, or pairwise bank was
  disabled;
- no score, threshold, authority, `Tied`, or `ABSTAIN` rule changed;
- only repeated construction of an identical intermediate vector was removed.

Measured facts:

- pre-fix live outlier: `L3 = 82.290 ms`, total preedit `83.652 ms`;
- post-change debug hot readout over 1,200 iterations:
  `p99 = 1.812 ms`, `max = 1.943 ms`, debug gate `<=5 ms` passed;
- release hot context-phase readout over 1,200 iterations:
  pre-change `p99 = 165 us`, `max = 664 us`;
  post-change `p99 = 164 us`, `max = 182 us`;
- release full sentence readout with 14 pair views and 12 candidates over 1,200
  iterations: `p50 = 444 us`, `p99 = 628 us`, `max = 688 us`;
- immediately after installing `1.0.14`, while graphify and build work were
  still running, the GUI trace still contained L3 outliers: `91.933 ms` for
  token `с` and `64.348 ms` for token `служ`; these are post-fix observations,
  so the physical GUI latency gate remains open even though the isolated full
  sentence readout stays below `1 ms`;
- the wider unique-prefix candidate-gate test remains above its historical
  `1.5 ms` release budget: observed maximum `6.323 ms`, with L2 material up to
  `4.500 ms` and L3 context up to `1.614 ms`; this experiment does not declare
  the complete preedit latency gate closed;
- context-phase behavioral suite: `83/83 PASS` on 19 test threads.

What was not yet measured at the time of this architecture entry:

- post-install physical GUI p50/p95/p99 under an idle development workload;
- recurrence rate of scheduler or page-fault outliers during multi-day input;
- fixed L1.1 restoration proof, which is outside this result-preserving L3
  intermediate-vector change.

Verdict scope:

- the identified duplicate L3 scene construction is removed;
- the context-phase maximum improved in the focused release measurement, while
  the wider candidate gate still fails its existing latency budget;
- post-install loaded-system telemetry still has outliers and prevents a live
  PASS claim;
- verdict is `WATCH`: clean physical typing telemetry remains the final latency
  confirmation.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_L3_SHARED_SCENE_PREEDIT_2026-08-08.json
```

Runtime authority changed:

- `false`

## 14. 2026-08-06 Short-Function Boundary Shift And Space Timing

### Observed Input Shape

The live input

```text
какие документ ыим
```

is not a missing committed Space. The physical key sequence placed Space before
the final `ы`, so the committed tail contained two surfaces:

```text
документ | ыим
```

The existing `moved_prefix_pair` producer correctly emitted the structural
boundary-shift candidate:

```text
какие документ ыим
-> какие документы им
```

The candidate was rejected later by
`boundary_shift_unstable_token_mass`, because the structural veto required both
result tokens to contain at least four characters.

### Canonical Structural Gate

A boundary shift still cannot change letters. It may only redistribute the
existing tail characters across the last two token boundaries. Each resulting
token must have independent lexical support.

For a token shorter than four characters the additional contract is:

```text
length >= 2
AND known Russian phrase part
AND known short Russian function word
AND exact surface phase center
```

This is a class-level rule, not a word-specific exception. It admits supported
short pronouns and function words while keeping arbitrary short fragments
blocked.

### Space Hot-Path Measurement

The new `ibus_space_key_timing` and `ibus_space_autocorrect_timing` events split
the physical Space route into setup, DecisionCore, replacement and commit time.
The live trace for `склееватся` measured:

```text
Space total                 217876 us
autocorrect DecisionCore    217769 us
Space commit                    67 us
status                  no_decision
```

Additional live outliers reached `224644 us` and `408007 us`. Therefore the
remaining freeze owner is the synchronous committed-token DecisionCore call on
the IBus Space hot path. Boundary commit and replacement are not the dominant
cost. Version `1.0.11` adds exact telemetry and the short-function
boundary-shift admission, but does not claim that the Space latency gate has
passed.

### Evidence Scope

What was tested or measured:

- the live physical sequence and committed-tail surfaces were read from
  `/home/ubu/.local/share/lay/ibus_engine_debug.jsonl`;
- `moved_prefix_pair` produced the expected boundary-shift candidate in the
  diagnostic route;
- release binaries `1.0.11` were built, installed and the Lay runtime was
  restarted without restarting the global `ibus-daemon`;
- global `ibus-daemon` PID remained `3702` during installation.

What was not tested:

- post-install physical GUI confirmation of `документ ыим -> документы им`;
- latency p50/p95/p99 after removing synchronous DecisionCore work from Space;
- fixed heldout L1.1 or L2 quality proof;
- wider boundary-shift corpus coverage.

Verdict scope:

- `1.0.11` is installed with the generalized short-function boundary-shift
  gate;
- physical behavior remains user-verification pending;
- Space latency remains a measured open defect, not a PASS.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_SPACE_BOUNDARY_SHIFT_TIMING_2026-08-06.json`

Runtime authority changed:

- `true`, limited to boundary-shift candidates whose short side satisfies all
  independent lexical and exact-phase checks above.

## 15. 2026-08-06 Nonblocking Space Autocorrect Prefetch

### Rejected Runtime Shape

The `1.0.11` live trace proved that the physical Space handler synchronously
owned the complete correction calculation:

```text
Space key
-> DecisionCore, up to 249579 us in the observed post-install trace
-> commit Space, 690 us
```

This ordering is forbidden. A correction calculation may be expensive, but it
must never delay or suppress the user's physical word boundary.

### Canonical Runtime Shape

Version `1.0.12` uses one process-wide latest-only prefetch worker:

```text
printable committed character
-> publish exact (engine path, tail epoch, tail, layout) key
-> background DecisionCore calculation

physical Space
-> exact completed key available: consume its decision
-> missing, pending or stale key: commit Space immediately
```

The worker stores at most one desired request and one completed result. New
input replaces pending desired work. A result is published only if its
generation is still current. The Space route accepts a result only when engine
path, tail epoch, complete committed tail and active layout all match.

Therefore:

- Space contains no synchronous DecisionCore call;
- stale correction output cannot be applied to newer text;
- a calculation that is not ready may skip that one autocorrection, but cannot
  delay or consume the physical Space;
- the existing `AuthorizedEdit`, structural verifier and exact one-trailing-
  Space contract still own any prefetched correction that is applied.

### Evidence Scope

Measured input fact that caused the change:

```text
post-1.0.11 Space total       250388 us
autocorrect DecisionCore      249579 us
Space commit                     690 us
```

What was verified in this step:

- release compilation of `lay`, `lay-daemon` and `lay-ibus-engine` succeeded;
- installed CLI and GNOME extension report `1.0.12`;
- `lay-daemon` and `lay-ibus-engine` restarted;
- global `ibus-daemon` PID remained `3702`;
- the executable Space path no longer calls
  `decide_active_composition_autocorrect` synchronously.

What was not tested:

- physical GUI latency distribution after installation;
- prefetched correction hit rate during real typing;
- correction quality impact when Space arrives before prefetch completion;
- fixed heldout L1.1/L2 proof.

Verdict scope:

- architectural blocking owner removed from the Space hot path;
- installed runtime awaits physical user verification;
- quality and latency gates are not promoted until live telemetry supplies
  denominators.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_SPACE_NONBLOCKING_PREFETCH_2026-08-06.json`

Runtime authority changed:

- `true`; a not-ready prefetch now fails open for the physical Space and closed
  for the autocorrection.

## 13. 2026-08-05 Atomic Space And Nonblocking L3 Refresh

### Candidate Birth And Blocking Points

```text
physical Space
-> committed-token autocorrect decision
-> AuthorizedEdit replacement
-> exactly one committed word boundary

next printable key
-> bounded L2 candidate birth
-> read current immutable L3 composite
-> live candidate readout
```

Two coupled runtime defects were observed in the same typing sequence:

- an autocorrect replacement and its triggering physical Space did not have an
  explicit executor-level contract requiring exactly one trailing boundary;
- `with_default_memory()` synchronously loaded a changed L3 composite manifest
  on the hot preedit thread before scoring the next token.

The canonical runtime contract is now:

- a successful Space autocorrect must carry exactly one trailing ASCII Space in
  the authorized replacement; an invalid boundary fails closed and the managed
  route commits the physical Space normally;
- manifest polling may detect a new L3 generation on the readout path, but one
  bounded background worker owns package loading;
- live readout continues against the previous immutable `Arc<L3CompositeMemory>`
  while the worker loads the new generation;
- the worker swaps the ready composite under the write lock; candidate scoring,
  L3 weights and text-edit authority do not change.

### Measured Facts

- live pre-fix trace for token `ош`: total `777948 us`, L2 material `2358 us`,
  L3 context `775051 us`;
- additional live pre-fix examples included `ту`: L3 `83588 us`, and `пу`: L3
  `96816 us`;
- post-change debug cache-miss probe over six distinct prefixes: maximum total
  `34338 us`, maximum L3 stage `11998 us`;
- committed-tail focused tests: `8/8 PASS`;
- one-Space autocorrect sequence tests: `3/3 PASS`.

### Scope And Gate

What was not tested at this point:

- post-install physical GUI p50/p99 under a newly admitted online L3 delta;
- application-specific surrounding-text behavior in every GTK, Chromium and
  WeChat surface;
- full L1.1 thirteen-class restoration proof.

The wider `ime_correction::tests` gate is not green in the current checkout:
sequential execution produced `17 PASS / 14 FAIL`. The failures include stale
source-owner expectations (`personal_phrase` versus `glued_phrase`) and missing
live decisions. They are recorded as a separate existing gate and are not
reported as proof of this focused executor/latency change.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_ATOMIC_SPACE_NONBLOCKING_L3_2026-08-05.json`

Runtime authority changed:

- `false`

## 14. 2026-08-05 Bounded Typo Plus Boundary Repair

### What was tested

- live canonical route for
  `Готовь докуентыдля -> Готовь документы для`;
- `BoundaryCell32` candidate birth for a damaged glued current token;
- proposal admission through the existing
  `current_token_boundary_split_or_repair` contract;
- preservation of the known-word split guards for `уровне` and
  `на уровне`.

### Measured facts

- before the change, both `full-wave` and `canonical-l2-field` produced no
  applicable candidate for `Готовь докуентыдля`;
- the deterministic route could describe `Готовь документы для`, but it was
  `SuggestOnly/boundary_operator_changes_surface`;
- the live canonical route now has `17` candidates: `1` applicable and `16`
  suggest-only;
- the selected candidate is `Готовь документы для` from
  `Nanda:CanonicalL2FieldBoundary`;
- the selected gate is `Eligible/class_allows_apply`;
- the edit is verified as a bounded current-token `GluedWords` operation;
- no phrase-specific replacement table was added.

### General contract

```text
damaged current token
-> BoundaryCell32 proposes known lexical parts
-> current_token_boundary_split_or_repair
   requires unchanged left context
   requires one damaged current token
   requires one or two added word boundaries
   requires known replacement parts
   rejects an already-known original token
   requires Damerau-Levenshtein distance <= 2
-> BoundaryMergeSplit verifier
-> common L2 readout
```

### What was not tested

- broad glued-token recall and false-split percentages;
- physical GUI behavior outside the installed IME probe;
- the fixed L1.1 thirteen-damage-class heldout proof.

### Verdict scope

`PASS_targeted`: the canonical live L2 owner can apply a verified typo repair
and boundary split on the current token. This is not a broad boundary-quality
claim.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_BOUNDED_TYPO_BOUNDARY_REPAIR_2026-08-05.json`

Runtime authority changed:

- `true`, limited to candidates already proved by
  `current_token_boundary_split_or_repair`

## 13. 2026-08-04 Standalone L2 First-Space Warmup

### What was tested

- installed `LAY-L2-RU-FULL-v13.bin` first touch through
  `lay --explain-correct "ЕланаПросит "` with `LAY_L2_FIELD_TRACE=1`;
- a second canonical L2 readout in the same process;
- the existing verified boundary case `Еленапросит -> Елена просит`;
- focused boundary and correction-core tests plus `scripts/check-lay-changed.sh`;
- installation and restart of only the managed Lay runtime processes.

### Measured facts

- installed standalone L2 package: `135,121,803` bytes (`128.86 MiB`);
- cold standalone field load/readout: `379.144 ms`;
- second standalone field readout in the same process: `1.297 ms`;
- second complete canonical L2 materialization: `8.116 ms`;
- the old cold load happened synchronously on the first boundary readout and
  was therefore visible as a pause after Space;
- `warm_up_l2_for_ime()` now loads and indexes the standalone L2 field on its
  existing background warmup thread before candidate memory is published as
  ready;
- candidate birth, scoring, boundary authority, package format, and package
  contents did not change;
- installed Lay runtime PIDs changed from daemon `1853387` / engine `1853423`
  to daemon `1938013` / engine `1938039`;
- global `ibus-daemon` remained PID `3702`.

### What was not tested

- a broad latency distribution across physical GUI applications;
- cold startup on hardware without a warm Linux page cache;
- broad glued-word recall or false-split rate;
- the fixed L1.1 thirteen-damage-class heldout proof.

### Verdict scope

`PASS_targeted`: the measured `~379 ms` package first touch was moved out of
the first-Space hot path into background IME warmup. The measured post-load
canonical L2 path remains single-digit milliseconds for this probe. This is a
latency lifecycle result, not a broad quality claim.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_FIRST_SPACE_BACKGROUND_WARMUP_2026-08-04.json`

Runtime authority changed:

- `false`

## 13. 2026-08-04 Two-Content Glued-Word Boundary Birth

Canonical route added by this experiment:

```text
one glued Cyrillic token
-> enumerate internal boundaries
-> require independent left and right lexical/form centers
-> require at least one strong L2 surface center
-> reserve at most 2 Boundary candidates in the canonical L2 field
-> common L2/L3 lattice
-> TransitionDecisionCore and verifier
-> Winner | Tied | Abstain
```

The generic two-content route requires at least `4 + 4` characters. The
earlier `3 + 3` experiment was rejected because it admitted the false split
`поспорта -> пос порта`.

Clean whole-surface authority is conjunctive safety evidence. If the original
token already has a known Russian word/form center, generic two-content birth
is suppressed. A known whole surface may be split only when the existing
contextual boundary operator independently confirms the same replacement.
This preserves contextual `у насесть -> у нас есть` without allowing
`улетели -> улет если`.

Measured facts:

- `Еленапросит -> Елена просит` is selected by the live correction core;
- source is `CanonicalL2FieldBoundary`;
- class is `GluedWords`;
- gate is `Eligible/class_allows_apply`;
- the explain readout contained `13` candidates: `1` applicable Boundary
  candidate and `12` one-word candidates retained as `SuggestOnly`;
- boundary reserve is `2` candidates;
- L2 unit birth, canonical bridge reserve, live correction-core selection,
  known-whole preservation, multi-letter-preposition safety, and contextual
  known-glue preservation passed in focused sequential tests.

What was not tested:

- broad heldout glued-word recall and false-split rate;
- latency distribution under a live typing workload;
- physical application through the installed IBus engine;
- the fixed L1.1 thirteen-damage-class proof, which is not a boundary proof.

Known separate boundary debt:

- the pre-existing reverse operation `тако й -> такой` currently fails to
  birth in `boundary_scan_candidates`; this two-content glued-token experiment
  does not claim to fix two-token merge recovery.

Verdict scope:

- `PASS_targeted` for generic two-content glued-token birth and live canonical
  L2 selection;
- broad boundary quality is `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed: `true` in release `1.0.7`;
- global `ibus-daemon` PID stayed `3702` during the managed runtime restart.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_TWO_CONTENT_GLUED_BOUNDARY_2026-08-04.json`.

## 14. 2026-08-04 Class-Conditioned Sparse-Omission Reserve

The canonical live bridge keeps a bounded class-conditioned reserve instead of
using one undifferentiated top-N cut:

```text
L1.1 lattice seeds: 16
-> general L2 material frontier: 8
-> sparse-internal-multi-omission reserve: at most 2 additional surfaces
-> one canonical L2 local field
-> L3 and common admission
```

This reserve changes candidate retention only. It does not mint Winner
authority. Reserved candidates still obey the L2 local verdict and remain
`SuggestOnly` under `Tied` or `Abstain`.

Measured facts for the live-log case:

- input: `на компанию Хунлу можем подврдить `;
- L1.1 had `подтвердить` as seed `16/16`, score `1813`;
- the former general frontier retained `8` candidates and discarded that seed;
- after the reserve, correction-core candidate count changed from `10` to
  `11`;
- `подтвердить` now reaches the common lattice as
  `SparseInternalMultiOmission`;
- both `подтвердить` and the competing `подводить` remain `SuggestOnly` under
  `canonical_l2_field_local_tie`;
- final action remains `keep`, so the change fixes candidate visibility without
  reintroducing the observed false autocorrection to `подводить`.

What was tested:

- focused canonical bridge retention test for a sparse omission below the
  general frontier;
- source explain for the exact live-log phrase;
- two existing sparse-omission correction-core contracts passed.

What was not tested:

- fixed heldout sparse multi-omission percentages;
- broad false-candidate cost of the two-slot reserve;
- sentence continuation replay after the following word becomes available;
- physical installed IME behavior at receipt creation.

Known separate failure:

- the existing `переподлчаю -> переподключаю` authority test currently births
  the expected candidate but selects no transition. This is an L2 authority
  baseline failure, not evidence that the new reserve regressed candidate
  retention.

Verdict scope:

- `PASS_targeted` for class-conditioned candidate retention;
- automatic semantic restoration is `NOT_CLAIMED`;
- broad sparse-omission quality is `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed: `true` in release `1.0.7`;
- installed hot readout retains `подтвердить` and returns `Tied/ABSTAIN` with
  no selected transition;
- global `ibus-daemon` PID stayed `3702` during the managed runtime restart.

Cold fail-closed follow-up:

The first request immediately after a managed restart exposed a separate
authority leak. When the `12 ms` L1.1 socket request timed out, L2 inverse
lookup could still birth `подводить` and promote it as a lexical winner despite
having no L1.1 seeds. The canonical ownership contract is now explicit:

```text
no confirmed L1.1 seeds
-> no standalone L2 lexical field
-> no inverse-only Winner authority
-> keep / ABSTAIN
```

This is a general fail-closed rule. It does not special-case `подврдить` or
`подтвердить`; it prevents every cold, unavailable, or timed-out L1.1 request
from being replaced by autonomous L2 lexical authority.

Installed verification in release `1.0.7`:

```text
release SHA-256              3387bfc4f4716853ee632868d4866d35d833fdbb745a8e1abd4fa3b3d57c29e4
cold first Nanda candidates  0
cold first Nanda selection   none
hot Nanda candidates         11
hot target                   подтвердить / SuggestOnly
hot wrong competitor         подводить / SuggestOnly
hot selection                none
glued-word regression        Еленапросит -> Елена просит
lay-daemon PID               1830167
lay-ibus-engine PID          1830194
lay-l1.1-serve PID           1830227
global ibus-daemon PID       3702 -> 3702
```

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_SPARSE_OMISSION_RESERVE_2026-08-04.json`.

## 15. 2026-08-04 Reference-Backed Short-Participle Ambiguity

Observed live failure:

```text
input                       подлючен
wrong live winner           подлечен
expected visible candidate  подключен
```

The installed L1.1 package has no `подключен` or `подключён` surface. Its
top-16 field contains noun forms such as `подключение`, while L2 one-edit
inverse lookup independently births the valid but contextually wrong
`подлечен`. System Hunspell confirms that both `подключен` and `подлечен` are
real short passive participles, so deleting or globally suppressing either
surface would be incorrect.

Canonical route added by this experiment:

```text
one-letter omission geometry
-> derive bounded candidate surfaces
-> require an explicitly attested long participle in the reference lexicon
   (for example подключенный -> подключен)
-> reserve at most 2 reference-backed short forms without authority
-> combine with the ordinary L1.1/L2 cohort
-> unresolved equal-distance forms force Tied/ABSTAIN
-> sentence context may resolve them later
```

Measured source facts:

```text
candidate count before      15
candidate count after       16
подключен                   missing-letter / SuggestOnly
подлечен                    letter-substitution / SuggestOnly
local verdict               Tied
selected transition         none
```

Safety reasoning:

- no surface string is hardcoded;
- candidate birth requires exact long-form reference evidence;
- the reference donor cannot grant Winner authority;
- the rule only preserves one-edit ambiguity and therefore cannot rewrite an
  unrelated token;
- a real sentence-level context remains responsible for choosing between two
  valid meanings.

What was tested:

- long-form backing for masculine and inflected short participles;
- rejection of a fabricated unbacked short form;
- exact source explain for `подлючен`;
- both valid candidates survive as `SuggestOnly` and no transition is selected.

What was not tested:

- broad short-participle recall and false-ambiguity rate;
- sentence contexts that should resolve `подключен` versus `подлечен`;
- fixed L1.1 thirteen-class restoration proof, because the package is unchanged.

Verdict scope:

- `PASS_targeted_source` for preventing the false singleton;
- automatic semantic restoration is `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed: `true` in release `1.0.7`.

Installed verification:

```text
release SHA-256             5a53ef90a1e47007176a13e41b2c241db85eb7bb60db6ecd1b621d7cd791178f
hot candidate count        16
подключен                  missing-letter / SuggestOnly
подлечен                    letter-substitution / SuggestOnly
selected transition        none
glued-word regression      Еленапросит -> Елена просит
sparse reserve regression  подтвердить retained; selected none
lay-daemon PID             1853387
lay-ibus-engine PID        1853423
lay-l1.1-serve PID         1853447
global ibus-daemon PID     3702 -> 3702
```

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_REFERENCE_BACKED_SHORT_PARTICIPLE_AMBIGUITY_2026-08-04.json`.

## 13. 2026-08-04 Internal Layout-Key Projection Contract

Observed live failure:

```text
typed surface                 ye;ty
exact physical projection    нужен
live Space result             unchanged
```

The IME trace proved that all five characters, including `;`, remained in the
committed-tail field. The failure was therefore not character loss or a split
replacement range. The final source-level root cause was a second settlement:
the layout lane first proved the exact known projection `ye;ty -> нужен`, then
context-free L2 morphology moved that result to the same-lemma neighbour
`нужна`. The verifier correctly abstained when `нужен` and `нужна` conflicted.

Canonical rule:

```text
ASCII token with internal layout-letter key
-> exact full-token keyboard projection
-> known opposite-layout word/form
-> keep exact layout candidate eligible
-> exact projection is lexical authority
-> L2 morphology may settle only unknown/noisy projections
```

This is class-based, not a word exception. The internal-key set is the existing
layout alphabet (`;`, `[`, `]`, `,`, `.`, `'`, and their shifted variants).
Known English words and technical surfaces such as `pdf`, URLs, CLI options and
brand tokens retain their protection.

What was tested:

- `ye;ty -> нужен` through the committed-tail manual-toggle planner:
  `5` deleted characters, exact replacement `нужен`;
- `ye;ty -> нужен ` through the live Space decision with active English layout;
- `pdf` remains unchanged with active English layout;
- the candidate constructor retains exact `нужен` instead of settling it to
  `нужна`;
- debug explain emits one accepted layout candidate, `нужен`, and no `нужна`
  competitor;
- all focused tests passed when run independently through
  `scripts/cargo-guard.sh`.

Measured facts:

```text
exact projection tests       4/4 PASS
manual delete span           5 characters
false protected pdf apply    0
accepted layout candidates   1
morphology competitors       0
debug output                 нужен
```

What was not tested at this point:

- aggregate L1.1 heldout quality, because this change does not alter the L1.1
  package or its readout.

Installed runtime facts:

```text
release                         lay 1.0.7
installed explain              ye;ty -> нужен
confidence                     SingleCandidate
second candidate               none
lay-daemon.service             active
active engine                  lay-ime-ru
global ibus-daemon PID         3702 -> 3702
changed-file gate              PASS
```

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_INTERNAL_LAYOUT_KEY_PROJECTION_2026-08-04.json`

Runtime authority changed at this documentation point:

- `true`; release `1.0.7` was installed and the managed Lay runtime was
  restarted without restarting the global IBus daemon.

## 14. 2026-08-04 Typo-Tolerant IME Completion Lane

What was tested:

- Russian IME completion after one insertion, deletion, substitution, or
  adjacent-transposition error in an unfinished prefix;
- the observed `переспектив...` family;
- separation from same-size full-token repair, which remains owned by the
  Space/autocorrect route;
- IME rendering as a full-token replacement accepted only by explicit `Tab`;
- hot-path latency and the existing IME latency budget suite.

Measured facts:

```text
damaged prefix                 переспектив
returned family candidates    12
examples                      перспективный, перспективна,
                              перспективно, перспективней
cold targeted readout         7,867 us
hot cache readout                  6 us
existing IME latency suite    p50 26 us / p90 36 us / p99 46 us / max 62 us
```

The corrected-prefix lane starts at `7` Cyrillic characters, admits at most
`2` corrected prefix basins, and reserves at most `8` L2 candidates plus one
final display slot. Exact-prefix candidates are retained. Early ambiguous
states such as `пересп` are not forced to `перспективнее`, because real
`переспать...` and `преспокойных...` basins still compete there.

What was not tested:

- aggregate IME hit-rate over a fixed heldout typo-prefix corpus;
- physical `Tab` acceptance after installing the new binary;
- typo-tolerant ASCII completion;
- full L1.1 13-class restoration proof, because package data and boundary
  restoration were not changed.

Verdict scope:

- targeted corrected-prefix family coverage: `PASS_targeted`;
- existing hot latency suite: `PASS`;
- broad IME quality promotion: `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed at this checkpoint: `true` (`lay 1.0.6`).

Installed runtime facts:

```text
global ibus-daemon PID       3702 -> 3702
managed engine PID        432941 -> 464498
lay-daemon PID            432906 -> 464453
active engine                       lay-ime-ru
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_IME_TYPO_TOLERANT_COMPLETION_2026-08-04.json
```

## 13. 2026-08-04 Single-Edit Inverse Lane And Tied Readout

What was tested:

- canonical standalone L2 restoration for `переспективнее`, `отвликайся`,
  `переделаем`, and the ambiguous observed surface `наденный`;
- package-indexed inverse lookup for every one-step Damerau operation:
  insertion, deletion, substitution, and adjacent transposition;
- tied-cohort authority for length-changing versus shape-preserving repairs;
- preservation of valid Russian verb forms through reusable ending relations,
  without word-specific runtime rules.

Measured facts:

```text
переспективнее -> перспективнее
отвликайся     -> отвлекайся
переделаем     -> переделаем
наденный       -> наденный
```

The inverse lane remains bounded to `16` package form references and performs
direct package index lookups. It does not scan the complete L2 field. When the
L2 readout remains tied, insertion/deletion candidates are `SuggestOnly`;
substitution/transposition may retain independently verified authority.

What was not tested:

- fixed heldout percentages for all L1.1 damage classes;
- full-corpus L2 recompilation or package-format changes;
- weak IME preedit coverage beyond the four direct smoke probes;
- physical typing after installation.

Verdict scope:

- bounded single-edit inverse lane: `PASS_targeted`;
- observed false-authority containment for `наденный`: `PASS_targeted`;
- broad language-quality promotion: `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed at this checkpoint: `true` (`lay 1.0.5`).

Installed smoke facts:

```text
global ibus-daemon PID       3702 -> 3702
managed engine PID        4002343 -> 432941
lay-daemon PID            4002297 -> 432906
active engine                       lay-ime-us
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_SINGLE_EDIT_INVERSE_AND_FEEDBACK_SANITATION_2026-08-04.json
```

## 13. 2026-08-03 Inverse Length Birth and Tied-Cohort Authority

### Tested change

The standalone L2 runtime now performs a bounded inverse lookup for forms that
are exactly one insertion or one deletion from a damaged token:

```text
damaged token
-> bounded one-length-edit variants
-> binary search in the existing sorted DecoderGraph
-> at most 16 additional lexical seeds
-> existing L2 context/competition readout
-> Winner | Tied | Abstain
-> existing transition verifier
```

This is candidate birth only. It does not scan the field, add word-specific
rules, recompile V13, or grant apply authority by itself. Exact one-edit forms
enter with the same lexical energy as the strongest L1 seed because they are
alternative explanations of the same damaged signal. Learned context and
competition remain responsible for separating them.

`L2FieldAuthority::Tied` now carries the tied surfaces. A tie cannot promote an
L2 surface, but it also cannot veto an independently verified candidate that is
already a member of that tied cohort. Foreign candidates are still demoted to
`SuggestOnly`.

### Measured facts

- synthetic package inverse lookup:
  `окное -> [окне, окно]`,
  `перхвачу -> [перехвачу]`,
  clean `окне -> []`;
- focused standalone L2 tests: `29/29 PASS`;
- focused IME regression:
  `клавиатурой не перхвачу -> клавиатурой не перехвачу`, `1/1 PASS`;
- installed V13 readout for `перхвачу`:
  `Tied(первачу=2038, перехвачу=2038)`;
- final correction-core selection for that case:
  verified deterministic `missing_letter -> перехвачу`;
- installed V13 readout for `у меня в окное`:
  `Abstain`; the lattice contains `окне` and `окно`, but neither receives
  authority;
- debug-process timing after initialization was about `36 ms` for the
  `перхвачу` probe. This is not a release latency measurement.

### Rejected experiment

An exact-key suffix backoff
`у меня в _ -> меня в _ -> в _` was tested against installed V13. It produced
no non-zero slot, neighbor, or competition evidence for `окное`, so it was
removed from the code. Verdict: `NO_EFFECT_NOT_RETAINED`.

### Not tested

- fixed heldout per-error-class percentages;
- release hot p50/p99 after installation;
- physical WeChat typing after binary replacement;
- a trained L2/L3 contextual winner for `у меня в окное -> у меня в окне`;
- full IME module parity, whose current environment-dependent baseline still
  has unrelated pre-existing failures.

### Verdict and authority

- verdict: `PASS_TARGETED_PERHVAHU_WATCH_OKNOE`;
- package changed: `false`;
- L1.1 restoration authority changed: `false`;
- L2 tied-cohort authority handling changed: `true`, narrowly for independently
  verified members of the reported tied cohort;
- exact receipt:
  `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_INVERSE_LENGTH_TIED_AUTHORITY_2026-08-03.json`.

### Installed state

- release build: remote `20`-CPU host, `CARGO_BUILD_JOBS=20`;
- installed version: `1.0.1`;
- installed binaries: `lay`, `lay-daemon`, `lay-ibus-engine`;
- active engine after replacement: `lay-ime-ru`;
- global `ibus-daemon` PID before/after: `3702/3702`;
- installed explain route confirms
  `клавиатурой не перхвачу -> клавиатурой не перехвачу`;
- installed explain route confirms `у меня в окное` remains `Abstain`.

## 16. 2026-07-31 Live Input Log Feedback Gate

What was inspected:

- `/home/ubu/.local/share/lay/recent_actions.jsonl`;
- `/home/ubu/.local/share/lay/nanda_wave/word_usage_events.jsonl`;
- `/home/ubu/.local/share/lay/ibus_engine_debug.jsonl`;
- `/home/ubu/.local/share/lay/nanda_wave/l3-online/state.json`.

Measured facts:

- `142` valid recent action records;
- `2,708` valid usage events;
- `871` manually completed visible prediction matches;
- `12` explicit completion accepts and `134` completion rejects;
- `2` double-`Shift` auto-undo rejections;
- the online L3 reader consumed `511,831` source bytes but still had
  `generation = 0` and no pending relation.

Three concrete runtime failures were separated by mechanism:

- `40 000 р -> 40 000 h` and `Екб -> Tr,` were short Cyrillic-to-ASCII
  layout candidates that received apply authority and were then explicitly
  undone by the user;
- `cnjq` had the correct raw projection `стой` in the IME prediction path,
  while the after-space correction path performed a second typo pass and
  applied `сотой`.

Canonical correction:

- a one-to-three-character Cyrillic-to-ASCII layout candidate is
  `KeepOriginal`; learned state may not promote it to an automatic edit;
- once the raw layout projection is an established Russian surface, it
  settles before any secondary typo repair.

What was not tested at the time of recording:

- repair of the full correction-core baseline;
- a post-install multi-hour live input window.

Verification update:

- focused structural gate:
  `short_cyrillic_to_ascii_layout_is_never_applyable_from_logs` -> `PASS`;
- focused correction-core gate:
  `short_russian_word_does_not_autoswitch_to_ascii_from_logs` -> `PASS`;
- focused raw-projection gate:
  `stable_layout_projection_precedes_secondary_typo_repair_from_logs` -> `PASS`;
- source-built probes:
  `40 000 р -> None`,
  `Екб -> None`,
  `cnjq -> стой`;
- route contracts:
  `typing_transition_authority_contract = 20/20`,
  `text_mutation_monopoly_contract = 15/15`,
  `input_gate = 6/6`;
- the wider sequential correction-core run was `84/105 PASS`; its remaining
  `21` authority failures are therefore recorded as `WATCH`, not hidden by
  the focused result;
- two representative failures,
  `deterministic_mode_corrects_multiword_wrong_layout_tail` and
  `unique_transposition_certificate_repairs_short_word`, also failed against
  the unchanged `0.2.333` source in an A/B control. The wider red set is
  baseline debt; it is not promoted to PASS by this experiment.

Exact receipt:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_LIVE_INPUT_LOG_FEEDBACK_2026-07-31.json`.

Runtime authority changed:

- `true`, limited to release `0.2.334`:
  short Cyrillic-to-ASCII candidates are kept original and stable raw layout
  projections settle before secondary typo repair.

Installation verification:

- installed `lay 0.2.334`;
- `lay-daemon`, `lay-l3-online`, the GNOME extension and the L1.1 service are
  active;
- the IBus daemon retained PID `3793`;
- the Lay engine changed from PID `3236279` to PID `2989683`, matched the
  release SHA-256 and answered its D-Bus health probe;
- the previous engine mode `lay-ime-us` was restored;
- installed probes remained:
  `40 000 р -> None`,
  `Екб -> None`,
  `cnjq -> стой`.

## 13. Standalone Full-Neighbor V13, 2026-07-30

V13 closes the cold standalone `L2` package over the final global `L1.1`
field. It does not recompile `L1.1` and does not store a second lexical
restorer. The package binds existing `L1.1` terminal identities to a larger
materialized morphology field and local context competition.

```text
L1.1 bounded lattice
-> StandaloneL2Field
   -> terminal/surface form binding
   -> same-lemma expansion
   -> morphology-slot centers
   -> document-split near-neighbor couplings
   -> directional competition edges
-> Winner | Tied | Abstain
-> L3
-> verifier
```

The context teacher was built from a public Russian literature corpus with an
80/20 document-level split. A surface is admitted to the neighbor proof only
when an independent heldout document exists. No surface, lemma, product or
phrase-specific runtime rule was added.

Measured package facts:

```text
source unique surfaces                  1,875,032
L1.1-bound forms                          517,257
L2-materialized forms                   1,357,775
lemma centers                              93,672
morphology bindings                    3,255,785
context modes                              41,967
slot centers                                  225
neighbor couplings                         15,922
directional competition edges             215,121
train scenes                                58,117
heldout scenes                           2,543,808
package bytes                         135,121,803
package size                              128.86 MiB
package SHA-256
bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
```

Fixed heldout proof:

```text
same-lemma total                         2,501,613
same-lemma target coverage              99.998081%
same-lemma false authority                       0

noun target coverage                    100.000000%
adjective target coverage               100.000000%
pronoun target coverage                 100.000000%
verb target coverage                     99.986490%

near-neighbor total                        42,195
near-neighbor target coverage            100.000000%
near-neighbor false authority                      0
near-neighbor tied                         41,832
near-neighbor correct winners                 363

cold load                                  477,477 us
hot p50 / p99                                22 / 97 us
proof workers                                     20
```

The high tied count is intentional: an unseen local scene does not acquire
fake authority merely because several forms share one lemma. The proof gate is
target retention plus zero false authority, not winner count in scenes that
remain linguistically underdetermined.

Product query:

```text
context                 сокольим глазком _
L1.1 seed               посмотреть, evidence 1000
L2 form                 посмотри, evidence 760 + explicit competition 486
L2 local score          1246
readout                 Winner(посмотри)
```

This result comes from corpus evidence keyed by context mode and morphology
features. The executable contains no `посмотреть -> посмотри` branch.

Cold build measurements:

```text
corpus preparation       69.41 s, 99% CPU, 1,972,860 KiB peak RSS
package compile          20.72 s, 332% CPU, 3,557,868 KiB peak RSS
fixed proof              20.03 s, 489% CPU, 3,928,852 KiB peak RSS
```

What was tested:

- full final package decode and standalone status;
- complete same-lemma and near-neighbor heldout denominators;
- per-POS target coverage and false authority;
- exact `L1.1` package fingerprint binding;
- bounded runtime latency;
- a context-driven same-lemma form movement from `посмотреть` to `посмотри`.

What was not tested:

- every possible semantic distinction in unrestricted Russian text;
- multi-day live daemon stability with V13;
- broad discourse meaning beyond the local L2 context window.

Verdict scope:

- `PASS_standalone_field` for packaged local morphology/context competition;
- runtime authority did not change during the cold experiment;
- release installation still requires the common daemon/IME verifier smoke.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RUSLIT_FULL_NEIGHBORS_V13_2026-07-30.json
```

### 13.1 Release cutover 0.2.333

The proven V13 package and matching binaries were installed atomically after
an isolated installation check. Only `lay-daemon.service` and
`lay-l3-online.service` were restarted. The GNOME extension was reloaded to
show the new version; the managed IBus engine was not restarted.

```text
installed version                         0.2.333
installed package        LAY-L2-RU-FULL-v13.bin
installed SHA-256
bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
installed package status                    ready
IBus PID before / after         3236279 / 3236279
tray reported version                     0.2.333
daemon / L3-online service          active / active
daemon cgroup memory after reload           331 MiB
daemon process RSS                      161,668 KiB
IBus process RSS                        130,340 KiB
```

Installed live probes:

```text
Нужно ... Apple b  -> Apple и       selected
Apple b            -> no selection
в коде             -> no correction-core selection
врмея              -> время         selected
```

The double-Shift exact autocorrect rollback contract passed both its static
authority contract and daemon pending-undo runtime test. No physical keyboard
event was injected during release verification.

Runtime authority changed in release `0.2.333`: `true`, only through the
existing L2 local readout, L3 context and transition verifier chain.

### 13.2 Public V13 package distribution, 2026-08-01

GitHub issue `radislabus-star/lay-public#40` exposed a release-distribution
defect rather than an L2 field defect. The source installer required a local
canonical package under `data/l2/`, but that 128.86 MiB artifact is not stored
in the public Git checkout. A clean installation therefore built every Rust
binary and then failed before installing the user service.

Release `0.2.341` makes the proven V13 artifact an immutable GitHub Release
asset and pins its complete contract in the installer:

```text
artifact               LAY-L2-RU-FULL-v13.bin
bytes                                      135121803
SHA-256  bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
release URL   .../releases/download/v0.2.341/LAY-L2-RU-FULL-v13.bin
cache                    ~/.cache/lay/models/
install       ~/.local/share/lay/nanda_wave/l2/
```

The resolver accepts, in order, a verified explicit/source artifact, the
already installed artifact, or the verified cache. Only when none exists may
it download over HTTPS. Byte count and SHA-256 are checked before any release
binary is installed and checked again on the atomic package copy. Offline
updates reuse an already verified installation. Missing or corrupt input stops
before a partial binary installation.

Measured release checks:

```text
clean-checkout fixture download and install          PASS
offline reuse of installed package                   PASS
corrupt package rejection                            PASS
no binary installed on package failure               PASS
public install/update/uninstall regressions          PASS
real local V13 bytes and SHA-256                      MATCH
remote release build, 20 Cargo jobs                  PASS, 1m59s
public anonymous HTTPS asset download                PASS, 23.27s
download resolver peak RSS                           13,920 KiB
public downloaded bytes                              135121803
public downloaded SHA-256                            MATCH
isolated cache-to-install route                      PASS
isolated installed version                           0.2.341
local installed version                              0.2.341
local daemon / L3-online                      active / active
local GNOME extension version                        0.2.341
local IBus PID before / after               1630206 / 1630206
```

What was not tested at this checkpoint:

- a completely blank operating-system installation including dependency
  installation, service activation and a new desktop login;
- any new L2 quality behavior, because package bytes and runtime authority are
  intentionally unchanged.

Verdict scope: `PASS_public_install`. Runtime authority changed: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/PUBLIC_INSTALL_CANONICAL_L2_V13_2026-08-01.json
```

## 13. Pairwise Context Witness Boundary

The canonical live local route is:

```text
L1.1 bounded lattice
-> L2 candidate field
-> L3 directed pair certificate
-> L4 witness resolution
-> transition verifier
-> one selected edit or ABSTAIN
```

L2 context support and an L3 pairwise certificate are different signals. L2
may keep several candidates alive; only the directed L3 certificate identifies
which contextual relation won. L4 must preserve that distinction instead of
merging both signals into one boolean support flag.

The certificate does not manufacture text and does not grant direct apply
authority. It only removes losing semantic classes from the already bounded
L2 lattice. The verifier remains the sole owner of whether the selected edit
may mutate visible text.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_LAYOUT_RUNTIME_CLOSURE_2026-07-30.json
```

## 13. Canonical V7 full-lemma package, 2026-07-30

The final L1.1 base contains one seed for every admitted Russian lemma. The
canonical L2 compiler then materializes non-L1 wordforms inside L2 instead of
requiring L1.1 to duplicate the complete morphology surface set.

```text
L1.1 WordCenter                           852,582
morphology source bindings             3,255,785
unique morphology surfaces             1,875,032
lemma centers                              93,672
unseeded lemmas                                  0
L1-bound forms                            517,257
L2-materialized forms                   1,357,775
competition edges                          54,407
context modes                                 123
package bytes                         130,595,163
package SHA-256  436b2b8cc99f16c48f240f5fbeef0a64dc2ccb7c84b898e948d34f0adaf3e41e
compile wall                                20.42s
compile peak RSS                       3,514,936 KiB
compile swap                                     0
```

Full heldout:

```text
evaluated scenes                        2,501,613
unresolved                                      0
target lattice coverage                   99.9977%
winner top-1                              45.8284%
false authority                                  0
hot p50 / p99                         21 / 97 us
proof workers                                   20
proof wall                                  17.77s
proof peak RSS                         3,881,396 KiB
```

| POS | Cases | Target coverage | Winner top-1 | False authority |
|---|---:|---:|---:|---:|
| adjective | 1,592,125 | 100.000% | 38.812% | 0 |
| noun | 554,148 | 100.000% | 80.545% | 0 |
| pronoun | 46 | 100.000% | 52.174% | 0 |
| verb | 355,294 | 99.984% | 23.124% | 0 |

The low winner percentage is not an error hidden by aggregate coverage. L2
keeps morphologically valid alternatives tied where local evidence cannot
choose safely; L3 and the verifier remain responsible for wider context and
edit authority. Near-neighbor proof is `20/20`.

The old morphology shadow runtime and same-lemma donor are removed from the
live ownership graph. The executable route is:

```text
L1.1 bounded lattice
-> StandaloneL2Field V7
-> one Winner | Tied | Abstain readout
-> L3
-> verifier
```

Tested: complete source corpus, all lemma reachability, full heldout,
per-POS denominators, near-neighbor field, package latency and zero false
authority. Not tested here: broad semantic sentence understanding or a global
IBus restart. Runtime authority did not change during the remote proof.

Code verification:

```text
lexical_grokking unit tests                 103/103
l2_field unit tests                          30/30
context_phase unit tests                     70/70
new/changed owner tests                       5/5
typing transition contracts                 20/20
text mutation monopoly contracts            15/15
IBus committed-tail and double-Shift        18/18
daemon full-undo preservation                 1/1
```

The broad correction-core comparison remains `WATCH`: clean `0.2.329` and this
change fail the same 22 test names in the same remote environment. Baseline was
`89 passed / 22 failed`; this change is `90 passed / 22 failed` because its new
semantic-drift owner regression passes. No new wide failure was introduced.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU_FULL_V7_ALL_LEMMAS_2026-07-30.json
```

## 13. 2026-07-29 Standalone RU L2 V6 Evidence Authority

### 13.1 Kernel Ownership

The accepted standalone package route is:

```text
L1.1 bounded terminal lattice
-> up to 4 evidence-ranked lemma hypotheses
-> L2-owned generated-form decoder
-> global morphology-slot phase
-> lemma-specific neighbor pressure
-> directional competition
-> Winner | bounded Tied lattice | Abstain
-> L3
-> verifier
```

The package stores generated UTF-8 surfaces itself. A form absent from L1.1 has
`l1_terminal_id = u32::MAX` and a valid `decoder_ref` in the L2 decoder. L1.1
therefore owns lexical seed birth, while L2 owns morphology materialization.

Competition provenance is part of the evidence contract:

- ordinary morphology competition may settle forms inside one lemma;
- only an explicit near-neighbor teacher edge may independently authorize a
  cross-lemma competition transition;
- global morphology-slot evidence identifies a grammatical slot, but is not
  independent evidence for lexical lemma identity;
- if cross-lemma evidence is insufficient, the readout preserves the bounded
  candidate lattice instead of manufacturing a singleton;
- finite verb forms with the same person and number remain tied across
  underdetermined tense or mood when no lemma-specific evidence separates them.

No word-specific exception list or target surface rule was added.

### 13.2 Compiled Package

Measured on `e@192.168.3.94`:

```text
source morphology bindings       3,255,785
source unique surfaces           1,875,032
source lemmas                       93,672

admitted lemma centers              76,500
unseeded lemmas                     17,172
admitted forms                   1,410,190
L1.1-bound forms                   500,085
L2-materialized forms              910,105
morphology bindings              2,405,261
context modes                         123
slot centers                          225
neighbor couplings                  11,847
competition edges                   40,491
decoder bytes                   31,824,107

package bytes                   96,594,655
package MiB                          92.12
compile wall seconds                  19.09
compile average CPU                  351%
compile peak RSS KiB             3,511,952
compile swap bytes                       0
```

Artifact:

```text
/home/e/build/lay-l1-shadow/artifacts/l2-v6-evidence-authority-2026-07-29/LAY-L2-RU-FULL-v6.bin
SHA-256 b9b0d43c17dfd55562a42d325ff529d5d070c571dd1ca046ca5135f8b7f0093d
```

### 13.3 Fixed Heldout Proof

Proof artifact:

```text
/home/e/build/lay-l1-shadow/artifacts/l2-v6-evidence-authority-2026-07-29/proof-final-zero-authority.json
```

Measured facts:

```text
heldout scenes available          2,501,613
evaluated with at least one seed  1,847,790  73.863943%
unresolved without any L1 seed      653,823  26.136057%

resolvable target coverage          99.997078%
resolvable winner top-1             46.463072%
resolvable false authority                   0
resolvable abstain                          51

POS          evaluated     target coverage     false authority
noun           554,148        100.000000%                     0
adjective    1,041,226        100.000000%                     0
verb           252,370         99.978603%                     0
pronoun             46        100.000000%                     0

near-neighbor scenes                    20
near-neighbor top-1               100.000%
near-neighbor false authority             0

cold load                           347.807 ms
hot p50                                  20 us
hot p99                                  92 us
proof workers                                20
proof wall seconds                       16.31
proof average CPU                         431%
proof peak RSS KiB                   3,785,392
proof swap bytes                              0
```

The proof passes the decision contract on the resolvable domain. It does not
prove the `17,172` lemmas that have no L1.1-bound form. Those lemmas cannot be
born from the current L1.1 terminal lattice and remain an explicit corpus
boundary, not a hidden failure inside the evaluated denominator.

### 13.4 Rejected Safety Experiments

The following experiments were rejected:

1. seed-count readout majority:
   removed false singletons but also blocked legitimate context-driven
   lower-support lemma transitions;
2. global slot-only tie:
   removed false authority but collapsed winner top-1 to `0.339054%`;
3. treating ordinary within-lemma competition as cross-lemma authority:
   produced `25` false-authority winners on the first full V5 proof.

The accepted rules are structural evidence rules. They do not reference
individual words from the failure set.

### 13.5 Verdict And Authority

```text
standalone package build                         PASS
resolvable per-POS target coverage >=99%         PASS
resolvable false authority = 0                   PASS
near-neighbor top-1 and false authority          PASS
package format and size                          PASS
hot-path latency                                 PASS
all-source-lemma reachability                    WATCH 17,172 unseeded lemmas
isolated full-route V6 compare                   PASS 0 false authority
source default package changed                   true
running IME/daemon authority changed             false
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU_FULL_V6_EVIDENCE_AUTHORITY_2026-07-29.json
```

### 13.6 Full L1.1 -> L2 -> L3 -> Verifier Replay

What was tested:

- the complete isolated correction route with the installed L1.1 V8 package,
  canonical RU L2 V6 package, L3, decision core, and verifier;
- both `FullWave` reference and `L2FieldShadow` live-owner routes over every
  correction receipt that contains `lay_from`;
- package discovery through the normal installed path, without an explicit
  `LAY_L2_PACKAGE` override;
- parallel deterministic replay with `20` workers;
- persisted usage-memory rebuild after changing the accepted-event projection.

Measured facts:

```text
correction log records seen                  2,945
records with a replayable lay_from             975
workers                                          20

reference eligible applies                      70
reference applies matching user target          68
reference false authority                        2

L2 V6 owner eligible applies                    28
L2 V6 owner applies matching user target        28
L2 V6 owner false authority                      0

selected surface divergences                    71
selected gate divergences                       76
selected provenance divergences                101

wall time                                    9.77 s
average CPU                                  1,569%
peak RSS                                  543,608 KiB

targeted release tests                         72 PASS
targeted release failures                       0
wide nanda_wave current                    541 PASS / 8 FAIL
wide nanda_wave HEAD baseline              529 PASS / 8 FAIL
```

The final two false-authority cases were not repaired with word-specific
conditions. They exposed a derived-cache compatibility error:

1. automatic `autocorrect` and `layout` applies were already excluded from new
   positive feedback;
2. old schema-13 usage snapshots still contained counts compiled before that
   exclusion;
3. usage snapshot schema `14` invalidates those derived counts and rebuilds them
   from the raw event log;
4. signed-memory state and target IDs cover the complete normalized phrase,
   preserve case and punctuation, and therefore do not collapse unrelated
   scenes onto the last token.

Concrete surfaces from the failure log occur only in regression tests. The
production rule checks event provenance and signed state identity; there is no
word allowlist, denylist, or hardcoded replacement.

What was not tested:

- a restart of the user's global IBus engine or running desktop daemon;
- runtime behavior for the `17,172` source lemmas with no L1.1 seed;
- a claim that the smaller number of eligible V6 applies is a complete
  correction-quality improvement outside the measured user-target receipts.

The wide `nanda_wave` gate is not green, but it did not introduce a new failing
test. The same eight test names fail on `HEAD 0.2.328` and on this change. They
cover stale tracked L3 schema fixtures, historical `LayoutWordCell32` ownership,
legacy FullWave trace expectations, one language-quality fixture, and two
environment-sensitive completion checks. They remain a separate `WATCH`; the
focused L2, signed-memory, transition-identity, and usage-projection tests pass
`72/72`.

Verdict scope:

- the isolated installed-package route passes the zero-false-authority gate on
  all `975` replayable real correction receipts;
- source and release discovery may select L2 V6 by default;
- the already running desktop authority is unchanged until a separate safe
  daemon/IBus restart;
- the package remains bounded by the standalone V6 proof in section 13.3.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V6_LIVE_OWNER_SIGNED_FEEDBACK_2026-07-29.json
```

Remote replay evidence:

```text
/home/e/build/lay-runtime-replay/v6-default-full-owner-schema14.json
/home/e/build/lay-runtime-replay/v6-default-full-owner-schema14.time
/home/e/build/lay-runtime-replay/baseline-0.2.328-nanda-wave.log
/home/e/build/lay-runtime-replay/current-0.2.329-nanda-wave.log
```

## 13. 2026-07-28 IBus L2 Cache Budget

The initial attribution to the L2 lexical cache alone was wrong. The compact
`62,424,748 B` lexical phase package is mmap-backed, but the process also
loaded the L3 composite and retained bounded lexical completion readouts.

```text
initial L2 preload prefixes                 1,536
initial preload material limit                 96
live IME material limit                         48
initial maximum cache entries                1,536
L3 runtime manifest deltas                        0
```

The L2 preload used a different material-limit key from live IME requests and
therefore produced few useful cache hits. More importantly, the zero-delta L3
manifest still passed its complete base package through the shard reducer.
That no-op reduction created the large anonymous runtime regions.

Baseline after warmup:

```text
RSS                           245,812 KiB
PSS                           215,609 KiB
anonymous PSS                 177,772 KiB
file PSS                       37,837 KiB
swap                                0 KiB
```

Three intermediate configurations were rejected:

```text
256 prefixes, material 32, cache 256
  8-second RSS                  110,128 KiB
  2-minute RSS                 217,720 KiB
  verdict                      REJECT: early-only reduction

bootstrap only, material 48, cache 64
  cold "п"                         78,727 us
  cold "пров"                      55,898 us
  cold sentence ending "д"         80,742 us
  verdict                      REJECT: cold latency regression

78-prefix warmup, RU 192 / EN 96, cache 128
  8-second RSS                  106,284 KiB
  2-minute RSS                 118,500 KiB
  5-minute RSS                 226,904 KiB
  5-minute PSS                 196,794 KiB
  5-minute anonymous PSS       159,028 KiB
  verdict                      REJECT: delayed allocator growth
```

The accepted configuration and loader behavior are:

```text
bootstrap preload prefixes                     2
Russian bootstrap prefix                    "пр"
English bootstrap prefix                    "ex"
preload mode                      CompletionOnly
Russian preload/live cache key                192
English preload/live cache key                 96
maximum cache entries                         128
zero-delta L3 manifest              direct base load
L3 shard reduce when deltas == 0          disabled
```

Russian preedit requests `24` candidates and therefore uses
`24 * 2 * 4 = 192`; English requests `12` and uses `12 * 2 * 4 = 96`.
Those exact cache keys are unchanged. Only speculative startup materialization
was narrowed: rare prefixes still traverse the complete optimized DAFSA lane
on first use, and no candidate, posting, or decoded-surface frontier was cut.
A non-empty L3 delta list still uses the existing composite reducer.

Measured on the same T480 after a managed child-engine restart:

```text
metric                      baseline      16 sec       2 min       5 min
RSS                     245,812 KiB  105,376 KiB  105,376 KiB  105,376 KiB
PSS                     215,609 KiB   75,374 KiB   75,373 KiB   75,375 KiB
anonymous PSS           177,772 KiB   40,580 KiB   40,580 KiB   40,580 KiB
file PSS                 37,837 KiB   34,795 KiB   34,793 KiB   34,795 KiB
swap                           0 KiB        0 KiB        0 KiB        0 KiB
```

At five minutes this is `-57.1%` RSS, `-65.0%` PSS, and `-77.2%` anonymous
PSS against the original warm baseline. More importantly, the delayed
five-minute rebound of the rejected 78-prefix configuration did not recur.

The first timing table below was produced by an unoptimized debug test. It is
retained as diagnostic evidence, not presented as production latency:

```text
hot samples                                      140
hot p50 / p90 / p99 / max       29 / 36 / 43 / 53 us
cold "п"                                      50,307 us
cold "пр"                                      2,322 us
cold "пров"                                   43,678 us
cold "file"                                    1,035 us
cold sentence ending "д"                      55,616 us
```

The cache-key mismatch and cold DAFSA path were corrected in `0.2.327`.
Production release measurements for the final two-prefix bootstrap on the same
fixed samples:

```text
sample                         before       final
Russian "п"                    7,990 us    6,908 us
Russian "пр"                     455 us      829 us
Russian "пров"                 8,077 us    6,726 us
English "file"                   116 us      152 us
English sentence ending "d"    2,690 us    2,237 us
Russian sentence ending "д"   11,431 us    8,067 us
Russian long context "при"     1,944 us    2,344 us
hot p99                           22 us       10 us
```

The remaining `6.7-8.1 ms` rare cold cases are genuinely new decoded-form
basins, not cache-key misses. They retain the complete `1,152`-surface material
lane. The runtime now visits atoms without allocating one `Vec<u8>` per byte
n-gram, computes phase and center keys in one pass, carries DAFSA character
depth through recursion, and reuses the terminal character count.

Tested:

- `precognition_candidate_generation_stays_under_budget`: PASS;
- lexical cache projection regression: PASS;
- streaming atom summary parity against materialized atoms: PASS;
- lexical phase runtime completion tests: `9 / 9` PASS;
- zero-delta L3 composite fast-path regression: PASS;
- `scripts/check-lay-changed.sh`: PASS;
- release `lay-ibus-engine 0.2.327` built and loaded;
- live process PID `3236279` used the installed release;
- managed child-engine restart retained an `xkb:ru::rus` fallback and did not
  restart the global IBus daemon;
- no IBus daemon restart and no swap.

Not tested in this checkpoint:

- multi-day cache churn at the 128-entry bound;
- end-to-end physical key-to-GNOME-frame latency;
- full L2/L3 quality proof;
- memory with one or more admitted L3 delta packages.

Verdict scope:

- `PASS_runtime_memory_5m`;
- no scoring, candidate-birth, settlement, package, or authority coefficient
  changed;
- this is not a restoration-quality promotion claim.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_IBUS_CACHE_BUDGET_2026-07-28.json`.

Runtime authority changed:

- `false`.

## 13. 2026-07-27 Full Russian Package And Live Lattice Boundary

What was tested:

- compiled the complete Russian L2 teacher corpus
  `data/morphology/lay_ru_l2_full_pos_v3.tsv`;
- repeated the package build and compared SHA-256;
- ran the fixed full heldout proof with `20` proof workers;
- installed the package and exercised the real
  `L1.1 lattice -> standalone L2 -> correction core` route for
  `звгрузи -> загрузи`;
- measured the standalone field separately from the complete socket route.

Measured package facts:

- package: `data/l2/LAY-L2-RU-FULL-v4.bin`;
- SHA-256:
  `1980f89ca2930dfb4abdba489ebc83313b4e0c1851bd5a29e65b25e838a95108`;
- deterministic repeat: identical SHA-256;
- package size: `23,359,260 B`;
- admitted forms: `500,085`;
- lemma centers: `76,500`;
- morphology bindings: `770,261`;
- slot centers: `221`;
- neighbor couplings: `5,280`;
- competition edges: `18,337`.

Measured fixed heldout facts:

- evaluated scenes: `651,029`;
- unresolved teacher scenes: `1,850,584`;
- target coverage: `99.9963%`;
- winner top-1: `81.8406%`;
- false authority: `0`;
- near-neighbor: `18 / 18`;
- standalone cold load: `121,895 us`;
- standalone hot p50 / p99: `6 / 19 us`.

Per-POS target coverage:

- noun: `100.0000%`;
- verb: `99.9047%`;
- adjective: `99.9923%`;
- pronoun: `100.0000%`.

Live boundary correction:

- `Restore` remains the final L1.1 `Winner | Tied | Abstain` contract;
- a separate `Lattice` service request now exposes the bounded L1.1 frontier
  before final authority classification;
- the canonical L2 route consumes this lattice and no longer depends on the
  collapsed L1.1 winner;
- for `звгрузи`, the lattice contains `загрузить`; standalone L2 expands the
  same lemma and the focused live route selects `загрузи`;
- focused live route test: `1 / 1 PASS`.

Measured integration limitation:

- complete socket-route latency for the focused case was approximately
  `39.5 ms` p50 and `39.7 ms` p99;
- this exceeds the accepted live p99 budget of `5 ms`;
- standalone L2 is not the bottleneck; the remaining cost is L1.1 lattice
  materialization and socket/decode work.

Build CPU observation:

- the canonical production release profile uses `codegen-units=1` with LTO,
  so the final crate/link stage is inherently close to one-core;
- the integration build with `codegen-units=20`, LTO disabled and
  `CARGO_BUILD_JOBS=20` reached approximately `1200% CPU`;
- full proof readout is parallel, while parsing the `435 MB` teacher TSV
  remains single-threaded technical debt.

## 14. 2026-07-27 L1.1 Service CPU And Lattice Transport Experiment

What was tested:

- measured why the remote 20-thread machine stayed at low CPU under concurrent
  L1.1 lattice requests;
- compared candidate-birth atom and posting budgets on the full L1.1 package;
- replaced the diagnostic Lattice socket payload with the compact typed
  `terminal_id + surface + authority + score_milli` transport;
- replaced one new OS thread per socket connection with a fixed reusable
  20-worker pool and a bounded queue;
- tested a smaller 64-candidate phase frontier and checked the complete
  `звгрузи -> загрузи` route.

Measured service facts:

- the old thread-per-connection service processed 20,000 requests at
  `937.1 req/s`;
- the fixed 20-worker pool processed the same 20,000 requests at
  `5,582.9 req/s`, a `5.96x` gain;
- a 100,000-request run sustained `5,931.3 req/s`;
- a separate 40,000-request CPU sample used `110.09` service CPU-seconds in
  `6.415` wall-seconds, or an average of `17.16` CPU cores;
- all `100,000 / 100,000` long-run probes retained the required L1 seed
  `загрузить`;
- resident memory after warming 20 reusable scratches was approximately
  `2.0 GiB`.

Measured posting-budget facts for `звгрузи`:

- one birth atom per channel selected 10 atoms and 10,219 postings, touched
  7,493 centers, but lost `загрузить`;
- two birth atoms per channel selected 20 atoms and 28,094 postings, touched
  20,199 centers, and retained `загрузить` at rank 5;
- a 20,000 global posting budget selected 18 atoms and 17,374 postings, touched
  13,489 centers, and retained `загрузить` at rank 5.

Compact transport facts:

- a 16-seed response is 1,410 bytes;
- a 64-seed response is 5,573 bytes;
- with the unchanged full 128-candidate phase frontier, limit-16 hot latency
  measured p50 `4,711 us` and p99 `5,443 us`;
- this still misses the strict `5,000 us` gate by `443 us`.

Rejected experiment:

- reducing only the L2-facing phase frontier from 128 to 64 produced p50
  `4,018 us` and p99 `4,810 us`;
- L1 still retained `загрузить` at rank 5;
- the complete canonical L2 route lost `загрузи`;
- verdict: `REJECT_quality_regression`; the phase frontier remains 128.

New canonical L2 blocker:

- the teacher corpus contains
  `F загрузить загрузи verb:imp_excl:sg:imp:perf`;
- the L1.1 and L2 package fingerprints match;
- the current L1.1 package does not contain an exact `загрузи` WordCenter;
- direct `Restore("загрузи")` returns `ABSTAIN`, surface `загрузки`,
  geometry distance 1;
- canonical L2 can only emit terminal IDs materialized by the L1.1
  DecoderGraph, so it cannot output a surface missing from L1.1;
- current L2 uses `strongest_lemma_count`, so the larger noun seed cohort
  `загрузка` suppresses the weaker verb lemma seeded by `загрузить`;
- the readout becomes `Abstain` over noun forms and never births `загрузи`;
- two general corrections are required: admit the morphology surfaces required
  by canonical L2 into the L1.1 corpus, then replace count-majority lemma
  selection with a bounded evidence-weighted multi-lemma settlement;
- neither correction may add a word-specific exception.

What was not tested:

- no fixed heldout per-damage-class proof was run for the experimental 20,000
  posting budget;
- no full ambiguity or false-certainty proof was run for the compact transport;
- no canonical L2 heldout proof was run after replacing
  `strongest_lemma_count`;
- no production daemon latency was measured after installation.

Verdict scope:

- fixed worker pool: `PASS_throughput_and_retention_probe`;
- compact typed transport: `PASS_protocol`, `FAIL_latency`;
- 20,000 posting budget: `PASS_probe_only`, not promoted;
- 64 phase frontier: `REJECT`;
- complete canonical L2 route: `FAIL`;
- runtime authority changed: `false`.

Exact receipt:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L11_L2_SOCKET_POOL_AND_LATTICE_BUDGET_2026-07-27.json`

Exact receipts:

- `docs/structural_gates/receipts/L2_CANONICAL_FULL_COMPILE_V7_2026-07-27.json`;
- `docs/structural_gates/receipts/L2_CANONICAL_FULL_PROOF_V7_2026-07-27.json`.

What was not tested or promoted:

- the complete protected IME live gate after installing a release containing
  the new `Lattice` protocol;
- the full correction-core suite still has `82 PASS / 21 FAIL`, inherited from
  the old live-owner assumptions and separate transition-phase authority;
- the `5 ms` complete-route latency gate has not passed;
- no global IBus restart was performed.

Runtime authority changed:

- `false`; package and functional route are proven, but promotion remains
  blocked by complete-route latency and protected live regressions.

## 13. 2026-07-27 Canonical Noun Package Full Safety Proof

What was tested:

- deterministic rebuild of
  `/home/ubu/projects/lay/data/l2/LAY-L2-RU462K-NOUN-v1.bin`;
- fixed heldout readout over every available noun scene;
- per-feature winner, tied-target coverage, abstain, and false-authority
  denominators;
- cold load, hot p50/p99, package size, and peak compile/proof RSS.

Measured facts:

```text
forms                         462,314
lemmas                         47,766
morph bindings                633,016
train scenes                    1,548
heldout scenes                554,148
context modes                      59
slot centers                       60
neighbor couplings              1,548
competition edges               6,144
package bytes              19,244,056
package SHA-256  db8087fb642d29fe270133b5eb08dac12828db9679e64899dddf691ea3b86be6

evaluated                 554,148 / 554,148
winner correct                   450,772
winner top-1                     81.3450558%
tied contains target             103,376
target coverage                 100.0000000%
abstain                                0
false authority                        0

second locative target coverage 100.0000000%
second locative false authority          0

compile wall                         12.82 s
compile peak RSS                   664,392 KiB
proof wall                            ~56 s
proof peak RSS                     746,572 KiB
proof cold load                    852,717 us
proof hot p50 / p99                 68 / 152 us
```

The earlier `68` false-authority cases had two structural causes:

- the context identity retained only the nearest preposition, collapsing
  distinct governors such as `лежит на _` and `сосредоточен на _`;
- an ambiguous surface could borrow pressure from an unrelated homonymous
  lemma and become a false singleton despite equal same-lemma slot evidence.

The canonical correction is:

- context mode and lexical anchor now cover the same bounded two-token window
  used by the scene wave;
- pressure is accumulated inside one lemma and selected across alternative
  lemmas without additive homonym amplification;
- equal positive slot evidence inside one lemma produces `Tied`, never an
  artificial `Winner`.

The reduction from `81.7952966%` to `81.3450558%` winner top-1 is not hidden:
`2,496` additional syncretic or homonymous cases moved to a target-containing
tied lattice. Target coverage improved to `100%` and false authority fell to
zero. This is the required safety trade: ambiguity remains explicit instead of
being reported as false certainty.

What was not tested:

- near-neighbor competition, because the current fixed corpus contains only
  same-lemma morphology scenes;
- verbs, adjectives, pronouns, and auxiliaries;
- installed live package behavior and daemon/IME regression;
- standalone runtime promotion.

Verdict scope:

- canonical Russian noun same-lemma/morphology-slot safety gate: `PASS`;
- full canonical L2: `NOT COMPLETE`;
- runtime authority changed: `false`.

Exact receipts:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU462K_NOUN_COMPILE_CONTEXT_V2_2026-07-27.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU462K_NOUN_PROOF_FULL_V6_2026-07-27.json`

## 14. 2026-07-27 Full Russian POS Teacher And Cross-Lemma Contract

What was implemented:

- the existing noun feature values remain binary-compatible;
- the free bits in the 32-bit feature mask now encode verb, adjective,
  pronoun, number, gender, person, tense, mood, aspect, and POS-specific form
  kind;
- infinitives, finite verbs, imperatives, gerunds, full/short adjectives,
  participles, comparatives, pronouns, and auxiliary forms can share the same
  fixed-width `MorphBinding` package section;
- `LemmaCenter.primary_pos` is derived from admitted bindings instead of being
  hard-coded to noun;
- typed `NT` / `NH` teacher rows represent bounded cross-lemma competition;
- cross-lemma edges are indexed from both endpoint lemma families so runtime
  field birth can reach the relation from either active L1.1 seed;
- proof now has an explicit near-neighbor denominator and failure examples
  instead of reporting `tested=false`.

Cold-teacher generation facts on the remote 20-core build machine:

```text
noun visible form centers                  462,314

adjective/participle lemmas                 48,294
adjective/participle bindings            2,240,679
adjective/participle scenes              1,600,679

verb/infinitive/gerund lemmas                21,772
verb/infinitive/gerund bindings             381,936
verb/infinitive/gerund scenes               337,418

pronoun lemmas                                   24
pronoun bindings                                160
pronoun scenes                                  122

typed near-neighbor relations                    20
corpus bytes                            432,122,626
generation wall                              52.39 s
generation peak RSS                      1,495,744 KiB
```

The corpus artifact is:

`/home/e/projects/lay-l2-build/data/morphology/lay_ru_l2_full_pos_v1.tsv`

Measured facts do not yet imply package/runtime promotion. At this point:

- corpus generation: `PASS`;
- feature/parser/compiler microtests: `PASS`;
- full package compile: running;
- full per-feature and near-neighbor heldout proof: not yet measured;
- runtime authority changed: `false`.

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

## 15. 2026-07-30 One-symbol layout birth for L3 context

`L2FieldShadow` now births the complete bounded one-symbol layout lattice
instead of asking the multi-character layout helper for one winner. For a
single ASCII key, `short_token_candidates` supplies the exact keyboard
projection and configured visual alternatives. The bridge preserves trailing
whitespace and sends every surviving surface through ordinary transition
admission.

```text
one-symbol surface
-> L2 short-token candidates
-> bounded competing surfaces
-> transition admission
-> L3 sentence-context pressure
-> Winner | Tied | ABSTAIN
-> verifier
```

No one-symbol candidate receives authority from its birth order. When two
layout alternatives remain close and L3 has no pairwise certificate or strong
phrase evidence, transition admission keeps the result unresolved. This
prevents an arbitrary first candidate from becoming an autocorrection.

The same candidate constructor is used by the cold L3 probe, so learning and
runtime no longer disagree about the `b` lattice. The visual replacements are
the existing configured lexical surfaces, not a product or sentence-specific
branch.

Measured:

```text
generic candidate birth test                    PASS
unknown-context abstain test                     PASS
L3 context-phase tests                         74/74
targeted L3 relation proof                       PASS
full 80k differential L3 proof                   PASS
new false authority                                 0
```

What was not tested:

- every one-symbol visual ambiguity;
- multi-day live service behavior;
- physical input in every toolkit.

Runtime authority changed during this experiment: `false`.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_LAYOUT_PAIRWISE_DELTA_2026-07-30.json
```

## 2.3 Local Readout Safety Gate Inside `L2FieldShadow`

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib l2_field`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- targeted route spot checks on
  `докурчиват`, `ЯДРА`, `ене`, `смеа`, `сделам`, `сли,`, `вошеьные`:
  selected surface parity restored;
- `scripts/cargo-guard.sh run --bin lay-nanda-wave-eval -- --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2939`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 16 / 134`,
  `compact_apply = 25 / 134`,
  `shadow_apply = 25 / 134`,
  `user_target_match.compact = 7 / 134`,
  `user_target_match.shadow = 7 / 134`,
  `user_target_match.both = 7 / 134`.

Measured implementation facts:

- the generic local readout shell still lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- the near-neighbor donor is now explicitly prevented from collapsing the field
  when its internal winner is not the current lexical leader;
- the same donor may still return `Tied` or `Abstain`, but no longer upgrades a
  nonleader surface into a singleton shadow winner;
- the local readout now keeps the bounded lexical surface field intact on the
  measured real regressions where compact `L2` remained tied or selected a
  different surface through the shared lattice;
- the route-level tests now lock parity on those cases through
  `/home/ubu/projects/lay/src/correction_core/candidate_sources.rs`;
- runtime authority did not change.

What was not tested in this step:

- fixed heldout `L2` proof for local readout winner/tie calibration;
- live IME/daemon authority promotion;
- standalone `L2` package latency, RSS, and cold-load budget;
- broader donor families beyond same-lemma morphology and near-neighbor lexical
  competition.

Verdict scope:

- `L2FieldShadow` now has a safer internal local readout shell above the
  self-born lexical field;
- on the measured 134 real correction-log inputs, selected surface parity and
  selected gate parity with `CompactL2` are restored after tightening this
  winner admission;
- provenance still diverges by design on selected Nanda surfaces because the
  shadow route uses `L2FieldShadowSurface` instead of `L2LexicalPhaseCell32`;
- this is still shadow-only evidence and not yet a promotion to runtime
  authority.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_LOCAL_READOUT_GATED_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_TARGETED_NONLEADER_CASES_2026-07-26.json`

Runtime authority changed:

- `false`

## 2.4 First Owner-Swap Pass: `L1.1` Seeded Birth Inside One Local Field

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib l2_field`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `scripts/cargo-guard.sh run --bin lay -- --compare-candidate-routes --candidate-route l2-field-shadow 'врмея '`
  kept selected surface parity and selected gate parity while collapsing the
  shadow route to one local readout candidate;
- `scripts/cargo-guard.sh run --bin lay -- --compare-candidate-routes --candidate-route l2-field-shadow 'пку '`
  restored abstain parity on the short ambiguous token after adding the short
  seeded-birth guard;
- `scripts/cargo-guard.sh run --bin lay-nanda-wave-eval -- --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2939`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 17 / 134`,
  `compact_apply = 26 / 134`,
  `shadow_apply = 26 / 134`,
  `user_target_match.compact = 7 / 134`,
  `user_target_match.shadow = 7 / 134`,
  `user_target_match.both = 7 / 134`.

Measured implementation facts:

- the seeded birth merge lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs` inside
  `shadow_surface_seed_candidates(...)`;
- `L2FieldShadow` no longer emits a separate shadow-side `L2FieldShadowL11`
  candidate during route materialization;
- authoritative `L1.1` restore output is now internalized into the same local
  `L2` field as bounded surface evidence and may surface as
  `L2FieldShadowSurface` or `L2FieldShadowReadout`;
- existing lexical candidates receive only bounded `L1.1` score/overlap boosts;
- seed-only insertion is bounded to at most two authoritative surfaces and only
  for token length `>= 4`;
- token length `<= 3` bypasses seeded birth entirely, which restored parity on
  the measured `пку` short-signal regression;
- on `врмея`, the compact route still exposes a wider 7-candidate lattice while
  `L2FieldShadow` now settles the same selected surface into one local readout
  candidate with no surface/gate divergence;
- runtime authority did not change.

What was not tested in this step:

- fixed heldout `L2` proof for the seeded-birth route;
- live IME/daemon runtime promotion;
- latency and RSS budget of the per-request `L1.1` seed request path under
  sustained daemon load;
- broader seeded-birth replay beyond the measured 134 real correction-log
  inputs and the targeted `врмея` / `пку` probes.

Verdict scope:

- this is the first real owner-swap pass toward
  `L1.1 bounded lattice -> one real L2 local field -> one local readout -> L3 -> verifier`;
- inside `CandidateReadoutRoute::L2FieldShadow`, `L1.1` now feeds the local
  field as formal bounded seed birth rather than as a separate route-level
  sidecar candidate;
- on the measured 134 real correction-log inputs, selected surface parity and
  selected gate parity with `CompactL2` are preserved;
- provenance still diverges by design because the shadow route now reports its
  own internal field ownership instead of `L2LexicalPhaseCell32`;
- this remains shadow-only evidence and is not yet a runtime promotion.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_L11_SEEDED_BIRTH_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_VRMEYA_L11_SEEDED_BIRTH_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_PKU_L11_SEEDED_BIRTH_2026-07-26.json`

Runtime authority changed:

- `false`

## 2.5 Live Owner Flip For IME And Daemon Local Correction

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib ime_correction`: passed, `25 / 25`;
- `scripts/cargo-guard.sh test --lib l2_field`: passed, `9 / 9`;
- `scripts/cargo-guard.sh check --bin lay`: passed;
- `scripts/cargo-guard.sh check --bin lay-daemon`: passed.

Measured implementation facts:

- `/home/ubu/projects/lay/src/candidate_contract.rs` now makes
  `CandidateReadoutRoute::live_default()` return `L2FieldShadow`;
- the live local IME route under
  `/home/ubu/projects/lay/src/ime_correction.rs` now expects boundary-owned
  Space/autocorrect authority as `L2FieldShadowBoundary`;
- `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs` no longer carries
  temporary `LAY_DEBUG_SHADOW_L2_FIELD` logging;
- `/home/ubu/projects/lay/src/ime_correction.rs` no longer carries temporary
  `LAY_DEBUG_IME_CORRECTION` logging;
- the shadow local donor winner multiplier is explicit and fixed as
  `SHADOW_DONOR_WINNER_WEIGHT = 5`;
- tied and abstain donor bonuses remain bounded and unchanged; only the winner
  multiplier was made explicit.

What was not tested in this step:

- fixed heldout `L2` proof for the live local owner route;
- standalone `L2` package latency, RSS, and cold-load budget;
- broader `L2` donor families beyond same-lemma morphology and near-neighbor
  lexical competition;
- promotion of `L2FieldShadow` from donor-reusing owner contour to a fully
  standalone packaged `L2`.

Verdict scope:

- the live local IME/daemon correction route is now owned by one real
  `L2FieldShadow` local field above bounded `L1.1` evidence;
- the route now reads as
  `L1.1 bounded lattice -> one real L2 local field -> one local readout -> L3 -> verifier`
  for live local correction;
- this is a runtime authority change for the local route;
- this is not yet proof that the final standalone canonical `L2` package is
  complete.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_LIVE_OWNER_IME_DAEMON_GATE_2026-07-26.json`

Runtime authority changed:

- `true`

## 2.6 Divergence Buckets On Real Corrections Window

What was tested for this code step:

- `scripts/cargo-guard.sh run --bin lay-nanda-wave-eval -- --l2-route-compare-report --limit 200 --examples 200`
  on `/home/ubu/.local/share/lay/corrections.jsonl`.

Measured implementation facts on Monday, July 27, 2026:

- the same `134` usable records in the `200`-line window now reran as
  `surface_diverged = 19 / 134`,
  `gate_diverged = 19 / 134`,
  `provenance_diverged = 33 / 134`;
- `reference_apply = 27 / 134`;
- `shadow_apply = 30 / 134`;
- user-target exact-normalized matches on this rerun were:
  `reference = 7 / 134`,
  `shadow = 8 / 134`,
  `both = 5 / 134`;
- the divergences are not one amorphous problem; they split into five concrete
  buckets:
  - `8` cases: shadow false apply or false suggest after reference abstain;
  - `3` cases: shadow found the user target while reference abstained;
  - `3` cases: shadow missed a user-target hit that reference found;
  - `4` cases: reference picked an off-target winner while shadow abstained;
  - `1` case: both routes committed to different off-target winners.

Operational interpretation:

- the main unfinished `L2FieldShadow` problem is now explicit: `8 / 19`
  divergent cases are unsafe local-field winner births where the route should
  emit tied lattice or abstain instead of selecting a local winner;
- the next recall problem is narrower: `3 / 19` cases where `L2FieldShadow`
  abstains but `FullWave` actually hits the target;
- the route also already has `3 / 19` positive wins that should be preserved
  while tightening the unsafe bucket.

What was not tested in this step:

- fixed heldout proof after any bucket-specific field/readout changes;
- full IME/daemon replay of these exact `19` cases;
- runtime latency or RSS impact after tightening local readout.

Verdict scope:

- this rerun converts the vague “18/134” or “19/134” discussion into a real
  `L2` work queue;
- the priority is no longer abstract parity with `FullWave`, but the concrete
  `8` shadow false-apply/false-suggest cases that should collapse into tied or
  abstain readout;
- this is measurement only; runtime authority did not change in this step.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_DIVERGENCE_BUCKETS_CORRECTIONS_200_2026-07-27.json`

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
3. the old `CompactL2` route is no longer executable, but the live
   `L2FieldShadow` route still reuses donor packages and is not yet a
   standalone packaged canonical `L2`.

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

The standalone packaged `L2` may replace donor-reusing `L2FieldShadow` only
when:

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

### 9.2 Stage B: Reference Compare Readout

This stage is complete for live ownership. The remaining compare shape is now:

```text
CandidateReadoutRoute::FullWave
CandidateReadoutRoute::L2FieldShadow
```

`L2FieldShadow` already owns the live local route; `FullWave` remains the
reference compare path.

### 9.3 Stage C: A/B Receipts

On fixed corpora and selected live logs compare:

- `FullWave` reference;
- live `L2FieldShadow`;
- later standalone packaged `L2`.

The comparison must show where the new route wins, ties, abstains, or regresses.

### 9.4 Stage D: Runtime Flip

This stage is complete for the live local route:

```text
live route
L2FieldShadow
-> one local readout above bounded L1.1 input
```

`L1.1` is no longer a separate live sidecar on that route; it is already
internalized as bounded lexical input to the local field.

### 9.5 Stage E: Remove Ownership Drift

After standalone package promotion:

- remove the remaining donor-reuse ownership drift inside `L2FieldShadow`;
- keep the old lexical route only in historical receipts, not in executable
  code paths;
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

## 12. 2026-07-27 Local Readout Safety Tightening

What was changed in this step:

- `L2FieldShadow` local readout now demotes tied/abstained local surface cohorts
  from `Eligible` to `SuggestOnly` instead of leaving the correction core to
  pick an arbitrary surface winner;
- short dense growth clusters now emit `l2_field_shadow_local_tie` or
  `l2_field_shadow_local_abstain` for affected local surface candidates;
- live double-`Shift` rollback of layout-only autocorrect was restored in the
  daemon correction memory path.

Measured facts from direct route compares on 2026-07-27:

- `смеа `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored;
- `докурчиват `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored;
- `сли, `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity preserved;
- `слои `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored after demoting the short growth tail
  (`соли`, `слови`, `слоги`, `сложи`, `сломи`) to suggest-only;
- `ене `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored; `пение ` remains suggest-only under
  `short_sparse_multi_omission_requires_tie_or_context`;
- `сделам `:
  direct live compare currently gives
  `FullWave.selected = "сделай "`,
  `L2FieldShadow.selected = "сделай "`,
  so it must stay in live parity coverage rather than the abstain-only bucket.

What was tested:

- direct `lay --compare-candidate-routes` probes for
  `смеа `, `докурчиват `, `сли, `, `слои `, `ене `, `сделам `;
- focused unit coverage in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`
  for dense missing-letter and long-form tie clusters;
- focused daemon/lib undo checks for double-`Shift` rollback memory.
- exact receipt:
  `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_SHORT_GROWTH_GATES_2026-07-27.json`.

What was not yet completed:

- refreshed full 134-record divergence bucket receipt after this tightening;
- wider replay over the remaining divergence buckets after the short-growth fix;
- promotion of this safety pass into a finished live-owner compare gate.

Runtime authority changed:

- `false`
