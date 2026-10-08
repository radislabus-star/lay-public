# Space repair of a displaced last-pair boundary

Status: SOURCE_RELEASE_PASS_PHYSICAL_PENDING; excess diagnostic replay retired.
Current scope: closing Space, unknown right word, move its first Unicode
scalar left, independently attest both reconstructed words, then perform one
authorized final-pair edit. Original left-word validity does not block repair.
Installed bytes are824e2f5f3c6d1b65c6219199f9f6aac9eb94ebee70a34f2d61e9b7640cbcc520.
The complete source release gate passed. Physical acceptance remains pending.
Source base: `bee14b089d5d64c03b0a6fe5b06e8f12a16ffa7d`.
Implementation checkout: `/home/ubu/projects/lay-space-boundary-shift-20261007`.
Branch: `codex/space-boundary-shift-20261007`.

## Authorization and existing mechanism

On 2026-10-07 the user requested the class-level repair
`должн ыбыть ` -> `должны быть ` on the closing Space of the right word.
The original left word may be correct, for example
`план ыработают ` -> `планы работают `. Test both reconstructed words;
do not require the original left word to be corrupt or encode fixture text
as runtime conditions. The unknown right token is evidence only when the
material provider is available; unavailability remains unknown.

The user initially froze IME delivery. They subsequently explicitly allowed
all necessary IME changes for this mechanism and required verification in
all available real windows. This is permission to implement the connected
repair, not to weaken SafetyGate, add a second chooser/executor or claim
source tests as physical acceptance.

Existing owners: BoundaryShift producer, TransitionDecisionCore, verifier,
EditAction/AuthorizedEdit, ContextAdmissionReducer, WindowInteraction and
the existing IME output adapters. C02/C03/C05/C06/C08 are affected contracts
to preserve. Protected Double Shift and accepted Space/layout contracts remain.

## First established loss and evidence gap

The producer already permits ordinary clean left words. Current adapter
`src/ime_correction.rs:296-300` projects a selected action over an unchanged
left prefix. A requested pair rewrite cannot pass that projection.
Existing Space snapshots, deletion counts and active composition commits
also own only the last token; removing strip_prefix is not a repair.

Private observational evidence is recorded in
`/home/ubu/.cache/lay/project-study-20261007/space-boundary-live-evidence-20261007.json`:
first Space NoApply/Rank, second gate Apply with selected receipt but prepared
NoApply/Verifier. The exact live selected surface was not logged. A focused
production-core test must establish it independently before editing runtime.

## Consequence analysis and design alternatives

Preferred direction: an exact final-pair scope admitted by the existing
reducer and bound to the existing field/focus/epoch/input/material/config
lease. Retain ordinary current-token scope when no independent pair witness
exists. The semantic action must retain the same selected BoundaryShift
receipt and original full surface; no second ranking or synthesized winner.

Full committed pair: reuse the existing no-cursor suffix replacement with
exact pair snapshot and matching winner action. Use the whole original
pair for feedback/undo and preserve exactly one trailing Space.

Mixed legacy preedit: the left word/separator are in the client, the right
word is IME-owned composition. Require exact unselected client prefix and
exact end-of-preedit witness under one owner. A typed projection must account
separately for retiring composition and deleting client text. Never delete
the composition length from the client. Native lineage/start proof and projection/output contracts are bounded in
source; independent connected review and installed-client proof are required
before promotion.

Alternative: always widen last-token scope to a local mirrored last pair.
Rejected: a mirrored prefix is context, not permission to mutate prior text;
UnknownStart/current-token ownership cannot retrospectively prove an earlier
word. Require existing observed-boundary lineage or an exact client snapshot.

Alternative: a new after-Space fallback/output handler.
Rejected: races ordinary correction, stale results and partial effects;
adds a second owner and can duplicate edits or the closing Space.

Candidate retention/ranking: preserve independently grounded candidates and
current scores. Generic stem-plus-suffix plausibility is not an independent
lexical certificate for a boundary rewrite. Use exact hot surfaces or warmed
imported canonical form readings; unavailable readings remain unknown. This
does not grant a producer apply authority. Unsupported scope must abstain,
never promote generic uncertainty or fabricate model/lexical evidence.
Cold packages or online reloads must invalidate the same existing identity.
No new cache, worker, timer, retry, polling, dictionary or model is justified.

Latency/CPU/RSS/allocation estimates: bounded two-token inspection and short
typed witnesses are intended; cost is unmeasured. Keep the existing callback
wait deadline. Do not add per-key model work or a late Apply after NoApply.
Concurrency: validate the same lease before effects; revoke on focus, Reset,
selection, cursor movement, unknown input, revision/config/material changes.
Unknown/partial output must not fall back or start a second mutation.
Feedback must await the existing observed postcondition. Undo must restore
the exact original pair, not only the old right token.
Compatibility: terminal, GTK SurroundingText, legacy preedit and atomic have
different physical partitions; prove each separately. Shared authority must
not silently reinterpret terminal erase geometry or atomic effect ordering.
Maintenance/removal: extend the existing owner contracts with one bounded
scope/projection; no parallel permanent route or duplicate raw-word buffer.
Rollback boundary: restore the prior source/artifact; no installed change yet.

## Fixed proof and promotion gates

First run one focused semantic panel through real InputGate/DecisionCore and
adapter APIs: supported shifts with clean and corrupt left surfaces, paired
clean controls, known original right words and unsupported reconstructions.
Assert surfaces, verdicts, BoundaryShift proof, two-token scope and SafetyGate
effects. Preserve the old current-token refusal when only read-only context
exists. Collect the complete panel before diagnosing a shared loss.

After the scope design is complete, add production reducer/adapter tests for
both Spaces and the real mixed/committed projections, stale leases, input
interleavings, one dispatch, one Space, exact feedback and undo. Prove the
regression detects the old implementation or a controlled invalid partition.
Run remote-only under existing resource/Cargo guards. Obtain independent
connected review and mandatory source/architecture/release gates.

Finally identify exact installed bytes and exercise every available real
window/field, including clean controls, next input, Tab, Backspace/undo,
focus change and protected Double Shift/layout behavior. Enumerate actual
fields and results; an unavailable field is NOT_TESTED, not PASS. Do not send
test messages or modify user documents. Physical owner acceptance is separate.

## Reproduction: full-reference library panel

Receipt: `/home/ubu/.cache/lay/development/run-ub90izym/RESULT.json`.
Remote guarded execution: 3 tests executed, 0 passed, 3 failed; 38.4 seconds.
The configuration named IME but the integration library retained its default
FullReferenceAllowed process policy. These results are not live-IME parity.

Measured: the shared core selected the requested BoundaryShift for
`должн ыбыть` and `расчет ыготовы`; a physical action without the pending Space
was refused as weak_boundary_edit_shape. The core incorrectly admitted
`должен ыбыть` -> `должены быть`. A generic stem/suffix recognizer treats the
stored stem as support without checking the imported inflection paradigm.
`план ыработают` selected a competing MissingLetter candidate; `допусти
мнабираю` was SuggestOnly at Normal safety. Do not force either fixture into
Apply by a word rule or a new score. The fixed panel retains these observations.

The current-word adapter refused all pair edits; one refusal was Rank rather
than Verifier because the shared core did not select Apply. Its original
test expected Verifier for every input, an incorrect stage assumption.

No production or installed bytes changed. Live hot-policy provider admission,
typed pair scope, mixed physical partitions, quality aggregate, latency,
release and all real fields remain NOT_TESTED.

## Bounded implementation decision

Retain the semantic action including exactly one pending ASCII Space and run
the existing SafetyGate unchanged. A sealed PendingSpace projection alone may
remove that virtual separator from the physical deletion source. Independently
dry-run the no-cursor physical plan; retain the selected receipt, two changed
tokens, BoundaryShift operator and Boundary proof. Other whitespace, partial
plans, different targets and unverified actions cannot use this projection.

Capture optional final-pair evidence in the existing InputFrameIdentity and
revalidate by exact recapture before output. Its provenance is existing reducer
observed-boundary lineage, an exact current client snapshot, or an exact owned
preedit start witness. Keep lexical source_window on the current token.

Extend the existing suffix executor with a typed owned-preedit partition.
Verify the full semantic pair, but validate and delete only the actual client
prefix when the right token is owned preedit. Retire preedit and use the same
delete/commit transport, same partial-effect failure propagation, same full
pair feedback/undo. No separate output owner or fallback is introduced.

Independent review round 1 found that generic recorded undo rejects a changed
two-word boundary (first loss: UndoRecord -> SafetyGate). Preserve the selected
forward action in the existing pending undo record and derive only its exact
sealed inverse; verify the inverse physical plan and current visible suffix.
Generic multiword UndoRecord remains refused. Partial output consumes/revokes
the paired undo capability rather than retrying a possibly dispatched edit.

The first hot-policy reproduction retained all three original failures:
`/home/ubu/.cache/lay/development/run-xrty95_7/RESULT.json`, 3 executed,
0 passed, 16.6 seconds. First implementation panel:
`/home/ubu/.cache/lay/development/run-h6jsvleb/RESULT.json`, 5 executed,
2 passed, 3 failed, 37.1 seconds. Clean/unsupported controls passed, but no
canonical package was admitted by the hermetic integration sandbox: inflected
targets lost independent support. Add a small membership-only fixture to
contract tests and separately prove the actual installed canonical package;
the fixture has no L1 links/ranking/context edges and is not quality evidence.

## Provider and executor verification — 2026-10-07

First connected owner panel: `/home/ubu/.cache/lay/development/run-szy60pcz/RESULT.json`,
695 executed, 686 passed, 9 failed, 86.5 seconds. Integration membership-only
contract tests passed 5/5. Five new binary tests encountered the immutable
canonical-provider unavailable result admitted by earlier tests in the same
harness. Isolate these tests in exact-name child harnesses with fixture
admission before the production owner starts; never reset the production
OnceLock or change unrelated tests' environment.

Four existing owner regressions exposed a physical-plan mistake: a logical
replacement length was used instead of the already authorized physical
plan's deletion count. Restore the ordinary executor's authorized count;
only the typed owned-preedit partition supplies a distinct client count.
Their assertions and manifest remain intact. Reverification pending.

Actual imported canonical package panel (without L1/productive models):
`/home/ubu/.cache/lay/development/space-boundary-actualcanon-7r85pub5/RESULT.json`,
5 executed, 4 passed, 1 failed. Canonical package SHA-256:
`cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b`.
The user's `должн ыбыть` pair projection, unsupported-form controls,
competing/suggested candidate retention and current-word-only refusal passed.
The selected-pair test failed for `расчет ыготовы`: KeepOriginal, no selected
candidate. Its first failed layer is not established yet; collect exact
imported readings and the bounded BoundaryShift lattice before changing
any mechanism. This is actual-provider evidence, not full installed parity
or aggregate quality evidence.

Pair frame evidence stores only the final pair and revision/lease identities;
client snapshots are reconstructed at dispatch. This avoids a new full-field
clone on every key. Latency improvement is a hypothesis until measured.
Owned-preedit output fails closed if its client partition cannot be rebuilt.
Native mirror start zero cannot manufacture a left-word start: existing
KnownStart and an earlier actually observed separator are required.

Installed bytes, runtime authority, input sources and services are unchanged.
Independent review pass 2, graph/canon, release gates, latency, actual-provider
regression closure and real-window acceptance remain pending.

## Connected owner regression closure

`run-pjo8yiz7/RESULT.json` stopped during compilation of the diagnostic test:
a collection needed an explicit Vec type. No tests executed; no runtime change.
`run-awougwnf/RESULT.json`: 699 executed, 696 passed, 3 failed. Two new
committed-pair fixtures had retired ManagedCommit mode while loading a prior
separator directly into the mirror; reestablish the declared right-word mode
after fixture construction. The real both-Space and owned-preedit key paths
already passed. The remaining old regression required preserving the ordinary
logical-request backend eligibility check even for an append-only physical
plan. Keep the old selector input for ordinary actions; only the proven owned
partition supplies a different physical client range. Assertions remain intact.

Final focused owner panel: `/home/ubu/.cache/lay/development/run-fj31ubsp/RESULT.json`,
701 selected and executed, 701 passed, 0 failed, 77.2 seconds. Full IME binary
owner tests and all six new shared-core integration tests were selected using
the existing discovery/isolation runner. The ten new owner regressions include
both Spaces, ordinary and exact-refresh mixed preedit, full committed client,
stale witness, native bounded provenance/refusal, native atomic one-frame
output, forward and inverse partial failures, and exact recorded undo.
This is synthetic membership-only contract evidence, not model quality,
installed-byte proof or physical-client acceptance.

Actual-provider diagnostic: `/home/ubu/.cache/lay/development/space-boundary-actualcanon-diagnostic-xkmulm9m/RESULT.json`,
6 executed, 5 passed, 1 failed. For `расчет ыготовы`, warmed imported readings
for `расчеты` were confirmed empty, `готовы` has adj/pl/short identity.
The BoundaryShift lattice is empty: first loss is exact target support at
candidate generation, before ranking and authority transfer. This panel did
not mount actual L1 or productive material; no installed failure is inferred.
The е/ё hypothesis and actual L1 membership remain unverified.

No installed bytes, runtime authority, sources or services changed.

## Explicit replan after connected review pass 2

Review pass 2 repaired the paired-undo blocker but identified an independent
provider-preservation gap: the observed-right clean-surface veto does not
explicitly consult cached imported canonical forms, while reconstructed targets
do. A provider-positive original right word must veto the transfer even if
the old hot/full-reference readout misses it. Before further production edits,
replan the next bounded change: reuse the exact same attestation accessor for
that preservation guard; prove it with a controlled canonical-positive right
form in an isolated child harness. No new chooser, owner, cache or score.
Keep every previous test assertion and provider-unavailable refusal.

The native AtomicEffectBuilder regression proves one physical effect frame
with atomic.active=false. It is not ProposeKeyEvent/client receipt-settlement
or rejected-proposal proof; generic atomic tests remain required, and physical
atomic client acceptance is NOT_TESTED. Next independent review is limited
to the declared provider-preservation delta after this explicit replan.

Bounded alias diagnostic: `/home/ubu/.cache/lay/development/space-boundary-one-diagnostic-saudpkps/RESULT.json`,
1 selected/executed, 0 passed, 1 failed; all six integration identities checked
and source/workspace immutable. Actual canonical contains `расчёт`/`расчёты`
(noun sg/pl nom/acc, lemma 69762), while unaccented `расчет`/`расчеты` have empty
readings. No L1/productive provider was mounted. Do not reinterpret this as an
installed regression or force an unsupported form by weights or literal rules.

The first known-right control failed fixture admission: changing a surface
in-place violated the canonical package's strict form order. Keep that guard
unchanged; use a correctly ordered alternate membership fixture with one
additional positive observed-right form. This is a controlled provider truth
for preservation, not a claim that the synthetic spelling is a Russian word.
`run-znfaud70` stopped at a private-module test reference during compilation;
no tests executed. `run-mw76zdlq`: 702 executed, 701 passed, only alternate
fixture admission failed. Existing 695 binary owner tests still passed.

Provider-preservation replan verification: `/home/ubu/.cache/lay/development/run-pk_qeii0/RESULT.json`,
7 selected/executed, 7 passed, 0 failed, 17.5 seconds. The alternate provider
was admitted normally without changing package validation; its synthetic known
right form has imported readings. The actual moved-prefix registry rule is
enabled/evaluated and returns no candidate, and DecisionCore retains no
BoundaryShift from that known right form. This tests a controlled provider
truth, not an invented runtime vocabulary exception.

The last production-veto binary panel already passed all 695 selected owner
tests in run-mw76zdlq; only integration fixture bytes changed afterward. No
functional code or configuration changed after those owner results. Full
canonical gates will discover the union again without altering old rows.

