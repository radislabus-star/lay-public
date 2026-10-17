# TD-131 — единый bounded-tail invariant, 2026-10-09

Status: CODE_REVIEW_ACCEPTED_PUBLICATION_METADATA_PENDING; source-only.
Baseline: `2401a4e4dca5f99cc87c42415ab52c108c26bd74`.
Task: [TD-131](../../tech_debt/131-one-unicode-tail-limit.md).
Decision: [one-unicode-tail-limit](decisions/2026-10-09-one-unicode-tail-limit.json).
Contract: C01,C03,C05,C08,C10, existing preedit/tail_memory owners.

## Наблюдение и выбранная граница

Ordinary append вызывает preedit::trim_tail_buffer с PREEDIT_TAIL_LIMIT=160.
Composition commit вызывает частную копию trim_committed_tail_buffer с local160.
Оба сохраняют последние160 Unicode scalars, удаляя prefix по UTF-8 boundary.
Настоящий trim defect не наблюдался; duplicate invariant — подтверждённый долг.
WordLineage bounded offsets опираются на тот же предел. Существующее
revocation после trim/replace остаётся отдельным admission contract.

Оставить копии:4/10; открыть существующий fixed helper только sibling:9/10,
выбрано; новый BoundedTail object/owner:3/10 для этого случая. Generic token
limit остаётся private. Изменяется только helper visibility и один consumer
call; копия удаляется. Порядок invalidation, preedit reset/rebuild, trim,
suppression refresh/retirement и publish_tail_handoff сохраняется.

## Проверка и последствия до реализации

Усилить existing tail_buffer_stays_bounded, не добавляя test identity:
159/160/161/177 scalars, точный suffix ASCII/Cyrillic/emoji/mixed combining
UTF-8, ordinary append и composition commit с/без closingSpace. Test oracle
не использует проверяемый helper. Сначала current duplicate-source GREEN
characterization; это не RED/GREEN bug fix. Затем remote affected closure,
canonical graph/canon и independent fresh-context review<=2.

Новые candidate/rank/weights/model/package/learning semantics отсутствуют.
Нового owner, state field, timer, allocation algorithm, queue, fallback или
executor нет. Latency/allocation equivalence ожидается, не измерена. Unicode
normalization и grapheme geometry не добавляются: контракт в scalars. Все
outputs/DecisionCore/verifier/SafetyGate и IME delivery routes сохраняются.
Откат — один source revert; не пользовательские journal bytes и не установка.

Installed IME остаётся accepted1.0.81/2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559.
Runtime authority NOT_CHANGED; новые source bytes не установлены. Physical,
performance/RSS и all-window acceptance для refactor NOT_TESTED. Исторический
native62PASS/2FAIL/64 и original browser24/30FAIL не переименовываются в PASS.
Exact measured outcome/receipts добавляются после выполнения, до task DONE.

## Characterization и implementation

Remote current-production characterization:707/707 PASS; wrapper wall84.8s,
remote execution74.7456s;
`/home/ubu/.cache/lay/development/run-8is73yuu/RESULT.json`.
Это GREEN обеих прежних копий с усиленным existing test identity; RED defect
не заявлен. Expected suffix oracle использует reverse/take/reverse и fixed160,
а не production helper/constant. Composition cases также сохраняют prefix и
closingSpace. Тестовая матрица:4 Unicode patterns ×4 lengths × helper, ordinary
append, composition без/сSpace. Число test identities не выросло.

Production delta: existing fixed helper получил pub(super); единственный
composition consumer вызывает его на прежнем месте; duplicate/localLIMIT
удалены. Generic helper остался private; suppression/trim/handoff order не
менялся. Affected closure/graph/canon и review ещё pending.

## Pass1 и один grouped repair

Automatic remote affected contracts PASS:3044 selected correctness/package,
all-lane command0; wrapper wall428.4s, remote419.6954s.
Receipt: `/home/ubu/.cache/lay/development/run-i1rymxbv/RESULT.json`.
Graph/canon first PASS: `/home/ubu/.cache/lay/development/tab-graph-61t7f1qw/`,
8 outputs fetched with source stability. Эти факты не закрывают task сами.
Independent review pass1 REQUEST_CHANGES7/10:
[review](../../tech_debt/evidence/2026-10-09-td131-review-pass1.md).

