# TD-113: Restore Hybrid Nanda Autocorrect

Status: DONE
Priority: P0
Stage: 1
Class: user-visible candidate coverage and correction authority
Size: M
Base commit: 1158b02df10ca433117b514a2baa0fa3ac6efa5d
Admission: explicit user report and authorization on 2026-09-02

## Outcome Required

When nanda_autocorrect is enabled, Nanda must augment the ordinary
deterministic correction source instead of replacing it. The live CLI, daemon,
and IME Space routes must build one bounded candidate lattice from both source
families and then use the existing single DecisionCore, verifier, and
authorization owner. Each live event must build exactly one edit plan and
dispatch it to exactly one already-selected mutation backend: IME for an
IME-owned event or daemon output for a daemon-owned event.

Pure Nanda-only behavior remains available for isolated diagnostics, comparison
tools, and evaluator code. It must no longer be selected merely because the
user enabled nanda_autocorrect in the live configuration.

## User-Visible Problem

The installed 1.0.61 configuration has:

- nanda_autocorrect=true;
- correction_safety=experimental;
- auto_replace=true.

Despite the broadest correction profile, ordinary errors frequently remain
unchanged. Reproduced classes include:

| Input | Deterministic candidate | Current Nanda-configured result |
|---|---|---|
| плозо | плохо | no replacement |
| обьяснить | объяснить | no candidate |
| верменно | временно | no candidate |
| текст е | тексте | no candidate |
| т ыпочитай | ты почитай | no candidate |
| протколах | протоколах | competing Nanda surface can select пр отколах |

These strings are regression fixtures only. Runtime code must never branch on a
word, phrase, suffix, fixture ID, test name, or source-ID spelling.

## Read-Only Root Cause

The first failure is a source-selection defect:

    nanda_autocorrect=true
    -> live adapters choose CorrectionMode::NandaOnly
    -> L2CandidateSource::for_mode returns only Nanda
    -> deterministic layout, typo, and boundary candidates never enter the lattice
    -> DecisionCore cannot rank or authorize evidence it never received

Measured source locations at the base commit:

1. src/correction_core.rs defines only DeterministicOnly and NandaOnly.
2. src/correction_core/candidate_sources.rs maps those modes to mutually
   exclusive one-element source arrays.
3. src/ime_correction.rs maps live nanda_autocorrect=true to NandaOnly.
4. src/bin/lay_daemon/typing_assist_runtime/decoder/gate.rs does the same for
   the daemon Space route.
5. src/main.rs does the same in the configuration-aware correction-core
   explanation route.
6. proposal_only_substitution_competitor is retained as SuggestOnly on the
   Nanda-only route, but SuggestOnly is not automatic Apply authority.
7. TD-112 tested плозо through DeterministicOnly. It proved correction-safety
   authority but did not prove that the live Nanda-enabled mode retained the
   deterministic producer.

TD-112 is not reverted. Its end-to-end correction-safety policy remains the
only profile-aware Apply policy after all candidates are retained.

### Root-cause refinement after the compiling red test

The initial hybrid composition repairs candidate birth but does not by itself
restore Apply. A paired route matrix over the same `плозо -> плохо` event found
the next two shared authority defects:

| Mode / route | Measured candidate state | First authority loss |
|---|---|---|
| DeterministicOnly / FullWave | deterministic `плохо` is Eligible and Experimental applies it | none; this is the compatibility baseline |
| DeterministicAndNanda / CanonicalL2Field | Nanda returns zero candidates; deterministic `плохо` remains in the lattice but becomes SuggestOnly | canonical `Abstain` is applied globally to an independently grounded deterministic candidate |
| DeterministicAndNanda / FullWave | Nanda returns the same `плохо` surface plus its bounded alternatives; the merged target has Eligible deterministic and L2Surface evidence and a certified operator-consensus witness | the merged primary L2Surface owner is rejected later as `short_same_length_surface_drift` even though the same verified deterministic operator independently reconstructed the surface |

Exact debug evidence for the canonical route was `candidate-lattice
source=nanda count=0`, followed by `source_id=single_letter_substitution`,
`gate=SuggestOnly`, and no selection. The FullWave route retained 21 raw Nanda
proposals before same-surface merge; the target reached
`operator_consensus=true`, then was rejected specifically as
`short_same_length_surface_drift`. The deterministic-only TD-112 route remained
green and selected the same target under Experimental.

The shared mechanism is therefore **negative authority leakage from an
augmenting source**:

1. absence or abstention in Nanda is treated as contradictory evidence against
   an independently Eligible deterministic repair;
2. agreement between Nanda and deterministic evidence is recognized strongly
   enough to obtain a certified operator-consensus witness, but one later
   surface-drift guard ignores that certificate.

`Abstain` and `Unavailable` are not contradictory evidence. They may continue
to withhold authority from Nanda-owned surfaces, but must not lower an already
Eligible deterministic alias. A different canonical Winner, an exact negative
transition, verifier failure, protected-token veto, edit-safety failure, or
profile insufficiency remains contradictory or blocking evidence and is not
relaxed.

### Refined options and scores

| Option | Score | Assessment |
|---|---:|---|
| Preserve independently Eligible deterministic evidence across only canonical Abstain/Unavailable, and let an already certified same-surface operator consensus satisfy the later surface-drift guard | **9/10** | Selected. Repairs both measured loss layers in their existing owners, keeps one lattice/DecisionCore/verifier path, and leaves explicit contradictory evidence and all safety gates intact. |
| Keep primary deterministic ownership whenever a Nanda alias merges into the same surface | 7/10 | Likely repairs the FullWave symptom, but changes the established source-order-independent ownership contract and still needs a separate canonical-Abstain repair. |
| Run deterministic correction as a fallback after hybrid returns no Apply | 4/10 | Creates a second resolution and ranking pass, cannot reason about competing Nanda surfaces in one lattice, and violates the single-authority contract. |
| Switch the live route from CanonicalL2Field to FullWave | 3/10 | FullWave reproduces a later rejection, adds a much larger candidate frontier, and changes the accepted live route rather than fixing authority transfer. |
| Disable canonical demotion or surface-drift rejection globally | 2/10 | Over-broad safety regression; it would also admit unsupported Nanda surfaces and removes useful contradictory-evidence handling. |
| Add exceptions for the reported words | 1/10 | Forbidden fixture-specific runtime behavior and does not repair the shared mechanism. |

Selected refined direction: the scoped two-part authority-transfer repair,
9/10. It may preserve authority but may never mint it: a deterministic alias
that was already SuggestOnly stays SuggestOnly, an Nanda-only candidate still
obeys canonical authority, and operator consensus must pass the existing
verifier plus hidden-state certificate before it can affect the drift guard.

### Cross-class refinement after the complete six-case red test

The first green authority-transfer tests repaired `плозо`, but the required
cross-class matrix exposed three pre-existing losses that the live hybrid route
now reaches instead of hiding:

- `обьяснить -> объяснить`: the dedicated deterministic rule produces the
  correct one-character repair, but the broad L2 surface foundation also
  contains the damaged spelling. Proposal admission treats that coverage-only
  membership as a clean-surface certificate and lowers the repair to
  `SuggestOnly` with `known_current_word_surface_drift`.
- `текст е -> тексте`: the deterministic boundary producer emits an Eligible,
  verifier-passed `BoundaryMergeSplit`, but the later latent-state guard treats
  the final one-letter fragment `е` as a complete known lexical state and
  rejects the structural transition as `latent_known_word_drift_needs_state_proof`.
- `т ыпочитай -> ты почитай`: the verified boundary-shift candidate preserves
  all surface mass, but a competing `extra_letters` candidate deletes `ы` and
  wins by 0.064 rank units. The existing structural-dominance path does not
  recognize the one-character loss because its generic 120-milli threshold is
  larger than this case's 111-milli normalized loss.

These are two shared mechanisms, not three word exceptions:

1. **coverage/authority conflation** — broad lattice membership is incorrectly
   used as proof that an input surface is clean;
2. **structural proof loss** — a verifier-passed, mass-preserving boundary
   transition is later evaluated as an ordinary single-token lexical drift or
   an ordinary score competitor.

| Follow-up option | Score | Assessment |
|---|---:|---|
| Use the existing clean-surface certificate, rather than broad L2 coverage, for unknown-to-clean one-edit admission; exempt only verifier-passed boundary transitions from the later lexical-drift interpretation; and let a zero-loss verified structural competitor dominate a close lossy candidate | **9/10** | Selected. Reuses typed evidence already present, changes no producer or verifier, and preserves clean/user/technical protection plus exact negatives. |
| Remove the damaged spelling from generated lexical packages | 5/10 | May repair one corpus snapshot, but requires package regeneration and does not fix the boundary authority losses or future noisy coverage entries. |
| Lower global structural thresholds or increase the boundary rank bonus | 3/10 | Tunes unrelated candidates and makes the result calibration-dependent instead of consuming the existing structural proof. |
| Disable known-word and competitor vetoes globally | 2/10 | Creates broad false-accept risk and weakens protections that existing log regressions require. |
| Add exceptions for the three reported strings or source IDs | 1/10 | Forbidden case-specific runtime behavior and does not repair the shared mechanisms. |