Independent explicit provider-preservation replan review 1: PASS, no blocking
issues in the declared delta. Reviewer `/root/boundary_scope_review`, production
diff SHA-256 `0fc49910932a4d3625e61603099702b0056a554997430bed5ec10bc8f546ec55`.
Original connected review pass 2 remains BLOCKED historically; the new review
closes that specific guard gap, without claiming atomic settlement, latency
or physical acceptance. Private combined review receipt:
`/home/ubu/.cache/lay/development/space-boundary-full-gate-20261007/CONNECTED_REVIEW.json`.

## Canonical sealing repair after graph refusal

Full-source attempt b1 stopped at the architecture graph gate after 67.6
seconds. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b1/RELEASE_GATE_RESULT.json`.
Discovery preserved all 3040 old rows exactly and admitted only the 17 declared
new identities; formatting and the initial canon check passed. No full Cargo
gate, MSRV gate, artifact export, installation or physical input ran.

The first failed contract is C05: the new recorded-boundary inverse issued and
attached its verifier receipt independently of the existing central sealer.
The graph guard requires one production issuance and one attachment site.
Before changing source, explicitly replan this bounded repair: move the
existing issuance body into one private ready-action sealer; both the ordinary
planner and the exact, validated inverse call that same sealer. Preserve
inverse validation, generic multiword refusal, authority matching and every
graph/canon assertion. Independent review and a fresh source gate must bind
the resulting production diff. Runtime authority remains unchanged.

Independent C05 repair review 1: PASS, reviewer
`/root/boundary_scope_review`, production diff SHA-256
`5969910bebe3b6ab4f81ffe90b5f95dd899f02a8d50c9b8c3d83dff1c2a6e95a`.
Ordinary planning retains the same safety and authority checks; the inverse
retains its exact forward receipt, text and physical-plan validation before
the shared sealer. This is static review only; graph and functional gates
remain pending. The earlier failed graph receipt is retained unchanged.

Attempt b2 stopped on formatting after 3.32 seconds, before discovery, graph
or functional checks. Only the `seal_ready_action` signature required the
canonical multiline layout. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b2/RELEASE_GATE_RESULT.json`.
The formatting-only correction changes no Rust tokens or behavior.

C05 replan review 2: PASS for that formatting-only delta, same independent
reviewer; final production diff SHA-256
`8da02e2dba8f49f0ecd4b0a00048ee0b14555d45f350b415a6ba5154e3d7ba26`.
Review 1's sealing conclusion remains valid. This closes the two-pass bounded
repair review; it does not promote either failed gate attempt to PASS.

## Full fixed-set gate b3: rejected before installation

Receipt:
`/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b3/RELEASE_GATE_RESULT.json`.
The complete source attempt took 524.8 seconds; its full-gate command took
443.1 seconds. Mandatory graph refresh, canon, formatting, audit50 and
discovery passed. Discovery preserved all 3040 old identities/rows and added
only the declared 17; the known-failure ledger remains zero.

The completed fixed set exposed 16 BoundaryShift semantic regressions
(14 library, 2 daemon), plus one historical protected-artifact identity
failure. All 695 IME owner and 7 new integration tests passed, but these
component passes cannot override the failing complete set. Verdict: rejected
for source promotion. Lints, MSRV, release artifact export, installation and
physical acceptance were not reached. Installed bytes and runtime authority
remain unchanged. Diagnose the first shared semantic transition and retain
the old artifact receipt; any successor must be explicit, independently
reviewed and bound to exact bytes rather than exempted from its guard.

## Explicit replan: preserve independent exact evidence

Completed b3 fixed set: 3031 selected/executed, 3014 passed, 17 failed.
Bounded identities and first-transition evidence:
`/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b3/FAILURE_PACKET.json`.
The shared semantic loss occurs both before candidate lattice admission and
in operator verification of supplied lossless candidates. The fixed corpus
measured clean 0/220 false applies, proposals 75/188 (39.89%), direct recovery
71/188 (37.77%). Its unchanged required gates are at least 98% proposal recall
and 80% direct recovery; neither may be lowered.

Static diagnosis: the exact accessor returns false immediately on warmed
provider `Some([])`, so it never consults an independently allowed exact
reference dictionary. An incomplete provider's absence is not contradictory
evidence against that other exact source. Before editing, replan the smallest
systemic repair: use nonempty cached readings as a positive OR, then retain
the existing policy-controlled exact dictionary check for both empty and
unavailable cached readings. FieldSnapshotOnly remains unable to admit cold
reference dictionaries. Keep exact reconstructed-target requirements and
provider-positive original-right preservation unchanged. Add one isolated
controlled regression proving that an incomplete warmed provider cannot erase
independent exact reference evidence, then rerun the complete affected fixed
families. This is a hypothesis for the b3 regressions until measured; do not
claim that every absent surface was instrumented in that run.

The separate artifact failure reflects intentionally changed protected source
bytes, not semantic success. Preserve the historical preflight unchanged;
record and independently review an exact source successor before changing its
identity guard. No installed bytes, physical input or runtime authority changed.

## Measured provider OR repair and explicit authority-scope replan

Focused receipt `/home/ubu/.cache/lay/development/run-rsoamg0n/RESULT.json`:
2789 selected/executed, 2773 passed, 16 failed; every semantic failure identity
and the corpus counts are unchanged from b3. The new isolated provider-OR
regression passed. Thus that independent-source repair is correct but
insufficient; the earlier causal hypothesis does not explain the full loss.
No installed bytes or physical fields changed.

First shared mechanism: exact imported surface membership was applied globally
to a broader existing form-generating candidate lattice and its generic
BoundaryShift verifier. This deletes grounded inflected targets before
competition and invalidates supplied lossless candidates, permitting lossy
competitors. The existing corpus and all its thresholds remain authoritative.

Before editing, explicitly replace that overbroad scope: restore the original
moved-prefix producer and generic transition verifier byte-for-byte. Keep the
new independently attested-target accessor and positive-source OR, but require
unknown observed right and exact reconstructed forms inside the shared
`plan_space_boundary_edit` permission factory, before sealing the pending Space.
This grants only the newly requested final-pair physical scope; it neither
selects a candidate nor reranks or runs another correction. Preserve the core
winner, one sealer, SafetyGate, exact owner witnesses and all output executors.

The newly introduced negative integration assertions had incorrectly demanded
absence from the general lattice. Their scope must explicitly become refusal
of the new physical pending-Space permission, with a controlled positive core
selection followed by a known-right or unsupported-target refusal. This is a
contract correction, not relaxation of the old fixed tests or quality gates.
Independent review must inspect both candidate retention and effect refusal.

The restored verifier remains under its immutable TD-113 source guard. Only
`text-edit-gate` needs a new exact successor: bind the old manifest/hash/mode,
new gate hash/mode, explicit decision and independent source review; preserve
the historical manifest and every other artifact guard.

Integration scope replan receipt
`/home/ubu/.cache/lay/development/run-dp_n__0_/RESULT.json`: all seven identities
selected/executed/passed, no failures; formatting passed, wrapper 39.5 seconds.
The known-right control retained a selected BoundaryShift with executable
ordinary semantic authority, but the pending-Space factory and actual pair
adapter refused it. Full fixed set and physical acceptance remain pending.

Before final connected review, make the already declared unknown-right
contract explicit: unavailable/overflowed cached provider readings cannot
prove absence. The new pair permission requires a bounded warmed provider
answer with zero readings as well as no independent exact right-form witness.
Both targets still require independent positive evidence. This is inside the
same shared HotField/factory path; no new cache, provider, worker or route.

Updated production static review PASS by `/root/boundary_scope_review`,
sorted `git diff --no-ext-diff --binary HEAD -- src` SHA-256
`1ecc1d2dfdfe7f4e9b905424069afc6a946b8893db27b4e27084e999a3b79d7c`.
Unknown-right requires available bounded empty cached readings; exact positive
reference evidence remains monotonic. The old producer/verifier are unchanged.
The separate exact gate successor is recorded in
`decisions/2026-10-07-space-boundary-authority-scope.json`; its guarded source
SHA-256 is `ef5c53b43df58adfdb0c42319db0930c216f53c07ec2da94e4ef8d66ca42fd31`.
Final combined test/artifact guard review and complete source gates are pending.

Final combined authority-scope review 2: PASS for static production, tests and
exact artifact guard, reviewer `/root/boundary_scope_review`, production diff
`1ecc1d2dfdfe7f4e9b905424069afc6a946b8893db27b4e27084e999a3b79d7c`.
Guard SHA-256 `a4e6af8ef3435df109a7a444ee9db29faf20aca020b0013e7af47fa28f06f7c8`;
integration SHA-256 `fcd54eabc6a99ac9b6f89323d38c3aab3f79ba776132db9752c6b5c8475991d2`.
The historical immutable TD-113 manifest and every other artifact guard stay
unchanged; only the exact gate successor is admitted. The review does not
certify fresh functional, installed or physical results. Private source-chain
and prospective install/matrix/rollback helpers also received static PASS.

## Explicit metadata repair after b4 canon refusal

Receipt `/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b4/RELEASE_GATE_RESULT.json`: stopped after 3.52 seconds at
CANON_PRECHECK because the new scope decision omitted the required nonempty
`verification` string. Formatting passed; discovery, graph, full tests, MSRV
and artifacts were not reached. This is a metadata defect, not a measured
functional regression. Installed bytes remain unchanged.

Before editing, explicitly replan this metadata-only repair: add the truthful
verification field describing static review, intermediate integration evidence
and still-pending gates; update only its exact ADR checksum in the existing
successor guard. Preserve production diff, guard logic, historical decision
manifest, prior review2 and all b4 failure evidence. Independent bounded
metadata review then permits a fresh immutable source snapshot.

## Explicit Clippy spelling repair after b5

b5 receipt `/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b5/RELEASE_GATE_RESULT.json` failed after 565.14 seconds.
The complete fixed set passed: 3032 selected/executed/passed, correctness2996
and package36, zero semantic and infrastructure failures. Exact preserved
summary `/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b5/FULL_TEST_SUMMARY.json`, SHA-256
`4e1a7093ded6b37d0d9f8146ae80726e075fb0a5e23d95c22ea265dd0bb83e92`.
Graph/canon/audit50 passed. Complete release verdict remains FAIL: default
Clippy rejected one `clippy::question_mark` spelling in the owned-prefix
witness check; MSRV and artifact export were not reached. Installed bytes
and physical acceptance are unchanged/pending.

Before editing, explicitly replan the single shared lint repair: replace
`let Some(before_left) = strip_suffix(...) else { return None; };` by the
equivalent `let before_left = strip_suffix(...)?;`. Preserve every refusal,
owner witness, return value and side effect; add no allow/suppression and
change no guard or threshold. This is one expression at one production
check, with independent bounded review and a fresh full source snapshot.
No new test identity is justified; existing owner tests exercise both results.

## Explicit detached-probe transport replan after b6

b6 receipt `/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b6/RELEASE_GATE_RESULT.json`
failed after 527.86 seconds: 3032 selected/executed, 3031 passed, one failed,
zero infrastructure failures. The exact preserved summary is
`/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b6/FULL_TEST_SUMMARY.json`,
SHA-256 `283d00eed871e307ec537f447fa45e51c123474f883ce02f5c9eaa0a01797c9e`.
Graph/canon/audit passed; lints, MSRV and artifact export were not reached.
No installation occurred and the zero known-failure ledger is unchanged.

Independent read-only diagnosis identified the first failed transition in the
test transport, after a handled ManagedCommit probe: the fixture consumed a
raw next packet expecting UnknownObject Error, but observed Signal. Its exact
member/body was not captured, so that packet is not declared a valid commit.
The two handled probe sites run production processing and callback observation
concurrently with the detached zbus dispatcher; their relative reply order is
not guaranteed. All reducer, lineage and authority assertions before that
transport read passed. The equivalent lint spelling has no causal path here:
the probe tail has no separator and exits pair capture before that expression.

Before editing, explicitly replan only these two test probe consumers: preserve
all exact property, CommitText, path, body and no-extra-text-effect assertions;
use the existing exact registered-UnknownObject filter, then its strict
dispatcher Ping and empty reply-count assertion. Do not add a skip loop, queue,
runtime change, failure waiver or retry-until-green. Extend the existing replay
test with controlled real-socket Signal-before-Error and Error-before-Signal
orders. A second MessageStream may observe the latter Error without consuming
the primary stream. This proves the transport repair and must deterministically
reject the former raw-Error-first consumer; it does not prove physical output.
Independent bounded review and fresh complete gates remain required.

Focused IME receipt `/home/ubu/.cache/lay/development/run-ir2ad_i6/RESULT.json`,
SHA-256 `e73d354622b97d41b80b4d727591463e1489260f10bf7c7b1ba76be536e66452`:
all 695 existing IME identities selected/executed/passed, zero failures;
formatting passed, wrapper 90.8 seconds. Controlled negative receipt
`/home/ubu/.cache/lay/development/space-boundary-harness-negative-fv70822o/RESULT.json`,
SHA-256 `2edb7be16dfc33c21b4e4e3d74bde97107d6ad1c85cfb4f8e372e1dd940ae2ae`:
one exact existing identity selected/executed/failed as expected after restoring
only the premature raw reply consumer in a separate immutable test snapshot.
It deterministically failed Signal versus Error at the original assertion,
before commit validation; exact test execution 0.0187 seconds, guarded wrapper
45.20 seconds. All 695 discovered identities and 1497 file/mode bindings were
verified. This is a causal transport-regression proof, not a functional PASS
for the negative snapshot. The full release and physical-client gates remain
pending; original b6 failure and runtime bytes are unchanged.

## b7 source release and first installed QA attempt

Full receipt `/home/ubu/.cache/lay/development/space-boundary-full-frozen-20261007-b7/RELEASE_GATE_RESULT.json`
is PASS_FULL_SOURCE_RELEASE_GATE: all 17 commands passed, all 3032 selected
tests passed (2996 correctness, 36 package), zero known or infrastructure
failures, both Rust MSRV profiles passed. The 1497-file frozen source is bound
by `FETCHED/FINAL_SOURCE.json` SHA-256
`d5047a73b474fb6c6677c83f0a813241f763ea9b72ff8b7dfe4ccdaeb87cf6e1`.
Only the ten approved graph/manifest/ledger exports were imported. Total gate
903.75 seconds; full-gate phase 783.76 seconds. Earlier failures remain failures.

Installation receipt `/home/ubu/.cache/lay/development/space-boundary-physical-20261007/install-758eea57cb89/INSTALL.json`
is PASS. Installed and loaded IME SHA-256 is
`b341c34d601eede8e5e415d9bb392f650f028ae3a5db28ffea03878bec7a10f7`;
8154640 bytes, version 1.0.80. The owned empty GTK installation preserved
daemon, L3, L1.1 and global IBus PIDs, sources and configuration; only the IME
channel was reloaded. Runtime now contains the new final-pair permission;
decision and output ownership remain with their existing actors. The pinned
scope decision records the earlier static, uninstalled state. This later
evidence annotation does not change the compiled production source.

First physical pilot receipt
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/pilot-f41fcca69350/windows/WINDOW_MATRIX_LIVE.json`
is FAIL_OR_BLOCKED: zero executed cases, no text input. The prospective wrapper
omitted the `wide_matrix_v2.print` binding normally supplied by the historical
outer launcher. It failed at the setup-report call before opening a test field.
The virtual keyboard was removed, the same-byte daemon was detached/restarted,
the original source was restored, global IBus and IME PID/bytes were preserved.
Physical correction and client acceptance remain untested.

Explicit QA-wrapper replan before editing: bind the standard `print` function
in the new outer wrapper, leaving all frozen historical observers, assertions,
input routes and cleanup unchanged. Retain this failed pilot; independently
review the single helper change and execute a fresh pilot with fresh receipts.
No production repair or repeated source build is justified by this helper defect.

## Actual physical matrix, failed promotion and rollback

The reviewed one-line QA-wrapper repair passed a fresh real GTK3-entry pilot:
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/pilot2-d6074d44b3f9/windows/WIDE_MATRIX_V2_20261007T225957.json`,
SHA-256 `b2fd2aefdca463f1f8ae635dcb3bfb53e61a146294c880b985aa0b2ba08cde0c`.
One case executed/passed: first Space preserved `они должн `; closing Space
corrected the actual pair to `они должны быть `; real Double Shift restored
`они должн ыбыть `. This establishes only that one physical case.

