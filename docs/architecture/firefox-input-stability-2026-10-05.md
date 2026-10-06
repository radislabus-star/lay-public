# Firefox input stability — 2026-10-05

## Current route and goal

Source: lay-contextual-completion-20261005, commit8c5cb609. Actual installed
IME4a48c5526f62bd52555ff312d1ba9ec8ca72656f211fdbf1726c6516af93cdd1,
daemon4a61557717887e5daa5a2929641f8005d4156293d93e4c10ef147d49fad9ccef.
Current owner is ContextAdmissionReducer / WindowInteraction, manual detector
is daemon ManualToggleV3. C01/C03/C04/C06/C08/C09/C10 apply. User authorizes
Firefox repair and repeated diagnostic trials with exact rollback; global IBus,
input source IDs and input/edit owners remain protected.

Goal: retain safe text authority across legitimate Reset/key callback schedules
and prove Tab+one space, Backspace/retype/Tab, held Shift, eight individually
visible Double Shift pairs with actual next-letter/Backspace and RU/EN icon,
focus return and applied auto-flip undo in owned input/textarea/contenteditable
fields. Source PASS, installed bytes and physical acceptance remain distinct.
100-command protocol is frozen privately; user changed priority to Firefox
before the100-command run. Do not present that protocol as executed evidence.

## Established facts and unresolved boundary

Old79 physical textarea pair6 (current Reset source unchanged) refuses text
authority after replay; release41836 accepted before Reset41837. Merged
missing_predecessor_or_post_reset_token cannot establish which option or
predicate failed. Minimal controlled2/2 and extended3/3 serial regressions PASS
but do not reproduce that physical refusal. Prior success-logging diagnostic
trials did not reproduce; do not treat this as a repair.

Current79 baseline V17_RUN_20261005T210502_053fd288844e: input/textarea each
pass8word/icon pairs+nextletter. Editable fails before gestures while its
observer snapshot is stale; later state is exact ghbdtn/focused. Original FAIL
is preserved, classification is an observer limit, not a proved owner failure.
Current required composition run210730_39392a22665d: textarea/editable each
PASS8pairs+8actual letters+8Backspaces. Input BLOCKED because the pre-Backspace
snapshot cannot separate the typed probe from the changing rendered suffix.
No production change has occurred. These successes do not erase old failures.

## Selected diagnostic and two alternatives

Selected: failure-only, nonempty Reset capture/arm metadata in the existing
opt-in trace. Original predicates evaluate first; only an already rejected
branch emits diagnostic metadata. No success record, new detector, callback,
RPC, queue, timer, cache or authority is added. Capture reasons and option
presence distinguish the first unknown transition on actual callbacks.

Alternative1: repeat the detached serial fixture. It is viable for regression
closure but cannot distinguish the missing physical capture predicate; already
PASS. Alternative2: success-and-failure V2 logging. It can sample all states,
but unnecessary work perturbs the race and previous trials were inconclusive.
Failure-only logging reduces that cost. Neither alternative justifies removing
a safety check or selecting a speculative runtime repair.

## Consequence check before code

- Lattice/ranking/false authority: diagnostic cannot create/drop/rerank candidates
  or grant text rights. Return values and predicate order remain identical.
- Latency/tails: no successful capture logging. Rejected nonempty capture adds
  one bounded snapshot/JSON record through the existing logger; timestamped
  diagnostic observations are after-predicate samples, not atomic witnesses.
  Async schedules can still change; NOT_REPRODUCED stays inconclusive.
- CPU/RSS/allocation: counts/flags/owner paths only, bounded to existing state;
  no corpus/model scan. Trace append/serialization are the only extra work.
- Cache/reload/package: existing config trace cache, packages and candidate
  revisions unchanged. Future package/delta updates cannot grant extra rights
  through this diagnostic; current input hashes remain protected in trial.
- Learning: no new event/classification/weight. Physical Tab/Enter tests may
  create normal owner-controlled feedback, separated from diagnostic authority.
- Concurrency: local state sampled under the existing callback lock; no new
  reducer snapshot/revalidation/await. Preserve observer-before-handler order.
- Failure/rollback: only temporary IME bytes; exact current79 backup and fresh
  empty owned GTK field before exchange and restore. Retain stale/rejected
  attempts; no global IBus/L3/L11 restart or live-profile replacement.
- Compatibility: daemon and existing IBus contracts unchanged; same legacy
  callbacks and ManualToggleV3. Metadata has no raw user text.
- Maintenance: diagnostic is a private patch, not default release behavior.
  Remove it on exact finally rollback; a causal repair needs a new decision,
  controlled old-vs-new regression and full gate/physical proof.

## Finite proof and promotion boundary

Before trial, check exact baseline source1472 rows, sole observation metadata
delta plus owning docs/decision/generated graph, fmt/canon and guarded remote
IME compile. Bind toolchain1.97.1/research-tools, immutable artifact SHA and
backup SHA; root is sole20CPU heavy executor. No compile receipt is a fix.
One diagnostic scenario pass across all3Firefox fieldtypes, retaining first
nonempty refusal with serial, owner/token/scope/count and arm-option metadata.
If no body refusal, report NOT_REPRODUCED rather than patching speculation.
Any real repair must preserve stale/foreign owner/caret/selection refusal,
exact target retention, physical single-owner detector and verifier authority;
run changed/full release gates and obtain exact changed-byte physical proof.

Evidence/private audits live under release79/firefox-repair-20261005; source
audit CURRENT_RESET_SOURCE_AUDIT.md SHA726da33c6c983a8f604688415911c768d2ce07ebfa98c87c5798179a65a51942.

## Current measured controls after preflight

Baseline V17_RUN_20261005T212109_833b68730035:27/33 PASS. Three apparent
Backspace/second-word Tab failures stop at the pre-Tab DOM event-floor wait;
Tab was not dispatched. Actual applied-worker generations and new tail epochs
republish the same full word/cursor after Backspace; Firefox need not emit a
new compositionupdate when the rendered text is unchanged. This is a test
readiness defect, not measured Tab refusal. The other3failures produce кторое
instead of lexical correction которое; separate candidate/decision diagnosis
remains pending and is not a client ownership failure.

Prospective owned-field native-Tab contrast V17_RUN_20261005T212856_4721b211359d
PASS3/3: same scenario, exactly one physical Tab, exact word+one space, ended
composition and exact caret in input/textarea/contenteditable. Only the
diagnostic stimulus allows retained event; pre-dispatch candidate authority
UNKNOWN. Existing strict checks/old receipts unchanged. First shim run
V17_RUN_20261005T212809_20c916601458 selected0because renamed case missed its
filter; retained as zero execution, then explicit case-name binding corrected.
No runtime behavior changed for these controls.

Remote diagnostic-only build PASS:2focused tests,1520closure checks, only
declared observation/doc/generated deltas; artifact5b6878582a9e90b1a9a7a87e43c7b707a024e84531ac25d9e1b74f70faf60783.
Temporary diagnostic run REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261005T212958_35fcc8b4121b
PASS3/3, each8word/icon pairs+8actual letters+8Backspaces; new prospective
probe observer retains exact typed/caret/focus proof under one KEY_F deadline.
Exact current79 installed/loaded IME4a48 restored, original mode/source/engine
and protected identities verified by trial-failure-only/TRIAL_RESULT.json.
Old Double Shift refusal NOT_REPRODUCED; no production repair selected.

Failure-only metadata nevertheless separates a nonempty rejection: local suffix3
versus UnknownStart lineage0, token matches owner and scope; candidate absent
but post-reset token present. This is a real intermediate state in a passing
trial, not proof of the old terminal failure. Next discriminant: actual callback
ordering/current recovery versus old release41836-before-Reset41837, then a
controlled production-adapter regression before any authority change.

## First failed transition established on current bytes

Current79 source4a48 production proof
REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261005T213513_7c328aaa27b3: input pair5
FAIL; textarea/editable8pairs PASS. Input Reset530 observer ingress precedes
sixth native insert528; mirror6 changes, settlement528 refuses, later stamp
timeouts and subsequent exact6 cannot restore missing predecessor.
Independent temporary diagnostic input2
REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261005T213857_5814afa32f8f: pair6 FAIL,
exact4a48 rollback PASS. Prefix4 press375 settled; observer queued376..380
and Reset381 precede native E377 and T379. Release376 retires harmlessly,
press377 mirror5 settlement refuses and local revoke clears stamps. Native
press379 still mirrors6 but no received callback remains. Capture rejects
local6/Unknown0 with token_scope=false, later missing predecessor token.
First failed transition is settling a received native replay insert retired by
an already observed word Reset, before capture. Changing suffix equality would
mask lost proof. These two failures share a mechanism; old release41836 before
Reset41837 remains a separate UNKNOWN, not retroactively explained.

## Repair design boundary before production code

Selected hypothesis: retain only bounded native append provenance in the
existing pending Reset receipt after authenticated same-owner Reset retires
received inserts. It stays unconfirmed and cannot edit, predict authoritatively
or learn positively. A later authenticated Reset and exact collapsed client
receipt must align the projected tail epoch through the existing reducer before
restoring observed suffix authority. No second owner/executor/detector/source ID.
Implementation is NOT selected yet: current generic revoked-key predicate does
not prove Reset origin, and unchanged-epoch soft settlement cannot accept the
projected epoch. Neither condition may be loosened. A typed proof extension of
existing owner state needs explicit admitted origin and epoch-chain verification.

Alternatives: generic retired-key skip avoids a second poison but loses epoch
and predecessor; insufficient. Delaying replay keys or adding a client RPC/
retry route would hide event-order failure and change latency/transport; rejected.
No new generation, timer, queue/cache or broad recovery fallback is justified.

Consequence analysis: candidate/ranking/model/learning routes and packages stay
identical. Only a proved Reset/native-replay state transition may retain inert
provenance. No false authority from dispatch, mirror, timeout or projection.
Unknown/foreign owner, FocusOut, selection, wrong surface, ContentType/sensitive
change, stale activation/serial/token, discontinuous epoch or incomplete replay
must fail closed. Existing verifier/SafetyGate and output plans remain unchanged.
New work must be bounded to existing callback/receipt state; no per-key IO/model
scan. Source/RSS/latency and mandatory release/physical gates remain required.
All callback evidence and current token/epoch must be coherent under existing
locks; no forced two-thread gap inside a synchronous executor segment.
Rollback remains fresh owned-empty exact installed bytes, preserving configs,
models, global IBus/L3/L11 and input IDs. Maintenance proof must reject old code
on the actual observer-first adapter schedule and retain sequential controls.

Next private experiment:4settled native inserts → observer receives remaining
key headers and Reset before handlers → actual existing native callbacks →
authenticated Reset+exact6 → probe/Backspace → ManualToggle. No token/scope
assignments. Also selection, wrong surface and FocusOut negative schedules.
Root is sole guarded remote executor; no production Rust modified yet.

## Controlled observer-first causal regression — executed

Remote `firefox-observer-first-20261005/RESULT.json`: OLD_FAILURE_REPRODUCED;
33.793s, exact HEAD8c5cb609 and 1472 baseline rows; production bytes unchanged.
Positive `firefox_observer_first_native_insert_reset_recovers_only_after_exact_receipt`
actually FAIL1/1 with expected `observer-ahead Reset lost validated replay provenance`.
Negative selection/wrong-surface/FocusOut each PASS1/1; three unchanged FULL_V2
sequential controls each PASS1/1. Compile/discovery4/nonzero/source closure PASS.
Local receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/observer-first-import/RESULT.json.
This proves the original adapter schedule, not a repaired build, quality or physical acceptance.
No runtime authority changed.

## Selected causal repair, before implementation