The selected extension remains fail-closed: an original surface with the
existing clean/user/live/technical certificate stays protected; a replacement
must have that clean certificate; the edit must already have a typed one-step
shape; boundary relief requires the existing transition verifier; and
structural dominance applies only to a verified zero-loss structural candidate
inside the existing close-rank and overlap checks. Implementation preflight V4
binds this scope before the additional runtime edits.

Preflight V2 was retained as `BLOCKED_BEFORE_CODE` before further source edits.
It exposed two contract-coverage omissions, not a runtime failure: the
`candidate_owner_rewrite` veto lacked a static source tripwire, and the first
mutating test-edit transition referenced a parity check instead of an explicit
fault-injection check. V3 adds those two fail-closed bindings without weakening
any invariant or widening implementation scope.

## Aggregate Log Audit

The repository receives only aggregate counts. Raw user text stays in the
private local audit directory and is never committed.

Historical correction log:

- 2,987 valid records;
- 989 user-correction events;
- accepted user-fix window: 2026-05-02 through 2026-08-21;
- 683 reconstructable accepted fixes;
- 983 rejected Lay outputs;
- accepted operation classes:
  - surface typo: 262;
  - single-character edit: 256;
  - boundary: 58;
  - mixed layout: 52;
  - layout: 33;
  - transposition: 22.

The corrections log file stopped changing on 2026-08-23 while current runtime
activity continued through 2026-09-02. This means the historical denominator
is real but incomplete for current typing. Repairing that logging gap is a
separate Stage 2 task and is not allowed to expand the immediate authority fix.

Current IME trace window:

- 2,841 valid trace records;
- 42 Space-autocorrect decisions;
- 37 full_no_apply/rank;
- 4 full_no_apply/infrastructure;
- 1 not_authorized;
- 0 applied;
- 58 candidate-quality observations: 27 apply and 31 non-apply;
- applied source mix in those observations: deterministic 25, Nanda 2.

Candidate presence or preedit completion availability is not a quality proof.
The audit separates candidate availability, selection, Apply authorization,
and visible mutation.

Aggregate evidence:
tech_debt/evidence/td113-log-audit-v1.json.

## Options And Scores

The score is fitness for this defect now.

| Option | Score | Assessment |
|---|---:|---|
| Add one explicit DeterministicAndNanda mode and select it only for live nanda_autocorrect=true | 9.5/10 | Selected. Restores lost coverage, keeps one lattice and one authority path, preserves NandaOnly diagnostics, and changes only source composition plus three live mappings. |
| Make every correction request always run both sources and remove pure modes | 6/10 | Restores coverage but destroys useful isolation for evaluators, makes diagnostics ambiguous, and broadens the change unnecessarily. |
| Run Nanda first and fall back to deterministic only when Nanda returns no selected candidate | 5/10 | Helps empty results but creates a second sequential decision pass, misses competing-candidate failures such as протколах, and duplicates work/authority reasoning. |
| Disable Nanda in the user configuration | 3/10 | A reversible workaround, not a repair. It discards useful Nanda evidence and contradicts the enabled feature. |
| Add word-specific substitutions or tune individual Nanda thresholds | 1/10 | Forbidden case patching. It cannot repair missing deterministic layout/boundary classes and would accumulate legacy exceptions. |

Selected direction: explicit hybrid live mode, 9.5/10.

## Minimal Design Contract

### Typed source mode

Add exactly one CorrectionMode variant:

    DeterministicAndNanda

Its source sequence is exactly:

    [Deterministic, Nanda]

The order controls deterministic collection only; it does not preselect the
winner. Existing lattice merge, scoring, DecisionCore ranking, verifier, and
TD-112 authorization policy decide the result.

Add one shared mapping helper owned by correction_core:

    nanda_autocorrect=false -> DeterministicOnly
    nanda_autocorrect=true  -> DeterministicAndNanda

The live IME, daemon, and configuration-aware CLI route consume that helper.
They must not keep private boolean-to-mode interpretations.

### Preserve isolated modes

- DeterministicOnly remains available for deterministic unit/proof lanes.
- NandaOnly remains available for Nanda evaluator, route comparison,
  explanation, and isolated research lanes.
- Explicit Nanda diagnostics must not silently become hybrid.

### One candidate lattice

Both source families extend the same L2CandidateLattice during one request.
There is no Nanda-first fallback request and no second resolution pass.

When a replacement surface is proposed by both families, existing alias merge
semantics retain both evidence identities. Raw alias count is not new authority;
TD-112's independent-domain policy remains controlling.

The existing proposal-only substitution competitor is present whenever the
selected mode includes Nanda. It remains SuggestOnly unless existing
DecisionCore evidence upgrades the merged surface. The fix must not globally
admit SuggestOnly candidates.

The existing short-Cyrillic layout suggestion remains present whenever the
selected mode includes Deterministic.

### One authority and mutation path

The two mutually exclusive live events remain:

    IME-owned Space input
    -> deterministic producer + Nanda producer
    -> one L2CandidateLattice
    -> one signal evaluation
    -> one transition verifier
    -> one DecisionCore rank
    -> one TD-112 Apply authorization
    -> one edit plan
    -> IME mutation backend

    daemon-owned Space input
    -> deterministic producer + Nanda producer
    -> one L2CandidateLattice
    -> one signal evaluation
    -> one transition verifier
    -> one DecisionCore rank
    -> one TD-112 Apply authorization
    -> one edit plan
    -> daemon mutation backend

IME and daemon mutation backends are distinct nodes but cannot execute in the
same named event. The source-composition change must not edit either backend,
the edit-plan implementation, the Space prefetch/wait owner, or the 3.5 ms
Space wait budget.

Forbidden:

- second ranker, verifier, authorization path, edit planner, or mutation owner;
- source-local automatic Apply overrides;
- admitting every SuggestOnly candidate;
- deleting a grounded candidate because another source exists;
- weakening SafetyGate, transition verification, edit-plan validation, or
  correction-safety policy;
- word- or fixture-specific runtime conditions.

## TDD Contract

Tests must be red before runtime implementation and green afterward.

### Mode/source contract

1. DeterministicOnly yields exactly the deterministic source.
2. NandaOnly yields exactly the Nanda source.
3. DeterministicAndNanda yields both sources exactly once in deterministic,
   documented order.
4. Shared live mapping returns DeterministicOnly when disabled and
   DeterministicAndNanda when enabled.
5. No live config mapping returns NandaOnly.

### Candidate and selection matrix

For each selected regression case, tests inspect:

- retained replacement surfaces;
- typed source/alias evidence where both producers agree;
- selected replacement;
- forbidden competing replacement;
- gate action and no-apply reason when the active profile intentionally
  withholds Apply.

Required Experimental-profile expectations:

| Input | Required selected replacement | Forbidden selected replacement |
|---|---|---|
| плозо | плохо | none other |
| обьяснить | объяснить | none other |
| верменно | временно | none other |
| текст е | тексте | none other |
| т ыпочитай | ты почитай | none other |
| протколах | протоколах | пр отколах |

Strict and Normal tests prove TD-112 profile semantics still apply to the
hybrid lattice. A candidate may remain retained without automatic Apply; the
test must not manufacture corroboration merely to force the same answer in all
profiles.

### Negative matrix

Hybrid mode must introduce zero Apply decisions for the existing clean and
protected regression fixtures, including:

- clean known Russian forms;
- protected multiword tails;
- technical tokens and URLs;
- known-word-to-neighbor drift;
- completed forms that Nanda might extend to an infinitive;
- exact-layout and verifier-negative cases.

### Route parity

Tests cover:

- active-composition IME Space request with nanda_autocorrect=true;
- daemon word-boundary Space mapping;
- configuration-aware CLI correction-core mapping;
- nanda_autocorrect=false parity;
- pure Nanda diagnostic parity;
- closed exact route preservation.

### Compiling red baseline

Before adding the new enum variant, add and execute at least one compiling
behavioral test against an existing live/public surface. It must fail because
the current live nanda_autocorrect mapping selects NandaOnly and loses a
required deterministic correction. A compile error from referencing the future
variant is not accepted as the red baseline.

After that behavioral failure is recorded, add the typed enum/source contract
and make the complete matrix green.

### Executable implementation-scope gate

The green suite must include a source-contract test over every touched runtime
adapter. It must:

- inspect only runtime sections, excluding cfg(test) bodies;
- reject all TD-113 fixture strings in runtime sections;
- require IME, daemon, and configuration-aware CLI live mappings to call the
  shared correction-core mapping;
- reject NandaOnly inside those live mapping bodies while allowing it in
  explicit diagnostic/evaluator bodies;
- reject added resolution/authorization calls and any edited mutation backend
  or Space-prefetch file;
- bind the protected backend and prefetch files by preflight SHA-256;
- rerun the observed-source route gate with separate IME and daemon events.

This source-contract test, protected-artifact parity, and the observed route
receipt jointly enforce the no-hack/no-second-path claim. Invented marker names
alone are not sufficient.

## Consequence Analysis

### Candidate quality

Expected benefit: deterministic typo/layout/boundary candidates are no longer
lost, and Nanda candidates can still compete or corroborate inside one lattice.

Primary risk: a lower-quality Nanda candidate could outrank a deterministic
repair. The протколах fixture is the mandatory competition test, and the
negative matrix checks false accepts.

### Correction-safety profiles

TD-112 continues to own Apply admission. Hybrid composition may add independent
typed-source agreement for the same surface, but duplicate aliases from one
producer do not create extra evidence domains.

### Latency, CPU, RSS, and allocation

Hybrid mode runs deterministic collection in addition to the already enabled
Nanda route. It must not run Nanda twice, create a second lattice, or repeat
DecisionCore.

The existing live Space owner waits at most 3.5 ms for prefetched full evidence;
that file, constant, and proof remain byte-identical. The existing hermetic
performance lane remains mandatory.

A pre-change local direct-resolution sample of 240 warmed NandaOnly requests
measured p50 65,267 us, p90 75,289 us, p99 87,840 us, and max 92,211 us. It
failed the old direct 5 ms absolute gate in the current package environment, so
TD-113 must not misreport that historical absolute gate as a new PASS. The
measurement is a direct resolution diagnostic, not the prefetched Space wait.

Post-change performance uses a paired same-process comparison after warmup:

- fixed six-surface round-robin;
- at least 60 NandaOnly and 60 DeterministicAndNanda observations;
- alternating order to reduce warm-cache/order bias;
- hybrid p50 no more than NandaOnly p50 + 5,000 us;
- hybrid p99 no more than NandaOnly p99 + 10,000 us;
- hybrid mean process-CPU time no more than NandaOnly + 10,000 us/request;
- retained RSS after the warmed hybrid loop no more than +16 MiB;
- existing prefetched Space wait proof and performance lane pass unchanged.

The first post-change paired run exposed a separate hot-path defect rather than
an authority failure. In debug it measured NandaOnly versus hybrid p50
6,860/167,782 us, p99 69,722/329,192 us, and mean process CPU
21,800/168,218 us. The optimized release run still failed at p50
1,210/24,694 us, p99 9,820/42,416 us, and mean process CPU 3,182/23,075 us.
Retained RSS passed in both runs (+16 KiB debug and +4 KiB release).

Stage tracing located the first shared mechanism before DecisionCore:

    hybrid source composition
    -> deterministic_text_candidates
    -> primary typing evaluator -> glued_phrase search
    -> composite fallback -> a second typing evaluator -> glued_phrase search

The existing word-material cache already memoizes the ordinary deterministic
typo primitives, but `correct_glued_russian_phrase` performs the same bounded
structural search without positive or negative memoization. Hybrid made that
legacy cost part of every Nanda-enabled request. The Nanda producer still runs
once and the merged lattice still reaches one DecisionCore.

Performance-fix options:

| Option | Score | Assessment |
|---|---:|---|
| Reuse the existing bounded, material-generation-aware word cache for positive and negative glued-phrase results | **9/10** | Selected. Removes duplicate work without a second cache owner, preserves the algorithm and invalidation boundary, and requires no dependency or persisted format change. |
| Reuse only the primary evaluator result inside the composite fallback | 7/10 | Removes the intra-request duplicate but leaves the first expensive structural search on every request and is unlikely to meet the paired CPU budget alone. |
| Add a new broad prefilter or rewrite the segmentation algorithm | 6/10 | May improve first-touch cost but changes structural coverage and carries substantially more regression risk. |
| Parallelize deterministic and Nanda producers | 2/10 | Cannot meet the CPU budget and adds concurrency and ownership complexity. |

The selected change adds one value kind to the existing 512-entry in-memory
LRU and routes the existing pure glued-phrase result through its established
material-generation key. It must cache `Some` and `None`, retain boundedness,
invalidate with the existing generation, and preserve all glued/split semantic
tests byte-for-byte at their public surface. It does not cache final decisions,
authority, edit plans, or mutation receipts.

There is no allocation-counting allocator in this repository. TD-113 therefore
makes no direct transient-allocation-count claim; it bounds retained allocation
through RSS, total work through process CPU, and hot-tail effect through paired
latency plus the unchanged prefetch deadline.

After the glued-phrase and prefix fixes, the unchanged absolute canonical lane
still exposed two shared hot residuals. A 20-sample warmed debug run measured
p50 7,036 us and p99/max 10,564 us. Candidate production took 3,459-5,393 us;
the full decision section took 3,211-5,162 us while the inner DecisionCore took
1,479-2,194 us. The difference is the correction peak material preparation.

Source and function-level tracing then separated the two mechanisms:

1. `prepare_correction_peak_context` calls the full
   `correction_l2_word_candidates` projection. `center_resonance` reads only
   surface, kind, score, and L1/L2/motif overlap, but the full projection also
   resolves per-surface morphology identities and builds fields used only by
   IME display ranking.
2. `proposal_only_substitution_competitor` repeats the same static near-surface
   scan on every request. Under GDB its first call took 258.6 ms and three
   warmed calls took 1.71-1.99 ms each. The result is lexical material, not a
   decision or authority receipt, and is therefore eligible for the existing
   material-generation-aware word cache.

Residual-fix options:

| Option | Score | Assessment |
|---|---:|---|
| Add one compact correction-peak projection and memoize proposal-only substitution material in the existing bounded generation-aware cache | **9/10** | Selected. Removes both measured repeated enrichments while preserving candidate geometry, ranking inputs, invalidation, DecisionCore, and verifier ownership. |
| Cache the prepared peak context or final correction resolution | 3/10 | Rejected. Context/usage can change, and caching a decision would mix material with authority. |
| Skip peak evidence or weaken the 5 ms p99 gate | 1/10 | Rejected. It hides the hot-path defect and changes the proof contract. |

The compact projection must preserve the ordered correction surfaces and the
exact kind/score/overlap tuple used by `center_resonance`; it may omit only
usage fields, target evidence, and morphology identities that the peak consumer
does not read. The proposal cache gets its own typed key so proposal-only and
automatic substitution authority cannot alias. Both positive and negative
material follow the existing 512-entry LRU and material-generation invalidation.
No final decision, authority, edit plan, or runtime state is cached.

### Residual root cause after the material-cache implementation

The compact peak projection reduced the decision section, and the distinct
proposal material key worked as designed, but the unchanged absolute canonical
gate still failed. A fresh 20-sample warmed run measured p50 8,831 us and p99
13,684 us. Kernel uprobes separated the hot request as follows:

| Owner | Warming-excluded observed range |
|---|---:|
| canonical candidate readout, including boundary join | 1,443-3,944 us |
| proposal-only substitution competitor | 3,158-5,118 us |
| compact correction peak | 1,320-2,261 us |
| inner DecisionCore | 1,516-2,500 us |

The proposal material cache was not missing or evicting the hot entry:
`propose_single_letter_substitution_candidate` ran 12 times while the uncached
`select_single_letter_substitution` computation ran once. The repeated cost was
inside `TransitionDecisionCore::admit_candidate_proposal`. A nested uprobe run
measured 2,399-2,718 us per hot request, dominated by
`known_current_word_gets_unproven_surface_drift_with_facts` (893-1,097 us) and
`structural_context_gate` (1,378-1,554 us).

That full lexical admission is redundant for this one producer. The producer
already proves a non-identical, syntax-compatible, one-letter substitution
material candidate, and its contract then caps the candidate to
`SuggestOnly` even when generic admission returns `Eligible`. It can never be
automatic Apply authority. Re-running every Apply-oriented lexical guard does
not change its effective gate action; it only changes a diagnostic reason and
spends the largest remaining hot-path slice.

Final residual options:

| Option | Score | Assessment |
|---|---:|---|
| Give the proposal-only producer an explicit `SuggestOnly` gate after its existing material, non-identity, and syntax checks | **9/10** | Selected. Preserves the effective authority cap, removes no candidate, caches no decision or authority, and leaves DecisionCore, verifier, SafetyGate, and mutation ownership unchanged. |
| Cache the full context-sensitive proposal admission result | 6/10 | Would reduce latency but would cache an authority-adjacent receipt and require broader invalidation than immutable lexical material. |
| Optimize the complete proposal-admission lexical fact path | 5/10 | Potentially useful later, but it is a wider refactor and the measured cost is unnecessary for a candidate that cannot Apply. |
| Cache the boundary readout | 4/10 | Boundary costs 1.3-2.1 ms on the normal hot samples and is not the dominant residual; this alone cannot satisfy the unchanged 5 ms p99 gate. |
| Relax the latency budget or drop candidate sources | 1/10 | Rejected because it hides the defect or degrades the candidate lattice. |

The selected correction is an authority reduction, not an authority bypass:
the proposal-only candidate is born with a typed upper bound of
`SuggestOnly`. Tests must prove its surface/source/class identity, that it
remains present for the boundary-ambiguity case, that it never becomes
`Eligible` or selected Apply, and that all fixed quality and latency gates stay
conjunctively green.

### Final latency owner after the proposal-only authority cap

The proposal-only cap removed the redundant full admission pass, but the
authoritative hermetic performance lane still failed in its normal debug test
profile. A fresh 120-sample warmed run on 2026-09-02 measured p50 4,144 us,
p90 5,803 us, p99 7,310 us, and max 7,959 us against the unchanged p99/max
limits of 5,000/10,000 us. The separately compiled release diagnostic passed
at p99 954 us and max 965 us; that confirms product optimization health but
does not waive the debug-artifact lane used by `scripts/check-lay-tests.sh
performance`.

Post-cap tracing showed one remaining repeated material owner:

    canonical readout
    -> ime_l2_boundary_candidates(context_prefix, token, limit)
    -> TailContext + L1 + BoundaryCell32 material construction
    -> identical material reconstructed for the next identical request

This is immutable candidate material for one package/material generation. It
contains no DecisionCore result, verifier receipt, correction-safety decision,
edit plan, mutation receipt, usage prior, or learning state. The complete key
must bind `context_prefix`, exact original `token`, `limit`, and
`candidate_material_generation`; both positive and empty results are material.

Final latency options after the cap:

| Option | Score | Assessment |
|---|---:|---|
| Cache the pure boundary readout at its existing L2 readout owner with a bounded generation-aware LRU | **9/10** | Selected. Removes only repeated material construction, binds every semantic input, caches positive and empty results, and preserves ranking/authority/verifier ownership. |
| Reuse boundary material only inside one resolution request | 6/10 | Preserves behavior but does not remove the measured cost between warmed repeated requests, so it cannot close the hermetic p99 gate. |
| Move boundary values into the canonical-field single-flight cache | 4/10 | Conflicts with that cache's explicit boundary exclusion and couples a display/readout projection to packaged field ownership. |
| Cache the final resolution or admission result | 2/10 | Mixes immutable material with context-sensitive authority and creates unsafe invalidation requirements. |
| Relax the debug latency budget because release is already fast | 1/10 | Hides a repeated hot-path computation and weakens the accepted conjunctive gate. |

The selected cache remains inside the L2 readout module, reuses its existing
mutex-protected bounded material-cache owner, and retains only the pure ordered
`Vec<L2ImeWordCandidate>` projection. A generation change makes every prior
entry unreachable and stale entries are removed before insertion. Tests must
prove exact positive and empty-result reuse, context/token/limit key isolation
(including token case),
generation invalidation, capacity enforcement, ordered value parity, and the
unchanged strict latency assertions.

### Residual root cause after the boundary-readout cache

The boundary cache removed the repeated `TailContext + L1 + BoundaryCell32`
construction and all focused cache/source tests passed, but the unchanged
500-sample debug gate still measured p50 3,563 us, p90 4,419 us, p99 6,295 us,
and max 7,015 us. The p99 limit is 5,000 us; the max limit remains 10,000 us.

Gated function timing then isolated the remaining deterministic work inside
`ranked_correction_candidates`. On the warmed 64-candidate field, usage lookup
took 3-6 us, the lexical-cache hit took 2-3 us, material mapping took 172-245
us, and sort/dedup/truncate took 1,076-1,526 us. The comparator recomputed
`typed_damage_geometry_priority` for both operands at every comparison. That
function repeatedly materializes character vectors and may run edit-distance
logic even though the normalized input and every candidate surface are
immutable for the duration of the sort. Surface character length and the
combined structural/usage rank were also recomputed by the comparator.

Final comparator options:

| Option | Score | Assessment |
|---|---:|---|
| Precompute geometry priority, combined rank, and surface character length once per candidate, then compare the stored keys in exactly the existing order | **9/10** | Selected. Converts repeated O(n log n) metric construction into O(n) construction without adding state, changing candidates, changing tie-break order, or caching usage/ranking/authority. |
| Reuse the complete ranked vector request-locally between canonical readout and peak preparation | 6/10 | The two paths do not currently own the same material, and plumbing a second payload through the canonical authority result would widen the change. |
| Add a persistent correction-peak cache keyed by a new usage-memory generation | 4/10 | Can be correct, but adds a new invalidation contract and state owner to eliminate work that is only redundant inside one sort. |
| Relax the debug p99 budget because release already passes | 1/10 | Rejected; it hides repeated allocation and weakens the accepted gate. |

The selected change is a pure comparator-key hoist. It introduces no cache,
side effect, dependency, public type, authority path, or invalidation rule. A
focused test binds every stored key to the original direct calculation, while
the fixed TD-113 behavioral and latency gates prove end-to-end parity.

### Final paired-lane root cause after the comparator-key hoist

The comparator-key hoist closed the absolute canonical gate: the fixed
500-sample warmed proof measured p50 2,762 us, p99 4,779 us, and max 5,484 us.
The separate paired proof still exposed a different cost. Before the final
repair, Nanda-only measured p50/p99 740/3,067 us and mean CPU 1,791 us, while
hybrid measured 10,979/20,520 us and mean CPU 12,105 us. Retained RSS delta was
zero. Per-stage tracing showed identical deterministic target evidence being
constructed by a primary producer and then admitted again by one or more
composite fallbacks before the replacement-only lattice merge. Depending on
the surface, the same target crossed proposal admission two or three times.

This was not a candidate-ranking or verifier defect. The duplicate work existed
before the lattice, and `AdmissionLexicalFacts` is intentionally local to one
admission call. Caching a final `CandidateGateDecision` was rejected because
admission reads request-time lexical/L2/HotField state and owns authority.

Final paired-lane options:

| Option | Score | Assessment |
|---|---:|---|
| Reuse only a fully identical, unmerged deterministic evidence record before a composite fallback repeats admission | **9/10** | Selected. Identity binds replacement, source, origin, source ID, and error class; merged, morphological, or authority-bearing candidates take the original path. The composite is still admitted normally when it wins with distinct evidence. |
| Share request-local immutable lexical facts across every distinct candidate | 7/10 | Potentially safe, but widens the admission API and does not remove repeated producer work. Keep as a later optimization only if measurements justify it. |
| Cache a final admission or resolution decision | 1/10 | Rejected. It would cache authority across state that is not represented by a complete invalidation key. |
| Relax paired latency/CPU budgets | 1/10 | Rejected. It would hide duplicated hot-path work and weaken the accepted gate. |

The selected fix retains the duplicate candidate's vector position until the
existing lattice merge, but clones only a candidate with exactly one matching
evidence record and no merged morphology or authority evidence. A test-only,
thread-local counter proves that five representative repeated targets cross
proposal admission exactly once; exact-evidence tests prove that a distinct
source ID, origin, class, or merged record cannot be coalesced. The counter is
diagnostic only and compiles out of production.

The then-final 60-sample release-profile paired checkpoint measured Nanda-only
versus hybrid p50 151/1,044 us, p99 546/2,223 us, and mean CPU 294/1,327 us,
with a 0 KiB retained-RSS delta. All unchanged paired budgets passed. A
separate debug-profile diagnostic measured p50 731/7,907 us, p99
3,428/16,536 us, mean CPU 1,760/9,640 us, and +4 KiB retained RSS; it exceeded
the paired p50 and p99 limits and is recorded as diagnostic FAIL, not silently
substituted for the canonical release-profile contract. This proof does not
claim physical-desktop behavior, broad language quality, or long-window
learning quality; those remain separate gates. The performance-only repair
adds no runtime authority, cache owner, persisted state, or mutation path.
Receipt: `tech_debt/evidence/td113-performance-v1.json`.

### Cache and package identity

No model package, DAFSA, lexical corpus, persisted cache schema, dependency, or
config format changes. The existing bounded in-memory word-material cache gains
typed result kinds for glued-phrase search and proposal-only substitution; its
capacity, generation source, LRU policy, and single cache owner remain unchanged.
The compact peak projection is request-local and creates no cache. The final
boundary cache is another typed value family inside the existing L2 readout
material-cache owner; it adds no persisted bytes or package identity. Existing
Nanda warmup and package identity remain unchanged.

