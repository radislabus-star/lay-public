# IME Canonical Target Authority Paper

Date: 2026-08-17
Revision: 8, after durability-latency and quarantine-state review
Status: selected paper architecture; production implementation blocked
Scope: live IME target evidence and authority above L1.1/Productive V90
Runtime authority changed: false

## 1. Decision And Claim Boundary

Lay must replace field-wide and producer-wide authority with target-specific
evidence, a separate cohort verdict, and an event-specific authorization path.

The target correction route is:

```text
exact context-neutral material identity
-> one bounded prepared lexical/geometry field
-> exact frame binding and projection
-> candidate validity per exact framed target
-> Winner | Tied | ABSTAIN cohort verdict
-> L2Certified or ContextCertified automatic certificate
-> one DecisionCore
-> compound fail-closed edit authorization
-> one event mutator
```

This route does not mean that display, Tab, Space, stale publication and double
Shift are one physical call chain. They share immutable target material and
identity contracts, but each event has a separate owner and side-effect
contract. Section 9 defines those routes.

The automatic authority rule is:

```text
exact target identity
+ at least one independently grounded relation witness
+ exact replay geometry
+ complete conflict-cohort enumeration
+ exactly one compatible grounded target
+ no original-preservation veto
+ no target rejection, unresolved settlement conflict, absolute authority blocker
  or evidence overflow
-> L2Certified

several compatible grounded targets
and complete cohort enumeration
-> Tied(Complete)
-> bounded context selection
-> ContextCertified or ABSTAIN

incomplete enumeration at any layer
-> Tied(Incomplete) only when at least two targets are known, otherwise ABSTAIN
-> never automatic authority

no compatible grounded target
-> ABSTAIN
```

`MeasuredMargin` is not admitted in this revision. Until a separate fixed
heldout calibration proves a threshold, two compatible targets always remain
`Tied` and are never converted into automatic authority by a score gap.

`ContextCertified` is part of the type contract but is not currently eligible
for live emission. A context selector must first freeze an independent heldout
denominator, prove its exact decision rule and pass the context-authority slice
in Section 13. Until then context may rank display candidates and produce shadow
settlement only; every automatic lexical tie remains `ABSTAIN`.

The paper selects the architecture but does not authorize source edits. The
previous broad implementation preflight is superseded because it covered
several behavior-changing slices at once and omitted event routes, durable
baseline evidence and the new evidence memory contract. Every implementation
slice requires its own preflight against the source bytes produced by the
preceding slice.

## 2. Adversarial Review Resolution Ledger

The first revision was internally coherent only for the narrow autocorrection
chain. It did not justify its broader claims. The following findings are now
part of the normative design, not informal review notes.

| ID | Defect found by paper/source review | Normative resolution |
| --- | --- | --- |
| R01 | Header said implementation was blocked while the verdict said `READY_TO_IMPLEMENT`. | Source implementation is blocked; old preflight is superseded. |
| R02 | Slice 0 required durable evidence, but its logs existed only in remote `/tmp`. | Raw logs are durable and hash-pinned as historical diagnostics; authority requires a new run from a pre-captured immutable source archive. |
| R03 | One preflight authorized all migration slices even though Slice 1 changes later baseline SHA values. | One preflight per slice; no preflight survives a source-mutating slice. |
| R04 | One `TargetEvidence` value destroyed several independent witnesses for the same target. | A bounded `TargetEvidenceSetV1` retains up to four deterministic independent witnesses. |
| R05 | Candidate validity, cohort resolution and final authority were one state machine. | They are three distinct types and transitions in Section 7. |
| R06 | `NeedsContext` was incorrectly treated as a candidate state. | Ambiguity belongs to `CohortVerdict::Tied`, not `CandidateState`. |
| R07 | A context-selected result was still called `L2Certified`. | Context selection produces `ContextCertified`; `L2Certified` is lexical-only. |
| R08 | An uncalibrated score margin could bypass ambiguity. | Margin authority is disabled and requires a separate future proof. |
| R09 | `ProtectedClean` was modeled as target grounding. | Original preservation is a separate veto/default contract, never lexical grounding for a replacement. |
| R10 | The original input was implicitly another replacement candidate. | Original state is not a candidate and cannot be pruned by candidate top-k. |
| R11 | Target identity, span indexing, case and punctuation projection were undefined. | Section 4 defines exact identities and replay boundaries. |
| R12 | Context evidence could be cached inside immutable lexical material. | Cacheable `PreparedTargetMaterialV1` and per-frame `FrameTargetEvidenceV1` are separate. |
| R13 | The paper said L3 is used only for ties although current code may compute L3 while evaluating candidates. | Computation/observation is distinguished from authority use; only a `Tied` cohort may consume context as automatic authority. |
| R14 | Display, Tab, Space, stale result and rollback were collapsed into one route. | Section 9 defines separate event routes and owners. |
| R15 | Transition verification and backend authorization were presented as one unchecked step. | The compound boundary has explicit ordered subchecks and no fallback. |
| R16 | Versioning happened after physical validation. | The release candidate is versioned and hashed before software and physical validation. |
| R17 | `36/36` was described too broadly. | It is one small IME regression denominator, separate from L1, Productive, Boundary and event proofs. |
| R18 | No ablation or shortcut baseline was required. | Section 12 requires five fixed ablations. |
| R19 | Regex vetoes were treated as hardcode/route absence proof. | Regex checks are tripwires only; AST/source parity, runtime cardinality and fixed proof remain required. |
| R20 | One observed-route marker checked only `use crate::`. | Revised observed-source contract must bind the exact call at `candidate_sources.rs:33`. |
| R21 | Ignored suggestions could become negative target learning. | Non-acceptance is censored; only explicit later events provide new evidence. |
| R22 | Worker/cache/L3/verifier failures were incompletely specified. | Section 10 has a complete per-event fail-closed matrix. |
| R23 | `live_scene()` was described as sentence context. | Current capability is exactly two normalized left tokens; no right or full-sentence claim is allowed. |
| R24 | A bounded witness set had no overflow semantics. | Witness overflow blocks automatic authority and remains display-only; two or more known targets are incomplete `Tied`, otherwise the cohort abstains. |
| R25 | Physical testing could accidentally validate bytes different from the promoted binary. | Build SHA, installed SHA and tested SHA must be byte-identical before promotion. |
| R26 | Cacheable targets contained frame-bound replacement spans and source hashes. | Material target identity and framed target identity are separate; only the former is cacheable. |
| R27 | Productive V90 currently consumes left context while constructing the cached lattice, so context can hide an alternative before tie settlement. | Prepared enumeration must be context-neutral; frame context may only settle an already complete grounded cohort. |
| R28 | "Context uniquely selects" had no calibrated decision rule. | `ContextCertified` live emission is disabled until a separate frozen context-authority proof passes. |
| R29 | Final `36/36` was required although the first-loss map contains absent targets. | Target birth/retention has its own slice and denominator; authority migration cannot claim a birth repair. |
| R30 | Witness identity included provenance even though two producer lanes from one evidence root were supposed to deduplicate. | Semantic evidence-root identity determines independence; provenance is a merged non-authoritative annotation. |
| R31 | The 24-byte witness ceiling named full hashes without specifying how they fit. | The compact witness stores bounded typed references into immutable prepared-field tables; full dereferenced bytes remain equality authority and enter the retained-byte budget. |
| R32 | The 74-target bound did not prove enumeration completeness. | Every producer and relation carries `Complete` or `Overflow`; any hidden conflict alternative blocks singleton authority. |
| R33 | "Compatible cohort" did not define which overlapping or differently sized edits compete. | One conflict cohort is formed from exact edit footprints; independent components cannot be silently composed into one automatic edit. |
| R34 | Deferred double-Shift rollback had no expiry or retry bound. | Pending rollback is frame-bound, finite, never auto-retried, and one later explicit gesture may retry only while the exact receipt remains valid. |
| R35 | The original lost dirty-source bytes made Slice 0 permanently unreproducible. | Old logs remain historical diagnostics; the authoritative baseline must be rerun from a new immutable source archive and exact manifest. |
| R36 | The Slice 1 touched-file inventory omitted current evidence and authority owners. | The preflight must pin `live.rs`, `bridge.rs`, `ime_readout.rs`, candidate adapters, frame types and every new module boundary. |
| R37 | Normalization and keyboard mapping were implicit. | A versioned normalization/layout profile and exact canonical bytes participate in material, target and geometry identity. |
| R38 | Online context/usage generations were not bound to frame evidence. | Every context/overlay generation is frame-bound; an overlay may not change lexical birth or grounding without becoming a material package generation. |
| R39 | Active-composition correction library cases were conflated with the physical active-composition Space event. | API correction and physical event denominators are separate; the current physical Space event commits raw composition exactly once. |
| R40 | Backend output failure semantics assumed atomicity without proving it. | Promotion requires an atomic backend primitive or a proved compensation state machine; partial output can never be described as a successful raw-Space fallback. |
| R41 | `CONTEXT_NEGATIVE`, overflow, preservation and target-local invalidity were all called hard contradictions, allowing context or incompleteness to erase a grounded target before cohort settlement. | Section 5.8 defines five disjoint classes: target rejection, cohort settlement reason, absolute authority blocker, preservation veto and context observation. Only target rejection changes `CandidateState`. |
| R42 | An incomplete cohort could return `Tied` even when only one target identity was retained. | `Tied` requires at least two known grounded targets. Zero or one retained target with incomplete enumeration is `ABSTAIN(IncompleteEnumeration)`; context cannot certify an incomplete tie. |
| R43 | A token-only material key and single-center grounding could not represent Boundary merge or prove both sides of Boundary split. | `ObservedContourV1` distinguishes one token from an exact boundary window, and `CompositeBoundaryGroundingV1` binds every part, segmentation and separator policy. |
| R44 | Rollback failure unconditionally restored a pending receipt even after the backend might have emitted partial output. | A rollback output transaction restores `Pending` only after proved zero output; partial output enters a bounded `RecoveryRequired` state and is never blindly retried. |
| R45 | The paper listed a 128-member L1.1 lattice and a 74-target field without naming the intervening 32-member authority readout. | Section 10.1 distinguishes the 128 seed-service lattice from the 32 retained restoration readout plus explicit `TiedOverflow`; the 74 envelope begins only after that typed boundary. |
| R46 | The revision-3 material/frame route receipt placed context evidence before conflict-cohort construction, contradicting the prose contract. | The revision-4 route forms a complete lexical cohort first; context receives only a complete `Tied` cohort and cannot feed target validity or membership. |
| R47 | Binding an online-overlay generation to a certificate did not prove that the generation was eligible to change live authority. | Unpromoted overlay generations are display/shadow-only. `ContextCertified` may consume only an explicitly proof-pinned authority-eligible generation. |
| R48 | Frozen case names did not require a one-to-one immutable execution receipt, so a run could silently filter or duplicate cases. | Slice 0 results bind every case ID to exact binary, command, selector, environment, exit status and assertion receipt; missing, duplicate or filtered cases fail the denominator. |
| R49 | A compatibility adapter could round-trip a legacy singleton while silently projecting a new multi-witness or overflow value back to one legacy witness. | Legacy round-trip applies only to the legacy domain. Reverse projection of multi-witness, composite or incomplete evidence is explicitly unsupported and fail-closed. |
| R50 | The Slice 1 failure cleanup said to revert a diff in an already dirty worktree. | Implementation uses a dedicated worktree and touched-file manifest; failure retains the isolated diff and receipt and never runs a broad revert over user changes. |
| R51 | Morphology and operator conflicts were grouped with overflow as absolute blockers, which would make future context settlement impossible. | Complete lexical conflicts are `CohortSettlementReasonV1`; only explicitly calibrated reasons may reach context. Overflow, incompleteness and multiple edit components remain absolute blockers. |
| R52 | The first output-transaction graph covered committed Space and rollback but omitted explicit acceptance and active-composition Space from the common transaction protocol. | Revision 5 added all four mutating families; Revision 6 supersedes its outcome model with pre-mutation refusal, full effect vectors and crash durability. |
| R53 | Lexical live promotion required final `36/36` before the context slice even if a frozen case genuinely required context to resolve a complete tie. | Slice 0 freezes disjoint birth/retention, lexical-authority and context-settlement subsets; final `36/36` waits for every nonempty applicable subset. |
| R54 | Slice 1 pinned many source files but did not state which files were actually allowed to change. | A twelve-path maximum source allowlist and byte-identical complement are now part of the preflight; expansion requires a new revision before editing. |
| R55 | Missing immutable manifests had one-byte placeholder expectations, so creating `{` could eventually satisfy a syntactic baseline without real proof. | Blocked V4 introduced an impossible SHA sentinel; blocked V5/V6 preserve it and are never promoted in place. After Slice 0, V7 pins the measured complete manifests by exact SHA, size, mode and semantic validator receipt. |
| R56 | Several paragraphs said all events shared one backend mutator, contradicting the source topology and the event route. | All events share one authorization/transaction protocol, but committed-tail correction/raw Space/rollback use `committed_tail_mutator` while explicit acceptance/active-composition Space use `active_composition_mutator`; every attempted output has exactly one. |
| R57 | Slice 0 required execution receipts for all case definitions although future-contract tests intentionally did not exist, creating a preflight/code deadlock. | V6 froze 105 definitions with future cases explicitly deferred to their owning slice. Active V7 preserves the same 49 baseline cases and freezes 59 future cases; final full promotion, not Slice 1 entry, requires all 108 executed receipts. |
| R58 | A conflicting `Born` target was excluded from the grounded cohort and could therefore leave another target looking like a complete singleton. | A `Born` target caused by incomplete/failed lookup is an unresolved alternative: it is not a grounded tie member, but its conflicting footprint makes the cohort incomplete and forces `ABSTAIN`. |
| R59 | `ordered cohort hash` had no canonical order, so producer arrival, score or provenance could change context identity. | `CanonicalCohortOrderV1` sorts exact footprint and target bytes plus semantic evidence/completeness identity; producer order, score and provenance are forbidden inputs. |
| R60 | The 74-target storage bound did not bound context-neutral enumeration work. | Every producer records deterministic scan/replay/grounding work counters; exhaustion of a preregistered work budget becomes explicit `Overflow(BudgetExceeded)` and cannot issue authority. |
| R61 | Raw Space partial output and non-rollback zero-output outcomes were absent, and prose allowed ambiguous post-refusal fallback. | `CommittedSpacePlanV1` is selected once before authorization; a started/refused transaction never re-enters plan selection. Revision 7 applies pre-mutation refusal, attempted no-effect and partial outcomes to raw/corrected Space, explicit acceptance, active Space, rollback and manual layout. |
| R62 | `RecoveryRequired` assumed the exact visible post-failure state was already known. | Recovery begins with `ObservationPending | ExactAfter | ObservationUnavailable`; no compensation runs until exact visible bytes are observed, and unavailable observation expires fail-closed. |
| R63 | A lexical-only release was allowed while the global gate still unconditionally required `36/36`. | `LexicalOwnerRelease` and `FullTargetAuthorityRelease` now have separate claim boundaries; only the latter may claim final `36/36`, and every release receipt must name its profile. |
| R64 | The planned new `target_evidence.rs` path had no measured pre-edit existence state. | Slice 0's source manifest records an existence bit for every allowlisted path and must prove this path absent before Slice 1; a pre-existing path is a blocker requiring a new plan. |
| R65 | Monotonic rollback expiry could be misread after process or boot restart. | Every finite receipt binds a `MonotonicEpochIdentityV1`; restart/clock-domain change invalidates it without mutation or manual-toggle fallthrough. |
| R66 | `ComposedSurface` named only a derivation hash despite the rule that hashes never define equality. | It carries a typed immutable `derivation_ref` plus an accelerator hash; exact dereferenced derivation bytes remain equality authority. |
| R67 | Context promotion named a confidence bound but no numeric risk policy. | Slice 9 code is blocked until a separate preregistered receipt freezes confidence level, maximum aggregate/per-family upper bounds, minimum sample counts and lineage-separated partitions before calibration data is read. |
| R68 | Revision 5 called authorization refusal a zero-output result while its route forced every such result through a mutator. | `RefusedBeforeMutation` invokes zero mutators. `AttemptedNoEffect` is a distinct post-mutator result and requires a complete effect-vector equality proof. Plan selection is never re-entered in either case. |
| R69 | "Zero output" counted inserted/deleted text but could ignore cleared preedit, moved caret, changed layout, changed engine tail, armed learning or a synthetic key left down. | `OutputEffectSnapshotV1` covers every externally visible and authority-relevant effect. Any changed or unknown dimension is `RecoveryRequired`, never `NoEffectProved`. |
| R70 | The bounded compensation state machine was in-memory, so a process crash after the first non-atomic effect could erase the only recovery receipt. | Promotion requires one crash-atomic backend operation or a durable write-ahead transaction identity committed before the first effect. Startup reconciliation and kill-point fault injection are mandatory; in-memory recovery alone is rejected. |
| R71 | `InputFrameIdentityV1` identified text state but not one physical key event, allowing duplicate delivery or repeat to mint another plan. | `InputEventIdentityV1` identifies each backend event; `InputCommandIdentityV1` binds either one press, one exactly paired release-triggered command or one exact gesture. One command mints at most one plan, and every member event belongs to at most one command. |
| R72 | An overflow carried retained targets but no authority scope, so hidden targets could not be assigned safely to a conflict cohort. | `CompletenessScopeV1` is part of every lane result. Unscoped overflow blocks the whole prepared field; a narrower footprint/relation scope is accepted only after an exhaustive pre-truncation partition proof. |
| R73 | The event atlas covered Tab but current source also accepts an unmodified Alt/ISO-level release, and neither route explicitly bound the appended separator to the selected target. | Tab and unmodified Alt are separate adapters into one `ExplicitAcceptPlanV1`, which binds the visible receipt, exact target bytes, exact separator policy and event identity. Modifier use can never become acceptance. |
| R74 | The material/frame graph still ended at a generic `event_mutator` after the output graph had split state-specific mutators. | Material/frame Revision 6 ends at a typed certificate or `ABSTAIN`; event-plan and output mutation ownership exist only in the event/output graphs. The duplicate generic mutator is removed from the design. |
| R75 | Lexical live-owner promotion preceded implementation and fault proof of the transaction protocol, leaving a window where new authority could use old unsafe output semantics. | A dedicated event-identity/output-transaction slice now precedes every live-owner flip. It is shadow/fault-only until refusal, crash, recovery and exactly-once gates pass. |
| R76 | Only Productive latency had a numeric gate; printable, Space, explicit acceptance, rollback, failure tails and steady RSS could regress without blocking release. | `EventRuntimeBudgetV1` fixes route-specific p99/max, no-synchronous-field-work and RSS ceilings before behavior code. Failure and recovery-entry paths are measured separately; remote compute latency never substitutes for local desktop latency. |
| R77 | `engine path` in a frame hash distinguished IBus and daemon evidence but did not prove that only one runtime owned mutation for the focused input. | `RuntimeInputOwnerLeaseV1` grants one focus lineage exactly one mutating runtime. Handoff revokes unstarted frames/plans/leases; a started or unresolved transaction transfers to one durable recovery owner. A new owner may mutate only after an exact terminal state; quarantine remains native-only. Observer routes remain non-mutating. |
| R78 | The `lost/duplicate Space = 0` rule was scoped to successful or compensated events, so a proved pre-mutation refusal could still swallow the consumed user Space and pass the written gate. | Every physical Space has one terminal input disposition. Before any effect, a proved eligible `NativePassthrough` may return the original event unconsumed; otherwise success or recovery must account for it. A consumed Space with no emitted/passthrough separator is a release failure. |
| R79 | The material V5 graph allowed a lexical certificate without traversing original-preservation evidence. | Material V6 evaluates exact preservation before every cohort readout; both lexical and future context certificates are unreachable when the preservation verdict abstains. |
| R80 | Alt acceptance was release-triggered while the exactly-once rule said releases could never mint a plan. | A release-triggered command consumes one exact unmodified press-arm and matching release; the press itself mints no plan. `OnPress` and `OnMatchingRelease` are mutually exclusive command triggers. |
| R81 | Double Shift was called one `InputEventIdentityV1` although it is an ordered multi-event gesture. | `InputGestureIdentityV1` binds the exact member event IDs, order, owner lease, modifier constraints and monotonic epoch, and enters one `InputCommandIdentityV1`. |
| R82 | Rollback mentioned a later manual layout toggle, but the event/output graphs did not contain that competing route. | `DoubleShiftPlanV1` selects exactly one of rollback, defer, reject-without-fallthrough or manual layout. Manual layout uses the shared durable transaction and one dedicated layout mutator. |
| R83 | Output V6 named a durable intent but did not route it through the journal before mutation. | Output V7 persists and synchronizes the exact intent before the first effect; journal prepare failure is `RefusedBeforeMutation` with zero mutators. |
| R84 | V6 transferred mutation authority only for success events even though no-effect and partial outcomes also invoked a mutator. | Every attempted outcome now has the same explicit plan-to-authorizer-to-state-mutator authority route; refusal and passthrough have none. |
| R85 | Success/no-effect could be observed before a durable terminal outcome existed. | Exact terminal state is persisted before success publication, learning or any later same-lineage state change. A terminal-persistence failure remains recovery-owned by the unfinished durable intent. |
| R86 | Startup recovery and emergency key cleanup invoked mutation owners without an authorization owner. | V7 adds state-specific recovery and key-cleanup authorization routes bound to exact journal/key-state evidence. |
| R87 | Journal corruption, incompatible schema and storage failure had no operational state that kept the keyboard usable. | Unknown durable state enters nonterminal `RecoveryQuarantined`: Lay mutation is disabled, evidence is preserved, independently proved synthetic keys are released, and only proved native passthrough remains available. |
| R88 | Handoff could revoke an active transaction and orphan partial output. | Prepared/applying/recovery transactions are never revoked as ordinary capabilities; one recovery lease follows them across handoff and blocks new Lay mutation until an exact terminal state. Quarantine does not reopen Lay authority. |
| R89 | Package generation participated in witness independence and could count the same derivation twice after reload. | Semantic independence excludes package aliases: generation binds validity/cache staleness, while canonical grounding, operator and derivation lineage define one evidence root. |
| R90 | Frame identity omitted caret, selection, exact source window and preedit cursor state. | These values now participate in exact frame equality; any change invalidates framed targets, receipts and event plans. |
| R91 | A malformed or geometry-failing witness could reject an entire target even when another independent witness was valid. | Witness validity is local. Complete target-level absence may reject a target; malformed/incomplete witness integrity blocks authority but cannot erase an independent valid witness. |
| R92 | Certificates used hashes/field-local references without defining cache-eviction lifetime. | Certificates use typed exact references under a bounded `PreparedMaterialLeaseV1`; hashes remain accelerators, and lease loss/saturation fails closed without authority. |
| R93 | The rollback route could emit negative learning after a failed gesture rather than an exact restore. | Negative correction feedback is emitted only after `AppliedExact` restores the recorded original; refusal, no-effect, partial output, expiry and quarantine teach nothing. |
| R94 | The proof section froze context risk policy before Slice 8 although context authority belongs to Slice 9. | Lexical Slice 8 is independent of context calibration; the numeric context policy is a Slice 9 precondition only. |
| R95 | Release steps could build from a dirty tree and later imply that a receipt-only commit was the binary source. | The versioned release-source commit is frozen before the one final build. Post-physical evidence may use a distinct receipt-only commit, which never changes or relabels the tested binary bytes. |
| R96 | The crash window around returning a handled/unhandled backend event could still lose or duplicate Space. | Event disposition is an effect dimension with a backend-specific atomicity/idempotency proof; otherwise the durable transaction owns the ambiguous event and promotion remains blocked. |
| R97 | Revision 7 required a synchronous durable prepare and terminal state for every event but postponed latency proof until Slice 11, so Slice 7 could implement an architecture that was structurally safe and operationally unusable. | `TransactionDurabilityStrategyV1` is selected by an isolated local microproof before Slice 7 behavior code. The journal-required default is one ordered group-commit owner; two independent foreground durability waits per event are rejected. |
| R98 | `RecoveryQuarantined` appeared beside terminal transaction outcomes even though corruption or native post-crash divergence leaves the original transaction unresolved. | `OutputTransactionStateV1` distinguishes terminal `AppliedExact | AttemptedNoEffect` from nonterminal `RecoveryRequired`; `RuntimeRecoveryModeV1::RecoveryQuarantined` is a separate operational state and never proves success, no-effect or compensation. |
| R99 | Native typing could change a focus lineage after an effect but before its terminal state became durable, making recovery unable to distinguish user input from a partial Lay effect. | `SameLineageStateBarrierV1` blocks both Lay mutation and native state-changing dispatch until the prior terminal state is durable. The next prepare may co-commit the prior terminal state in one ordered barrier. |
| R100 | Quarantine kept native typing available but had no bounded scope or evidence-preserving exit, risking either a dead keyboard or permanent unexplained Lay disablement. | `QuarantineScopeV1` is exact-lineage when integrity permits and runtime-global otherwise. Explicit `QuarantineResetPlanV1` preserves the incident, rotates journal generation, captures a new baseline and emits no success or learning; storage failure remains native-only. |
| R101 | The V6 case manifest did not contain direct contracts for durability selection, the same-lineage terminal barrier or native-divergence reset. | V7 freezes three additional future cases. The denominator is now 108 definitions: 49 baseline and 59 future. |
| R102 | Startup reconciliation forced every exact observation through a recovery mutator and then reached an observer without a new durable terminal state. | Startup recovery now splits `ObserveExact` from `CompensateExact`. Observation invokes zero recovery mutators; compensation has one authorized mutator; both persist exact terminal settlement before opening the lineage barrier. |
| R103 | Quarantine reset and emergency key cleanup reached observers without independent effect evidence. | Reset persists `QuarantineResetReceiptV1` before re-enabling Lay. Key cleanup publishes only through an exact idempotent `KeyCleanupEffectProofV1`; neither route can imply text success or learning. |