Единственный blocking P2 — permanent guard gap: preedit.rs не входил в exact
PROTECTED; initial ADR сам по себе не защищает future one-file edits. Grouped
repair сохраняет прежнюю composition-trimmer protection: один path entry,
existing negative test с one-file/no-new-decision subcase и explicit decision
для scripts/check_architecture_canon.py/tests/test_architecture_canon.py.
Нового guard framework/import traversal/owner нет. Before guard edit — RED
existing Python canon route, затем GREEN и fresh graph/binding proof. При
неизменных behavioral Rust hashes исходный3044 proof сохраняет scope; новые
Python guard/compiled graph metadata требуют своих проверок, не quality claim.
No-prefix composition уже covered existing committed_space_keeps_next_word_separated_in_tail_memory;
новая bounded matrix с prefix не выдаётся за исчерпывающий lifecycle proof.

Guard RED на прежнем PROTECTED:15 tests,14PASS/1expectedFAIL, только subcase
preedit.rs без нового decision. Receipt:
`/home/ubu/.cache/lay/development/tab-guard-test-hxfq3jpm/RESULT.json`;
ошибка `protected_change_without_new_decision:…/preedit.rs not found in []`.
После этого добавлен ровно preedit.rs в existing exact PROTECTED. Test identity
test_protected_change_needs_new_decision сохранена; новые runner/guards нет.
Actual all-lane source log подтверждает3044 selected,0known semantic failures,
0infrastructure failures. Следующие проверки — GREEN15 и fresh graph/binding.

Guard GREEN:15/15 PASS,
`/home/ubu/.cache/lay/development/tab-guard-test-gx5pj6xl/RESULT.json`.
Current canon link/decision check также PASS на repaired checker.
[Proof reuse identity](../../tech_debt/evidence/2026-10-09-td131-proof-reuse.json)
сверяет relevant behavioral source/fixtures/Cargo/lane hashes и modes с3044
snapshot; mismatch0. Generated architecture receipt — отдельная metadata
dependency, для неё final refresh/check и exact compiled receipt identity.
Final review pass2 и завершение metadata проверок pending. Итоговый authoritative
packet — [final source evidence](../../tech_debt/evidence/2026-10-09-td131-final-source.json);
его PENDING не приравнивается к PASS или DONE.

## Итог реализации и последний publication checkpoint

Final independent [review pass2](../../tech_debt/evidence/2026-10-09-td131-review-pass2.md)
ACCEPT9/10, P2 closed, новых correctness findings нет. Всего два прохода,
один grouped repair. Reviewer независимо сверил все931 relevant files/modes,
source/core/test delta совпадает с проверенным3044 snapshot. Manifest, ledger,
quality/functional claims и installed bytes не менялись. Guard15/15 сохранил
existing identities и exact-path policy. Это source-only acceptance; full release,
installation/activation, новые physical/latency/RSS/heldout quality NOT_TESTED.

Current compiled receipt проверен отдельной existing identity
architecture_contract::tests::generated_receipt_proves_every_required_architecture_check:
1/1 PASS,0ignored,1837filtered;
`/home/ubu/.cache/lay/development/tab-compiled-receipt-test-n4ig2mkc/RESULT.json`.
Это только metadata identity, не ещё один denominator функционального качества.
Second canonical graph/canon PASS,8fetched members:
`/home/ubu/.cache/lay/development/tab-graph-q4t6db9d/RESULT.json`.

После этой owning entry выполняется canonical publication graph refresh.
Его exact receipt, hashes/freshness, необходимость нового compiled metadata
check и фактический verdict записываются в единственный
[final source packet](../../tech_debt/evidence/2026-10-09-td131-final-source.json).
Пока packet PENDING, task не получает DONE. Завершение metadata не является
новым architecture experiment и не меняет source semantics; эта entry и packet
входят в тот же task commit. Commit/push receipt создаётся отдельно после Git
operations, не объявляется заранее. Runtime authority NOT_CHANGED, accepted2bd
сохраняется, native62/64FAIL и original browser24/30FAIL остаются историей.
Rollback: source revert всего TD131 (call/visibility/removal/guard/test вместе).
### TD-130 connected private inverse proof — 2026-10-09

