# L2 Productive Paradigm Field: Paper Implementation V1

Status: `PAPER_IMPLEMENTATION_COMPLETE`, `COMPILER_CONTOUR_IMPLEMENTED`,
`FULL_SHADOW_PACKAGE_MEASURED`, `FIXED_PROOF_PENDING`,
`RUNTIME_AUTHORITY_UNCHANGED`.

Date: 2026-08-11.

Owning design:
`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-canonical-design.md`.

This document turns the canonical direction into an implementable specification.
It closes the twelve open design decisions in the owning design at paper level.
It does not claim that source, package, quality, latency, or product gates pass.

Implementation ledger:

- V43 typed/event and package foundation is implemented in
  `src/nanda_wave/l2_field/productive_v1/{types,scene,events,format}.rs`;
- exact V39 per-lemma runtime scheduling is restored; V42 receipts remain
  preserved as a rejected latency experiment;
- V43 proves only typed slot width/applicability, lemma-owned split/folds,
  full event SHA-256 identity, provenance separation, canonical 60-cell scene
  encoding, sharded spool sequence/CRC/SHA integrity, checked package V1
  roundtrip, section corruption rejection, and mmap-backed package loading;
- trie, geometry parity, learned evidence, calibration, composite lattice,
  delta promotion, fixed quality proof, latency, and physical product gates
  remain unimplemented or unproved;
- exact receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_TYPED_EVENTS_V43_2026-08-10/receipt.json`.
- V44 implements the paper-defined canonical anchor, scalar edit-program DP,
  byte-exact interpreter, two-TRAIN-lemma transfer gate, exact paradigm
  compatibility, deterministic single-parent productive trie, scalar-prefix
  sharing, and trie instantiation in shadow-only code;
- V44 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_INDUCTION_TRIE_V44_2026-08-11/receipt.json`.
- V45 implements incremental character/keyboard OSA, typed atom refcounts with
  terminal undo, character/keyboard simhash accumulators, relative-anchor trie
  traversal, and the bounded V39 speed-parity top-32 comparator; micro parity
  passes, while the fixed-corpus parity gate remains pending;
- V45 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_GEOMETRY_RUNTIME_V45_2026-08-11/receipt.json`.
- V46 implements independent fitted phase banks, all fixed-record codecs, exact
  `SPV1`/`ADV1` pool semantics, and fail-closed deep mmap validation for range
  ownership, program termination, segment/decoder references, single-parent
  acyclic trie reachability, terminal attribution, phase polarity,
  coefficients, calibration, provenance, and delta manifest;
- the complete local `productive_v1::` gate is `45 passed / 0 failed`; no
  corpus-trained package, fixed heldout quality denominator, remote latency/RSS,
  or physical product gate is claimed;
- V46 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_PHASE_FORMAT_V46_2026-08-11/receipt.json`.
- V47 implements the canonical decoder for all four typed event kinds, bounded
  chunked external sort, bounded multi-pass merge, full-event deduplication,
  global shard merge, and the streaming TRAIN morphology reduce that retains
  only one lemma in memory while assigning canonical lemma/form/variant IDs and
  preserving complete event SHA-256 provenance;
- the complete local `productive_v1::` gate is now `48 passed / 0 failed`; the
  sort micro reduces `80` records to `40` byte-identical unique records through
  multiple bounded runs and at least two merge passes; target usage is
  `8,138,199,040 B` under the `12 GiB` Cargo budget;
- V47 does not yet claim transition-support reduce, corpus-trained package,
  fixed heldout quality, remote resource bounds, composite ownership, or live
  authority;
- V47 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_STREAM_REDUCE_V47_2026-08-11/receipt.json`.
- V48 implements an explicit fingerprinted morphology axis schema, canonical
  anchor-to-form transition emission, bounded external transition sorting,
  distinct-TRAIN-lemma transfer support, streaming support merge join,
  lemma-order replay, exact paradigm signatures, canonical paradigm IDs, and
  bounded compatibility index/postings; no transition or mass-paradigm lemma
  group is retained in memory;
- V48 also resolves terminal surface ownership: productive trie terminals use
  `SURFACE_FROM_TRIE` with `decoder_ref=0`, so generated words are not duplicated
  as dictionary strings; checked exact/speed-parity terminals retain a nonzero
  segment reference;
- the complete local `productive_v1::` gate is `50 passed / 0 failed`; the
  bounded induction micro covers `300` TRAIN lemmas, `600` transition
  observations, numeric ordering beyond lemma ID `255`, one shared paradigm,
  `300` bindings, and its compatibility posting; target usage is
  `8,139,276,288 B` under the `12 GiB` Cargo budget;
- V48 remains shadow-only and does not claim a corpus-trained package, learned
  evidence/calibration, fixed heldout quality, composite ownership, remote
  resources, or live authority;
- V48 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_TRANSITION_INDUCTION_V48_2026-08-11/receipt.json`.
- V49 implements deterministic assembly and atomic publication of all 22
  Productive V1 package sections, including canonical pools, paradigm and
  lemma-local programs, generated-surface trie terminals, phase-profile
  ownership, fitted coefficients, hashed calibration rows, compatibility,
  provenance, and the shadow delta manifest;
- the V49 compiler micro runs the complete typed-event, external-sort,
  one-lemma reduce, transition-induction, evidence-fit, calibration-fit,
  package, deep-validation, mmap-reopen route twice and obtains byte-identical
  output; it includes `300` lemmas sharing a transferable paradigm and one
  additional lemma with an exact local allomorph;
- measured output is `14,600 B`, SHA-256
  `593a234a0448b37528649e3568f429ac440edfaa71df6a4964fef5d0cfe59ca8`,
  with `2` paradigms, `301` bindings, `4` programs, `9` operations, `7` trie
  nodes, `5` trie arcs, `3` generated terminals, and `5` calibration rows;
- the complete local `productive_v1::` gate is now `51 passed / 0 failed`;
  target usage is `8,139,993,088 B` under the `12 GiB` Cargo budget;
- V49 remains a bounded synthetic compiler proof. It does not claim
  corpus-trained coefficients or calibration quality, full-corpus package
  size/RSS, runtime traversal, fixed heldout quality, composite ownership,
  remote resources, or live authority;
- V49 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_PACKAGE_COMPILER_V49_2026-08-11/receipt.json`.
- V50 closes a paper/runtime reproducibility defect found before mmap traversal:
  the four Jeffreys-smoothed TRAIN count priors were required by section 10.2
  but absent from the package. `EVIDENCE_PRIORS` is now required section `23`
  and stores exact odd `2 * count + 1` values for lemma, paradigm, slot, and
  directional channels;
- deep loading requires exactly four canonical prior rows for
  `PRODUCTIVE_V1_MODEL` and none for `V39_SPEED_PARITY`; compilation rejects
  u64 smoothing overflow, and loading rejects zero, even, repeated, missing,
  or reordered channels;
- the amended deterministic compiler micro remains `51 passed / 0 failed` and
  produces `14,728 B`, SHA-256
  `ef0bc135fba29c952b2c3353f6850288f2b3dbeb330a5b125e809e067e351ac0`;
  the exact cost of the required directory entry plus four 24-byte rows is
  `128 B` over V49; target usage is `8,140,054,528 B` under `12 GiB`;
- V50 changes no runtime authority and does not claim corpus-fitted priors or
  quality; it makes such a corpus-trained runtime score reproducible rather
  than manually assigned;
- V50 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_EVIDENCE_PRIORS_V50_2026-08-11/receipt.json`.
- V51 closes the remaining packaged-runtime wire ambiguities before mmap
  traversal: `SlotPhaseProfileV1` now owns independently typed
  `explicit_anti_support`, feature 14 is owned by that profile's positive
  support, feature 15 is the measured minimum of nonzero binding/paradigm
  stability, and directional evidence uses the canonical scene hash from
  section 10.2;
- the complete local `productive_v1::` gate remains `51 passed / 0 failed`;
  the deterministic compiler micro now produces `14,744 B`, SHA-256
  `336ad181cc8f53e498bbdb824f5a834a3fecdc191751bfa019bfaff13928b185`;
  the exact `16 B` increase over V50 is four slot profiles widened from 40 to
  44 bytes;
- V51 changes no runtime authority and does not claim trained mmap traversal,
  fixed heldout quality, remote resources, or product gates;
- V51 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_RUNTIME_WIRE_CLOSURE_V51_2026-08-11/receipt.json`.
- V52 implements the first trained mmap shadow runtime in
  `productive_v1/packaged_runtime.rs`: it verifies L1.1/canonical-L2 package
  fingerprints, caches only `15` Q16 coefficients plus four TRAIN priors
  (`124 B`), binary-searches typed package indexes, traverses trie records
  lazily, computes all 15 trained features, retains exact top-32 plus one
  overflow sentinel, and settles packaged `Winner | Tied | ABSTAIN` without
  changing live authority;
- the end-to-end compiler micro now reopens the published mmap package as the
  trained runtime, applies one real compiled lemma binding, retains both
  licensed source/target terminals, reconstructs the target from trie actions,
  rejects a mismatched package fingerprint, and proves that the lemma-local
  irregular allomorph is absent from the productive lane;
- the full local Productive V1 gate is `52 passed / 0 failed`; package bytes and
  SHA-256 remain `14,744 B` and
  `336ad181cc8f53e498bbdb824f5a834a3fecdc191751bfa019bfaff13928b185`;
- V52 does not claim fixed-corpus geometry parity, corpus-trained quality,
  composite lane ownership, remote latency/RSS, or live authority;
- V52 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_MMAP_RUNTIME_V52_2026-08-11/receipt.json`.
- V53 implements the bounded shadow composite lattice in
  `productive_v1/composite.rs`: it retains separate `32`-identity grounded and
  productive lanes, groups equal display surfaces without deleting terminal or
  productive identities, protects an original grounded L1.1 Winner from a
  contradictory productive Winner, and emits a typed immutable L3 handoff with
  both verdicts, contradiction certificate, overflow, and integrity state;
- the complete local Productive V1 gate is now `54 passed / 0 failed`; the
  composite micro retains all `32 + 32 = 64` pre-dedup identities with grounded
  identity loss `0` and grounded Winner protection violations `0`; target usage
  is `8,140,705,792 B` under the `12 GiB` Cargo budget;
- V53 remains shadow-only. It does not yet claim conversion into the existing
  L3 `WordCandidate` input, L3/verifier replay, fixed heldout quality, remote
  resources, physical product gates, or live authority;
- V53 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_COMPOSITE_LATTICE_V53_2026-08-11/receipt.json`.
- V54 implements the typed shadow adapter in
  `productive_v1/l3_handoff.rs`: the immutable handoff now carries complete
  grounded and productive records, projects one legacy `WordCandidate` per
  identity rather than flattening to top-32, and lets the existing L3 context
  observer inspect all `32 + 32` candidates while retaining typed provenance in
  an aligned sidecar;
- the adapter assigns neutral legacy `energy/risk` and introduces no manual
  cross-lane weights. Trained productive Q16 scores and grounded L1.1 evidence
  remain in the typed handoff for the later calibrated readout; the adapter is
  context observation only and has no mutation authority;
- the complete local Productive V1 gate is now `56 passed / 0 failed`; the
  handoff micro reports grounded identity loss `0`, grounded Winner protection
  violations `0`, manual cross-lane weights `0`, and target usage
  `8,140,812,288 B` under the `12 GiB` Cargo budget;
- V54 does not yet claim fixed-corpus L3/verifier retention, heldout quality,
  remote resources, physical product gates, or live authority;
- V54 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_L3_HANDOFF_V54_2026-08-11/receipt.json`.
- V55 paper audit found two contracts that were still underspecified before the
  full-corpus driver: imported canonical L2 IDs are zero-based while
  sidecar-owned IDs are one-based, and `NT` competitors are direct contextual
  contradictions rather than feedback; sections 4-6, 19, 21, and 23 now define
  their exact namespaces, one-pass grounding join, evidence ownership, and
  fail-closed verification;
- V55 is a paper closure only. It changes no source or runtime authority and
  claims no full-corpus package or quality result.
- V56 implements the complete local compiler contour in
  `productive_v1/orchestrator.rs` and exposes it through the distinct
  `--compile-productive-paradigm-v1` CLI route. One raw corpus read emits typed
  morphology and context spools; bounded per-shard sorts execute in parallel;
  imported ownership, transition/paradigm induction, context replay, evidence
  fitting, bootstrap packaging, calibration against actual packaged top-32
  candidate sets, final deterministic packaging, and mmap reopen then execute
  without rereading the raw corpus;
- calibration target retention is measured from the actual packaged candidate
  frontier. Any lost calibration target blocks productive authority globally;
  calibration tables are then refitted only from retained actual candidates;
- the complete local `productive_v1::` gate is `62 passed / 0 failed`; the
  repository changed-code gate is `PASS`, and CLI help exposes the complete
  input manifest. The current deterministic micro package is `14,744 B`,
  SHA-256
  `2e085876940e5a9e0b07f3e579770ab106936704949e12e5222cb8b7abbc2193`,
  with `2` paradigms, `301` bindings, `4` programs, `9` operations, `7` trie
  nodes, `5` trie arcs, `3` physical terminals, `5` calibration rows, `2`
  logical runtime terminals, and `124 B` resident runtime cache;
- V56 is a local contour proof only. The canonical V13-aligned v3 corpus
  compile (`434,934,248 B`, SHA-256
  `85d9b5493e22c96569e3b331cc0059ae80853bd98e976c626ca8f791e75f22a6`),
  fixed seen/bank-unseen/slot-heldout/lemma-heldout/ambiguity and 13 damage-class
  proofs, package/RSS/startup/hot-latency budgets, physical application matrix,
  and production authority remain untested. Phase support is packaged but the
  micro selects zero phase centers; the full corpus must determine whether this
  is expected evidence sparsity or an architecture defect;
- V56 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_ORCHESTRATOR_V56_2026-08-11/receipt.json`.
- The first two remote probes reached imported ownership in under one minute
  and failed closed on lemma `я` being proposed as canonical ref `0`. The
  external spool had sorted length-prefixed wire identities, where a short
  UTF-8 lemma precedes ordinary lexical order; canonical L2 assigns lemma refs
  from ordinary normalized string order, where `абажур` is first. The second
  probe showed all `6` source pairs were outside ref `0`, proving this was an ID
  order defect rather than one missing anchor;
- the probes also used stale corpus v1 (`3,255,791` F rows), while V13's receipt
  records `3,255,785` morphology bindings and `2,501,613` same-lemma heldout
  scenes, exactly matching corpus v3. V57 therefore retains the strict complete
  binding-set equality gate, changes the physical spool key to ordinary
  `(language, normalized_lemma)` lexical bytes, and pins the full run to v3;
- V57 changes no live authority and introduces no ownership exception. Full
  compilation restarts from immutable v3 because both the corpus manifest and
  sorted evidence identity differ from the rejected probes;
- V57 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_IMPORTED_OWNERSHIP_V57_2026-08-11/receipt.json`.
- The first corrected v3 raw pass failed closed after `25.46 s` at
  `202,860 KiB` peak RSS because the axis dictionary omitted `imp_excl`. A
  complete corpus-label scan found exactly two missing labels:
  `imp_excl` and `imp_incl`; both already exist in canonical L2 feature masks;
- V58 assigns those two labels to the paper-defined `variant_kind` axis `12`
  as values `2` and `3`. The verb applicability mask already contains axis 12,
  so no slot width, POS mask, coefficient, runtime branch, or authority rule
  changes. Both exclusive and inclusive imperative schema parses pass locally;