### Rejected live-service snapshot-cache experiment

The removal of the unsafe cross-snapshot client seed cache exposed a direct
canonical diagnostic above its historical absolute debug threshold. A bounded
128-entry lattice-seed cache owned by `HostedMemory::Ready` was therefore built
behind a passed structural route and implementation preflight. Atomic reload
replaced the host and cache together; successful empty values were material,
errors were not cached, and focused server tests passed. This was an experiment,
not accepted production scope.

The first live trace showed that a cached L1.1 seed request was only a small
part of the full request while productive materialization remained the dominant
owner. The accepted paired gate was then run against isolated current-source
services in both server variants. The release measurements were:

| Server variant | Nanda p50/p99 | Hybrid p50/p99 | Nanda/Hybrid mean CPU | Hybrid RSS delta | Gate |
|---|---:|---:|---:|---:|---|
| snapshot-owned cache | 11,792/19,884 us | 10,410/19,505 us | 15,688/16,409 us | +48 KiB | PASS |
| direct current-snapshot read | 14,024/23,907 us | 13,414/20,295 us | 15,905/16,571 us | +304 KiB | PASS |

The single sequential A/B suggests a modest wall-time improvement from the
cache, but only 162 us lower Hybrid process CPU. Both variants satisfy every
accepted relative latency, CPU, and retained-RSS budget. The cache is therefore
not required to restore Hybrid behavior or close TD-113, while retaining it
would add a mutex-protected LRU, key/value cloning, capacity policy, a new state
owner, and reload-specific maintenance surface.

Options after the ablation:

| Option | Score | Assessment |
|---|---:|---|
| Keep direct current-snapshot seed reads | **9/10** | Selected. Passes the accepted release gate, observes the active snapshot naturally, and adds no state or invalidation contract. |
| Keep the server snapshot cache | 6/10 | Structurally safe and potentially useful, but below the minimum weighted benefit for this P0 correctness task. It needs a separately admitted repeated randomized A/B proof before reconsideration. |
| Cache final resolution, verifier, edit plan, or authority | 1/10 | Rejected because request-time authority cannot be retained behind an incomplete material key. |

The complete experimental server diff and its tests were removed. The final
`src/bin/lay_l1_1_serve.rs` is byte-identical to base commit `1158b02d`; neither
the installed runtime nor global IBus changed. The debug cache stress run is
retained separately as diagnostic RED and is not substituted for the accepted
release-profile proof. Receipt:
`tech_debt/evidence/td113-live-service-snapshot-cache-ablation-v1.json`.

### Rejected broad boundary-dominance experiment

The final Hybrid regression was not a candidate-birth, retention, rank-weight,
verifier, SafetyGate, edit-plan, or mutation-backend defect. A focused
DecisionCore test retained and verifier-evaluated both candidates for the same
current token:

- exact zero-loss boundary split `документыим -> документы им`, rank 1.111;
- close single-token repair `документыим -> документыми`, rank 0.976.

The exact split was then rejected by
`close_single_token_repair_competes_with_boundary`, so the batch selected
nothing. The paired repaired-boundary negative
`документаим -> документы им | документами` remained correctly blocked.

A six-line experiment allowed every verifier-passed
`current_token_boundary_split` to bypass that conflict guard. It passed the
focused DecisionCore matrix, all 50 IME tests, all active `td113_` tests, and
the six runtime-source contract tests. The complete changed gate then exposed
the counterexample `самка схема парочинная -> самка схема паро чинная`.
That candidate was an exact whitespace-only split and verifier-passed
`BoundaryMergeSplit`, but neither side of the split supplied the lexical
evidence needed to distinguish a real phrase boundary from a content-word
fragmentation. The verifier proved geometry only; it did not prove semantic
correctness. The first incorrect authority transfer was therefore the broad
DecisionCore conflict-guard bypass, not the producer or verifier.

The experiment was rejected and rolled back. After rollback,
`src/typing_transition/decision/admission.rs` again has SHA-256
`db1d18cb774a616d3bd4b5886dc5981a531fd8d3e77bf37095bbf85d12464ab4`.
No installed binary or live process was changed. The experiment, focused gate
outcomes, false accept, rollback bytes, and claim boundary are recorded in
`tech_debt/evidence/td113-boundary-dominance-broad-ablation-v1.json`.

Refined options:

| Option | Score | Assessment |
|---|---:|---|
| Preserve the existing close-repair veto except for an exact zero-loss current-token split where one exposed part is a known short Russian function word and the other is a known phrase part | **9.5/10** | Selected. Reuses the existing phrase lexicon, distinguishes `документы|им` and `документы|для` from `паро|чинная`, and changes no producer, rank, verifier, or physical authority owner. |
| Require independent L3/L4 context support for every boundary split | 7/10 | Safe, but rejects the required frameless positives when the context packages abstain or are unavailable. |
| Add a new semantic or field-margin comparator | 5/10 | Wider, calibration-sensitive, and unnecessary for the measured distinction. |
| Change global boundary rank weights | 4/10 | Affects unrelated candidates and does not encode the missing semantic distinction. |
| Delete the competing candidate or weaken the false-accept fixture | 2/10 | Forbidden: hides the conflict instead of resolving it and violates candidate-retention requirements. |

The selected predicate is conjunctive. It may bypass only this existing
close-single-token-repair veto when all of the following are already true:

1. the boundary candidate is verifier-passed `BoundaryMergeSplit`;
2. `current_token_boundary_split` proves the left context and all
   non-whitespace symbols are unchanged;
3. exactly the existing split parts are inspected, with at least one part
   satisfying `phrase_lexicon::is_short_russian_function_word` and the other
   satisfying `phrase_lexicon::is_known_russian_phrase_part`;
4. the boundary explanation has zero lost mass and strictly better
   preservation than the lossy competitor;
5. the competing repair remains verifier-passed, overlaps the same token, is
   within the existing rank-proximity bound, and has no lower risk.

No fixture string, source ID, new dictionary, global weight, second decision
pass, or producer-local Apply path may enter runtime code. TDD must cover both
right-side function words (`документы|им`, `документы|для`), function-word
position symmetry, multiple content-content splits, the live
`паро|чинная` false-accept case, a repaired boundary that changes letters, and
retention/evaluation of both competitors. Existing boundary, profile,
authority-binding, protected-surface, and full changed gates remain
conjunctive. The experiment did not test broad language quality or physical
desktop behavior, so neither is claimed by this ablation.

### Concurrency and stale state

Source composition and authority changes remain request-local. The performance
repairs create no new authority-state owner: word-level typo material stays in
the existing word-material cache, while boundary readout material stays in the
existing L2 readout cache. Both use the same candidate material generation for
invalidation. Existing runtime config snapshots continue to supply one boolean
that is converted to a typed mode before collection.

### Learning and logs

No learning weights or feedback semantics change. The stale corrections-log
writer is recorded but not repaired here. Aggregate audit evidence contains no
raw user text.

### Rollback and compatibility

The enum is internal Rust API. No persisted mode value exists. Reverting the
dedicated TD-113 commit restores 1.0.61 source behavior without config or data
migration.

### Maintenance

One shared live mapping prevents future CLI/daemon/IME drift. Pure source modes
remain explicit. No generic plugin/source registry is introduced.

## Post-review systemic repair and source proof

The second and final independent code-review pass scored the then-current
post-V28 implementation `7/10` with `HOLD`. Its blocking High finding was that
the intermediate `-ее` branch treated every attested `-ый` lemma as gradable
without independent Hunspell `E` evidence. The historical score is retained
exactly; no third review pass or replacement score is invented. Objective
closure is recorded separately from the review verdict.

The implementation repairs the first shared mechanism of each code finding:

1. The L2 boundary producer reports only exact target-bound structural
   grounding; it does not decide semantic competition. One complete
   Damerau-distance-one frontier over all 33 lowercase Russian letters is
   consumed at the existing DecisionCore Apply owner. A clean one-word conflict
   withholds only frameless surface-lexical Apply authority; it does not erase
   the split evidence or independent context, pairwise L3, or exact L4 evidence.
2. A boundary cache hit re-reads the material generation after lookup and
   before return. A deterministic `7 -> 8` turnover returns empty material.
3. The verification path exposed two sides of one morphology-certificate
   boundary. True soft adjective inflections cannot be backed by an
   incompatible hard-stem lemma. Synthetic comparative `-ее` from an `-ый`
   lemma requires exact Hunspell `E` membership; a regular non-velar `-ий`
   lemma independently retains its neuter `-ее` form. `-ой`, unknown, or
   unavailable class data grants no comparative authority. No fixture string
   becomes a runtime condition.