An issue is considered resolved on paper only when the replacement contract is
explicit. It is not considered implemented or proven until the corresponding
slice and fixed proof pass.

## 3. Problem Boundary And Current Defect

This paper changes authority transfer after candidate material exists and
defines the prerequisites that make that transfer truthful. It does not retrain
L1.1, rewrite Productive V90 package bytes, relax the verifier, insert a
word/phrase exception, or authorize deployment. Missing-target birth is a
separate source slice; it is included in the migration order only because the
final IME denominator cannot pass without it.

The current field defect has three connected parts:

1. `PreparedCanonicalTokenField::from_lattice()` sets
   `common_l3_required = lattice_surface_count(&lattice) > 1`.
2. `materialize_live_candidates()` consequently demotes otherwise grounded
   lexical/geometric targets to `SuggestOnly` by a field-wide decision.
3. `boundary_text_candidates()` runs as a parallel historical producer and may
   enter the merged lattice as `Eligible` before competing with the whole-token
   V90 target under one evidence contract.

Current source shape:

```text
CorrectionRequest
-> candidate_sources.rs:33 canonical_text_readout_observed()
-> bridge.rs:37 rayon::join
   |-> canonical_owned_text_candidates_observed()
   `-> boundary_text_candidates()
       `-> ime_l2_boundary_candidates()
           `-> boundary_split_candidates()
-> merge by replacement text
-> L2CandidateLattice
-> TransitionDecisionCore
-> transition decision
-> backend authorization
-> committed-tail mutation
```

Source evidence:

- `src/correction_core/candidate_sources.rs:31-36` is the exact canonical
  correction callsite;
- `src/nanda_wave/l2_field/bridge.rs:37-54` executes and merges the two producer
  branches;
- `src/nanda_wave/l2_field/productive_v1/live.rs:91-116` stores field-wide
  `common_l3_required` and authority;
- `src/nanda_wave/l2_field/productive_v1/live.rs:255-410` projects that field
  decision onto individual targets;
- `src/nanda_wave/l2_field/bridge.rs:177-215` constructs the parallel Boundary
  candidates;
- `src/nanda_wave/l2.rs:152-213` calls the historical Boundary scan;
- `src/correction_core.rs:471-523` collects candidates and performs one common
  decision;
- `src/bin/lay_ibus_engine/state.rs:425-497` performs transition and backend
  authorization checks before mutation.

L3 cannot manufacture missing lexical proof. It may choose among compatible,
independently grounded targets. It may not turn a source score, lane, lemma
basin or ungrounded surface into authority.

### 3.1 First-loss map

| Failure family | Birth | Retention | Evidence | Readout | First loss |
| --- | --- | --- | --- | --- | --- |
| ordinary one-edit repairs represented by `прохоил`, `видешь`, `врмея`, `читайл` | present | retained | target grounding exists | no apply | field-wide L3 requirement |
| duplicate layout prefix plus lexical word | absent | absent | none | none | contour birth |
| missing first layout letter | absent | absent | none | none | contour birth |
| valid whole-token repair versus plausible split, represented by `перхвачу` | whole target present | retained but demoted | Boundary separately eligible | false split may win | competing Boundary authority |

The surface strings above are diagnostic representatives only. They must never
appear in a runtime condition, weight table or authority branch.

Existing diagnostic results were:

```text
exact-layout admission                         5 / 5 PASS
fixed small canonical correction gate         18 / 36 PASS
deep representative diagnostics                2 / 12 PASS
```

These results locate mechanisms. They are not promotion proof.

Consequently, candidate-specific authority can repair only rows whose target is
already born and retained. The two contour-birth rows must not be counted as
authority failures, and an authority slice must not claim to fix them. The
final 36-case gate is conjunctive over both independent routes.

### 3.2 Architecture alternatives and consequence analysis

The decision is not based on the representative failures alone. Four routes
were compared across the full authority and lifecycle surface:

| Route | Retention and authority | Hot path and memory | Cache/reload and learning | Failure and maintenance | Verdict |
| --- | --- | --- | --- | --- | --- |
| Current dual route: field-wide Productive authority plus parallel Boundary | Grounded lexical targets are demoted by unrelated field cardinality; Boundary can gain authority before whole-token competition | Existing bounded lanes, but duplicate producer work and route-specific latency remain | Context-shaped cache membership; source-specific feedback semantics | Two owners and ambiguous rollback provenance | `VETO`; baseline only |
| Alternative A: widen top-k/reserves and keep field-wide readout | Improves some target retention but still confuses candidate count with ambiguity; hidden alternatives can still create false singleton | Higher CPU/RSS and tail latency scale with the widened frontier | Existing context-shaped key and package invalidation defects remain | Cheapest patch, but preserves every ownership defect | Rejected |
| Alternative B: make L3/context the first live selector over producer outputs | Can choose endings when the correct complete tie already exists, but can also hide a target before grounding and cannot prove missing lexical evidence | Adds context work to more frames and makes online-overlay drift authority-bearing | Cache identity must include context/overlay; every online update can change live correction | Calibration, rollback and source attribution become one coupled owner | Rejected as lexical authority; retained only after complete tie |
| Alternative C: separate fast lexical and Boundary mutators with event-specific priority | Low latency for selected classes, but overlapping edits never form one cohort and priority manufactures authority | Fast common case, duplicate computation and unbounded route growth over time | Separate caches and learning receipts diverge | Recreates the current defect with more owners | Rejected |
| Selected: context-neutral material, exact frame binding, complete target cohort, event-specific authorization | Preserves typed targets, distinguishes missing birth from readout, and permits authority only from a complete exact target/cohort | Adds bounded evidence storage and may increase preparation work; hot IBus threads remain lease/commit-only | Material cache is context-neutral; package changes invalidate material, overlay changes invalidate frame settlement only | One target truth, one policy owner, one transaction protocol; zero mutators before refusal and exactly one state-correct mutator per attempted output; compatibility routes have a scheduled removal slice | Selected |

Measured facts are limited to the current lane/cache bounds, historical
diagnostic results and existing source topology. The selected route's quality,
latency, allocation and RSS effects remain hypotheses until their owning
slices run. Predeclared consequences are:

- candidate/lattice retention can improve, but automatic coverage may decrease
  because an incomplete singleton now abstains instead of impersonating a
  winner; both changes are reported separately;
- context-neutral Productive enumeration may cost more CPU than current
  context-pruned enumeration; no live promotion occurs unless remote hot p99,
  allocation, cache occupancy and RSS gates pass without synchronous IBus
  field work;
- Slice 1 may add at most the Section 10.2 memory delta while changing no rank,
  cache schema, authority, display or mutation behavior;
- a package, normalization or producer-configuration generation change
  invalidates prepared material; context/usage/overlay changes invalidate only
  frame settlement and cannot rewrite cached membership;
- newly learned overlay generations may improve display immediately but are
  barred from automatic authority until separately promoted;
- stale jobs, saturation and reloads may reduce suggestion availability but
  cannot create a late edit or second producer;
- partial output changes the event into a recovery transaction and may expose a
  temporary fail-closed state; it must never be hidden by raw Space or a blind
  rollback retry;
- temporary adapters increase code size only through the named migration
  slices and have a mandatory deletion gate in Slice 9. Permanent dual routes
  are not an accepted outcome.

These are expected regressions and costs, not measured improvements. Any
unbounded consequence, unexpected authority increase, cache-identity drift or
new owner returns the work to paper review before another source revision.

## 4. Exact Identity Contract

### 4.1 Input and frame identity

Two identities are deliberately separate:

```text
PreparedMaterialKeyV1
    exact ObservedContourV1 bytes and scalar sequence
    normalization/layout profile identity
    ordered typed-contour identity
    ordered L1.1 lattice identity and completeness state
    L1.1/canonical/Productive package generations
    lexical producer configuration identity
    evidence schema version

InputFrameIdentityV1
    runtime input-owner lease identity
    engine path
    focus lineage and backend focus serial
    tail epoch
    monotonic epoch identity
    exact committed tail
    context prefix
    exact observed token
    exact source-window bytes and scalar coordinate domain
    caret scalar position and selection range
    active-composition bytes, cursor and visibility state
    active layout
    config identity
    context/usage/online-overlay generations

InputEventIdentityV1
    runtime input-owner lease identity
    exact InputFrameIdentityV1
    backend event sequence or owner-assigned monotonic sequence
    keyval, keycode and modifier state
    Press | Release | Repeat classification
    monotonic epoch identity

InputGestureIdentityV1
    runtime input-owner lease identity
    exact ordered member InputEventIdentityV1 values
    gesture schema and modifier constraints
    monotonic epoch identity and bounded inter-event timing

InputCommandIdentityV1
    OnPress(InputEventIdentityV1)
    | OnMatchingRelease { exact_press_arm, exact_release_event }
    | Gesture(InputGestureIdentityV1)
```

`ObservedContourV1` is one of:

```text
TokenContour {
    exact_token_bytes,
    exact_token_scalars,
}

BoundaryWindowContour {
    exact_left_token_bytes,
    exact_separator_bytes,
    exact_right_token_bytes,
    exact_window_scalars,
    separator_profile_id,
}
```

The boundary window is bounded by the same typed operator and memory contracts
as a token contour. It is not arbitrary sentence context. It exists so a merge
can delete an exact separator and a split can compete against the exact whole
token under one material identity.

The first key permits reuse of immutable material. The second grants one GUI
frame the right to consume that material. Equality of material keys never
substitutes for equality of frame identities.

`RuntimeInputOwnerLeaseV1` binds one focus lineage and generation to exactly one
mutating runtime path. IBus, daemon and any compatibility adapter may observe the
same physical activity, but only the lease owner may mint input identities,
event plans or output transactions. Handoff revokes the old owner's frame
material, prepared leases, visible receipts, press arms, pending event plans and
not-yet-started transactions. A transaction already in `Prepared`, `Applying`
or recovery state is not revocable metadata: it transfers to exactly one durable
recovery lease. The new owner cannot mutate that focus lineage until the
obligation has an exact durable terminal state. If reconciliation instead
enters quarantine, Lay mutation remains disabled at the proved quarantine scope;
handoff never treats quarantine as a terminal capability release. A stale owner
is display/telemetry-only.

`InputEventIdentityV1` identifies one backend key event, not a whole gesture.
`InputCommandIdentityV1` is the exactly-once plan key. A command is triggered at
one declared phase only: `OnPress`, `OnMatchingRelease`, or after completion of
one exact bounded gesture. For release-triggered Alt acceptance, the press stores
an unconsumed arm but mints no plan; only its exact unmodified matching release
may consume that arm and mint one command. Double Shift uses a gesture identity
whose member events cannot be reused by another command. An auto-repeat is
rejected by the frozen per-key policy or receives a distinct event and command
identity. Frame equality without command equality never authorizes replay.

No focus receipt, tail epoch, replacement span, surrounding context, context
score, online usage generation or selected winner is allowed in cacheable
material. The current `CanonicalTokenKey::scene_bytes` and the current
context-shaped Productive evaluation are therefore migration inputs, not the
selected final contract. Before candidate-specific authority can go live, two
different left contexts over the same lexical input must produce exactly the
same prepared target identities, grounding witnesses and completeness state.
Only frame settlement may differ. If Productive V90 cannot enumerate that
context-neutral set within the declared bounds, the migration remains blocked;
context-pruned enumeration is not an accepted approximation.

`NormalizationLayoutProfileIdV1` binds the exact normalization algorithm,
Unicode assumptions, case policy, script classifier, RU/EN keyboard map and
their schema versions. "Lowercase" or "normalized" without this identity is
not sufficient cache or proof evidence.

`MonotonicEpochIdentityV1` binds the runtime boot/process generation and clock
domain used for finite deadlines. Monotonic timestamps are comparable only
inside the same epoch. A restart, persisted receipt from another epoch or clock
domain mismatch invalidates the finite capability; it never resets or extends
the deadline.

### 4.2 Replacement span

`ReplacementSpanV1` is expressed in Unicode scalar offsets over the exact
source snapshot, never UTF-8 byte offsets:

```text
ReplacementSpanV1 {
    scalar_start,
    scalar_len,
    source_scalar_len,
    exact_source_slice_hash,
}
```

Before authorization, scalar offsets are replayed against exact source bytes.
Invalid UTF-8, an out-of-range span, a source-hash mismatch, a combining-mark
boundary not supported by the verifier, or changed surrounding text rejects
the edit. Byte offsets may be derived only after this validation.

### 4.3 Material and framed target identity

Cacheable lexical material and a concrete GUI edit deliberately use different
identities:

```text
MaterialTargetIdentityV1 {
    exact_normalized_target_scalars,
    canonical_lexical_or_boundary_bytes,
    normalization_layout_profile_id,
    separator_profile_id_or_none,
}

FrameTargetIdentityV1 {
    material_target_id,
    replacement_span,
    exact_projected_target_bytes,
    case_projection_id,
    punctuation_projection_id,
    frame_identity_hash,
}
```

Hashes accelerate lookup but never define equality alone. A hash match must be
confirmed against exact scalar/byte content. Case and punctuation projections
are explicit versioned operations, not implicit string transforms. A cached
material target is rebound and replayed against the current source snapshot to
create a framed target; a cached replacement span is forbidden.

The same target may have several operation witnesses. Those witnesses are
merged under one target identity; they do not create duplicate candidates.
For a boundary target, exact separators are part of target equality rather
than display punctuation projected later.

### 4.4 Witness identity and independence

`EvidenceRootIdentityV1` determines whether evidence is independent:

```text
(relation schema, canonical operator program, grounding namespace and identity,
 derivation-lineage identity)
```

Two lane names or source IDs derived from the same upstream terminal/form and
operator replay are one witness, not independent confirmation. Producer/source
provenance is stored as a bounded merged annotation and is never part of
independence or authority. Exact duplicate semantic roots merge deterministically
even when their provenance or package generation differs. Package generation
still participates in material validity, cache identity and stale-reference
checks; it cannot turn the same canonical derivation lineage into a second vote.

The compact `TargetWitnessV1` stores typed indices into canonical immutable
tables owned by the same prepared field. It does not store truncated hashes as
equality authority. Dereferenced relation, operator, grounding, generation and
derivation bytes define equality; their canonical hash is only an accelerator.
Cross-field or cross-generation witness handles are invalid.

## 5. Prepared And Frame Evidence

### 5.1 Cacheable prepared material

```text
PreparedTargetMaterialV1 {
    key: PreparedMaterialKeyV1,
    original: PreparedOriginalMaterialV1,
    targets: BoundedTargetSetV1,
    completeness: EnumerationCompletenessV1,
    evidence_tables: PreparedEvidenceTablesV1,
    integrity: PreparedIntegrityV1,
}

PreparedTargetV1 {
    identity: MaterialTargetIdentityV1,
    witnesses: TargetEvidenceSetV1,
}
```

The exact Rust owner selected for Slice 1 is
`src/typing_transition/target_evidence.rs`. This module owns identity, witness,
candidate/cohort and certificate vocabulary because display and correction are
consumers of one typing-transition contract. L2, correction and IBus modules
may adapt to it but must not define competing evidence enums.

Prepared material may contain context-neutral retrieval, grounding and
token-relative geometry. It must not contain a replacement span, frame
rejection, selected winner, context verdict, correction lease, `AuthorizedEdit`,
feedback update or mutation permission.

Field-local target and witness references are usable only under:

```text
PreparedMaterialLeaseV1 {
    exact prepared material key and integrity digest,
    immutable field generation and allocation identity,
    runtime-owner lease identity,
    monotonic epoch and finite expiry,
    one bounded consumer state: Display | FrameSettlement | EventPlan,
}
```

The lease pins the exact immutable evidence tables until its one consumer is
finished or expired. Cache eviction may remove lookup ownership but cannot
reuse or free pinned table identity. Lease saturation, eviction before pinning,
generation reuse or missing exact bytes produces display drop/`ABSTAIN`; it can
never be repaired by hash equality or a stale field-local index. Exact pin
counts and retained bytes are part of the cache and RSS budgets.

### 5.2 Per-frame evidence

```text
SettledFrameEvidenceV1 {
    frame_identity: InputFrameIdentityV1,
    prepared_material_key,
    framed_targets: BoundedFrameTargetAssessmentsV1,
    original_preservation: FrameOriginalPreservationV1,
    conflict_cohorts: BoundedConflictCohortsV1,
    context_settlement: ContextSettlementV1,
    authority_certificate: Option<AuthorityCertificate>,
    expiry_and_generation,
}