The user requested Stage2 continuation. First shared tooling failure is the
historical caller's inconsistent pair selection and unchecked cleanup status;
the independent shared two-file cleanup defect remains TD-137. Compared viable
designs: parameterized shared cleanup5/10, and immutable case plus mandatory
feedback observation in the existing private consumer9/10. The latter is selected
under [the explicit decision](decisions/2026-10-09-isolated-inverse-proof-binding.json).

Actual consumer is `scripts/proof/ime-client/run.py` → `driver.py`, opt-in
`inverse-first-word` / `inverse-prefixed`. One frozen case supplies input,
expected apply/undo and changed target words. Its observed real context,
candidate identity, effects and interval bind the read-only selector. Each run
has one case and all four existing learning paths in its fresh private sandbox;
no shared host journal/counters are deleted or overwritten. Historical helpers
and receipts are not modified. Missing/partial/contradictory feedback stops the
case and cannot release dependent work.

Consequence analysis: no change to candidate lattice/rank, verifier, models,
learning weights, hot key timing, delivery routes, cache/reload or runtime
authority. Bounded journal work occurs after the measured gesture; retain the
existing1.5s feedback bound. Resource/Cargo guards and asset hash admission remain
required. New work is remote-only. Rollback reverts proof tooling, never user
data. Planned tests and real IBus smoke are NOT_TESTED until receipts exist;
this entry is implementation scope, not DONE or original host-window acceptance.

The expanded driver is an explicit V3 successor, not a rewrite of V2 evidence.
V2 SHA `d80447f21db4d689ea49d39742feb36e2202b23979d820380c1fcef916b88c12`
is preserved with commit55fd32bf/blobb9ae78c3; V1 provenance remains recorded.
The default five-case function retains its exact digest. Configuration schema
V1 remains accepted; current proof/run metadata use V3 with exact new driver and
selector pins. A first remote self-test correctly rejected the unversioned
expansion at the V2 identity guard; that is not a semantic RED or a runtime fault.

Measured first tooling verification: remote Python suite selected115, passed114, skipped1, failed0 (`run-a792hqb5/RESULT.json`). The wrong-pair equality guard was removed only in a separate temporary remote copy: exactly one named semantic test failed as expected (`td130-selector-red-20261009.json`); this is a controlled violation, not an original runtime RED.

Real private IBus pilot `td130-connected-BKOyAG/inverse-first-word/receipt.json` failed before inverse with `KeyError: character`: readiness `fixture_shift` telemetry was included in text presses. Saved effects establish one exact forward delete(-12,12) and `должны быть ` commit in the real client. Private daemon reaped, no candidate remains; no inverse/shared cleanup executed. The collector now selects actual ProcessKeyEvent text presses and rejects malformed text events; a regression covers fixture Shift, releases and other contexts. This is a tooling collector fix; delivery routes and runtime bytes remain unchanged. The revised collector suite and smokes are recorded below.

The collector revision then passed the remote suite:116 selected/115 passed/
1 skipped/0 failed (`run-e6bf4tww/RESULT.json`), including8 new case tests.
Both real private IBus scenarios passed, each with one observed forward edit,
one inverse and a complete two-row episode. Exact source and execution bindings
are in `td130-bound-inverse-execution-20261009.json`. Those receipts remain
evidence for that revision, not the following changed proof guards.