Use the existing ExactReplay suppression to retain its already admitted source
token and bounded source epoch. The existing reducer records the last generic
revocation using its existing revocation counter; authenticated Reset alone
preserves that watermark. ContentType Set, missing input, malformed ingress,
foreign connection/profile and ordinary revocation invalidate old provenance.
FocusOut is excluded immediately by existing pending lifecycle/activation checks.
This watermark is necessary: current revocation cannot distinguish Reset from
changed ContentType; storing only equal content values misses change-away/back.
It adds no generation, owner, timer, queue or controller.

Only a received key retired by that Reset-only chain, carrying either the actual
validated native replay effect or an unchanged release, skips the second poison.
It does not settle a key, advance reducer epoch or grant word authority. The
received Reset remains available. Its actual callback may arm a completed replay
candidate in the existing pending receipt. Require complete exact plan, local/
shared identity, unexpired existing lease, original admitted token/epoch, bound
external prefix, current owner/activation and fresh collapsed exact target.
At first exact receipt the existing reducer aligns only latest_tail_epoch;
Unknown0 remains unchanged. Consume this completed witness once. Existing soft
Reset suffix and epoch equality, verifier, edit plans and output routes stay
literal. Further managed input/probe/Backspace uses the existing pending path.

Alternative retaining a per-key projected pending witness needs more state and
origin/epoch handling; completed replay reuses the existing bounded projection.
Generic retired-key skipping cannot establish Reset origin and is rejected.
Independent design audits identified expiry, ContentType continuity and pending
lifecycle as mandatory gates. Old unrelated41837 remains UNKNOWN. Required proof:
old positive RED, repaired positive GREEN, adverse selected/wrong/focus/content/
incomplete/stale/malformed paths reject, sequential and route guards unchanged.

## First changed-source focused run, rejected before promotion

Remote firefox-reset-repair-focused-20261005/RESULT.json: ACTUAL_TEST_FAILURE,
33.882s; compile PASS, 4 physical_double_shift_owner source guards PASS1/1 each.
All 7 causal/negative/sequential tests FAIL at shared legacy_key fixture observer:
its wire body was (), while separately supplied callback arguments were triples.
Strict malformed-input invalidation correctly rejects this inconsistent fixture.
No changed-byte repair acceptance or runtime change. Preserve this failed receipt.
Fix semantic fixture transport to send the real key tuple; rerun original-source
RED with corrected transport, then repaired-source GREEN. No assertion/deadline
changes are admitted. Original old RED is historical, not the corrected-input proof.
Independent review repaired concrete source defects before further execution: retain
completed witness across repeat Reset; consume source proof in local/shared scope
once after exact receipt; capture/revalidate the original admitted token rather
than mint a successor after a same-epoch revocation. Source review PASS_WITH_LIMITS,
no runtime/physical claim. Existing reducer/soft-reset/output contracts stay intact.

## Corrected real-tuple RED/GREEN, executed

Frozen OLD `firefox-observer-first-typed-v2-20261005/RESULT.json`:
OLD_FAILURE_REPRODUCED,34.780s; exact original production rows/modes preserved,
positive FAIL1 with original marker; other10 PASS. Only shared test key transport
and causal fixtures/docs differ from original. Frozen repaired
`firefox-reset-repair-focused-v2-20261005/RESULT.json`: ALL_PASS,34.931s,
11/11 actual executions: positive recovery,3 negative,3 sequential,4 explicit
physical_double_shift_owner guards. Same real key tuples and assertions.
Local immutable imports: firefox-repair-20261005/{old,green}-typed-v2-import.
Source review blockers0 after recorded repairs; still no changed-byte physical
or release acceptance. Additional non-Reset/expired/incomplete guards and full
mandatory gates remain pending. Installed exact79 unchanged.

## Additional adverse execution and fixture exchange repairs

GREEN3 actual16/17; only malformed-negative fixture timed out because it assumed
cancelled bridge refused before Ping. Actual begin_bridge_fence always sends Ping
then publishes marker before complete fence cancellation; source confirms this.
The repaired fixture responds to exact actual Ping and requires exact Cancelled,
unchanged timeout, no word rights/output/learning. GREEN4 actual17/18: all six
new adverse cases PASS; an existing strengthened malformed/read-only negative
failed its empty-output assertion because bridge marker remained undrained.
Consume only its exact signal path/interface/member/nonce; keep empty client
output assertion literal. These are fixture exchange failures, no runtime PASS.
Header-only normal key fixtures now send valid (uuu) zeros; semantic key helpers
send their actual callback tuple; explicit malformed fixtures retain unit body.
No source authority changed since corrected GREEN2. Receipts retained at remote
firefox-reset-repair-focused-v{3,4}-20261005/RESULT.json and local imports.
Final focused and release/physical acceptance are pending.

## Final focused source proof

GREEN5 ALL_PASS18/18,36.767s, corrected typed causal/6adverse/oldread-only
negative/3sequential/4physical-owner contracts; no ignored or zero executions.
Remote firefox-reset-repair-focused-v5-20261005/RESULT.json; local green-v5-import.
Original corrected-input RED remains positive FAIL+10PASS, production8c exact.
Final source closure sealed; only guarded remote rustfmt imported. Generic soft
Reset equality and SafetyGate/output routes remain unchanged. Full release gates
and changed-byte physical acceptance still pending; installed79 unchanged.

## First release-source attempt — canon refusal preserved

Remote firefox-reset-release-80-20261005/RELEASE_GATE_RESULT.json FAILED83.138s.
Fmt/fetch/discovery/fresh AST graph PASS; canonical discovery old rows literal,
new delta exactly11 tests, zero known-failure exceptions. Canon refused protected
committed_tail.rs missing from ADR path list. Its only code change initializes
source_token=None in an existing expired replay test fixture; no runtime route.
Add the actual affected test/struct paths to the existing new decision. Do not
weaken canon, reroute code or reinterpret this as release acceptance. Latest
source80, full tests/build and physical acceptance remain pending.

## Second release attempt — version declaration mismatch

Remote firefox-reset-release-80-v2-20261005/RELEASE_GATE_RESULT.json FAILED109.943s.
Canon PASS after explicit affected-path declaration. Changed gate stopped at
version consistency: source tray_support.js APP_VERSION remained1.0.79 while
Cargo and metadata were1.0.80. Update the existing declaration to1.0.80; no
Rust or installed extension/runtime changes. Source tests/full build/physical
acceptance remain pending; preserve failed logs in release80-v2-import.

## Third release attempt — nested fixture future stack abort

Remote firefox-reset-release-80-v3-20261005/RELEASE_GATE_RESULT.json FAILED276.739s.
Canon/version/fmt PASS; daemon270/270 and IME bulk458/458 PASS. The isolated
td121_aborted_second_exact_capture_does_not_consume_enter process aborted with
stack overflow before a test status; canonical verdict BLOCKED_INFRASTRUCTURE,
not semantic PASS. Exact process log retained in release80-v3-import.
The common word-scope bounded helper embeds the complete large nested caller
future inside another async frame and race; new proof fields increase those
inline frames. Candidate fixture repair boxes work in the same bounded race,
without an additional async enclosing frame. Preserve default stack, Timer1s,
all assertions/test identities and production source. No special failing-test
branch, stack increase, ignore or ledger waiver. Test the exact original abort
and all18 focused contracts before another full gate. Runtime79 unchanged.

## Bounded fixture allocation proof

GREEN6 ALL_PASS19/19,37.213s: the formerly aborting exact second-capture test
now executes PASS with the default stack and unchanged1s deadline. All18 causal,
adverse, sequential and single-detector source contracts remain PASS. Production
Rust unchanged from GREEN5; the sole additional source delta is common test
future allocation. Remote firefox-reset-repair-focused-v6-20261005/RESULT.json,
local green-v6-import. Independent allocation review PASS_WITH_LIMITS/blockers0.
Full release and physical acceptance still pending; live79 remains exact.

## Separate fast Space correction observation

Baseline79 V17_RUN_20261005T212109_833b68730035 rnjhjt cases use RU decoding
and produce кторое: this is an omission case, not layout restoration. First
measured refusal is correction lease not_ready at Space (3450/3483/3472us),
followed by managed_fallback_commit of the literal word. Editable full worker
gen13/epoch4502 completes after Space as superseded. Candidate presence/rank
remain UNKNOWN; trace.rs candidates=0 is a hardcoded telemetry field. Do not
call this an empty lattice or rank error. This repair does not change the
correction budget or authorize a late edit. Next discriminating evidence is
that same final-generation lattice/Apply/NoApply reason, separately from
publication readiness. Baseline exact logs and failures remain preserved.

## Fourth full attempt — systemic safety regression, rejected

firefox-reset-release-80-v4-20261005/RELEASE_GATE_RESULT.json FAILED471.778s;
changed suite BLOCKED_CONTRACT with exactly one unexpected semantic failure:
td121_batched_reset_callbacks_cannot_restore_invalidated_provenance, text gap.
IME667selected/1failure. Preserve release80-v4-import/process failure; neither
ledger waiver nor runtime installation is allowed. First lost safety boundary:
contradictory SurroundingText clears pending but retains local/shared ExactReplay
source_token; later Reset creates completed fallback and matching receipt can
resurrect it. Prior one-shot spend only covered confirmation, not cancellation.
Repair the existing cancellation boundary: definitive rejection spends the
current matching local/shared seed through existing consume operation, including
pending with completed_replay=None. Retain intentional prior-surface/strict-prefix
branches and repeated Reset; no blanket reset/revoke, new controller or route.
Also cancel seed on actual capability/content/input-gap invalidation. The exact
old negative's six gap variants and all prior focused proofs must pass before
new mandatory full verification. Source claim remains rejected; installed79 exact.

## Cancellation focused proof and capability review

GREEN7 ALL_PASS22/22,39.239s: original batched invalidation negative passes all
six gaps; seven prior-surface contradiction variants and positive batched
receipts pass, alongside all19 prior contracts. Runtime79 unchanged. Source
review additionally found raw PREEDIT bit could change while surrounding/exact
refresh stay unchanged, avoiding legacy-preedit derived-condition detection.
Add independent raw-bit cancellation; strengthen the original batched negative
with that seventh variant (surrounding remains enabled). Original six variants
and all assertions remain literal. This new source is not certified by GREEN7;
final focused/full gates and physical acceptance pending.

## Final cancellation/preedit source proof

GREEN8 ALL_PASS22/22,39.254s. Original batched negative now passes all seven
gap variants, including raw PREEDIT-only toggle with surrounding retained;
all previous assertions/deadlines remain. Positive observer-ahead recovery,
repeated/batched Reset, incomplete/expired/malformed refusals and four physical
DoubleShift source ownership guards remain PASS. Local green-v8-import and
remote firefox-reset-repair-focused-v8-20261005/RESULT.json bind exact source.
Only guarded rustfmt imported. Failed full v4 remains rejected/historical;
new full gates and changed-byte physical acceptance still pending. Live79 exact.

## Fifth whole-suite attempt — independent normal proof must survive gain

Before changed-suite closure IME667selected reports two failed existing source
contracts: firefox_deferred_manual_refresh_retains_retired_preedit_from_previous_prefix
and firefox_manual_toggle_defers_until_the_client_replies_after_rpc. Both enable
preedit/exact-refresh after an independently settled normal Reset predecessor;
blanket new capability cancellation discarded that valid pending proof. Reject
this source; installed79 unchanged. This establishes that raw capability gain
is not a universal field identity gap. Split existing helper actions by typed
provenance: capability/preedit/exact-refresh change retires the newly retained
ExactReplay seed and any pending completed_replay that depends on it; preserve
independently settled normal pending. Actual surrounding loss, content/input
gaps and contradictory receipts still discard all pending plus the replay seed.
The newly added seventh raw-PREEDIT variant assumed the wrong universal oracle;
replace that new variant with a dedicated observer-ahead seed-only negative,
keeping the original six variants and all pre-existing assertions literal. Add
both existing deferred-preedit positives to focused proof. No runtime word/app
exceptions, ignored tests, waived failures or weakened safety conditions.

## Typed cancellation replan and withdrawn new oracle