The full finite matrix attempted all 26 declared field routes using unchanged
frozen observers, real uinput events and independent client observations:
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/windows2-a86679a30071/windows/WIDE_MATRIX_V2_20261007T233033.json`,
SHA-256 `3a8908fc6fc98969e3bdedf4400ba11d11db97e5c297ae1037ff5c15f05365de`.
Verdict FAIL_OR_BLOCKED: 755 recorded rows; 688 executed, 598 passed, 90 failed;
67 blocked, no fatal error, 759.743 seconds. Row counts are distinct from the
multiple physical keys/pairs within a case. The case filter covers Tab,
held Shift, fast Double Shift, alternating words, ordinary autocorrection
and the eight new pair cases. It does not certify hint absence or every
possible user window. Unavailable observations are not PASS.

| Pair case | Passed / executed | Blocked |
| --- | ---: | ---: |
| Forward correction | 2 / 14 | 3 |
| Exact recorded inverse | 1 / 14 | 3 |
| Correction followed by next word | 1 / 14 | 3 |
| Preserve known right word | 14 / 14 | 3 |
| Refuse invalid reconstructed left | 14 / 14 | 3 |
| Actual full-provider `расчет ыготовы` | 0 / 14 | 3 |
| Focus roundtrip then correction | 1 / 9 | 8 |
| Eight pair toggles and next decoders/icon | 0 / 9 | 8 |

The forward/inverse mechanism is physically possible but unstable. Positive
results do not erase failures. Firefox/GOST/GTK/Qt fields commonly retained
the unchanged pair plus one Space; some original autocorrection and alternating
word cases also failed. No causal claim that the new source caused every old
case failure is established without an exact-byte predecessor comparison.
Native first-pair mirror zero remains unsupported without an actual start
witness; positive fixtures supply the earlier observed separator.

Independent evidence gaps are retained. Chrome startup failed before IME
input: pthread_create EAGAIN, renderer zygote failure and four pids-limit hits
in the original 128-task test scope. Its ordinary desktop main process migrated
outside the agent slice, so that process was not CPU/RAM constrained by the
wrapper. Only the task-created empty bootstrap owner was terminated after exact
PID/start/profile/unique-argv and no foreign-page checks. Other missing routes
include Tor owner binding, unavailable native preedit observations, the GNOME
terminal ready observer, gedit discovery bounds and a changed LibreOffice
executable binding. Browser/focus-observer failures are separate from core
quality or physical correction failures; none receive a waiver.

The matrix process's actual cgroup was verified at a 1792 MiB ceiling under the
shared 720-percent CPU / 6 GiB agent slice; its task-limit events were zero at
that observation. The server was independently bounded at 128 MiB. The test
keyboard was closed, the same-byte daemon detached/restarted, the original
source restored and global IBus preserved. IME PID 62782/starttick 302405151
and candidate SHA-256 stayed identical throughout the matrix. These facts are
bound by `windows/SPACE_BOUNDARY_PHYSICAL_BINDING.json`, SHA-256
`8a5ab8ab583b3221a7cd1ab180967a16410693c1f22010d93e6df4558c95a9b2`.

Existing runtime logging was already enabled; no trace setting was changed.
Original browser failure snapshots contain no copied callback logs, so their
first failed internal transition is UNKNOWN. A later bounded read-only sampler
captured existing GTK/Qt/terminal trace deltas during the same matrix:
`windows2-a86679a30071/LIVE_QA_TRACE_SAMPLES.jsonl`, 21219997 bytes, 127 samples,
zero missing overlaps, SHA-256
`4cc1cbe7bd50b045bcc7cd44e7453a48fb5e47c0173b83d9de88f84ad3f23f40`.
Capture times are not invented event timestamps. Some exact frames exhaust
the existing approximately 3.5 ms Space lookup while full evaluation finishes
tens of milliseconds later and is superseded; applicable and semantic-refusal
frames must be distinguished. A logged lease outcome `ready` also covers
NoApply and does not by itself prove an applicable action. Pair-scope presence
is not explicitly traced. Diagnosis remains pending; no deadline, ownership,
SafetyGate or ranking rule has been relaxed.

Failed physical promotion was rolled back in a fresh witnessed empty GTK field:
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/windows2-a86679a30071/ROLLBACK.json`,
SHA-256 `4b790017e81cfb6b46ae767396f76e5aff4cf2d6ab93f491e5aefc39b639c6af`.
Verdict PASS: installed/loaded IME restored to exact predecessor SHA-256
`03a8ef6023e15afcf315bfe4c8ec941550dad4a498f4a9163bcaf4194f82dc6a`,
IME PID 412339. Only the IME channel reloaded; current daemon/L3/L1.1/global
IBus PIDs, configuration and source list were preserved. Runtime authority
therefore returned to the predecessor. The b7 source release PASS remains
valid prior source evidence; it is not successful physical promotion. Production
source is unchanged since b7. A remote metadata-only graph/binder/canon refresh
is prepared but has not run; implementation repair requires a separately
recorded causal replan and fresh evidence.

Independent decoded-stream review is retained at
`windows2-a86679a30071/READONLY_TRACE_REVIEW.json`, SHA-256
`7258eb033133134af7cadb1688c5760e357168426bc92db531d4b99299cdb9d0`.
Of 22 reconstructed positive closing-Space streams, 14 had a prepared
Rank/SuggestOnly refusal, two missed the lookup deadline and later produced
same-identity Apply as Superseded, one had a prepared adapter Verifier refusal,
and five logged application. Six invalid-left negative streams are excluded.
This sampled denominator has zero exact joins to physical case identities;
it cannot replace the window matrix or attribute a sampled cause to a row.
The remaining gap is the actual bound logical request, pair provenance and
selected/retained BoundaryShift candidate at those refusals.

Before changing production behavior, replan a bounded diagnostic extension
within the existing opt-in correction telemetry: capture only fingerprints,
scalar counts, pair presence/provenance/revision identities, existing selected
class/target and a fixed-size projection of existing BoundaryShift candidates
and their gate/evidence tags. Keep user text out of new metadata. Read only
already owned inputs and already computed decisions; do not repeat decisions,
query or warm providers, add polling/RPCs, change short-circuit checks, deadlines,
rank weights, the pinned pair factory or any delivery/authority owner. Diagnostic
hashes join observations, never issue an authority certificate. Validate that
logging on/off leaves decisions and effects identical and that absent pair,
retained-but-refused candidate and late applicable result remain distinct.
The extension is still unimplemented; it needs independent review, fresh source
gates and exact-byte installed diagnostic evidence before a causal repair.

Graph-only helper static review 1, retained outside the source checkout at
`/home/ubu/.cache/lay/development/space-boundary-graph-refresh-20261007/GRAPH_ONLY_REVIEW1.json`
(SHA-256 `84c5839503bc1d9de6c25fec6ae64be92865fbb4ed4369be332288fd7eca1aea`),
is FAIL before execution: transferred helper hashes were checked after importing
common.py, and import rollback copied directly into live paths. Explicitly
replan only these two private transport repairs: verify helper bytes in the
bootstrap before importing either helper, and restore backups through the same
temporary-file/os.replace primitive as forward import, preserving modes and
cleaning all temporary files. Preserve failed review 1 and require independent
review 2 before any freeze/remote lease. No project guard is weakened.

Private graph-only review 2 is static PASS, retained as
`/home/ubu/.cache/lay/development/space-boundary-graph-refresh-20261007/GRAPH_ONLY_REVIEW2.json`,
SHA-256 `67fc643a53b0b87f28d54f102a86cad9f6b5c810d2c3b6efdf26d84ed90bacae`.
Reviewed helper SHA-256
`fb5e1747fc6da881a888ddc0466884e08608eef4d8aff620d169ada05a38219b`.
Both exact pre-import helper checks and atomic rollback/mode/temp cleanup are
present; all 1497 members, eight graph outputs, manifest and ledger retain
their existing checks. Review 1 remains FAIL. The root now authorizes the sole
guarded remote metadata-only graph/binder/canon refresh on this unchanged
production source plus the evidence annotation. This is not a functional build,
installed-byte certification or physical acceptance.

Post-QA graph-only refresh completed as PASS_GRAPH_REFRESH_ONLY in 62.99 s:
`/home/ubu/.cache/lay/development/space-boundary-graph-frozen-20261007-post-qa/RESULT.json`,
SHA-256 `402930dac41b20944c004ef6203b246b5e68607883d2e1b57ce2e53d2d13e223`.
The unchanged architecture-refresh script and both canon checks ran remotely;
no Cargo, installation or physical actions ran. Exact post-refresh 1497-member
source closure is `FETCHED/FINAL_SOURCE.json`, SHA-256
`1e075be277363bf15ec531abfa1d92a152cc5adc6b0be2cefd40b316588909fe`.
Atomic import receipt `IMPORT_RESULT.json`, SHA-256
`403fce1c3fb1366504cb24aaada08bd94daed6498404a8d05a3d90ff0682e686`,
records exactly eight graph outputs (six changed); every live member/hash/mode
matched that closure. The manifest and zero-failure ledger remain exact b7
bytes. This result closes the graph-only experiment, not physical promotion.

The diagnostic replan now has a source implementation, pending independent
review and remote verification. It extends only `ime_correction.rs`, existing
`trace.rs` and two existing test identities in `space_boundary_shift_mechanism.rs`
and `trace.rs`. No members, test identities, modes or runtime configuration are
added. The shared decision still executes once. Only when debug_action_log is
enabled, its already computed result yields input/pair/selected fingerprints
and scalar counts, selected class, total retained BoundaryShift count and at
most four candidate metadata records (target fingerprint/count, gate tags,
authority-evidence kind). The trace serializes existing pair provenance and
its cursor/revision/witness/floor, plus publication outcome and frame identity.
An explicit omitted count prevents treating four records as the complete pool.
New fields contain no raw user text. Hashes are joins, never authority proofs.
The pinned pair factory, candidate producer/verifier, short-circuit conditions,
provider calls, deadline, queue and delivery paths are unchanged.

Design comparison: the baseline bounded log cannot identify pair presence or
candidate retention at the sampled Rank/Verifier refusals. The chosen extension
reads the existing result and fixed-size metadata; a separate diagnostic
evaluation would add a second decision/provider owner and race, and is rejected.
Raw candidate-text logging is rejected because counts/fingerprints suffice for
controlled fixture joins and preserve the metadata privacy contract.

Consequence scope: retention, ranking, false authority, learning, rollback and
output compatibility retain their existing owners and guards. No cache, package,
model, worker or RPC is added. Opt-in projection scans the existing bounded
candidate pool and hashes at most four boundary targets plus the selected target,
input and pair; it does not clone candidate strings. Metadata grows by a fixed
four-record array, and JSON allocations/trace I/O remain under the existing
bounded logger. CPU/latency overhead is an unmeasured risk when debugging is on,
particularly with the existing 3.5 ms lookup; no speedup is claimed. Exact-byte
source gates and later installed diagnosis must measure its practical scope.
Tests compare logging-off/on surfaces, complete EditAction plans/safety/receipts
and refusal stages for the fixed mechanism fixtures, and assert required trace
fields, absent/Client/OwnedPreedit distinctions, prepared/superseded outcomes and
absence of raw fixture text. They extend existing identities rather than add a
parallel simulated implementation. A controlled missing-metadata violation is
planned remotely. This is diagnostic work, not a correction fix or acceptance.

Diagnostic focused attempt 1 is FAIL before test discovery/execution. Static
source review1 PASS is retained at
`/home/ubu/.cache/lay/development/space-boundary-diagnostic-review-20261008/DIAGNOSTIC_REVIEW1.json`,
SHA-256 `dddec1f545f0812d35628549ebfe869796468abbbb0f30f8839d3c50711fc4fc`;
private full-helper review1 is separately static PASS, SHA-256
`4a212e40b51443ca972e1829fcf232cf1e65e2b4bc30046978f5be8de5c241ac`.
Neither establishes functional acceptance. Actual unchanged remote dev-check
with lib:lay, bin:lay-ibus-engine and test:space_boundary_shift_mechanism is
`/home/ubu/.cache/lay/development/run-eil5gs5w/RESULT.json`, SHA-256 `535380e8d48b7123bf43e54d0d3adc3525105e3aaf6de7fb3ba99e87d2524e12`.
FMT passed in 2.85 s; the focused command failed in 26.89 s; worker total was
29.87 s and transport total 38.1 s. No identities executed; expected selection
is 2519 (1817 library, 695 IME, seven integration), not a zero-test PASS.
The exact rustc diagnostic is retained as `BOUNDED_COMPILER_DIAGNOSTIC.json`,
SHA-256 `c590d78c939641448415bc01d368f4df475885425707b5770e8874e2be5750cf`: recursion limit reached
while expanding serde_json json_internal at trace.rs:605. One compiler error
affects the bin and bin-test; secondary abort messages are not extra errors.

Before editing, explicitly replan the single diagnostic serializer repair:
split the oversized JSON macro into two fixed object literals and merge their
owned maps, preserving every existing/new top-level field. Keep the compiler
recursion setting, tests, schemas and all authority/output paths unchanged.
The extra fixed map allocation remains opt-in logging work after publication;
latency remains unmeasured. Preserve attempt 1 and require independent review2
and a fresh immutable focused attempt; do not retry full or install.

The single serializer repair preserves all 43 unique top-level key/value
expressions without changing recursion limits, tests or production decisions.
Independent narrow review2 is static PASS at
`/home/ubu/.cache/lay/development/space-boundary-diagnostic-review-20261008/DIAGNOSTIC_REPAIR_REVIEW2.json`,
SHA-256 `b0e13a25d6071f0f4dc3f80ff8730b9fdd990026ba4603adf4f1dfe545e495da`.
Reviewed trace SHA-256 is
`fe1c758e7672e2364ba5506684b021b08185be80e26964522892d810df60422d`;
sorted production diff SHA-256 is
`80d345fa2c2805d8f97305ded66f0fe3434f8f0ae55d0842a0d27c033f89408b`.
The compiler failure and source review1 remain immutable historical evidence.

Fresh remote focused attempt 2 is PASS: 2519 selected/executed/passed, zero
failures (1817 library, 695 IME, seven integration). Existing identities,
lane/kind/isolation metadata and every 1497 member/hash/mode independently
match the frozen reviewed source; no test identities were added or waived.
Discovery contains 2543 total rows; the selected correctness/package subset
contains 2519. Excluded rows retain their existing lanes and metadata.
`/home/ubu/.cache/lay/development/run-st3ipjuj/RESULT.json`, SHA-256
`c05349274232dfc65396b1969fe907f905d75082487f48924cbdcc6e3399f06c`;
`request.json`, SHA-256
`db2e69faf9140bc53503509eb9142b6f56d8586e715bbcf521d6e66e7284340d`;
`IDENTITY_VERIFICATION.json`, SHA-256
`82d3794e8fc65b90cb7d3c56f8598c47726e39684445b3390c26cf81f8e8cd66`.
Worker time 180.8453 s; transport total 188.8901 s. Source test scope only:
logging on/off preserves the fixture decisions and complete edit actions;
this does not measure actual debug-on client latency or physical acceptance.