Independent review pass1 scored7.5/10 and identified two future false accepts:
oversized deletion could be hidden by client projection clamping; inverse
outputs were sliced before pending callbacks during feedback/focus-out.
One grouped repair now derives exact requested geometry and full ordered
effects from the immutable case, dispatches callbacks in the existing feedback
bound, and revalidates the full inverse through focus-out before PASS. The final
context/engine checkpoint uses the existing snapshot route once, never RPC
polling; typing/readiness/feedback deadlines are unchanged. Interval endpoints
are serialized with feedback. Semantic tests use the actual Client callback
projection, actual driver finish function and actual GLib drain with controlled
external callbacks; they do not implement a second runtime reducer. Final remote semantic proof selected122/passed121/skipped1/failed0, all14 new
identities PASS. Three isolated guard violations selected one identity each
and produced the expected1/2/6 failed assertions, with no errors or skips.
The final source hashes match the suite archive and both real private IBus
smokes; each has exact forward/inverse frames, complete2/2 feedback and no
remaining candidate after cleanup. Times are4.579s and4.621s, descriptive
samples only. [The bounded evidence packet](../../tech_debt/evidence/2026-10-09-td130-connected-inverse.json)
records exact paths/hashes and the preserved pilot failure. Final independent
review pass2 accepted the connected private proof at9/10, closing both findings
in one grouped repair; no implementation repair remains. The evidence packet
supplies the final canonical refresh/compiled metadata verdict separately.
The original host receipts and unresolved133/137 are not relabelled; physical
acceptance and shared cleanup/recovery remain NOT_TESTED. Runtime authority
and installed/loaded2bd bytes remain unchanged.

Remote proof assets are existing read-only hard-link aliases: all9 roles, model bytes copied0. The accepted2bd executable is staged only for the private factory (8185744 bytes), without installation/restart. The worker phase artifact has its own recorded SHA; this environment is not physical acceptance or equality with the user's installed model set. Exact setup/execution receipts remain private under `/home/ubu/.cache/lay/development/td130-*`.

### TD-137 source characterization — 2026-10-09

Baseline fce8978b; current owner remains the existing usage-persist writer,
native compiler/loader and historical maintenance helper. The demonstrated
failure is a CAS refusal after counts replacement, before journal replacement.
The source also leaves append-to-replace exposure after its last CAS; its
false-success effects are now measured below. Two file replacements are not
one transaction.

Design options: retry/hash polling2/10 (no exclusion); existing native rebuild
and fail-closed stop9/10 for investigation; a new writer/lock/transaction
protocol3/10 before necessity is established; future private one-case proof
isolation9/10 for the already completed130 scope, not shared recovery. Current
learning flag is the OR of logging/precognition/autocorrect, so logging-off
alone cannot quiesce the writer and all-off changes the tested feature.

Next experiment runs exact historical helper and pinned existing compiler on
remote temporary synthetic files with controlled external append at three
commit points. No user journal/config/model writes, runtime build, installation,
service restart, client launch or delivery-route change is authorized or needed.
The result must preserve refusal/partial/false-PASS distinctions and separately
record surviving-journal native rebuild. Runtime authority stays unchanged;
field ownership, live recovery and physical acceptance are NOT_TESTED. The
experiment selects a repair boundary; characterization alone is not task DONE.

Measured TD-137 result:7 controlled temporary cases matched the expected
mechanisms. The last-CAS-to-replace case returned PASS while losing the injected
positive row; subsequent native rebuild could not restore it. The observed
partial/crash/exception states remain distinct, and same-length replacement
kept source_len compatible while cached counts differed from native reference.
The production loader itself was not executed for that case. All7 quiescent
fixture count rebuilds through the existing compiler matched the exact native
reference and left journal/static feedback byte-identical. Old-row/other-owner
preservation is measured; live exclusion/hot cache/field ownership are NOT_TESTED.
[The bounded packet](../../tech_debt/evidence/2026-10-09-td137-characterization.json)
records exact source/receipt hashes and the separate failed tooling pilot.

Recommended next boundary is existing private isolation and retirement of
unsafe shared mutation9/10, compared with more CAS/retries2/10 or coordinated
writer/reader/cleanup protocol5/10. The user has been asked to choose this
complex Stage2 boundary as requested; no live action or runtime fix is inferred.
Source characterization is complete; TD-137 remains OPEN and no global user
learning files/config/models or delivery routes were changed.

### TD-137 approved retirement boundary, 2026-10-09

The user selected test isolation and stopping old live cleanup. This supersedes
the shared-recovery implementation scope with supported-entrypoint retirement;
it does not repair the demonstrated historical helper defect. The explicit
[decision](decisions/2026-10-09-retire-shared-inverse-cleanup.json) keeps one
supported inverse route: existing `run.py --remote-worker` under the remote
resource guard → private V3 IBus consumer, one immutable inverse case per
fresh sandbox, all four usage paths
set on its engine process, read-only observation and nonzero stop on missing
or contradictory evidence. No executable or production-owner change is needed.