FrameTargetAssessmentV1 {
    identity: FrameTargetIdentityV1,
    witness_assessments: BoundedWitnessAssessmentsV1,
    candidate_state: CandidateState,
    target_rejection_or_none,
}

FrameSettlementValidityV1 =
    Valid
    | Invalid(FrameInvalidationReasonV1)
```

This record is never reusable across focus, tail epoch, surrounding text,
layout, config or package generation changes.

Candidate validity is derived here after material integrity, exact frame
binding and witness-local geometry replay. Stale input/package/profile state
invalidates the whole frame settlement; it is not copied into every target as
a reusable rejection and can never be cached inside prepared material.

### 5.3 Bounded multi-witness set

Every target retains at most four witnesses:

```text
MAX_TARGET_WITNESSES = 4
```

Merge order is deterministic after exact semantic-root deduplication:

1. stable relation schema ID;
2. grounding namespace and identity;
3. canonical operator program bytes;
4. derivation-lineage identity.

Source score and source priority do not determine authority. Support values may
rank two witnesses with the same typed identity, but may not collapse distinct
relations or groundings.

If more than four materially distinct valid witnesses exist, the target is
marked `witness_overflow`. Truncation may retain diagnostic witnesses, but the
target cannot receive automatic singleton authority from the incomplete set.
It remains displayable. A cohort with at least two known grounded targets is
`Tied(..., Incomplete, ...)`; a cohort with zero or one known target is
`ABSTAIN(IncompleteEnumeration)`. Neither is context-certifiable.

Every witness is assessed before the display sample is truncated. A rejected
witness cannot erase another independent valid witness and cannot count as
positive support. Overflow stores a saturating total, an order-independent
digest of all semantic witness identities and an explicit
`IncompleteForAuthority` state. A malformed/incomplete root creates an
integrity blocker; it is never silently dropped so a retained positive sample
can impersonate complete proof. No score, generation alias or producer order
chooses which evidence is considered for integrity.

### 5.4 Relation evidence

Supported relation families are typed operator schemas, not authority labels:

```text
ExactLayout
MissingLetter
ExtraLetter
Substitution
AdjacentTransposition
NonAdjacentTransposition
SparseOmission
RepeatedFragment
MixedLayout
LayoutThenTypo
MorphologySlot
BoundarySplit
BoundaryMerge
Unsupported
```

Identity/preservation is not a replacement relation. Every relation records
the affected scalar positions, source and target lengths, layouts, operation
count and schema version.

### 5.5 Grounding evidence

```text
GroundingRefV1 =
    None
    | L11Terminal { terminal_id, package_generation, score, verdict_membership }
    | CanonicalForm { form_ref, lemma_id, slot_id, support, package_generation }
    | ProductiveSurface { lemma_id, slot_id, support, package_generation }
    | ComposedSurface {
          lemma_id,
          slot_id,
          derivation_ref,
          derivation_hash,
          support,
          schema,
      }

CompositeBoundaryGroundingV1 {
    ordered_part_groundings: BoundedNonEmpty<GroundingRefV1>,
    exact_segmentation_scalars,
    separator_profile_id,
    merged_target_grounding_or_none,
}
```

Grounding names the exact target. A lemma basin does not ground every surface
inside the lemma. `ProtectedClean` is intentionally absent.

A split is grounded only when every emitted lexical part has an exact grounding
and the segmentation matches its operator replay. A merge additionally binds
the exact merged lexical target while clean two-token preservation remains a
separate original-state contract. One strong side never grounds the other.

Every grounding also carries an `EvidenceRootIdentityV1`. Two package adapters
that expose the same terminal/form derivation remain one root. Distinct source
labels alone never increase independent support.

`derivation_ref` indexes immutable canonical derivation bytes in the prepared
field. `derivation_hash` accelerates lookup only; a hash match without exact
dereferenced byte equality is not grounding.

### 5.6 Geometry witness

Prepared geometry proves that one declared token-relative operator program
transforms the exact observed token into the material target. Frame geometry
then replays that program over the exact replacement span and proves that it
changes nothing else. Together they bind:

- exact source and target identities;
- operation sequence and scalar positions;
- layout mapping, where applicable;
- preserved prefix, suffix, separators and punctuation;
- resulting exact target bytes;
- verifier schema version.

A hidden second edit, consumed separator, changed punctuation or replay
mismatch rejects that witness. A stale source slice invalidates the whole
frame. A target becomes `Rejected` only after a complete target-specific
namespace proves that no valid grounding/geometry remains; one failed alias
cannot override an independent valid witness.

### 5.7 Original preservation evidence

```text
PreparedOriginalMaterialV1 {
    exact_material_identity,
    lexical_status,
    clean_or_protected_status,
    script_and_token_status,
    punctuation_status,
}

FrameOriginalPreservationV1 {
    frame_identity,
    prepared_original_material_hash,
    independent_preservation_evidence,
    verdict: Preserve | ReplacePermitted,
}
```

The original input is the default observable state, not a replacement
candidate. It is never subject to candidate top-k. A protected clean original
causes `ABSTAIN(PreserveOriginal)` unless an explicitly admitted, independently
measured contradiction contract says otherwise. Generic uncertainty, context
preference or source score cannot override preservation.

### 5.8 Rejections, blockers and context observations

These states are deliberately disjoint:

```text
WitnessRejectionReasonV1
    GROUNDING_REF_MISMATCH
    GEOMETRY_REPLAY_MISMATCH
    MALFORMED_EVIDENCE_ROOT
    STALE_WITNESS_GENERATION

TargetRejectionReasonV1
    TARGET_ABSENT_FROM_GROUNDING
    COMPLETE_NO_VALID_GEOMETRY
    UNSUPPORTED_TARGET_IDENTITY

FrameInvalidationReasonV1
    STALE_INPUT_IDENTITY
    STALE_PACKAGE_GENERATION
    STALE_NORMALIZATION_OR_LAYOUT_PROFILE
    UNSUPPORTED_SCRIPT_OR_TOKEN
    SOURCE_WINDOW_OR_CARET_MISMATCH

CohortSettlementReasonV1
    INDEPENDENT_L11_WINNER_CONFLICT
    INDEPENDENT_OPERATOR_CONFLICT
    MORPHOLOGY_SLOT_CONFLICT
    BOUNDARY_FALSE_SPLIT_RISK

AbsoluteAuthorityBlockerV1
    WITNESS_OVERFLOW
    TARGET_SET_OVERFLOW
    UPSTREAM_ENUMERATION_INCOMPLETE
    EVIDENCE_INTEGRITY_INCOMPLETE
    MULTIPLE_EDIT_COMPONENTS

OriginalPreservationVetoV1
    PROTECTED_ORIGINAL
    CLEAN_BOUNDARY_WINDOW

ContextObservationV1
    SUPPORTS
    CONTRADICTS
    AMBIGUOUS
    UNAVAILABLE
```

Only a complete target-specific `TargetRejectionReasonV1` may produce
`CandidateState::Rejected`. `WitnessRejectionReasonV1` rejects one witness;
another independent valid witness may still make the target `Grounded`.
Malformed or incompletely accounted witness material additionally raises
`EVIDENCE_INTEGRITY_INCOMPLETE`, so local rejection cannot manufacture a clean
singleton. `FrameInvalidationReasonV1` aborts the entire frame settlement and
issues no certificate. A cohort
settlement reason keeps every otherwise grounded target visible and prevents a
lexical singleton; with at least two known compatible members it yields
`Tied`. Only reasons explicitly marked context-resolvable by the frozen context
contract may later produce `ContextCertified`. An absolute authority blocker
keeps diagnostic/display evidence but forbids both L2 and context automatic
authority. Preservation acts on the settled cohort, not on target grounding.
Context observations are computed only after a complete `Tied` cohort exists;
they may select among its existing members after calibration but can never
change target birth, grounding, `CandidateState` or cohort membership.

All rejection and blocker evidence is accumulated before display truncation.
`TARGET_SET_OVERFLOW`, witness overflow, upstream L1.1 `TiedOverflow`, a
Productive overflow or a relation-reserve overflow makes the affected conflict
cohort incomplete and therefore unable to emit `Winner` or
`ContextCertified`. Context is not allowed to choose among a visible prefix
while hidden alternatives exist.

## 6. Context Capability And Limits

Current `live_scene()` reads at most two normalized left tokens from
`context_prefix` and has no right-context field. Its exact capability is:

```text
LeftLocalContext2
```

It is not full-sentence context. No proof, trace, UI text or architecture claim
may describe it as such. Sentence-level or right-context authority requires an
explicit `InputFrameIdentityV1` extension, privacy and cache rules, a new route
contract and a separate heldout proof.

Context evidence is:

```text
Unavailable
ObservedOnly
Supports { context_relation_id, target_id, alternatives, support }
Contradicts { context_relation_id, target_id, support }
Ambiguous
```

L3/L4 computation may still run for display ranking or telemetry. That fact
does not grant automatic authority. Context may produce `ContextCertified`
only after a valid `CohortVerdict::Tied` and only among the exact grounded
members of that tie. It cannot birth, ground or resurrect a target.

The context selector is itself versioned:

```text
ContextSelectorContractV1 {
    selector_schema_and_model_hash,
    exact context capability,
    ordered cohort hash,
    context/usage/overlay generations,
    calibrated decision rule id,
}
```

A context score maximum is not a certificate. Live `ContextCertified` requires
an independently frozen heldout calibration, zero false authority on its
promotion denominator, complete evaluation of every tied member and an exact
positive/contradictory evidence rule fixed before the run. The current status
of that rule is `DISABLED_PENDING_PROOF`; failure or absence yields `ABSTAIN`.
An online overlay may rank or settle only at this frame stage. If an overlay is
ever allowed to change target birth or lexical grounding, it becomes a
material-generation input and requires a new cache/schema proof.

Generation binding alone does not make an overlay authoritative. Each overlay
generation has an explicit capability:

```text
DisplayOnly | ShadowSettlement | AuthorityEligible(proof_receipt_hash)
```

Only `AuthorityEligible` may participate in `ContextCertified`, and its proof
receipt must name the same selector, context capability, heldout denominator
and overlay bytes. Online learning creates a new unpromoted generation; it may
improve display immediately but cannot silently change automatic correction.

## 7. Three-Level Decision Model

### 7.1 Candidate validity

```text
CandidateState =
    Born
    Grounded
    Rejected(reason)
```

- `Born`: a framed target identity exists but its evidence is incomplete.
- `Grounded`: at least one valid relation + exact target grounding + exact
  geometry witness exists, with no complete target-level rejection.
- `Rejected`: a complete target-specific namespace proves exact target absence,
  no valid geometry, or an unsupported target identity. Witness-local failure
  alone is insufficient.

Candidate state says nothing about whether another valid target exists.

`TARGET_ABSENT_FROM_GROUNDING` is valid only after the target's complete
grounding namespace was searched and returned no exact target. If grounding
lookup overflowed, failed, exhausted its deterministic work budget or was not
run, the target remains `Born` with explicit incompleteness; it may not be
converted to `Rejected` merely to remove an alternative.

`Born` and provisional grounding may be derived from prepared material, but the
final state is always frame-bound because projection and exact replay are frame
facts. Frame staleness invalidates settlement globally rather than manufacturing
per-target rejections. Prepared material never stores a reusable frame verdict.

### 7.2 Cohort verdict

```text
CohortVerdict =
    Winner(target_id)
    Tied(at_least_two_bounded_target_ids, completeness, reason)
    Abstain(reason)
```

An `EditFootprintV1` binds the exact source snapshot, scalar interval, consumed
separator(s) and projected output interval. Two targets belong to one conflict
cohort when their footprints overlap, consume the same user event/boundary, or
cannot both be applied while preserving the exact snapshot. Exact duplicate
outputs merge witnesses before this step.

The cohort's grounded member list contains every `Grounded` target in one
complete conflict component preserved by the typed relation reserves. A
`Rejected` target is excluded. A conflicting `Born` target is retained as an
unresolved alternative: it is not a grounded tie member, but it marks that
component incomplete. One grounded target plus one conflicting `Born` target
therefore yields `ABSTAIN(IncompleteEnumeration)`, never `Winner` or a synthetic
two-member tie. Independent components are not silently combined: this version emits
at most one automatic edit per user event, so more than one nonempty component
forces `ABSTAIN(MultipleEditComponents)` unless a future explicitly typed
compound operator proves one atomic plan. Rules:

1. zero compatible targets -> `ABSTAIN`;
2. original-preservation veto -> `ABSTAIN(PreserveOriginal)`;
3. one compatible target with complete evidence and no overflow -> `Winner`;
4. two or more compatible targets, or an unresolved settlement conflict with
   at least two known members -> `Tied`;
5. incomplete enumeration with two or more known targets ->
   `Tied(..., Incomplete, ...)`, never context authority;
6. incomplete enumeration with zero or one known target ->
   `ABSTAIN(IncompleteEnumeration)`, never a synthetic one-member tie.

Candidate count before grounding is not ambiguity. One surviving ungrounded
surface is not a winner.

A settlement reason with fewer than two explicit grounded alternatives yields
`ABSTAIN(UnresolvedConflict)`, not a synthetic tie. Context consumes only a
complete tie whose reason is explicitly admitted by its frozen selector
contract. `AbsoluteAuthorityBlockerV1` is never context-resolvable.

Every complete cohort has one canonical order before hashing or context use:

```text
CanonicalCohortOrderV1
    exact EditFootprintV1 canonical bytes ASC
    exact projected target scalar/byte sequence ASC
    MaterialTargetIdentityV1 canonical bytes ASC
    semantic evidence-root set digest ASC
    completeness identity ASC
```

Exact duplicate outputs are merged before this order. Arrival order, thread
schedule, source/provenance ID, support score and display rank are forbidden
ordering inputs. The `ordered cohort hash` in the context selector means this
canonical order and no other order.

### 7.3 Automatic authority certificate

```text
AuthorityCertificate =
    L2Certified {
        prepared_material_lease_id,
        exact_frame_identity_ref,
        exact_framed_target_ref,
        exact_cohort_and_preservation_ref,
        frame_identity_hash,
        exact_projected_target_hash,
        evidence_hash,
        cohort_hash,
        completeness_hash,
        material_and_frame_generations,
        schema_versions,
        monotonic_epoch_identity,
        expires_at_monotonic,
    }
    ContextCertified {
        prepared_material_lease_id,
        exact_frame_identity_ref,
        exact_framed_target_ref,
        exact_cohort_and_preservation_ref,
        frame_identity_hash,
        exact_projected_target_hash,
        evidence_hash,
        cohort_hash,
        completeness_hash,
        context_hash,
        selector_and_authority_eligible_overlay_generation,
        material_and_frame_generations,
        schema_versions,
        monotonic_epoch_identity,
        expires_at_monotonic,
    }
```

`L2Certified` is emitted only from an L2 `Winner`. `ContextCertified` is
emitted only when bounded context uniquely selects one member of an exact,
complete `Tied` cohort containing at least two known grounded members.
Unavailable, failed, ambiguous, incomplete or unpromoted-overlay context
returns `ABSTAIN` for automatic authority.

The fields above are normative rather than implied metadata. Typed references
are dereferenced under the exact bounded prepared-material lease and exact bytes
are replayed at consumption; hashes are lookup accelerators and cannot replace
byte equality. Lease loss, epoch mismatch or evicted/reused table identity
invalidates the capability. A certificate is a single-frame capability, not a
cacheable label and not a serializable fallback.
The current implementation plan enables `L2Certified` first; live
`ContextCertified` remains disabled until its separate calibration slice.

Tab acceptance and double Shift rollback do not use these automatic
certificates as user intent. They use exact explicit event receipts described
in Section 9, while retaining the same edit authorization boundary.

### 7.4 Projection and policy

```text
Born                    -> hidden or diagnostic only
Grounded                -> displayable suggestion
Tied                    -> displayable; no automatic edit
L2Certified             -> eligible for DecisionCore automatic policy
ContextCertified        -> eligible for DecisionCore automatic policy
Rejected                -> hidden and non-authoritative
```

`SuggestOnly` becomes a projection of evidence/cohort state, not evidence by
itself. DecisionCore may still reject a certificate on product or safety
policy. A rejection is terminal for that frame.

## 8. Typed Relation Authority Matrix

| Relation | Minimum target grounding | Geometry | Cohort rule | Context role | Automatic result |
| --- | --- | --- | --- | --- | --- |
| Exact layout | exact mapped lexical target | exact key-map replay | one compatible target | none | L2Certified |
| Missing/extra/substitution | exact terminal/form | exact one-edit replay | one target; no margin shortcut | tie selector only | L2Certified or Tied |
| Transposition | exact terminal/form | declared swap replay | one target; no margin shortcut | tie selector only | L2Certified or Tied |
| Sparse/composite omission | exact terminal/form | complete declared program | one target; no hidden edit | tie selector only | L2Certified or Tied |
| Mixed layout | exact target | exact segment mapping | one target | none | L2Certified |
| Layout plus typo | exact target | layout plus declared edit | one target | tie selector only | L2Certified or Tied |
| Morphology slot | exact form/derivation | exact slot transition | all compatible slots retained | shadow selector until calibrated | Tied unless slot unique; context authority disabled initially |
| Boundary split | both exact side targets | separator insertion only | competes with whole token | tie selector only | L2Certified or Tied |
| Boundary merge | exact merged target | separator deletion only | competes with two-token reading | tie selector only | L2Certified or Tied |

Boundary proof requires two lexical centers but cannot dominate an independently
grounded whole-token target. Both hypotheses enter one cohort before any
certificate is issued.

If the Boundary reserve overflows, all produced hypotheses may remain
displayable, but the conflict cohort is incomplete and automatic authority is
forbidden. The current reserve of two is therefore a storage/display bound, not
proof that only two lexical splits exist.

## 9. Event Route Atlas

### 9.1 Printable display refresh

```text
printable event
-> capture one InputEventIdentityV1 under the live runtime-owner lease
-> mint one OnPress InputCommandIdentityV1
-> derive one InputFrameIdentityV1
-> schedule one bounded background prepared-material request
-> validate returned material and frame generation
-> display-only candidate ranking
-> preedit rendering
```

No `AuthorizedEdit` or mutation permission exists on this route. A stale or
failed result is dropped. Current code may observe L3 while ranking; authority
remains display-only.

### 9.2 Explicit candidate acceptance

```text
Tab press or unmodified Alt/ISO-level release
-> exact OnPress or OnMatchingRelease InputCommandIdentityV1
-> live runtime-owner lease
-> exact visible-candidate receipt
-> explicit user selection
-> ExplicitAcceptPlanV1
-> compound edit authorization
-> active-composition commit
-> provisional acceptance learning receipt
```

`ExplicitAcceptPlanV1` binds the exact visible receipt, projected target bytes,
source span, `NoSeparator | ExactlyOneSpace` policy and input-command identity.
The separator is part of the authorized event plan even though it is not part of
the lexical target identity. Tab and Alt are separate event adapters into this
one plan type; they do not define separate selection, authority or mutation
routes. Alt used with Shift or any other command modifier is never acceptance.
Its press may only arm the command; it cannot itself mint a plan, and an
unrelated/repeated/modified release cannot consume the arm.

Explicit acceptance does not reuse automatic correction authority. It consumes
the exact candidate the user saw. A stale selection, changed buffer, changed
frame or failed authorization produces no mutation. Acceptance learning remains
provisional until the next stable word boundary; immediate editing can revoke
it.

`VisibleTargetReceiptV1` binds frame identity, material and framed target IDs,
exact rendered bytes, display slot, ordered visible-list hash, renderer
generation and expiry. The display slot alone is never identity. A stable word
boundary is a successfully committed separator/punctuation event followed by a
new-token frame with the same focus lineage; focus loss, edit, rollback or
backend failure revokes provisional learning instead of confirming it.

### 9.3 Committed-tail Space autocorrection

The event selects exactly one plan before any output capability is minted:

```text
CommittedSpacePlanV1 =
    CorrectAndAppendSpace { verified_correction_plan, exactly_one_separator }
    | RawSpace { exactly_one_separator }
```

```text
Space with empty active composition
-> capture one OnPress InputCommandIdentityV1 under the live runtime-owner lease
-> capture exact current frame
-> take matching prepared correction lease
-> certificate + DecisionCore + correction-plan verification
   -> CorrectAndAppendSpace
   `-> RawSpace when correction is unavailable or rejected before output authorization
-> compound edit authorization
-> one committed-tail mutation including exactly one separator
```

`NotReady`, `Stale`, `ABSTAIN`, policy refusal or correction-plan verification
refusal selects `RawSpace` before the final authorization/transaction. This is
one event-plan decision, not a second producer or a post-output fallback. Once
either plan enters final output authorization, refusal, proved zero output or
partial output is terminal for that attempt and may not re-enter plan selection.