The separately reviewed controlled violation removes only the
pair_scope_present serializer entry in an isolated snapshot. Negative-helper
static PASS receipt is
`/home/ubu/.cache/lay/development/space-boundary-diagnostic-review-20261008/NEGATIVE_HELPER_REVIEW1.json`,
SHA-256 `d92881c1f6ef7ede09074cb6ed96d4ad420898b08a35e12799b5ecf679ed2ac5`.
Execution result is EXPECTED_CONTROLLED_VIOLATION: one selected/executed test,
zero passed, one failed; all 695 selected IME identities and all 698 total
IME discovery rows remain exact. The
existing trace required-field test reports precisely `correction metadata is
missing pair_scope_present`, with no compilation or infrastructure substitute.
`/home/ubu/.cache/lay/development/boundary-metadata-negative-frozen-20261008-n1/RESULT.json`,
SHA-256 `f1b773492ce02a75bba8620fe4f3b3dc3c83bbcf2639ef6b52606ce680cb29fc`;
`REQUEST.json`, SHA-256
`82e1290691c717e02db76baaa9203019cc9f0cee8f637e2be8db8bab70cd886e`;
patch SHA-256
`d635fc604c80711dd57525e166c85f1f7f2fe0a1d451d30c536a760bc1cdf32b`;
`FETCHED/EXACT_TEST.log`, SHA-256
`f01dd374a54bed9eafcb2fa23229d3bdf5bc44864c6de01efe54dd579ff0608a`.
Worker time 46.3068 s; exact test harness 1.4376 s. This proves the diagnostic
test detects a controlled missing field, not the historical correction defect.
Authoritative source and runtime were not changed by the negative execution.

Next promotion requires final review2 binding of these document-only measured
annotations and the privately extended full-helper evidence contract, followed
by all original 17 release commands / 3032 selected tests. No further code
repair is admitted under this two-pass plan. A fresh four-route physical
diagnostic fixture directory is prepared but not installed/executed:
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/diagnostic-d1-e878d34775d2`.
Its 24 observers and reviewed install/rollback/outer matrix bytes are unchanged.
Runtime authority remains with the rolled-back predecessor. The actual
mechanism repair and all-window physical acceptance remain pending.

## Measured diagnostic release and native run — 2026-10-08

The diagnostic source received final static review2 PASS, bound by
`/home/ubu/.cache/lay/development/space-boundary-diagnostic-review-20261008/DIAGNOSTIC_REVIEW2.json`
(SHA-256 `c26730d589e1f28fac652783c821da1e08a7127ff0e55966b53f0b92269fefee`).
The successor full-helper review2 is PASS at the same directory's
`FULL_HELPER_REVIEW2.json`, SHA-256
`d41319de143572fcc0cd9336449a6a6d8c44941bee58be8c50be99e674c312be`.

Remote full result: PASS_FULL_SOURCE_RELEASE_GATE, all 17 commands and
3032/3032 selected/executed/passed tests, zero semantic/infrastructure failures.
Correctness 2996 and package 36 remain separate; all 3058 baseline manifest
rows, exclusions, identities and ledger are unchanged. Worker 904.83 s,
FULL_GATE 787.23 s. The exact receipt is
`/home/ubu/.cache/lay/development/space-boundary-diagnostic-full-frozen-20261008-d1/FETCHED/RELEASE_GATE_RESULT.json`,
SHA-256 `1fa462a183d6f99ee6d2709a936124ab96b648c1de09d8f70e20e3edd7298e45`.
Its `FINAL_SOURCE.json` SHA-256 is
`3c6260bc2c425f6bdcfce2597292839b60e31cfb6b482d800768efd5bf0c260d`;
all 1497 source members/modes matched after the approved atomic imports.
`FULL_TEST_SUMMARY.json` SHA-256 is
`6448f1f14d33adb7025f0f13bdf79e81ac6fb77e274339cfc1a42a4c45424b10`.
This source gate does not establish physical acceptance.

The source-bound diagnostic IME, SHA-256
`61c948fb0816d68521573c423ef263b1f768eeaecd71af94c1ae5d4deb92bf8c`,
8168592 bytes, default + research-tools, was installed in a fresh owned empty
GTK3 field. Installation retained daemon/L3/L1.1/global IBus owners, source
list and configuration; only the IME channel reloaded. Artifact mode 0775 and
the preserved installed mode 0755 are distinct. Installation receipt:
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/diagnostic-d1-e878d34775d2/INSTALL.json`,
SHA-256 `90d5a3d3856f3b564ac8fa9e9a89952c0ad28c68ff5590694b61fc165937fdbf`.

The prospective scope was expanded from four to five native routes before
input, adding the prior positive GTK4 multiline contrast. Five routes
(GTK3 entry, GTK4 entry/multiline, Qt entry, Kitty readline) attempted the
same eight existing boundary cases: 40 declared, 38 executed, 12 PASS,
26 FAIL, two BLOCKED, no fatal error, 136.395 s. Forward 0/5, inverse 1/5,
next-word 0/5, clean-right preservation 5/5, invalid-left refusal 5/5,
actual full-provider calc 0/5; focus return 1/4 executed with one blocked,
eight-pair decoder/icon 0/4 executed with one blocked. These are diagnostic
observations, not a predecessor comparison or an accepted version.

Receipt:
`/home/ubu/.cache/lay/development/space-boundary-physical-20261007/diagnostic-d1-e878d34775d2/windows/WIDE_MATRIX_V2_20261008T010052.json`,
SHA-256 `9d62da76b9be9635512d976c5f79538eeb07c21b7c3818256b8b1d11f5086b36`.
Its `SPACE_BOUNDARY_PHYSICAL_BINDING.json` binds IME PID 822894/starttick
303163631 before and after, all 24 frozen observer hashes, finite uinput and
the exact install receipt. The actual matrix process PID 824216 was under
the shared guarded slice: job memory 1792 MiB, TasksMax 128, parent CPU 720%
and memory 6144 MiB; initial memory/pid limit events were zero. The physical
harness restarted the same-byte daemon to attach/detach its owned device;
daemon PID continuity is not claimed across physical QA. No global IBus
restart, source migration or model/configuration replacement occurred.

First bounded evidence: GTK3/GTK4 entry forward snapshots retain the exact
BoundaryShift target fingerprint 11642176497644798718, with an eligible
proposal and an owned-preedit pair of 11 scalars. The full input fingerprint
892355790400007033 has a prepared result and matching ready Space lease.
Nevertheless no candidate is selected. The scoreboard's 0 Apply / 50 Suggest
does **not** prove the Boundary proposal was downgraded: that scoreboard
counts unselected eligible candidates as suggestions. The first inner
admission/ordering rejection remains UNKNOWN. The calc snapshot separately
retains/selects its Boundary target but is superseded after a matching
not-ready Space lease; final adapter decision is absent. These distinct
failures cannot be collapsed into delivery failure or timing alone.
Rolling snapshot membership does not by itself establish exact per-case
event identity; source/frame fingerprints and lease identities establish
the narrower joins above. Debug-on latency and baseline regression causality
remain unmeasured.

The diagnostic was rolled back in a fresh owned empty field. Exact predecessor
SHA-256 `03a8ef6023e15afcf315bfe4c8ec941550dad4a498f4a9163bcaf4194f82dc6a`
is installed/loaded, IME PID 868896. Rollback retained then-current daemon
858552, L3 1206382, L1.1 3296699 and global IBus 270775, sources/configuration.
`diagnostic-d1-e878d34775d2/ROLLBACK.json` SHA-256 is
`804c661971d125db663eb44fc5680ab769efd1c79df66d67a5d0022b2f93c900`.
`ROLLBACK_LAUNCHER_REVIEW1.json` is static PASS, SHA-256
`c7614df9ad343cf9671ad5d1b21b0373b4ebe050da444e74786609c76215389e`.
Its fresh-empty wrapper witness is distinct from the unchanged inner rollback,
which does not consume that witness receipt. Runtime authority is again the
predecessor; functional repair and all 26 window routes remain pending.

Next bounded causal replan compares three options: change admission now
(rejected without its actual rejection reason); add further runtime metadata
(requires another source gate and physical version); or use existing debug
rejection output in one remote, source-bound production-library replay with
fixed mechanism cases and explicit provider/config/frame identities. Prepare
the third first. The historical canonical-only six-test panel and synthetic
membership fixture cannot stand in for the live 50-candidate environment.
No parameter search, fitting, deadline change, second decision worker,
SafetyGate/verifier weakening or installation is authorized by a replay result.
An actual mechanism repair requires the first demonstrated shared transition,
independent review, controlled regression, fixed proof and full release gates.

Independent read-only causal review now binds all five forward requests and
all five calc requests by exact decoded stream, input/pair/target fingerprints
and Space lease identity. Eight GTK/Qt requests also have a closing-Space
callback clock between the saved pre-Space phase and client snapshot; the two
Kitty requests lack that independent physical clock witness and retain UNKNOWN
time attribution. Five forward cases are Ready with no selected candidate.
Four calc cases are Ready with a selected BoundaryShift receipt and an explicit
adapter `verifier` no-apply stage; one GTK3 calc case is NotReady (3436 us),
then Superseded. The four Ready calc refusals establish a separate adapter
construction failure even without a deadline miss. Inner admission and strict
pair-factory refusal subreasons remain UNKNOWN.

Exact review receipt:
`/home/ubu/.cache/lay/development/space-boundary-diagnostic-review-20261008/DIAGNOSTIC_D1_CAUSAL_REVIEW.json`,
SHA-256 `0e259ee7facb81939aa20f6e44892fd770e996f3f661506a61c2c5ffc93b0342`.
The separate all-window setup audit accounts for all 67 historical blocked
rows in seven mechanisms, without accepting or editing their observers:
`QA_CAUSAL_REPLAN.json` in the same review directory, SHA-256
`26f1adbc04cec8d82e3c4ffa10bf5cd3fbaef0c9aad1d409ea003dc29e1d3bfd`.
Stale executable pins, unavailable composition observations, focus return,
startup and discovery limitations require prospective reviewed QA repairs;
none explain away the measured native semantic/adapter failures.

## User-directed cleanup and return to the rule — 2026-10-08

The user rejected the expanded L1–L4 replay stand and explicitly requested
cleanup before returning to the original Space-boundary idea. The replay
stager failed locally on duplicate `check` arguments before remote directory
creation, transfer, build, service startup or any of its eight selectors.
Zero selectors executed; no retry was made. Its preserved failure receipt is
`/home/ubu/.cache/lay/development/space-boundary-causal-frozen-20261008-p1/STAGE_FAILURE.json`,
SHA-256 `64d85488bea8ac57a4a5a881fd261974e2ad2c37ded500bf24f05cd1ca806b11`.
The preceding replay plan is superseded, not an active prerequisite.

Cleanup restores `src/ime_correction.rs`,
`src/bin/lay_ibus_engine/trace.rs` and
`tests/space_boundary_shift_mechanism.rs` byte-for-byte to the frozen b7
source. This removes only the later diagnostic extension and its assertions;
the connected pair-edit implementation and existing diagnostics remain.
The disposable 25-file environment copy, its two archives and unused probe
helpers/crates are removed: 1,110,022,816 bytes. Frozen release/physical
evidence, preparation/review/failure receipts and small removed-source
backups are retained. Installed binaries, original models and configuration
are untouched. Remote inspection found no leftover d1/causal jobs, private
model services or causal replay directory. The exact cleanup receipt is
`/home/ubu/.cache/lay/development/space-boundary-cleanup-20261008/CLEANUP.json`.

Next implementation work is limited to the existing `moved_prefix_pair`
producer, its admission in DecisionCore and the existing exact-pair adapter.
The five Ready forward requests with no selected candidate and four Ready
calc adapter refusals remain measured failures; their inner subreasons remain
UNKNOWN. No new replay environment, extra correction worker, literal-word
exception, changed delivery route or weakened verifier/SafetyGate follows
from this replan. The rule must run once within the existing Space operation,
after ordinary correction has had its opportunity. Final changed-runtime
acceptance still requires the user-requested window checks.

## Final attested rule inside DecisionCore — 2026-10-08

The user explicitly requires continuation until the feature is ready and
forbids delegated agents. Root therefore owns implementation, remote execution,
source review and physical checks; no independent-agent review is claimed for
this slice. Completed historical reviews retain their original scope.

Baseline: b7's connected pair executor remains in place. Five native Ready
requests retained the exact eligible BoundaryShift but selected no candidate;
four Ready requests selected a different fixture's BoundaryShift and failed
adapter construction. Exact old inner rejection subreasons remain UNKNOWN.
This change implements the user's requested final rule rather than inventing
an unmeasured explanation for those rejections.

Chosen change: after ordinary DecisionCore ordering, and only when retained
exact-layout authority is absent, inspect the existing lattice for
the one eligible, verifier-proven right-to-left single-scalar boundary shift.
Require one closing ASCII Space, plain Cyrillic final words, an available
unknown-right witness, independently attested reconstructed words and the
existing correction-safety policy. Original left-word validity is irrelevant.
Conflicting candidate authority, duplicate owners, invalid L4 certificates,
exact negative memory or a valid witnessed L4 rejection still refuse it.
Use the same candidate evaluation, semantic receipt, pair permission, sealer,
prefetch lease and physical executor. Cold/failed/overflowed dictionary reads
remain unknown and cannot authorize the rule. No provider warmup, extra RPC,
worker, deadline, score weight or delivery-route change is introduced.

Alternative: change broad field admission/calibration for every boundary.
Rejected: unnecessary to the final attested rule and expands the old fixed-set
regression surface. Alternative: a separate after-Space callback or correction
worker. Rejected: creates an additional owner and can race the existing edit.
The chosen additional positive vocabulary evidence stays in C02's one chooser;
C03/C05/C08 and the unchanged verifier/SafetyGate remain mandatory.

Proof plan: extend the existing selected-boundary integration identity from
its historical two-winner subset to all four existing mechanism fixtures.
Run that unchanged identity against b7 as an old-source negative control,
then the actual corrected source and affected contracts remotely under the
existing guards. Preserve every old test identity and known-failure rule.
Canonical full gates, source-bound artifact installation and real-window
acceptance follow; a focused or synthetic-vocabulary pass cannot certify the
installed feature. No source, runtime or physical PASS is claimed yet.

### Old-source negative control and final ordering refinement

The unchanged b7 DecisionCore with the extended existing integration identity
executed seven tests: six PASS, one FAIL. The failing identity retained the
right BoundaryShift in both failing cases, but selected a MissingLetter
letter insertion for one and abstained for the other. Exact cached canonical
readings confirmed both target words and absence of each malformed right word.
This is an old-source failure proof, not a new-source verdict. Receipt:
`/home/ubu/.cache/lay/development/run-4x3710hf/RESULT.json`; elapsed 43.0 seconds.

Consequently the final rule also takes precedence over an ordinary selected
letter mutation when the boundary target preserves the complete letter stream.
A selected frame-bound lexical authority or existing lossless operation retains
precedence. Retained exact-layout valid/invalid handling remains unchanged.
The reconciliation issues one receipt after the choice; no edit is executed
before it and no second chooser or callback is introduced.

The first focused run of the new source stopped during test compilation:
seven integration checks and library checks were not executed. A failure-only
test diagnostic moved a non-Copy gate out of a borrowed candidate; it now
borrows that gate. Functional rule bytes were unchanged by this repair. Receipt:
`/home/ubu/.cache/lay/development/run-6bq8hhzl/RESULT.json`, 38.4 seconds.
This infrastructure failure is preserved and is not a feature PASS.

### Cached canonical lexical evidence repair

The first compiled new-source focused proof executed 1824 identities: 1823
PASS and one FAIL. All 1817 library identities passed. The remaining boundary
fixture stayed SuggestOnly with `edit_transition_not_verified`, despite both
reconstructed surfaces having imported canonical bindings and the malformed
right surface having none. Receipt:
`/home/ubu/.cache/lay/development/run-y4c9znkl/RESULT.json`, 128.0 seconds.