[The retirement packet](../../tech_debt/evidence/2026-10-09-td137-retirement.json)
binds exact unchanged consumer/test/development-wrapper bytes to the recorded
TD-130 suite and two real private smokes. Bounded source inventory finds no
historical cleanup dispatch in supported proof/development scripts. Exact
historical helper/caller/snapshot paths, sizes, modes and hashes are recorded;
they remain unchanged and forensic-only. Native/physical shared-learning
inverse is BLOCKED before input; removing the cleanup flag does not admit it.
This supported-route rule does not prevent manual OS execution and does not
claim whole-machine absence of other callers. The existing atomic compositor
stand and headless private PASS do not establish native legacy parity.

Generic live recovery, hot-cache recovery, native/physical inverse and future
native isolation remain NOT_TESTED. Original native62/64 and TD-133's two FAIL
remain unchanged. Runtime authority, accepted installed2bd, user learning
data/config/models and delivery routes remain unchanged. Final independent
review and canonical remote refresh gate DONE_RETIRED_UNSAFE_SHARED_CLEANUP;
source characterization and fixture rebuilds retain their narrower verdicts.
Final review found that the default `dev-check.py client` wrapper does not
forward scenario-set. The contract therefore names the guarded direct runner
used by the exact existing inverse receipts; no executable change is needed.

## TD-136 — bounded effect map and one joined callback characterization

Status: DONE_BOUNDED_MAP_AND_SOURCE_CHARACTERIZATION. Baseline:60e0d095.
[Task](../../tech_debt/136-functional-test-gaps-and-test-file-boundaries.md) ·
[Decision](decisions/2026-10-09-cap41-focus-callback-characterization.json) ·
[Pre-test source map](../../tech_debt/evidence/2026-10-09-td136-effect-map.md).
The map review accepted8/10:12 inspected families,24 exact manifest identities
and13 source/manifest identities matching the historical3044 proof request.
This is a bounded pre-change audit, not coverage% or fresh execution. The new
test/manifest successor needs its own proof and hashes; the old map is not
silently rebound to changed source.

The measured test gap is within the inspected families: native Tab after a
first-field prefix, real FocusOut/FocusInId to a distinct context, caps41 before
the first new-field printable, actual SetSurroundingText and exact Space output.
Existing Chrome starts caps9 then gains41; existing caps41 uses direct facts.
No runtime defect follows from this gap. The selected minimal source test uses
the existing P2P helpers/reducer/WindowInteraction, with no synthetic keyboard,
additional state owner, clock, wait, protocol or file extraction. Three existing
callback primitives are composed; the small glyph helper only captures effects.

Assert old owner revocation and prefix exclusion, native Tab without text edit,
no Space frame before a fresh receipt, managed per-character CommitText, and
real Backspace/retype equal-text ABA rejection. A fresh equal-text rereceipt
then permits one exact DeleteSurroundingText(-7,7)/CommitText("работает ") and
silent handled Space release. Deterministic prepared worker evidence separates
authority/output from asynchronous worker scheduling. Adjacent stronger
selection/fragment/focus negatives remain; no duplicate broad matrix is added.

Remote original-code GREEN is characterization, not a repaired runtime RED.
One narrowly declared private legacy-output geometry violation must fail the
exact delete assertion; never change the accepted production source. Run the whole affected fixed
proof, canonical discovery/manifest/graph/canon, then independent fresh-context
implementation review, maximum2passes. Actual measured results follow below.
Installed2bd/delivery/learning/models remain unchanged. Native TD-128 and TD-133
causality, heldout/per-class quality, package-reload/cold reader semantics,
latency/RSS and universal acceptance remain UNKNOWN/NOT_TESTED, separately owned.


### Characterization results and limits

[Source evidence](../../tech_debt/evidence/2026-10-09-td136-source-characterization.json)
records all five phases and exact raw receipt hashes. Initial keycode failure
(п mapped to з) and then US-factory/RU-mode disagreement failed before the new
field correction. Fixture now starts with an actual agreeing RU factory and
bootstrap profile, not a manually restored layout after FocusIn. Original-code
GREEN:1selected/1passed/0failed,1.7603s execution; no runtime defect repaired.