The lease and certificate must name the same frame, target, cohort digest and
generations. In the first live authority slice only `L2Certified` is admitted;
`ContextCertified` remains shadow-only until its separate proof.

### 9.4 Active-composition Space

```text
Space with active composition
-> capture one OnPress InputCommandIdentityV1 under the live runtime-owner lease
-> commit exact user composition plus one Space
```

Current code explicitly does not autocorrect on this route. It must not be
silently merged with committed-tail Space in a proof denominator.

Library functions that calculate a hypothetical active-composition correction
belong to the correction-API denominator, not this physical event denominator.
Passing such a library test does not prove that physical active-composition
Space applies an autocorrection, and this paper does not authorize one.

### 9.5 Stale publication

```text
worker result
-> exact material generation check
-> exact InputFrameIdentityV1 check
-> publish or discard
```

Discard is terminal. A stale result cannot render, authorize, mutate, teach or
invoke a fallback producer.

### 9.6 Immediate double Shift rollback

```text
DoubleShiftPlanV1 =
    Rollback { exact receipt, inverse edit plan }
    | DeferRollback { exact receipt, pending observation }
    | RejectRollbackNoFallback { invalid_or_expired_receipt }
    | ManualLayoutToggle { exact source and target layout }

double Shift
-> capture one InputGestureIdentityV1 under the live runtime-owner lease
-> mint one Gesture InputCommandIdentityV1
-> check whether a pending autocorrection receipt existed at gesture start
   -> valid exact snapshot: Rollback
   -> snapshot not yet available: DeferRollback
   -> pending but invalid/expired: RejectRollbackNoFallback
   `-> no pending receipt: ManualLayoutToggle
-> for a selected mutating plan, use compound authorization and one durable
   output transaction
```

The four variants are mutually exclusive. If the exact snapshot is not yet
available, the engine requests it, retains the pending rollback and defers. It
does not fall through to manual layout toggle while an autocorrection rollback
was present at gesture start. Authorization refusal or a backend failure proved
to have emitted no effect across the complete vector may restore the pending
receipt for one later explicit retry. Once output starts, the receipt moves to `Applying`;
partial output moves to `RecoveryRequired { exact_before,
observation: ObservationPending | ExactAfter | ObservationUnavailable,
known_and_unknown_effects, durable_transaction_identity,
remaining_or_compensating_plan_or_none, monotonic_epoch, expiry }` and can
never be treated as the original pending inverse edit. No compensating plan is
constructed or emitted until an exact current `OutputEffectSnapshotV1` has been
observed.
If observation remains unavailable, recovery expires fail-closed and reports an
unresolved output failure rather than guessing visible bytes.

`ManualLayoutToggle` is not an untracked fallback. It enters the same event-plan
identity, durable intent, effect snapshot and recovery protocol through exactly
one `layout_mutator`. Its output vector includes layout generation, synthetic
key state, preedit, tail and event disposition. A pending/invalid rollback and a
manual layout plan can never be selected for the same gesture.

The pending state is bound to the same focus lineage, tail epoch, exact
postcondition, `MonotonicEpochIdentityV1` and a finite monotonic expiry. It
performs no automatic retry. Process restart, boot change or clock-domain
change invalidates it without mutation.
One later explicit double-Shift gesture may retry a zero-output pending receipt
while it is still valid; focus/cursor/text change or expiry cancels it without
mutation. `RecoveryRequired` is handled only by the bounded output-recovery
state machine and never by gesture replay. A process restart invalidates the
gesture capability but does not erase a durable unfinished output transaction;
startup reconciliation owns that separate obligation. The
gesture that discovered an invalid/expired rollback never falls through to a
manual layout toggle; a subsequent new gesture may use the ordinary manual
route. This gives a finite state machine and preserves user intent.

Negative correction feedback is emitted only after rollback `AppliedExact`
restores the recorded original bytes and all effect dimensions settle. Refusal,
defer, no-effect, partial output, expiry, quarantine and manual layout emit no
negative target label.

The exact monotonic TTL and maximum recovery steps are frozen before the event
proof, not selected after a failure. `max_explicit_rollback_retries` is one.

### 9.7 Compound edit authorization boundary

The logical authorization owner contains ordered fail-closed subchecks:

```text
selected action and exact visible state
-> exact InputCommandIdentityV1 and live RuntimeInputOwnerLeaseV1
-> transition decision and structural edit-plan verification
-> exactly one selected event plan
-> backend authorization producing AuthorizedEdit for that exact plan
-> authorized-plan equality and output-capability recheck
-> select the proof-pinned TransactionDurabilityStrategyV1
-> commit one exact durable transaction intent before the first effect
   -> RefusedBeforeMutation with zero mutators if prepare fails
   `-> one state-correct mutator
      -> exact full effect snapshot
      -> append exact durable outcome state
      -> SameLineageStateBarrierV1
         -> durable terminal: AppliedExact | AttemptedNoEffect
         `-> unfinished: RecoveryRequired
```

Correction ineligibility discovered before event-plan selection may select the
typed raw-Space plan as Section 9.3 defines. After a plan is selected, failure
at any authorization/output subcheck is terminal for that attempt. No producer,
ranker, alternative plan or fallback may run after final refusal or output
start.

The logical authorization and transaction protocol is shared; the physical
mutator is not global. Committed-tail correction, raw Space and rollback use
`committed_tail_mutator`. Explicit acceptance and active-composition Space use
`active_composition_mutator`. Manual layout uses `layout_mutator`.
`RefusedBeforeMutation` has cardinality zero for all three. Every attempted
output has cardinality one for its selected mutator and zero for the other two;
no outcome may invoke more than one.

The attempt disposition and durable transaction state are separate:

```text
OutputAttemptDispositionV1 =
    RefusedBeforeMutation { reason, event_disposition }
    | Transaction(OutputTransactionStateV1)

OutputTransactionStateV1 =
    Prepared { durable_intent, before }
    | Applying { durable_intent, effect_step }
    | EffectObservedPendingTerminal { before, observed_after, event_disposition }
    | AppliedExact { before, after, effect_digest, terminal: true }
    | AttemptedNoEffect { before, after_equal_proof, event_disposition, terminal: true }
    | RecoverySettledExact {
          before,
          final_state: IntendedAfter | OriginalBefore,
          recovery_steps,
          exact_effect_digest,
          terminal: true,
      }
    | RecoveryRequired {
          before,
          observation: ObservationPending | ExactAfter | ObservationUnavailable,
          known_and_unknown_effects,
          durable_transaction_identity,
          bounded_recovery_policy,
          terminal: false,
      }

RuntimeRecoveryModeV1 =
    Normal
    | RecoveryQuarantined {
          durable_transaction_identity_or_corrupt_record_digest,
          unresolved_transaction_state,
          scope: QuarantineScopeV1,
          reason,
          lay_mutation_disabled: true,
          native_input_policy,
      }
```

`AppliedExact` and `AttemptedNoEffect` are publishable only after their exact
terminal journal state is durable. `AttemptedNoEffect` is terminal only when
the complete effect-vector equality proof is durable. If terminal persistence
fails, the original write-ahead intent remains unfinished and recovery-owned;
success, learning and every later same-lineage Lay or native state change are
forbidden until the barrier settles or quarantine takes ownership. Every
compensation action has a durable
`RecoveryStepIdentityV1`, exact precondition/postcondition, maximum attempt
count and write-ahead state, so a second process death cannot apply one step
twice or skip observation.

Startup reconciliation first observes. If the exact current snapshot already
equals the intended `after` state, it persists `AppliedExact`; if it equals the
complete `before` state and event disposition proves no consumed effect, it may
persist `AttemptedNoEffect`. Neither path invokes a recovery mutator. Only an
exact observed compensable state may authorize one bounded recovery mutator,
whose postcondition becomes durable `RecoverySettledExact` before the lineage
barrier opens. Any unknown or externally diverged state enters quarantine.

Here `success publication` means exposing a terminal receipt to learning,
feedback, later Lay authority or a new same-lineage state transition. It does
not pretend that an already attempted backend effect was visually hidden while
its terminal state was pending; that visible effect remains recovery-owned.

`OutputEffectSnapshotV1` is not a text-byte counter. It binds the exact committed
text window, preedit bytes and cursor, caret/selection, active layout and layout
generation, synthetic key-down set, engine tail/epoch, pending rollback and
learning state, and physical-event consumption disposition. `AttemptedNoEffect`
requires equality on every dimension and no unknown observation. Clearing
preedit, moving the caret, changing layout/tail state, arming feedback or leaving
one key down is a partial effect even if inserted/deleted text bytes are zero.

Every physical event has exactly one terminal disposition:

```text
HandledByExactMutation
| NativePassthrough { original_event_unconsumed, capability_proof }
| RecoveryOwnsConsumedEvent
| RefusedUnconsumed
```

`NativePassthrough` is permitted only before any output/internal effect and only
for an event/backend pair with a fixed proof that returning the original event
unhandled produces exactly one native action. It is not a second correction
plan. For Space, a consumed event must end in one exact separator or a recovery
receipt that owns it; a consumed Space with no separator can never be excluded
from the lost-Space denominator as a mere failure.

The callback return that commits `handled` versus `unhandled` is itself part of
the effect protocol. Each backend must prove that this boundary is atomic or
idempotent for the exact event identity. If process death can leave consumption
unknown, the durable transaction owns that ambiguity; neither native replay nor
synthetic re-emission is allowed until exact reconciliation. This kill point is
part of the Space denominator.

Before promotion, the output owner must prove either one atomic backend
operation for the complete effect vector or a crash-durable compensation state
machine whose write-ahead transaction identity is committed before the first
non-atomic effect. In-memory recovery is insufficient. Startup must reconcile
every unfinished durable transaction before that focus lineage can mutate
again. Synthetic key output must be one crash-atomic batch or have an independent
cleanup owner that survives mutator death. If output can fail after a partial
deletion/insertion/preedit/layout/key effect, "raw Space once" is not an adequate
fallback claim; the event is an output failure and must retain an exact recovery
receipt.

Journal open/write/fsync failure before the first effect is a zero-mutator
refusal. Corrupt, truncated, foreign-schema or unreadable journal state at
startup never authorizes compensation. It enters the separate nonterminal
`RecoveryQuarantined` runtime mode, releases any independently provable
synthetic key state, disables Lay mutation at the proved scope and keeps proved
native input available. Quarantine is fail-closed for Lay edits but must not
disable the user's keyboard or imply that the old transaction completed.

### 9.8 Durability strategy, crash scopes and quarantine exit

`TransactionDurabilityStrategyV1` has two admissible forms:

```text
BackendAtomicReceiptV1 {
    complete_effect_vector_atomicity_proof,
    exact event-disposition identity,
    post-crash query or idempotent replay proof,
}

OrderedGroupCommitV1 {
    one dedicated journal owner,
    exact append order and checksum chain,
    durable prepare barrier before effect,
    terminal-state append after exact observation,
    SameLineageStateBarrierV1,
    bounded tail-flush deadline and queue,
}
```

The existing backend has no accepted complete-vector atomic receipt, so the
selected paper default is `OrderedGroupCommitV1`. `BackendAtomicReceiptV1` may
replace it only through a new design receipt and kill-point proof; a claim that
one text API call is "probably atomic" is insufficient.

The ordered journal protocol is:

```text
first transaction N
-> append Prepared(N)
-> one durable barrier
-> apply N
-> observe exact effect
-> append terminal(N), not yet published
-> close SameLineageStateBarrierV1

next Lay transaction N+1
-> append Prepared(N+1)
-> one ordered durable barrier commits terminal(N) + Prepared(N+1)
-> publish N, open N's lineage barrier, then apply N+1

no next Lay transaction
-> bounded journal-owner tail flush commits terminal(N)
-> publish N and open the lineage barrier