First failed transition: the existing verifier's lexical-mass consumer reads
HotField's exact word authority, which did not consult the already warmed
canonical bindings. The final rule correctly refused unverified evidence.
Repair that missing positive evidence once at `HotFieldSnapshot::word_readout`:
OR existing common/hot surface authority with an exact cached canonical form
binding. Use the validated binding index for one existential read, allocating
no candidate/reading vectors, warming no provider and making no RPC. Cold or
failed providers remain unavailable. Existing morphology/reference paths stay
intact; no global strict-membership replacement is introduced. The generic
verifier, SafetyGate, pair factory and candidate producer remain unchanged.
The existing isolated warm-provider unit now proves cold unavailability, exact
positive surface/input authority and unknown malformed-right preservation.

This additive authority repair broadens the consumer dependency closure and
requires the entire unchanged fixed correctness/package proof after focused
checks. No target quality, latency, source release or installed/physical PASS
is claimed from this replan. The new explicit decision is
`decisions/2026-10-08-space-boundary-canonical-evidence.json`.

### Focused source proof after positive-evidence repair

Actual guarded remote source proof PASS: 1824 selected/executed/passed, zero
failures (1817 library plus seven integration identities), 155.5 seconds.
All four original mechanism pairs now select the exact BoundaryShift and
project pending Space through the same receipt. Current-word-only edits still
refuse the pair, clean right words and invalid reconstructed targets do not
acquire pair permission, and recorded inverse remains receipt-bound. Exact
receipt: `/home/ubu/.cache/lay/development/run-hiat7huk/RESULT.json`.
This is focused synthetic-vocabulary/source evidence only; full canonical
3032 identities, graph/canon/lints/MSRV, an exact source artifact and installed
physical windows remain pending. Installed authority is still the predecessor.

### Full source gate, physical pilot refusal and rollback

The frozen source passed all 3032 fixed identities and all 17 mandatory remote
release checks. Exact source artifact SHA-256:
`4d885d227fc083b7823502e825628fa7477d5ffc1fd27d54a75a4fa2fc76de12`.
Receipt: `/home/ubu/.cache/lay/development/space-boundary-final-source-20261008/frozen/FETCHED/RELEASE_GATE_RESULT.json`.
This establishes source acceptance only, not dictionary/runtime/client parity.

That artifact was installed with the existing models and configuration. The
first GTK3 entry physical pilot executed four cases: zero PASS, four FAIL,
zero BLOCKED. Both tested pair surfaces remained unchanged; no partial text
mutation was observed. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-physical-final-20261008/windows/WIDE_MATRIX_V2_20261008T042656.json`.
Existing text-free prefetch metadata contains Ready/SuggestOnly refusals as
well as missed 3500-us leases. Inner semantic refusal reasons are UNKNOWN;
timing alone cannot explain the Ready refusals. Metadata packet:
`/home/ubu/.cache/lay/development/space-boundary-physical-final-20261008/PILOT_ROUTE_METADATA.json`.

The predecessor IME was restored and loaded with SHA-256
`03a8ef6023e15afcf315bfe4c8ec941550dad4a498f4a9163bcaf4194f82dc6a`.
Global IBus, other service binaries, models, config and input sources were not
changed. Rollback receipt:
`/home/ubu/.cache/lay/development/space-boundary-physical-final-20261008/ROLLBACK.json`.
Current installed feature verdict is FAIL, with predecessor runtime authority.
The remaining windows were not tested with this artifact after this refusal.

Replan: verify exact installed dictionary surfaces/bindings without copying
models, then identify the first failed semantic transition on the existing
chooser path. Do not infer a lexical or memory veto from timing, change the
prefetch deadline, introduce another worker, or repeat the unchanged matrix.

### Native feedback cleanup and installed timing replan — 2026-10-08

The existing explanation CLI identified an exact state-specific L4 repel of
eight for the main pair. Four negative journal rows came from two earlier
owned physical inverse-test episodes, independently bound to their IME install
receipts. The existing native UsageEventProjection compiled their exact counter
delta. Only those four rows and their counter contributions were removed;
828 other journal rows, every unaffected counter, the August feedback base,
models and configuration were preserved. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-physical-final-20261008/OWN_QA_REJECTION_CLEANUP.json`.
This is cleanup of our own QA effects, not a weaker L4 veto or runtime rule.
The explanation changed from exact negative refusal to selecting the pair.

The same fully tested artifact `4d885d227fc083b7823502e825628fa7477d5ffc1fd27d54a75a4fa2fc76de12`
was installed again. Current runtime authority changed to these bytes; global
IBus, other binaries, models and configuration were preserved. Install receipt:
`/home/ubu/.cache/lay/development/space-boundary-physical-clean-20261008/INSTALL.json`.
GTK3 entry pair plus next word passed one of one actual cases. The subsequent
partial matrix executed seven cases: four PASS, three FAIL, zero BLOCKED.
Pair, next word, clean-right preservation and invalid-left refusal passed.
The canonical dictionary attests `расчёты`, not the earlier `расчеты` fixture;
the prospective physical test now types and expects the exact attested spelling.
Earlier failures remain recorded. Inverse was explicitly excluded pending
provenance-bound cleanup of its QA feedback. Receipts:
`/home/ubu/.cache/lay/development/space-boundary-physical-clean-20261008/windows/WIDE_MATRIX_V2_20261008T051950.json`
and `WIDE_MATRIX_V2_20261008T052754.json` in the same directory.

The calculation pair obtained an Apply gate, eligible selected candidate,
transition receipt and final adapter decision. Its closing frame nevertheless
missed the existing 3500-us lease: queue 64117 us, evaluation 53442 us, total
117560 us; publication was superseded. After returning focus the main pair
also obtained a final Apply decision but arrived after Space: queue 27355 us,
evaluation 42589 us, total 69945 us. Actual focus restoration succeeded.
Their first failed transition is timely publication by the existing prefetch
worker, not pair authority or text delivery. The icon case failed because its
probe path was not supplied (`NOT_CAPTURED`); that is missing probe evidence.

Explicit replan: account for the unmeasured candidate-production time and
obsolete work inside the existing chooser/prefetch path before selecting a
minimal repair. Do not add a second chooser/worker, extend the deadline or
weaken frame validation. Preserve all grounded candidate retention, ordinary
frame-bound authority and negative feedback. A source repair requires remote
focused and complete gates, a new artifact and new actual window evidence.
Remaining windows, inverse, stable timing and the icon probe are not accepted.

Chosen first repair under that replan: carry an optional computation-liveness
check from the existing Space worker through the shared adapter/InputGate/core.
Use the existing latest-request generation, checked between producers and before
peak/decision work. Obsolete work returns cancellation, not NoApply; the worker
discards it and takes the newest coalesced request. Uncancelled callers retain
the complete existing lattice, selection, receipt and output. No stage is skipped
for a current request. Deadline, worker count, material generation and final
publication validation stay unchanged. Explicit decision:
`decisions/2026-10-08-space-prefetch-obsolete-computation.json`.

First focused attempt stopped in test compilation: the three existing proof
helper calls still used the old two-argument signature, producing four compiler
errors. No tests ran and no artifact was installed. Receipt:
`/home/ubu/.cache/lay/development/run-pmluzq_f/RESULT.json`, 38.5 seconds.
The second bounded pass updates those calls with always-current liveness and
unwraps the authorized proof fixture result. The latency contention helper
still deliberately computes full work; its workload/assertions are not weakened.

Second focused pass PASS: 2519 selected/executed/passed, zero failures, 188.5
seconds: 1817 library, 695 IME and seven boundary integration identities.
Cancellation both before work and after a newer request yields no terminal
decision; the current request matches the ordinary selected surface, edit plan
and apply permission. Existing contextual full-worker proof also remains Apply.
Receipt: `/home/ubu/.cache/lay/development/run-2pij_3fk/RESULT.json`.
This is source evidence only. No measured latency reduction, new installed bytes
or remaining-window acceptance follows from it. Root reviewed this repair;
no independent agent was used, as explicitly requested. Full canonical 3032
identities and all existing release checks remain required before installation.

### Full tests pass; explicit lint-coordinate-only replan

The frozen cancellation source passed all 3032 correctness/package identities:
2996 correctness and 36 package, zero known semantic or infrastructure failures.
The encompassing release command then failed its default dead-code inventory.
Two pre-existing warning records moved by exactly115bytes after the removed
imports. Their messages, subjects, targets and occurrence counts did not change.
No new artifact was built or installed. Remote receipt:
`/home/e/projects/lay-development-runner/space-boundary-full-50UFJY/RELEASE_GATE_RESULT.json`;
test summary: `source/target/test-lanes-results/SUMMARY.json` under that run.
Full command 462.6 seconds, including test lanes 404.5 seconds; release verdict FAIL.

Explicit replan before metadata edits: refresh only those existing source spans
in the default and research inventories, preserving every other field and
inventory entry. Verify each new span against its literal current field name.
Do not change lint policy, ignore a warning, alter a semantic test or increase
the known-failure budget. Functional source remains exactly the2519-focused and
3032-full-test source. A fresh complete source release gate remains mandatory.
Decision: `decisions/2026-10-08-space-prefetch-lint-locations.json`.

The location-only comparison passed: three location objects per baseline,
with all 532 default and 356 research entries retained. Every field outside
locations remains literal exact; each new span names the current field.
Neither runtime source nor lint rules changed. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-obsolete-source-20261008/LINT_LOCATION_REFRESH.json`.
The failed release and its full-test summary were retained in `frozen/FAILED_FETCHED`
under that private run. A fresh source freeze will rerun the complete release
command; the previous failed release is not reclassified as PASS.


The fresh lint-location source also passed3032/3032 tests (403.4 seconds), but
release failed again (461.5 seconds): refreshed entries were not in the existing
canonical order. Root changed coordinates without resorting the inventory.
Receipt: `/home/ubu/.cache/lay/development/space-boundary-obsolete-source-20261008/frozen-lint-locations/RELEASE_GATE_RESULT.json`;
remote run `space-boundary-full-svAEH8`. No artifact was installed.
Before the second bounded metadata pass, explicitly retain the exact entry
multiset and sort it by the existing `lint_inventory.canonical` rule. Run the
actual lint contract first on fresh remote source; only after it passes rerun
the unchanged complete release gate. Functional source and all test identities
remain literal exact. The previous failed releases retain their FAIL verdicts.


### Cancellation source: complete release PASS; installation/physical pending

The canonical entry-order refresh preserved the exact warning-entry multiset.
Real guarded remote lint preflight PASS in76.4 seconds: default532 and research356
entries, zero non-dead diagnostics; source members remained unchanged. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-obsolete-source-20261008/frozen-canonical-order/LINT_PREFLIGHT.json`.
The fresh unchanged full release then PASSed all17 commands, including3032/3032
correctness/package identities (2996+36), both lint inventories, public issue and
installer regressions, MSRV default/compiler and exact IME artifact build.
Full gate775.7 seconds; entire release894.3 seconds. No known failures.
Receipt: `/home/ubu/.cache/lay/development/space-boundary-obsolete-source-20261008/frozen-canonical-order/FETCHED/RELEASE_GATE_RESULT.json`.
Artifact8162832 bytes, SHA-256
`08be0e60a0bab2e35af19a54ae552dee613bc549e6352c190fbb73bb7e967188`.
The1500-member final source closure is independently bound by
`FETCHED/FINAL_SOURCE.json`; all ten generated exports were imported exactly.
Only this owner and cancellation decision subsequently record measured source
results; their metadata-only successor hashes are pinned separately. No functional
source changed after the complete gate. Installed bytes and all real windows
are separate pending evidence; no latency reduction is claimed yet.


## Installed cancellation experiment and bounded replan — 2026-10-08 07:06 EEST

Installed08be0e60 via an owned empty field, preserving global IBus PID270775,
daemon/models, source identity and all existing delivery adapters. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-obsolete-physical-20261008/INSTALL.json`.
Actual GTK3 entry pilot executed7 cells:4 PASS,3 FAIL,0 BLOCKED
(`windows/WIDE_MATRIX_V2_20261008T065140.json` in that receipt directory).
The task-owned idle Chrome used115% CPU and had task-limit failures. It was
closed by its verified owner; user browser processes were untouched. A
justified repeat of the3 failed cells then produced1 PASS,2 FAIL,0 BLOCKED
(`windows/WIDE_MATRIX_V2_20261008T065616.json`). The calculation pair now passes.
The focus-return pair still misses the unchanged lease:50 retained candidates,
correct final Apply with receipt, queue13426us and evaluation46610us.
DecisionCore took26639us, including L33643us; canonical L113209us and
productive10662us. No latency improvement or physical acceptance is claimed.

The added Shift probe test sends Escape between pairs. That input revokes
word ownership by the existing contract; the next toggle correctly refuses
context authority. Its FAIL receipt remains unchanged. A prospective distinct
protocol will retain the owned word while probing the next decoder; the
original protected fast eight-pair sequence will also run separately.

Next bounded source experiment: evaluate the complete ordered candidate vector
on the existing calling worker rather than the shared Rayon pool. The same
process also submits blocking L1.1 contour requests and display work to Rayon.
Pool contention is a hypothesis, not a measured attribution. Warm CLI evidence
shows18 evaluations plus selection complete in4537us, whereas the physical
50-candidate DecisionCore took26639us with3643us L3. All candidates, input
readouts, order, calculations, settlement, selection, receipts and safety rules
remain identical. No new owner/cache/worker/timer/deadline or pruning.

Root review only, as explicitly requested by the user. At most two bounded
physical attempts; retain failures and explicitly replan if latency remains.
Focused source checks and the unchanged3032/full17-command release gate must
pass before installation. Real focus-return acceptance is the deciding scope;
source tests alone do not establish latency or physical correctness.

Focused ordered-vector source check PASS:2519/2519,0 failures,201.1 seconds.
Receipt `/home/ubu/.cache/lay/development/run-981co09l/RESULT.json`.
Only product change from08be is replacing the three candidate-evaluation
parallel iterators with ordered iterators and removing their unused import.
The distinct prospective Shift protocol passed8/8 word/decoder/icon steps
on installed08be in GTK3 entry (`windows/WIDE_MATRIX_V2_20261008T070955.json`,
1 executed case); this does not accept the new source candidate.

## Ordered-vector source release and first physical attempt — 2026-10-08

Unchanged3032/3032 correctness/package identities and all17 release commands
PASS; full gate787.0 seconds, enclosing release905.6 seconds. Exact artifact
8b4e0be6f7b0669ce44b39653d2a036e8743f4dff063e40295564d4f8787e958,8155920bytes.
Receipt `/home/ubu/.cache/lay/development/space-boundary-vector-source-20261008/frozen/FETCHED/RELEASE_GATE_RESULT.json`.
Installed through an owned empty field; new IME PID2729739, same global IBus,
daemon/models/config/sources. Receipt in
`/home/ubu/.cache/lay/development/space-boundary-vector-physical-20261008/INSTALL.json`.

First GTK3 physical run executed73 cells:70 PASS,3 FAIL,0 BLOCKED;
`windows/WIDE_MATRIX_V2_20261008T073150.json` under that physical directory.
This included the original fast eight Shift pairs, Tab, held Shift, no-hint
and alternating-word sequences. Main repair, next word, clean-right,
invalid-left and distinct per-pair decoder/icon protocol passed. Calculation,
focus-return and ordinary layout-plus-typo case missed timely correction.
Focus retains50 candidates, correct Apply+receipt, queue1386us and
evaluation50730us; DecisionCore22342us, including L33312us.
Canonical L112772us, productive20465us. This experiment did not establish
a latency solution. Complete physical acceptance and promotion remain pending.

Own inverse maintenance originally refused native500KiB journal rotation;
the inverse had restored exact original text but its2 rejection rows remained.
The QA helper now accepts only an unambiguous byte-identical retained old
suffix as current prefix, matching native compaction, then removes only the
exact two receipt-owned new rejection rows. All other859 current rows,
positive counters and historical feedback baseline were preserved. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-obsolete-physical-20261008/OWNED_INVERSE_5p5dfp3z/CLEANUP.json`.
Old failed/contaminated physical receipts remain failed; no runtime veto changed.