- V58 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_AXIS_SCHEMA_V58_2026-08-11/receipt.json`.
- The first Run5 resume with multi-POS basins and exact-only TRAIN exclusions
  reopened the reduced morphology and induction artifacts, completed evidence
  reduce plus bootstrap packaging, and reached actual packaged calibration. It
  then failed closed after `45.72 s` at `730,564 KiB` peak RSS because a
  transferable trie replacement whose end-relative offset did not fit a shorter
  grounded source was reported as package corruption;
- this is a runtime applicability distinction, not a license to weaken package
  validation. Compiler construction still rejects a source span outside the
  template anchor. In-memory, reference, and packaged traversal now prune only
  the current incompatible branch with `Ok(false)`/`Ok(None)`; invalid opcodes,
  anchors, wire ranges, graph ownership, and arithmetic overflow remain
  fail-closed integrity errors;
- the complete local `productive_v1::` gate is now `68 passed / 0 failed`,
  including a regression where a replacement learned from a longer source is
  attempted against a one-scalar source and is pruned without changing its
  cursor. Cargo target usage is `8,988,991,488 B` under the `12 GiB` budget;
- the failed remote run produced no final receipt or accepted package. The
  branch-prune correction has not yet completed the full Run5 resume, fixed
  quality proof, resource gates, or physical product matrix. Runtime authority
  remains unchanged;
- V59 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_BRANCH_PRUNE_V59_2026-08-11/receipt.json`.
- V60 completed the Run5 multi-POS exact-only resume on the remote 20-logical-CPU
  host with `19` configured workers. It reused all raw, sort, ownership,
  induction, and context-spool stages and completed evidence fit, actual packaged
  calibration, final deterministic publication, deep mmap reopen, and SHA-256
  verification in `59.948 s` internal time (`59.97 s` wall), with
  `768,244 KiB` peak RSS;
- the measured shadow package is `36,641,392 B` (`34.94 MiB`), SHA-256
  `5dbf20c68920e721a2763729c478f33fa882e149467493bbd2df69496f991238`,
  mmap-backed, and retains only `124 B` of constant runtime cache. It contains
  `5,606` paradigms, `94,151` lemma/POS bindings, `195,406` programs, `646,762`
  operations, `197,446` trie nodes, `191,840` trie arcs, and `169,790`
  terminals;
- imported identity verification passed over `2,600,857` TRAIN forms/events and
  `74,852` TRAIN lemmas. Induction measured `2,600,857` transition observations,
  `2,575,167` transferable observations, and `25,690` exact-allomorph
  observations. Exact-only exclusion measured `74` forms/events and excluded
  `49` context-occurrence events;
- actual packaged calibration retained the target in only `119 / 1,348` source
  groups (`8.83%`) and lost it in `1,229 / 1,348` (`91.17%`). It fitted `119`
  groups over `3,780` candidate rows. Evidence was also sparse: `20` direct
  contradictions and `20` training pairs, with `0` selected phase centers across
  `129,945` phase profiles;
- the scoped verdict is therefore exactly `PASS_shadow_suggest_only_package`.
  `authority_blocked_by_target_loss=true`; no promotion manifest is admitted and
  runtime authority remains unchanged. Seen, bank-unseen, slot-heldout,
  lemma-heldout, ambiguity, 13 damage-class, false-authority/singleton, startup,
  hot-latency, and physical-product proofs remain untested;
- V60 receipt and resource evidence:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_FULL_PACKAGE_V60_2026-08-11/`.
- V61 added a read-only packaged proof route over the immutable global proof
  spool and ran it on the V60 artifact with `19` workers and `100` selected
  cases per damage class for both `SEEN_EXACT` and `LEMMA_HELDOUT`. It scanned
  `2,501,633` proof events, sampled `4,800` source events per cohort, and
  evaluated `2,600` unique-resolvable cases (`13 x 100 x 2`);
- the measured aggregate `SEEN_EXACT` result is `436 / 1,300` top-1
  (`33.54%`), `833 / 1,300` top-16 and readout retention (`64.08%`), with no
  empty lattices. `LEMMA_HELDOUT` is `20 / 1,300` top-1 (`1.54%`),
  `100 / 1,300` top-16 (`7.69%`), `101 / 1,300` readout retention (`7.77%`),
  and `1,183 / 1,300` empty lattices (`91.00%`);
- all `2,600` shadow readouts abstained. Integrity errors, shadow false
  singletons, Winners, and Tied verdicts were all zero. This is fail-closed but
  not useful restoration: every measured class is below the strict `>95%`
  target-retention contract;
- V61 cold mmap load was `267.379 ms`, proof evaluation was `2.858 s`, sampling
  was `10.789 s`, and process peak RSS was `251,352 KiB`. Per-case latency is a
  closed evaluation diagnostic without queue or product IPC; it does not pass
  or fail the normative single-client or 20-client product latency gate;
- V61 verdict is `FAIL_measured_shadow_gates`, `promotion_eligible=false`, and
  `runtime_authority_changed=false`. `BANK_UNSEEN`, `SLOT_HELDOUT`,
  `MULTI_LABEL`, `UNSUPPORTED`, complete L1.1 grounded-lattice retention,
  L3/L4/DecisionCore/verifier transfer, queue-inclusive latency, and the
  physical product matrix remain untested;
- V61 receipt:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_FIXED_SHADOW_PROOF_V61_2026-08-11/receipt.json`.
- V62 reran the same frozen V61 proof with three additional read-only birth
  denominators. For `SEEN_EXACT`, the target lemma was born in `1,300 / 1,300`
  cases, while the target slot and exact target were both born in only
  `833 / 1,300`; all `833` exact births reached top-16. The `467` losses are
  therefore pre-top-16 slot-birth losses, not top-32 ranking losses;
- for `LEMMA_HELDOUT`, the target lemma was born in only `117 / 1,300` cases,
  the target slot and exact target in `101 / 1,300`, and the lattice was empty
  in `1,183 / 1,300`. This confirms that the paper-required complete
  `LexicalLemmaObservationV1 -> ColdLemmaBindingV1` route is absent from the
  packaged implementation; changing ranking coefficients cannot repair it;
- V62 changes no package bytes, coefficients, calibration, or authority. The
  parked next implementation point is the general lexical-observation/cold-
  binding owner plus geometry-aware bounded slot birth. L1.1, canonical L2,
  verifier, and the `256 / 256 / 16 / 32 / 196608` limits remain unchanged;
- V62 receipt and parking state:
  `docs/structural_gates/receipts/L2_PRODUCTIVE_V1_BIRTH_DIAGNOSTIC_V62_2026-08-11/`.

Normative terms are `MUST`, `MUST NOT`, `SHOULD`, and `MAY`. An implementation
that differs from a `MUST` is a new architecture experiment and requires an
updated design, an explicit receipt, and a full affected proof.

## 1. Exact Product Scope

### 1.1 Supported capability

V1 generates an inflectional surface for a lemma that is already grounded by
the immutable L1.1 or canonical L2 lexical field. The productive sidecar may
generate a form that is absent from the exact terminal bank and absent from the
productive training surfaces for that lemma, provided that a train-learned
paradigm licenses the transition.

The central supported case is:

```text
known lexical lemma
+ observed source form or grounded lemma binding
+ train-learned compatible paradigm
+ context-supported target morphology slot
-> generated inflectional surface
```

### 1.2 Explicit non-goals

V1 does not claim to:

- invent an entirely unknown lemma that has no L1.1 or canonical L2 grounding;
- perform unrestricted derivational word formation;
- generate a semantic neologism;
- replace L3 sentence interpretation;
- apply an edit without the downstream verifier.

Comparatives, participles, and other productive families are admitted only when
their corpus annotations, edit programs, and heldout cohorts pass the same gates
as ordinary inflection. Their presence in a paradigm does not expand the product
scope to arbitrary derivation.

### 1.3 Four distinct meanings of unseen

Every receipt MUST label one of these scopes:

```text
BANK_UNSEEN
  target is absent from every exact L1.1/canonical-L2 terminal bank consulted by
  the productive runtime, but may have been present in productive sidecar
  training

SLOT_HELDOUT
  target slot/surface for a known lemma was excluded from productive training;
  other forms of the lemma may be present

LEMMA_HELDOUT
  all productive training records for the target lemma were excluded; the
  immutable lexical layer may still ground the lemma identity

CORPUS_UNSEEN
  lemma and surface are absent from all lexical and productive training; out of
  scope for V1 and MUST NOT be reported as a V1 success
```

`BANK_UNSEEN` is a mechanism result, not a generalization result. Promotion
quality is measured separately on `SEEN_EXACT`, `SLOT_HELDOUT`, and
`LEMMA_HELDOUT` cohorts.

## 2. Non-Negotiable Invariants

The implementation MUST preserve all of these invariants.

| ID | Invariant |
|---|---|
| I-01 | Broad lemma frontier remains 256. |
| I-02 | Active lemma frontier remains 256. |
| I-03 | Morphology features per lemma remain 16. |
| I-04 | Productive form lane remains 32. |
| I-05 | Atom relation budget remains 196,608. |
| I-06 | Every grounded L1.1 candidate survives in a protected lane. |
| I-07 | A grounded L1.1 Winner is downgraded only by an independently calibrated contradiction certificate. |
| I-08 | Productive morphology alone never erases a competing grounded lemma basin. |
| I-09 | No literal word, lemma, suffix fixture, application, damage class, or proof ID appears in runtime branches. |
| I-10 | SafetyGate, edit-plan validation, and verifier authority are not weakened. |
| I-11 | Candidate identity, evidence provenance, and tie/abstain reason remain inspectable. |
| I-12 | Same package, input, configuration, and delta generation produce byte-identical output. |
| I-13 | Raw corpora are each read once; later stages replay typed spools, not raw files. |
| I-14 | L1.1 and canonical L2 packages are not recrystallized for this sidecar. |
| I-15 | Generated candidates remain SuggestOnly until every conjunctive gate passes. |

The fixed `32` limit is the productive lane, not permission to evict L1.1. The
runtime handoff is a bounded composite lattice:

```text
CompositeL2Lattice
  grounded_lane   = complete bounded L1.1 input lattice
  productive_lane = exact top-32 after evaluation of the complete logical
                    productive terminal set
```

Surface dedup may merge display strings, but it MUST retain every contributing
identity and MUST preserve the protected grounded identity.

## 3. Layer Ownership

### 3.1 Input scene encoder

The scene encoder is a read-only sensor. It emits `L2LocalSceneV1` and has no
candidate birth, ranking, readout, or edit authority.

```text
L2LocalSceneV1
  current token bytes and normalized scalar sequence
  nearest left token 1
  nearest left token 2
  nearest right token 1 when already committed
  nearest right token 2 when already committed
  boundary kind before and after current token
  punctuation kind and adjacency
  script/layout observations
  typed local morphology observations
  continuation state for the current preedit
```

The local window is exactly two committed lexical tokens on each side. A phrase
boundary stops the window. No sentence embedding, topic state, distant token,
L3 score, L4 goal state, or verifier decision may enter `L2LocalSceneV1`.

### 3.2 L1.1

L1.1 owns damaged-signal lexical grounding. Productive L2 consumes the complete
bounded L1.1 lattice and its original `Winner | Tied | ABSTAIN` verdict.

Canonical L2 exposes a read-only `LexicalLemmaObservationV1` for a grounded
lemma:

```text
lemma_id
known POS domains
known exact source form identities
complete morphology slots when available
canonical source-form preference
```

This view is immutable lexical input, not productive training. It is required
to infer a cold binding for a `LEMMA_HELDOUT` lemma. The proof manifest records
the exact lexical observations exposed for every heldout target.

### 3.3 Productive L2

Productive L2 owns:

- known-lemma to compatible-paradigm binding;
- generation of licensed inflectional surfaces;
- same-lemma morphology-slot competition;
- bounded local syntax/morphology evidence;
- productive `Winner | Tied | ABSTAIN` readout;
- contradiction certificates for a grounded L1.1 Winner.

Productive L2 does not own full-sentence semantics. Productive evidence may
settle candidates inside one lemma basin. Across lemma basins it may annotate
support, but unique cross-lemma authority requires the existing independent
canonical L2 lexical evidence or L3 evidence.

### 3.4 L3

L3 receives the immutable composite L2 lattice. L3 owns evidence involving:

- more than two lexical tokens on either side;
- sentence-wide agreement or dependency evidence;
- semantic compatibility and discourse continuation;
- cross-lemma interpretation unresolved by local lexical evidence.

L3 MUST NOT feed a score or selected candidate back into productive L2. It may
produce a downstream attributed candidate through the existing L3 contract.
This prevents cycles and double counting.

### 3.5 Verifier

The verifier checks only structural safety of the proposed transition. It does
not repair missing birth, ranking, target retention, or calibration.

## 4. Typed Identities

Identity namespace is part of the type, not an undocumented integer convention:

```text
ImportedL11TerminalRefV1       owning L1.1 terminal index; base-package semantics
ImportedCanonicalL2LemmaRefV1 owning canonical L2 lemma index; zero-based, zero valid
ImportedCanonicalL2FormRefV1  owning canonical L2 form index; zero-based, zero valid
SidecarSlotIdV1               dense one-based ID; zero means absent
SidecarParadigmIdV1           dense one-based ID; zero means absent
SidecarProgramIdV1            dense one-based ID; zero means absent
SidecarPoolRefV1              checked one-based or typed byte reference as specified
```

Sidecar-owned IDs are deterministic dense `u32` values. ID zero is invalid for
a present sidecar identity and IDs are assigned after lexicographic sorting of
canonical serialized keys. Imported L1.1 terminal IDs, canonical L2 lemma refs,
form refs, and decoder refs retain their owning base-package values and are
interpreted only under the header fingerprints and checked base-package counts.
The sidecar never adds one, subtracts one, or otherwise renumbers an imported
identity. An optional imported reference uses an explicit presence flag or
`Option`, never integer zero as an absence sentinel.

Source fields implemented before V55 and named only `lemma_id` or `form_ref`
MUST be migrated to the typed imported wrappers wherever canonical L2 owns the
value. A serializer still writes the wrapped `u32`, so this clarification does
not enlarge fixed records. Validation of an imported zero is valid; validation
of an out-of-range imported reference or a package-SHA mismatch fails closed.

### 4.1 Morphology axis encoding

`MorphologySlotKeyV1` is exactly 16 bytes:

```text
offset  bytes  field
0       1      part_of_speech
1       1      number
2       1      case
3       1      gender
4       1      person
5       1      tense
6       1      mood
7       1      aspect
8       1      voice
9       1      form_kind
10      1      degree
11      1      animacy
12      1      variant_kind
13      3      reserved, zero
```

For every axis:

```text
0 = INAPPLICABLE
1 = UNKNOWN_OR_UNANNOTATED
2..255 = value from the versioned axis dictionary
```

`INAPPLICABLE` and `UNKNOWN_OR_UNANNOTATED` are never equal. A slot key is valid
only when the POS-specific applicability mask agrees with every
`INAPPLICABLE` value. Invalid combinations fail compilation.

The event wire deliberately carries typed values, not human-readable labels.
Compilation therefore has one additional mandatory, immutable input:

```text
MorphologyAxisSchemaV1
  schema_version
  for each POS value: exact 13-bit applicability mask
  for every encoded value 2..255: (axis, value, normalized UTF-8 label)
  canonical SHA-256
```

The schema is declared in the raw-source manifest, read once, and its canonical
bytes contribute to `training_manifest_sha256`. The compiler MUST validate every
event slot against the POS applicability mask and MUST emit all referenced
labels into `AXIS_DICTIONARIES`. It MUST NOT infer applicability from whichever
axes happen to be nonzero in one event, invent labels from numeric values, or
use a built-in Russian/English POS table. Missing, duplicate, conflicting, or
unreferenced dictionary values fail compilation.

### 4.2 Form identity

Multiple valid forms in one complete morphology slot remain separate:

```text
FormIdentityV1
  canonical_l2_lemma_ref
  canonical_l2_form_ref
  sidecar_morphology_slot_id
  variant_id
```

This identity denotes a grounded exact canonical-L2 form. Both imported refs are
zero-based and may equal zero. `sidecar_morphology_slot_id` and `variant_id` are
one-based. `variant_id` is assigned by normalized surface bytes and provenance
bytes. It does not rank variants. Syncretic forms that share bytes but represent
different slots also remain distinct until display dedup.

For a dynamically generated productive candidate,
`ProductiveCandidateIdentityV1.normalized_surface_id` is the terminal's
nonzero `stable_identity_hash`. The compiler derives the initial value from the
paradigm, program, target slot, variant, and canonical transition bytes. If the
32-bit value is zero or already occupied, it appends a deterministic nonzero
probe counter to those same canonical bytes and rehashes until it reserves a
free nonzero value. Probe order follows the canonical terminal compile order,
so identical inputs produce identical package bytes. The complete identity
also carries `lemma_id`, so one transferable terminal applied to two lemmas
remains two different candidate identities without hashing request-time UTF-8
bytes.

The generated normalized UTF-8 surface is still retained in the bounded
request arena and is used as the final total-order key and for display dedup.
Runtime MUST NOT derive authority identity from a truncated hash of those
request-time bytes. If a reserved terminal identity is zero, duplicated, or
inconsistent with its package attribution, package loading fails closed. A raw
32-bit SHA prefix collision is resolved by the compiler and is not itself a
package failure.