The first controlled mutation removing only fresh-local-receipt SURVIVED:
1passed/0failed. Managed glyph append also clears surrounding_text_snapshot
(preedit.rs); the new sequence proves snapshot retirement and frame denial,
not isolated floor-guard sensitivity. Keep this failed sensitivity experiment
visible. It does not count as RED or justify removing the guard. Source phase
inspectors never claim a stale Space effect merely from frame=None.

Controlled legacy-output mutation changes only emitted nchars to nchars+1 in
one independent private remote source copy. The same test then fails at exact
geometry (-7,8) vs (-7,7), while authorized plan/focus/epoch/model contracts
remain unchanged. This is CONTROLLED_TRANSPORT_RED, not baseline runtime RED.
Prepared lease separates deterministic authority/output from worker latency.
The pre-test map stays bound to60e0d095; new test/manifest hashes are successors.
At that initial cutoff, whole affected proof, canonical artifact checks and code review were pending; the measured successor is below.

Canonical remote manifest refresh PASS:
`/home/ubu/.cache/lay/development/tab-manifest-m_zsgmld/RESULT.json`.
Exactly one added correctness/test/process identity, no removed or reclassified
identity and unchanged38targets:3071listed/3045required. Formatting fetch kept
terminal_delivery.rs byte-identical to the GREEN and both mutation experiments
(SHA29e23c2c…), so those scopes are reused without re-execution. Manifest totals
remain inventory, not coverage. Canonical graph and automatic whole affected
check are next, followed by the independent implementation review.


### Completed bodies and separate metadata admission

The automatic remote check retained its **FAIL/exit1**, not a generic PASS:
`/home/ubu/.cache/lay/development/run-zib5c0pj/RESULT.json`. Formatting and the
existing five-suite self-test returned0. All3045 required bodies completed
(3009correctness+36package,33targets); only the post-body known-failure admission
failed, because its manifest SHA still referenced the old inventory. The lane
receipt is BLOCKED_CONTRACT; neither receipt nor original logs are rewritten.

The new decision covers this protected metadata change. Only manifest_sha256
in known_failures.json is rebound to the discovered successor. Zero failure
records and the historical observation path/hash stay byte-equivalent in
value. That older observation is historical ledger provenance, not current
quality or source evidence. No failure/test is waived or reclassified.

[Guarded evidence adjudication](../../tech_debt/evidence/2026-10-09-td136-fixed-proof-adjudication.json)
replays278 retained harness logs with the existing parser: exact3045 identities,
3045ok,0failed, exact success totals, no missing/extra log. All1176 archived and
current relevant source/data/fixture/tooling files and modes match; known ledger
and embedded graph receipt are explicitly separate metadata. Unchanged lane
code reaches the ledger only after source stability and every harness exit/
status check. Existing load_known_failures and compare_known_failures now PASS
against the fresh guarded remote snapshot. This is **PASS_BODY_REUSE_AND_METADATA_ADMISSION**,
not a repeated execution or PASS of the failed all-command. Canon/link/decision
checks report0errors. No 3045 unchanged bodies are rerun to fix one fingerprint.

[Implementation review1](../../tech_debt/evidence/2026-10-09-td136-code-review-pass1.md)
accepted the source characterization8/10 without blocking findings; its older
whole-gate PENDING cutoff is retained. Final implementation review2 and final
canonical graph/embedded-receipt checks still gate the bounded completion.


### Final implementation review

[Review2](../../tech_debt/evidence/2026-10-09-td136-code-review-pass2.md)
ACCEPT8/10, no blockers, final implementation pass2/2. The reviewer separately
verified all278 raw log hashes,3045 unique expected successful identities,
no duplicate/missing/extra status lines,1176 file hashes/modes and the pinned
toolchain specification, plus the empty-ledger admission. Original FAILED
command and separate adjudication stay distinct. No production logic changed.
The original review cutoff leaves final graph/compiled metadata pending;
root must record their measured successor before the bounded task is DONE.


### Bounded completion and publication boundary