next native state-changing event
-> trigger/wait for terminal(N) durability first
-> only then return the exact proved native disposition
```

Thus every journal-required event has at most one foreground durability wait.
The terminal state may still require a second storage sync when no next prepare
arrives before the bounded tail-flush deadline; that sync is background work,
not erased work. If the next event arrives while it is pending, the event waits
behind the same-lineage barrier and that inherited wait counts in its foreground
latency. Co-commit reduces physical sync count only for overlapping events and
is never assumed from human typing cadence. The proof reports actual sync calls,
bytes, queue depth and I/O time per event. Group commit never permits effect-
before-prepare, success-before-terminal or event reordering. A queued next event
is not an authority fallback: it remains unconsumed and cannot mutate the same
lineage until the barrier opens. Barrier timeout, queue saturation or persistence
failure enters refusal before effect or recovery after effect; it never drops,
replays or natively leaks a consumed Space.

Before any Slice 7 behavior code, an isolated local microproof must pin and test
the exact filesystem, mount options, storage device, journal record size,
sync primitive, worker priority, queue bound, cold/warm definition and clock
boundaries. It must include prepare, co-committed terminal-plus-prepare, tail
flush, next-native-event wait, saturation and injected persistence-failure
strata. Two independent synchronous foreground durability waits per event are a
design failure. Passing the isolated barrier microproof does not prove full IME
latency; the integrated route must still pass every Section 10.3 p99/max gate in
Slice 7 and again in Slice 11.

Crash scopes are explicit:

| Kill point | Durable evidence | Permitted result |
| --- | --- | --- |
| before durable prepare | no transaction effect authority | zero-mutator refusal or backend-proved original event disposition |
| after prepare, before first effect | exact intent and before snapshot | observe no effect; never replay blindly |
| during a non-atomic effect | exact intent plus known/unknown effect prefix | `RecoveryRequired`; bounded exact observation before compensation |
| after exact effect, before terminal durability | intent plus pending terminal bytes | same-lineage barrier remains closed; startup reconciles exact state |
| after terminal durability, before callback return | exact terminal state | backend callback atomicity/idempotency proof decides event disposition |
| native typing after process loss, before reconciliation | old intent plus a divergent current state | `RecoveryQuarantined`; no inverse, success or no-effect inference |

`QuarantineScopeV1` is `FocusLineage(exact_identity)` only when journal and
owner integrity prove that boundary. Corrupt chain/schema, unknown ownership or
storage failure is `RuntimeGlobal`. In both scopes native input uses the proved
OS/IBus passthrough route and never a Lay text/layout mutator. Further native
typing is recorded only as external divergence; it cannot settle or relabel the
old transaction.

Quarantine exits only through an explicit `QuarantineResetPlanV1`:

```text
preserve immutable incident bytes and digest
-> prove every synthetic key released
-> obtain exact current visible baseline and runtime owner
-> atomically rotate to a new journal generation
-> persist QuarantineResetReceiptV1
-> re-enable Lay mutation only for the new generation
```

Reset performs no text compensation, emits no target success/no-effect and
teaches nothing. Failure to preserve evidence, observe the baseline, release a
key or persist the new generation leaves native-only quarantine in place. It
never restarts or disables global IBus.

Emergency synthetic-key cleanup is a separate idempotent safety route. It may
emit only key-up operations bound to independently proved key-down state and
publishes through `KeyCleanupEffectProofV1`; it cannot edit text, settle the old
transaction, open a lineage barrier by itself or emit learning.

## 10. Boundedness, Cache And Failure Semantics

### 10.1 Existing source bounds

| Material | Current source bound |
| --- | ---: |
| L1.1 seed-service/main phase lattice | 128 |
| L1.1 restoration readout retained members | 32 plus explicit `TiedOverflow` |
| L1.1 geometry/operator/reconstruction reserves | 32 / 64 / 64 |
| L1.1 reconstruction/geometry scans | 8,192 / 1,024 |
| merged live L1.1 lattice | 128 |
| canonical form groundings | 32 |
| inverse forms per contour | 16 |
| direct/reference surface groundings | 4 / 2 |
| Productive grounded/productive/contour lanes | 32 / 32 / 8 |
| current separate Boundary candidates | 2 |
| ready/in-flight cache entries | 128 / 32 |
| waiters per key | 8 |
| Space lease wait budget | 8 ms |

The 128-member service lattice is not copied wholesale into Productive. The
typed L1.1 restoration contract emits one Winner, at most 32 retained tied
members, or `TiedOverflow` with the logical count. The following Productive
envelope starts only at that readout boundary. Any future route that consumes
more than 32 L1.1 members requires a new measured bound; it may not silently
reinterpret the current 74-target budget.

After Boundary internalization, the paper preserves the existing maximum stored
surface envelope:

```text
32 grounded + 32 productive + 8 contour + 2 boundary = 74 targets
```

Deduplication can reduce this count but may never be used to justify a larger
unmeasured bound. This is a storage bound, not an assertion that the logical
enumeration is always complete. Every lane records one of:

```text
Complete { logical_count, retained_count, set_digest }
Overflow {
    retained_count,
    logical_count_lower_bound,
    all_seen_digest,
    scope: CompletenessScopeV1,
    reason: StorageCapacity | WorkBudgetExceeded | UpstreamIncomplete,
}
Failed { reason }
```

`CompletenessScopeV1` is one of `WholePreparedField`, an exact canonical
`EditFootprintPartition`, or an exact typed `RelationPartition`. A producer may
emit a narrower scope only when it completed an exhaustive, order-independent
partition before storage/work truncation and stores that partition proof in the
prepared evidence tables. If the hidden alternatives cannot be assigned to an
exact proved partition, overflow is `WholePreparedField` and blocks every
automatic cohort in that prepared field. A retained target cannot infer a narrow
scope from its own footprint after overflow has already occurred.

Upstream L1.1 `TiedOverflow`, Productive overflow, contour overflow and Boundary
overflow propagate into `EnumerationCompletenessV1`. A conflict cohort covered
by an `Overflow`/`Failed` scope cannot issue automatic authority; whole-field
scope blocks every cohort. This prevents the first 74 retained surfaces from
impersonating the complete field when hidden target footprints are unknown.

Storage boundedness does not prove computation boundedness. Every producer must
also report deterministic work counters under `EnumerationWorkBudgetV1`, at
minimum posting visits, relation/operator replays, grounding lookups, generated
logical targets and operator steps. Wall-clock timeout is an additional safety
deadline, not the reproducible work measure. Exhausting any preregistered work
counter yields `Overflow(WorkBudgetExceeded)` with the retained prefix and lower
bound; it never yields `Complete` or a singleton. Slice 0 measures current
counters, and the Slice 2 preflight freezes exact per-producer and aggregate
ceilings before context-neutral enumeration code begins. Those numeric ceilings
are currently `UNFROZEN_BLOCKER_FOR_SLICE2`; the 74-target storage bound cannot
substitute for them.

### 10.2 Evidence memory budget

Normative pre-implementation ceilings:

```text
MAX_TARGETS_PER_FIELD                  74
MAX_TARGET_WITNESSES_PER_TARGET         4
MAX_PINNED_PREPARED_FIELDS              32
MAX_LEASE_CONSUMERS_PER_FIELD            8
size_of(TargetWitnessV1)              <=24 B
size_of(TargetEvidenceSetV1)         <=128 B
size_of(PreparedMaterialLeaseV1)      <=128 B
all active lease metadata           <=4,096 B
evidence payload per prepared field <=9,472 B
128 ready entries                  <=1,212,416 B
32 in-flight entries                 <=303,104 B
ready + in-flight evidence payload <=1,515,520 B (1.45 MiB)
new retained evidence delta/field  <=12,288 B
160-entry retained evidence delta <=1,966,080 B (1.875 MiB)
process RSS delta for Slice 1          <=5 MiB
```

The 24-byte witness is a compact set of typed indices into immutable
prepared-field tables, not an inline copy of full hashes or strings. Full
dereferenced canonical records, overflow digests, target identities and any new
owned table bytes count toward the 12,288-byte per-field delta. Existing target
strings, lattice storage and cache metadata are reported separately rather than
hidden inside the 1.45 MiB payload claim. Slice 1 must report actual `size_of`,
allocation counts, retained heap bytes and RSS delta. Any ceiling failure blocks
the slice; the budget cannot be raised after seeing a failing result without a
new paper revision and consequences analysis.

Pinned fields remain inside the existing 128-ready/32-in-flight cache envelope;
leases do not create an unbounded side cache. When all eligible entries are
pinned, a new automatic capability is refused or display-only instead of
evicting/reusing referenced tables. Soak proof reports pinned occupancy and the
maximum age of every lease.

### 10.3 Event runtime budget

The following are product gates, not post-hoc reporting targets:

```text
printable foreground p99                         <=5.000 ms
printable foreground max                         <20.000 ms
committed Space foreground p99                  <=10.000 ms
committed Space foreground max                   <20.000 ms
explicit accept / active Space / rollback / layout p99 <=16.000 ms
explicit accept / active Space / rollback / layout max  <32.000 ms
Productive prepared-field nominal p99            <=5.000 ms
synchronous prepared-field enumeration on IBus thread       0
isolated durability prepare/co-commit p99         <=2.000 ms
isolated durability prepare/co-commit max          <8.000 ms
same-lineage state changes before prior terminal durability  0
independent foreground durability waits per steady-state event <=1
physical durability sync calls/bytes/I/O time per event       REPORTED
L1.1 service steady RSS                         <=250 MiB
aggregate target-authority installed PSS delta   <=10 MiB
Slice 1 steady RSS delta over immutable baseline <=5 MiB
```

`EventRuntimeBudgetV1` freezes exact clock boundaries, warm/cold classification,
minimum sample counts, machine identity and route labels in Slice 0. The
foreground timer starts at `ProcessKeyEvent` entry and ends when the method has
returned its handled/unhandled disposition; background completion is reported
separately. Recovery observation/compensation has its own bounded duration, but
entering `RecoveryRequired` must still satisfy the foreground gate. Failure,
refusal, passthrough, cache miss and owner-handoff paths are separate strata and
cannot be omitted from p99/max.

The isolated durability limits are an early feasibility gate, not a subtraction
from the integrated event gates. A strategy must pass them before Slice 7
behavior code and the complete event must still pass its own foreground p99/max.
The measured barrier count includes a wait inherited from the preceding event;
classifying that wait as background is forbidden. One final bounded tail flush
after an idle transition is reported separately. Background terminal sync calls
still count toward physical sync/byte/I/O totals and the next-event wait. Any
design with two independent foreground durability waits for the same steady-
state event is rejected even if an average hides the tail.

The PSS delta is the sum across all Lay-owned processes against the same frozen
idle/hot baseline; per-process RSS is also reported but is not summed as if
shared pages were private. Cache growth must reach a bounded plateau during the
fixed soak instead of passing only at startup.

Cargo and large lexical proofs run on `e@192.168.3.94`, but local IBus/WeChat/
Telegram event latency is measured on the desktop that receives the physical
input. Remote compute numbers cannot prove local compositor latency. A failed
gate blocks promotion; neither the denominator nor threshold may be changed
after results without a new consequence review.

### 10.4 Typed preservation before final bound

```text
collect bounded evidence per relation
-> validate target identity, grounding and geometry
-> deduplicate exact target and witness identities
-> accumulate contradictions and completeness before truncation
-> preserve every grounded L1.1 Winner/Tied member admitted by the L1 contract
-> preserve one valid target per active relation cohort
-> mark overflow instead of silently dropping proof
-> form cohort
-> final bounded display projection
```

No global untyped top-k may erase an admitted grounded L1.1 target or a unique
relation cohort before compatibility is evaluated. If a fixed relation reserve
cannot retain all conflict alternatives, it must carry `Overflow`; retaining
one representative does not make the cohort complete.

### 10.5 Failure matrix

| Failure | Display | Explicit accept | Committed Space | Active-composition Space | Rollback |
| --- | --- | --- | --- | --- | --- |
| producer panic/error | keep previous display only if frame-identical; otherwise clear | reject stale/missing receipt | select `RawSpace` before final authorization | commit exact composition plus Space through its own plan | unrelated; preserve rollback receipt |
| producer hang/deadline | keep previous valid display only if frame-identical | reject | select `RawSpace` after bounded wait | commit exact composition plus Space through its own plan | unrelated |
| queue/cache saturation | drop request; no new worker | reject if receipt invalid | select `RawSpace` | commit exact composition plus Space through its own plan | unrelated |
| package/schema reload | invalidate material and frame receipts | reject | select `RawSpace` | commit exact composition plus Space through its own plan | preserve exact already-issued rollback receipt only when its reader/epoch remains valid |
| focus/cursor/tail/layout/config change | discard | reject | select `RawSpace` for the exact current event | commit only against exact active frame or reject | require exact current snapshot |
| witness/target overflow | display with non-authoritative status | explicit selection may proceed through verifier | no automatic edit | unaffected | unrelated |
| L3/L4 unavailable/fails | lexical display remains | explicit selection unaffected | a tied lexical cohort selects `RawSpace` before final authorization | unaffected | unrelated |
| DecisionCore or correction-plan verification refusal | display may remain | explicit-accept refusal is terminal and unconsumed | select `RawSpace` before final authorization | not invoked | rollback-plan refusal preserves Pending only as a pre-mutation refusal |
| final selected-plan/backend authorization refusal | no mutation | `RefusedBeforeMutation`, zero mutators, no learning | `RefusedBeforeMutation`; no alternate Space plan; native passthrough only with exact eligibility proof | `RefusedBeforeMutation`; composition unchanged | restore Pending only for same-epoch pre-mutation refusal |
| journal prepare/open/write/fsync failure | unaffected | zero mutators; no learning | zero mutators; no alternate plan after selection | zero mutators; composition unchanged | zero mutators; same-epoch Pending may be restored once |
| terminal persistence/barrier failure after effect | unaffected | unfinished intent owns recovery; no learning; same-lineage native/Lay state change blocked | unfinished intent owns consumed Space and recovery | unfinished intent owns composition/effects | unfinished intent owns recovery; never restore Pending |
| corrupt/unknown journal at startup | clear stale display authority | reject Lay mutation; native typing remains available | reject Lay mutation; proved native Space only | reject Lay mutation; native composition path only when independently proved | nonterminal quarantine; no guessed inverse/compensation; explicit evidence-preserving reset only |
| native state diverged before startup reconciliation | clear stale display authority | no old acceptance success/learning | no old Space replay or inverse; native typing remains available | no guessed composition repair | quarantine old transaction as unresolved; reset creates only a new baseline |
| journal queue saturation before effect | unaffected | zero-mutator refusal or bounded backpressure under the event gate | no dropped/consumed Space; no alternate plan | composition remains unconsumed until exact disposition | preserve same-epoch Pending; no automatic retry |
| output capability/postcondition unavailable | no mutation | `AttemptedNoEffect` only after full effect-vector equality, otherwise `RecoveryRequired` | passthrough only before effects; after attempt require full no-effect proof or recovery | full no-effect proof or recovery | defer before output; after output enter exact recovery |
| partial backend output | no success publication | `RecoveryRequired`; no acceptance learning | corrected or raw Space enters `RecoveryRequired`; no plan re-entry | `RecoveryRequired`; no duplicate composition/Space | `RecoveryRequired`; never restore the original pending receipt |
| recovery postcondition unavailable | unaffected | remain `RecoveryRequired` until exact observation or expiry | remain `RecoveryRequired`; no blind compensation | remain `RecoveryRequired`; no blind replay | remain `RecoveryRequired`; no gesture replay |
| process death after transaction start | unaffected | startup reconciles durable transaction before new Lay mutation | startup reconciles durable transaction; no unaccounted consumed Space | startup reconciles preedit/text/layout effects | startup reconciliation is separate from gesture retry; native divergence enters quarantine |
| runtime-owner handoff | retain only frame-identical display | unstarted old receipt rejected | unstarted event/plan/lease revoked; started transaction transfers to recovery owner | unstarted plan revoked; started transaction transfers to recovery owner | pending gesture uses byte-proved handoff; applying/recovery state is never revoked |
| stale late publication | discard | reject | never mutate after a selected raw-Space plan completed | cannot affect committed event | cannot consume rollback receipt |

No row permits an unbounded retry, second producer, plan re-entry after final
authorization, blind compensation or late mutation. `lost/duplicate Space = 0`
is evaluated over every physical Space ID, including refusal, passthrough,
failure and recovery. Injected backend refusal must produce either one proved
unconsumed native passthrough or a typed state that still owns the consumed
event; it may not swallow the separator or be mislabeled success.

Manual layout follows the same refusal/no-effect/partial/crash rows with the
dedicated layout mutator. Journal quarantine is fail-closed for all Lay
mutation but must preserve proved native keyboard input; it is never implemented
by stopping global IBus or leaving a synthetic key held.

## 11. Learning Semantics

| Event | Interpretation | Permitted update |
| --- | --- | --- |
| suggestion shown, user types another letter | censored; target intent unknown | exposure telemetry only; no negative morphology/target label |
| suggestion shown, timeout/no explicit acceptance | censored | no target label |
| Tab or unmodified Alt accepted | explicit choice, still provisional until stable boundary | positive exact target/context event after confirmation |
| automatic correction | system action | no positive learning |
| user continues after automatic correction | still not explicit approval | no positive learning |
| immediate double Shift rollback `AppliedExact` | explicit rejection of exact automatic edit | negative event bound to exact input, target, context, generations and exact restore receipt |
| double Shift defer/refusal/no-effect/partial/quarantine/manual layout | target intent not proved | recovery/gesture telemetry only; no target label |
| explicit quarantine reset | old transaction remains unresolved; new baseline only | incident/reset telemetry; no target label, success or no-effect learning |
| refused, failed or partial output | system did not complete the intended event | recovery telemetry only; revoke provisional acceptance and emit no target label |
| next typed frame | new evidence | separate immutable event; never rewrite prior receipt |

Every feedback record binds event kind, exact observed and target identities,
operation, available context identity, package generations, evidence/cohort
hash and mutation receipt. An online overlay may be updated separately; mmap
package bytes are immutable and learning cannot bypass promotion gates.
Recovery may create a later exact terminal-settlement receipt, but it never
rewrites the failed event into target success and never starts positive or
negative correction learning. Only a later independent explicit user action
may create a new learning interval.

## 12. Proof Contract

### 12.1 Stage denominators

Every case reports independently:

| Stage | Question |
| --- | --- |
| Admission | Was the input admitted to the declared operator domain? |
| Birth | Was the exact target born? |
| Retention | Did it survive every typed bound? |
| Material completeness | Were every relevant lane and conflict alternative completely enumerated? |
| Frame binding | Did the material target bind to the exact current span and projected bytes? |
| Candidate | Was it Born, Grounded or Rejected for the right reason? |
| Conflict cohort | Did every overlapping/competing edit enter the same complete component? |
| Cohort | Was Winner/Tied/ABSTAIN correct for that component? |
| Context settlement | Was context unused, shadow-only, ambiguous or calibrated as declared? |
| Certificate | Was L2/Context authority issued only when justified? |
| Policy | Did DecisionCore accept or refuse consistently? |
| Authorization | Did all compound verifier checks agree? |
| Mutation | Did exactly one edit produce exact bytes? |
| Feedback | Was the event recorded with the correct semantics? |

### 12.2 Independent fixed proof sets

```text
L1.1 restoration                 13 x 20,000 = 260,000
Productive V90 micro proof       13 x 100 x 2 = 2,600
IME canonical correction                              36
deep diagnostics                                      12
exact-layout admission                                 5
event and Boundary/authority contract definitions 108
  existing baseline-parity definitions             49
  required future-contract definitions              59
context authority calibration             NOT FROZEN; live emission disabled
physical WeChat and Telegram              separate checklist
```

The 108-case definition manifest is
`docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/event-and-boundary-cases-v7.json`.
Its exact denominator table is:

| Denominator | Baseline parity | Future contract |
| --- | ---: | ---: |
| display | 5 | 0 |
| Tab | 4 | 0 |
| committed Space | 9 | 1 |
| active-composition Space | 1 | 2 |
| stale publication | 5 | 0 |
| rollback | 7 | 3 |
| Boundary true split | 8 | 1 |
| Boundary merge | 0 | 4 |
| Boundary false split | 10 | 0 |
| Boundary overflow | 0 | 2 |
| context-neutral material | 0 | 3 |
| evidence completeness | 0 | 8 |
| context-authority safety | 0 | 1 |
| partial output | 0 | 5 |
| transaction refusal/effect vector | 0 | 2 |
| crash recovery | 0 | 2 |
| event identity/owner handoff | 0 | 4 |
| input continuity | 0 | 2 |
| explicit-accept aliases | 0 | 2 |
| event latency | 0 | 2 |
| evidence identity | 0 | 1 |
| frame identity | 0 | 1 |
| material lease | 0 | 1 |
| witness validity | 0 | 1 |
| preservation | 0 | 1 |
| manual layout | 0 | 2 |
| durable journal | 0 | 6 |
| release provenance | 0 | 1 |
| rollback feedback | 0 | 1 |
| **Total** | **49** | **59** |

This file freezes definitions, not PASS results. Slice 0 does not pretend that
future tests already exist. Its immutable result manifest accounts for every
one of the 108 IDs with exactly one of two disjoint dispositions:

```text
ExecutedBaseline {
    exact test binary, command, selector,
    source/binary/package/config hashes,
    environment projection, exit status, semantic assertion receipt,
}

DeferredFutureContract {
    required_test, first_required_slice,
    absence verified against the immutable source snapshot,
}
```

Exactly 49 IDs must be `ExecutedBaseline` in Slice 0 and exactly 59 must be
`DeferredFutureContract`. A future case blocks its named first-required slice,
not Slice 1. Each owning slice emits a new immutable execution receipt; the
Slice 0 manifest is never rewritten. Final full promotion requires a rollup
with all 108 IDs executed and passing. A missing ID, duplicate ID, wrong phase,
unexpected filtered test or aggregate count without per-ID receipts fails the
applicable denominator.

The 13 L1.1 classes are exactly:

```text
missing_letter, extra_letter, adjacent_transposition,
letter_substitution, sparse_multi_omission,
non_adjacent_transposition, double_substitution,
omission_transposition, repeated_fragment,
prefix_truncation, suffix_truncation,
layout_projection, punctuation_suffix
```

The Productive 2,600 set and the 36-case IME set are not substitutes for the
260,000-case L1 proof. Physical application checks are not substitutes for
software denominators. Each proof reports aggregate and per-class counts,
false authority, false singleton, clean preservation, lattice coverage,
package/RSS and latency where applicable.

The 36 IME cases are also reported by first-loss stage. An authority change is
scored on the fixed authority-applicable subset whose exact target was already
born and retained; a contour-birth change is scored on the fixed birth-loss
subset; a context-settlement change is scored only on cases whose complete
grounded cohort is genuinely tied before context. The three subsets are
disjoint by first loss, frozen in Slice 0 and may not change after
implementation starts. A case whose target is absent is never called a context
failure, and a case already lexically unique is never used to calibrate
context. Only the conjunction of every applicable subset may claim final
`36/36`. Physical
active-composition Space is a separate event denominator from library APIs that
calculate hypothetical active-composition corrections.

### 12.3 Required gates

Every release receipt names exactly one claim profile:

```text
LexicalOwnerRelease
    candidate-specific lexical/Boundary ownership only
    ContextCertified disabled
    every frozen BirthOrRetention and LexicalAuthority case passes
    ContextSettlement cases remain explicit ABSTAIN and are reported, not hidden

FullTargetAuthorityRelease
    all LexicalOwnerRelease gates
    calibrated ContextCertified enabled for its exact proved scope
    every frozen ContextSettlement case passes
    complete IME conjunction is 36/36
```

The planned end state is `FullTargetAuthorityRelease`. A lexical-only build may
be an intermediate physical release, but it may not claim contextual completion
or final `36/36` and cannot silently remove context cases from the denominator.

```text
L1.1 unique top-1, every class                  >95.0%
L1.1 lattice coverage, every class              >=99.0%
clean preservation                              >=99.9%
false certainty                                      0
IME BirthOrRetention subset               100% for applicable fixed IDs
IME LexicalAuthority subset                100% for applicable fixed IDs
IME ContextSettlement subset               100% for full profile; ABSTAIN otherwise
IME correction conjunction                 36 / 36 for full profile
false automatic authority                            0
false singleton                                      0
authority from incomplete/overflow cohort            0
authority outside completeness scope                 0
context-dependent prepared target membership         0
context-dependent candidate-state or cohort pruning  0
authority from unpromoted overlay generation         0
late/stale mutation                                  0
duplicate event mutator                              0
mutator after pre-mutation refusal                    0
duplicate plan for one InputEventIdentityV1           0
mutation from stale runtime-owner lease               0
lost/duplicate Space                                 0
consumed Space without separator or recovery owner    0
exact rollback mismatch                              0
partial rollback restored as pending                 0
partial output reported as successful fallback       0
unknown effect dimension reported as no-effect        0
unfinished non-atomic transaction lost on restart     0
synthetic key left down at any terminal/crash point   0
```

The Productive nominal latency gate remains 5.000 ms. Previously accepted
5.317 ms and measured 5.286 ms checkpoints are evidence, not a silent change
to the nominal gate. Any exception must be explicit, receipt-scoped and must
not hide hot IBus thread latency.

Before Slice 9 implementation, its risk-policy receipt must freeze the
one-sided confidence level, maximum aggregate and per-family false-authority
upper bounds, minimum samples, family taxonomy and lineage-separated
tuning/calibration/final partitions. These values are intentionally not chosen
after reading calibration outcomes. Until that receipt exists,
`ContextCertified` remains a compile/runtime-disabled capability regardless of
point estimates.

### 12.4 Required ablations and shortcut baselines

Run the same frozen cases with:

1. target certificate disabled;
2. context disabled;
3. internalized Boundary disabled;
4. old `surface_count > 1` field-wide rule restored as a non-live baseline;
5. source-priority selection as a non-live baseline;
6. current context-shaped Productive enumeration as a non-live baseline;
7. overflow/completeness propagation disabled as a non-live safety baseline.

Report target birth, retention, cohort verdict, authority, false accepts and
latency separately. An ablation is explanatory evidence only and never a live
fallback.

### 12.5 Hardcode and route proof

Static regex vetoes are early tripwires only. Promotion additionally requires:

- AST/source route parity for every event;
- runtime producer/ranker/authorizer/mutator cardinality;
- fixed semantic surface/verdict/proof assertions independent of source ID;
- zero fixture word/phrase, source ID, test ID or class-specific runtime branch;
- whole fixed proof rerun after every behavior change.

## 13. Migration Slices

Each slice starts from the exact source SHA produced by the preceding slice,
has a new implementation preflight, and changes no later behavior early.
Before a slice exits, every V7 future-contract case whose
`first_required_slice` equals that slice number must have a one-to-one execution
receipt. A missing or deferred owning-slice case blocks that exit and cannot be
rolled forward into a later aggregate.

Slice 0 freezes the denominator; it does not require later runtime routes to
exist early. A future contract that is byte-absent from the frozen source is a
valid `DeferredFutureContract` only when its exact `first_required_slice` is
recorded. That absence blocks the named owning slice and final promotion, not a
preceding vocabulary-only slice. Likewise, individually measured current-route
latency strata may pass their fixed limits while complete
`EventRuntimeBudgetV1` remains `FAIL_COVERAGE`. Neither result may be relabeled
as an overall latency PASS.

### Slice 0: new immutable baseline and denominator freeze

- retain the copied 36-case and 12-case logs as historical diagnostic evidence
  under `LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/raw`;
- do not invent the lost execution-time source bytes or promote those logs to a
  reproducible baseline;
- before a new run, create an immutable source archive containing commit,
  submodule state, every dirty tracked/untracked source byte used by the build,
  mode, size and SHA-256, plus package/config/toolchain identities;
- embed the complete pre-run execution plan, exact commands/selectors and
  expected case dispositions in the immutable
  `source-at-execution.sha256-manifest.json`; the post-run results manifest
  references that exact manifest hash and cannot substitute a later plan;
- record an existence bit for every Slice 1 allowlisted path; in particular,
  prove `src/typing_transition/target_evidence.rs` absent before creation;
- run the 36-case, 12-case and the 49 existing baseline-parity Boundary/event
  definitions remotely from that exact archive through `scripts/cargo-guard.sh`
  and store raw logs beside the archive manifest;
- account for the 59 future-contract IDs as `DeferredFutureContract` with the
  exact required test, first-required slice and source-snapshot absence proof;
  do not invent commands, binaries or PASS receipts for tests that do not exist;
- freeze every denominator and expected event result before the run;
- freeze a machine-readable execution plan that maps every case ID to one
  exact binary, command, selector and expected semantic receipt;
- record first loss for every case, including material completeness, frame
  binding and event mutation count;
- freeze disjoint `BirthOrRetention`, `LexicalAuthority` and
  `ContextSettlement` first-loss subsets for the 36-case set before any
  implementation or context calibration;
- reject the run if any frozen ID is missing, duplicated, unexpectedly
  filtered or reported only through an aggregate counter;
- record deterministic current producer scan/replay/grounding work counters as
  evidence for the separate Slice 2 budget decision;
- record every implemented current runtime stratum on the local desktop under
  the fixed Section 10.3 outer clock, including printable, committed Space,
  explicit accept, rollback, layout, refusal/failure and owner handoff, while
  Cargo and lexical proofs remain remote;
- label every required but not-yet-implemented runtime stratum explicitly and
  bind it to its owning slice; absence is never counted as a latency PASS;
- freeze current IBus/daemon runtime-owner and key press/release/repeat traces so
  later exactly-once claims have an immutable comparison denominator;
- retain the revised current-route `VETO`; no source behavior changes.

Exit: a new baseline is byte-reproducible, all 108 definitions have exactly one
valid Slice 0 disposition, the 49 executable cases have one-to-one receipts,
and every implemented local runtime stratum has an outer-clock result while
future strata have explicit absence labels and owning slices. This is baseline
freeze PASS only. It does not imply assertion-quality PASS, complete
`EventRuntimeBudgetV1` PASS, implementation correctness or deployment authority.
The historical partial logs remain useful but are not the authority baseline.

#### 2026-08-20 Slice 0 immutable execution result

Slice 0 baseline freeze has exited. Immutable source and remote execution
integrity passed, all implemented current runtime strata have valid outer-clock
measurements, and every missing future stratum is explicitly assigned to Slice
7. Three frozen baseline failures remain measured behavior debt. Complete
`EventRuntimeBudgetV1` remains `FAIL_COVERAGE`; that blocks Slice 7 exit and
final promotion, but it does not block the vocabulary-only Slice 1.

What was tested:

```text
frozen source files                         2,214 / 2,214 hash-valid
source archive bytes                         167,429,239
source archive SHA-256      9b49c1445df9e40f5a82d492b55a23dc71f5dbd5ca6bfb4fceb5cea467179427
planned remote executions                         97
valid one-case executions                    97 / 97
future contracts absent in frozen source     59 / 59
build receipts                                 4 / 4, exit 0
invalid or filtered executions                     0
runtime authority changed                      false
```

Measured assertion results:

```text
IME 36                                18 PASS / 18 FAIL
deep 12                                0 PASS / 12 FAIL
baseline 49                           46 PASS /  3 FAIL
aggregate                             64 PASS / 33 FAIL
```

The three frozen baseline failures are:

```text
false-split-ambiguous-short-shift
false-split-non-boundary-source
split-authority-binds-target
```

They are measured baseline failures, not permission for literal cases or an
after-the-fact denominator change. `PASS_INTEGRITY_ONLY` means that source,
plan, build, log and receipt identities are valid; it does not mean quality,
latency, Slice 0 exit or implementation readiness passed.

First-loss analysis found one shared typed-evidence defect rather than three
fixture defects. Boundary birth, target binding, proposal admission and final
DecisionCore admission currently reconstruct different booleans from surface
shape. Consequently:

- the `strong_short_left_field_boundary` birth path is absent from the exact
  target-evidence predicate, so a born split can lose its binding;
- a verifier-valid `BoundaryShift` can gain automatic authority although the
  verifier proves edit safety, not target correctness;
- a text-shaped split from a non-Boundary origin can enter a Boundary-specific
  suggestion gate before the stronger preservation result.

The required correction is one exact Boundary evidence value shared by birth,
IME display and full correction. It binds observed contour, exact target parts,
segmentation, typed operator and completeness. Structural verification remains
safety-only, and Boundary-specific admission requires the typed Boundary
origin/operator. This is an analysis result only; no source was changed.

Attempt history is retained without replacing evidence:

```text
attempt 1  ABORTED_NONAUTHORITATIVE   8 receipts; serial projection ~37 min
attempt 2  REJECTED_PARSER_V1        97 runs; 62 parser-invalid receipts
attempt 3  VALID_EXECUTIONS           97/97 valid; 64 PASS / 33 FAIL
```

Attempt 2's 62 rejected logs were independently reparsed with zero selector or
terminal-summary gaps. They remain rejected because the parser was frozen
incorrectly; attempt 3 is the only result-bearing run.

The authoritative local V3 probe used a private D-Bus/IBus owner with the exact
installed Lay `1.0.33` engine and the outer request/reply clock. It made zero
production-bus, physical-input or deployment calls and left all production
owners stable. Seven implemented current-route strata passed their own fixed
limits:

```text
printable                   512 samples, p99 0.347 ms, max 0.458 ms  PASS
committed Space             256 samples, p99 8.566 ms, max 11.033 ms PASS
explicit accept             128 samples, p99 0.864 ms, max 1.089 ms  PASS
rollback                    128 samples, p99 0.967 ms, max 2.304 ms  PASS
layout                      128 samples, p99 0.772 ms, max 0.856 ms  PASS
failure/refusal             128 samples, p99 0.401 ms, max 0.402 ms  PASS
owner handoff               128 samples, p99 0.289 ms, max 0.313 ms  PASS
```

Rollback restored `128/128` distinct frozen inputs with zero semantic failures.
The typed trace contains 14,546 contiguous rows with zero typed failures. One
`0.407 ms` first-touch observation is informational only and is not a cold-start
PASS. Active-composition Space, repeat identity, durability prepare/co-commit
and the same-lineage barrier do not exist in the installed runtime. Therefore:

```text
measured current-route strata          PASS, 7/7
latency verdict                        FAIL_COVERAGE
complete EventRuntimeBudgetV1          FAIL_CURRENT_RUNTIME_ROUTES_NOT_IMPLEMENTED
```

The old two-stratum local sample remains historical evidence only and is
superseded by V3 for current-route latency. V3 still makes no compositor,
focused-application or physical-key latency claim.

What was not tested or authorized:

- active-composition Space, repeat identity, durability prepare/co-commit and
  same-lineage barrier latency or behavior;
- a complete local `EventRuntimeBudgetV1` PASS;
- correction of the three frozen baseline failures;
- any Slice 1 source change;
- daemon/IBus restart, installed binary replacement or live-owner change;
- deployment or runtime-owner promotion.

Exact external evidence root:

```text
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820
```

Exact receipts:

```text
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/source-at-execution.sha256-manifest.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/output/run-summary.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/remote-run-validation.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/future59-source-absence.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/local-event-runtime-baseline.partial.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/baseline49-boundary-first-loss-analysis.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/partial-results-manifest.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/private-ibus-latency-probe-v3.py
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/private-ibus-latency-probe-execution-plan-v3.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/private-ibus-latency-probe-preflight-v3-receipt.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/private-ibus-latency-probe-run-v3.json
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/private-ibus-latency-probe-run-v3.trace.jsonl
```

The historical partial manifest SHA-256 is
`3610368ddc06b95eab2b1b7e72940b0b9e9488a8b024000e18d4af8ca2e83254`
and remains explicitly non-promotable. The final Slice 0 repository artifacts
are:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/immutable-rerun/source-at-execution.sha256-manifest.json
  SHA-256 45389cef6c5843473799a5e2df0c066c12ac77e43b5b598d2c3d91158f5af511
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/immutable-rerun/results-manifest.json
  SHA-256 986ea9be4a89c64c3e29b4102d9d554fb63843bd32dd0baa79de812a573c97da
```