Full v5 failed475.050s, CHECK_CHANGED391.256s, exactly the two existing
deferred-preedit contracts above (release80-v5-import). Earlier GREEN8 is
focused evidence for rejected bytes, not promotion evidence. Withdraw only
the newly added seventh universal capability variant: terminal_delivery.rs
is now identical to HEAD, with all original six gaps and assertions preserved.
The original positive contracts remain unchanged and become focused controls.
Capability changes spend the matching added replay seed and pending only when
completed_replay=Some; independently grounded normal pending=None survives.
Definitive surrounding/content/input/snapshot contradiction still clears both.
No new owner, transport, deadline, mutation, fixture-specific runtime rule or
ledger exception. A separate appended native observer-ahead fixture must prove
raw PREEDIT retirement of this specific seed before another full gate. Current
source is NOT_TESTED; installed exact79 remains unchanged.

## Typed cancellation focused proof

GREEN9 ALL_PASS25/25,39.936s. Both original normal deferred-preedit positives
pass unchanged alongside the new native raw-PREEDIT seed-only negative and all
22 prior contracts. Old terminal_delivery source remains literal HEAD. Exact
remote firefox-reset-repair-focused-v9-20261005/RESULT.json and local
green-v9-import bind source before/after; only guarded formatting imported.
Full gates and changed-byte physical acceptance remain pending; installed79 exact.

## Transport admission failure before source execution

Release v6 FAILED0.188s at input_Cargo.lock; no clone/build/test executed.
Python tar data-filter stripped group-write bits on all seven input overlays
(0664→0644), while every payload hash matched. Preserve failed receipt in
release80-v6-import. Next fresh transport restores only declared file modes
after archive SHA and input hash validation; release admission stays literal.
This is input transport repair, not a source/runtime functional change.

## Both complete semantic gates pass; bounded proof representation replan

Release v7 FAILED921.267s: CHECK_CHANGED400.386s and full canonical2993/2993
PASS, then strict Clippy rejected the new inline admission proof enlarging
AutocorrectSuppression to296 bytes (CurrentWord24), plus unnecessary mutable
fixture borrow. Exact failures preserved in release80-v7-import and guarded
CLIPPY_DIAGNOSTIC.err; no runtime installation. Clippy/gate not waived.
Alternative1 boxes the entire existing ExactReplay variant: wider caller and
allocation migration. Alternative2 boxes only the newly added inert source
proof: preserves every existing variant/caller and proof value/equality; chosen.
Allocate the optional proof before Shared lock. Existing one-shot invalidation,
owner/revision/expiry checks and source authority stay literal. No detector,
queue, owner, deadline or output route added. Reborrow shared test harness as
immutable without assertion changes. Require focused proof plus strict lint
before another whole release gate; source/full/physical claims remain pending.

## Bounded proof representation independent review

Value/equality/one-shot review finds semantic blockers0: Box compares immutable
AdmissionToken values, as_deref preserves original references, source clone into
pending remains the same unboxed value. Initial proof allocation is before
Shared lock. Derived replay clones can allocate another bounded proof Box,
including existing Shared scope cloning; no allocation-free claim is made.
This affects active existing replay only and disappears after seed consumption.
Immutable Arc would avoid these additional clone allocations but introduces
shared reference-counted storage; zero-extra-allocation is not a current gate.
Retain bounded Box implementation and require real-input timing/focus/output
acceptance under unchanged deadlines; no new allocation/polling loop or owner.

## Bounded representation focused and early strict lint proof

GREEN10 ALL_PASS25/25,40.095s. Guarded check-lay-lints.sh then PASS for both
default and research-tools scopes: original inventory and warnings-denial stay
unchanged, source rows literal after lint. Local green-v10-import retains all
logs and raw proof. No runtime/graph import or source release yet; full release
and changed-byte client acceptance remain pending.


## Full v8 source gate and first changed-byte physical trial — 2026-10-06

Measured full v8 PASS_FULL_SOURCE_RELEASE_GATE,1258.344s; canonical correctness
and package selection2993/2993 PASS, strict default/research lint, source canon,
installer/public-issue regressions and Rust1.88 compatibility PASS. Exact receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v8-import/RELEASE_GATE_RESULT.json`.
Immutable IME SHA2566a01e6b73374c9eb9ecd7ee5380f4b9f08a8da684d8f4f6a3043e065cd25537d.
This is source proof, not physical acceptance.

First temporary physical trial executed three owned Firefox field types.
Input and contenteditable PASS8/8 individually observed pairs each, including
actual next-letter input, actual Backspace, decoder and rendered GNOME icon.
Textarea FAIL at pair1: six initial letters appear literally without composition;
first key already passive/refused, before any replay in that field. Thus the
prior observer-ahead native-replay repair has a positive bounded physical result,
but this candidate is not admitted for stable installation or publication.
Matrix receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/windows/REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261006T002610_eea0ca2f31d2/WIDE_MATRIX_V2_20261006T002647.json`.
Trial receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/repair80-trial-c1c6f978f9a2499e866f1f53d7d7b6b0/TRIAL_RESULT.json`.
Finally rollback PASS_EXACT79_RESTORED; source/protected runtime identities
unchanged. Scope was temporary IME80 with79 CLI/extension/services. No global
IBus restart, input-source migration or stable80 installation occurred.

New first failed transition is lifecycle admission, not candidate ranking or
replay: transfer owner20 installed at trace L2734; FocusOut605 callback timeout
L2735 precedes observer ingress L2737; observer Disable606 L2738 and FocusIn608
L2739 arrive; Disable callback L2741 precedes FocusIn608 timeout L2744. First
key612 passive/refused L2749–2751; ManualToggle later rejects bridge_admission,
lifecycle_pending/revoked with zero unsettled keys. Trace source:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/windows/REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261006T002610_eea0ca2f31d2/traces/20261006T002634_firefox.textarea_each_shift_pair_real_letter_delete_composition/after_flush_ibus_engine_debug.jsonl`.
Exact interleaving of cleanup and observer publication is not logged. Hypothesis:
delayed source callback globally revokes the same retained owner and clears the
later authenticated FocusIn stamp. Require deterministic production-entrypoint
RED and negative transfer/owner guards before any additional runtime edit.

Replan: compare (1) increasing timeout/retrying activation, rejected because it
changes deadlines and hides the lost successor; (2) local-only cancellation on
all failures, not admitted because missing lifecycle observation cannot retain
old word/transfer authority; (3) preserve later authenticated lifecycle stamps
while retiring failed source word and seal through existing reducer/adapter,
preferred only after causal proof. Keep existing owners, exact sender/path/context
checks, original1ms callback budget,50ms activation budget and output routes.
New ready grants must be UnknownStart until fresh input proves a word. Broad
93-case/retained-Tab trials, actual Instagram/WhatsApp and changed-byte real
keyboard confirmation remain NOT_RUN/NOT_TESTED, not inferred from local fields.


## Retired Disable causal RED and narrow implementation

Controlled old-v8 production RED reproduced34.896s,7 exact tests: expected
positive FAIL with `retired Disable erased successor FocusIn stamp`,6 existing
focus/DoubleShift controls PASS. Original residuals preserved byte-for-byte;
only a new fixture appended. Receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/disable-red-v1-import/RESULT.json`.
The fixture uses real expired FocusOut callback and actual observer ingress,
valid-wire FocusInId, then controlled Reset retiring the source before delayed
Disable. Reset is a mechanism discriminator, not an assertion about the physical
trace. Original production bytes of v8 remain bound by its full1475-row manifest.

Choose the narrower fourth alternative after independent source review:
`disable_retired_by_later_ingress` requires typed observed Disable owner, current
retained reducer owner, exact source ticket owner, Revoked status and exact
disable_position. Only this already retired event drops matching local/Shared
source through existing discard_context_activation, preserving later stamps.
Every other denial still invokes the original strict global invalidation.
First callback timeout remains a refusal. No seal/transfer/token/output granted,
no deadline increase, retry, queue, detector, transport or new input owner.
New activation still uses existing native reply+marker SourceFree UnknownStart.
Independent causal/safety audit:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/TEXTAREA_FIRST_LIFECYCLE_AUDIT.md`.
Current changed production bytes are NOT_TESTED; focused/full/physical proofs
remain required. Installed exact79 unchanged.


## Retired Disable focused GREEN

GREEN28/28 PASS46.971s, including the formerly failing real-callback fixture,
all25 previous replay/cancellation contracts and two existing focus controls.
Original assertions and fixed callback/activation budgets unchanged. Exact
focused source1476 rows imported, only guarded rustfmt changes admitted; proof:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/disable-green-v1-import/RESULT.json`.
Independent runtime-delta review found blockers0: deleting the21-line adapter
helper and10-line observation branch restores exact v8 production hashes.
Shared cleanup still requires matching path and source owner generation, so a
successor published during the check cannot be cleared as the old source.
New full gate and physical textarea proof pending; installed79 unchanged.


## Full v9 declaration failure before broad execution

Full v9 FAILED36.186s before changed/full suites: discovery correctly found one
new canonical `bin:lay-ibus-engine` correctness identity, while private input
mistakenly declared target `lay-ibus-engine`. This is a declaration mismatch,
not a source semantic failure. Keep failed receipt in release80-v9-import.
Correct only the private added-test target to the canonical `bin:` identity for
a fresh full v10; discovery/equality/ledger gate remains literal. GREEN28 source
proof remains valid for unchanged production; no live installation occurred.


## Full v10 and physical acceptance closure — 2026-10-06

Full source release gate PASS1256.802s: canonical2994/2994, zero semantic or
infrastructure failures, changed correctness/package, strict lints, public-issue
and installer regressions, MSRV, fresh graph/canon. Exact1476-row source and
10 generated outputs imported. New immutable IME SHA256
`0902a3b87efd8b9398a4c6a49323731ed95c86775e422f3d1e3d338700eaecce`.
Gate: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v10-import/RELEASE_GATE_RESULT.json`.
This establishes source/build acceptance only.

Three finite diagnostic exchanges ran the same new IME with accepted79
CLI/daemon/extension/services; all ended PASS_EXACT79_RESTORED. No global IBus
restart, source migration, new input owner or stable80 installation occurred.
Same-byte daemon UInput attach/detach bookkeeping PID changes are retained in
trial receipts; L3/L1.1/global IBus/config/model/proof guards remain unchanged.
Summary with exact three trial receipts: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/V10_PHYSICAL_SUMMARY.json`.

| Actual client proof | PASS / executed | Scope and remaining limitation |
|---|---:|---|
| 8 Double Shift pairs with letter/delete/icon after each | 2/3 fields | input/editable8/8; textarea pair1 refused |
| Ordinary first/second-word Tab | 6/6 | exact proposal plus space |
| Held Shift and next letter | 3/3 | physical typed effect |
| Backspace variants, original fresh-event observer | 15/18 | three after-word partial-delete rows fail before Tab dispatch |
| Separate retained-hint actual Tab after partial delete | 3/3 | independent control, never upgrades original rows |
| Nonletter controls | 50/60 | failed rows include initial mode setup; not evidence that each symbol produced a hint |
| Space autocorrection | 2/6 | layout/missing-letter outcomes still incomplete |

Wide original93 rows complete in168.972s, original180s bound unchanged.
Retained-hint control3 rows11.779s. All predicates and historical failures remain
literal. Local input/textarea/contenteditable are not proof of Instagram,
WhatsApp or every third-party editor. No real-keyboard confirmation obtained.
Normal release promotion is refused while physical contracts fail.

### New first failed transition and explicit bounded replan

Current textarea trace has sourceSeal602 all preconditions true, fresh
owner21/activation21 installed, all native printable keys admitted/settled. The
old late-Disable timeout did not occur in this run; its fix is source proved,
but this physical run does not certify exercising that branch.
The new first loss is confirmed_strict_prefix_receipt at2781, then full-length snapshot
at2783 with revision_gap2, identity_current=true and no replay scope, rejected
at2784. Manual bridge later succeeds, but context_authority is false at2819.
Exact full-surface equality is not logged; gap2 alone proves sufficient rejection.
The strict-prefix branch intentionally retains inert lineage but does not advance
its revision witness; the following full receipt therefore cannot confirm it.
Trace: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/windows/REQUIRED79_COMPOSITION_PROBE_V2_RUN_20261006T012112_32d2d404ae1e/traces/20261006T012129_firefox.textarea_each_shift_pair_real_letter_delete_composition/after_flush_ibus_engine_debug.jsonl`.