[Measured final metadata](../../tech_debt/evidence/2026-10-09-td136-finalization.json)
records graph/canon PASS,0errors,8 stable exports and the exact compiled
architecture receipt test1selected/1passed/0failed. Review8/10 in2implementation
passes and the body-proof/metadata successor close the bounded map plus one
source callback gap. The original automatic command remains FAIL/exit1; its
3045successful bodies are reused under the explicit ledger-only successor,
not relabeled as a fresh full-run PASS. Final docs/generated-artifact successors
require only their own canonical freshness/compiled metadata check, not another
3045-body run. Installed accepted2bd remains unchanged.

TD-128 native focus behavior, TD-133 original causal failure, package-reload/cold
reader semantics, actual worker scheduling, isolated freshness-floor sensitivity,
quality/per-class heldout gates, latency/RSS and universal acceptance remain
OPEN/UNKNOWN/NOT_TESTED in their owning scopes. Larger file extraction has no
measured benefit here and remains a stage2 decision. Completion of the bounded
source task does not erase these limits or change runtime authority.


## TD-133 — prospective original-activation setup contrast

[Prepared plan](../../tech_debt/evidence/2026-10-09-td133-original-activation-plan.json)
binds the unchanged parent probe, a minimal private successor and prospective
review9/10 pass1/2. It has NOT_EXECUTED/NOT_AUTHORIZED status: exactly2 streams
are proposed with original18/12ms input and1.5s visible deadline, restoring the
original same-RU normal ActivateLayout/select setup after raw clear. No new
production code, authority, delivery route, inverse/cleanup, install/restart or
remote desktop. Root cause remains UNKNOWN; full predecessor state is not
recreated and a successful contrast cannot prove a cause or close original64.
Earlier2 visiblePASS and original62PASS/2FAIL/64 remain separate unchanged facts.
Both old traces share InputContext_7; exact native PID/context binding and
pre-key Ready remain UNKNOWN. A fresh explicit local-test exception is required
by DEVELOPMENT.md; the prior one-shot grant is consumed and TD137's isolation
selection grants no new execution. This is a preparation record, not a runtime
experiment or proof. Further dependent execution remains stopped.


### Authorized measured successor

The user later granted exactly one original-activation diagnostic2. It ran
once using the exact reviewed ff3d3a19… script; the preparation packet/review
retain their previous NOT_AUTHORIZED cutoff. [Actual result](../../tech_debt/evidence/2026-10-09-td133-original-activation-result.json)
records2/2 committed-text/preedit/caret controls PASS, exact13 press codes/decoded characters per
field, callback serial/time joins and actual7-char/cursor7 surrounding receipt
between native first Space(false) and next right-token letter. Runtime
PID/starttick/2bd and RU tuple are preserved; debug logging restored; cleanup
errors0. No inverse/shared cleanup, install/restart, model copy/fit or RDP.
The new local grant is consumed. Source authorities/delivery routes unchanged.

This is not unqualified rendered acceptance: GTK's assertion snapshot has
`render_text=должен ыбыть ыбыть`(available=true,sequence52). A later retained
state(sequence82,1.275739099s later) has the matching `должен ыбыть ` layout.
Both observations remain in the result packet; exact settlement time and
continuous/pixel correctness are unproved. Qt render_text is unavailable.
The fixture's synchronous signal publication plus40ms timer is a possible
sampling-order explanation only, not a proved runtime or fixture cause.

ORIGINAL_COMMITTED_SURFACE_FAILURE_NOT_REPRODUCED; first original failing transition remains UNKNOWN.
Original62PASS/2FAIL/64 and previous standalone2 successful controls remain
separate. Same-RU normal activation alone did not reproduce the original
committed-surface failure in these2 observations;
full predecessor/warm/learning state was not recreated. No field-PID/context
or pre-keyReady proof is invented. Existing mixed native Space/IME printable
source paths are only an ordering hypothesis, not a measured causal defect.
No production repair is selected. [Final checkpoint review2/2](../../tech_debt/evidence/2026-10-09-td133-original-setup-review-pass2.md)
accepted9/10 without checkpoint blockers; its score applies to the bounded
diagnostic evidence, not original causality, rendered acceptance or TD133 DONE.