The results manifest says `PASS_BASELINE_FREEZE_ONLY`, preserves assertion
quality as `FAIL 64/97`, and preserves complete latency as `FAIL_COVERAGE`.
The private typed trace remains mode `0600` outside the repository and must not
be committed or published.

### Slice 1: evidence vocabulary only

- add `src/typing_transition/target_evidence.rs` as the only vocabulary owner;
- introduce material/frame identities, compact witness references,
  completeness, conflict footprints, candidate/cohort and certificate types;
- adapt `L2ImeTargetEvidence`, `ReplacementTargetEvidence`,
  `UnifiedCorrectionCandidate`, morphology and display metadata without
  changing current rank, cache key, authority, mutation or event routes;
- pin and inventory every current owner, including `productive_v1/live.rs`,
  `bridge.rs`, `l2/ime_readout.rs`, correction adapters, frame types and the new
  module export;
- restrict source edits to this exact maximum allowlist:
  `typing_transition/target_evidence.rs`, `typing_transition/mod.rs`,
  `typing_transition/live_candidate.rs`, `typing_transition/candidate.rs`,
  `nanda_wave/l2.rs`, `nanda_wave/l2/ime_readout.rs`,
  `nanda_wave/l2_field/bridge.rs`, `productive_v1/live.rs`,
  `productive_v1/composite.rs`, `productive_v1/scene.rs`,
  `correction_core.rs` and `correction_core/candidate_sources.rs`;
- require every source file outside that allowlist, especially cache,
  DecisionCore, display, mutation/rollback and verifier owners, to remain
  byte-identical in Slice 1;
- keep temporary compatibility adapters only at named module boundaries;
- keep package generation in material validity/cache identity but exclude it
  from semantic witness independence; package aliases of one derivation lineage
  must merge before the four-witness bound;
- permit legacy-to-common-to-legacy round-trip only over the exact legacy
  singleton domain; multi-witness, composite or incomplete common evidence
  must fail reverse projection instead of truncating;
- measure actual type size, allocation count, retained bytes and RSS delta.

Exit: exact semantic/event parity, evidence round-trip parity, deterministic
merge, overflow state and all Section 10.2 ceilings pass. No new type is yet an
authority owner.

Implementation result, 2026-08-20: `PASS_SLICE1_EVIDENCE_VOCABULARY_ONLY`.
The V8 preflight pinned the immutable Slice 0 manifests and authorized the
isolated vocabulary change. `target_evidence.rs` is now the common owner of the
bounded witness, material/frame identity, completeness-scope, lease,
candidate/cohort and certificate vocabulary. Compatibility projections remain
on demand and do not participate in live rank, admission, display, cache,
mutation or verifier authority.

The final remote candidate V4 produced `4/4` build PASS, `5/5` focused contract
groups, `49/49` valid frozen executions with zero status differences and `49/49`
exact normalized semantic logs. Missing and lossy adapter fault injections were
both rejected. The three frozen baseline failures remain exactly `46 PASS / 3
FAIL`; this slice proves no behavior change and does not claim their repair.

```text
TargetWitnessV1                         24 B
TargetEvidenceSetV1                   128 B
PreparedMaterialLeaseV1               112 B
74 evidence sets                    9,472 B
PreparedTargetMaterialV1           11,376 B
active lease metadata               3,584 B
160-field retained delta         1,820,160 B
remote median RSS delta             1,280 KiB / 5,120 KiB limit
```

The `9,472 B` payload is exactly the 74 bounded evidence sets. The larger
`11,376 B` complete prepared object also includes material target identities and
the context-neutral envelope and remains below the separate `12,288 B` retained
delta ceiling. Malformed semantic-root accelerators are canonicalized before
retention. Narrow completeness scope is unrepresentable without a non-zero
reference to an exhaustive pre-truncation partition proof; mismatched scopes
merge to a deterministic whole-field integrity failure. An exhaustive Rust
destructuring test prevents frame-bound fields from entering
`PreparedTargetMaterialV1` unnoticed.

Final receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE1_EVIDENCE_VOCABULARY_2026-08-20/final-receipt.json`

Runtime authority changed: `false`. Deployment actions: `0`. Slice 1 does not
authorize Slice 2; the next source mutation requires a new implementation
preflight with deterministic enumeration-work budgets.

### Slice 2: context-neutral material and exact frame binding shadow

- publish a new paper/preflight revision that freezes deterministic
  per-producer and aggregate `EnumerationWorkBudgetV1` ceilings from Slice 0
  measurements before source edits;
- build the prepared lexical/geometry target set without context, focus, tail
  epoch, replacement span, usage overlay or selected winner;
- derive frame targets by exact span binding, projection and replay after cache
  lookup;
- bind caret, selection, exact source window, focus serial and active preedit
  bytes/cursor into frame equality;
- introduce bounded prepared-material leases so field-local references cannot
  survive eviction, generation reuse or owner handoff;
- compare multiple left contexts over identical lexical material and require
  identical target/witness/completeness digests;
- keep the current context-shaped cache/live route unchanged while this work is
  offline/shadow-only; do not add a second hot-path worker;
- if bounded context-neutral Productive enumeration is impossible with the
  current package/API, stop and revise the Productive contract before code
  promotion.

Exit: context-neutral target membership is 100%, stale frame reuse is zero and
the new material/frame split has bounded storage and work-budget receipts;
budget exhaustion is observed only as explicit incompleteness.

### Slice 3: candidate validity shadow

- derive `Born | Grounded | Rejected` only after exact frame binding;
- assess witness failures locally; preserve an independently valid witness and
  convert malformed/incomplete witness integrity into an authority blocker;
- require a complete grounding namespace before `TARGET_ABSENT_FROM_GROUNDING`;
  overflow/failed lookup retains `Born` as an unresolved authority blocker;
- add prepared original material and frame preservation verdict outside the
  replacement target set;
- accumulate contradictions before truncation and propagate all upstream
  overflow states;
- compare states against the immutable baseline without changing live rank or
  gate.

Exit: source-neutral state derivation, zero false grounding and zero reusable
frame rejection inside cached material.

### Slice 4: conflict cohort and verdict shadow

- form edit footprints and complete conflict components after target validity;
- include every footprint of an unresolved `Born` alternative in completeness
  settlement even though only grounded targets can be tie members;
- merge exact duplicate targets by semantic evidence root, not producer ID;
- apply `CanonicalCohortOrderV1` before cohort hashing or context observation;
- apply every `CompletenessScopeV1` before declaring a component complete;
  whole-field overflow blocks all cohorts and narrow scopes require their
  exhaustive partition proof;
- derive `Winner | Tied | ABSTAIN` with margin and context authority disabled;
- consume exact original-preservation evidence before every Winner/certificate
  path, including future context settlement;
- retain all admitted grounded L1.1 Winner/Tied members and block authority for
  any hidden overflow alternative or multiple independent edit component;
- observe current L3/L4 only as display/shadow evidence.

Exit: truth-table parity, zero false singleton, zero lost grounded L1.1 target,
deterministic output and no authority from incomplete cohorts.

### Slice 5: missing target birth and retention shadow

- operate only on the fixed cases whose first loss is Birth or Retention;
- add bounded typed contour relations for the shared missing-initial,
  duplicate-prefix and other frozen mechanisms, never literal surfaces or a
  deterministic fallback route;
- feed born targets through the same material identity, grounding, geometry,
  completeness and conflict-cohort contracts;
- report birth and authority denominators separately and rerun the whole fixed
  proof after every mechanism change.

Exit: the fixed birth-loss subset reaches exact target birth/retention without
regressing any L1.1 class, clean preservation, false authority or latency gate.
This slice alone may not claim `36/36` correction.

#### Slice 5 identity-provider correction after the first measured smoke

The first real-package Slice 5 smoke rejected the assumption that the canonical
L2 form bank alone spans the fixed birth denominator. It retained `4/8` targets
with `0` work overflows and `0` authority grants. The four losses were not
operator or frontier losses: the correct transformed bytes were reachable, but
their exact surface identity was absent from canonical L2 v13. Raising the work
budget or adding more edit operators would therefore not address the first
loss.

Context-neutral contour birth consequently resolves identity against a bounded
union of three independently versioned exact providers:

```text
CanonicalForm(package generation, form ref)
L11Terminal(package generation, terminal id)
ReferenceSurface(embedded snapshot generation, exact surface ref)
```

Provider origin is preserved as a semantic witness root. Duplicate bytes merge
only after all provider roots are collected. Host Hunspell files, live usage
counts, context, source IDs and fixture-specific strings are excluded from this
identity union. Canonical membership, L1.1 terminal membership and immutable
reference membership establish `Born` identity only; none can mint `Grounded`,
Winner or mutation authority without the separately admitted evidence and frame
contracts.

The L1.1 provider is an exact terminal lookup, not an L1.1 restoration query and
not a second candidate ranker. In proof it may be loaded directly from the
pinned package. A future live implementation must use one bounded batch/index
owner and include its package generation in `PreparedMaterialKeyV1`; it must not
perform one IPC request per generated surface or duplicate the full L1.1 model
inside every client.

Measured receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE5_CONTOUR_BIRTH_2026-08-20/slice5-smoke-13x1.json`

Measured scope: fixed `8` birth cases plus `13x1` shadow material/cohort smoke.
Not tested: fixed `13x100`, live latency, daemon/IBus, display, authorization or
mutation. Verdict: reject canonical-only identity resolution; runtime authority
remains unchanged.

#### Slice 5 typed contour-reserve correction after exact identity union

The first exact-provider-union smoke proved birth but exposed a different first
loss. Exact birth reached `8/8`, work remained within budget for `8/8`, contour
work overflow remained `0`, authority grants remained `0`, false singleton and
integrity errors remained `0`, and maximum class p99 was `4.621 ms`. Retention
was only `5/8`: `зва -> pdf` and the two frozen `yt -> не` rows were born but
then lost when the prepared material sorted every productive and contour
surface into one byte-ordered list and truncated that list at `74`.

This rejects neither the provider union nor the fixed `74`-target envelope. It
rejects the implementation that ignored the already specified typed lane
reserves. The next correction is therefore:

```text
complete bounded contour enumeration
-> exact surface deduplication and provider-root merge
-> merge roots into an already retained productive surface without lane cost
-> retain at most 8 novel contour surfaces
   -> direct ExactLayout surfaces first
   -> deterministic round-robin across remaining typed relation partitions
   -> byte order only inside one relation partition
-> Overflow(StorageCapacity, WholePreparedField) when logical contour surfaces
   exceed the reserve
-> no Winner and no authority from the incomplete field
```

The contour reserve is storage selection, not candidate authority. The complete
logical contour set and its all-seen digest are computed before storage
selection. Hidden alternatives remain represented by explicit whole-field
overflow, so a retained surface cannot impersonate a complete singleton. The
global `74`, the `16,384` lookup/step budgets, SafetyGate, verifier, live bridge,
daemon, IBus and installed runtime remain unchanged.