Previous native-replay and retired-Disable repair rounds are closed at their
recorded limits; start a distinct bounded causal repair, not retry-until-green.
Compare: blanket revision-gap relaxation (reject), advancing every inert prefix
(reject: hides generic gaps/replay), or a single inert exact prefix-echo witness
from a previously confirmed managed append chain, consumed only by its next
exact bounded full snapshot with unchanged owner/token/epoch and no replay.
This third alternative is HYPOTHESIS, not implemented or granted authority.
First prove old failure in real production callback tests, then positive/negative
contracts, independent review, source gate and new physical bytes. Original
SafetyGate/verifier, exact replay fences and all old assertions remain mandatory.

Five independently audited wide mode failures stop before symbols are sent.
Their tails have activation_ready after publication, without activation_install
or post_activation_update. Owner of that separate investigation remains the
existing activation mechanism; no synthetic wake key, fallback or deadline
change is authorized by a failing setup. Exact cause/repair still pending.

### Space79 discriminator: observation with failed preservation gate

The separate finite Space79 diagnostic consumed a ready full NoApply(rank)
lease4us, epoch437/gen13; final word remained uncorrected. Prepared total141.952ms
(queue74.174/eval67.771ms). `rank` only means InputGate did not ApplyReplacement:
Keep, SuggestOnly, Veto, missing candidates or candidate-authority failure remain
indistinguishable. Target retention/numerical rank UNKNOWN; literal candidates0
is not coverage proof. Overall wrapper FAIL_BINDINGS, because actual input
updated word_usage_events.jsonl and word_usage_counts.json. All other protected
bytes/helper/sourceIDs/IME/L3/L1.1/global IBus checks passed. This is not a PASS
receipt or a product latency result; statistics were not rolled back over user
events. Readout: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/SPACE_READY_WAIT79_V2_RANK_READOUT.md`.


## Confirmed managed prefix echo causal RED and narrow implementation

RED on unchanged production v10: expected1FAIL29PASS48.161s. The positive
actual-key/CommitText/Reset/surrounding fixture fails with the exact marker
`confirmed native append delayed prefix lost next exact receipt authority`,
before bridge/RPC preparation. Seven negative scenarios and original28 replay,
lifecycle and physical-owner guards pass. Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-red-v1-import/RESULT.json`.
The controlled fixture supplies full exact equality absent from metadata.

Choose one inert revision witness in existing PendingContextResetRereceipt.
Only a previously confirmed ManagedCommit chain, no local/shared ExactReplay,
no quarantine or completed replay, same current token/owner/epoch, can retain
its exact strict-prefix echo. It is spent at the next observation. Only checked
armed+2/echo+1 with unchanged identity and original full bounded snapshot may
use ordinary confirmation/consume. First-unconfirmed, duplicate prefix, wrong
surface, selection, focus/content/navigation gap remain denied. All constructor
rearms/native appends clear the witness; no timer, polling, input owner, routing
or generic revision-gap relaxation added. Original fixture assertions literal.
ADR: decisions/2026-10-06-managed-strict-prefix-echo-rereceipt.json.
Independent design reviews exclude inherited replay aliases explicitly; new
source awaits focused/full/physical proof, installed79 unchanged.


## Managed prefix echo GREEN v1: causal assertion passes, new dependent oracle fails

Focused30 executed48.110s:29PASS1FAIL; receipt `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-green-v1-import/RESULT.json`.
The new positive reaches and passes exact_manual_handoff confirmation, then
fails its added cycle09_tail_is_authoritative check before manual consumption.
All seven negative scenarios and original28 controls pass. Keep complete FAIL;
confirmation is not full manual-route acceptance. Independent runtime-delta
review found hard blockers0, unchanged exact receipt/consume functions and
literal original fixture prefix. Review the new dependent VisibleTailSource
oracle against the actual existing contract before another prospective proof;
no old assertion or historical result may be rewritten. Installed79 unchanged.


## New dependent oracle correction, unchanged production witness

Independent source audit proves the new fixture retained the literal boundary
space from known_engine plus four actual CommitText effects. VisibleTail returns
the whole committed_tail, so expecting only the four-letter word was an incorrect
NEW dependent oracle. Readout itself can admit confirmed Reset rereceipt and
does not consume it; no contrary ownership claim is made. Change only this new
post-causal oracle to exact expected space+typed word, assert buffer preservation,
actual ManualToggle3/false, then full authoritative VisibleTail+buffer equality
using the already accepted manual-first sequence. All older assertions unchanged.
The causal marker and every step before it are byte-identical, so its old RED
still proves detection; GREENv1 remains complete FAIL48.110s. Runtime production
witness source unchanged; next prospective GREENv2 required.


## Managed prefix echo GREEN v2 closes focused source round

ALL_PASS30/30, zero failed,48.139s. Positive runs ASCII and Unicode native-key
callbacks with exact CommitText/full-snapshot proofs, preserved boundary+word,
actual ManualToggle3/false and strict whole-buffer VisibleTail. Consumption
keeps UnknownStart and bounds its observed suffix; no KnownStart grant claimed.
Seven negative scenarios and all original28 controls pass. Independent V1
production delta review hard blockers0; unchanged production reused for v2.
Exact1477-row focused source imported, only guarded formatted2 Rust outputs.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-green-v2-import/RESULT.json`; import: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/prefix-green-v2-import/IMPORT_RESULT.json`.
New bytes still require full source release gate and changed-byte physical
acceptance; exact79 remains installed. V1 FAIL and old physical outcomes retained.

Next connected repair investigates no-key activation publication/consumption.
Existing adapter acquisition task owns native Get/marker; existing engine install
and native InputMode publisher remain sole application owner. Compare existing
later-callback-only consumption (demonstrably insufficient), same-task exact
completion delivery (proposed), and polling/synthetic wake/timer/fallback
(rejected). Require causal old-source callback proof before selecting delivery.
No source/engine-ID change or second layout decision is proposed.


## Acquisition completion causal RED v1: rejected control set

Measured remote source-only RED v1 executes35 tests:33PASS/2FAIL,50.576s.
Native actual-wire positive fails at the predeclared
C09_ACQUISITION_COMPLETION_WITHOUT_KEY_MISSING_INPUT_MODE marker, proving
completed acquisition alone does not deliver its installed native mode.
An unchanged older control, firefox_manual_toggle_defers_until_the_client_replies_after_rpc,
also fails at tests.rs:558: serve_ping_and_marker receives a message without a
member header. This second failure rejects the frozen causal_failure_exact
protocol. Verdict FAIL_ADMISSION_OR_EXECUTION, not accepted causal RED and not
permission to promote the delivery patch. First wire message type/phase of the
older failure remains UNKNOWN pending bounded audit; no old assertion, deadline
or membership filter has changed. Production delivery patch remains unapplied.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-red-v1-failed-import/RESULT.json`.
Installed exact79 unchanged; runtime authority false.

Independent V17 editable boundary audit identifies the actual missing delimiter
before '(' and before subsequent Reset. Clear-empty/Hide and CommitText(' ')
await precede the local tail becoming5, but client receipt remains4/4; final
fresh DOM is5/5. The signal send does not prove insertion acknowledgment.
Exact wire delivery/client application remains UNKNOWN. This is a separate
Space boundary failure, not a proved nonletter hint failure.
Audit: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/V17_012249_FIREFOX_EDITABLE_NO_HINT_BOUNDARY_AUDIT.md`.
No field-specific exception, probe, timer or authority change applied.


## TEST29 header diagnostic and delivery review round one

Source-only exact old control, header metadata only before the unchanged Ping
assertion: ALL_PASS1/1,32.134s. All five Ping stages receive MethodCall/Ping.
The earlier headerless offender was not reproduced and its type remains UNKNOWN.
This prospective PASS does not rewrite the earlier35-test FAIL. No assertion,
reply filter, timeout or production source changed; no runtime import occurred.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/test29-header-v1-import/RESULT.json`.
Next controlled witness will exercise an actually unregistered engine and its
exact detached Reset reply serial before any bridge; not a blind retry.

Independent delivery review round one REQUEST_CHANGES_ONE_STATIC_RACE.
Separate ready and pending reads can miss matching publication and return Denied
after publication clears pending. Completion must inspect matching request,
fence and path consistently using existing pending→ready→reducer lock order
and existing changed listener. Generic Bridge wait remains unchanged. The
initial pending.ready suspicion was withdrawn: Acquisition marker processing
returns before the Bridge-only ready flag. No implementation based on that
withdrawn suspicion was made.
Review: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/ACTIVATION_COMPLETION_DELIVERY_INDEPENDENT_REVIEW.md`.
Observed Acquisition/ready grants deliberately survive fence expiry in the old
callback contract. Activation deadline is not a new token wall-clock expiry;
autonomous delivery latency beyond blocked engine interface remains unmeasured.
New delayed-predecessor listener fixture is NOT_RUN and does not claim a
deterministically reproduced mutex-race seam. Exact79 remains installed.


## Controlled detached Reset witness closes harness diagnosis

ALL_PASS1/1,32.194s. For actual registered→removed engine objects, direct
observed Reset serial19315 and19415 each generates exactly TypeError,
UnknownObject, the matching reply serial and no member, before any bridge.
No filter/drain occurs between Reset and the raw witness. This establishes
a concrete scheduling-dependent harness gap; the historical header itself
remains UNKNOWN. Correct only the two absent-engine helper flags through
the existing exact-serial consumer; all prior assertions/deadlines stay literal.
Decision: 2026-10-06-detached-reset-controlled-peer-reply.json.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/test29-detached-v1-import/RESULT.json`.
No blanket filter, sleep, authority or runtime change. A fresh strict causal
RED over the corrected source-control schedule is required; prior REDv1 FAIL
and header-diagnostic PASS retain their original separate scopes.


## Acquisition completion strict RED v2 accepted; connected source change

OLD_FAILURE_REPRODUCED36 executed/35PASS/1expectedFAIL,50.600s. Actual
no-key native completion reaches the exact missing InputMode marker; all30
prefix/manual controls, four native publisher controls and exact detached
Reset witness pass. RED production source unchanged. Only two absent-engine
helper flags use exact-serial consumption; all original assertions/deadlines
and remaining fixture prefix bytes are preserved. Exact guarded formatted
residuals imported, then appended GREEN-only delayed-predecessor listener.
A local length-based import assertion initially counted the two false→true
character reductions incorrectly and stopped before fixture mutation; corrected
exact transformed-prefix comparison passed. No source proof result changed.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-red-v2-import/RESULT.json`.
V1 rejected control set and historical UNKNOWN header remain recorded.

Applied independently reviewed V2 same-task acquisition delivery. Consistent
pending→ready→reducer checks remove the reviewed split-read race; exact full
ready token is checked before successor request, existing listeners/deadline
and cancellation retained, generic Bridge wait unchanged. The served engine
revalidates full current outcome after awaits, respects atomic exclusivity,
uses original install/ACK and guarded native InputMode publisher. No new
owner, queue, detector, RPC, source migration, model or text-edit capability.
Review: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/V2_REVIEW.md`.

Applied separate opt-in existing-gate Clear/Hide/Commit metadata diagnostic
plus legacy callback serial binding and preserved output/error-order unit.
No raw text, new callback state, locks, timers or output semantics. Join only
unique PID/path/existing callback-clock; orphan/ambiguous is UNKNOWN. Signal
success does not establish client insertion; logging can perturb timing.
Review blockers0 for patch93af4ae1; this remains diagnostic-only source scope.
The combined source now requires GREEN, full gates and physical acceptance.
Runtime authority unchanged: accepted exact79 remains installed.


## Combined GREEN v1 compile failure, bounded correction

FAIL_ADMISSION_OR_EXECUTION29.143s: discovery0 returns101 before execution.
Compilation E0252 found EnginePath imported twice in observation.rs: new
header import and existing owning-section import. Remove only the duplicate
new import; no predicate, assertion, timeout, identity or test selection changes.
No test executed, so no semantic PASS/FAIL denominator claimed. Static design
reviews did not claim compilation. Original failed receipt retained:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v1-failed-import/RESULT.json`.
Installed exact79 unchanged.