Lemma-local exact allomorphs remain in the grounded/exact lane with their
imported exact form identity. They are stored as binding-owned programs and
MUST NOT also appear as productive-trie terminals. This prevents one irregular
surface from acquiring both exact and generated ownership.

### 4.3 Lemma binding

```text
LemmaParadigmBindingV1
  canonical_l2_lemma_ref
  paradigm_id
  canonical_source_form_ref
  observed_slot_set_ref
  positive_support
  explicit_anti_support
  stability
  provenance_ref
  flags
```

A lemma may bind to several compatible paradigms. Incomplete evidence is not
forced into one paradigm.

### 4.4 Imported canonical L2 ownership manifest

Productive compilation has one immutable base-identity input:

```text
ImportedCanonicalL2ManifestV1
  canonical_l2_package_sha256
  canonical_l2_format_version
  canonical_l2_form_count
  canonical_l2_lemma_count
  canonical_l2_binding_count
  canonical_l2_source_corpus_sha256
  canonical_l2_source_corpus_bytes
  canonical_l2_compiler_contract_version
```

The source corpus entry MUST match the corpus provenance used by the installed
canonical L2 package. The package itself remains read-only and is not
recrystallized.

The raw pass resolves normalized surface bytes to the package's zero-based form
ref and carries the canonical legacy feature mask. Lemma ownership is finalized
after external sorting, not guessed per row:

```text
all F rows from the one raw pass
-> retain only rows whose exact surface resolves in canonical L2
-> group normalized lemma bytes in canonical lexical order
-> propose the same zero-based lemma ref as the canonical compiler contract
-> compare the complete sorted (form_ref, legacy_feature_mask) binding set
   against that lemma ref's actual canonical L2 binding range
-> require every canonical binding and every admitted source lemma to be claimed
   exactly once
```

Lexical order proposes the join; the immutable package binding set proves it.
A missing form, ambiguous ownership, repeated claim, unclaimed binding, count
disagreement, or byte-level package mismatch aborts compilation. Productive V1
MUST NOT assign a fresh dense lemma ID and MUST NOT accept lexical order alone as
proof of ownership.

## 5. Training Data Contract

The compiler consumes five different typed sources. They MUST NOT be flattened
into one ambiguous corpus.

### 5.1 Morphology events

```text
MorphologyEventV1
  language
  lemma bytes
  surface bytes
  imported canonical L2 form ref
  canonical legacy feature mask
  complete MorphologySlotKeyV1
  frequency/support
  provenance
```

These events teach form groups, paradigm bindings, and edit programs. They do
not provide sentence context unless the source explicitly carries a separately
validated `ContextOccurrenceEventV1`.

### 5.2 Context occurrence events

```text
ContextOccurrenceEventV1
  target imported canonical L2 lemma and form refs
  target morphology slot
  L2LocalSceneV1 features
  source document/event identity
  support
  provenance
```

These events train local slot evidence. Full-sentence features are excluded and
belong to L3 training.

### 5.3 Direct context contradiction events

An `NT` row emits its positive target as a `ContextOccurrenceEventV1` and emits
one independent direct contradiction event containing all exact imported
identities licensed by each explicitly named competitor surface:

```text
ContextContradictionEventV1
  target imported canonical L2 lemma and form refs
  target morphology slot
  L2LocalSceneV1 features
  competitor identities:
    imported canonical L2 lemma ref
    imported canonical L2 form ref
    canonical legacy feature mask
  source document/event identity
  support
  provenance
```

The competitor list is sorted and deduplicated by complete imported identity.
Every competitor surface MUST resolve through the immutable canonical L2
package. A homonymous surface retains every exact binding identity; display
dedup does not collapse evidence ownership. The event contributes positive
evidence to the target and direct explicit-anti/hard-negative evidence only to
the listed competitors in that scene. It MUST NOT be represented as user
feedback, an unlabeled alternative, a generic absence penalty, or a hand-tuned
coefficient.

`NH` uses the same resolved target and competitor identities only in a read-only
proof event. No `NH` count enters train, calibration, phase fitting, priors, or
threshold selection.

### 5.4 Feedback events

```text
FeedbackEventV1
  proposal identity and package generation
  visible input
  proposed form identity
  user outcome: accept | continue | revert | replace | ignore
  resulting committed surface when observable
  local scene
  timestamp bucket
  provenance
```

Only explicit outcomes become anti or contradiction evidence. Non-selection,
another valid positive, and lack of observation are not anti-evidence.

### 5.5 Proof events

Proof events carry all valid target identities and, for `NH`, all explicit
invalid competitor identities. They are immutable and read-only. They never
enter training, calibration, package induction, delta learning, or threshold
selection.

### 5.6 Deterministic split

Productive split ownership is by lemma, not by row:

```text
h = stable_hash(language || normalized_lemma || split_seed) mod 10,000
0..7,999     TRAIN
8,000..8,999 CALIBRATION
9,000..9,999 HELDOUT_LEMMA
```

All productive records of a lemma follow the same split. The immutable lexical
package may still contain a heldout lemma so that the proof tests paradigm
transfer rather than unknown-lemma invention.

For TRAIN-only model selection:

```text
inner_fold = stable_hash(language || normalized_lemma || split_seed ||
                         "productive-v1-inner") mod 5
```

Every event and candidate group for a lemma remains in that lemma's fold.

The split set is computed before context-event emission. A context occurrence
containing a `CALIBRATION` or `HELDOUT_LEMMA` lemma in any target or neighbor
position is excluded from TRAIN slot/phase aggregation. This prevents a heldout
surface or identity from returning through neighbor features. The immutable
lexical observation remains available only through the explicitly attributed
runtime input described in section 3.2.

`SLOT_HELDOUT` is a second deterministic cohort inside TRAIN lemmas. At least one
eligible target slot is selected by stable hash and removed from all productive
training structures for that lemma. A lemma is eligible only when at least three
complete slots remain, the canonical source slot remains visible, and removing
the target does not remove all evidence for its POS. The heldout target form is
absent from every exact runtime terminal bank used by the proof and remains
available only to the proof oracle.

The proof enforces this with a read-only exact-terminal exclusion mask bound to
the cohort manifest SHA-256. It does not recrystallize or alter the installed
L1.1/canonical-L2 packages. Any masked identity reaching candidate birth is an
annotation leak and fails the proof.

For `LEMMA_HELDOUT`, no productive event for the lemma is allowed. After all
paradigms and coefficients are frozen, the compiler MUST derive a marked
`ColdLemmaBindingV1` for every eligible base lexical lemma that lacks a trained
binding, using only `LexicalLemmaObservationV1` and train-learned paradigms.
Runtime uses the same deterministic procedure for a lemma introduced by a later
lexical overlay:

```text
for each train-learned paradigm with matching POS:
  require every exposed lexical source slot to be compatible
  instantiate all source ranges against the exposed canonical source
  reject any invalid range or conflicting exposed exact form
  retain every compatible paradigm as a cold binding
```

If lexical input lacks POS, source slot, or a valid anchor, productive L2
abstains and the proof counts target loss. Heldout target bytes and slot labels
never participate in cold binding.

No threshold, feature, paradigm, edit program, or delta may use heldout records.

### 5.7 Event identity and deduplication

Every normalized typed event has:

```text
event_sha256 = SHA-256(
  event_schema_version || event_kind || canonical field bytes || provenance)
```

Canonical integers are little-endian. Canonical strings are `u32 byte length`
followed by normalized UTF-8 bytes; optional values have a leading `u8` presence
flag. Identical full hashes are idempotent and contribute once. Events with the
same semantic key but different provenance remain
separate observations and their support is added during the sorted reduce.

Physical external-sort order is
`(event_kind, primary_identity_bytes, split, event_sha256)`. This keeps all raw
morphology observations for one lemma contiguous so imported identity ownership
can be proved across every split without rereading the corpus. The split remains
an immutable event field and reducers admit only their declared split. Any
truncated hash used as an index is accompanied by collision comparison against
the full 32 bytes. A collision never merges events or package identities.

## 6. One-Raw-Pass Crystallization

One-pass means one sequential read of each declared raw source. It does not mean
that calibration is computed before model parameters exist.

```text
each raw source exactly once
-> validate and normalize
-> resolve exact surfaces through immutable canonical L2
-> emit typed event into bounded sharded spool
-> external deterministic sort
-> imported canonical identity join and complete binding verification
-> train reduce
-> freeze model coefficients
-> calibration replay from typed calibration spool
-> freeze authority tables
-> package assembly
-> read-only heldout proof
```

### 6.1 Sharding

The spool shard is selected by stable hash of the primary identity. Shard count,
split seed, compiler version, and normalization version are package inputs.
Memory is bounded by configured write buffers. A shard record contains a CRC and
monotonic sequence number.

Independent shards MAY be sorted concurrently by a bounded compiler-owned
worker pool. Each shard retains its canonical input index and produces its own
deterministic run/merge result. The final cross-shard merge remains ordered by
the normative event key, so worker scheduling cannot change package bytes.

The raw-source manifest records absolute source role, content SHA-256, byte
count, schema version, and provenance owner. Two manifest entries with the same
content SHA-256 are rejected unless the manifest explicitly marks one as an
idempotent alias, in which case it is read zero additional times.

### 6.2 Train reduce order

The deterministic reducer performs:

1. imported canonical lemma/form ownership join and complete binding verification;
2. lemma form grouping with preserved zero-based canonical L2 lemma refs;
3. edit-template extraction;
4. paradigm signature induction;
5. lemma-to-paradigm compatibility;
6. productive prefix-trie compilation;
7. grounded local scene feature extraction;
8. positive, explicit anti, hard-negative, ambiguity, and contradiction count aggregation;
9. model fitting;
10. fixed-point coefficient quantization;
11. train-only consistency checks.

### 6.3 Calibration replay

Calibration records remain in the typed spool until coefficients are frozen.
The compiler first publishes a checked bootstrap package, reopens its mmap
runtime, and scores each typed calibration group through the actual packaged
top-32 candidate traversal. It records target-retained and target-lost groups;
any target loss blocks productive authority rather than being hidden by a
margin fit. The compiler then refits margin/tie tables from the retained actual
candidate sets, publishes the final package, and reopens it through the same
checked mmap path. It never rereads the raw corpus.

### 6.4 Determinism

All hash maps are converted to sorted vectors before serialization or floating
point reduction. Model fitting uses a fixed seed, fixed iteration order, fixed
stopping rule, and one compiler-owned numeric implementation. Two clean builds
MUST produce the same package SHA-256.

## 7. Paradigm Induction

### 7.1 Canonical anchor

For each lemma, the anchor form is selected by this total order:

1. highest train support;
2. complete slot annotation before unknown annotation;
3. smallest serialized `MorphologySlotKeyV1`;
4. shortest normalized scalar length;
5. normalized UTF-8 bytes;
6. provenance ID.

No linguistic label is hardcoded into the choice.

### 7.2 Edit template

Every source-to-target alignment produces an `EditTemplateV1` over Unicode
scalars. The bounded instruction set is:

```text
COPY_SOURCE_RANGE(start_anchor, start_delta, scalar_count)
DROP_SOURCE_PREFIX(scalar_count)
DROP_SOURCE_SUFFIX(scalar_count)
EMIT_SEGMENT(segment_id)
REPLACE_SOURCE_RANGE(end_relative_offset, delete_count, segment_id)
EMIT_EXACT_ALLOMORPH(form_ref)
TERMINATE(slot_id, variant_id)
```

`start_anchor` is `START` or `END`. Offsets are signed 16-bit scalar offsets.
Counts are unsigned 16-bit values. Segment bytes are normalized UTF-8 stored in
the shared segment pool. For `REPLACE_SOURCE_RANGE` only, `segment_id=0` means
that the addressed source range is deleted without emitting a segment, and
`delete_count=0` with a nonzero `segment_id` means an internal insertion. Both
fields zero is an invalid no-op. `EMIT_SEGMENT` always requires a nonzero
reference to a nonempty segment.

`COPY_SOURCE_RANGE.scalar_count = 65,535` is the reserved
`COPY_TO_RETAINED_EDGE` sentinel. It copies from the addressed start through the
last source scalar not declared by `DROP_SOURCE_SUFFIX`; a declared prefix drop
must already place the monotonic source cursor at the addressed start. The
sentinel is required for one paradigm transition to apply to lemma anchors of
different lengths. It is not an observed length: all concrete scalar counts
remain bounded by 65,534. No other instruction accepts this sentinel.

`EMIT_EXACT_ALLOMORPH` is lemma-local and is never included in a transferable
paradigm signature. `form_ref` is the compiler-internal exact-form identity;
when serialized as `MorphOpV1`, it is resolved to the exact normalized
`decoder_ref` in `SEGMENT_POOL`. Runtime package code never interprets a
sidecar-local form identity as an imported canonical-L2 row.

The compiler derives the shortest valid program by scalar edit DP with this
deterministic tie order:

```text
COPY
DROP_EDGE
EMIT_EDGE
REPLACE_INTERNAL
EXACT_ALLOMORPH
```

Program interpretation is exact:

```text
source scalars are immutable
output starts empty
DROP_SOURCE_PREFIX/SUFFIX declare excluded edge ranges and emit nothing
COPY_SOURCE_RANGE appends the addressed source range
EMIT_SEGMENT appends the referenced segment
REPLACE_SOURCE_RANGE consumes the addressed source range and appends the
  referenced segment at that point
EMIT_EXACT_ALLOMORPH appends the exact lemma-local decoder form and must be the
  only emitting instruction
TERMINATE emits nothing and closes the identity
```

All copied/replaced ranges in a transferable program are non-overlapping and
monotonic in source order. Every source scalar is exactly one of copied,
replaced, or explicitly dropped. There is no implicit copy at termination.
Program validation reconstructs the normalized train target byte-exactly; the
separate decoder attribution reconstructs its display surface.

The alignment objective minimizes this tuple lexicographically:

```text
scalar edit cost
number of internal replacements
number of emitted scalar segments
instruction count
serialized instruction bytes
```

`EMIT_EXACT_ALLOMORPH` is considered only after no transition template shared by
at least two distinct TRAIN lemmas reconstructs the target. A transferable
paradigm transition therefore has evidence from at least two train lemmas; the
support count is still a learned score feature rather than manual authority.

The program bound is evidence-derived:

```text
package.max_program_ops = maximum admitted train program length
```

The wire format ceiling is 65,534 operations and scalar length 65,534. A package
whose measured bound reaches the ceiling is rejected. Runtime never silently
truncates a program. A non-transferable program remains an exact lemma-local
allomorph or produces no generated terminal.

### 7.3 Normalized paradigm signature

A transferable transition key is:

```text
source_slot_key
target_slot_key
edit opcode sequence
relative source anchors and counts
inserted segment identities
```

Lemma IDs, source surface bytes, exact form refs, corpus row IDs, and support
counts are excluded from the signature.

A `ParadigmSignatureV1` is the sorted set of transferable transition keys for
one POS domain.

### 7.4 Compatibility

An incomplete lemma signature `L` is compatible with paradigm `P` iff:

```text
POS(L) == POS(P)
and every observed transition in L exists identically in P
and L contains no conflicting target for the same complete source/target slot
and every source range in P is valid for the lemma anchor when instantiated
```

All compatible paradigms are retained. Their support and stability are learned
features, not hard admission weights. If no paradigm is compatible, exact forms
remain available and productive generation abstains.

Two complete paradigms are equivalent only when their sorted transition-key
sets are byte-identical. Approximate clustering is a later experiment and may
not be inserted into V1.

The package contains a compatibility index keyed by `(POS, exposed source slot)`
with postings of candidate paradigm IDs. Cold binding probes the intersection
of postings for every exposed source slot, then executes the complete
compatibility predicate above. It never scans unrelated paradigms and never
truncates the compatible set. Precomputed cold bindings and runtime-derived
cold bindings must have byte-identical semantic identity hashes.

## 8. Productive Prefix Trie

### 8.1 Canonical graph choice

V1 uses a prefix trie. It does not use a suffix-minimized or general convergent
FST for runtime geometry.

This is required because OSA rows, previous generated units, path length, atom
state, and decoder provenance depend on the complete emitted prefix. A future
FST experiment is valid only if traversal identity is the product state
`(fst_node, complete_path_state)` and exact parity plus a measured gain pass.