V24 is the pre-edit contract for the review findings. V25 pins the exact clean
HEAD morphology-owner baseline, but was necessarily run after the prototype
had exposed that previously undeclared owner. It is recorded as retrospective
design evidence and not represented as a before-code gate. Neither preflight
claims implementation correctness or deployment authority.

The first repository wrappers then exposed three real regressions on the
previously claimed final bytes: exact structural evidence was lost for
`тоесть -> то есть` and `когдая -> когда я`, and the known comparative state
`точнее` published unrelated corrected-prefix replacements. The changed gate
and full gate both failed with exactly those three tests; the diagnostic and
repair chronology is preserved in
`tech_debt/evidence/td113-repository-gate-red-v1.json`.

Post-repair focused verification on `e-MEGA-MINI-M1-13th` with 20 logical CPUs:

- compile-once gate: `PASS` in `26.705 s`;
- all three former repository reds: `3/3 PASS`;
- TD-113 focused tests: `23 passed`, `0 failed`, `1 ignored`;
- close boundary negatives: `3/3 PASS`; clean-prefix negative: `1/1 PASS`;
- live candidate safety matrix: `6/6 PASS`;
- executable source contract: `7/7 PASS`;
- complete narrow package: `PASS` in `115.749 s`;
- regenerated hermetic inventory after the test-ownership correction: `2,467`
  tests across 36 targets = `2,406` correctness + `36` package + `11`
  performance + `14` ignored, manifest SHA-256
  `64511fec25873ab76fc43152787bacf5e673d0ae530751b0f82343fa3bc46e25`,
  bound to an empty known-failure contract;
- observed-source route rerun: `PASS`, 15 routes, 16 nodes, 41 edges,
  `57/57` source markers, zero issues and warnings.

The first post-repair full wrapper correctly stopped on its stale architecture
receipt after the deterministic architecture suites passed `22/22 + 2/2`; no
test lane or performance result was claimed from that stopped run. After the
graph refresh, the restarted full wrapper passed those architecture suites and
all `11/11` named ownership checks, then reached the complete correctness and
package lanes. It held at `2,439/2,441` with exactly two failures. Both were
stale test-ownership assertions: they still required semantic whole-word
competition to erase exact target-bound L2 structural grounding, although the
accepted owner split retains that evidence at L2 and consumes the clean
single-token competitor veto only at DecisionCore Apply authority. The run is
a historical repository `HOLD`, not a quality PASS. Its log SHA-256 is
`306edac33bd8419e63382f4c0dfe1950e023bc2279e96637f65f33cc5a8e3291`;
the `1,239`-file source closure was unchanged before and after at
`585fb665ea1e432f2f86519bb18e21e620ceed243ccbf017393371f0f71485bf`,
and Cargo target usage remained `2,884,943,872` bytes. Performance and runtime
smoke were not run.

The test-only ownership correction then passed an exact six-test contract:
`6/6 PASS`. It proves both sides of the boundary: genuinely unproven fragment
splits remain ungrounded, while `авторручка -> автор ручка` retains exact
target-bound structural evidence without gaining Apply over its clean
single-token competitor. The log SHA-256 is
`8d5c099ce017ebad047b6368ce094809c058e71bd54077ae0fb71d78136865f6`.
The bound test hashes are `d41b067c75c6318980a4c7fe2bb27caff418f2939a8752151b7e582d0b68eae4`
for `src/correction_core/tests.rs`,
`9efd4d89c35f5cc76661b913d3ef6f02e38f54388f7c7ff2f35912524a9c1608`
for `src/nanda_wave/l2/tests.rs`,
`0c792ea07db0dcae0b417d6e1e804d455f0cd1f0bbdb188ec82716a3fae756ea`
for `src/nanda_wave/l2_field/bridge/tests.rs`, and
`5b6057b68d2bd79c3b1144f07c159b92bc02e8e844a97bc58ea04d04d853180f`
for the executable source contract. This focused PASS does not replace the
historical HOLD and does not claim a final full, changed, performance, install,
or live-runtime result. Installed runtime remains 1.0.61 and was not changed by
these source tests.

### Final-wrapper chronology after the ownership-contract correction

1. V3 ran against unchanged source closure
   `95c21a29be3eb0ac207f1de380d24b7420a251815fee54c8ec46d8881d9f1133`.
   All `2,442/2,442` Rust correctness/package tests passed, then default Clippy
   held on `clippy::let_and_return` in `tail_scan_adapter.rs:67` and
   `clippy::single_element_loop` in `decision/tests.rs:781`. Performance did not
   run. Full-log SHA-256:
   `017f7d3d4c583cbbf290925739c7c773f9426fb26cf7707e301fd4b6b262456d`.
2. The two mechanical Clippy repairs produced source hashes
   `dd475acef2817636e00c039c6f2be2be21884493b656afb5b6c2558ba74a1378`
   and
   `aa46001e082143812df748cfc244872e74285140c19b43c23085ee573a88f690`.
   Default Clippy, research-tools Clippy, and the exact source contract passed;
   their log hashes are recorded in the repository-gate receipt.
3. V4 ran against unchanged source closure
   `c588779593048f6aae33e0501616224379cbdcf8f14281c146c467b946d0b639`
   and reached `2,441/2,442`. Every runtime test was clean; the only failure was
   the brittle source-shape assertion
   `td113_boundary_competitor_veto_is_complete_and_generation_independent`.
   Full-log SHA-256:
   `d5a8c0be8c8b1e190e695442d188ff8a825bb16c23431dde2e640236b742f8e1`.
4. The behavior-bound contract repair has SHA-256
   `32df43279fd1de46b9d0265f817a0394b5f15cf2ef4f63e19dfb5534f97c6942`.
   Its integration contract passed `7/7`, and default plus research-tools
   Clippy passed. This closes only the brittle contract failure.
5. V5 ran against unchanged source closure
   `fee75b6982e497fd946aee99f3f38a17e35033e14024d162c91be0d6ea9ebc23`.
   Rust correctness/package passed `2,442/2,442` and lint passed. The wrapper
   then stopped with exit `127` at `node --check` because the remote non-login
   `PATH` omitted `/home/e/.local/bin`; `/home/e/.local/bin/node` exists and
   reports `v24.18.1`. No install or system mutation occurred, and performance
   did not run. Full-log SHA-256:
   `a7198aa7f50b35d5254d32e62818adc9405d57490d046e94db9af4018259b7cf`.
6. V6 resolved the executable lookup and again passed `2,442/2,442` Rust
   correctness/package tests, lint, Node syntax, Python compile, and shell
   syntax. Its CLI process exited `0`, but emitted `chosen: none` without a
   `confidence:` field, so the proof `grep` exited `1` and the wrapper held
   before the release build. The repository smoke had inherited absent remote
   user configuration and therefore default `auto_replace=false`; this was a
   non-hermetic proof dependency, not a runtime behavior failure. The source
   closure was unchanged, but its full digest and V6 duration were not
   authoritatively captured; only prefix `1abaed9` is recorded. Full-log
   SHA-256:
   `898e9a69b505643bb73a76f5a2dca9724b494281b917238311b0577921671435`.

### Hermetic full-gate repair and strict V7

| Option | Score | Decision |
|---|---:|---|
| Bind the repository CLI proof to the repository-owned autocorrect proof config | **9/10** | Selected. It makes the assertion hermetic without changing CLI semantics or remote user state. |
| Change the CLI default or semantics | 5/10 | Rejected. It changes product behavior to satisfy a proof harness. |
| Provision the remote user's `HOME` config | 2/10 | Rejected. It preserves a hidden mutable machine dependency. |

The selected repair scopes
`LAY_CONFIG_PATH="$ROOT/scripts/proof/autocorrect-proof-config.json"` only to
the CLI explain smoke in `scripts/check-lay-full.sh`. The script SHA-256 is
`ba6ddf988f26796847ca078b0797e93c78bec945fc67e88a724c189b85875214`.
An empty-`HOME` preflight passed with CLI exit `0`; its log SHA-256 is
`cb316767842bdd9dd843127c57de1c602418a447ac013f11d92dbeaddc1fff9c`.

V7 then completed the strict full repository gate in `459 s` with exit `0`:
architecture, `2,442/2,442` Rust correctness/package tests, lint, Node/Python/
shell syntax, hermetic CLI smoke, release build, and `git diff --check` all
passed. The source closure remained
`edc6f8d9ba4836e04a6f388ac7bf0a2cff7bd861137fdabcdc8ac898f3c5c2f7`;
Cargo target usage was `3,540,627,813` bytes before and after, delta `0`.
Full-log SHA-256:
`b0f8b41e05e09a97bfebf22c304a54945003514fe37e90e034243ffc4d49f816`.

V7 is retained as a historical pre-morphology `PASS`, not final-byte evidence.
Any performance measurement bound to those bytes is historical for the same
reason.