Opt-in prepared InputGate diagnostic independently reviewed blockers0: borrow
the already computed result once, before original conversion; no new ranker,
pool generation or admission query. Actual action/reason/returned pool size,
selected receipt and existing frame binding remain metadata only. Scoreboard
apply is selected-index count, suggest includes unselected Eligible; no claim
of complete authority-admitted pool. final_decision_present is prepared
conversion, not physical Apply. Existing source/generation/prepared-vs-superseded
trace binds observations; no late edit permission. Debug/log overhead is outside
original route.total_us and still needs runtime latency measurement.
Review: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/space-gate-metadata/SPACE_GATE_METADATA_INDEPENDENT_REVIEW.md`.


## Combined GREEN v2: positives pass; explicit harness replan

FAIL_ADMISSION_OR_EXECUTION40 executed/28PASS/12FAIL,51.054s. Compilation
succeeded. Native and compatibility actual served no-key acquisition/property
positives both PASS; output-order/error-propagation invariant PASS. This proves
the causal repair only in those source fixtures, not full source or client
acceptance. Twelve older/new listener controls encounter TypeError where Signal
or Ping is expected. Terminal excerpts expose reply serials21020,22600,41100.
The exact error names of these unseen headers remain UNKNOWN.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v2-failed-import/RESULT.json`.

Explicit replan before further changes: new delivery must obtain the actual
served interface. Existing harness often sends synthetic callbacks to an
unregistered engine and then directly invokes a local callback. Investigate
whether interface lookup starts zbus dispatch earlier, making their unresolved
reply transport observable. Controlled detached-Reset witness already proves
exact UnknownObject replies for19315/19415, but does not label all twelve.
Compare an existing-server-only public lookup if the dependency provides one,
versus explicit controlled transport semantics for manual callback fixtures.
Reject blanket error filtering, changing assertions/deadlines, per-case runtime
bypasses, skipping controls or retry-until-green. Keep all normal real-wire
callback paths unchanged. No further runtime producer or ownership change
until this common first transition is established. Independent agents audit
only dependency API and shared fixture seams; ROOT owns any eventual edit.
New guard fixture prepared separately NOT_RUN; no PASS claim for its negatives.
Production remains uninstalled; accepted exact79 unchanged.


## Controlled FIFO header diagnostic confirms shared transport class

Source-only diagnostic executes three existing controls with unchanged
assertions/filters/deadlines:0PASS/3FAIL,32.740s. Unexpected headers are exactly
Error/UnknownObject/memberNone with replyserial30100 in forward_marker,
21020 and41100 in legacy_effects. This confirms those three first failures
are synthetic detached callback transport, not rejected engine output. Remaining
nine failure headers are not individually labelled by this sample.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/fifo-header-v1-failed-import/RESULT.json`.

Dependency source zbus5.15.0 public object_server() starts lazy dispatch.
No public existing-server-only lookup is available; private ensure/root APIs
cannot be a runtime workaround. method_message already sets NoReplyExpected;
zbus rejects missing objects before per-method reply suppression. Therefore
adding that existing flag again cannot repair this transport class. Registered
normal ProcessKeyEvent replies (including48bool checks), bootstrap/Get/Ping
and real-wire defaults must retain strict behavior.

Selected bounded direction: common test-only manual/detached callback sender
records exact expected optional reply serials with the existing UnknownObject
filter, preserving order and all effect/state/denial assertions. Duplicated
callback serials must not collapse accounting or introduce sleeps between
batched adverse events. No blanket ignore, owner registration cache, client
exception, production timing or normal-reply route change is allowed.
Independent common-helper proposal/review still pending; no new test seam
applied yet. Diagnostics2files remain private, unimported. Exact79 installed.


## Exact manual callback transport: reviewed implementation, execution pending

Applied reviewed patch735a2643d6593da7decacf39e343f80b78f06b0053b0480db5c98ef752e613f2 to four controlled-test files under ADR2026-10-06-manual-callback-exact-reply-accounting. Final independent review PASS_WITH_LIMITS, zero blockers; execution NOT_RUN. One counted declaration per explicit manual send preserves duplicated serials; existing strict UnknownObject filter, normal48 boolean replies, served no-key fixtures and original assertions/deadlines remain literal. Added exact repeated-serial witness and separately reviewed post-interface-wait delivery refusal fixture (atomic active, foreign adapter, consumed grant), both NOT_RUN. No production change in this step; current activation completion implementation and telemetry remain uninstalled. Accepted exact79 is unchanged. Next scope: original40 tests plus these two fixtures and actual registered48 replies, on guarded remote source only.


## Combined GREEN v3: 42/43, residual first transition differs

FAIL_ADMISSION_OR_EXECUTION43 executed/42PASS/1FAIL,53.947s. Native/compatibility no-key positives, new repeated-serial accounting, post-interface-wait atomic/foreign/consumed-grant refusals and registered48 boolean replies PASS. All prior twelve transport-failing cases except one now pass; remaining read-only exclusion control fails at observer Denied, before original bridge-revocation assertion. Exact member not yet known. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v3-failed-import/RESULT.json. No failed-source import, installation or promotion. Explicit replan: instrument only the existing fixture before its unchanged observer assertion to identify member/key/release and first denial. Do not weaken assertions, change deadlines or repeat until green. Private diagnostic remains source-only; accepted exact79 unchanged.


## Remaining control diagnostic and bounded wire-type correction

Diagnostic0/1FAIL31.740s: first packet ProcessKeyEvent/keyval97/keycode30/state0 reaches observer Denied. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-denied-diagnostic-failed-import/RESULT.json. Static source analysis identifies manual inline builder lost the original typed_key_message u32 argument constraint: keycode literals default to i32, while strict ingress requires (u32,u32,u32). This type-inference explanation is distinct from the measured denial; historical wire signature was not logged. Restore exact original unsigned tuple constraint in this test branch only. All prior assertion/deadline/callback values remain literal; production unchanged. Following explicit replan, run all43 again to verify the corrected seam, not retry unchanged bytes. Exact79 remains installed.


## Combined GREEN v4: causal repair and controls all pass

ALL_PASS43/43,zero failed/ignored,53.950s. Original40 identities, exact repeated-serial witness, three post-interface-wait delivery refusal subcases and actual registered48 boolean replies pass. Both native and compatibility served no-key InputMode positives pass; prefix rereceipt and original rejection controls pass. Corrected unsigned tuple restores old wire contract without assertion/deadline changes. Exact formatted declared Rust source imported only after full input/parent/after closure; all source rows match. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/activation-green-v4-import/RESULT.json. Scope: focused source only; new full release/physical gates NOT_RUN. Current production implementation remains uninstalled; accepted79 bytes unchanged. Next: one guarded full source gate with all new identities and fresh graph/canon, then finite native Firefox input trials with exact79 rollback.


## Full source v11: canon documentation rejection, no promotion

FAILED83.682s: compile/discovery and fresh architecture graph PASS; CANON refuses two new decision records missing reason and uncovered composition_commit metadata. No full suite or immutable release artifact was produced. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v11-failed-import/RELEASE_GATE_RESULT.json. Explicit bounded documentation correction: complete original detached-Reset and manual-transport record schema and add separate Space-boundary metadata observation decision. Runtime Rust, old guards/assertions/deadlines and focused43/43 proof stay literal. New full gate must run from that proven exact parent plus only these docs; no failed generated output imported. Installed exact79 unchanged.


## Full source v12: additional manual transport coverage failures

FAILED472.934s at CHECK_CHANGED388.942s. Canon/schema and fresh graph PASS; all original canonical rows preserved plus exactly9 new identities. Engine target678 selected reports11 failures; focused43/43 is not full acceptance. First excerpts: expected Signal receives Error in generic forward/drain helpers and native_transfer boundary; some residuals instead unwrap absent header member. Those error names are UNKNOWN until exact header capture. Raw11 process logs retained in /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/v12-engine-partial-failures; full receipt /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v12-failed-import/RELEASE_GATE_RESULT.json; canonical summary /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v12-failed-import/CHECK_CHANGED_SUMMARY.json. No final generated import or immutable artifact; no live changes.

Explicit replan: earlier22 manual callsite mapping was incomplete. Classify every actual send/build/handler ownership in five controlled fixture files, preserving all real served RPC/48bool routes. Confirm one representative remaining header class, then extend the existing explicit per-emission declaration seam systematically. Do not automatically declare all NoReply packets, blanket-filter errors, change old assertions/deadlines, or add production ownership/cache/bypass. Production patch freezes while test transport closure is established. This replaces selective failure-by-failure repair; ROOT owns the next sealed source-only run and requires full fixed proof. Exact79 remains installed.


## Full v12 aggregate and exact remaining transport header proof

CHECK_CHANGED target log totals:3003 selected,11 failures,2992 passes across32 target rows. Hermetic summary BLOCKED_CONTRACT because the original empty known-failure ledger refuses those11 unexpected failures; infrastructure_failures0 and no ledger edits. Other later release gates remain NOT_RUN.

Private source-only header diagnostic repeats exactly11 original failing identities:0/11PASS,11FAIL36.710s. Original assertions and exact receive filter unchanged. Every first unexpected header is measured TypeError/UnknownObject/memberNone with reply serial respectively3219,3224,7504,7604,3400,3600,3237,3200,3180,3215,3215. This proves the common unaccounted manual transport transition for these exact fixtures; subsequent unseen behavior remains UNKNOWN. Receipt /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/manual-transport-closure-diagnostic-failed-import/RESULT.json; per-identity headers /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/manual-transport-closure-diagnostic-failed-import/HEADER_PACKET.json. Diagnostic tests.rs remains private and unimported. Complete manual-versus-served callsite inventory is being built before any extension of the existing declaration helper. Accepted exact79 remains installed; no new production change or runtime authority.


## Complete explicit manual transport closure imported for execution

Final patchc4101c04457a02b3c506454711dcedf5d339eb7eeb697226042860d63bf6b67b applied with exact five-file before/after bindings, exactly3 changed test files. Complete158-site inventory76676f5563d5c87d92b83798aa43887e23d58ec89bea5b7e154a02a6f6dcad04 maps105/105 previously undeclared manual sites, preserving normal5 registered RPC sites, raw2 adverse ingress, remaining Signal caller of original mixed helper, old typed key builder/u32 ABI, all old assertions/deadlines/observer order, exact filter/counter and already tracked legacy_key. Independent actual-byte review PASS_WITH_LIMITS, zero blockers. Execution NOT_RUN at import. ADR complete-manual-test-transport-closure governs this test-only step. New runtime production unchanged and uninstalled. Next fixed scope54 identities = prior43 + every11 full-proof failure; full3003 gate remains required. Exact79 remains live.


## Complete manual closure GREEN: all fixed54 pass

ALL_PASS54/54,zero failures/ignored,59.313s. Includes the prior43 authority/mode/Space/error-order/normal48 controls and every11 previously failing full-proof identity. Exact source closure and formatted three-test-file import completed, all other source rows literal. Receipt: /home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/manual-closure-green-v1-import/RESULT.json. This establishes the common harness transport repair in its fixed scoped set; full3003 and later release gates must still pass. Current production changes remain uninstalled, physical Firefox for these bytes NOT_RUN. Exact79 remains live.

## Full source v13: semantic gates pass, new test lint refuses release

FAILED920.128s at FULL_GATE437.897s. CHECK_CHANGED and the repeated full
canonical correctness/package set both PASS3003/3003, zero known semantic or
infrastructure failures, including678 engine tests. Fresh graph/canon PASS.
Default dead-code inventory532 matches the unchanged baseline. Strict Clippy
then refuses two clone_on_copy calls on Option<WordScope> in the new
post-interface-wait refusal fixture; no production lint failure is reported.
The original lint wrapper discarded its temporary JSON; a separately guarded,
read-only scoped Clippy diagnostic establishes both exact locations12144/12186.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v13-failed-import/RELEASE_GATE_RESULT.json`.
Exact diagnostic: same directory `CLIPPY_DIAGNOSTIC.err`.