Measured receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE5_CONTOUR_BIRTH_2026-08-20/slice5-smoke-exact-union-13x1.json`

Verdict scope: exact identity union PASS for birth, typed contour storage
retention FAIL pending the lane-reserve correction; runtime authority remains
unchanged.

#### Slice 5 final typed-reserve proof

The lane-reserve implementation passed the fixed and aggregate functional
proofs without changing runtime authority:

```text
fixed contour birth / retention / born-only       8/8 / 8/8 / 8/8
fixed contour work within budget / overflow       8/8 / 0
fixed authority grants                              0
13x100 lemma-heldout hypothesis target members 1280/1280
material work within budget                    1300/1300
failed material / integrity errors                  0 / 0
false singleton / lost grounded target              0 / 0
multiple-component authority / preservation bypass  0 / 0
full wall / CPU / peak RSS             33.76 s / 1037% / 567076 KiB
```

The material proof recorded `389` complete fields and `911` explicit
fail-closed incomplete fields: `77` `StorageCapacity` and `834`
`UpstreamIncomplete`. No incomplete field issued a Winner.

The absolute 20-worker Productive readout p99 remained above the release gate:
`11.157 ms`. A paired run of the same binary and corpus with contour disabled
was worse at `12.533 ms`. The Productive timer starts after material/contour
preparation and the readout remained byte-equivalent (`2600/2600` base
projection comparisons, `0` failures). Therefore Slice 5 did not introduce the
latency failure, but it also did not cure it. The absolute `<=5 ms` contract
remains open for Slice 11 and blocks deployment.

Receipts:

- `/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE5_CONTOUR_BIRTH_2026-08-20/slice5-full-typed-reserve-13x100.json`
- `/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE5_CONTOUR_BIRTH_2026-08-20/slice5-paired-baseline-no-contour-13x100.json`

Verdict scope: `PASS_SLICE5_BIRTH_RETENTION_NO_LATENCY_REGRESSION`; overall
release verdict remains FAIL because the inherited absolute latency gate is not
met. Not tested: live bridge, daemon/IBus, display, authorization, mutation or
physical multi-client behavior. Installed runtime and authority remain
unchanged.

### Slice 6: Boundary internalization shadow

- emit split/merge as typed targets inside the context-neutral prepared field;
- construct them from `TokenContour` or exact `BoundaryWindowContour` and bind
  every split part through `CompositeBoundaryGroundingV1`;
- require two-sided/merged grounding and exact separator-only geometry, or an
  explicitly typed compound operator for any additional edit;
- compare split, merged and whole-token footprints before cohort settlement;
- propagate Boundary reserve overflow instead of treating two retained splits
  as complete;
- leave the old Boundary route live only while the new route is
  non-authoritative shadow and compare exact outcomes.

Exit: fixed true split, merge, multi-split-overflow and false-split denominators
pass with no target loss or new synchronous hot-path work.

### Slice 7: event identity and crash-safe output transaction

- pass a dedicated Slice 7A preflight whose allowlist contains only an isolated
  durability harness and proof artifacts, forbids runtime source edits and
  changes no installed/runtime authority;
- under that preflight, implement only the isolated durability harness, freeze
  `TransactionDurabilityStrategyV1` and pass the Section 9.8/10.3 local
  prepare, co-commit, tail-flush, next-native-event, saturation and failure
  microproof; a failed strategy returns to paper review rather than being
  patched inside the runtime;
- create a separate Slice 7B runtime implementation preflight that pins the
  exact passing strategy receipt, harness bytes and measured environment before
  any event/transaction behavior source is edited;
- create one shared event/transaction vocabulary owner, planned as
  `src/text_edit/event_transaction.rs`; its exact existence state, module
  boundary and touched-file allowlist require a new slice preflight;
- bind every mutating adapter to `RuntimeInputOwnerLeaseV1` and
  `InputEventIdentityV1`, including press/release/repeat classification;
- construct typed `CommittedSpacePlanV1`, `ExplicitAcceptPlanV1`, active-Space
  and rollback plans exactly once before output authorization;
- distinguish `RefusedBeforeMutation` (zero mutators) from
  `AttemptedNoEffect` (one mutator plus full effect-vector equality);
- implement `OutputEffectSnapshotV1`, durable transaction identity, startup
  reconciliation and bounded recovery, or prove one crash-atomic backend
  primitive for the complete effect vector;
- split startup reconciliation into zero-mutator exact observation and one-
  mutator authorized compensation; persist `AppliedExact`,
  `AttemptedNoEffect` or `RecoverySettledExact` before opening the barrier;
- admit native passthrough only before every effect and only for a frozen
  backend/event capability; account for every consumed Space;
- prove Tab and unmodified Alt adapters create the same typed acceptance plan,
  while modifier Alt cannot accept;
- execute every V7 future-contract case whose `first_required_slice` is 7 and
  inject process death at every non-atomic effect boundary;
- prove `InputCommandIdentityV1` for press-triggered, paired-release and
  multi-event gesture commands, with no member-event reuse;
- select exactly one double-Shift rollback/defer/reject/manual-layout plan and
  route manual layout through its dedicated mutator under the common protocol;
- commit one exact intent before the first effect through the selected
  durability strategy, persist a terminal state before success/learning or any
  later same-lineage state change, and make every recovery step crash-idempotent;
- prove the same-lineage barrier covers native input as well as Lay mutation,
  and that ordered group commit co-commits the prior terminal state with the
  next prepare without a second independent foreground wait;
- inject journal open/write/fsync failure, corruption, incompatible schema,
  terminal-barrier failure, native post-crash divergence, callback-return death
  and active-transaction owner handoff;
- prove quarantine disables only Lay mutation, preserves native keyboard input
  and never stops global IBus or leaves a key down; prove explicit reset
  preserves the unresolved incident, persists `QuarantineResetReceiptV1`,
  creates only a new baseline and teaches nothing; prove emergency key cleanup
  has exact idempotent `KeyCleanupEffectProofV1` and cannot settle text output;
- first compare the protocol against the current live candidate decisions; only
  after the event/output proof passes may it become the live transaction owner
  for those unchanged decisions.

Exit: one runtime owner, one command plan, one selected and early-measured
durability strategy, zero mutators before refusal, exactly
one state-correct mutator after an attempt, durable intent and terminal-state
ordering, same-lineage native/Lay barrier, full no-effect equality,
crash-idempotent recovery, keyboard-safe nonterminal quarantine and reset,
exclusive rollback/manual-layout dispatch, no stuck key and no lost/duplicate
Space. Candidate ranking and automatic lexical authority are still
byte/decision-parity with the baseline.

### Slice 8: lexical candidate-specific live readout

- promote one candidate/cohort model to the only automatic lexical readout;
- issue `L2Certified` only for a complete lexical `Winner`;
- keep `ContextCertified` disabled and convert every unresolved tie to
  `ABSTAIN`;
- remove `common_l3_required` and producer/source priority from authority;
- preserve display and exact explicit-accept semantics;
- route every admitted action through the already-proved Slice 7 transaction;
- turn the old Boundary/field-wide routes non-authoritative before promotion;
  retain compare observation for one bounded run only.

Exit: the fixed lexical-authority and birth subsets, every non-context 36-case,
L1/Productive/Boundary/event gates, zero false authority/singleton and focused
concurrency parity all pass. Final `36/36` is claimed here only if Slice 0 froze
an empty `ContextSettlement` subset; otherwise it remains pending through
Slice 9.

### Slice 9: context authority calibration and promotion

- before implementation, freeze the numeric confidence/risk policy, minimum
  sample counts and family/lineage partitions in a separate immutable receipt;
- freeze an independent heldout tie-cohort denominator before selecting a
  context decision rule;
- separate tuning, calibration and final heldout partitions by lexical family
  and context lineage; no threshold or overlay may be selected on final
  heldout outcomes;
- bind the exact `LeftLocalContext2` capability, selector/model hash, all tied
  members and overlay generations;
- prove complete-cohort evaluation, aggregate and per-family coverage, zero
  observed false authority, and the predeclared finite-sample confidence bound
  under prescribed ablations;
- keep every newly learned overlay generation display/shadow-only until a
  separate receipt promotes those exact bytes as `AuthorityEligible`;
- run shadow first; only a new preflight and PASS receipt may enable
  `ContextCertified` for Space.

Exit: either `ContextCertified` is separately proven and promoted, or it remains
disabled without blocking a lexical-only release. A lexical-only release must
say explicitly that contextual automatic authority is not implemented and may
not claim final `36/36` when the frozen `ContextSettlement` subset is nonempty.
If context is promoted, that subset and then the complete 36-case conjunction
must pass without changing membership.

### Slice 10: remove compatibility routes

- delete the parallel `boundary_text_candidates()` authority path;
- remove redundant evidence enums/adapters after all consumers migrate;
- remove old field-wide authority transfer and every temporary shadow
  worker/compare route;
- prove no post-refusal or package-failure fallback;
- regenerate observed-source route contracts for every event.

Exit: one prepared-material route, one runtime-owner lease and one
event-specific singleton owner for rank, authorization and attempted mutation,
with no permanent dual computation.

### Slice 11: performance, cache and failure proof

- run Cargo and large lexical proofs only on `e@192.168.3.94` through
  `scripts/cargo-guard.sh`;
- run local IBus/WeChat/Telegram latency and physical event accounting on the
  desktop receiving the exact installed candidate bytes;
- report cold/hot p50/p95/p99/max, CPU, allocations, full retained cache bytes,
  cache occupancy, per-process RSS and aggregate PSS;
- execute the V7 event-latency future contracts and rerun every Slice 7 refusal,
  crash, partial-effect, passthrough, event-identity and restart fault;
- rerun the selected durability strategy inside the installed event route,
  including inherited prior-terminal waits, native-event barriers, tail flush,
  queue saturation and explicit quarantine reset; report barrier count per
  physical event rather than only journal-worker timing;
- inject producer panic/hang, saturation, reload, stale lease, context failure,
  verifier refusal, owner handoff, process death, partial backend effect and
  rollback expiry;
- prove no heavy synchronous field work on printable or Space IBus threads;
- prove every Section 10.3 gate, exactly one Space, no late edit, finite rollback
  and atomic/crash-durable output semantics;
- prove pre-mutation rollback refusal returns to `Pending`, while any changed or
  unknown effect enters `RecoveryRequired` and cannot be replayed by double
  Shift.

Exit: all software promotion gates pass for one fixed release-candidate build.

### Slice 12: versioned release candidate and physical promotion

1. declare `LexicalOwnerRelease` or `FullTargetAuthorityRelease` and verify the
   corresponding fixed denominator before changing version metadata;
2. update version, metadata, recovery reader and all release-source documents;
3. create and push one frozen release-source commit; record its exact tree and
   prove the build worktree byte-identical to it;
4. build the final release candidate once on the remote build host and record
   release-source commit, binary SHA and package/config/recovery-reader SHAs;
5. run final software proof against those exact bytes and execute every one of
   the 108 case definitions required by the declared profile/owning slices;
6. snapshot rollback bytes and durable transaction journal; install and verify
   the byte-pinned recovery/schema reader before any live-owner flip;
7. install the exact proved binary without rebuilding;
8. verify installed SHA equals proved SHA;
9. restart only the required Lay-managed component, never global IBus;
10. physically verify WeChat and Telegram, including no stuck key, repeated
   character/Space, lost separator, stale edit, owner handoff, partial effect,
   crash reconciliation and failed or expired double Shift;
11. issue the live-owner promotion receipt and update architecture/graphify in
    a distinct receipt-only evidence commit; do not modify source, metadata,
    package, recovery reader or tested binary bytes, and never relabel the
    evidence commit as the binary's release-source commit.

Exit: physical PASS on the exact versioned SHA. Any source, metadata or binary
change after step 4 invalidates the release candidate and restarts Slice 12.

## 14. Rollback Boundary

Before any live deployment preserve:

- exact installed binary, version and SHA;
- L1.1, canonical L2 and Productive package bytes and hashes;
- installed extension files and metadata;
- configuration;
- prepared-field/cache schema identities;
- durable output-transaction journal schema, unfinished transaction set and
  startup reconciliation receipt;
- feedback-ledger frozen prefix and natural suffix policy;
- software proof and event-route receipts.

Rollback restores binary/config/package readers but preserves naturally arrived
feedback suffix records. It may stop reading a new overlay schema; it must not
delete, truncate or rewrite user events. Double Shift receipts issued before
rollback remain usable only if the rollback binary has a byte-proved compatible
reader or the deployment process converts them to the previous exact schema
before restart. Otherwise they are explicitly invalidated without mutation;
the plan must not promise that an older binary understands a newer in-memory or
persisted receipt. The physical promotion checklist tests this boundary.

Rollback cannot simply discard an unfinished output transaction created by the
new binary. Before binary replacement, every transaction is either reconciled
to an exact terminal effect snapshot or converted by a byte-proved reader into
the rollback binary's recovery schema. If neither is possible, rollback is
blocked while the user-visible state is quarantined; treating an unknown partial
effect as clean rollback is forbidden.

## 15. Preflight Contract

The broad revision-1/revision-3 preflights and the pre-audit Revision-4 Slice 1
preflight are superseded. A post-Slice-0 Slice 1 preflight V8 or later may become
`READY_TO_IMPLEMENT` only after it pins:

1. a new immutable source archive, 49 executable baseline receipts, 59 exact
   deferred dispositions, durable Slice 0 logs and frozen manifests;
2. exact source existence/SHA/size/mode for every touched path, including
   proved pre-edit absence of `target_evidence.rs`;
3. the Revision-4 target event-route, Revision-6 material/frame and Revision-8
   output-transaction design
   receipts;
4. the 74-target/four-witness/24-byte/128-byte, bounded material-lease and
   retained-delta ceilings;
5. normalization, material target, frame target, witness-root, completeness and
   conflict-footprint contracts;
6. deterministic merge, fail-closed compatibility projection and overflow
   fault tests;
7. proof that rank/display/authority/mutation are unchanged in Slice 1;
8. an isolated implementation worktree, touched-file manifest and fail-closed
   partial adapter migration without broad revert commands;
9. forbidden effects covering hardcode, unbounded growth, cache authority,
   synchronous hot waits, verifier weakening and deployment;
10. the next preflight invalidation rule after any source mutation.

The Slice 1 receipt also rejects any changed source path outside the exact
allowlist above. The allowlist is a ceiling, not an instruction to edit every
file. Expanding it requires a new preflight revision before the additional
edit, not an after-the-fact receipt update.

`LAY_IME_TARGET_AUTHORITY_SLICE1_IMPLEMENTATION_V7_BLOCKED_2026-08-17.json`
is deliberately non-promotable. Its absent immutable-rerun entries carry an
impossible all-zero SHA sentinel so merely creating files at those paths cannot
turn this historical blocked receipt into `READY_TO_IMPLEMENT`. Slice 0 must:

1. write and semantically validate the complete source manifest and the
   49-executed/59-deferred results manifest;
2. record their actual SHA-256, size and mode;
3. create a new Slice 1 preflight revision containing those exact measurements
   and the validator receipt;
4. retain V5/V6/V7 and their blocked receipts unchanged.

No one-byte placeholder, post-run expected-result edit or in-place relaxation
is an accepted path to readiness.

The touched-file baseline must include every existing evidence or authority
owner discovered in the current route, not only the intended new module. At
minimum that includes `nanda_wave/l2.rs`, `nanda_wave/l2/ime_readout.rs`,
`nanda_wave/l2_field/bridge.rs`, `productive_v1/live.rs`, cache/correction
adapters, `typing_transition/live_candidate.rs`, frame identity, DecisionCore,
display, mutation/rollback and verifier modules. An omitted owner is a blocker,
not permission to change it implicitly.

Subsequent slices require new manifests. A `READY_TO_IMPLEMENT` receipt for
Slice 1 never authorizes Slice 2 or deployment.

Current revised structural receipts:

```text
current observed correction route
  docs/structural_gates/receipts/
  LAY_IME_TARGET_AUTHORITY_CURRENT_OBSERVED_VETO_2026-08-17.json
  VETO, source markers 21/21, two execution paths

target event-route design
  docs/structural_gates/receipts/
  LAY_IME_TARGET_AUTHORITY_EVENTS_ROUTE_DESIGN_PASS_V4_2026-08-17.json
  PASS, 33 nodes, 51 edges, 12 routes, zero issues;
  exact event/command/gesture identity, exclusive rollback/manual plan and handoff

material/frame/context-neutral design
  docs/structural_gates/receipts/
  LAY_IME_TARGET_MATERIAL_FRAME_ROUTE_DESIGN_PASS_V6_2026-08-17.json
  PASS, 23 nodes, 44 edges, 7 routes, zero issues;
  preservation precedes every lexical/context certificate route

output transaction design
  docs/structural_gates/receipts/
  LAY_IME_TARGET_OUTPUT_TRANSACTION_ROUTE_DESIGN_PASS_V8_2026-08-17.json
  PASS, 49 nodes, 94 edges, 57 routes, zero issues;
  one durable protocol, three state-specific mutators, authority for every
  attempted outcome, ordered durability strategy, same-lineage barrier,
  startup recovery/key cleanup authorization and nonterminal quarantine/reset

durable diagnostic baseline
  docs/structural_gates/receipts/
  LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/manifest.json
  PARTIAL_DURABLE_BASELINE, 13 raw logs hash-verified

frozen event/Boundary definitions
  docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/
  event-and-boundary-cases-v7.json
  108 definitions = 49 executable baseline + 59 future contracts

Slice 1 revision-7 implementation preflight
  docs/structural_gates/receipts/
  LAY_IME_TARGET_AUTHORITY_SLICE1_IMPLEMENTATION_PREFLIGHT_V7_BLOCKED_2026-08-17.json
  BLOCKED_BEFORE_CODE, revised paper/routes/case manifest pinned;
  exactly two missing immutable-rerun artifacts
```

The design receipts prove only internal coherence in their declared scopes.
Their conjunction still does not authorize code while Slice 0 and the
slice-specific preflight remain open.

## 16. Rejected Shortcuts

- increase global top-k;
- use source score or source ID as authority;
- keep `surface_count > 1` as ambiguity;
- promote every grounded V90 surface;
- give Boundary permanent priority;
- run Boundary before Productive as a separate fast owner;
- require L3 for every lexical repair;
- use context to invent lexical grounding;
- cache a frame replacement span or frame rejection as lexical material;
- allow context or an online overlay to prune/birth targets before the complete
  lexical conflict cohort is formed;
- emit `ContextCertified` from an uncalibrated score maximum;
- treat a retained target prefix as complete after any upstream overflow;
- exclude a conflicting `Born` target and call the remaining grounded target a
  singleton;
- let score, producer arrival or provenance define cohort order;
- count package reload aliases of one derivation as independent witnesses;
- reject an entire target because one witness alias failed while another exact
  independent witness remains valid;
- dereference a field-local target/witness ID without a live bounded material
  lease or after cache generation reuse;
- count a deferred future-contract definition as an executed PASS;
- use the 74-target storage limit as an enumeration-work limit;
- retry the alternate Space plan after final authorization or output starts;
- compensate partial output before observing the exact full effect snapshot;
- call text-byte equality a no-effect proof while preedit, caret, layout, tail,
  feedback or synthetic-key state is changed or unknown;
- invoke a mutator after `RefusedBeforeMutation`;
- rely on in-memory recovery for a non-atomic effect across process death;
- invoke a no-effect/partial/recovery mutator without the same explicit
  authority route required for success;
- publish success or learning before the durable terminal journal record;
- implement two independent synchronous durability waits per steady-state event
  and postpone feasibility measurement until the final performance slice;
- allow native or Lay state change on a focus lineage while its prior terminal
  state is not durable;
- revoke or orphan a started transaction during runtime-owner handoff;
- execute manual layout outside the one double-Shift plan and common durable
  transaction;
- guess compensation from a corrupt journal or disable global IBus/the user's
  keyboard as a quarantine mechanism;
- call quarantine a terminal transaction result, clear it on focus loss, or
  re-enable Lay mutation without preserving the incident and establishing a
  new durable baseline;
- treat Alt press and matching release as two plan-producing commands or treat
  a multi-event double Shift gesture as one raw key event;
- let IBus and daemon mint plans for the same focus lineage;
- treat a consumed but missing Space as an acceptable failed denominator;
- promote lexical authority before the event/output transaction gate passes;
- compare monotonic deadlines across process/boot epochs;
- claim an authority-only slice repaired target birth or final `36/36`;
- restore deterministic correction after DecisionCore/verifier refusal;
- tune coefficients against the 18 failing examples;
- treat ignored display as negative morphology learning;
- call two-left-token context full-sentence understanding;
- raise memory or latency budgets after a failed implementation without a new
  consequences analysis;
- build release bytes from an uncommitted tree or claim that a later
  receipt-only commit was their source.

## 17. Slice 2 Deterministic Work-Budget Freeze, 2026-08-20

The proof-only measurement implementation counted one target-blind grounding
and cold-binding preparation plus one Productive traversal for every
lemma-heldout field. Oracle, exact baseline, clean, probe and seen-exact
executions were excluded from the work denominator. The measurement ran on the
remote 20-core build host against the exact active V90/V9/V13 package tuple and
the unchanged 1,300-entry frozen hypothesis manifest.

The fixed `13 x 1` smoke completed `13/13` measurement samples. The fixed
`13 x 100` proof then completed `1,300/1,300` unique lemma-heldout samples and
`2,600` total semantic comparisons. Aggregate reconstruction was exact, the
order-independent sample digest was
`31e49ea32a391580f2f9b5c56256a5aecf1b2a25a0d39a7072c228d713e62ea8`,
and the non-latency semantic report normalized to the same SHA-256 as the
accepted V90 baseline:
`905ce2d6ad7cb5c28e852fa0e603927feabb5e2afde031ab7826c8c51f256b4b`.

Measured per-field work was:

```text
producer / counter                 p50       p95       p99       maximum
canonical grounding lookups         43       133       207           229
cold binding posting visits      19,906    44,487    74,992        88,130
cold binding relation replays    13,527    51,441    57,385       103,763
cold binding operator steps      53,027   187,876   212,196       382,150
productive generated targets        953     6,147     6,263         6,348
productive relation replays         953     6,147     6,263         6,348
productive operator steps          1,216     7,752     8,629         9,406
aggregate operator steps          57,412   188,100   212,742       382,340
```

The normative `EnumerationWorkBudgetV1` ceilings for this exact package
generation are the smallest powers of two not below each fixed-proof maximum.
This gives deterministic headroom without making a sampled raw maximum the
runtime limit. A package, axis schema or frozen-manifest identity change
invalidates this budget and requires a new fixed proof; ceilings do not silently
carry across generations.

```text
producer                  posting   relation   grounding   generated   operator
canonical_grounding             0          0         256           0          0
cold_binding               131,072    131,072           0           0    524,288
productive_traversal             0      8,192           0       8,192     16,384
aggregate                  131,072    131,072         256       8,192    524,288
```

The aggregate ceiling is an independent whole-field limit, not the sum of
producer allowances. Every counter is checked before additional work. Crossing
either a producer or aggregate ceiling yields
`Overflow(WorkBudgetExceeded)` with the retained prefix and lower bound; it can
never yield `Complete`, `Winner`, `L2Certified` or `ContextCertified`.
Wall-clock deadlines remain separate operational gates and are not substitutes
for these counters.

Measured safety remained `H=1,280`, `H -> B=0`, `B -> S0=0`, false singleton
`0`, integrity errors `0`, and probe parity `26/26` in the smoke and
`2,600/2,600` in the full proof. The instrumented 20-worker proof used
`391,408 KiB` peak RSS and `19.47 s` wall time. Those timing values include
proof instrumentation and concurrent-host effects; they are measurement facts,
not a Productive hot-path latency PASS.

Tested: deterministic counters, producer/aggregate reconstruction, merge-order
digest, exact fixed denominator, semantic non-latency parity, frozen package
identity and measurement failure closure. Not tested: context-neutral material
membership, exact frame reuse, budget exhaustion propagation in the future
runtime, daemon/IBus latency, physical applications or automatic authority.
Runtime authority changed: `false`. Deployment actions: `0`.

Exact evidence:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_WORK_MEASUREMENT_2026-08-20/final-receipt.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_WORK_MEASUREMENT_2026-08-20/slice2-work-full-13x100.json
```

## 18. Current Verdict