### Post-V7 adjective-paradigm morphology finding

The fixed morphology proof then exposed a distinct certificate gap. A single
soft-versus-hard suffix classification cannot represent Russian adjective
paradigms whose spelling mixes ending families after velars and sibilants. The
exact test
`russian_lexicon_tests::adjective_surface_certificate_respects_lemma_paradigm_and_stem_spelling`
failed with exit `101` first on valid `русского`: the certificate could not
connect it to the attested lemma `русский`.

| Option | Score | Decision |
|---|---:|---|
| Map each surface-suffix family to typed lemma endings and stem classes | **9/10** | Selected. It admits the attested paradigm while preserving spelling-specific negatives. |
| Broadly expand both soft and hard suffix families | 4/10 | Rejected. It admits cross-paradigm invalid forms. |
| Add literal surface exceptions | 1/10 | Rejected. Fixture strings must not become runtime authority. |

Preflight V26 correctly returned `BLOCKED_BEFORE_CODE` with
`safe_to_implement=false` because the preservation reference
`morphology-form-owner` was unknown. After that owner was pinned, V27 returned
`READY_TO_IMPLEMENT`, `safe_to_implement=true`, with zero blockers. These are
design receipts only:

- `tech_debt/evidence/td113-adjective-paradigm-implementation-preflight-receipt-v26.json`;
- `tech_debt/evidence/td113-adjective-paradigm-implementation-preflight-receipt-v27.json`.

The typed repair source SHA-256 is
`555dfe8f8f2c25f90b12bf968f522d99d0d4b3628f90951b526a9d63ebde10a4`;
the test SHA-256 is
`72a692fc32dbf6ba81d6077659c9a33e88af19843e741fef2851871274f2bc07`.
Focused verification passed:

- exact adjective-paradigm regression: `1/1`, log SHA-256
  `942e33929735983081f7b31e0bb81b583b6f16fc1662eb121b086357d4fa321b`;
- Russian lexicon suite: `14/14`, log SHA-256
  `866ef0b10f9824a88c06a0c7ed4c05837bff571c6f8b59339e6f7588b9dd81ae`;
- TD-113 DecisionCore scope: `33/33`, log SHA-256
  `db49b48f40d227f608583f47127a94c78ff0d78b4ed7cf3f19ac66cc535c4656`;
- executable source contract: `7/7`, log SHA-256
  `caaec3644a06b4419bda1f40206575d255898e6e9f6202c856b8a30d2795db29`.

This focused GREEN did not restore final repository or performance authority.
At that checkpoint the manifest, observed-source route, changed/full wrappers,
final-byte performance, Graphify/architecture refresh, commit/push, 1.0.62
release, rollback-safe installation, and live verification remained pending.
The later E-comparative checkpoint below supersedes only the manifest fact;
overall status remains `HOLD`, broad quality remains `UNKNOWN`, and installed
runtime authority is unchanged.

### Review-pass-2 E-comparative High and scoped objective closure

Review pass 2 found one High false-certainty mechanism after the typed
adjective-paradigm repair: the intermediate `-ее` route admitted every
attested `-ый` lemma without proving that the dictionary marks it as gradable.
The exact regression returned `0/1`, exit `101`, first on `почтовее`; the same
RED matrix covers `атомнее`, `даннее`, and `школьнее`, while requiring
`точнее`, `синее`, and `хорошее` to remain accepted. RED log SHA-256:
`fe866cc16259d63b3e7655636553c1349f77418a79c6611392eddb91aec2b99c`.

V28 remains the historical pre-review classification contract. Append-only
preflight V29 returned `READY_TO_IMPLEMENT`, `safe_to_implement=true`, with
zero blockers and canonical manifest SHA-256
`0267f17d184a3cda741f69d9fc012924175deec28a957382b50c103450b2f3c9`.
The minimal route is:

```text
Hunspell word/flag record
-> collect exact E membership beside existing A/O projections
-> russian_adjective_has_comparative_ee()
-> -ый + ее requires E

regular non-velar -ий + ее
-> preserve as neuter through the existing regular-A class

missing E / unknown lemma / loader failure
-> false; no comparative certificate
```

Final focused-source SHA-256 values are:

- `src/russian_lexicon/forms/backed.rs`:
  `2c982e16a21544232779075fbbb42ca8401929680768df2b35143603161eef23`;
- `src/russian_lexicon_tests.rs`:
  `cd8c2cd53a06345e49c7a7c7d3782469adc40eb087a469ee88dc2706b720db78`;
- `src/russian_lexicon.rs`:
  `659cc4a059c128c28a01643fa9a9b91f95800b4a3574fb3dbbcc930ce02939e7`;
- `src/russian_lexicon/hunspell.rs`:
  `aefc384c73936d83d27f88003ac0e7880f465a059c26737999d912b036b53eed`.

Focused GREEN passed:

- exact E-comparative regression `1/1`, log SHA-256
  `b840fc182a7dc47ea85b360b207fb497b0e674a904095e87dca53afbfd956f48`;
- Hunspell loader `2/2`, log SHA-256
  `c65e4cbc0b9b134a0b5116d6f34c275c1240da04652f50197fd923f08e4eb789`;
- Russian lexicon `17/17`, log SHA-256
  `98dbe0d1f350d2d9dcea6c9955f4090351123af36acd85387ec86dc5ac52abdd`;
- TD-113 DecisionCore `33/33`, log SHA-256
  `eafff142f5e241877ff14b0d738c6564c08edd2b3d6400e1876622113c795284`;
- executable source contract `7/7`, log SHA-256
  `fdd1bb02218e0d9c6c3a26d0de8470a697727dc638eab66ea50de71f229c122e`;
- Clippy exit `0`, zero non-dead-code diagnostics and `366` dead-code
  warnings, log SHA-256
  `0a3bd17532d6d87b7cdb05e095f6dc684d091262cb29cfd589155d9eceaa10be`.

Cross-host parity is confirmed only for newline-normalized projections built
by `awk` word/flag extraction followed by `LC_ALL=C sort -u`: regular `A -ий`
is `24,723` rows with SHA-256
`4d734e5fd62e10a2817246fdb76bb4a12ad67f740da0d524a93d9bb9309b8693`,
possessive `O -ий` is `170` rows with SHA-256
`101d65585a282daac494a3bc0216c5e240bd0d28d1eb4761e87983ff44c85f3f`,
and all `E` entries are `1,054` rows with SHA-256
`dc3f1550cbb9f4856801e74cd9d1b684f2479522f8c5c101bdd633e2e723c102`.
The full dictionaries intentionally differ and are not the parity denominator.
Exact receipt: `tech_debt/evidence/td113-hunspell-projection-parity-v1.json`.

Historical pre-V30 checkpoint: the then-current manifest had `2,469` tests across 36 targets: `2,408`
correctness, `36` package, `11` performance, and `14` ignored; SHA-256
`27355256ce8212fb12cdfc62c5c160fd565cff7d6c93c3dde0e114c136a4ee35`.

That pre-V30 performance checkpoint ran against unchanged source closure
`e2b5f76b24a0c9205400e6cfd27dcd45a623f9ffb93ae15f4328e14b5ec3223f`.
The canonical V3 proof completed all `500` requested samples in `9 s`, exit
`0`: p50 `1,759 us`, p90 `1,898 us`, p99 `2,110 us`, maximum `2,502 us`;
log SHA-256
`902a982520d2af3d8290eefac1d73817f254df460d924147aa6fba69cdb74178`.
An earlier harness invocation requested `500` but produced only `120` samples;
it exited `0` but is preserved explicitly as `NOT_FINAL`, log SHA-256
`1f7085954bf4460931d18495ff1b32ed5766b75b17dc37e042a03054d7b64a69`.

The pre-V30 paired release proof completed `60` samples per mode and passed. Nanda had
p50/p99 `83/644 us` and CPU `295 us`; Hybrid had p50/p99 `849/2,860 us` and
CPU `1,306 us`. RSS moved from `246,248` to `246,252 KiB`, delta `4 KiB`.
The run exited `0` in `281 s` including LTO; log SHA-256
`b1ad54ea02697c857473ee7e5444978cf0aa47446e19cd6ca660c56d14fddbbf`.
This is `PASS_FINAL_BYTES` for performance only.

These facts objectively close the High finding in focused scope without
rewriting the historical `7/10 HOLD`. A fresh observed-source rerun also exited
`0`: `PASS`, 15 routes, 16 nodes, 41 edges, `57/57` source markers, zero issues,
and zero warnings. Its fresh and accepted receipts have the same canonical JSON
SHA-256 `8ca39ee3c745b56beda881b3560c875d75d9f2c57d5b6d25347f493458e754a9`;
only object-key order differs. The pre-V30 architecture wrapper also exited `0`:
Graphify rebuilt `21,546` nodes and `53,618` edges, all `11/11` ownership
checks passed with zero violations, and the deterministic suites passed
`22/22 + 2/2`. Historical receipt:
`tech_debt/evidence/td113-architecture-refresh-v1.md`. Final changed/full,
completion, installation, and release gates remain pending.