Bounded correction removes only those two redundant clones in the new test.
Copied values, assertions, deadlines and runtime Rust stay unchanged; no lint
allow, baseline update, known-failure entry or skipped gate. Recheck that exact
fixture and strict Clippy before another full release gate. Later MSRV/build/
installer/physical gates are NOT_RUN; no failed generated import or release
artifact was admitted. Exact79 installed bytes remain unchanged.

## Test-copy correction: focused and strict lint PASS

The changed post-interface-wait refusal fixture passes1/1,31.778s; exact
formatted source closure1482 rows matches ROOT without a formatter import.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/lint-clone-green-v1-import/RESULT.json`.
Separate guarded strict lint then PASS for default and research-tools profiles:
non_dead_diagnostics0 in each; unchanged dead-code inventories532/356.
Receipt in the same directory: `STRICT_LINT.out` and `STRICT_LINT.err`.
Only two Copy-value test expressions changed. This combines the prior54/54
focused proof and v13 two3003/3003 semantic passes with the newly measured
lint correction; it does not replace a full gate for the exact final source.
Full source v14 is the next required gate; physical acceptance and ordinary80
installation remain NOT_RUN. Accepted exact79 and runtime authority unchanged.

## Full source v14: second semantic pass exposes asynchronous state loss

FAILED884.987s at FULL_GATE401.133s. CHECK_CHANGED PASS3003/3003;
the repeated canonical full set reports3002/3003, one engine failure in
residual_reset_rereceipt_failures_never_delete_gui_text. The unchanged
DeleteSurroundingText/CommitText denial assertion passes; the subsequent
unchanged local-tail assertion fails at3314: after="", before="abcde".
The seven-case gap identity and first state mutation are not yet measured.
This is different evidence from earlier UnknownObject transport failures.
Receipt: `/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/release80-v14-failed-import/RELEASE_GATE_RESULT.json`.
Raw exact case: same directory `REFUSED_MANUAL_TAIL_FAILURE.log`;
hermetic summary `FULL_GATE_SUMMARY.json` retains BLOCKED_CONTRACT and the
original empty known-failure ledger. No failed source/generated/artifact import.

Explicit replan before another full gate: inspect the new no-key acquisition
completion alongside the old denied-manual and delayed-interface registration
sequence. Separate a real stale/evolved-context installation from a controlled
fixture registration transition; neither is established yet. Capture actual
gap, token/owner/scope/tail at the first transition, then use controlled event
ordering rather than sleeps or retry-until-green. Do not weaken denial or
tail assertions, existing deadlines, activation guards or verifier authority.
Two independent source reviews are pending; production is frozen. Later lint/
MSRV/release/physical gates NOT_RUN on v14. Exact79 remains installed.

## V14 tail-loss diagnostic: nonfailure capture, cause still UNKNOWN

One private logging-only source run of the unchanged seven-gap test passes
1/1,32.001s. Metadata before the bridge and after its old helper captures
unchanged abcde for mismatch/selection/second_receipt/focus_out/caps9;
navigation/sensitive already have empty tails before the bridge. Original
assertions, callback order and budgets stay literal. Receipt:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005/refused-tail-gap-diagnostic-v1-import/RESULT.json`;
raw metadata in the same directory `TEST_0.err`. Diagnostic source is private,
never imported. Logging may perturb scheduling; this passing sample does not
identify the original failed gap or establish a repair.

Two independent static reviews identify different concrete possibilities:
activation installation/discard can clear the local tail, but the original
SourceFree grant normally ACK-consumes at the first key; alternatively, queued
manually handled messages may reach automatic zbus dispatch after late engine
registration. Direct bridge authority refusal does not itself clear the tail.
Reports: `V14_RERECEIPT_TAIL_FLAKE_STATIC_AUDIT.md` and
`V14_ACQUISITION_TAIL_STATIC_REVIEW.md` in the same Firefox private directory.

### Finite collection replan and withdrawn barrier

The proposed retrospective absent-dispatch barrier is **DO_NOT_APPLY / NOT_RUN**.
Its remaining optional reply counts can include packets received before zbus
subscribed its automatic dispatcher; zbus does not replay those packets. Waiting
for every count therefore does not prove dispatcher closure and can time out.
The private correction is
`V14_ABSENT_DISPATCH_BARRIER_PROTOCOL_CORRECTION.md`, beside the reports above.
Any future controlled transport closure must first obtain an actual dispatcher
readiness acknowledgement, then establish completion of each exact emission.

The next source-only diagnostic is a fixed 32 calls of the literal original
seven-gap test, without stopping after success. A bounded cfg(test) memory
recorder observes installation, discard and focus cleanup using metadata only;
it prints event payload only after an observed tail mismatch. Caught original
assertion failures, gap exposures and lost diagnostic events remain separate
from successful completion of collection. Zero observed failures cannot close
the original V14 failure. This private overlay is not imported into ROOT or
the installed runtime. Runtime authority remains unchanged.

The fixed collection completed all 32 original calls and 224 gap entries with
zero assertion failures, zero observed tail changes and zero lost diagnostic
events. Test execution took 3.767 seconds; guarded preparation/compilation plus
collection took 35.453 seconds. The receipt is
`refused-tail-fixed32-diagnostic-v1-import/RESULT.json` in the same private
Firefox directory. The recorder perturbs scheduling; this nonfailure result
does not identify the V14 mutator and does not certify the runtime. The original
V14 failure remains open. No diagnostic source was imported and installed79
hashes still match the accepted bytes.
Next proof uses an actual held engine interface and queued callback, with a
trailing reply-bearing standard Peer.Ping as a completion fence. The real
engine interface has `spawn = false`; its dispatcher therefore finishes the
earlier engine call before dispatching the trailing Ping. Readiness of the
dispatcher is established by a separate actual Ping before the fixture emits
callbacks. This proves or rejects duplicate dispatch under a controlled
violation; it must not be described as reproducing the original V14 mutator.

The independently inspected two-seam transport isolation design is readiness
in the existing harness bootstrap and strict absent-path completion before
the refusal helper publishes its engine. It retains observer-first ingress,
original assertions, callback budgets and normal registered RPCs. Unexpected
messages remain failures. No per-key wait, production change or acceptance
waiver is planned. Runtime production is frozen; accepted exact79 and input
authority remain unchanged.

The first controlled-dispatch witness attempt stopped at compilation: Rust
E0716 in the private four-field preedit decoder borrowed `Value<'_>` from a
temporary message body. No test executed. The two proposed transport helper
files are unaffected. Receipt:
`dispatch-isolation-witness-v1-failed-import/RESULT.json`; elapsed 28.754 seconds.
The second finite repair binds that existing message body before decoding;
wire types, old assertions and all runtime source remain unchanged. Any further
unresolved issue requires another explicit replan rather than an unreported
third repair.

The repaired controlled witness and original seven-gap test passed 2/2 in
32.474 seconds including compilation. Receipt:
`dispatch-isolation-witness-v2-import/RESULT.json` (SHA256
`359907463dc2293b2977ce74a8510535f3c25473ebe1a3c6b4144673577f7529`).
One real NoReply Reset packet executed both manually and through the served
engine; the inert existing alt-completion flag was cleared by the second body.
Measured tail remained `abcde`, epoch remained 6 and token/reducer revocation
remained 4. No text-edit signals occurred. This establishes duplicate fixture
delivery, not a reproduction of V14 tail loss; the old mutator remains UNKNOWN.

Only the independently reviewed two-file helper isolation was imported into
ROOT. No private witness, recorder or diagnostic test was imported; all 3003
canonical identities remain. The next fixed proof covers the original failure,
every direct caller of the changed refusal helper and the earlier transport
closure controls. Production/runtime authority remains unchanged.

The helpers-only fixed affected proof passed 58/58 in 61.892 seconds; receipt
`dispatch-isolation-helper-green-v1-import/RESULT.json` SHA256
`572135413f62111cb620c0ccccdafe9ae9ffbc5738e7e17608dcc80f1fc70c92`.
Exact formatter-produced helper bytes were imported. Its source snapshot
contains 1483 rows and zero private diagnostic tests. Full V15 starts from that
proven source and overlays only the current owning document/decision; all
3003 canonical identities and original zero-failure ledger remain mandatory.
Original V14 causal attribution remains UNKNOWN; changed production bytes still
require physical-client proof. Installed79 remains unchanged.


### Full V15 and finite same-byte Firefox trials — 2026-10-06

Full V15 passed all 15 release stages in 1260.297 seconds, including both
3003/3003 canonical suites, strict Clippy, public issue/installer regressions,
Rust 1.88 checks, and the generated-graph/canon checks. Exact imported source
contains 1483 rows and ten generated graph files. IME candidate SHA256 is
`0d0c0617d61fc22f94d1c82014a38a513fa36b4d4c01bbe20fca92e3a7271bc5`.
Full receipt: private Firefox directory `release80-v15-import/RELEASE_GATE_RESULT.json`
SHA256 `df684e39b37144df98fd5c642ea9d5456b860226e5a7aced67a34717c6b2dbd6`;
source manifest `SOURCE_AFTER.json` SHA256
`d8a3085057a73c87f534a9d142daba975d087a8a62b0e692222f18b45ef585a2`.
This is source acceptance, not an all-Firefox runtime claim. Original V14
causal attribution remains UNKNOWN.

All source rows remained frozen during three predeclared finite temporary
IME80 trials. Every trial restored exact accepted79 bytes and closed its
source/protected-runtime checks successfully. Global IBus, L3 and L1.1 were
preserved; ordinary full80 installation did not occur. The synthetic device
is not actual physical-keyboard acceptance.

| Same-byte scope | Measured result | Remaining limitation |
|---|---|---|
| Eight sequential Double Shift pairs, next letter, Backspace and rendered icon in input/textarea/editable | 3/3 fields PASS, 38.423 s | owned local fields, daemon79 and synthetic device |
| Fixed wide matrix | 86 PASS / 92 executed, 6 FAIL, 1 BLOCKED, 140.460 s | all three autocorrect-rnjhjt and partial-delete-after-word rows failed; editable second-word Tab was not sent |
| Actual Tab with retained visible hint after partial deletion of second word | 3/3 PASS, 11.954 s | cannot reinterpret wide preflight failure as PASS |
| Digits/special-symbol hint suppression within wide matrix | 60/60 PASS | explicit fixed examples only |

Summary with exact trial/matrix hashes and paths:
`V15_PHYSICAL_SUMMARY.json` in the private Firefox directory. Trial receipts:
`repair80-trial-99131b0ba28f454d9b3cd71a066d6ae1/TRIAL_RESULT.json`,
`repair80-trial-886328e7fa3a49d38ff9b4d64f6aba9f/TRIAL_RESULT.json`, and
`repair80-trial-d400a5c5070744a2b763bfea25b7ba76/TRIAL_RESULT.json`.
Wide physical matrix SHA256
`beebd07a8d8b5e7dee5a634a5b67229d2031d82cceaf330de3ce5f30704a70dc`.
Three wide partial-delete failures stop before Tab: browser composition-event
freshness admission fails after actual native Backspace update. Exact retained
Tab success establishes a different measured boundary, not a waiver of the
wide predicate. The three autocorrect cases leave `кторое `; the first loss
in candidate/decision/verifier authority is under receipt analysis.