### 8.2 Stored graph

The package stores a trie of edit-program emission tokens per paradigm. A node
has exactly one parent except the root. Arcs are sorted by:

```text
opcode
source anchor
source offset/count
segment bytes
target slot key
variant id
```

Shared segment bytes are stored once globally. Program prefixes are shared only
when their emitted-token prefixes are identical.

The compiler builds a radix trie and then splits every arc at every branch
longest-common-prefix boundary. This applies to both source-copy ranges and
literal segments. After splitting, one arc emits one maximal scalar run shared
by all descendants, and no two sibling arcs have a non-empty emitted scalar
prefix in common.

### 8.3 Runtime instantiation

For one active lemma binding, trie arcs are interpreted against its canonical
source form. `COPY_SOURCE_RANGE` emits source scalars incrementally;
`EMIT_SEGMENT` emits segment scalars incrementally. No complete candidate UTF-8
string is allocated before a terminal survives productive top-32.

Edit alignment, source ranges, and trie emissions use the normalized scalar
view from section 9.1. Decoder traces retain the attributed display form and
case reconstruction separately. Geometry never mixes display bytes with the
normalized path.

Invalid source ranges invalidate that binding/program and increment a package
integrity counter. They do not fabricate or truncate output.

### 8.4 Logical and physical lattices

Every valid terminal reachable from every admitted active lemma, target slot,
and compatible program is part of the logical productive lattice. V1 evaluates
all of them exactly.

During terminal evaluation, an exact stable min-heap retains the best 32
productive candidates. This is physical readout, not approximate prefiltering.
No branch-and-bound pruning is allowed in V1. Later pruning requires a formally
admissible score upper bound and 100% candidate/order parity.

## 9. Exact Shared Geometry

### 9.1 OSA semantics

V1 uses Optimal String Alignment distance, not unrestricted Damerau-Levenshtein.
For generated prefix `g[1..i]` and observed input `o[1..m]`:

```text
D_i[0] = i
D_0[j] = j

D_i[j] = min(
  D_(i-1)[j] + 1,
  D_i[j-1] + 1,
  D_(i-1)[j-1] + [g_i != o_j],
  D_(i-2)[j-2] + 1
    when i>1, j>1, g_i==o_(j-1), and g_(i-1)==o_j
)
```

The same recurrence runs independently for Unicode scalar units and normalized
keyboard units. The final geometry adapter MUST be score-identical to V39.

V1 normalization is exactly the V39 `compositional.rs` contract: trim outer
whitespace, trim outer `! , . ? ; :`, lowercase, then derive Unicode scalar and
physical keycode-plus-shift sequences. For either lane:

```text
denominator = max(observed_length, generated_length)
similarity_milli = 1000                              when denominator == 0
similarity_milli =
  (denominator - min(OSA_distance, denominator)) * 1000 / denominator
geometry_milli = max(character_similarity_milli, keyboard_similarity_milli)
```

Division is unsigned integer floor, as in V39. A normalization change is not a
productive-trie optimization and requires a separate candidate/quality proof.

### 9.2 Traversal state

```text
GeometryTraversalStateV1
  current character row:  [u16; observed_len + 1]
  previous character row
  current keyboard row
  previous keyboard row
  previous generated scalar
  previous generated key unit
  generated scalar length
  character normalization state
  keyboard normalization state
  AtomAccumulatorV1
  decoder trace reference
  lemma/paradigm/slot/program/variant identity
```

Rows are allocated from a per-request arena sized from the checked observed
length and package maximum output depth. DFS reuses rows after a branch returns.
No state is cached by trie node without its complete parent path identity.

All row values are `u16`. Compiler and runtime reject any observed or generated
length for which the maximum possible distance exceeds `u16::MAX - 1`.

### 9.3 Atom accumulator

`AtomAccumulatorV1` contains only state required to reproduce existing typed
features incrementally:

```text
two start markers plus first character units
last four character units
two start markers plus first keyboard units
last four keyboard units
current scalar position
source/output scalar length counters
boundary pending state
character and keyboard bigram/trigram accumulators
bag-trigram accumulators
distance-2..4 skip-gram accumulators
typed atom refcount table and DFS undo log
expected/shared unique atom weights
character simhash unique-atom refcounts and 64 signed bit counters
keyboard simhash unique-atom refcounts and 64 signed bit counters
support and provenance counters
```

V39 productive parity has exactly eight typed atom channels:

```text
1 character bigram, weight 1
2 character trigram, weight 3
3 keyboard bigram, weight 1
4 keyboard trigram, weight 3
5 character bag trigram, weight 3
6 keyboard bag trigram, weight 3
7 character distance-2..4 skip gram, weight 2
8 keyboard distance-2..4 skip gram, weight 2
```

Each emitted character/key unit updates exactly the atoms that become complete
at that edge. Two V39 start markers initialize each lane; two end markers are
applied temporarily at a terminal. The terminal operation is undone before DFS
continues to a sibling.

Repeated atom keys count once, exactly as V39 `sort_unstable + dedup` and
`BTreeSet` do. The DFS accumulator therefore maintains atom-key refcounts. A
descent records every increment in an undo log; backtracking restores the prior
counts. A `0 -> 1` typed-atom transition updates expected total weight and, when
the atom exists in the observed atom set, shared weight. Other increments do not
change set weight.

At a terminal:

```text
atom_similarity_milli =
  1000 when both unique atom sets are empty
  2000 * shared_weight / (observed_weight + expected_weight) otherwise
```

The result is capped at 1000 with integer floor division. Character and keyboard
simhash use their own unique hash-atom refcounts. On `0 -> 1`, each set bit adds
one and each clear bit subtracts one from the corresponding 64 signed counters;
the terminal bit is one iff its counter is non-negative. This reproduces the
V39 64-bit character plus 64-bit keyboard `SurfaceWaveCode` without materializing
the generated surface.

Atom keys, markers, domains, weights, and `mix64_golden` hashing are inherited
byte-exactly from `src/nanda_wave/l2_field/compositional.rs`. A byte-gram,
boundary-prefix, or L1.1-only channel is not added in V1 speed-parity mode.

The existing observed-surface to lemma atom/wave birth runs before productive
traversal and remains unchanged. Incremental generated-path atom/wave coherence
is a new learned terminal feature and is disabled in speed-only V39 parity mode;
it gains a non-zero coefficient only in the later trained-model stage and must
pass its ablation and full quality gates.

Accumulation order is path order and uses fixed-point integers. Reordering
floating point additions is forbidden in an exact-parity implementation.

### 9.4 Exact top-32 comparator

Speed-only parity mode uses the exact V39 productive tuple:

```text
higher family specificity
higher profile evidence milli
higher positive support
lower anti support
higher geometry evidence milli
stable generated identity
```

No trained feature changes this mode. Candidate identity and order hashes must
match V39 before trained readout is enabled.

The final trained terminal comparator is a total order:

1. higher learned score;
2. higher grounded lemma evidence;
3. lower exact OSA distance;
4. exact form before generated form only when all learned evidence is equal;
5. smaller lemma ID;
6. smaller paradigm ID;
7. smaller slot ID;
8. smaller variant ID;
9. normalized surface bytes.

Identity tie-breakers make ordering deterministic but never create Winner
authority.

### 9.5 Local scene wave and multimodal centers

Productive V1 inherits the canonical L2 width:

```text
L2_SCENE_PHASE_CELLS = 60
positive subcenters  = learned 0..4
anti subcenters      = learned independently 0..4
hard-negative centers = learned independently 0..2
ambiguity subcenters = learned independently 0..8
```

The bounds are package-format limits, not required counts. The compiler selects
the count independently for each bank and typed slot profile.

The scene encoder emits canonical feature records in this order:

```text
left lemma identity at positions -2 and -1 when grounded
right lemma identity at positions +1 and +2 when grounded
POS and applicable morphology axes at each position
boundary and punctuation kind
adjacency and local token-order shape
current script/layout observation
preedit continuation state
```

Service-word behavior is learned through grounded lexical identities and typed
POS/axis evidence. Runtime code contains no literal token, preposition, suffix,
or phrase list.

Each feature is serialized as `(kind:u16, position:i8, flags:u8, value:u64)`.
Feature records are sorted and deduplicated. For each record:

```text
L2_SCENE_V1_SEED = 0x4c32_5343_454e_4531
h = stable_hash64(serialized_feature, L2_SCENE_V1_SEED)
cell_a = h mod 60
cell_b = (cell_a + 17) mod 60
sign = +1 when bit 8 of h is zero, otherwise -1
acc[cell_a] += 9 * sign
acc[cell_b] += 5 * sign
```

Feature kinds are fixed: `1=LEMMA_ID`, `2=POS`, `3=NUMBER`, `4=CASE`,
`5=GENDER`, `6=PERSON`, `7=TENSE`, `8=MOOD`, `9=ASPECT`, `10=VOICE`,
`11=FORM_KIND`, `12=DEGREE`, `13=ANIMACY`, `14=BOUNDARY`,
`15=PUNCTUATION`, `16=ADJACENCY`, `17=SCRIPT_LAYOUT`, and
`18=PREEDIT_CONTINUATION`. Unknown kinds fail package compilation.

The accumulator is checked `i32`. After all features, divide every cell by the
maximum absolute cell and scale to `[-120,120]` with integer round-to-nearest,
ties away from zero. An empty scene is sixty zeros. The same encoder is used for
train, calibration, proof, and runtime.

Each bank is trained independently. Positive observations never imply anti
observations for another valid slot. The candidate center counts `k=0..limit`
are evaluated by the same five lemma-owned inner folds as the linear reducer.
Positive events are direct valid slot observations. Anti events are explicit
reverts, replacements, or direct typed contradictions. Hard-negative events are
POS/axis combinations rejected by the versioned morphology schema independent
of corpus frequency. Ambiguity events contain two or more directly valid slots
for the same observable L2 scene. Lack of an observation creates no event.
For each `k>0`, deterministic clustering is:

```text
sort waves by event identity
center 1 = smallest event identity
next center = wave with lowest maximum cosine to existing centers
  ties by event identity
repeat assignment/recompute for at most 32 rounds
assignment = highest integer cosine, ties by center index
recompute = component-wise checked i64 mean, normalized to [-120,120]
stop when assignments are unchanged
```

Seed selection considers only observations not already selected as seeds. The
component mean uses signed integer division toward zero and is then normalized
with the same round-to-nearest, ties-away-from-zero rule as the scene encoder.
A candidate `k` that produces an empty cluster after assignment is
inadmissible; an empty cluster is never repaired by copying, jittering, or
merging another center. This makes duplicate modes fall back to the smaller
`k` instead of introducing an implementation-defined center.

Integer cosine is
`1_000_000 * dot(a,b) / max(1, isqrt(dot(a,a)*dot(b,b)))`. All sums and the
integer square root are deterministic. A zero vector has coherence zero.

The chosen `k` minimizes inner-fold candidate ranking loss for positive/anti/
hard banks and maximizes valid-set retention subject to zero false singleton for
the ambiguity bank. Ties choose smaller `k`. If no `k>0` improves every fold
over `k=0`, the bank is absent. Positive, anti, hard-negative, and ambiguity
events are never clustered together.

The fold objectives are exact. Define `q(w,C)` as the maximum integer cosine
between wave `w` and center bank `C`, or zero when `C` is empty. A ranking
selection group contains one or more bank-member waves and only independently
typed comparator waves from the same observable scene. Positive-bank members
are direct valid-slot observations and comparators are explicit anti or
schema-hard observations. Anti-bank members are explicit anti observations and
comparators are direct valid observations. Hard-negative-bank members are
schema-hard observations and comparators are direct valid observations. An
unknown or merely unselected candidate never becomes a comparator.

For every member/comparator pair the loss is `0` when member coherence is
higher, `1/2` when equal, and `1` when lower. Pair losses are averaged inside a
scene group so each group has total weight one; fold loss is the mean group
loss. A fold without at least one independently licensed pair makes every
`k>0` inadmissible for that slot bank. A ranking-bank `k>0` is eligible only
when its loss is strictly lower than `k=0` in all five folds. Eligible values
are ordered by mean fold loss and then smaller `k`.

An ambiguity selection group contains all directly valid slot identities for
one scene and has at least two identities. A valid identity is retained when
its own slot-profile bank has `q(w,C)>0`. The group is a false singleton when
exactly one valid identity is retained. A fold with no ambiguity group makes
the ambiguity bank absent. An ambiguity `k>0` is eligible only when every fold
has zero false singleton and strictly greater retained-valid fraction than
`k=0`; eligible values are ordered by total retained-valid fraction descending,
then total coherence descending, then smaller `k`. These are TRAIN inner-fold
objectives. Calibration and proof labels never select `k`.

`PhaseCenterV1.support` stores the assigned unique-event count capped at
`65,535`; flag bit 0 (`SUPPORT_SATURATED`) records that the exact count was
larger. `mass` stores the center's rounded share of its bank on `[0,65,535]`.
The exact uncapped counts remain compiler evidence and are used by fitting;
the two `u16` fields are package diagnostics and never grant authority.

`SlotPhaseProfileV1.support` is the deduplicated TRAIN positive count for its
slot and `SlotPhaseProfileV1.explicit_anti_support` is the independently typed
TRAIN revert, replacement, or impossible-slot count for the same slot. The
latter is not inferred from missing positives or from another valid form.
These two counts feed features 4 and 5 through the global slot prior channel.

## 10. Learned Evidence Model

### 10.1 Feature vector

Every terminal produces this typed feature vector:

```text
phi(candidate, observed, scene)
  1  lemma positive log evidence
  2  lemma contradiction magnitude
  3  paradigm compatibility log evidence
  4  slot positive log evidence
  5  slot explicit anti magnitude
  6  maximum positive-center coherence
  7  maximum anti-center coherence
  8  maximum hard-negative-center coherence
  9  normalized character geometry
  10 normalized keyboard geometry
  11 atom/phase coherence
  12 directional positive residual
  13 directional anti residual
  14 log support
  15 stability
```

Missing observation is represented as neutral zero after prior centering. It is
not anti-evidence. Center coherence features are
`max(0, maximum_integer_cosine / 1_000_000)`. Ambiguity-center coherence and
exact/generated provenance are readout/calibration inputs, not ranking bonuses.
At runtime, positive ambiguity-bank coherence marks the observable calibration
stratum as same-lemma multi-label evidence. Candidate-set structure separately
adds syncretic-slot and cross-lemma bits. This evidence changes neither the
15-feature score nor slot ranking; the measured calibration tie envelope owns
its effect on readout.

The numeric feature scale is exact:

```text
features 1..5, 12, 13 = centered smoothed log odds from section 10.2
features 6..8          = max(0, integer_cosine) / 1_000_000
features 9, 10         = similarity_milli / 1000
feature 11             = atom_similarity_milli / 1000
feature 14             = log(1 + support)
feature 15             = stability / 65535
```

`support` in feature 14 is the candidate's train-only positive support after
event deduplication. Every division above is evaluated in the reference `f64`
implementation and then quantized only at the section 10.5 boundary. A missing
count pair, center, residual, support, or stability value emits exact zero.

For a packaged productive terminal, feature 14 is its owning
`SlotPhaseProfileV1.support`. Feature 15 is
`min(binding.stability, paradigm.stability)` only when both values are nonzero;
if either component is unmeasured, feature 15 is exact zero. Runtime does not
substitute morphology frequency, lemma ID, or a manually selected default.

### 10.2 Evidence normalization

Count evidence is converted to a centered smoothed log odds using train-only
counts:

```text
log_evidence_k =
  log((positive_k + 0.5) / (contradiction_k + 0.5))
  - log(train_positive_prior_k / train_contradiction_prior_k)
```

For each paired channel, the supportive magnitude is
`max(0, log_evidence_k)` and the explicit negative magnitude is
`max(0, -log_evidence_k)`. If both candidate counts are absent, both magnitudes
are exact zero before prior centering. A one-sided feature such as paradigm
compatibility retains only the supportive magnitude. Priors are strictly
positive TRAIN totals after Jeffreys smoothing; calibration and proof counts do
not enter them.

The `0.5` Jeffreys prior is global and is not assigned per word or proof class.
Geometry remains in its exact normalized V39 scale and receives a learned
coefficient.

The four TRAIN prior channels are serialized in `EVIDENCE_PRIORS` in this
fixed order:

```text
1 lemma
2 paradigm compatibility
3 slot
4 directional residual
```

Each row stores `positive_prior_twice = 2 * raw_positive_count + 1` and
`contradiction_prior_twice = 2 * raw_contradiction_count + 1`. The two values
are nonzero odd integers, so the Jeffreys-smoothed ratio is represented exactly
without package floating point:

```text
train_positive_prior / train_contradiction_prior
  = positive_prior_twice / contradiction_prior_twice
```

All four rows are mandatory for `PRODUCTIVE_V1_MODEL`, sorted by channel ID,
and absent from `V39_SPEED_PARITY`. Missing, repeated, even, zero, or overflowed
prior values reject compilation or package loading. Runtime MUST read these
rows; deriving priors from the active candidate set or assigning them manually
is forbidden.

Directional residual rows use this single train/runtime scene identity:

```text
source_scene_key = low_u32(SHA-256(
  "lay-productive-directional-scene-v1\\0" || canonical_scene_bytes))
```

Zero is invalid and fails event reduction rather than being salted. The
canonical scene bytes are exactly the section 9.5 input encoding. Runtime uses
a binary lookup by `(source_scene_key, from_slot_id, to_slot_id)`; it does not
derive the key from display text or application identity.

### 10.3 Score

Higher score is better. Feature IDs `1,3,4,6,9,10,11,12,14,15` are supportive;
IDs `2,5,7,8,13` are explicit negative magnitudes:

```text
S_theta(c, x) =
  sum_(k in supportive) theta_k * phi_k
  - sum_(k in negative) theta_k * phi_k
```

Every `theta_k` is constrained non-negative. Explicit anti and contradiction
magnitudes are subtracted by construction. Missing evidence cannot lower a
score.

Surface or lemma identity interactions are forbidden. V1 adds no free
polynomial interactions to the final linear reducer. The
context-slot phase profile is itself the typed learned interaction between an
L2 scene and a morphology slot. Any additional interaction changes the feature
schema version and requires a paper amendment plus the full ablation gate.

### 10.4 Training objective

Training groups contain all valid candidates for one scene. Multi-label gold
forms are a positive set. Pairwise examples are generated only between:

- a gold-valid form and an explicitly contradicted form;
- the corrected/accepted form and an explicitly reverted proposal;
- a structurally valid form and a schema-grounded impossible slot.

Two uncontradicted positive or unknown forms never become a negative pair.

Coefficients minimize deterministic L2-regularized pairwise logistic loss:

```text
L(theta) = sum_(a,b) w_ab * log(1 + exp(-(S(a)-S(b))))
           + lambda * ||theta||_2^2
```

For a scene group with `p` admitted pairs, every pair weight is `w_ab = 1/p`.
Thus one scene has total weight one and a highly ambiguous scene cannot dominate
training merely by producing more pair combinations. A group with no explicit
positive-versus-contradicted pair contributes no ranking loss.

Count smoothing is the fixed Jeffreys prior `alpha=0.5`. `lambda` is selected
from `{0, 2^-12, 2^-11, ..., 2^8}` by five deterministic lemma-owned inner
folds. The selected value has the lowest mean inner-fold pairwise log loss;
ties choose the larger lambda and then the lexicographically smaller serialized
coefficient vector. Calibration and heldout cohorts do not select it.

For a lambda tie, the serialized vector is the concatenation of the five
validation-fold coefficient vectors in ascending fold order. The final packaged
vector is then refitted once on all TRAIN pairs using the selected lambda.

The reference optimizer is projected cyclic coordinate Newton descent:

```text
theta = 0
for sweep in 0..128:
  for feature_id in ascending order:
    compute gradient and diagonal Hessian over pairs in stable event order
    theta_k = project_sign(theta_k - gradient / (hessian + 2*lambda))
  stop only after a full sweep when max absolute coefficient delta < 2^-24
reject compilation if 128 sweeps do not converge
```

`project_sign` clamps every stored coefficient to `[0,+infinity)`; feature
polarity in section 10.3 determines addition or subtraction. The
reference calculation uses IEEE-754 `f64`, round-to-nearest ties-to-even, no
fast-math, a pinned pure-Rust `log/exp` implementation, and no parallel floating
point reduction. A zero diagonal Hessian leaves the coefficient unchanged.
Package bytes are owned by the quantized coefficient vector, not by
host-specific intermediate bits.

### 10.5 Quantization

Final features and coefficients are quantized to signed Q16.16. Score reduction
uses signed `i64` with checked saturation forbidden: overflow rejects package
compilation. Quantized train ordering must match the fitted reference on every
train and calibration candidate group before packaging.

## 11. Calibration And Readout

### 11.1 Calibration strata

Calibration is fitted on disjoint calibration lemmas. Runtime never selects a
threshold from an oracle proof label. A stratum key is computed from observable
candidate provenance:

```text
derived transition class
EXACT | TRAINING_SEEN_GENERATED | UNOBSERVED_LEMMA_SLOT | COLD_LEMMA_BINDING
support bin
ambiguity kind
```

The derived transition class is produced by the same deterministic
`classify_typing_transition(observed, decoded_candidate)` adapter in calibration,
heldout proof, and runtime. Proof damage labels are reporting denominators only.

Proof cohorts map to observable provenance as follows:

```text
SEEN_EXACT    -> EXACT
BANK_UNSEEN   -> TRAINING_SEEN_GENERATED
SLOT_HELDOUT  -> UNOBSERVED_LEMMA_SLOT
LEMMA_HELDOUT -> COLD_LEMMA_BINDING
```

The support bin is
`min(15, floor(log2(max(1, minimum_independent_support))))`, where independent
support is the minimum of lemma, paradigm, and applicable slot support. The
ambiguity kind is a bitset for syncretic slot, same-lemma multi-label,
cross-lemma basin, and generated overflow.

Sparse strata back off in this exact order:

1. remove support bin;
2. merge `UNOBSERVED_LEMMA_SLOT` and `COLD_LEMMA_BINDING` into
   `TRAINING_UNSEEN`;
3. remove derived transition class;
4. global generated/exact stratum.

No stratum with fewer than 200 calibration groups may grant unique authority.

### 11.2 Winner threshold

For each stratum, sort groups by descending leader margin. The authority
threshold is the lowest margin prefix satisfying all of:

```text
calibration groups >= 200
wrong unique winner count == 0
false singleton count == 0
grounded Winner protection violations == 0
```

If no prefix satisfies the conditions, that stratum has no Winner threshold and
returns `Tied` or `ABSTAIN`. Thresholds are evidence-derived; runtime constants
do not assign a word, suffix, or class bonus.

### 11.3 Tie envelope

For each ambiguity stratum, PAVA isotonic calibration maps score difference from
the leader to empirical gold-membership retention. The tie radius is the largest
calibrated difference required to retain at least 99% of all valid alternatives.

At runtime, all productive top-32 candidates inside the calibrated tie radius
are emitted as `Tied`. If the complete calibrated tie set exceeds 32, output is
`ABSTAIN` with `productive_overflow=true`; the protected grounded lane remains
available and no singleton authority is emitted.

### 11.4 Contradiction certificate

A grounded L1.1 Winner may be downgraded only when productive L2 emits:

```text
ContradictionCertificateV1
  grounded candidate identity
  competing candidate identity or impossible-slot identity
  independent evidence source kind
  calibration stratum ID
  support count
  calibrated margin
  false contradiction count, required zero
  package/delta generation
  provenance range
```

Allowed independent sources are explicit corrected/reverted feedback,
structurally impossible typed agreement, or a separately measured local-context
anti center. Generic uncertainty, another positive slot, low support, a tie, or
L3 abstention cannot produce the certificate.

### 11.5 Final L2 verdict

```text
Winner
  exactly one candidate passes calibrated authority
  and all competing valid candidates are outside its tie envelope
  and cross-lemma ownership is satisfied
  and grounded Winner protection is satisfied

Tied
  two or more candidates remain inside the calibrated tie envelope
  and the complete retained tie set fits its bounded lane

ABSTAIN
  no licensed terminal, contradictory evidence, missing package evidence,
  productive overflow, unsupported irregular form, or no calibrated authority
```

`ABSTAIN` may still carry bounded suggestion candidates. It never means an edit
is authorized.

## 12. Composite Lattice And Authority Transfer

### 12.1 Lane structure

```text
CompositeL2LatticeV1
  grounded_candidates[]
  productive_candidates[0..32]
  surface_groups[]
  original_l1_verdict
  productive_verdict
  contradiction_certificate optional
  overflow and integrity flags
```

The grounded array is copied by identity from L1.1. Productive candidates never
consume its capacity.

Canonical L1.1 emits at most 32 tied candidates, so the composite identity bound
is 64 before display dedup: up to 32 grounded plus 32 productive. The bridge to
L3 MUST accept the lane structure or all 64 identities. Flattening both lanes
back to one top-32 array is forbidden because it can violate I-06.

### 12.2 Surface dedup

When several identities decode to the same normalized surface, the display
layer may show one string. The internal surface group retains:

```text
all lemma IDs
all slot and variant IDs
all exact/generated origins
all scores and evidence references
grounded protection bit
```

### 12.3 Cross-lemma settlement

Within one lemma, productive score may produce Winner/Tied/ABSTAIN. Across
different grounded lemmas, productive morphology alone cannot create a unique
winner. The result remains a composite tie unless canonical L2 local lexical
evidence or L3 provides independently calibrated cross-lemma evidence.

### 12.4 L3 handoff

L3 receives both lanes and all provenance. L3 must not reinterpret a productive
identity as grounded. L3 can resolve the composite tie but cannot remove the
grounded lane before its own readout/proof contract runs.

## 13. Package Format V1

All integers are little-endian. Readers use checked byte decoding; they do not
cast mmap bytes to unchecked Rust structs. Every section offset is 8-byte
aligned and lies inside the file.

### 13.1 Header and directory

`ProductivePackageHeaderV1` is 256 bytes:

```text
offset  bytes  field
0       8      magic = "LAYP2V1\0"
8       2      format_version = 1
10      2      header_bytes = 256
12      4      flags
16      32     L1.1 package SHA-256
48      32     canonical L2 package SHA-256
80      32     training manifest SHA-256
112     32     payload sections SHA-256
144     8      section_directory_offset
152     4      section_count = 23
156     4      byte_order_marker = 0x01020304
160     2      maximum observed scalars
162     2      maximum generated scalars
164     2      maximum program operations
166     2      reserved = 0
168     4      slot count
172     4      paradigm count
176     4      binding count
180     4      program count
184     4      operation count
188     4      trie node count
192     4      trie arc count
196     4      terminal count
200     4      slot phase profile count
204     4      calibration cell count
208     8      split seed
216     4      normalization version
220     4      compiler version
224     8      productive package byte budget
232     4      steady RSS KiB budget
236     4      peak RSS KiB budget
240     4      cold publish budget, microseconds
244     4      hot p99 budget, microseconds
248     4      header CRC32 with this field zeroed
252     4      reserved = 0
```

`SectionDirectoryEntryV1` is 32 bytes:

```text
kind:u16 flags:u16 record_size:u32
offset:u64 bytes:u64 count:u32 crc32:u32
```

Header algorithm flags are `bit0=V39_SPEED_PARITY` and
`bit1=PRODUCTIVE_V1_MODEL`. Exactly one must be set. All other V1 header bits
are reserved zero. Runtime authority is not a mutable package bit; it is owned
by the package-bound promotion manifest in section 14.5.

### 13.2 Fixed record widths

```text
MorphologySlotKeyV1          16 bytes
LemmaParadigmBindingV1       40 bytes
ParadigmCenterV1             48 bytes
ParadigmCompatibilityIndexV1 16 bytes
ParadigmPostingV1             4 bytes
MorphProgramHeaderV1         16 bytes
MorphOpV1                    16 bytes
ProductiveTrieNodeV1         16 bytes
ProductiveTrieArcV1          24 bytes
ProductiveTerminalV1         32 bytes
SlotPhaseProfileV1           44 bytes
PhaseCenterV1                76 bytes
DirectionalResidualV1        24 bytes
ModelCoefficientV1           16 bytes
CalibrationCellV1            32 bytes
ProvenanceRecordV1            32 bytes
DeltaManifestV1              192 bytes
DeltaRecordHeaderV1          32 bytes
EvidencePriorV1              24 bytes
```

Strings and variable evidence lists live in checked pools referenced by
`offset:u32/count:u32` ranges. If any pool or record count exceeds `u32`, package
compilation fails; it does not widen records implicitly.

The fixed record field layouts are:

```text
LemmaParadigmBindingV1, 40 bytes
  lemma_id:u32 paradigm_id:u32 source_form_ref:u32 observed_slot_set_ref:u32
  positive_support:u32 explicit_anti_support:u32 stability:u16 flags:u16
  program_start:u32 program_count:u16 reserved:u16 provenance_ref:u32

ParadigmCenterV1, 48 bytes
  pos_domain:u16 flags:u16 root_node:u32
  transition_start:u32 transition_count:u32
  slot_profile_start:u32 slot_profile_count:u32
  program_start:u32 program_count:u32
  support:u32 stability:u16 calibration_class:u16
  provenance_ref:u32 signature_hash_low:u32

ParadigmCompatibilityIndexV1, 16 bytes
  pos_domain:u16 flags:u16 source_slot_id:u32 posting_start:u32
  posting_count:u32

ParadigmPostingV1, 4 bytes
  paradigm_id:u32

MorphProgramHeaderV1, 16 bytes
  source_slot_id:u32 target_slot_id:u32 op_start:u32 op_count:u16 flags:u16

MorphOpV1, 16 bytes
  opcode:u8 anchor:u8 flags:u16 arg0:i32 arg1:u32 arg2:u32

ProductiveTrieNodeV1, 16 bytes
  arc_start:u32 arc_count:u16 terminal_count:u16 terminal_start:u32 flags:u32

ProductiveTrieArcV1, 24 bytes
  child_node:u32 stable_order:u32
  opcode:u8 anchor:u8 flags:u16 arg0:i32 arg1:u32 arg2:u32

ProductiveTerminalV1, 32 bytes
  program_id:u32 target_slot_id:u32 variant_id:u16 flags:u16
  decoder_ref:u32 evidence_ref:u32 calibration_class:u16 reserved:u16
  provenance_ref:u32 stable_identity_hash:u32

SlotPhaseProfileV1, 44 bytes
  slot_id:u32 feature_schema_id:u32
  positive_start:u32 anti_start:u32 hard_negative_start:u32 ambiguity_start:u32
  positive_count:u16 anti_count:u16 hard_negative_count:u16 ambiguity_count:u16
  calibration_class:u16 flags:u16 support:u32 explicit_anti_support:u32

PhaseCenterV1, 76 bytes
  cells:[i8;60] feature_mask:u32 context_mode_id:u32
  support:u16 mass:u16 polarity:i8 flags:u8 reserved:u16

DirectionalResidualV1, 24 bytes
  source_scene_key:u32 from_slot_id:u32 to_slot_id:u32
  positive_support:u32 explicit_anti_support:u32 flags:u32

ModelCoefficientV1, 16 bytes
  feature_id:u16 flags:u16 coefficient_q16:i32 train_support:u32
  feature_schema_hash_low:u32

CalibrationCellV1, 32 bytes
  stratum_key_id:u32 winner_margin_q16:i32 tie_radius_q16:i32 support:u32
  correct_winner_count:u32 false_winner_count:u32 tied_count:u32 flags:u32

ProvenanceRecordV1, 32 bytes
  source_kind:u16 flags:u16 source_id:u64 event_start:u64 event_count:u32
  source_hash_prefix:u64

DeltaManifestV1, 192 bytes
  base_package_sha256:[u8;32] previous_generation_sha256:[u8;32]
  generation:u64 event_start:u64 event_end:u64 section_count_ref:u64
  coefficient_generation:u64 calibration_generation:u64
  proof_receipt_sha256:[u8;32] requested_scope:u32 flags:u32
  payload_sha256:[u8;32] reserved:[u8;8]

DeltaRecordHeaderV1, 32 bytes
  kind:u16 flags:u16 generation:u64 typed_key_hash:u64
  payload_offset:u32 payload_bytes:u32 crc32:u32

EvidencePriorV1, 24 bytes
  channel_id:u16 flags:u16 positive_prior_twice:u64
  contradiction_prior_twice:u64 reserved:u32
```

Every unused flag and reserved value is zero. Readers reject unknown mandatory
flags and may skip only section kinds explicitly marked optional.