Next causal measurement uses the already gated release library and its existing
DecisionCore timing switch in a small read-only CLI, with synthetic lexical
frame coordinates and actual installed models/sockets. No model copies,
additional service, fit, source/runtime change or output authority. It may
locate CPU work inside DecisionCore; it is not a real input/frame/ownership or
physical acceptance proof. Warm and first-call timings remain separate.

### Measured redundant full-vector copying; bounded implementation replan

The read-only synthetic-frame probe linked the exact gated release library.
With the actual installed package/socket it retained50 candidates and Apply
for the main pair; no input/output or ownership authority was exercised.
Four warm50-candidate samples spent11394..12221us in serial evaluations
and5293..7604us in selection;18-candidate samples spent4678..4903us and
1447..1661us respectively. Exact full samples and cold/warm separation:
`/home/ubu/.cache/lay/development/space-boundary-vector-source-20261008/PROFILE_RESULT.json`.

Source locates an avoidable quadratic copy: authority_lane_allows_apply clones
the complete candidate and evaluation vectors for every authority lane,
changes just one slot, then removes that slot. Every other slot is identical.
Next bounded change reuses one lazily allocated call-local scratch vector pair,
substitutes the same slot, invokes the unchanged admission rules and restores
the original slot before the next lane, including refusal. Returned lane
provenance/evaluation stays owned and exact. All competitors, order, signals,
selection, verifier, SafetyGate, receipts and delivery remain unchanged.
Restore the original parallel evaluations: the serial scheduling experiment
did not solve the physical failure and adds unnecessary per-candidate latency.
No persistent cache, extra chooser, producer, worker, timer or deadline change.
Extend an existing semantic identity to prove both allowed and refused lanes
leave all candidate/evaluation slots unchanged. Focused and full3032/all17
source gates precede installation; two bounded physical attempts then replan.

### Call-local authority batch implementation and focused proof

One lazy scratch vector pair now serves the existing decision call. Both
substituted slots are restored after admission and refusal. Original ordered
parallel candidate evaluation is restored. Existing semantic proof checks all
candidate/evaluation slots across Normal, Experimental and Strict policies,
including an actual refusal; selected surfaces and exact receipts remain tested.
Focused2519/2519PASS (695IME,1817library,7mechanism), failed0:
`/home/ubu/.cache/lay/development/run-mc1dxuot/RESULT.json`. Full source release and new installed physical latency are pending.
Only decision.rs and its existing tests changed functionally since8b4; no
producer, authority rule, candidate, deadline, SafetyGate, verifier or delivery
route changed. This record is source proof, not physical acceptance.

### Scratch batch: complete source release PASS

Exact source closure1502 completed all17 guarded release commands, including
3032/3032required tests, zero known/infrastructure failures, unchanged lint
inventories, MSRV1.88default/compiler and exact release artifact. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-authority-source-20261008/frozen/FETCHED/RELEASE_GATE_RESULT.json`. ArtifactSHA256`824e2f5f3c6d1b65c6219199f9f6aac9eb94ebee70a34f2d61e9b7640cbcc520`,
8160272bytes. No new installed or physical result yet.
One existing empty-field/IME-only transaction will replace8b4, preserve the
global IBus process and all other installed/model/config bytes, then execute
fixed physical cases. Current strict pair factory, generic verifier, admission
rules, workers and delivery are byte-identical to8b4. No latency claim yet.

### Installed scratch batch: two bounded physical passes; latency unresolved

Owned-empty installationPASS, IME-only reload, global IBus preserved:
`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/INSTALL.json`. FirstGTK3entry13cases:11PASS,2FAIL
(rnjhjt and calculation). Second same fixed13cases:12PASS,1FAIL(rnjhjt).
Main pair, next word, clean/refusal, focus return and protected fastShift pass
both. Calculation first has correct18candidate Apply+receipt but queue37001us
plus evaluation76695us=113697us, published superseded; second passes.
Receipts`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/windows/WIDE_MATRIX_V2_20261008T080645.json` and
`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/windows/WIDE_MATRIX_V2_20261008T081023.json`.
These are failed promotion attempts, not physical acceptance. Previous failures
remain unchanged. Replan:`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/PHYSICAL_REPLAN.json`.
Next read-only probe uses existing deterministic/core/canonical timing switches
and the exact gated library, without model copies, service or input/output.
No source change yet; locate unaccounted producer work before optimizing it.

### Existing-timer separation after two physical passes

Read-only exact-gated-library probe, synthetic frames and installed data:
`/home/ubu/.cache/lay/development/space-boundary-authority-source-20261008/profile-three/PROFILE_RESULT.json`.
Five rounds retain Apply for both pairs. Warm totals main23382..33985us,
calculation20831..27195us; calculation composite alone7400..11961us.
First query-cold ordinary missing-letter example has31420us deterministic
work (typing25973us), total61917us, and no Apply even when warm.
The canonical cohort retains both competing surfaces, with tied authority;
ordinary target producer is SuggestOnly. This identifies a separate semantic
admission question, not a delivery failure. Its first proposal reason and
process-policy parity with native IME require verification before changing it.
Outside-DecisionCore peak preparation is194..570us warm and1948..2347us
query-cold in this probe; it does not substantiate the hypothesized62ms peak
bottleneck. CLI default process policy is not yet proven native-IME-equivalent.
No product source, installed bytes, model, gate or physical receipt changed.

### Exact ordinary proposal refusal and native process-policy probe

Explicit HotFieldPolicy::ime probe on the same gated release library confirms
the ordinary example has no Apply, cold and warm. Raw producer-lattice reason
is `l2_field_tie_requires_context`, imposed by
`apply_authority_to_candidate_lattice` for tied length-changing repairs.
The authority bridge is byte-identical to source basebee14b0; this specific
rule is existing source behavior, not introduced by the pair mechanism.
Do not remove it or rewrite the historical physical FAIL to obtain PASS.
Exact read-only receipts:
`/home/ubu/.cache/lay/development/space-boundary-authority-source-20261008/profile-ime/PROFILE_RUN.out`
and `PROFILE_RUN.err`; `BUILD.json` binds the same release library and native
IME process policy. Both boundary pairs retain Apply in all5rounds.
This source observation does not establish predecessor-installed parity.
Next bounded CPU sampling measures the read-only probe, without user input,
model copies, services, product edits or a physical acceptance claim.

### Bounded wider physical diagnosis after explicit replan

CPU sample of the read-only probe provides no single measured native latency
bottleneck and justifies no further product edit. Preserve824bytes and all
fixed expected outcomes. Root replan now admits one first pass over9previously
untested browser/native fields,14fixed cases per field, inverse last with
exact own-feedback cleanup. Historical ordinary ambiguous-case FAIL stays
FAIL; report it separately from the8new connected mechanism cases.
Receipt plan: `.../space-boundary-authority-physical-20261008/PHYSICAL_REPLAN.json`.
No runtime authority or safety/transport/deadline changes.

Shared guard refused the first wider launch before input because reserved RAM
was insufficient. Release only exact task-owned Chrome/supervisor, preserve
all shared ceilings and unrelated apps, then run6native fields sequentially
under the original1792MiB guard. Browser fields follow separately. The first
rejection remains `WIDE_DIAGNOSTIC_1.log`; no physical cells executed there.

### Wider native result and prospective observer repair

Six native fields14cases:75executed,69PASS,6legacy ambiguous-case FAIL;
9BLOCKED, zero fatal. Connected mechanism39executed/39PASS; all remaining
9blocks are3cases each in Qt entry/multiline/rich after intentional focus
excursion. Source/installed824 unchanged. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/windows/WIDE_MATRIX_V2_20261008T083251.json`.
All3executed inverse cases cleaned exactly2own rejection rows each, no other
journal/counter changes. Guarded native observed peak167956480bytes;
measured during execution, not claimed final peak.

First failed QA transition: observer.wait_state calls exact field.focus once
after closing the owned excursion window; a transient failure immediately
poisons subsequent rows despite the specified2s wait. Prospective focus-v2
waits within that existing2s for the unchanged owner/focus guard, exact text
and additional caret4 check; no keys until proof. Ordinary waits/Space
timing remain unchanged. V1BLOCKED receipts stay BLOCKED. Root review pins
the new distinct case ID and code before execution.
Tor prospective private-profile preflight combines already reviewed Firefox
prelaunch/style-pool and MesaLP limits with2CPU affinity, within one admitted
384-task scope and unchanged shared512/6GiB/CPU ceiling. Prior default128/256
setup failures remain BLOCKED. No user Tor/proxy/security/sandbox modification.
Only QA code changed; no source-release gate is reopened by private observers.

Tor contained384-task preflight no longer exhausts resources: task peak256,
max-events0, memory peak719507456bytes, noOOM. Fresh exact owned browser window
is found, but focused editable Text binding is still unproved; no keyboard
input or IME case accepted. User TorPID838834/start247091505 preserved, owned
scope gone after cleanup. Receipt:
`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/tor-contained-384-preflight/RESULT.json`.
This moves the first setup failure from resource admission to independent
field observation; a bounded owned-window tree observation is next, not a
weaker Text/focus guard. Qt second targeted physical pass now uses the
prospectively reviewed focus-v2 case and the two previously blocked followers.

### Qt inverse: client publication loss, not an IME signal loss

Installed 824 remains unchanged and unaccepted. Qt entry/rich forward repair
passes, but recorded inverse waits for a missing exact post-edit snapshot.
The hash-only protocol receipt
`/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/QT_INVERSE_HASH_PACKET.json`
binds the owned Qt PID/starttick and native Wayland plugin. The client sends
the same pre-edit 10-character snapshot after both forward and undo refresh
requests; IBus suppresses forwarding identical cached values to the engine.
The visible corrected widget has 16 characters. No inverse output or negative
feedback was produced. Old failures remain failures.

Read-only upstream source, saved under
`/home/ubu/.cache/lay/development/space-boundary-authority-source-20261008/protocol-readonly/`,
shows Qt 6.10.2 suppressing the text-input-v3 commit after a surrounding delete
(`m_condReselection`). Mutter 50.1 serves RequireSurroundingText from its last
committed client state. This explains the measured stale callback; it does
not authorize a predicted snapshot, a terminal fallback, synthetic cursor
movement, or a new inverse executor. Those alternatives are rejected.

The inherited Qt module priority is `wayland;ibus`, despite QT_IM_MODULE=ibus.
Prospective diagnosis selects `ibus;wayland` only for owned Qt fixtures, using
the already installed standard plugin and the same physical cases. This is
a different client configuration, never acceptance of the failed Wayland
configuration. No global environment, IBus, source or library is changed.
Scope: `QT_DIRECT_IBUS_PROSPECTIVE.json` in the physical receipt directory.

Private focus-v2 could accept a cached pre-excursion sample. Its distinct
focus-v3 successor requires two fresh exact owner/text/caret samples after
the owned excursion closes, within the same two-second bound. Ordinary
Space/key timing and all frozen observer guards remain unchanged; prior
focus-v1/v2 results are preserved. No product-code change is justified yet.

Direct Qt IBus diagnosis finished: 42 executed, 39 PASS, zero BLOCKED/fatal;
all 24 connected cases PASS, including three exact inverses and focus-v3.
The three remaining failures are the preserved legacy ambiguous `кторое`
expectation. Actual owned Qt process maps confirm the installed standard
libibusplatforminputcontextplugin.so alongside the Wayland display plugin.
Receipt: `QT_DIRECT_IBUS_RESULT.json` in the physical evidence directory.
This proves the proposed client priority, not the old Wayland bridge.

Tor setup was a Tor Launcher startup-failure modal, independently read from
the exact owned AT-SPI subtree. The private local-file fixture now runs offline,
does not start Tor, and points control access to a nonexistent private socket;
no user Tor controller, proxy/security/sandbox preference or profile is changed.
Static UUID aria-label/autofocus removes a fixture dependency on inline JS.
The no-input preflight bound all original guards; its keyboard-close helper
failed because no UInput existed, and its exact owned scope was confirmed gone.
Keep that harness failure distinct from input acceptance.

First Tor 14-case matrix: 0 executed, 16 BLOCKED, no fatal. First-word Tab
is forwarded with unknown-start admission and no preedit, then moves focus;
all following cases are dependent blocks. Raw clear had independently proved
empty text/caret. Trace advertises capabilities 41, so this is not evidence
of no IBus connection or a general private-content veto. The receipt remains
`windows/WIDE_MATRIX_V2_20261008T091534.json` in the physical evidence directory.
Explicit replan tests the unchanged connected eight cases in fresh Tor fields
without the preceding blocked Tab case. Fresh profiles use --no-remote and
exact pidfd/profile/starttick cleanup, including after focus loss. Cleanup
uses no keys and never targets the user's existing Tor process. No product
guard, expected text, source byte or deadline changes.

## 2026-10-08 — Confirmed Chromium composition ordering failure and bounded repair

Installed IME `824e2f5f3c6d1b65c6219199f9f6aac9eb94ebee70a34f2d61e9b7640cbcc520` is not accepted. Chrome 152.0.7977.82: 42 cases executed, 21 PASS and 21 FAIL, no BLOCKED. All three main pair cases visibly produced `они должн должны быть ` instead of `они должны быть `. This is output delivery failure, distinct from the legacy `rnjhjt` ranking refusal. Receipt: `/home/ubu/.cache/lay/development/space-boundary-authority-physical-20261008/CHROME_CONNECTED_LEGACY_1.log`.

The first failed transition is the immediate mixed preedit/client replacement inside synchronous ProcessKeyEvent. The exact Chromium primary source buffers composition commits in sync mode while DeleteSurroundingText calls the client independently. Clearing the engine preedit is not proof that the renderer has finished its composition. The source hash and bounded repair are recorded in `decisions/2026-10-08-space-boundary-client-commit-order.json`.

Replan: ordinary closing Space commits the user composition; the existing postcondition record retains the already-selected action once. Only the actual exact complete original-plus-Space client snapshot in the same unchanged lease can run the existing committed-tail replacement. No second chooser, timer, worker, output route, relaxed verifier or retry is admitted. Prior mixed-partition source PASS remains historical and cannot certify browser behavior. New focused/full source and physical evidence are pending; runtime authority has not changed.

Forward-only focused check `/home/ubu/.cache/lay/development/run-whjub_t_/RESULT.json`: 2520 selected/executed/PASS, zero failures, 213.2s. Subsequent exact causal inverse receipt edits are not covered by that earlier source snapshot. Fresh final focused check is running. The inverse retains only the actual full original client snapshot, separately from live observations, bound to the same post-dispatch owner/focus/epoch/tail/config; equal scalar and UTF-8 lengths keep delete coordinates unchanged. A present contradictory snapshot vetoes it. No positive outcome is inferred from this receipt.

Bounded inverse-receipt repair: the fresh regression exposed a recursive shared-state mutex acquisition while checking causal inverse authority. The frozen test child PID730119/start158884948 in remote `lay-verify-1000-729030.scope` was terminated exactly through pidfd so the owning runner can report failure and release its lease. No runtime install occurred. The existing input-frame predicate is now factored into its identical local checks plus shared-owner check; callers already holding SharedState check that same owner directly. This removes recursion without dropping any authority predicate. The failed snapshot/receipt remains historical. Fresh focused evidence and full release are required for these final bytes.