A fourth predeclared one-attempt-per-field Space-DOM diagnostic sampled the
own field immediately after Space and before the next symbol. It passed
textarea and editable, failed input: no sample in the bounded interval showed
`тест `; later samples showed `тест(`. This classifies SPACE_NEVER_OBSERVED_IN_BOUNDED_SAMPLES,
not proof that a space never transiently existed or identification of a Gecko
branch. Elapsed 13.361 s; 2 PASS / 3 executed, zero BLOCKED. Exact79 rollback
and all1483-row closure passed. Receipt:
`repair80-trial-space-dom-ca01836cc0784d42aaace321e9616c9a/TRIAL_RESULT.json`;
inside its `SPACE_DOM_RUN_3c7414a6cc1347af9646beb8c859234e`, matrix SHA256
`7748a50b60ce9284126c43dd1709a05be25916fd9fe6b341ed8c282d4adefc58`
and failed input panel SHA256
`277043928d01acfdc508efff63af8e5fa362485de5e5ee19e5854be65982981b`.
The next diagnosis joins exact Space callback identities to output-await
metadata and own DOM receipts. Successful emission cannot establish client
insertion. No additional text executor, field-name bypass, lowered safety gate
or changed production path is authorized by these failures.

Instagram/WhatsApp, actual physical keyboard, combined daemon80/IME80 and
ordinary80 installation remain NOT_TESTED. Installed runtime remains exact79.

### Native Space transport experiment: source admission — 2026-10-06

The exact failed input callback emitted Clear, Hide and Commit(separator)
successfully, yet the bounded own-field DOM samples did not contain the space.
The causal alternative is confined to a private four-file prototype: ordinary
managed Legacy NoApply/manual-suppressed Space clears/hides the suffix and
passes the original key through; Atomic, legacy active-word composition and
authorized correction paths retain their existing behavior. The existing
native-input observer closes the boundary. No new owner, editor, key detector,
candidate policy or application-name condition is introduced.

Focused source controls passed 12/12 in 38.582 s, including four protected
Double Shift ownership tests. The first run passed 11/12; the new Atomic test
failed during suppression setup, before its transport assertions. A test-only
repair publishes its token through existing push_tail_char; production bytes
did not change during that repair. Exact receipt:
`native-space-green-v2-import/RESULT.json`, SHA256
`42bdf940fec0ba94761abc456f1c366202ac83b9cd67e1b7f51e6f9937f0dbbc`.

The two unchanged old Legacy Space contract tests were also executed and both
failed at their original handled-true assertions. This is an actual 0/2 result,
not a release waiver: `native-space-old-contract-v1-import/RESULT.json`, SHA256
`4e7261b707e29c4fa30b92352dcf403b469e97c2da67396e58156cdf444e069e`.
Preserving next-word authority after Reset remains an unproved consequence.

Target-only guarded compilation passed in 181.857 s. Experimental IME SHA256:
`76eca112c20563c375705b4a86b93f530c2f5473ed66b296e23390176613e50a`.
Receipt: `native-space-artifact-v1-import/RESULT.json`. Source import contains
1483 exact rows; focused and build manifests have different serialization
hashes but identical parsed rows and file hashes/modes. Both original receipts
are retained and checked independently. The prototype remains private; ROOT
production source is unchanged. This is EXPERIMENTAL_NOT_RELEASE, not full
source acceptance, physical-client success or installation authority. The
one-attempt-per-field Space-DOM comparison is prepared; next-word Tab/manual,
actual keyboard and authenticated site fields remain NOT_TESTED. Installed79
is unchanged at this source-admission milestone.

The finite native-Space DOM comparison subsequently passed 3/3 fields in
11.229 s. Each field's first fresh sample already contained the exact word and
one space; the first sample after the next symbol preserved that space.
Receipt: `native-space-experiment-trial-2ace3c72ce634cd485ae3bd93b06c1da/TRIAL_RESULT.json`,
SHA256 `5971fdca903cb46d998d5bc1cc4c97fef252239954f1563eba9bf31f9b1a4cee`.
Matrix SHA256 `ed03e4464d6b0654b805f64bf7786951a5d21b8a1fc290bec80e07a87a57d397`.
All source/input pins remained unchanged and exact79 restoration passed.
Global IBus and L3/L1.1 remained intact; candidate authority was temporary and
limited to this causal experiment. This supports native Space as a transport
alternative for the measured loss, without identifying the actual Gecko branch
or proving all-field reliability. No full80 installation occurred. The next
finite check examines next-word completion and actual Tab; the two preserved
old contract failures and Reset/manual availability remain unresolved.

The same private IME then passed the separate finite next-word test 3/3 in
11.278 s: native Space is independently observed, a fresh current hint is
proved for the next prefix, and one actual Tab produces `тест почему `,
caret12, ended composition and preserved focus in input/textarea/editable.
The strict V17 freshness predicate was retained; no V18 retained-surface
waiver, authority override or Double Shift stimulus was used. Receipt:
`native-space-next-word-tab-trial-fda80a156f5041b9a76b1af7907436e1/TRIAL_RESULT.json`,
SHA256 `1e1c74dce6d7d40318396dc320bce0f407e44feebd07ca246b50a3cfacaeaab9`;
matrix SHA256 `f73b5f721496b130407328dc71ddcf51cd516b7d4a3ef9dc58bc00be07aceea9`.
All source/input pins and exact79 rollback passed. This proves the following
word's Tab in these six combined finite field cases, not last-word manual
authority, Backspace variants, real keyboard or authenticated site editors.

Read-only analysis locates the remaining predecessor loss at false Space
PRESS in `advance_context_reset_rereceipt_after_key`: the local native append
is not owned append, so the existing pending predecessor is cleared. The
next boundary still settles KnownStart, consistent with the measured next-word
Tab success. A prospective production migration must preserve an eligible
predecessor as unconfirmed and require the original strict real client receipt
before reuse; it must retain all old downstream/manual/negative checks.
Transport-specific false/noCommit assertions need an explicit decision and
must detect duplicate IME insertion, not waive the old semantic guarantees.
Read-only receipts: `native-space-diagnostic/NEXT_WORD_AUTHORITY_ANALYSIS.md`
SHA256 `93d80ee7b21f7da630862f015376eb39247c52ecf407a763d403c34e1b778190`
and `PRODUCTION_CONTRACT_OPTIONS_REVIEW.md` SHA256
`2e57542956adcaad42fcc839a4a6c2a8867080b3c1675b871798a4653701f13c`.
The production repair is NOT_IMPLEMENTED at this milestone; ordinary runtime
remains exact79.

Independent preflight review accepted the connected native transport proposal
with a required call-local eligibility binding captured before await from the
actual Legacy output and pre-effect managed mode. Post-effect false return,
exact one-Space delta, epoch+1, unchanged revision and current same-owner
KnownStart must be revalidated. No persistent flag or new authority owner is
needed. Receipt: `native-space-diagnostic/PREFLIGHT_INDEPENDENT_REVIEW.md`,
SHA256 `b837c7450465ce9d55e17a45678ec0af9ceab88e8b362b7ce25f88763db73ba0`.
Decision: `decisions/2026-10-06-native-space-observed-boundary.json`. This
authorizes one private connected RED/repair and explicit transport-test
migration; production import, complete source acceptance, physical acceptance
and ordinary installation still require their own results. The original
downstream/manual/negative semantics and failure ledger remain mandatory.

The first connected tests-only RED attempt did not reach runtime assertions.
Discovery compilation stopped on E0616 because residuals.rs accessed the
private completed_replay and strict_prefix_echo_revision fields. Actual
receipt: native-space-connected-red-v1-import/RESULT.json, SHA256
1fd3b3fe969174151288d7b39b86cbed47eb901fb8889fb5141c424d08d07505;
28.879 s, compile_succeeded=false, no test execution. The immutable A parent
and ROOT runtime source were unchanged. This is a test-authoring failure,
not a reproduced semantic failure or quality result. The next tests-only
repair must use existing observable predicates without widening production
field visibility. The causal marker and all prior controls remain required.

The corrected tests-only connected RED compiled and reproduced the exact
first loss in 35.401 s: 8 identities executed, one expected causal failure
(native Space erased eligible boundary predecessor), seven unchanged controls
PASS including all four physical_double_shift_owner_ controls. Receipt:
`native-space-connected-red-v2-import/RESULT.json`, SHA256
21337cf89a09a9378ca5bd41e006eee21e56ab5bc36947e4a16f5095af56490d.
Immutable A production was unchanged; compilation visibility repair used
existing observable no-replay and no inherited-display/manual predicates.
This establishes the causal regression detector, not repair success. Later
positive Reset/receipt/manual cells remain unexecuted after that first panic.
Runtime authority is unchanged; exact79 remains ordinary installed.

The private connected kernel passed its focused GREEN in 33.870 s: all five
identities passed, including the causal test with four internal cells
(ASCII/Unicode x ordinary/manual-suppressed) through actual Reset, exact
receipt, ManualToggle and authoritative visible-tail consumption; all four
unchanged physical_double_shift_owner_ identities passed. Receipt:
`native-space-core-green-v1-import/RESULT.json`, SHA256
d99b4daae990534fb3018ad174fc95f09bb91988cd15466e7a6f518d2c36e373.
Only the reviewed observation kernel and tests-only RED overlay changed on
immutable A. Callback-local eligibility and strict post-effect guards retain
existing pending provenance unconfirmed; original confirmation consumers
remain. This is focused source evidence, not complete affected-closure or
release acceptance. The conflicting old A pendingNone oracle awaits explicit
strengthening in the connected transport migration; it is not waived. ROOT
production and ordinary installed79 remain unchanged; no GUI experiment or
runtime authority change occurred in this source check.

The connected first affected-closure run executed all 117 unique identities
in 104.712 s: 115 PASS and two FAIL. Both stopped at the same old handled-true
expectation in the implicit ASCII Space helper, before their downstream
Backspace/command-gap semantics. Exact failures: residual_observed_boundary_
backspace_cannot_cross_a_command_input_gap and residual_observed_boundary_
backspace_keeps_start_and_retires_old_token. Raw receipt:
`native-space-connected-green-v1-import/RESULT.json`, SHA256
9462e38b742d1303c015968d172976f368dbdcf21647a0fba69f5cd87e54b5f8.
This is not connected PASS. One shared test-transport migration remains;
its original downstream guards must remain and all 117 will be rerun.
Actual canonical selection is 111 baseline plus six added identities,
without duplicates; the lexical index missed two implicit Space callers.
The compiled/formatted observation kernel hash is
633dac8291b22c44b8932e4171d15b10b38dd2c1458d0d409e5d204a84eac3cf,
distinct from frozen raw input 6fb00d28. All runtime source remains private;
ordinary installed79 and runtime authority are unchanged.

Connected Round2 passed all 117 exact identities in 104.515 s, with zero
failures/errors and unchanged compiled runtime kernel633dac. The only Round2
change was the shared Space setup; all non-Space/downstream assertions stayed
literal. Receipt: `native-space-connected-green-v2-import/RESULT.json`,
SHA2561477b664b6cd674af4af162cd58adc8e80562305ed11ea98e76540da5f29acbf.
Nine actual formatted source files were imported against their exact ROOT
before hashes; `native-space-connected-green-v2-import/ROOT_SOURCE_IMPORT.json`
records the import. Independent connected+Round2 review e8098dd15b2cf4621d89e4af84c6abe1a98e855ab931bfd6ff535cbf8fd85d46
found no additional source/authority blocker. Old failed receipts remain.
This establishes affected-source acceptance only. Complete source release
gate, changed-byte physical matrix, actual keyboard and ordinary installation
remain NOT_TESTED. ROOT source changes, installed79 does not; runtime authority
is unchanged. The full gate must retain all old canonical rows and zero ledger.

## Explicit affected-closure replan after full V16 — 2026-10-06