`CalibrationCellV1.winner_margin_q16 = i32::MIN` is the single wire sentinel
for `NO_WINNER_AUTHORITY`. Every other winner margin is nonnegative.
`tie_radius_q16` is always nonnegative. A missing authority threshold therefore
survives package roundtrip as `ABSTAIN`; it is never decoded as a permissive
numeric margin.

V1 reference and opcode semantics are exact:

```text
stable IDs (slot_id, paradigm_id, program_id, calibration_class)
  nonzero identities assigned in canonical serialized order

calibration_class
  one-based row identity in CALIBRATION_CELLS used as the terminal fallback;
  the section count MUST fit u16

stratum_key_id
  nonzero low 32 bits of SHA-256 over the canonical observable calibration
  backoff key; CALIBRATION_CELLS are sorted by this key and compilation rejects
  every collision instead of choosing a salt. Runtime computes the same key for
  each section 11.1 backoff key and binary-searches the section; the terminal
  calibration_class is used only when no more specific measured key exists

*_start fields
  zero-based row offsets into the named fixed-record section

*_count fields
  checked row counts; start + count must remain inside that section

provenance_ref
  zero means absent; otherwise one-based row identity in PROVENANCE

source_form_ref / form_ref
  immutable imported lexical form identity; not a sidecar row offset

decoder_ref / segment_ref
  byte offset of a checked SEGMENT_POOL entry; zero means absent only where
  the owning record explicitly permits absence

evidence_ref / context_mode_id / source_scene_key
  nonzero stable typed keys; they are not unchecked array indexes
```

The two program ranges in `ParadigmCenterV1` both address
`MORPH_PROGRAM_HEADERS`. `transition_start/transition_count` is the contiguous
subset of transferable transition programs that forms the normalized paradigm
signature. `program_start/program_count` is the complete contiguous
paradigm-owned program range and MUST contain the transition range.
`LemmaParadigmBindingV1.program_start/program_count` addresses the same section
but owns only lemma-local programs, including exact allomorph programs. A
program header is owned exactly once by either one paradigm range or one lemma
binding range; center and binding ownership ranges are pairwise disjoint. Every
`MORPH_OPERATIONS` row is owned by exactly one program header, the final
operation is `TERMINATE`, and no earlier operation is `TERMINATE`.

`ProductiveTrieArcV1` stores its emitted action inline; it does not reference or
own a row in `MORPH_OPERATIONS`. This is required because radix splitting and
prefix sharing produce arc actions that are not a one-to-one partition of any
terminal program's contiguous operation range. Arc actions use a separate typed
enum because `COPY_TO_RETAINED_EDGE` also carries the compiled
`retained_end_delta`, which is derived from the complete program and is not a
field of the source `MorphOpV1`. `TERMINATE` is absent from the arc enum and
remains terminal attribution owned by the program header plus terminal row.

`ProductiveTrieArcV1` action opcodes and arguments are:

```text
1 COPY_SOURCE_RANGE
  anchor=START|END arg0=source_delta:i16 arg1=scalar_count:u16 arg2=0
2 COPY_TO_RETAINED_EDGE
  anchor=START|END arg0=source_delta:i16
  arg1=retained_end_delta:i16 encoded in the low 16 bits arg2=0
3 DROP_SOURCE_PREFIX
  anchor=INVALID arg0=0 arg1=scalar_count:u16 arg2=0
4 DROP_SOURCE_SUFFIX
  anchor=INVALID arg0=0 arg1=scalar_count:u16 arg2=0
5 EMIT_SEGMENT
  anchor=INVALID arg0=0 arg1=segment_ref arg2=0
6 REPLACE_SOURCE_START
  anchor=END arg0=end_relative_offset:i16 arg1=delete_count:u16 arg2=0
7 EMIT_EXACT_ALLOMORPH
  anchor=INVALID arg0=0 arg1=form_ref arg2=0
```

All arc flags are zero. Scalar counts are nonzero and at most 65,534 except
`REPLACE_SOURCE_START.delete_count`, which may be zero for an internal
insertion whose following emitted arc is owned by the same program. Signed
fields must be exact sign extensions of their 16-bit wire values. The low-16
encoding of `retained_end_delta` has all upper 16 bits zero and is decoded back
to `i16` before runtime traversal.

`ParadigmCenterV1.slot_profile_start/slot_profile_count` addresses
`SLOT_PHASE_PROFILES`. Every slot phase profile is owned by exactly one
paradigm range. Compatibility posting ranges address `PARADIGM_POSTINGS` and
are likewise complete, non-overlapping ownership ranges.

`SEGMENT_POOL` starts with `magic="SPV1":u32` and `entry_count:u32`. Every
entry is 8-byte aligned and encoded as
`byte_length:u32 scalar_length:u16 flags:u16 utf8:[u8;byte_length]` followed by
zero alignment bytes. Flags are zero in V1. References point to the entry's
`byte_length` field, never into its UTF-8 payload. Entries are sorted by
normalized UTF-8 bytes and deduplicated. Load validates UTF-8, scalar length,
ordering, deduplication, padding, and every reference.

The compiler admits both instruction-owned segments and every maximal emitted
scalar run produced by deterministic radix compaction. Adjacent
`EMIT_SEGMENT` instructions, and an emitted replacement followed immediately
by `EMIT_SEGMENT`, may therefore add their concatenated arc payload to the same
deduplicated pool. The compiler validates this estimate against the actual
compacted trie before package layout. If the actual trie adds a payload, the
pool is rebuilt in canonical byte order and all previously encoded morph-op
references are remapped before section serialization. Runtime never
concatenates or interns new segment records.

`AXIS_DICTIONARIES` starts with `magic="ADV1":u32` and `entry_count:u32` and
uses the same aligned envelope. Entry payloads are typed:

```text
kind=1 AXIS_LABEL
  kind:u8 axis:u8 value:u8 reserved:u8 utf8_label:[u8]

kind=2 OBSERVED_SLOT_SET
  kind:u8 reserved:[u8;3] count:u32 slot_ids:[u32;count]
```

For an `AXIS_LABEL` envelope, `scalar_length` is the Unicode scalar count of
`utf8_label` only, excluding the four typed-prefix bytes. For an
`OBSERVED_SLOT_SET` envelope, `scalar_length` is zero. Directory `count` equals
the pool header `entry_count` for both `ADV1` and `SPV1`.

Axis labels are sorted by `(axis,value,label_bytes)`. Slot sets contain sorted,
deduplicated nonzero slot IDs. `observed_slot_set_ref` points to a kind-2 entry.
Unknown kinds, nonzero reserved bytes, and references to the wrong kind reject
the package.

`MorphOpV1` argument ownership is:

```text
COPY_SOURCE_RANGE
  anchor=START|END flags=0 arg0=start_delta:i16 sign-extended to i32
  arg1=scalar_count:u16 (65,535 retains its section 7.2 meaning) arg2=0
DROP_SOURCE_PREFIX / DROP_SOURCE_SUFFIX
  anchor=INVALID flags=0 arg0=0 arg1=scalar_count:u16 arg2=0
EMIT_SEGMENT
  anchor=INVALID flags=0 arg0=0 arg1=segment_ref arg2=0
REPLACE_SOURCE_RANGE
  anchor=END flags=0 arg0=end_relative_offset:i16 sign-extended to i32
  arg1=delete_count:u16 arg2=segment_ref; either argument may be zero but not
  both. A zero segment_ref emits nothing and performs a pure internal deletion.
EMIT_EXACT_ALLOMORPH
  anchor=INVALID flags=0 arg0=0 arg1=decoder_ref arg2=0
TERMINATE
  anchor=INVALID flags=0 arg0=0 arg1=target_slot_id arg2=variant_id:u16
```

All fixed-record flags are zero in base V1 except
`PhaseCenterV1.flags bit0=SUPPORT_SATURATED` from section 9.5 and
`ProductiveTerminalV1.flags bit0=SURFACE_FROM_TRIE`. Phase polarity is exactly
`1=positive`, `-1=explicit anti`, `-2=hard negative`, and `0=ambiguity`; the
containing section and polarity must agree.

`ProductiveTerminalV1` does not duplicate a generated word in the segment pool.
For a transferable productive program, `SURFACE_FROM_TRIE` MUST be set,
`decoder_ref` MUST be zero, and the normalized output accumulated along the
complete trie path is the terminal surface. Display casing is projected later
from the input-scene policy and is not stored in the terminal. A speed-parity or
lemma-local exact terminal that does not set `SURFACE_FROM_TRIE` MUST provide a
nonzero checked `decoder_ref`; that pool entry is its exact normalized surface.
The two forms are mutually exclusive and fail closed on load. Productive V1
paradigm-owned trie terminals MUST use `SURFACE_FROM_TRIE`.

Trie nodes and arcs form a forest. `ParadigmCenterV1.root_node` is a zero-based
node offset. Every node is reachable from exactly one paradigm root, every
non-root has exactly one incoming arc, roots have none, arcs never cycle, and
node arc/terminal ranges do not overlap another node's owned range. Every trie
arc contains one checked non-`TERMINATE` inline action and its child is inside
`TRIE_NODES`. A terminal row is owned by exactly one node; its
program and target slot exist, its decoder reference is valid, and reserved
fields are zero.

### 13.3 Required sections

```text
axis dictionaries
slot keys
paradigm centers
lemma bindings
paradigm compatibility index and postings
edit program headers and operations
segment pool
productive trie nodes and arcs
terminals
slot phase profiles
positive phase centers
explicit anti phase centers
hard-negative phase centers
ambiguity phase centers
directional residuals
model coefficients
calibration cells
provenance table
delta manifest
evidence priors
```

Section kinds are fixed:

```text
1  AXIS_DICTIONARIES
2  SLOT_KEYS
3  PARADIGM_CENTERS
4  LEMMA_BINDINGS
5  PARADIGM_COMPATIBILITY_INDEX
6  PARADIGM_POSTINGS
7  MORPH_PROGRAM_HEADERS
8  MORPH_OPERATIONS
9  SEGMENT_POOL
10 TRIE_NODES
11 TRIE_ARCS
12 TERMINALS
13 SLOT_PHASE_PROFILES
14 POSITIVE_PHASE_CENTERS
15 ANTI_PHASE_CENTERS
16 HARD_NEGATIVE_PHASE_CENTERS
17 AMBIGUITY_PHASE_CENTERS
18 DIRECTIONAL_RESIDUALS
19 MODEL_COEFFICIENTS
20 CALIBRATION_CELLS
21 PROVENANCE
22 DELTA_MANIFEST
23 EVIDENCE_PRIORS
```

Morph opcodes are fixed:

```text
0 INVALID
1 COPY_SOURCE_RANGE
2 DROP_SOURCE_PREFIX
3 DROP_SOURCE_SUFFIX
4 EMIT_SEGMENT
5 REPLACE_SOURCE_RANGE
6 EMIT_EXACT_ALLOMORPH
7 TERMINATE
```

Anchor values are `0=INVALID`, `1=START`, and `2=END`. Axis dictionary labels
are normalized UTF-8, sorted by bytes, and assigned values from 2 upward; values
0 and 1 retain their fixed meanings from section 4.1.

### 13.4 Load contract

Load performs header, fingerprint, range, count, UTF-8, enum, graph acyclicity,
single-parent trie, terminal attribution, and CRC checks before publishing the
view. Failure leaves the old package active. Large relation arrays remain mmap
views; no process-sized unpacked duplicate is allowed.

The payload SHA-256 is computed over each referenced section byte range in
ascending section-kind order, excluding alignment gaps and the header/directory.
The header CRC is computed with bytes 248..252 zeroed. The external artifact
SHA-256 covers the complete file.

## 14. Incremental Delta Protocol

### 14.1 Delta scope

An append-only delta may add:

- lemma binding support;
- paradigm support;
- exact explicit anti or contradiction observations;
- ambiguity observations;
- directional residual evidence;
- a fully fitted coefficient/calibration generation;
- new exact lemma-local allomorph evidence.

It may not patch L1.1, canonical L2, verifier rules, proof fixtures, or runtime
word branches.

Every delta inherits the base split seed and lemma ownership. Feedback involving
a CALIBRATION or HELDOUT_LEMMA identity is retained in an audit spool but cannot
update train evidence, coefficients, centers, or bindings. A rollover that
changes the split seed is a new base package and requires a new frozen proof,
not an ordinary delta.

### 14.2 Identity and ordering

```text
DeltaManifestV1
  base package SHA-256
  previous generation SHA-256
  generation:u64
  event range
  section counts
  coefficient generation
  calibration generation
  proof receipt SHA-256
  requested authority scope, non-authoritative without PromotionManifestV1
  payload SHA-256
```

Deltas load in a single contiguous generation chain. A gap, duplicate generation
with different bytes, wrong base fingerprint, or missing proof status rejects
the entire new chain and retains the last good generation.

### 14.3 Conflict resolution

Evidence records are immutable and additive by typed key. A correction uses a
signed `SUPERSEDES(record_id)` record; last-write-wins mutation is forbidden.
Duplicate event IDs are idempotent.

Coefficients and calibration are atomic generations. Runtime never combines
coefficients from generation N with calibration from generation M. A new
coefficient generation remains shadow-only until its differential and fixed
proofs pass.

### 14.4 Compaction

Base plus deltas may be compacted offline by replaying typed records in
generation order. Compaction MUST produce semantic hash parity against the
uncompacted chain before atomic installation.

### 14.5 Promotion manifest

Model bytes and runtime authority are separate. `PromotionManifestV1` is an
external 256-byte little-endian record installed atomically by the release gate:

```text
offset  bytes  field
0       8      magic = "LAYP2PR1"
8       2      version = 1
10      2      bytes = 256
12      4      authority scope
16      32     base productive package SHA-256
48      32     complete base-plus-delta semantic chain SHA-256
80      32     installed binary SHA-256
112     32     L1.1 package SHA-256
144     32     canonical L2 package SHA-256
176     32     conjunctive offline receipt-bundle SHA-256
208     32     physical product-matrix receipt SHA-256
240     8      model generation
248     4      flags
252     4      CRC32 with this field zeroed
```

Authority scopes are `0=INVALID`, `1=SHADOW`, `2=SUGGEST_ONLY`, and
`3=APPLY_ALLOWED`. Missing, malformed, mismatched, or stale manifests resolve to
`SHADOW`. `APPLY_ALLOWED` requires non-zero matching offline and physical
receipt hashes and a model generation equal to the loaded delta chain. A package
or delta cannot grant itself authority.

## 15. Runtime Algorithm

### 15.1 Request pseudocode

```text
productive_l2(observed, l1_lattice, local_scene):
  assert package fingerprints match
  grounded_lane = preserve_all_grounded(l1_lattice)

  lemma_frontier = birth_lemmas_from_l1_atoms(
      l1_lattice,
      broad_limit=256,
      atom_relation_limit=196608)
  active_lemmas = exact_stable_top(lemma_frontier, 256)

  productive_heap = ExactTopK(32, terminal_total_order)

  for lemma in deterministic_active_order(active_lemmas):
    bindings = packaged_bindings(lemma)
    if bindings is empty:
      bindings = derive_cold_bindings(lexical_lemma_observation(lemma))
    slot_features = exact_stable_top(
        score_local_slots(bindings, local_scene), 16)

    for binding in deterministic_binding_order(bindings):
      traverse_prefix_trie(binding, slot_features):
        update_character_osa()
        update_keyboard_osa()
        update_atom_phase_state()

        on each valid terminal:
          features = finalize_terminal_features()
          score = fixed_point_dot(coefficients, features)
          productive_heap.insert_complete_candidate(score, attribution)

  productive_lane = decode_only_heap_survivors(productive_heap)
  productive_verdict = calibrated_readout(productive_lane, local_scene)
  certificate = maybe_build_contradiction_certificate(
      l1_lattice.verdict, productive_verdict)

  return CompositeL2LatticeV1(
      grounded_lane,
      productive_lane,
      productive_verdict,
      certificate)
```

No terminal is excluded before exact geometry and score. The top-32 heap changes
storage, not evaluation coverage.

### 15.2 Error path

Package corruption, incompatible fingerprints, graph violation, numeric
overflow, unavailable calibration, or productive overflow produce productive
`ABSTAIN`. They do not remove the grounded lane and do not interrupt keyboard
input.

### 15.3 Runtime experiment modes

Package flags admit exactly two algorithm modes:

```text
V39_SPEED_PARITY
  trie terminals encode exactly the V39 rule/family-lane candidate set
  V39 comparator from section 9.4
  generated-path learned features disabled
  required candidate identity and order parity = 100%

PRODUCTIVE_V1_MODEL
  exact paradigm compatibility and complete logical terminals enabled
  trained score and disjoint calibration enabled
  authority determined by the external package-bound promotion manifest
```

The candidate-set delta between the two modes is reported by first-loss bucket.
Changing modes cannot happen from an environment variable against the same
package bytes; it requires a package flag covered by the payload and artifact
hashes. Without a valid promotion manifest, both modes are shadow-only.

## 16. Concurrency Ownership

One service-level `MorphExecutor` owns all productive work. Nested Rayon pools
and one task per lemma are forbidden.

The executor has a configured worker count recorded in every benchmark receipt.
One request is split into deterministic contiguous ranges of the sorted active
lemma frontier. Shards use request-local arenas and local exact top-32 heaps.
The final merge uses the same total comparator and is independent of completion
order.

Scheduling policy:

```text
one admitted request   -> executor may assign several free workers
several requests       -> round-robin request shards with bounded queue age
worker saturation      -> no hidden inner parallelism
```

The first implementation MAY run one sequential request job if it already meets
all latency gates. Parallel traversal is admitted only through `MorphExecutor`.

## 17. Complexity And Memory

For observed length `m`, active lemma set `A`, and each lemma's emitted scalar
transitions `U_l` after radix-arc expansion, V1 work is:

```text
O(m * sum_(l in A) |U_l|
  + evaluated terminals
  + bounded slot/phase work
  + 32 * log(32))
```

There is no claim that suffix-merged graph edges share DP work. V1 uses only
prefix sharing.

Per-request DP arena is bounded by checked observed length, maximum generated
depth, two OSA lanes, and the executor shard count. The receipt reports arena
bytes, decoded string bytes, cache bytes, steady RSS, and peak RSS separately.

Promotion ceilings on the same proof host and benchmark protocol as V39 are:

```text
productive sidecar bytes       <= 81,688,382 B
steady RSS                     <= 314,888 KiB
peak RSS                       <= 337,016 KiB
cold package publish           <= 1,000 ms
single-request hot p99         <= 5.000 ms
admitted multi-client hot p99  <= 5.000 ms
```

A later lower ceiling requires a measured accepted architecture update. A
higher result is a failed gate, not an `unaccepted regression` judgment call.

## 18. Proof Protocol

### 18.1 Receipt environment

Every performance receipt records:

```text
CPU model, sockets, physical/logical cores
kernel and libc
CPU governor and frequency policy
NUMA topology
process and worker affinity
executor worker count
outer client count
arrival pattern and queue depth
warmup count and measured request count
hot/cold mmap and page-cache state
package and binary SHA-256
compiler flags
steady and peak RSS
major/minor faults
```

`workers=1` means one physical productive executor worker only when the receipt
also proves no other internal traversal worker ran.

The canonical release workloads are:

```text
single request:
  one closed-loop client
  13 damage classes x 100 fixed requests
  two complete warmup rounds excluded

multi-client:
  20 closed-loop clients
  each client runs 13 damage classes x 100 fixed requests
  executor workers = 19
  two complete warmup rounds excluded

latency statistic:
  end-to-end admitted request wall time including queue time
  nearest-rank p50/p95/p99/max globally and per damage class
```

Requests are deterministically permuted by the receipt seed. The single and
multi-client manifests, host fingerprint, affinity, and package hash are fixed
before the run. A different workload is diagnostic and cannot satisfy the
promotion latency gate.

### 18.2 Geometry proof

Compare V1 traversal with the V39 exhaustive scorer over:

- all pairs of alphabet-size-three strings through the admitted exhaustive
  length;
- both character and keyboard lanes;
- all insertion, omission, substitution, and adjacent-transposition positions;
- reused-prefix branches;
- edge deletion, inserted segments, internal alternation, and exact allomorph;
- every real fixed productive candidate set.

Required: candidate identity, OSA rows, terminal geometry, score inputs, and
stable order parity `100%` in speed-only mode.

### 18.3 Generalization cohorts

Report `SEEN_EXACT`, `BANK_UNSEEN`, `SLOT_HELDOUT`, and `LEMMA_HELDOUT`
separately. `BANK_UNSEEN` cannot substitute for either heldout cohort.

For `SLOT_HELDOUT` and `LEMMA_HELDOUT`, each of 13 damage classes has at least
20,000 fixed cases. Cohort manifests and hashes are frozen before model fitting.

### 18.4 Unique and ambiguous denominators

Every damage class is partitioned before scoring:

```text
UNIQUE_RESOLVABLE
  exactly one valid target under the available L2 scene

MULTI_LABEL
  several valid targets/slots under the available L2 scene

UNSUPPORTED
  no safely licensed target
```

Promotion gates:

```text
UNIQUE_RESOLVABLE unique top-1, every class          >95.0%
UNIQUE_RESOLVABLE top-16 retention, every class      >95.0%
UNIQUE_RESOLVABLE readout retention, every class     >95.0%
MULTI_LABEL valid-set retention                       >=99.0%
MULTI_LABEL false singleton                                0
UNSUPPORTED false authority                                0
clean preservation                                   >=99.9%
grounded L1.1 candidate loss                               0
grounded Winner protection violations                      0
```

Aggregate top-1 is diagnostic only.

### 18.5 Authority proof

For every Winner, the receipt includes the winning candidate, strongest
competitor, margin, calibration stratum, support, provenance classes, original
L1.1 verdict, contradiction certificate if present, and verifier result.

False authority and false singleton are conjunctive zero across calibration,
heldout, replay, and physical product matrices.

### 18.6 Ablation

Measure full fixed proofs without each of:

```text
ParadigmCenter compatibility
slot positive evidence
explicit anti evidence
ambiguity evidence
directional residuals
character geometry
keyboard geometry
atom/phase evidence
learned interactions
```

A bank with no independent measured effect is removed before promotion.

### 18.7 Product matrix

Only after offline gates pass:

```text
CLI read-only
daemon shadow
IBus shadow
IME suggestion
autocorrect apply and double-Shift undo preservation
WeChat
Telegram
Chromium
GTK
Qt
Kitty/terminal passthrough
```

No global IBus restart occurs before installed binary/package health checks.

### 18.8 V61 measured packaged shadow proof

The first fixed packaged proof is diagnostic and non-authoritative. Percentages
below use exactly `100` cases per row; aggregate percentages cannot promote a
class that fails individually.

`SEEN_EXACT`:

| Damage class | Top-1 | Top-16 | Readout retained | Empty | p99 us |
|---|---:|---:|---:|---:|---:|
| adjacent transposition | 30% | 62% | 62% | 0 | 3,892 |
| double substitution | 34% | 63% | 63% | 0 | 3,382 |
| extra letter | 35% | 60% | 60% | 0 | 3,693 |
| layout projection | 38% | 65% | 65% | 0 | 5,261 |
| letter substitution | 34% | 60% | 60% | 0 | 3,572 |
| missing letter | 33% | 62% | 62% | 0 | 3,604 |
| non-adjacent transposition | 39% | 70% | 70% | 0 | 5,795 |
| omission + transposition | 33% | 62% | 62% | 0 | 3,090 |
| prefix truncation | 36% | 67% | 67% | 0 | 3,399 |
| punctuation suffix | 40% | 65% | 65% | 0 | 5,331 |
| repeated fragment | 39% | 69% | 69% | 0 | 5,933 |
| sparse multi-omission | 31% | 64% | 64% | 0 | 3,372 |
| suffix truncation | 14% | 64% | 64% | 0 | 4,063 |

`LEMMA_HELDOUT`:

| Damage class | Top-1 | Top-16 | Readout retained | Empty | p99 us |
|---|---:|---:|---:|---:|---:|
| adjacent transposition | 2% | 7% | 7% | 91 | 190,568 |
| double substitution | 2% | 10% | 10% | 89 | 191,611 |
| extra letter | 2% | 6% | 6% | 93 | 115,050 |
| layout projection | 3% | 10% | 10% | 89 | 191,613 |
| letter substitution | 2% | 6% | 6% | 93 | 64,587 |
| missing letter | 1% | 5% | 6% | 92 | 166,449 |
| non-adjacent transposition | 2% | 10% | 10% | 89 | 193,404 |
| omission + transposition | 1% | 10% | 10% | 90 | 125,095 |
| prefix truncation | 0% | 7% | 7% | 93 | 30,648 |
| punctuation suffix | 2% | 6% | 6% | 93 | 98,853 |
| repeated fragment | 1% | 9% | 9% | 91 | 24,141 |
| sparse multi-omission | 0% | 9% | 9% | 90 | 188,101 |
| suffix truncation | 2% | 5% | 5% | 90 | 140,358 |

Measured facts:

- package integrity errors: `0`;
- false singleton: `0`;
- Winner / Tied / ABSTAIN: `0 / 0 / 2,600`;
- package: `36,641,392 B`, mmap-backed, `124 B` constant cache;
- process RSS / peak RSS: `251,352 / 251,352 KiB`;
- runtime authority changed: `false`.

Verdict scope: V61 rejects promotion of the packaged productive readout. It
does not reject the immutable L1.1 or canonical L2 owners and does not measure
their existing product quality. The first observed loss mechanism is candidate
birth/retention: `SEEN_EXACT` always produces a lattice but loses the target in
`467 / 1,300` top-16 readouts; `LEMMA_HELDOUT` produces no lattice in
`1,183 / 1,300` cases. Ranking and calibration cannot recover a target that was
not born. The next implementation experiment MUST therefore repair productive
binding coverage and target retention before fitting new coefficients or
changing authority gates.

Receipt:
`docs/structural_gates/receipts/L2_PRODUCTIVE_V1_FIXED_SHADOW_PROOF_V61_2026-08-11/receipt.json`.

## 19. Implementation Module Map

### 19.1 Existing ownership bridge

The implementation starts from these current owners:

```text
src/nanda_wave/l2_field/mod.rs
  fixed 256 / 256 / 16 / 32 / 196608 limits and public field facade

src/nanda_wave/l2_field/productive.rs
  V39 exhaustive productive generation and score-parity reference

src/nanda_wave/l2_field/runtime.rs
  current per-surface orchestration; V42 chunks at line 518 are rejected

src/nanda_wave/l2_field/productive_format.rs
  current mmap sidecar reader/compiler donor

src/nanda_wave/l2_field/model.rs
  canonical 60-cell L2 phase width and existing typed center ownership

src/nanda_wave/l2_field/context.rs
  current bounded context adapter; literal lexical classes are not inherited

src/nanda_wave/l2_field/bridge.rs
  L1.1 lattice ingress and canonical L2/L3 handoff owner
```

`productive.rs` remains the score/candidate reference until prefix-trie parity
passes. The new runtime replaces its repeated per-surface expansion only after
the speed-only parity gate. `bridge.rs` is modified only after the composite
32-grounded plus 32-productive lane proof passes.

### 19.2 New bounded modules

The paper design maps to bounded modules rather than one enlarged runtime file:

```text
src/nanda_wave/l2_field/productive_v1/
  mod.rs                 public internal facade and constants
  types.rs               typed IDs, slots, candidates, certificates
  scene.rs               L2LocalSceneV1 adapter
  events.rs              typed compiler events and split ownership
  corpus.rs              one-pass raw parser, canonical L2 grounding, source manifest
  imported_identity.rs   zero-based base refs and complete binding-ownership join
  induce.rs              anchor, edit templates, paradigm compatibility
  trie.rs                prefix-trie compiler and checked view
  geometry.rs            shared OSA and AtomAccumulatorV1
  score.rs               feature extraction and fixed-point score
  calibrate.rs           calibration replay and readout tables
  compiler.rs            deterministic section assembly and package publication
  spool_sort.rs           bounded deterministic external sort and full-event deduplication
  reduce.rs              one-lemma bounded train morphology reduce and stable IDs
  evidence_reduce.rs     context/feedback/direct-contradiction count aggregation
  transition_reduce.rs   bounded transition sort/support/join and paradigm signatures
  orchestrator.rs        one-pass compile stages, budgets, telemetry, and final report
  runtime.rs             request orchestration, no format parsing
  packaged_runtime.rs    typed mmap adapter, trained traversal, score, and readout
  format.rs              checked mmap format V1
  format_validation.rs   lazy fixed-record scans and fail-closed graph/reference validation
  delta.rs               immutable overlay chain
  proof.rs               fixed proof harness and receipts
```

Dependency direction is:

```text
types
<- scene/events/induce/trie/geometry/score/calibrate/format/delta
<- runtime
<- bridge to existing canonical L2
<- L3 handoff
```

`runtime.rs` does not parse raw bytes, train coefficients, inspect proof fixture
text, or own service scheduling.

## 20. Delivery Sequence

1. Preserve V42 receipts and restore V39 source behavior only.
2. Introduce typed identities, event spool, and split manifest without runtime
   behavior change.
3. Compile and roundtrip package V1; enforce byte/RSS budgets.
4. Implement prefix-trie traversal in speed-only parity mode.
5. Pass exact geometry and candidate identity parity.
6. Train evidence model and calibration from disjoint spools.
7. Run fixed seen/slot-heldout/lemma-heldout/ambiguity proofs.
8. Add composite lattice bridge with grounded-lane protection in shadow.
9. Run L3 handoff, verifier replay, latency, and physical product matrix.
10. Promote generated authority only if every gate passes; otherwise keep V39
    source/runtime authority and preserve receipts.

No step recrystallizes L1.1 or canonical L2. No failed step is repaired by a
literal fixture condition.

## 21. Resolved Open Decisions

| Original decision | Paper resolution |
|---|---|
| Typed inapplicable axes | 16-byte slot key; `INAPPLICABLE=0`, `UNKNOWN=1`. |
| Edit-program instruction set | Seven bounded scalar instructions; package-derived maximum. |
| Incomplete paradigm compatibility | Exact subset compatibility with conflict rejection; retain all matches. |
| Multiple forms in one slot | Distinct variant identities until display dedup. |
| Trie versus FST | Prefix trie only in V1. |
| OSA row storage | Per-request DFS arena keyed by complete prefix path. |
| Atom accumulator | First/last bounded units plus fixed-point incremental typed atom state. |
| Factor combination | Train-only constrained linear score with explicit pairwise objective. |
| Calibration | Disjoint lemma calibration, PAVA tie envelope, zero-error Winner threshold. |
| Delta conflicts | Immutable additive records, explicit supersedes, atomic coefficient/calibration generation. |
| Record widths | Package V1 fixed widths in section 13. |
| Concurrency owner | One service-level MorphExecutor; no nested Rayon. |
| Imported identity namespace | Canonical L2 lemma/form refs remain zero-based and zero-valid; sidecar-owned IDs remain one-based. |
| Imported lemma ownership | One-pass F spool plus package binding-set equality; lexical order alone is insufficient. |
| `NT` competitor evidence | Typed direct context contradiction, never synthetic feedback or a manual weight. |
| `NH` competitor evidence | Read-only proof identity set; zero contribution to fitting or calibration. |

## 22. Formal Identifiability Gate

This section is a pre-build gate. It separates generation, identifiability, and
authority before another implementation variant is allowed.

Canonical short Russian statement:
`docs/l2-productive-morphology-identifiability-canonical-ru.md`.

Let:

```text
P        finite set of train-learned deterministic partial paradigm transducers
O        target-independent lexical observations {(slot_i, surface_i)}
t        heldout target slot
C(O)     paradigms in P that exactly reproduce every observation in O
G_t(O)   distinct surfaces generated at t by every paradigm in C(O)
```

The target bytes and the target slot label are not members of `O`. A paradigm
is compatible only after its complete edit program has been executed against
every exposed source form. Matching a signature, POS label, suffix, or length
without execution is not compatibility.

### 22.1 Finite-observation non-identifiability theorem

For any finite `O` that excludes `t`, if the hypothesis class contains two
transducers `p` and `q` such that

```text
forall x in O: p(x) = q(x)
p(t) != q(t)
```

then no algorithm using only `O` can soundly select one target surface for both
possible data-generating transducers.

Proof: the algorithm receives identical input `O` in both worlds and therefore
must return the same result. That result disagrees with either `p(t)` or `q(t)`,
or it returns a set/reject result. Consequently a forced singleton requires an
independent restriction of the hypothesis class or independent contextual
evidence. More ranking, a manually changed coefficient, or a larger candidate
frontier cannot remove this ambiguity.