The obsolete mixed owned-preedit/client deletion implementation is removed from committed-tail dispatch. OwnedPreedit provenance can establish the initial selected pair and ordinary Space commit only; it cannot directly enter committed-tail replacement. After the actual complete client receipt, the same verified semantic plan uses normal full committed-tail coordinates. Terminal and atomic routes retain their prior contracts.

Final focused source check `/home/ubu/.cache/lay/development/run-ojnisqm2/RESULT.json`: 2520 selected/executed/PASS, zero failures, 206.3s (696 IME owner, 1817 library, 7 integration). The production-owner regression now covers callback ordering, duplicate snapshots, pre-commit echo, exact causal inverse without final readout, owner/focus/contradictory-snapshot veto, selection/caret/text/config/material/deadline/next-key/closed-field invalidation, and consumption after partial output. The earlier recursive-mutex snapshot remains FAIL at `/home/ubu/.cache/lay/development/run-7hal2dyy/RESULT.json` (2519 PASS, 1 terminated regression failure), never relabelled.

First full client-commit attempt `/home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/frozen/RELEASE_GATE_RESULT.json` failed `CANON_PRECHECK` after4.32s: `protected_change_without_new_decision:src/bin/lay_ibus_engine/managed.rs`. No functional test/build command ran. Bounded metadata repair adds the actual next-key cancellation entrypoint to the explicit decision; no Rust source or guard changed, and focused2520/2520 remains applicable. The failed receipt is retained; full gate reruns in a fresh immutable source directory.

After the bounded two full attempts, explicit owner replan: `/home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/frozen-canon/RELEASE_GATE_RESULT.json` failed GRAPH_REFRESH after81.54s. AST graph contracts all PASS; the existing source guard correctly rejected a new ordinary active-composition commit call in committed_tail.rs. The ordinary closing Space must be executed by the existing managed.rs input owner. Move that user-input commit and attachment of its one selected continuation to managed.rs; committed_tail.rs continues to prepare the already-selected payload and execute only the subsequent exact committed-tail replacement. No architecture guard exception, owner, chooser, worker, timer or deadline is added. Discovery retained every3058 original manifest row literally and added exactly the one allowed correctness identity (3059 total discovery rows). Functional focused/full and physical proof are pending for this owner placement change; previous PASS/FAIL receipts are retained.

Owner-placement focused check `/home/ubu/.cache/lay/development/run-8zsv8w1u/RESULT.json`: 2520 selected/executed/PASS, zero failures, 210.9s. The managed input owner now performs ordinary Space and attaches the same one selected continuation; the failed direct commit call is absent from committed_tail.rs. Existing source ownership guards remain unchanged. Fresh full source release and physical windows remain pending.

Owner-placement full gate PYgC4h retained FAIL: /home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/frozen-owner/RELEASE_GATE_RESULT.json. FMT, discovery, graph and canon PASS; full fixed set stopped on exactly the protected text-edit gate successor binding, with no runtime installation. October7 immutable successor expected its historical gate bytes, while the explicit October8 client-commit decision adds only restoration of the already sealed Space projection. Bounded replan: preserve the entire old decision and baseline hashes; pin the new explicit successor decision, predecessor chain, actual gate hash and mode in the existing source-contract identity. No failure-ledger exception, new test identity, semantic safety or runtime code change. Focused contract and fresh full gate pending. The newly pinned decision is immutable; later results are recorded here only.

Protected-successor source contract `/home/ubu/.cache/lay/development/run-gz7ye7tf/RESULT.json`: 7/7 PASS, zero failures, 12.9s. Runtime Rust bytes remain identical to owner-placement focused2520/2520PASS; only the explicit successor decision, existing protected-artifact contract and owning record changed. Original predecessor and all other protected artifacts remain enforced. Fresh complete release and physical acceptance pending.

Client-commit successor complete source release PASS: `/home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/frozen-successor-local/FETCHED/RELEASE_GATE_RESULT.json`, remote rlaYZS,903.609s. All17 commands PASS,3033 selected/executed/PASS (2997 correctness,36 package), zero known semantic or infrastructure failures; MSRV1.88.0 default and lexical compiler PASS. Final source closure1503 files, SHA2567eaec809cffb3874af7232e62c0ad7f774c794d827f35bd7ca8361f236b44417; exact IME8176144B SHA2567b96e7403259813098291e17599ec0453887f194e4cea484df58767cdbe8d551. Imported only10 approved graph/manifest/ledger exports; all other functional source unchanged. Newly pinned client-commit ADR remains byte-identical. Installed runtime and physical acceptance pending at this entry. All previous FAIL/BLOCKED receipts retained; no models copied or changed.

New IME7b96e740 installed through exact owned-empty transaction: `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/INSTALL.json` PASS. Global IBus and other actor PIDs, config and models unchanged at install. Critical physical pilot1 retained `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/windows/WIDE_MATRIX_V2_20261008T110642.json`: two executed (one PASS, one FAIL), one Chrome setup BLOCKED, zero fatal. Native Qt inverse includes exact forward and inverse PASS. Qt standalone repair failed before the first left-word Space proof: visible `он идолжн`, and ordered engine trace places prefix Space before the preceding и callback; the new pair mechanism was never reached. Do not claim a global root cause or physical acceptance. Chrome startup128-task scope hit pids.events max2/EAGAIN and never loaded its owned fixture; preserve renderer failure. Bounded pilot2 uses a fresh exact task-owned256-task Chrome scope inside unchanged shared512, same flags/memory/observers/key timing. Product source and installed bytes unchanged; no repeat-until-green claim.

Critical pilot2 `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/CLIENT_COMMIT_CRITICAL_PILOT_2.log`: Chrome.input and native Qt.entry exact forward+inverse4/4PASS, zero BLOCKED/fatal,14.87s. Full six-field14-case pass1 retained `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/windows/WIDE_MATRIX_V2_20261008T111522.json`:84 executed64PASS20FAIL,0BLOCKED/fatal,191.27s. Six failures are the unchanged rnjhjt/кторое length-changing context guard. Three v2 Shift probes used strict DOM raw value and rejected legitimate active hint with exact typed caret. Chrome.editable inverse visibly restored the exact pair, but own feedback cleanup failed its precommit concurrent-generation guard after selected two rows and validated delta; those exact two task-owned negatives remain in the journal and precede Qt pair refusals. No physical acceptance claimed. Bounded QA replan: preserve all receipts; keep original23 observers and product/source/installed bytes unchanged; stabilize the journal/counters/baseline generation for100ms inside the existing1.5s cleanup bound, maintain all exact row/counter/CAS protections, recover only those two own rows, and run fresh affected cells. Version3 Shift probe uses the already pinned exact collapsed-caret and unambiguous DOM composition representation; all eight pairs, next-letter/Backspace/mode/icon checks, ordinary timing and expected typed text retained. Historical v2 remains FAIL. No runtime timer, model, weight, deadline or chooser changed.

Bounded connected pass2 retained `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/CHROME_QT_CONNECTED_SCOPE_2.log`:24 executed20PASS4FAIL,3 Chrome setup BLOCKED,0fatal,51.306s. Recovered two own Chrome.editable rows exactly; all three Qt main pair and next-word cases now PASS, supporting the earlier contamination diagnosis. Qt ordinary uninterrupted input still produced reordered/missing separators in calc and invalid-left cases before boundary authority; Qt.rich inverse visibly restored exact original but cleanup failed before selected-file creation. No release acceptance. Chrome owned process ended with code0 after previous per-field tab cleanup; the stale READY copy cannot bind a new matrix. Explicit bounded replan: retain both wide receipts, require complete two-row feedback episode plus quiet generation and completed native500KiB rotation inside the existing1.5s maintenance bound; no relaxed row/cap/CAS guard. Fresh recovery output preserves the failed cleanup folder. Compare ordinary Qt cases with exact824 predecessor through existing owned-empty IME rollback, then reinstall the same passed7b96 candidate using a fresh named receipt and verified existing824 backup. No new product edits or global IBus/source/config/model changes.

Exact824 predecessor diagnostic, through owned-empty rollback only: `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/ROLLBACK.json` PASS, global IBus/other actors/config unchanged. Two bounded same-six-cell Qt ordinary-input passes: QT_PREDECESSOR_ORDINARY_INPUT_1.log5/6PASS (one calc unchanged/no-apply), QT_PREDECESSOR_ORDINARY_INPUT_2.log6/6PASS, zero BLOCKED/fatal. This establishes an older no-apply example, but does NOT reproduce or prove the separator reorder cause; no general preexisting-ordering claim is made. Current source IBus interface already uses spawn=false ordered callbacks and is byte-identical to824; no dispatch edit is justified. Return to exact passed7b96 candidate and the original full14-case-per-field scope. The abbreviated connected-only run omitted the preceding six existing input/activation exercises; retain its failure separately. All14-case oracle/key timing retained, with only already documented v3 Shift observer and bounded exact feedback cleanup repair. No new runtime edits, fit, cache or producer.


## 2026-10-08 — Retain the current pair calculation across ordinary Space

Current7b96 source full3033/3033PASS remains historical; native Qt final14-case pass produced33executed28PASS5FAIL,9BLOCKED,0fatal at QT_FULL_SCOPE_AFTER_FEEDBACK_REPAIR_1.log. Three failures are the unchanged legacy rnjhjt refusal. Two connected entry cases reached NotReady at3500us: current focus calculation43995us, calculation case later cancelled61803us. Exact final-frame queue10us excludes queueing as the only cause. No new physical acceptance is claimed.

Existing remote diagnostic binary returned L11ServiceUnavailable; no service or model stand was created. Receipt: /home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/latency-readonly/STAGES_REMOTE.json. Earlier unchanged producer profiles show materialization gate and full-field work; they do not prove current native latency. Rather than widening the blocking deadline or pruning candidates, the explicit pending-calculation decision retains the one current full calculation through user Space in the existing prefetch slot and postcondition receipt. The existing worker sends one completion notification to the same engine; the existing committed-tail continuation requires both the full chooser's sealed BoundaryShift and actual exact client commit, in either order. No result from another frame or an ordinary non-boundary winner is admitted. Next pressed key and all original authority checks revoke it. Existing3500us blocking/1500ms postcondition limits remain. Focused/full source proof and renewed physical scope are pending. No new runtime bytes are installed.

The user additionally prohibited invoking remote desktop. Development uses SSH only; no remote desktop session or remote desktop tool is invoked.

Pending-calculation focused run-s4yvw9lq retained FAIL2520/2521: the new regression proved ordinary with_space layout synchronization invalidates the retained worker before the client commit. Bounded repair uses the existing current-layout composition options only when actual decoder already equals the factory decoder; otherwise retention is refused and ordinary Space follows its previous route. No required decoder change is skipped. The compilation-only run-b67xq6sh refusal is also retained; engine type re-export was missing and is corrected. Fresh focused proof is required.

Pending-calculation focused run-ujtv9ze_2521/2521PASS,208.1s. Separate native lint preflight found only the removed two obsolete field warnings and an816-byte continuation enum. Box its two owned alternatives, reducing inline state without suppressing the lint or changing authority. Repeat the fixed focused proof for final representation, then regenerate the native warning inventories; no new warning allowance is admitted. No runtime install occurred.

Final boxed representation focused proof: `/home/ubu/.cache/lay/development/run-kabon9pz/RESULT.json`,2521 selected/executed/PASS, zero failures,208.4s (697 IME owner,1817 library,7 integration). Both calculation/client-snapshot arrival orders, exact one-shot replacement, NoApply/non-boundary refusal and next-key/focus/material/deadline/text revocation pass. This source test does not prove the actual D-Bus notification or physical windows. Native lint regeneration completed for default and research-tools, no non-dead-code diagnostics and no new warning identities; only the two obsolete PreparedCorrectionLease observations were removed in each scope (532→530 and356→354). Exact comparison receipt: `/home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/pending-calculation-tools/LINT_INVENTORY_CHANGE.json`. Existing immutable client-commit decision, semantic gate and verifier remain unchanged. Fresh complete source release and actual local-window acceptance remain pending; installed IME still7b96e740.

Full source precheck peDeDW retained FAIL at `/home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/pending-calculation-full/RELEASE_GATE_RESULT.json`: required not_tested field missing from the new pending-calculation decision, so its composition_commit.rs coverage was refused. FMT and exact staging passed; no functional test or runtime installation was attempted by this full run. The bounded successor fills that mandatory evidence-limit field and records this result only. All runtime Rust, existing pinned decisions, focused2521 proof and both native lint inventories remain unchanged. Reuse those measured results and require a fresh frozen full release; do not weaken the architecture checker or overwrite the failed receipt.

Pending-calculation complete source release PASS: `/home/ubu/.cache/lay/development/space-boundary-client-commit-20261008/pending-calculation-full-record/FETCHED/RELEASE_GATE_RESULT.json`, remote QNRWlv,907.869s. All17 commands PASS;3034 selected/executed/PASS (2998 correctness,36 package), zero known semantic or infrastructure failures; MSRV1.88.0 default and lexical compiler PASS. Every3059 old discovery row remains literal, exactly one declared new correctness identity gives3060 total discovery rows. Final closure1504 files, SHA25652338c2f7f136d09335fe41ccc8830efded53efea47d23b474095e48af9afe17. Exact IME8183440B SHA2565d99e01ece1de986fb0b8269a869da6ed59b9d7ae3e77a12f5e9a01f26896615. Imported only10 approved graph/manifest/ledger exports; functional source and original pinned client-commit decision remain exact. Changed runtime installation, actual D-Bus completion notification and local-window acceptance remain pending at this entry. No remote desktop, fit, model copy or new service was invoked. Earlier failed precheck remains preserved.

Exact5d99e01e IME installed through the existing owned-empty transaction: `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/INSTALL_PENDING_CALCULATION.json` PASS. Only the IME bytes/process changed; global IBus PID270775, other actors, config, input sources and model packages remain unchanged. The exact7b96e740 predecessor is retained as rollback. Source PASS and install PASS do not prove physical delivery; fresh26-field14-case local scope is beginning with native Qt and Chrome. No remote desktop is invoked.

First six changed-byte local fields complete: Qt.entry/Chrome.input receipt `windows/WIDE_MATRIX_V2_20261008T124922.json`,28 executed26PASS2FAIL,0BLOCKED/fatal,58.485s; remaining Chrome.textarea/editable and native Qt.multiline/rich receipt `windows/WIDE_MATRIX_V2_20261008T125425.json`,56 executed52PASS4FAIL,0BLOCKED/fatal,111.444s. All48/48 connected boundary cases PASS, including actual full-provider calc, focus-v3, inverse and ordinary next-letter/Backspace/mode/icon checks; legacy30/36PASS with exactly the six preserved rnjhjt/кторое refusals. Exact installed/loaded5d99e01e PID/starttick and all observer pins bound before/after, global IBus/input sources preserved, owned inverse feedback cleanup PASS. This covers six local fields only; remaining20 fields and explicit notification trace proof are pending. A no-input attempt to launch the second group was refused by the shared RAM admission guard because its completed predecessor left a task-bound agent-browser controller. Exact pidfd cleanup proved route-file/PID/starttick/executable/cgroup ownership and preserved the owned browser: `PENDING_CONTROLLER_CLEANUP_1.json`. The actual group then ran once; no budget bypass or repeated physical pass occurred. All receipts are under `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008`.


## 2026-10-08 — Remaining local field evidence and bounded fixture repair