## Acceptance Criteria

TD-113 is complete only when:

1. The aggregate audit receipt is committed without raw user text.
2. The design code-route gate passes and reports one rank, authorization, and
   mutation owner for the hybrid event.
3. Implementation preflight returns READY_TO_IMPLEMENT with
   safe_to_implement=true before source edits.
4. A fresh-context task reviewer scores the specification at least 8/10, or,
   after the two-pass limit is exhausted, the actual score is retained and
   every High/Medium correctness finding is closed by objective gates under
   `tech_debt/README.md` rule 5.
5. Red tests prove the source-loss and live-mapping failures before the fix.
6. DeterministicOnly, NandaOnly, and DeterministicAndNanda source contracts pass.
7. All required Experimental-profile fixtures select the required surface.
8. протколах never selects пр отколах in the hybrid route.
9. Strict/Normal/Experimental retain their TD-112 semantics.
10. Negative fixtures have zero new false accepts.
11. IME, daemon, and configuration-aware CLI select the shared live mode.
12. Pure Nanda diagnostics remain NandaOnly.
13. A compiling behavioral red test fails before the enum/source implementation.
14. The executable runtime-source contract rejects fixture literals, private
    live mappings, added decision passes, and mutation/prefetch edits.
15. Separate observed IME and daemon routes each contain one edit plan and one
    event-specific mutation backend; no event contains both backends.
16. The paired performance proof satisfies the latency, CPU, and RSS budgets,
    and the existing prefetched Space performance proof remains unchanged.
17. Verifier, SafetyGate, edit-plan, exact authority, Space-prefetch, and
    mutation-backend semantics are unchanged.
18. Focused tests, scripts/check-lay-changed.sh, and
    scripts/check-lay-full.sh pass through scripts/cargo-guard.sh within the
    12 GiB target budget.
19. The observed-source route gate, owning architecture document, and Graphify
    graph are refreshed in the same change.
20. Fresh-context code review scores at least 8/10 with no unresolved
    correctness finding after at most two passes; when the two-pass limit is
    exhausted, `tech_debt/README.md` rule 5 applies: retain the actual score and
    require objective closure of every finding without inventing a third pass.
21. TD-113 is marked DONE, committed separately, and pushed.
22. Lay 1.0.62 is built, installed with the existing rollback-safe release
    procedure, live-verified, committed, and pushed.

## Stage 2 Follow-Ups

These are intentionally excluded from the minimal fix and require a separate
decision:

1. Restore continuous correction-learning logging. The corrections log stopped
   on 2026-08-23 while runtime actions continued. Before repair, determine the
   true writer owner, privacy boundary, retention/rotation policy, and whether
   IME and daemon events are deduplicated.
2. Build a fixed privacy-safe heldout corpus from accepted/rejected aggregate
   classes. Quality must report aggregate and per-error-class outcomes, not
   candidate availability.
3. Add long-window live metrics for candidate presence, selected source,
   no-apply stage, visible mutation, rollback, and user rejection as separate
   denominators.

Stage 2 must not delay the source-composition repair.

## Rollback

Before release, revert the dedicated TD-113 source commit. After 1.0.62
installation, use the existing atomic rollback-safe installer procedure to
restore the complete previous installed tree. Do not restart global IBus;
restart only Lay-managed processes and verify the unchanged global IBus PID.

## Evidence To Produce

- evidence/td113-log-audit-v1.json
- evidence/td113-code-route-design-v1.json
- evidence/td113-code-route-design-receipt-v1.json
- evidence/td113-implementation-preflight-v1.json
- evidence/td113-implementation-preflight-receipt-v1.json
- evidence/td113-task-review-v1.md
- evidence/td113-red-green-tdd-v1.json
- evidence/td113-performance-v1.json
- evidence/td113-implementation-scope-v1.json
- evidence/td113-code-route-observed-v1.json
- evidence/td113-code-route-observed-receipt-v1.json
- evidence/td113-verification-v1.json
- evidence/td113-code-review-v1.md
- evidence/td113-architecture-refresh-v1.md
- evidence/td113-final-review-repair-implementation-preflight-v24.json
- evidence/td113-final-review-repair-implementation-preflight-receipt-v24.json
- evidence/td113-morphology-certificate-implementation-preflight-v25.json
- evidence/td113-morphology-certificate-implementation-preflight-receipt-v25.json
- evidence/td113-adjective-paradigm-implementation-preflight-receipt-v26.json
- evidence/td113-adjective-paradigm-implementation-preflight-receipt-v27.json
- evidence/td113-adjective-classification-review-repair-implementation-preflight-v28.json
- evidence/td113-adjective-classification-review-repair-implementation-preflight-receipt-v28.json
- evidence/td113-e-comparative-review-repair-implementation-preflight-v29.json
- evidence/td113-e-comparative-review-repair-implementation-preflight-receipt-v29.json
- evidence/td113-hunspell-projection-parity-v1.json
- evidence/td113-repository-gate-red-v1.json
- evidence/td113-missing-duplicate-authority-code-route-design-v1.json
- evidence/td113-missing-duplicate-authority-code-route-design-receipt-v1.json
- evidence/td113-missing-duplicate-authority-implementation-preflight-v30.json
- evidence/td113-missing-duplicate-authority-implementation-preflight-receipt-v30.json
- evidence/td113-missing-duplicate-authority-repair-v1.json
- evidence/td113-architecture-refresh-v2.md

## Final Source-Gate Checkpoint

The final source closure is identical locally and on the 20-CPU remote host:
`1,238` files, SHA-256
`0062b6b55c6e0f9ec6d86619ea754d6693ab12ff4d25e626a259fbb40191aae9`.

- `scripts/check-lay-changed.sh`: `2,449/2,449 PASS`, zero semantic and
  infrastructure failures, unsafe-edit scoreboard `PASS`; log SHA-256
  `2819e85ad8e65bfca0f912224eb9f505eaa247f3ade157fe0c1c030e5008a199`.
- `scripts/check-lay-full.sh`: `2,449/2,449 PASS`, followed by lint, Node,
  Python, shell, hermetic CLI smoke, release build, and diff checks; log
  SHA-256
  `75966f288734e87e73cdcccf9321a145da12eea654f562821211f031b438341a`.
- canonical performance: `500/500 PASS`, p50/p90/p99/max
  `1,841/1,944/2,971/3,701 us`; log SHA-256
  `a5705197d3db6e607ecd2a968c3c359f8bd383b57cae9786d8096ac7c9c24f8d`.
- paired release performance: `60` samples per mode, `PASS`; Nanda
  p50/p99/mean CPU `97/446/286 us`, Hybrid `804/1,692/1,170 us`, retained RSS
  delta `4 KiB`; log SHA-256
  `8e928454ccf4c6cba82b755d123a63f6a741b1cc948130a05262a2a4b62ed2b7`.
- final architecture: `21,559` nodes, `53,638` links, `1,001` communities,
  `678` bound Rust sources, `11/11` ownership checks and `22/22 + 2/2`
  deterministic suites `PASS`; generated receipt SHA-256
  `758f6b4a708c246769fe32a66b7e0b34de3b42a9181f7892dd0e6898c1e9f540`.
- fresh-context final review: `9/10 PASS`, zero High or Medium findings. The
  one Low gap is the absence of a standalone live-hybrid exact-case test for
  `руских`/`агресивнее`; the shared route and full repository gates cover the
  mechanism, so this does not widen the minimal fix.

This checkpoint grants source-scope correctness and performance evidence only.

## 1.0.62 release completion

The exact pushed release-source commit
`83b379789da770bd5fbd0212642084dfcb54679e` passed the bounded remote full gate
on the 20-CPU host. The fixed correctness/package denominator passed
`2,449/2,449` with zero known semantic and infrastructure failures; lint,
Node/Python/shell syntax, CLI smoke, release build, and `git diff --check`
passed. The full-gate log SHA-256 is
`57fc9b959a628c0bd29fe1d3151bbdacaeda77d96d225cf30e0791850517a319`.

The rollback-safe controller installed 1.0.62 from those exact binaries and
reported `FORWARD_INSTALL_1_0_62=PASS`. CLI and DBus report 1.0.62, DBus ping
passes, daemon/L3/IME process hashes match the installed artifacts, exactly one
managed IME is running, the selected engine remains `lay-ime-ru`, and the
global IBus PID remains `4715`. The complete 1.0.61 rollback snapshot is
`/home/ubu/.local/state/lay/release-backups/1.0.62-preinstall-20260903-j1OKv0`.

TD-113 is `DONE`.