This theorem is the paper reason that morphology-only competition cannot be
forced to satisfy a unique top-1 gate on intrinsically ambiguous cases. Those
cases belong to `MULTI_LABEL`, then to L3, or to `ABSTAIN`.

### 22.2 Complete cold-binding retention theorem

Assume:

1. the true train-learned paradigm `p*` is in `P`;
2. its anchor and every exposed source form are present in `O`;
3. compatibility executes and verifies every exposed form exactly;
4. `C(O)` is enumerated completely, without a top-k cut;
5. target bytes and target-slot annotations never affect `O` or `C(O)`.

Then `p*` is in `C(O)`, and, when `p*(t)` is defined, the true target surface is
in `G_t(O)`.

Proof: by assumptions 1-3, `p*` satisfies the compatibility predicate. By
assumption 4 it cannot be truncated. Deterministic execution at `t` therefore
adds `p*(t)` to `G_t(O)`. Assumption 5 makes the result a heldout inference
rather than annotation leakage.

The only sound morphology-only verdict is therefore:

```text
|G_t(O)| = 0   -> ABSTAIN
|G_t(O)| = 1   -> Winner is permitted, subject to calibration and verifier
|G_t(O)| > 1   -> Tied lattice until independent context resolves it
```

Corollary: complete set-valued readout has zero morphology-induced false
singleton relative to `P`. A false singleton after this point is an ownership,
truncation, calibration, or authority-transfer defect.

### 22.3 End-to-end error budget

For damage class `c`, define sequential events:

```text
L  grounded target lemma survives L1.1/canonical L2
B  true paradigm survives complete cold binding
S  true target slot and surface are generated
R  readout retains or correctly selects the target
```

Without assuming independence:

```text
Pr(correct top-1 | c)
  = Pr(L | c)
  * Pr(B | L,c)
  * Pr(S | L,B,c)
  * Pr(R | L,B,S,c)
```

Therefore four stages that each merely pass `95%` can yield only
`0.95^4 = 81.45%`. To guarantee an end-to-end floor of `95%` by a conservative
union budget, the sum of the four conditional failure budgets must be at most
`5%`. An equal allocation requires each stage to retain at least `98.75%`.
Actual budgets MAY be unequal, but they MUST be stated before fitting and MUST
sum to at most `5%`; a later stage cannot hide an earlier loss.

Every receipt MUST report `L`, `B`, `S`, and `R` counts separately for every
damage class and cohort. The first failed event owns the failure bucket. An
aggregate top-1 number or a final target absence cannot identify that owner.

Define morphology-only identifiability for class `c` as:

```text
q_c = Pr(|G_t(O)| = 1 | target retained, c)
```

A sound morphology-only singleton rate cannot exceed `q_c`. Cases outside that
ceiling require L3 context; increasing morphology authority would increase
false singleton rather than solve the task.

### 22.4 Measurement consequence

The current `100` cases per class are a deterministic architecture diagnostic,
not the promotion denominator. Even under an IID binomial interpretation, the
one-sided 95% Clopper-Pearson lower bounds are approximately `91.08%` for
`96/100`, `95.34%` for `99/100`, and `97.05%` for `100/100`. The normative
`20,000` fixed cases per class remain required for promotion.

Before any post-V63 implementation, a pre-build note MUST state:

1. the theorem assumption or measured stage invariant being changed;
2. the expected affected `L/B/S/R` bucket and all protected buckets;
3. the fixed denominator, stop condition, CPU/RSS/disk cost, and expected time;
4. the literature-backed mechanism and the project-specific evidence gap;
5. the reason the change cannot be replaced by `Tied/ABSTAIN` or L3 context.

No code, release build, reinduction, or full proof may begin before that note is
reviewed. A failed V63 result is reported before any later experiment is named
or implemented.

### 22.5 V63 measured cold-binding diagnostic

V63 implemented the theorem-side mechanism before changing any coefficient or
authority gate:

```text
LexicalLemmaObservationV1
-> remove target form and target slot from exposed observations
-> enumerate every compatible train-learned paradigm
-> execute complete programs against every exposed form
-> retain every compatible ColdLemmaBindingV1
-> Winner | Tied | ABSTAIN readout
```

The remote reinduction reused the completed raw corpus, morphology sort,
canonical ownership reduce, context replay, and context sort. It did not
rebuild or mutate L1.1 or canonical L2. The resulting package was:

```text
path       /home/e/projects/lay-productive-v1-build-20260811/out/LAY-L2-PRODUCTIVE-PARADIGM-V1-SHADOW-V63.p2m
bytes      17,309,944
sha256     5b80513cb33d3b82b4b9829742ecab6e4fc3248694f215d252901b630b122238
mmap       true
cache      124 B
build      38:31.45 elapsed
peak RSS   611,204 KiB
```

The deterministic architecture diagnostic used `100` cases for each of 13
damage classes in both `SEEN_EXACT` and `LEMMA_HELDOUT`, for `2,600` cases on
19 requested workers. Columns below are percentages except p99:

`SEEN_EXACT`:

| Damage class | L | S exact | Top-1 | Top-16 | Readout | p99 us |
|---|---:|---:|---:|---:|---:|---:|
| adjacent transposition | 100 | 100 | 31 | 100 | 100 | 7,258 |
| double substitution | 100 | 100 | 40 | 100 | 100 | 10,113 |
| extra letter | 100 | 100 | 39 | 100 | 100 | 7,022 |
| layout projection | 100 | 99 | 38 | 99 | 99 | 10,001 |
| letter substitution | 100 | 100 | 39 | 100 | 100 | 7,062 |
| missing letter | 100 | 100 | 38 | 100 | 100 | 7,627 |
| non-adjacent transposition | 100 | 100 | 36 | 100 | 100 | 8,783 |
| omission + transposition | 100 | 100 | 34 | 100 | 100 | 6,490 |
| prefix truncation | 100 | 100 | 35 | 100 | 100 | 10,311 |
| punctuation suffix | 100 | 100 | 44 | 100 | 100 | 6,172 |
| repeated fragment | 100 | 100 | 37 | 100 | 100 | 9,039 |
| sparse multi-omission | 100 | 99 | 30 | 99 | 99 | 5,888 |
| suffix truncation | 100 | 99 | 14 | 99 | 99 | 7,704 |

`LEMMA_HELDOUT`:

| Damage class | L | S exact | Top-1 | Top-16 | Readout | p99 us |
|---|---:|---:|---:|---:|---:|---:|
| adjacent transposition | 96 | 92 | 3 | 89 | 92 | 138,793 |
| double substitution | 97 | 95 | 3 | 94 | 95 | 138,669 |
| extra letter | 96 | 94 | 2 | 94 | 94 | 144,976 |
| layout projection | 95 | 92 | 4 | 92 | 92 | 124,957 |
| letter substitution | 97 | 94 | 2 | 94 | 94 | 141,603 |
| missing letter | 96 | 87 | 3 | 83 | 87 | 138,771 |
| non-adjacent transposition | 96 | 95 | 1 | 95 | 95 | 88,949 |
| omission + transposition | 96 | 95 | 1 | 95 | 95 | 78,672 |
| prefix truncation | 97 | 94 | 8 | 94 | 94 | 91,360 |
| punctuation suffix | 97 | 95 | 5 | 95 | 95 | 86,412 |
| repeated fragment | 97 | 95 | 5 | 94 | 95 | 131,215 |
| sparse multi-omission | 96 | 93 | 2 | 93 | 93 | 85,758 |
| suffix truncation | 93 | 76 | 1 | 63 | 76 | 84,981 |

Aggregate movement against the V62 diagnostic was:

```text
metric                              V62       V63       delta
LEMMA_HELDOUT L lemma birth        9.00%     96.08%    +87.08 pp
LEMMA_HELDOUT S exact birth        7.77%     92.08%    +84.31 pp
LEMMA_HELDOUT top-16               7.69%     90.38%    +82.69 pp
LEMMA_HELDOUT empty lattice        1,183         51       -1,132
SEEN_EXACT S exact birth          64.08%     99.77%    +35.69 pp
```

Measured safety and runtime facts:

```text
verdict                         FAIL_measured_shadow_gates
Winner / Tied / ABSTAIN         0 / 0 / 2,600
false singleton                0
integrity errors               0
runtime authority changed      false
cold mmap load                 121.505 ms
proof RSS / peak RSS           226,780 / 226,780 KiB
proof elapsed                  59.30 s
```

The first shared measured `LEMMA_HELDOUT` loss is after `L` and before exact
target generation. Aggregate `L` is `96.08%`, but exact `S` is `92.08%`; suffix
truncation exposes the strongest chain at `L=93% -> S=76% -> top-16=63%`.
Ranking cannot recover an ungenerated surface. The receipt does not contain an
independent `B=true paradigm retained` denominator, so it cannot yet separate
compatibility/binding loss from program execution or generation loss. This is
a proof-harness gap and forbids a complete `L/B/S/R` claim.

For `SEEN_EXACT`, target generation and retention are `99.77%`, while aggregate
top-1 is `35.00%`. For `LEMMA_HELDOUT`, all `1,300` verdicts are `ABSTAIN` and
aggregate top-1 is `3.08%`. The current proof event has one labelled target but
does not establish whether other generated surfaces are contextually valid.
Therefore `MULTI_LABEL` versus `UNIQUE_RESOLVABLE` is not measured, and the low
top-1 cannot soundly be repaired by forcing coefficients. It requires the
specified identifiability partition and L3 context.

Not tested by this diagnostic:

```text
SLOT_HELDOUT frozen manifest
independent B retention
MULTI_LABEL valid-set retention
UNSUPPORTED false authority
grounded L1.1 lattice loss and Winner protection
L3/L4/DecisionCore/verifier authority transfer
queue-inclusive one-client and 20-client latency
physical product matrix
```

Verdict: retain the V63 cold-binding and compact-package mechanisms as measured
progress, reject V63 promotion, and stop before naming or implementing a later
version. Exact receipts:

`docs/structural_gates/receipts/L2_PRODUCTIVE_V63_COLD_BINDING_2026-08-11/`.

The post-measurement failure decomposition, source-level crowding proof,
surface-equivalence basin proposal, rejected routes, and no-reinduction proof
order are recorded for review in:

`docs/l2-productive-post-v63-prebuild-review.md`.

## 23. Scientific Basis

The architecture uses standard results but keeps Lay-specific authority and
proof contracts separate from literature claims.

1. Kimmo Koskenniemi. *Two-Level Morphology: A General Computational Model for
   Word-Form Recognition and Production*. University of Helsinki, 1983.
   Basis: finite-state separation of lexical and surface representations.
2. Kenneth R. Beesley and Lauri Karttunen. *Finite State Morphology*. CSLI
   Publications, 2003. Basis: lexical transducers, morphology composition, and
   explicit analysis/generation identities.
3. Mehryar Mohri. "Finite-State Transducers in Language and Speech
   Processing." *Computational Linguistics* 23(2), 1997, pp. 269-311. Basis:
   deterministic transducer algorithms and composition discipline.
4. Kemal Oflazer. "Error-tolerant finite-state recognition with applications to
   morphological analysis and spelling correction." *Computational
   Linguistics* 22(1), 1996, pp. 73-89. Basis: edit-distance traversal over a
   morphological finite-state search space.
5. Klaus U. Schulz and Stoyan Mihov. "Fast String Correction with Levenshtein
   Automata." *International Journal on Document Analysis and Recognition* 5,
   2002, pp. 67-85. DOI: `10.1007/s10032-002-0082-8`. Basis: sharing lexical
   traversal rather than rescoring every complete string independently.
6. Fred J. Damerau. "A Technique for Computer Detection and Correction of
   Spelling Errors." *Communications of the ACM* 7(3), 1964, pp. 171-176. DOI:
   `10.1145/363958.363994`. Basis: adjacent transposition as a first-class typo.
7. Roy Lowrance and Robert A. Wagner. "An Extension of the String-to-String
   Correction Problem." *Journal of the ACM* 22(2), 1975, pp. 177-183. Basis:
   distinction between unrestricted Damerau distance and restricted OSA
   semantics.
8. Eric Brill and Robert C. Moore. "An Improved Error Model for Noisy Channel
   Spelling Correction." ACL 2000. Basis: learned edit evidence rather than
   uniform manually assigned operations.
9. Dilek Hakkani-Tur, Kemal Oflazer, and Gokhan Tur. "Statistical
   Morphological Disambiguation for Agglutinative Languages." *Computers and
   the Humanities* 36, 2002, pp. 381-410. Basis: context-conditioned selection
   among morphological analyses.
10. Bianca Zadrozny and Charles Elkan. "Transforming Classifier Scores into
    Accurate Multiclass Probability Estimates." KDD 2002. Basis: calibration
    must be fitted separately from ranking.
11. C. K. Chow. "On Optimum Recognition Error and Reject Tradeoff." *IEEE
    Transactions on Information Theory* 16(1), 1970, pp. 41-46. Basis: explicit
    reject/abstain behavior instead of forced classification.
12. Farrell Ackerman, James P. Blevins, and Robert Malouf. "Parts and Wholes:
    Implicative Patterns in Inflectional Paradigms." In *Analogy in Grammar*,
    2009. DOI: `10.1093/acprof:oso/9780199547548.003.0003`. Basis: paradigm-cell
    predictability and the information carried by principal parts.
13. Greg Durrett and John DeNero. "Supervised Learning of Complete
    Morphological Paradigms." NAACL-HLT 2013, pp. 1185-1195.
    `https://aclanthology.org/N13-1138/`. Basis: learning complete paradigms from
    partial inflection tables instead of storing only observed target forms.
14. Mans Hulden, Markus Forsberg, and Malin Ahlberg. "Semi-supervised Learning
    of Morphological Paradigms and Lexicons." EACL 2014, pp. 569-578. DOI:
    `10.3115/v1/E14-1060`. Basis: joint paradigm and lexicon induction under
    incomplete observations.
15. Ryan Cotterell et al. "The SIGMORPHON 2016 Shared Task--Morphological
    Reinflection." SIGMORPHON 2016, pp. 10-22. DOI:
    `10.18653/v1/W16-2002`. Basis: explicit source-form, target-feature, and
    target-form reinflection evaluation across languages.
16. Manaal Faruqui et al. "Morphological Inflection Generation Using Character
    Sequence to Sequence Learning." NAACL-HLT 2016, pp. 634-643. DOI:
    `10.18653/v1/N16-1077`. Basis: character-level generation as a learned
    transduction problem; it does not remove ambiguity or authority gates.
17. Roee Aharoni and Yoav Goldberg. "Morphological Inflection Generation with
    Hard Monotonic Attention." ACL 2017, pp. 2004-2015. DOI:
    `10.18653/v1/P17-1183`. Basis: monotonic edit structure for inflectional
    transduction.
18. Katharina Kann, Arya D. McCarthy, Garrett Nicolai, and Mans Hulden. "The
    SIGMORPHON 2020 Shared Task on Unsupervised Morphological Paradigm
    Completion." SIGMORPHON 2020, pp. 51-62. DOI:
    `10.18653/v1/2020.sigmorphon-1.3`. Basis: evaluation of paradigm completion
    when complete lemma tables are not available during training.

These publications support algorithm families. They do not prove Lay quality,
latency, zero false authority, package size, or runtime ownership. Those claims
remain owned only by the fixed receipts in section 18.

## 24. Paper Completion Verdict

At paper level the implementation route is closed:

```text
typed source events
-> one raw pass and deterministic spools
-> verified zero-based canonical L2 identity join
-> learned exact paradigm signatures
-> prefix-only productive trie
-> path-correct shared OSA + atom traversal
-> constrained learned score
-> disjoint calibration
-> protected grounded lane + productive top-32 lane
-> Winner | Tied | ABSTAIN
-> L3
-> verifier
```

What remains is implementation and measurement, not an unspecified architecture
decision. Any implementation question not answerable from this document is a
paper-spec defect and MUST be resolved here before runtime code guesses it.

The V55 audit specifically closes the prior ambiguity around canonical L2 ID
zero and explicit `NT/NH` competitor ownership. Therefore the full-corpus driver
MUST implement the typed route above; it may not fall back to newly assigned
lemma ordinals, ungrounded neighbor hashes, old sidecar compilation, or
hand-authored evidence constants.