On exact5d99e01e, remaining17 fields: `windows/WIDE_MATRIX_V2_20261008T130943.json`,212 executed135PASS77FAIL13BLOCKED, zero fatal,554.927s. Firefox24/24 and GTK3/GTK4 four fields32/32 connected cases PASS. Gost three fields each3/8 connected PASS: their actual IBus capabilities9 exclude surrounding-text support, no SetSurroundingText is observed, and grounded previous-word scope is missing before verifier admission. The full candidate calculation is ready but correctly receives NoApply; do not weaken the verifier or fabricate surrounding text. Kitty7/8 connected PASS, focus-return pair remains unchanged and requires causal inspection. Native GNOME editor4/8 connected PASS. Three X11 editor fields fail their existing Ctrl+A/Backspace preparation before scenario input, with no observer phase: Ctrl+A under RU does not clear their field. GNOME Terminal fixture setup is BLOCKED before its direct-Python READY, not a product input verdict. Writer first two legacy cases export literal Cyrillic plus Tab; its held-Shift observer later refuses an incoherent Text/caret sample and blocks dependents. Those untested connected cases are not PASS.

Additional native Qt notification diagnostic `windows/WIDE_MATRIX_V2_20261008T125714.json` preserves1PASS1FAIL. Its calculation case received an unplanned printable keycode23/decoded ш before closing Space; visible prefix was already ошни. Exact raw archive `PENDING_NATIVE_NOTIFICATION_TRACE_1.jsonl`,SHA2560869971de082e795b9751f9f8daf437b4af4075edaa66d46e66f1e4666daceb8. The origin of that key event is UNKNOWN; this case does not prove a boundary-mechanism defect or actual deferred-completion notification. The later passive observer captured no deferred case; source regression PASS is not physical async-branch proof.

Tor fresh full14-case scope: `windows/WIDE_MATRIX_V2_20261008T131552.json`,0executed42BLOCKED,17.648s, zero fatal. First legacy Tab leaves the owned field; subsequent rows are dependent blocks. Explicit bounded replan runs the unchanged connected eight cases in fresh owned offline Tor fields, without that preceding legacy Tab. Preserve the full-scope BLOCKED result and unchanged user Tor process.

Explicit second-pass X11 fixture replan: in the already prospective `owned_visible_subtree.py` subclass only, select US before inherited physical Ctrl+A/Backspace preparation, then inherited clear restores the requested decoder. Require the same exact actual empty text/caret phase. Original23 observers,14 case texts, ordinary scenario key timing, runtime bytes, verifier and delivery contracts remain unchanged. Archive the previous extension/review, bind the new extension hash, then run exactly one fresh full14-case pass in the three affected X11 fields. Do not count first-pass preparation failures as exercised boundary inputs.

The second task-bound agent-browser controller was stopped through exact route-file/executable/cgroup/PID/starttick proof in `PENDING_CONTROLLER_CLEANUP_2.json`. The owned Chrome keeper completed after its six relevant fields; user Chrome processes and their profiles were untouched. All named physical receipts are under `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008`. Runtime authority remains the already installed5d99e01e; no remote desktop, new model or service.


### Native preparation replan after two bounded passes

X11 second pass `windows/WIDE_MATRIX_V2_20261008T132211.json`:42executed3PASS39FAIL,0BLOCKED/fatal,117.092s. Exact trace proves Ctrl+A still arrives with Cyrillic keyval1734 after coherent US IME selection; all39 failures remain before scenario input. This disproves the proposed US-selection-only preparation fix. Tor connected-only `windows/WIDE_MATRIX_V2_20261008T132359.json`:24executed0PASS24FAIL,0BLOCKED/fatal,77.793s. Its three fresh first pair scenarios retain the original text after an authorized full result;21 remaining rows fail physical preparation. No conditional pass is promoted.

Explicit fixture-only replan: after Ctrl+A invalidates any undo record, erase the exact owned collapsed line with ordinary guarded Backspace until a fresh empty text/caret is proved, at most128 keys/five seconds. Refuse non-string/oversized text, non-end caret or lost owner; no normalization, oracle change, scenario delay or runtime route change. Preserve previous extensions and all failed receipts. This prepares three X11 editors and fresh offline Tor fields using existing physical keys, then the original inherited empty-field assertion still runs. One additional full X11 pass and one connected Tor pass are planned; no repeat-until-green.

GNOME Terminal setup APP.log at `/run/user/1000/lay78-gnome.6wam8n92/APP.log` reports Failed to open PTY peer / Too many open files. The existing shared user terminal server PID3885988 is not restarted or signalled. Prospective fixture launches a separate temporary installed GNOME Terminal server under a UUID application ID and the same bounded human scope, checks its exact PID/start/executable/bus owner, and closes only its own direct-Python child and private server. All original child/PTTY/AT-SPI/text/submission checks remain. This is a temporary QA client, not a new Lay service or delivery backend. No source or installed bytes change.


### Native client preedit is UNKNOWN, final text remains independently observable

Bounded-erase full X11 pass `windows/WIDE_MATRIX_V2_20261008T133146.json`:42executed23PASS19FAIL,0BLOCKED/fatal,163.883s. Actual full-provider calculation and fresh focus-return correction PASS in all three editors. The three main pair and inverse bodies stop before closing Space: exact committed left is visible, but the native AT-SPI observer explicitly exports no preedit. This does not prove product failure. Some other rows still refuse coherent mode setup or the legacy mixed-decoder oracle; preserve them.

Prospective diagnostic uses distinct `_v4_preedit_unknown` identities for main pair, following word and inverse only. After exact independently observed unchanged committed left, guarded physical typing sends the planned right word. If that exact native owner still exports unchanged left/end caret and explicitly unavailable preedit, record preedit as UNKNOWN and press the planned closing Space. Final complete client text/caret, decoder, inverse and feedback checks are identical and independently observed. No product authority or observer identity/ownership/coherence predicate is changed; UNKNOWN preedit is never claimed observed. Original v3 failure remains unchanged. Execute these three distinct diagnostics once in the four native GNOME/Gedit editor fields. No broad source gate repeat is needed for private fixture-only changes.

Temporary native Terminal server setup succeeds and closes exactly: `windows/WIDE_MATRIX_V2_20261008T132810.json`,0executed14BLOCKED,3.316s. The original observer refuses an incoherent Text/caret sample after first-word Tab; dependents are not tested. Its direct-Python child, native server and UUID window were proved; `/run/user/1000/lay78-gnome.723ecv7z/PRIVATE_SERVER_CLOSE.json` proves private server gone, user shared server untouched. A fresh calculation-only diagnostic avoids the preceding Tab and keeps every original observation check, to determine whether the refusal is Tab-preedit specific. No observer normalization or count guard relaxation.


Native v4 diagnostic `windows/WIDE_MATRIX_V2_20261008T133514.json`:12executed5PASS7FAIL,0BLOCKED/fatal,60.337s. GNOME X11-auto main/next/inverse3/3PASS; X11-ibus main/next2/2PASS and exact inverse itself visibly succeeds, but subsequent owned feedback maintenance fails CAS after counters were written and journal concurrently appended. Preserve failed directory `OWNED_INVERSE_io_z9nj9`; Gedit's three following results are tainted by unremoved own inverse rejections and do not establish a product defect. Fresh standalone exact-row recovery reconstructs the native current counter generation, removes only the two already-pinned rejected rows, preserves all other rows/positive maps/static feedback. Private wrapper now blocks all dependent input after any cleanup failure. Require recovery PASS before any further input; then one fresh Gedit v4 diagnostic. Native shared GNOME editor three pair scenarios retain original text; no universal acceptance is claimed.


Exact owned inverse recovery PASS at `OWNED_INVERSE_RECOVERY_f4gdicx4/CLEANUP.json`:2 already-pinned rejected rows removed,873 other rows preserved, static feedback SHA2561d9a4d213f770c02c8ae8baea41bdba25e3126ff240a373762f52b3fd68f7725 unchanged. Do not relabel the failed prior cleanup or its tainted Gedit rows.

Writer explicit fresh-fixture replan: installed local primary schema `/usr/lib/libreoffice/share/registry/main.xcd` defines CapitalAtStartSentence under Common/AutoCorrect and Writer/AutoFunction/Format/Option. Disable only these competing capitalization properties in a new owned0700 UserInstallation before launch, never the user profile. Distinct `_v5_private_no_sentence_caps` identities keep original default-profile FAIL/BLOCKED separate. Run the eight connected scenarios, with v4 UNKNOWN-preedit handling only in the three appropriate cases; exact original final text/caret/owner/inverse/mode checks remain. This isolates Lay correction from the app's sentence autoformat; it does not certify default Writer formatting behavior.


Fresh Gedit after exact feedback recovery: `windows/WIDE_MATRIX_V2_20261008T133839.json`,3executed3PASS,0BLOCKED/fatal,11.957s. Main pair, following word and inverse pass, exact own rejection cleanup PASS. Prior tainted failures remain unchanged.

Terminal fresh calculation-only `windows/WIDE_MATRIX_V2_20261008T133902.json`:0executed1BLOCKED,1.811s; Writer private-caps v5 `windows/WIDE_MATRIX_V2_20261008T134005.json`:0executed8BLOCKED,7.67s. Both original observers reject Text/caret sampling during live composition; all private clients closed exactly, user profiles/shared terminal server untouched. No runtime failure is inferred from these blocked pre-Space tests.

Distinct native-commit v6 diagnostic keeps every existing observer and exact final surface/caret check. During planned right-word composition only, physical key-down uses the same complete guard_text metadata (exact PID/start/executable/UUID window/ancestry/focused visible Text and direct Python child where applicable) without requiring a live Text/caret value the client does not export coherently. Releases retain the existing release-only cleanup behavior. An actual exact original left-plus-Space is independently read before right-word input; final full target/caret/decoder/icon is independently read after closing Space. No preedit or intermediate caret is inferred. Two distinct fixed scenarios (main pair and valid original-left calculation) run once in fresh private Writer and native Terminal fixtures; final observer coherence predicates, product authority and Space deadlines remain unchanged. These are additional committed-surface proofs, never relabelled original full-matrix PASS.


Native v6 result `windows/WIDE_MATRIX_V2_20261008T134442.json`:2executed2PASS2BLOCKED,0fatal,12.516s. Fresh Writer profile main pair and calculation with valid original left both independently export exact final target/caret/decoder/icon after closing Space. This proves two committed-surface cases with Writer sentence capitalization disabled in its own QA profile; original8-case full acceptance remains BLOCKED. Native Terminal remains BLOCKED at the original coherent Text/caret read after its first prefix Space, even with correct metadata keyboard guards; no observable final result or physical acceptance is claimed. Its private server/child cleanup passes. Passive trace collector `PENDING_V6_PASSIVE_TRACE.log` observed no deferred branch; it is not actual-notification proof. Do not relax final text/caret coherence or continue an unbounded observer-repair loop. The already planned last Tor connected pass now uses a fresh output directory and the documented bounded physical preparation; original full and connected failures remain preserved.


### Additional fixed fast-typing physical race proof

Source/full3034 proof and installed5d99e01e remain unchanged. Original14-case ordinary18ms/12ms key timing is unchanged. One distinct `autocorrect_boundary_fast_pending_calculation_v7` case in GTK3.entry and Qt.entry types the attested valid-left calculation fixture. After an independently exact unchanged original-left-plus-Space, only the right word uses a declared physical8ms-down/7ms-up stress burst with the original per-key owner/focus guard. Closing Space retains ordinary18ms/12ms timing. No following key or extra Space is sent before the same1500ms exact final client text/caret/mode/icon check. Save the actual debug trace even on PASS and classify whether closing Space was really pending; a Ready-only PASS does not establish deferred completion. These two predefined cells supplement, never relabel, original ordinary-timing acceptance. No producer delay, fit, model copy, new service, runtime hook or remote desktop.


## 2026-10-08 — Final source, installed candidate, physical limits and cleanup

Frozen source release remains3034/3034PASS and exact installed IME5d99e01e/PID212279/start307415098. Read-only final1504-file binding confirms every functional source/mode/generated export exact; only this owning document has measured-result additions. Immutable client-commit decision SHA374db00b... and semantic gate SHA547bc379... remain exact. No full proof or build was repeated during these private-fixture diagnostics.

Fast pending-calculation v7 `windows/WIDE_MATRIX_V2_20261008T141006.json`:2executed2PASS,0BLOCKED/fatal,8.367s. GTK companion was Ready-only. Actual Qt Space callback46746,engine757,tail11716 retains computing generation7 without a Ready lease. Full calculation prepares after Space callback settlement (evaluation29857us,total35707us); actual original-plus-Space19-character snapshot arrives afterwards, the same committed-tail replacement follows, and independent Qt text/caret exports `они расчёты готовы ` before any following pressed key. Closing Space remains ordinary18ms/12ms; right-word-only15ms stress is distinct from the fixed ordinary scope. Exact causal packet/raw hashes: `PENDING_FAST_CALCULATION_V7_EVIDENCE.json`. This physically proves one pending-result delivery order; physical client-before-result and independently isolated notification-output branch remain untested, while both logical orders pass the source regression.

All26 planned fields were attempted. Primary frozen14-case scope has364 planned scenarios:213PASS83FAIL54BLOCKED14not exercised (native Terminal setup failure), with its setup row separate. Thirteen Chrome/Firefox/GTK/Qt fields pass all104/104 connected cases. Do not mix prospective repair protocols into this denominator. Three native X11 editors separately pass main/next/inverse v4 proofs; fresh Gedit3/3PASS and X11-ibus inverse1/1PASS after exact feedback recovery. Writer two committed-surface v6 proofs pass in its own capitalization-disabled profile. Native Terminal remains unverified, Gost lacks surrounding scope, Tor lacks the actual postcommit snapshot in the measured route, native GNOME default route and Kitty focus return remain failures. Legacy ranker refusals and observer/preparation failures are separate from source quality. Universal client acceptance is NOT MET; no accepted-version promotion is claimed.

Final Tor bounded-preparation receipt `windows/WIDE_MATRIX_V2_20261008T134925.json`:24executed4PASS20FAIL,0BLOCKED/fatal,98.877s; original full/connected failures remain preserved. Final physical/scope/receipt packet: `/home/ubu/.cache/lay/development/space-boundary-client-commit-physical-20261008/ACCEPTANCE_SUMMARY.json`.

Cleanup:22 exact own closed/failed-fixture Chrome/Tor profiles removed,513715939B, after fresh read-only argv/cwd/root/open-FD scan of all837 live processes and inode/UID checks. All source/receipts/user profiles preserved; one incomplete-provenance profile `/run/user/1000/lay78-gnome.tnsjqiec/owned-tor-profile` retained rather than guessed disposable. Exact own HTTP PID2334574/start305035042/port37457 stopped, port closed. Current native editor application metadata contains no own UUID windows; no GUI action or shared editor termination was needed. All test UInput devices closed. No recorded own inverse rejection rows remain; static feedback SHA1d9a4d21... unchanged and native counter generation current. The failed CAS cleanup and its original receipt remain historical; fresh exact-row recovery and fail-closed dependent-input protection are recorded. Global IBus PID270775/start198998438, user Tor PID838834/start247091505, user Terminal server PID3885988/start34945521 and config SHA20fe9fa7... preserved. Runtime authority remains the exact installed candidate; no remote desktop, agents, new service, model copy or fit.


## 2026-10-08 — User acceptance for publication; first-word issue remains open

The user reports that the current version works and requests a push of this
version. Publish the existing 1.0.80 source and its already accepted installed
IME, SHA-256
`5d99e01ece1de986fb0b8269a869da6ed59b9d7ae3e77a12f5e9a01f26896615`.
This is source publication only; reuse the recorded source and physical evidence.
No additional tests, builds, installation, runtime restart or client input is
part of this publication request. Historical per-client FAIL/BLOCKED results
remain recorded with their original scope.

The user-reported first-word issue remains open. Read-only inspection confirms
that the native terminal pair scope requires an earlier observed separator and
rejects a pair starting at mirrored offset zero; other first-word failures need
their own first-failed-transition evidence. The proposed first-word plan has not
been implemented and is not included in this publication. Existing correction
selection, safety checks and installed runtime bytes remain unchanged.