Full V16 did not pass: 3009 correctness/package identities selected, two
TD125 failures and 3007 passes; CHECK_CHANGED failed after388.802s, overall
472.991s. Receipt `release80-v16-import/RELEASE_GATE_RESULT.json`, SHA256
2d9912c3183dd3ceded78eaf80210704673db886c14f4c572d51b6b90846b671.
Raw IME log `release80-v16-import/bin-lay-ibus-engine.log`, SHA256
4c040edcc40b96fa23db1a88e1bca0826f7cea99b722d5757556bdbc4d55b8ab.
Both actual panics require handled=true at the uncapped Legacy NoApply Space
setup: td125_soft_reset_discards_owned_preedit_from_local_and_shared_authority
(line534) and td125_cursor_zero_backspace_cancels_owned_preedit_and_tracks_native_delete
(line593). Their later reset/delete/local/shared authority assertions were
not reached. This evidence establishes a missing transport-test migration,
not downstream semantic acceptance. Source graph and canon passed; later
full release stages/artifact and changed-byte clients remain unproved.

The previous two-round 117-identity closure plan is ended, not silently
extended. The explicit new plan in the owning ADR expands selection to
the complete TD125 legacy preedit module plus prior117, retains every old
identity and failure ledger, and allows at most two reviewed repair rounds.
Only those two direct transport expectations may migrate: require native
false, no Space Commit/Delete/Forward, exact Clear/Hide or no effects, one
local separator and unchanged downstream Reset/Backspace/authority guards.
No runtime change is currently indicated. After focused proof, run the entire
required source gate, then the changed-byte physical matrix and actual
keyboard. No failed assertion is waived; installed79 remains unchanged.

The explicitly replanned closure passed all136 unique identities in114.511s,
including the two previously failing TD125 sites and all19 module identities.
Receipt `native-space-full-closure-green-v1-import/RESULT.json`, SHA256
cedc88318aa6451ba168acbaf2c0eeb896c809f4fb325801088b15edc78807a4. Independent review
bfe13aa043003ceb8dccbadaa9eb70ead7f08cdd408f35b0351e52a727bc1ea9
found zero blockers; all original downstream semantic assertions remain.
Only the actual formatted test file was imported against parent5276a008.
The runtime kernel633dac and installed79 remain unchanged. This source-only
result does not establish full release or changed-byte physical acceptance.
Full source gate V17 is the next requirement; no failed receipt is discarded.

## V17 full-repeat failure and explicit dispatcher-boundary replan

V17 first correctness/package canonical pass was3009/3009 in399.612s.
FULL_GATE repeated canonical execution and failed on one pending-auto-undo
bridge test; overall886.238s, full gate401.900s. Failed receipt
`release80-v17-import-failed/RELEASE_GATE_RESULT.json`, SHA256
6b6c594fdf1a3c3af8581f2c6f8be5a2a5402a58b5692be7c63dfe57ab6e8950.
Actual isolated log `release80-v17-import-failed/failed-undo.log`, SHA256
0b5c6627b6a85eed4a4806ddee5d53f55ccef50704a4db4eb6a1896f182886c0
shows residual_pending_auto_undo_bridge_keeps_ime_ownership_without_delegation
at residuals5891: Ok(0,false), expected(1,false), elapsed0.01s. Ping and observer
assertions passed; later ownership/no-delegation/awaiting-exact assertions
were not reached. This is not a full source PASS or proven timeout cause.

The previous transport-test-only closure repair passed and is not silently
expanded into a runtime repair. A new explicit plan in controlled-manual-
dispatch-isolation ADR investigates the shared absent→served fixture boundary.
Static analysis identifies a final manually consumed Space RELEASE3001 that
may still reach the automatic object dispatcher after engine registration;
a duplicate may reuse its cached stamp, but the header has already been
settled; second reducer settlement refuses and revokes word authority.
The old failed run does not contain that serial/refusal trace, so original
causal timing remains UNKNOWN. A bounded controlled duplicate witness and
existing exact absent-dispatch fence will distinguish this mechanism before
repair. All semantic assertions, production guards, timers and failure ledger
remain mandatory; max two reviewed rounds. No runtime source change is
authorized by this fixture plan, installed79 remains exact and unchanged.

The new tests-only baseline handoff detector compiled and failed its exact
first assertion in33.736s; four unchanged physical-owner controls
passed. Receipt `undo-registration-red-v1-import/RESULT.json`, SHA256
64c26f67b081de2333b3f86199e0cd10e94f10b1c597e3e9d79d5736231bbfa7, statusOLD_FAILURE_REPRODUCED. Scope:
baseline known_engine leaves the declared final manual release3001 unfinished
at return. This does not reproduce or identify the historical V17 timing.
The later forced registered duplicate cell was not reached. Runtime source
and installed79 remain unchanged. GREEN137 is prepared, not yet accepted.

The reviewed registration-boundary GREEN passed137/137 in115.212s.
Receipt `undo-registration-green-v1-import/RESULT.json`, SHA256
19be9d8d469ea3c38bde137307b7ea6f6057a6631eb1d4c6e7b4ecd664dca95e. The new test actually reached its
controlled registered duplicate cell: same original tail/owner/epoch but
UnknownStart and revoked token after duplicate settlement; the strict bridge
refusal and empty output passed. Every previous semantic assertion remains.
Only actual formatted residual test source was imported against f9ebe2a4;
independent review a5419119 reports zero blockers. This supports the narrow
initial manual setup completion fence, not original V17 causal timing and
not later numeric/Reset/publication seams. Production source and installed79
unchanged. Full source gate V18, changed-byte Firefox and hardware remain
pending; no failed evidence was waived.

## V18 repeated canonical PASS, stale Clippy expectation blocks release

Both3010canonical runs passed. The actual full gate failed at default Clippy
afterwards, overall924.463s, full gate439.445s. Receipt
`release80-v18-import-failed/RELEASE_GATE_RESULT.json`, SHA256
597ae874b99f7997b1d33cac748d4a8afe510cf4bb34ee39fdd31dad037f945e. Its canonical_full field remains null because
FULL_GATE returned nonzero; the postfailure shared SUMMARY capture separately
reports3010/PASS, SHA256 26706d3440bd1a55c803aab3443e9cd82df58f3436c54c4df8def412194a18b2.
A guarded explicit bin Clippy reproduced exit101 in15.489s: obsolete
expect(clippy::collapsible_if) on composition_commit now unfulfilled.
Diagnostic SHA256 50d870da4dda17c61f328b324194167fc52e308112ea13499f2c5b6a149ba9ee. The old accepted bytes had
a nested branch; current native Space implementation no longer emits the lint.
Explicit decision remove-stale-composition-lint-expectation removes only
that annotation, preserving strict -Dwarnings, lint baselines and all runtime
statements. Independent review and strict all-target Clippy are pending;
then complete source and changed-byte physical proof remain mandatory.
Installed79 unchanged, no source release PASS is claimed.

Read-only current-hint audit also corrects an earlier overly broad lineage
claim: old observer-matrix V18 on full-source V15 had its first two Backspace
settlements UnknownStart. The Tab settlements KnownStart occur after accepted
boundary, not proof of pre-Tab KnownStart. Actual3/3 effect and old three
pre-Tab observer failures remain distinct. Source permits exact current
owned-preedit append under UnknownStart if all live scope/start gates pass.
Audit UNKNOWN_START_CURRENT_HINT_ADMISSION_AUDIT.md SHA6a1d3b47 and
V15_BACKSPACE_HINT_FRESHNESS_AUDIT.md SHA6a3441c8 are private read-only
evidence; current full-release V18 has no changed-byte physical result.

Annotation-only removal passed actual strict all-target default and research
Clippy in77.068s, including dead-code inventories532/356 and zero other
diagnostics, source/parent unchanged. Receipt
`stale-composition-expect-clippy-v1-import/RESULT.json`, SHA256
459aa389f257034d3a58afd7d0a974f05f4bc1e0698814eeffd9f8d4e1264220. Independent review59a25725 confirms
only the four-line expectation deleted and all other bytes literal.
Full release V19 is still required; installed79 remains unchanged.

V19 stopped at CANON in84.557s before canonical execution: the new lint
decision lacked required verification/not_tested text fields. This is a ROOT
document-format error; existing runtime and strict Clippy remain unchanged.
Failed receipt `release80-v19-import-failed/RELEASE_GATE_RESULT.json`, SHA256
08cbcd3036fdcdf49af3f10c763d63b635ee9a86bb3d2408401e6530ed85a05b. The missing fields now explicitly
record measured Clippy proof and untested full-source/physical scope.
No validation rule changed or failure waived; next full gate is V20.

## V20 full gate and actual installation80 — 2026-10-06

Latest runtime: **1.0.80 installed and loaded**, not an all-Firefox acceptance.
User repeated «установи 80» after the hardware prerequisite and install-without-check
option were explained. Decision:
[one local user-directed installation](decisions/2026-10-06-user-directed-version80-install.json).
The real-keyboard check is **NOT_PERFORMED_BY_USER_DIRECTION**, not PASS. The normal
AGENTS requirement and immutable original installer remain unchanged for future work.

Evidence directory:
`/home/local-user/.cache/lay/development/contextual-completion-20261005/20261004T211126Z/release79/firefox-repair-20261005`.

- Full `release80-v20-import/RELEASE_GATE_RESULT.json`: PASS,15 command stages,
  both canonical3010 PASS, no failures; SHA7685d75b6da165cae6c7eda937d58a29971299ce9b0d6fcc1a6f7db05e629767.
  Frozen1485-file `SOURCE_AFTER.json` SHA2b0a6cc5f5496815a2367325ff9f0f141c0844755630d1ed97d3a097fb7c1ae3.
- `V20_PHYSICAL_SUMMARY.json` SHA2d53e23ee18148c896be3408c78bf54eb4d7307f105ff54e1602c836db81ab50:
  same IME03a8 bytes, unchanged daemon79; all five exact79 rollbacks PASS.
  Space3/3; native-Space eight-pair3/3 (24 word/mode/icon transitions and next letter);
  active composition eight-pair3/3; retained proposal actual Backspace→Tab3/3 PASS.
  Wide87PASS/93executed,6FAIL,0BLOCKED; second-wordTab3/3, nonletters60/60,
  heldShift3/3 PASS. Original three strict DOM composition-event prerequisites stop
  before Tab; they remain FAIL. The separate actual-Tab control does not rewrite them.
- `V20_WIDE_FIRST_LOSS_AUDIT.md` and `V20_AUTOCORRECT_POLICY_READINESS_AUDIT.md`:
  all three correction cases retain `кторое `; the separator arrives. One Space
  result not_ready, two ready→SuggestOnly; target membership and exact refusal are
  UNKNOWN. The canonical config enables correction; no blanket Russian typo ban
  is established. These findings do not justify a literal exception or gate waiver.
- `USER_DIRECTED80_PRIOR_BINDING_FAILURE.json`: first installation admission
  rejected a draft pointing to initial79 rather than the actual accepted79
  frequency/command installation. No destination mutation. Corrected only the
  caller's prior receipt to pinned830b5c4a…; exact old-byte guards unchanged.
- Actual `ordinary80-install-109a2c631bd746c9bdf687efe77e8308/INSTALL.json`:
  PASS; SHAbd7b1757999440f739b033eec8b783fb3368c0deda1fe4068bd596e0433e4ad3.
  Three imported binaries plus metadata/tray version80; current IME and daemon
  /proc hashes and `lay 1.0.80` independently verified.35 protected paths, input
  sources/config and global IBus/L3/L11 preserved. Actual79 five-file backup and
  exact guarded rollback are in that transaction's `ROLLBACK_ACTUAL79.json`.

Not tested: genuine keyboard on V20; combined installed80 physical behavior;
Instagram/WhatsApp/every Firefox field. No full release publication or user
acceptance is inferred. These documentation outcomes were appended only after
successful transaction source closure; they do not change compiled V20 bytes.
