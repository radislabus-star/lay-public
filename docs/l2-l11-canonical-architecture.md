# L2 Canonical Architecture Above L1.1

Status: live owner route for local IME/daemon correction is now
`L1.1 bounded lattice -> one real L2 local field -> one local readout -> L3 -> verifier`;
standalone packaged `L2` promotion remains separately gated.

Last source audit: 2026-07-26.

Runtime authority: enabled for the live local IME/daemon correction route
through `CandidateReadoutRoute::L2FieldShadow`; old `CompactL2` route removal
from executable/public selection is complete; standalone canonical `L2` package
promotion remains pending.

This document records the canonical internal shape of `L2` above `L1.1`. It is
the owner contract for the local candidate field. The live local route now uses
that owner contour, the old lexical `CompactL2` route is no longer executable
through public route selection, and the full standalone `L2` package contract
is still separately gated.

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

Two intermediate configurations were rejected:

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
```

The accepted configuration and loader behavior are:

```text
curated preload prefixes                      79
preload mode                      CompletionOnly
preload/live material limit                    48
maximum cache entries                         128
zero-delta L3 manifest              direct base load
L3 shard reduce when deltas == 0          disabled
```

The curated set contains all `33` Russian letters, all `26` English letters,
and `20` common RU/EN two-letter prefixes. The cache key now matches the live
IME material limit. A non-empty L3 delta list still uses the existing
composite reducer.

Measured on the same T480 after a managed child-engine restart:

```text
metric                      before       8 sec       2 min       5 min
RSS                     245,812 KiB  105,784 KiB  105,404 KiB  107,844 KiB
PSS                     215,609 KiB   75,848 KiB   75,472 KiB   77,913 KiB
anonymous PSS           177,772 KiB   41,440 KiB   41,440 KiB   41,516 KiB
file PSS                 37,837 KiB   34,408 KiB   34,032 KiB   36,397 KiB
swap                           0 KiB        0 KiB        0 KiB        0 KiB
```

At five minutes this is `-56.1%` RSS, `-63.9%` PSS, and `-76.6%` anonymous
PSS against the warm baseline. The rejected two-minute rebound did not recur.

Candidate-generation timing:

```text
hot samples                                      140
hot p50 / p90 / p99 / max       29 / 36 / 43 / 53 us
cold "п"                                      50,307 us
cold "пр"                                      2,322 us
cold "пров"                                   43,678 us
cold "file"                                    1,035 us
cold sentence ending "д"                      55,616 us
```

The cold figures are reported, not hidden by the hot aggregate. They remain a
separate DAFSA readout optimization target and are not a blocker for accepting
the steady-state memory fix.

Tested:

- `precognition_candidate_generation_stays_under_budget`: PASS;
- lexical cache projection regression: PASS;
- zero-delta L3 composite fast-path regression: PASS;
- `scripts/check-lay-changed.sh`: PASS;
- release `lay-ibus-engine 0.2.326` built and loaded;
- live process PID `3024249` used the installed release;
- watchdog fallback restored `xkb:us::eng` after five minutes without user
  confirmation;
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