```text
paper architecture                       SELECTED, revision 8
current source correction route          VETO: parallel Boundary producer
candidate validity                       Born | Grounded | Rejected
cohort verdict                           Winner | Tied | ABSTAIN
automatic authority                      L2Certified | ContextCertified
margin authority                         DISABLED
original preservation                    separate non-candidate contract
cached context authority                 FORBIDDEN
sentence-context claim                   NOT SUPPORTED; left two tokens only
runtime authority changed                false
revision-1 target route PASS              SUPERSEDED: scope incomplete
revision-1 implementation preflight       SUPERSEDED: too broad
revision-2 event-route gate                PASS: event-only scope, insufficient alone
revision-3 material/frame route gate       SUPERSEDED: context ordered before cohort
revision-4 material/frame route gate       PASS: design only, 23/43/7
revision-5 material/frame route gate       VETO: authority stage reversal and preservation bypass
revision-6 material/frame route gate       PASS: design only, 23/44/7
revision-3 event-route gate                VETO: role reversal and disconnected routes
revision-4 event-route gate                PASS: design only, 33/51/12
revision-4 output transaction gate         SUPERSEDED: zero/raw failure coverage incomplete
revision-5 output transaction gate         PASS: design only, 21/42/20
revision-6 output transaction gate         SUPERSEDED: PASS but journal/authority/manual scope incomplete
revision-7 output transaction gate         SUPERSEDED: PASS but durability latency/quarantine terminal scope incomplete
revision-8 output transaction gate         PASS: design only, 49/94/57
durable diagnostic baseline               PARTIAL: raw logs pinned
event/Boundary case definitions            FROZEN V7: 108 = 49 baseline + 59 future
immutable Slice 0 execution integrity      PASS: 97/97 valid, 0 invalid
immutable Slice 0 assertions               FAIL: 64/97 PASS; baseline 46/49
future source-absence proof                 PASS: 59/59 absent
Slice 0 baseline freeze                    PASS: immutable manifests complete
measured current-route latency             PASS: 7/7 implemented strata
complete EventRuntimeBudgetV1              FAIL_COVERAGE: four Slice 7 routes absent
external historical partial manifest       PRESENT: superseded, non-promotable
final Slice 0 repository results manifest  PRESENT: PASS_BASELINE_FREEZE_ONLY
Slice 1 implementation preflight V7        SUPERSEDED: placeholder manifest hashes
Slice 1 implementation preflight V8        PASS: exact immutable manifests pinned
Slice 1 evidence vocabulary                PASS: 13/13 preregistered contracts
Slice 1 semantic parity                    PASS: 49/49 exact normalized logs
Slice 1 runtime authority changed          false
deployment authorized                     false
context automatic authority               DISABLED PENDING SEPARATE PROOF
Slice 2 deterministic work budgets         PASS: frozen for exact V90/V9/V13 tuple
Slice 2 context-neutral material/frame      PASS SHADOW: 1,300/1,300 pairs
Slice 2 explicit upstream incompleteness    877/1,300 fail-closed
Slice 2 exact frame bindings                3,864/3,864, stale accepts 0
Slice 2 runtime authority changed           false
Slice 3 candidate-state derivations         3,864/3,864
Slice 3 false grounding                     0
Slice 3 stale/cross-context accepts          0/0
Slice 3 original preservation               3,900/3,900 outside target set
Slice 3 runtime authority changed           false
Slice 7A buffered ext4 durability strategy  REJECTED: prepare max 325.387 ms
Slice 7A direct aligned-slot strategy        REJECTED: prepare/co-commit max 330.324/162.240 ms
Slice 9 numeric context risk policy         UNFROZEN BLOCKER FOR SLICE 9
next gate                                 BackendAtomicReceiptV1 design and kill-point proof
```

Slice 1 changed source only in the isolated implementation worktree. No package,
installed runtime or live authority was changed by the paper or vocabulary
slice.

## 19. Slice 2 Context-Neutral Material and Exact Frame Result, 2026-08-20

Slice 2 now separates reusable lexical material from per-input frame state.
Productive enumeration runs without context shaping under the frozen producer
and aggregate work budgets. The result is canonicalized into deterministic
bounded prepared material before any source window, caret, selection, preedit,
case or punctuation projection is attached. A frame can consume that material
only through an exact UTF-8 identity and a live bounded lease.

```text
context-neutral enumeration
-> deterministic prepared material, at most 74 targets
-> exact material digest
-> bounded material lease, at most 32 fields x 8 consumers
-> exact UTF-8 input frame
-> frame identity and material digest validation
-> shadow replay only
```

The remote fixed proof evaluated `13 x 100` lemma-heldout fields. Every field
produced one unique material pair and respected the frozen work budget. Three
context frames were compared per pair; frames without a bindable target were
excluded by the same explicit material completeness state rather than silently
converted to a smaller authority cohort.

```text
evaluated semantic comparisons             2,600
material pairs                         1,300/1,300
unique material pairs                  1,300/1,300
H target membership                    1,280/1,280
work budgets respected                 1,300/1,300
context comparisons                           3,900
bindable frame targets                 3,864/3,864
stale reuse accepts                    0 / 3,864
digest failures / failed material            0 / 0
false singleton / integrity errors           0 / 0
H / B / S0                       1,280 / 1,280 / 1,280
semantic non-latency gate                     PASS
```

Material completeness is not overstated. Only `423/1,300` fields are
`Complete`; `877/1,300` are explicitly `UPSTREAM_INCOMPLETE`. Those 877 fields
must remain ineligible for `Winner`, `L2Certified` and `ContextCertified` in
all later slices. The result proves exact preservation and framing of the
material available from the current upstream producers; it does not prove that
the upstream enumeration is complete for those fields.

The proof-only run used 20 workers, `20.72 s` wall time and `392,048 KiB` peak
RSS. Concurrent instrumentation raised maximum class p99 to `19.258 ms`, above
the `5 ms` product gate. Therefore the historical aggregate proof verdict is
`FAIL_measured_shadow_gates`; this is not relabelled as a latency PASS. The
separately computed semantic non-latency gate is `PASS` and is the only quality
claim made by this slice.

Tested: context-neutral enumeration, deterministic material identity, exact
frame identity/replay, frozen budgets, bounded leases, stale-frame rejection,
fixed denominator, semantic non-latency parity and fail-closed incompleteness.

Not tested: live candidate-state ownership, L3/L4/DecisionCore/verifier
authority transfer, queue-inclusive daemon/IBus latency, physical applications,
automatic mutation, package rebuilding or deployment.

Runtime authority changed: `false`. Deployment actions: `0`. Installed version
remains `1.0.33`. Slice 3 cannot reinterpret `UPSTREAM_INCOMPLETE` as complete.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_MATERIAL_FRAME_2026-08-20/final-receipt.json`

## 20. Slice 3 Frame-Bound Candidate Validity Result, 2026-08-20

Slice 3 implements candidate validity only after the Slice 2 material lease and
exact input frame have both been revalidated. Candidate validity has no context,
score, rank, display, admission or mutation input.

```text
prepared target material + live lease + exact input frame
-> exact projected target and replacement-span replay
-> witness-local geometry assessment
-> Born | Grounded | Rejected(reason)
-> absolute authority blockers
```

Witness-local rejection cannot erase an independent grounded witness. A target
can become `Rejected` only when its target-grounding namespace is complete. An
incomplete target namespace remains `Born`; a stale frame aborts settlement
instead of manufacturing a target rejection.

Field completeness and target validity are separate dimensions. The existing
`877/1,300 UPSTREAM_INCOMPLETE` materials retain an absolute authority blocker.
A retained target inside such a field may still be `Grounded` when its own exact
grounding witness is complete, but it cannot issue automatic authority while the
field-level blocker survives.

The original surface is classified from prepared lexical, script-token and
punctuation state. Its `Preserve | ReplacePermitted | Unresolved` verdict is
frame-bound and stored outside the replacement target set.

```text
fixed cases                                  13 x 100
evaluated semantic comparisons                  2,600
candidate-state derivations                3,864/3,864
Born / Grounded / Rejected                0 / 3,864 / 0
false grounding                                  0
cross-context mismatches                          0
stale candidate-state accepts                    0
original Preserve / Replace / Unresolved  48 / 1,080 / 2,772
original-preservation total                3,900/3,900
H / B / S0                          1,280 / 1,280 / 1,280
probe parity                              2,600/2,600
false singleton / integrity errors              0 / 0
```

The remote run used 20 workers, `19.77 s` wall time, `674%` CPU and
`392,048 KiB` peak RSS. The historical aggregate report remains
`FAIL_measured_shadow_gates` because Productive traversal reaches an
instrumented maximum class p99 of `16.181 ms`, above `5 ms`. Slice 3 does not
rename that result or claim live latency. Its scoped candidate-state,
completeness, frame, safety and parity contract passes.

Tested: state truth table, witness-local failure, incomplete target namespace,
field-level authority blockers, exact frame/lease failure, original separation,
fixed candidate-state counters and source-neutral parity.

Not tested: conflict-cohort settlement, live authority transfer, queue-inclusive
daemon/IBus latency, physical applications, automatic mutation, package rebuild
or deployment. Runtime authority changed: `false`; deployment actions: `0`;
installed version remains `1.0.33`.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE3_CANDIDATE_STATE_2026-08-20/final-receipt.json`

The next permitted change is Slice 4 conflict-cohort and
`Winner | Tied | ABSTAIN` shadow settlement under a new implementation
preflight.

## 21. Slice 4 Complete Conflict-Cohort Result, 2026-08-20

Slice 4 forms one context-neutral cohort from every retained target after exact
frame binding and candidate-state settlement. Exact duplicate outputs merge by
semantic roots before `CanonicalCohortOrderV1`; score, rank, producer arrival,
source ID and context are absent from ordering and membership.

```text
all frame-bound CandidateState values
-> exact EditFootprintV1
-> duplicate-output semantic-root merge
-> conflict components
-> original-preservation veto
-> Winner | Tied | ABSTAIN shadow verdict
```

The fixed remote `13 x 100` proof produced:

```text
cohort derivations                          3,900 / 3,900
Winner / Tied / ABSTAIN                     0 / 1,050 / 2,850
cross-context cohort/hash mismatches                         0
incomplete Winner / false singleton                          0 / 0
lost grounded target                                         0
multiple-component authority                                 0
original-preservation bypass                                 0
complete / upstream-incomplete material                423 / 877
H / B / S0                              1,280 / 1,280 / 1,280
semantic non-latency gate                                  PASS
```

The zero-Winner distribution is not claimed as useful automatic coverage. The
fixed corpus proves fail-closed settlement and target retention; the Winner
truth table is covered by focused unit tests. Missing target birth remains the
next independent problem.

The run used 20 workers, `19.52 s` wall time, `678%` CPU and `392,416 KiB`
peak RSS. The historical aggregate remains `FAIL_measured_shadow_gates` because
maximum class p99 is `14.566 ms > 5 ms`; latency promotion is not claimed.
Runtime authority, packages, daemon, IBus and installed version `1.0.33` were
unchanged.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE4_CONFLICT_COHORT_2026-08-20/final-receipt.json`

Next gate: Slice 5 bounded missing-target birth and retention shadow.

## 22. Slice 6 Boundary Internalization Result, 2026-08-20

Slice 6 moves exact separator birth into the same context-neutral material field
as Productive and contour birth, without changing runtime authority:

```text
exact input frame
-> bounded exact BoundarySplit | BoundaryMerge enumeration
-> two-sided package grounding
-> CompositeBoundaryGroundingV1
-> exact separator-only geometry
-> separate two-surface boundary reserve
-> productive/contour/boundary exact-surface dedup
-> Complete | Overflow(StorageCapacity)
-> Born-only boundary witnesses
```

The implementation does not infer words from literals in proof fixtures.
Split, merge, false-split and real multi-split-overflow cases are selected from
installed package identities. The old deterministic boundary producer is
observed only for coverage comparison. Its non-match is explicitly not a
promotion gate because Slice 6 is intended to internalize package-derived
coverage that the old route does not own.

The final remote `13x100` run on `e@192.168.3.94` used 20 workers and produced:

```text
boundary shadow verdict                    PASS_SLICE6_BOUNDARY_SHADOW
H / B / S0                                 1,280 / 1,280 / 1,280
target retention                           1,280 / 1,280
false singleton / integrity errors         0 / 0
boundary authority grants                  0
fixed contour birth / retention            8 / 8
wall time / CPU                            34.21 s / 1026%
peak RSS                                   650,852 KiB
maximum class p99                          11.238 ms
```

The scoped Boundary result passes. The whole receipt remains
`FAIL_measured_shadow_gates` because the pre-existing absolute latency contract
requires `<=5 ms`. Therefore Slice 6 is not promotion-eligible and nothing was
deployed. Runtime authority, package bytes, daemon, IBus and installed version
`1.0.33` remain unchanged.

The proof command requires all four explicit environment switches:
`LAY_PRODUCTIVE_WORK_MEASUREMENT=1`,
`LAY_PRODUCTIVE_MATERIAL_FRAME_PROOF=1`,
`LAY_PRODUCTIVE_CONTOUR_BIRTH_PROOF=1` and
`LAY_PRODUCTIVE_BOUNDARY_PROOF=1`. Omitting them executes the base proof but
does not execute the four shadow sections; that run is not Slice 6 evidence.

Tested: package-derived split and merge, two-sided grounding, exact geometry,
false split, multi-split overflow, boundary reserve, cross-producer dedup,
Born-only state, bounded work, target retention and source-neutral observation.

Not tested: live authority transfer, event transaction durability, automatic
mutation, queue-inclusive product latency, physical applications or deployment.

Exact receipt directory:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE6_BOUNDARY_INTERNALIZATION_2026-08-20`

Next gate: Slice 7A isolated crash-safe durability-strategy microproof. The
`<=5 ms` latency blocker remains conjunctive and cannot be waived by the Slice 6
functional PASS.

## 23. Slice 7A Buffered ext4 Durability Result, 2026-08-20

Slice 7A implemented and measured one concrete `OrderedGroupCommitV1` storage
primitive in isolation. It used a single bounded owner, a queue of 32, fixed
256-byte checksum-chained records, an 8 MiB file allocated and synchronized at
cold startup, fixed-offset `write_at`, and `fdatasync` durability barriers. The
test did not integrate this owner into IBus, the daemon or any live mutation
route.

The first `/tmp` run is invalid because `/tmp` is `tmpfs`. Two unchanged ext4
runs then reproduced a prepare maximum of `159.384 ms` and `166.626 ms`.
Removing hot file growth did not remove the mechanism. The final preallocated
ext4 run measured:

```text
filesystem / device                    ext4 / Intel NVMe SSDPEMKF010T8
samples / warmup                                      1,000 / 64
prepare p50 / p95 / p99                    0.345 / 0.818 / 1.265 ms
prepare maximum                                      325.387 ms  FAIL
co-commit p50 / p95 / p99                 0.341 / 0.814 / 1.164 ms
co-commit maximum                                     1.677 ms  PASS
tail-flush p99 / maximum                      0.775 / 0.865 ms
next-native p99 / maximum                     0.748 / 0.905 ms
prepare records / sync calls                       2,128 / 2,128
co-commit records / sync calls                     2,130 / 1,066
fault matrix                                                  PASS
focused output-transaction tests                              7/7
wall / peak RSS                                      4.45 s / 11,128 KiB
```

Measured gate: p99 `<=2 ms` passes for prepare and co-commit, but the strict
maximum `<8 ms` fails for prepare. The failure is not file growth, queueing,
record encoding or checksum work: the outlier remains inside the synchronous
durability barrier after capacity allocation was moved out of the hot path.
Therefore buffered ext4 `write_at + fdatasync` is rejected as the Slice 7
durability primitive. Repeating this run, changing a threshold or hiding the
outlier in an average is forbidden.

Not tested: direct I/O, a backend atomic receipt, live kill points, integrated
event latency, physical applications, recovery UI or deployment. Runtime
authority changed: `false`; installed runtime touched: `false`; installed
version remains `1.0.33`.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE7A_DURABILITY_MICROPROOF_2026-08-20/final-ext4-preallocated-receipt.json`

### Next storage primitive

The next and only admitted storage microproof is
`DirectAlignedSlotCommitV1`, still under the unchanged
`OrderedGroupCommitV1` transaction semantics:

```text
cold startup
-> fixed-capacity file + parent-directory durability
-> aligned 4 KiB slot pool and aligned buffers
-> O_DIRECT | O_DSYNC, or fail closed when unsupported

hot prepare/co-commit
-> pack one or two logical 256-byte chained records into one slot
-> one fixed-offset direct synchronous write
-> publish durability only after full-slot completion
-> no buffered page-cache write and no separate fdatasync
```

The ring retains sequence, generation, previous digest and checksum so startup
can select the highest complete contiguous chain and reject a torn, foreign or
ambiguous slot. Wrap, saturation, partial direct write, unsupported alignment,
checksum damage and sync failure are explicit fail-closed strata. Co-commit
still persists `terminal(N) + Prepared(N+1)` in one durability unit; direct I/O
does not weaken ordering or permit effect-before-prepare.

This option is selected because it attacks the measured buffered
page-cache/filesystem-sync tail. `io_uring` around the same write-plus-fsync,
SQLite/WAL, `mmap + msync`, another buffered `RWF_DSYNC` variant and tmpfs do not
remove that mechanism or do not provide crash durability. Direct I/O may still
fail because the NVMe device itself has unbounded tails; the same p99 `<=2 ms`
and maximum `<8 ms` gate decides that with one isolated proof. A failure rejects
local synchronous storage for this contract and returns architecture work to
`BackendAtomicReceiptV1`; it does not authorize more storage tuning.

## 24. Slice 7A Direct aligned-slot Durability Result, 2026-08-20

The one admitted `DirectAlignedSlotCommitV1` implementation replaced buffered
`write_at + fdatasync` with fixed 4 KiB aligned slots, an 8 MiB / 2,048-slot
ring, and one full `O_DIRECT | O_DSYNC` write per durability unit. Terminal N
and Prepared N+1 share one direct slot. Slot generation, sequence, previous
digest, checksum and the logical-record checksum chain are verified during
recovery. There is no buffered fallback.

The SHA-matched remote metrics binary passed eight focused tests. Its executable
fault matrix rejected checksum damage, torn slots and foreign generations,
preserved the latest complete chain across ring wrap, refused saturation and
kept prepare/terminal failures fail-closed. The one final ext4 proof measured:

```text
host / filesystem                                      t480 / ext4
samples / warmup                                      1,000 / 64
prepare p50 / p95 / p99                 0.702 / 1.289 / 1.657 ms
prepare maximum                                     330.324 ms  FAIL
co-commit p50 / p95 / p99              0.694 / 1.228 / 1.575 ms
co-commit maximum                                  162.240 ms  FAIL
tail-flush p99 / maximum                    1.483 / 154.736 ms
next-native p99 / maximum                    1.593 / 14.571 ms
physical write unit                                     4,096 B
fault matrix                                                   PASS
focused output-transaction tests                              8/8
wall / peak RSS                                      7.61 s / 11,940 KiB
```

Both prepare and co-commit satisfy p99 `<=2 ms`, but both violate the unchanged
maximum `<8 ms` gate. The outliers therefore survive removal of buffered page
cache writes, separate `fdatasync` and hot inode growth. Local synchronous
storage cannot provide the frozen tail bound on this machine. Repeating the
proof, relaxing the gate, hiding maxima in a percentile or trying a third local
storage mechanism is forbidden by the preregistered decision.

Verdict: `REJECTED_LOCAL_SYNCHRONOUS_STORAGE_FOR_SLICE7`. Slice 7 returns to
`BackendAtomicReceiptV1`, which requires a new design receipt and kill-point
proof for complete-effect-vector atomicity, exact event disposition identity,
and post-crash query or idempotent replay. Slice 7B integration is not admitted.

Not tested: an accepted backend atomic primitive, integrated event latency,
live crash/recovery, physical applications, recovery UI or deployment. Runtime
authority changed: `false`; installed runtime touched: `false`; installed
version remains `1.0.33`.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE7A_DURABILITY_MICROPROOF_2026-08-20/final-direct-aligned-slot-receipt.json`

Exact receipt SHA-256:

`f9595b67a6c1d36c20d94601f8d580e044cd6cff787f732d554cf004aa67ec3b`

## 25. Backend Atomic Receipt Design Return, 2026-08-20

After both local synchronous-storage strategies failed the unchanged maximum
gate, Slice 7 returned to paper review as preregistered. The exact installed
`ibus 1.5.34~rc2-1` source contains a synchronous post-process queue that can be
turned into an independent complete-frame owner without adding another local
storage mechanism.

Current IBus is not yet sufficient. It captures CommitText,
DeleteSurroundingText, preedit and forwarded-key operations while
`ProcessKeyEvent` is active, but engine failure does not explicitly discard a
partial frame, overflow is not a whole-frame refusal, and the client
post-process API cannot bind fetch success to the final handled disposition.
Separate asynchronous signals therefore retain strict-prefix kill points.

The selected design candidate is `IbusSynchronousPostProcessReceiptV1`:

```text
exact event + authorized edit + per-client capability
-> ibus-daemon collects one bounded frame
-> engine success seals; engine error/overflow discards
-> client takes one complete identity-bound frame
-> one client batch owner applies or refuses with zero effect
-> handled and learning only follow the backend receipt
```

Unsupported clients and mutation callbacks outside `ProcessKeyEvent` are
native-only. Cursor-key, terminal-control and layout vectors remain outside the
first proof. Each GTK, Qt, GNOME Wayland, terminal and Electron/Chromium profile
requires an independent capability and physical receipt.

The first machine route packet was retained with `VETO` for three relation-role
direction errors. Corrected V2 passed the design-only route gate with no issues
or warnings and one mutation owner. `safe_to_edit` remains false; source
behavior, kill points, client atomicity and runtime latency remain unproved.

Owning design:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/ime-backend-atomic-receipt-v1-2026-08-20.md`
