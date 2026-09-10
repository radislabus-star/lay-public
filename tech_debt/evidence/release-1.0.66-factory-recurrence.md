# Lay 1.0.66: observer recurrence on CreateEngine, 2026-09-07

Status: PHYSICAL_ACCEPTANCE_FAILED / PREVIEW_STOPPED / MINIMAL_ADAPTER_REFUSAL_REPAIR.
Owner: [TD-121](../121-preserve-word-across-ime-layout-handoff.md).
Baseline: `d49bb0180fdd957b7b91102f09c3c5336f258a61`.

Latest result:436/436 focused,2671/2671 affected,29/29 harness; runtime and
witness review9/10,H0/M0. Remote lifecycle3/3, post-ready restoration5/5;
immediate cold0/5 retained. Candidate706b4d81 subsequently FAILED physical
acceptance on Sep8 and was safely stopped after verified native selection.
The user rejected the overbuilt diagnostic detour and requests a minimal,
reliable adapter repair. The unfinished opt-in recorder is paused and will be
shelved recoverably, not shipped. Existing debug_action_log is already enabled.
Earlier preview/liveness results below are historical, not product acceptance.

## Live evidence and first loss

At 17:17 +03:00 the user reported that IME still did not work and suspected
unrestarted services. Read-only checks rule out stale process images: IME
PID1148501 has installed/process SHA256
`95348e4a81056a9a476c9aade72194978a4c359abe81ac6983a381989423a668`.
Daemon272240, L3272161 and L1.1271400 also match their installed binaries.
Global IBus4715 is unchanged; IBus and GNOME both select `lay-ime-ru` and the
user input-source list remains unchanged. IME is launched by IBus; the absent
`lay-ibus-engine.service` is not a failed required systemd service.

`Focused` is true, but `InputState` again fails with
`metadata observer cancelled`. The earlier `passive:no-focus` readback proved
only initial adapter availability, not durable live acceptance. The preceding
patch therefore failed physical acceptance despite its scoped tests/review.

Private preserved log (user text; never commit contents):
`/home/ubu/.cache/lay/development/release-1066-observer-recurrence-L5XnoP/ibus_engine_debug.jsonl`.
SHA256 `198c1e4e9b8b09ae37844a8d4bf9b58bab288f37f055b079cbd287babc4f5ab6`.
Events do not carry wall-clock timestamps; 17:17 is the observation time.

Observed sequence, metadata only:

1. `CreateEngine61` opens a factory request with owner/activation generation1.
2. Observer records `FocusOut63`, `Disable64`, then `FocusIn66`, `Enable67`.
3. Consumer gets the FocusOut stamp, but Disable64 is `refused_timeout` and
   FocusIn66 is `refused_revoked`; following keys are passive/refused.
4. `CreateEngine123` sees `transfer_revoked`, then `observer_ingress_refused`
   with disposition `denied`; the observer stops with `context admission denied`.
5. All **526/526** retained key-admission callbacks after that refusal fail.
   There are34 accepted callbacks before it; this is not a quality denominator.

The earlier timeout is the first observed authority loss. The later factory
event turns a failed transfer into process-lifetime observation failure.
`open_factory_request` rejects every retained ticket except Consumed, including
Revoked; the adapter promotes its None result into fatal Denied. Exact cause
of the earlier scheduling timeout is not measured by this trace.

## Explicit new scope and alternatives

The prior two-pass review belongs to the FocusOut/Disable patch and is complete.
This recurrence is a separately declared bounded replan, not an unreported
third repair under the old PASS. No broad architecture or Wave-quality work.

- Restart unchanged IME:2/10. Temporary availability, same fatal state remains.
- Restart every Lay/global IBus service:1/10. No stale-image evidence; broader
  disruption does not repair the demonstrated state transition.
- Restore preserved1.0.65:8/10 for temporary recovery, requires explicit user
  approval; no such approval has been received.
- Retire a revoked transfer on a fresh authenticated factory event and acquire
  a source-free UnknownStart context:9/10, recommended for durable recovery.
- Swallow all Denied as Passive:1/10. Rejected: observer survival without new
  input authority is still broken and trust failures must remain fatal.

## Consequence analysis and proposed proof boundary

Use the existing reducer, source-free acquisition, callback stamps and fenced
native receipt. Do not add a retry, polling loop, deadline extension, cache,
supervisor, new authority owner or fallback text-edit route. A revoked ticket
must never supply its source tail, old token, seal or ready result to a new
factory. New authority requires the ordinary current context/profile/marker
proof; UnknownStart remains unknown until a genuinely observed word boundary.

Candidate generation, lattice retention/ranking, lexical packages, learning,
SafetyGate, verifier and deletion geometry are unchanged. The proposal changes
only lifecycle admission: no extra hot-key RPC, background worker, unbounded
queue or added allocation structure. Existing acquisition/Space deadlines,
CPU/RSS limits, package identity and reload behavior remain unchanged.

Main hazards: reviving a revoked source; clearing a different-path successor;
letting an old factory binding, timeout, marker or ready publication damage
the new request; treating a pending valid transfer as cancelled; accepting a
foreign sender, malformed payload or backwards receive position. Preserve
receive order and the established pending-before-reducer lock order.

Independent read-only mechanism audit confirms the exact fatal path and the
asymmetry: revoked source-free reservations already count as non-live, whereas
revoked transfer tickets are rejected as if still live. Chosen retirement
invariant: only TicketStatus::Revoked and a factory receive position strictly
after all retained factory/focus-out/disable/target-focus positions may retire
the ticket, old owner and activation as transfer sources. Consumed handoff
behavior and live Pending/Ready overlap rejection remain unchanged.

A supporting stale-fence hazard is included in the proof: readiness refresh
currently returns on an unobserved marker before checking whether the fence's
request is still current. A revoked predecessor must not occupy the singleton
pending slot for the fresh source-free request. If causally demonstrated,
retire that non-current fence before the marker-ready early return, preserving
exact nonce/generation cleanup and pending-before-reducer locking. This does
not turn an unobserved current marker into readiness.

Old factory callbacks carry a ticket ID directly into `bind_factory_target`.
After terminal-ticket retirement, a mismatching old ID must return false
without revoking the successor. Keep existing rejection/revocation for invalid
binding of the SAME current ID. This is exact identity isolation, not relaxing
current-ticket target validation.

Pre-edit RED: remote `run-EnIzU9/tests/SUMMARY.json`,418/419 passed (pure
factory-retirement regression failed). Combined actual-observer RED:
`run-UWQ88T/tests/SUMMARY.json`,418/420 passed; the reducer test and the real
observer test both fail at the later factory, the latter with `Denied`.
Build4.21s, selected target5.777s, remote action14.313s. No production change
was present in either snapshot; these are causal failure reproductions.

Implemented the three bounded changes described above: terminal transfer
retirement into source-free acquisition, exact-ID stale binding isolation,
and non-current pending-fence retirement before marker readiness. No new
runtime subsystem or deadline. First repaired run `run-QeKehL` was419/420:
the new observer scenario advanced beyond the old failure and encountered an
invalid hyphenated D-Bus path in the new fixture. Corrected only the two test
object paths to valid underscore names; no expectation was weakened.
Final focused run `run-Bo4eDq/tests/SUMMARY.json`: **420/420 PASS**, build4.34s,
target5.631s, remote action14.290s. SUMMARY SHA256:
`2750ee3656ad606f3695375fa5994e29718af16a5667abc76c96f726846b3012`.

## User-authorized manual hot candidate, not release acceptance

The user explicitly chose «да давай я буду проверять на горячую» and required
«только без клавитуры меня не оставь». Prepare a temporary candidate process
from the delivery cache; do not overwrite the installed IME binary or mark
final canonical/release gates PASS. Use the existing native XKB staging before
terminating the exact failed managed IME, so ordinary keyboard input remains
available. Start the temporary candidate with a separately named user unit,
restore the captured Lay engine only after its process/name is available, and
retain a native-input fallback on startup/bridge failure. Do not restart global
IBus or the keyboard daemon, change input-source lists, or inject user input.

Remote candidate build/architecture refresh:
`/home/e/projects/lay-development-runner/ime-factory-hot-AtSYjX/`.
Local delivery cache:
`/home/ubu/.cache/lay/development/ime-factory-hot-n600Z6/`.
Independent bounded source review is running separately. This is the user-
approved manual candidate session, not permanent installation, final release,
task DONE or proof that the earlier timeout is fixed. Final release checks
remain required after useful physical acceptance; the known failing installed
artifact must not be called healthy merely because it is preserved for rollback.

### Executed hot preview and mandatory safe withdrawal

Remote release-profile build58.91s, architecture PASS; candidate SHA256
`240d009b76043ff5bce59bd44538080cba5dfe9f517778e2fcfa33335084958e`.
Bounded fresh-context review9/10,High0/Medium0, static only. At17:42:52 +03:00,
temporarily selected verified `xkb:ru::rus`, terminated only the old failed IME
PID1148501 and started the candidate from the cache as
`lay-ime-hot-240d009b.service`, PID1378493. Restored `lay-ime-ru`; initial
InputState was `passive:no-focus`. Installed IME bytes remained `95348e4a`.
Startup log `start-hot.log` in the local cache, SHA256
`1472f07cb91ab5d58ad05f8f4d341edebaeb7f1dcbb50e09825d82e7d4bbeb63`.

The subsequent live check FAILED: `metadata observer cancelled` recurred.
The candidate-specific metadata shows repeated normal focus/reset/property
events, followed by fatal `observer_ingress_refused / ProcessKeyEvent139`.
There were364 refused key admissions and no accepted key admissions in that
readback. This is a new unclosed key-path failure, not evidence that the
factory regression or420-test PASS establishes product recovery. No additional
runtime repair is silently admitted by the previous review score.

In accordance with the user's keyboard-safety requirement, selected and
verified `xkb:ru::rus` BEFORE stopping only the temporary candidate unit.
Final hot session state: STOPPED; native input engine `xkb:ru::rus`; global
IBus4715, keyboard daemon272240, L3272161 and L1.1271400 unchanged. Input-source
list unchanged; installed IME bytes still `95348e4a`. No permanent installation,
new commit/push, rollback to1.0.65 or final release acceptance occurred.
The source candidate remains unaccepted; TD121/TD125 are not DONE.

Private failure trace in the local cache: `failed-hot-trace.jsonl`, SHA256
`41fc5fc48a12a7fd0365270a9cdb5f85e66dbc9d76caa78c7496d37c57fc5ce6`.
Do not commit its user text. Withdrawal receipt: `stop-hot-native.log`.
Ordinary input is routed through the native engine, not the failed temporary
IME; physical keyboard readback still belongs to the user. Restoring Lay input
sources later may relaunch the still-unfixed installed image, so do not report
native fallback as restored Lay functionality.

TDD must reproduce the
old fatal factory transition through the actual observer and then prove both
survival AND later source-free key/bridge admission. Pure reducer tests cover
the relevant cancelled/consumed/live status matrix and stale identities;
existing sender/transport and successor-owner negatives remain required.
Do not relabel the initial timeout as fixed merely because a later factory
recovers. If first-use desktop handoff still loses authority, report that
separately and do not claim product acceptance.

All builds/tests remain remote under dedicated-20cpu and existing resource/
Cargo guards. Production binaries/services are unchanged during this diagnosis.
Any accepted successor needs scoped RED/GREEN, affected contracts, at most two
new bounded review/repair passes, exact-client evidence and honest physical
readback. Do not reuse the preceding five-case PASS as recurrence coverage.

## Key-path recurrence: bounded replan after failed preview

The user confirmed «Обычный нормально!»: native typing works. Keep
`xkb:ru::rus` selected while developing; no desktop/input-source-list changes.
Live read-only recheck: global IBus4715 and keyboard daemon272240 unchanged.
The private failure snapshot is a rolling512KB trace: the initial fatal entry
has rolled out of the retained file. ProcessKeyEvent139 is from the earlier
live filtered readback, not a newly reproduced parse of that snapshot.

Source-confirmed first mechanism to reproduce: the observer consumes and
publishes a fenced activation before the engine installs it. Reset/Properties
Set can revoke its word token in that interval; stale-ready cleanup discards
the grant, while the reducer retains its current owner/activation. The engine
has no local owner. Each later observed key is registered as unsettled, but
the ownerless callback only consumes its stamp and never settles the entry.
The65th such ingress exceeds the unchanged64-key bound and cancels observation.
This is a causal source hypothesis for the live schedule until the controlled
real-zbus regression runs; the rolling log cannot establish every ordering.

Options (scores are engineering judgment, not measured outcomes):

- Increase key/stamp budgets:1/10, only delays the same permanent failure.
- Account for rejected callbacks only:5/10, prevents the bookkeeping leak but
  leaves the engine unable to regain useful same-focus word handling.
- Move reducer activation consumption into the engine:6/10, viable larger
  ownership/timing redesign, not justified for this urgent bounded repair.
- Keep revoked grants invalid, account for every completed callback, and let
  an exactly observed key bind only the already-fenced current activation as
  empty UnknownStart:9/10 provisional recommendation. Needs explicit proof
  of lifecycle settlement and exact sender/path/owner/header identity; retained
  owner identity alone is insufficient after FocusOut/Disable/new acquisition.

Pre-production consequence check: do not revive a revoked transfer or replay
its tail/suppression/learning. An empty local word scope is not KnownStart;
only a subsequent actually observed boundary can rearm correction. Preserve
sender validation, stale-owner isolation, lifecycle/acquisition deadlines and
true queue-overflow refusal. No new owner, RPC, timer, retry, cache, unbounded
queue or physical Double Shift detector. Candidate/lattice retention, ranking,
models/package identity/reloads, online learning and verifier/SafetyGate are
unchanged; callbacks lacking proof still have no correction authority.

Key hazards: adopting a retained but unfocused owner; a delayed key affecting a
successor; minting a native receipt from a key; accepting old known lineage;
resetting a newer shared tail epoch; clearing outstanding work belonging to a
different owner. Require exact callback settlement/abandonment identity and
monotone empty-tail epoch binding, with fresh token revalidation at local
installation. Abandonment must not manufacture word evidence or bypass the
quiescence needed by bridge reads. Lock order and no-await critical sections
remain unchanged. CPU/RSS work stays bounded by the existing64 entries and
one local activation; no model work or new per-key allocation container.

Proof denominator: actual observer+engine Reset-before-ready-install followed
by more than64 keys, plus same-focus boundary/bridge recovery and stale-owner/
revoked-transfer negatives; then the focused IME target. Independent bounded
review remains separate from product acceptance. Installed bytes and working
native input are the rollback boundary. Do not make another hot candidate or
claim DONE on bookkeeping survival alone. No production key-recovery change
has been made at this analysis checkpoint.

Design review REJECTED generic key-token adoption: Foreign-to-Lay reentry can
retain owner/activation, so an arbitrary key is not a renewed focus receipt.
Chosen implementation instead retains the existing ready slot as an
uninstalled witness until the runtime acknowledges exact installation. Only
an authenticated Reset/Set, with that witness current BEFORE revocation and
lifecycle settled, may convert it to an explicitly empty ResetUnknown outcome.
This retains its original fenced context/receipt origin but never its source
tail. The old outcome/token remains revoked. Foreign profile changes, owner
loss and unrelated revocations still destroy invalid witnesses; pending focus
handoffs cannot qualify for ResetUnknown. No generic key-driven revival.

Installation of ResetUnknown rebinds an empty epoch strictly above reducer,
shared and local tails, revalidates its exact identity, clears local/shared
word and suppression state, then acknowledges only that exact witness. A
Reset racing between peek and install leaves its replacement witness intact;
the stale installer cannot acknowledge it. Existing deadline/marker work is
not repeated. This replaces eager runtime removal from the ready slot, not a
second pending queue or independent activation owner. The new internal outcome
distinguishes reset of proven context from a fresh native receipt.

Initial remote run `run-Jek5PL`:421/422 passed; the ownerless scenario stopped
early on a fixture assumption that soft Reset immediately empties the committed
tail. Soft Reset deliberately preserves that mirror. Moved that empty-tail
expectation to recovery, where no old tail may be installed; retained old-token
and UnknownStart rejection before recovery. The established-owner burst passes
unchanged, so no speculative lineage synchronization repair is included.
The corrected pre-production run must reach the predicted queue failure.

Corrected RED `run-BjdngU`:421/422 passed; actual legacy callback fails with
observer `Denied` while completing the more-than64-key sequence. The initial
mirror assertion no longer masks the queue failure. No production repair was
in this snapshot. Discovery4.58s and execution approximately5.8s are focused
checks, not desktop recovery evidence.

The existing stamp wait can also allow a ready witness to appear after the
callback's initial nonblocking readiness check. Recheck that same slot once
after receiving the exact stamp, with shared key-stamp validation and no
second await or retry. If no usable witness exists, abandon the exact observed
key and revoke its same-owner pending seal. Unobserved callback timeouts remain
outside this proven recovery claim; they cannot supply invented key evidence.

Implemented witness-preserving ResetUnknown plus exact abandoned-key handling.
Remote focused GREEN `run-A9s2oU/tests/SUMMARY.json`: **425/425 PASS**,
discovery4.573s, execution5.785s, remote action14.479s. SUMMARY SHA256
`b10ded4c8a677f3bb5c8a0dca39a87d426b0c8579d1bbd20821cacaa1f5e9138`.
The observer-ahead three-Reset regression now recovers on the first actual
Shift callback without synthetic FocusIn or a new owner, processes66 complete
Shift callbacks, then proves the first actual Space boundary and bridge
admission. The old word/tail/token remain invalid. Transfer peek-before-Reset
and Properties Set variants reject stale installation but preserve only the
empty replacement; Foreign-to-Lay and FocusOut-to-Reset do not revive it.
Pure negatives retain exact abandonment identity, sealed-transfer invalidation
and fail-closed behavior for genuinely64-unsettled-key overflow.

Runtime ACK validates and removes the exact ready witness under the existing
ready-before-reducer locks. The legacy destructive take helper is test-only;
runtime uses peek/install/ACK. Independent fresh-context review and optimized
candidate/private-client checks are pending. Native engine remains
`xkb:ru::rus`; no new hot preview, permanent installation or release acceptance
has happened at this checkpoint.

Independent fresh-context implementation review identified two HIGH races in
pass1: stale-ready pruning validated a cloned outcome but removed a replacement
by request/owner only; publication consumed reducer authority before exposing
the ready witness to concurrent Reset. Single repair pass serializes pruning
under ready->reducer and publication under pending->ready->reducer. Reset and
ACK use ready->reducer; no path holds these locks while acquiring pending or
awaiting. Publish/replacement/prune now share the ready critical section.
The425-test run predates these lock repairs and must be rerun. The optimized
artifact `ime-reset-hot-ZCdpqm`, SHA256
`f7166f9cd2421f935533dfc11327164e4b69efbd8db3c25809c4ff1e10cb33eb`, also predates
them: SUPERSEDED BEFORE HOT USE, never installed or launched on the desktop.

The same bounded review also tightened three related contracts: ACK now
requires a present exact witness (a current token alone is insufficient);
empty-tail bind failures exit the Shared lock before discarding provisional
local state; owner stamp generation is published inside the reducer-consume
critical section. Runtime's only installer caller owns strict peek/install/ACK;
the common local installer retains final original-outcome revalidation for
direct fixtures. The real66-key test additionally proves that a still-current
token cannot acknowledge an absent/already-installed witness. These are the
same review/repair pass, not new features or a relaxed acceptance boundary.
Intermediate remote `run-mnmSe2` PASS predates the final three tightenings;
final focused verification is required for the frozen source.

## Final source and temporary preview, 20:06 +03:00

Frozen final source: remote `run-dZPERN/tests/SUMMARY.json`,425/425 PASS;
discovery4.645s, execution5.895s, action14.634s. SUMMARY SHA256
`bbd4969a7697b8ee56616a9b156945feb4dfec9838bdc5e06372b84936b7c7cd`.
Fresh-context review9/10,High0/Medium0 after the single bounded repair pass;
static review is separate from actual-client/physical acceptance.

Final optimized build58.67s, architecture PASS:
`/home/e/projects/lay-development-runner/ime-reset-final-rQnmd5/`.
Architecture log SHA256
`9b557542eab32ae0cdf1dbb2534d84ca158911d3b2e370d17bdb7e995bfc63aa`.
Final target9,381,564,416 /12,884,901,888 bytes. Candidate SHA256
`157287646b31b1ea967804a446bce3d9530988063893b7ea3d20d264da09f5d0`.

Private client evidence is deliberately split:

- `run-Jw5ISf`: prelaunch configuration refusal; used noncanonical manifest
  basename. Corrected to existing admitted dependency-manifest.json, unchanged
  expected manifest hash and nine-role dependency contents. No candidate ran.
- `run-eEOK4x/client`: immediate cold FAIL in the first scenario; zero completed
  cases. Keys are admitted/settled KnownStart and transfer installs, but first
  Space returns prefetch_not_ready before exact warmup completes at111681us.
  Visible fixture word stays `ljv`; cold acceptance remains FAILED.
- `ime-reset-final-rQnmd5/client-post-ready`: distinct existing
  post-exact-ready schedule,5/5 PASS,2.726s service wall time. No scenario edits,
  sleeps, per-key retries or deadline changes. Both private candidate/IBus
  process trees were reaped. This PASS does not replace the immediate failure.

Local cache `/home/ubu/.cache/lay/development/ime-reset-hot-Azb9dB/` contains
the exact candidate, client config and rollback-on-start-failure preview script.
First launch was safely withdrawn BEFORE Lay selection: its startup probe
incorrectly requested InputState authority while still on native/no-focus.
Changed only the staging check to public Ping/Focused; when focused it still
requires InputState. First attempt log `start-hot-prefocus-refusal.log` retained.
No production code/assertion changed for this staging correction.

At20:06:38 temporary unit `lay-ime-hot-15728764.service`, PID2128979, started
from the cache; `/proc/PID/exe` hash matches the candidate. Selected `lay-ime-ru`.
Subsequent readback: Focused=false, InputState=passive:no-focus. Startup log
`start-hot-2.log`; no text was injected. User physical input is still pending.
Global IBus4715, keyboard daemon272240 and the source list remain unchanged.
Installed binary95348e4a remains unchanged. No permanent install, final release
acceptance, task DONE, new commit or push is claimed. If the candidate fails,
select and verify native `xkb:ru::rus` before stopping this exact temporary unit.

## Physical rejection and safe withdrawal, 20:24 +03:00

User: «неработает». The temporary15728764 candidate FAILED physical acceptance.
Selected and verified native `xkb:ru::rus` BEFORE stopping only
`lay-ime-hot-15728764.service`; PID2128979 exited. Global IBus4715 and keyboard
daemon272240 remain unchanged. Installed95348e4a bytes and source list unchanged.
Native selection is confirmed; no new claim about Lay functionality is made.

Retained private trace (never commit its user text):
`/home/ubu/.cache/lay/development/ime-reset-hot-Azb9dB/failed-physical-trace.jsonl`,
SHA256 `a10c42640dee4c66ec6ba365f4112287787484e9eabf5ed3561d8d45ead2b964`.
Metadata shows compatibility FocusIn14, source-free ready, FocusOut20 with
local installation, FocusIn21, Properties Set24, then all151 retained legacy
key admissions refused (no accepted settlements). CreateEngine83 retires a
revoked transfer; CreateEngine85 sees no_live_acquisition and is refused,
terminating the observer. First visible loss precedes the final factory error:
after compatibility refocus/Set the engine has no local owner although the
observer still identifies owner1. The exact failing reducer transition needs
a new causal proof; do not call the preceding focused/post-ready checks product
acceptance or silently extend their completed review scope.

Read-only rollback inspection verified the preinstall snapshot exists at
`/home/ubu/.local/state/lay/release-backups/1.0.66-preinstall-b4bd39/` with bin,
extension,L2 and systemd artifacts. Snapshot lay and lay-ibus-engine both report
1.0.65; extension metadata version1000065. No rollback was executed. Returning
the whole prior release is a separate user decision, not an inferred license
to overwrite installed artifacts or restart unrelated services.

## Accepted systemic lifecycle repair, 2026-09-07 20:44 +03:00

Status: SOURCE_REPAIR_VERIFIED / PRODUCTION_UNCHANGED / PHYSICAL_ACCEPTANCE_FAILED.
The user explicitly accepted the existing-mechanism repair after the read-only
root-cause report. This is one connected lifecycle repair, not another extension
of the earlier scoped 9/10 review. Main owns integration and remote execution;
test work may be delegated on disjoint files. At most two independent review
passes apply to this repaired scenario, and a remaining finding requires an
explicit replan. No release installation or global IBus restart is implied.

### Measured baseline and source-confirmed mechanisms

The failed physical candidate remains SHA256 `157287646b31b1ea967804a446bce3d9530988063893b7ea3d20d264da09f5d0`.
At 20:41 the current process was instead the installed `95348e4a` image,
PID2249560, started20:30:30. IBus4715 and daemon272240 were unchanged.
Focused=true / InputState=`metadata observer cancelled` confirms failed
admission despite process liveness. The dated native fallback above is not
the current engine: IBus currently selects `lay-ime-ru`.

IBus1.5.34-rc2 `bus/inputcontext.c` distinguishes ordinary focus-out (only
FocusOut) from disable (FocusOut plus Disable); focus-in also forwards the
field content type. Our reflexive handoff incorrectly requires Disable.
Authenticated Set/Reset uses full lifecycle revocation, cancelling a pending
focus request as well as word evidence. The prior ResetUnknown repair covers
ready-but-uninstalled outcomes, not pending acquisition. The physical trace
does not expose exact Get-reply/Set ordering; do not invent that timing.

A later source-free factory reservation is rejected while the preceding empty
reservation is pending, and the adapter promotes that scoped collision into
fatal observer cancellation. The old diagnostic omits source_free_factory and
therefore labels a pending reservation `no_live_acquisition`.

The exact private-client baseline is remote
`ime-reset-final-rQnmd5/client-post-ready/receipt.json`:5/5 cases,64/64 legacy
admissions and64/64 settlements, but19 FocusInId,2 FocusIn,0 Set and0 Reset.
Its copied IBus SHA256 `24338c0e7cfb749d2ac9b9254d7d54029e71b5ac9342aa1b8d78b1d55f5ac1b6`
matches the live IBus executable. Fresh-daemon capability caching and uniform
FREE_FORM fields explain why version parity is not lifecycle coverage. The
specific stale FocusId cache contents are unmeasured. The immediate cold
FAIL at `run-eEOK4x/client` remains separate and cannot be replaced by warm PASS.

### Alternatives and chosen invariants, before code

- Restart/reboot:3/10; clears process/cache state but retains invalid transitions.
- Restore preserved1.0.65:8/10 for temporary recovery; requires separate approval.
- Existing reducer repair:9/10, selected; uses existing typed tickets/requests,
  exact identities and the SourceFree/UnknownStart result, no new controller.
- Rewrite admission ownership:4/10 now; larger unbounded migration than needed.

1. Require Disable only for Factory transfers. A Reflexive transfer still
   requires authenticated focus-out, exact source seal, settled keys, matching
   native context/profile, original reply/marker ordering and single consume.
2. A valid authenticated word reset for the exact in-flight target may erase
   transfer authority while retaining the original focus-acquisition attempt.
   Convert that attempt to existing SourceFree/UnknownStart: no old tail,
   suppression, learning, KnownStart or old admission token survives. Preserve
   its request generation, nonce, receipt origin, reply/marker and deadline;
   retain no authority without the original context/profile/fence proof.
   The same exact-target degradation also applies to an observed early key
   whose ownerless callback abandons the old source word during same-path
   refocus. Otherwise a key before Set would cancel that identical acquisition.
   Abandonment still requires the exact outstanding owner/header; a source key
   for a different pending target keeps the existing full-revocation behavior.
   Validate the Properties.Set payload as Engine.ContentType/(u32,u32) before
   this recovery; unrelated/malformed updates must not gain it. Stale source,
   Foreign, focus-out, disconnect and real acquisition failure remain revocations.
3. A strictly later authenticated factory may supersede only an empty pending
   source-free reservation without owner/activation/tail. Old callback/binding
   identities cannot revoke its successor. Pending/Ready source-bearing
   transfers are not discarded by this rule. Keep one reservation, no queue.
   Reject late focus from a different, superseded reservation without revoking
   the current reservation/request; matching the old binding ID alone is not
   sufficient isolation. No old target may borrow the new target's profile.
   This new rule does not remove the earlier, separately tested retirement of
   an already Revoked transfer or an ownerless source-free acquisition. Those
   contain no live source-word authority; their strict later-position guards
   remain. Pending/Ready source-bearing transfers remain non-supersedable.
   Late predecessor FocusIn after successor installation is isolated by the
   existing consumed target receipt correlated with the current owner and
   activation, not by a permanent ban on another path. Retain the existing
   consumed source-free receipt through tail binding, only until the canonical
   revocation invalidates that admission. Revocation clears terminal consumed
   receipts and permits the previous fresh UnknownStart recovery; Pending/Ready
   tickets still become Revoked, not reusable. This includes authenticated Reset
   and the already supported loss-of-focus recovery. No new historical list or
   state generation. `run-AaMola` demonstrated why reset-only receipt clearing
   was insufficient:435/436 PASS, existing different-path successor-after-
   revocation test timed out while all new regression scenarios passed.
4. Add bounded metadata for the already existing reservation state. Do not
   change all Denied outcomes to Passive or turn process liveness into health.

### Consequence and budget check

Route: authenticated IBus callback -> existing observer/reducer -> existing
Get or native receipt plus marker -> SourceFree/Transfer grant -> local install
and exact ACK -> legacy/atomic key settlement -> existing bridge/verifier.
Only lifecycle transitions change. Candidate/lattice retention, model ranking,
false-authority thresholds, lexical package/cache identities, reload/delta
semantics and L1-L4 execution are unchanged. Reset still invalidates word
authority; password/sensitive-field policy and SafetyGate are not weakened.
No token may be manufactured from a remembered owner or a literal key alone.

Keep current acquisition and3,500us Space deadlines, no retry/polling, new
timer, RPC, worker, generation, cache or unbounded container. Existing bounded
state is reused; occasional lifecycle snapshots/clones do not add per-key
model work. Runtime latency/RSS improvement is not claimed without measurement.
Reset must discard pending feedback and transferred suppression as before;
fresh package loads cannot change which lifecycle receipt grants authority.

Primary second-order risks: accepting a stale field, replaying a reset tail,
using an old seal after word revocation, letting an old Set callback revoke a
successor, accepting an old property reply/marker/factory binding, and changing
lock order. Preserve pending->ready->reducer publication order and no-await
critical sections. Reuse or remove superseded paths instead of layering an
additional fallback. Any test-only access to existing private callback handlers
must not add a wire method or alternate implementation. No new Shift detector,
physical input injection, input-source migration or permanent installation.

Proof plan: scoped real-zbus/production-callback RED then GREEN for reflexive
focus without Disable (native and compatibility), Set/Reset during held
compatibility Get and during marker readiness, source-free recovery through a
real boundary and bridge, overlapping empty factories before/after binding,
late predecessor messages, and negative foreign/sensitive/malformed cases.
Use explicit focused discovery with nonzero exact test identities, then the
affected contracts, existing real IBus client and remote architecture refresh.
Full release gates and physical user confirmation remain separate obligations.
All execution is remote under the existing dedicated-20cpu resource/Cargo
guards. The installed bytes and unchanged desktop processes are the rollback
boundary; no user text from the private trace is copied into this repository.

Initial pure-transition RED (before production edits): remote
`run-OCKYOz/tests/SUMMARY.json`,428 selected/executed,426 PASS,2 FAIL. Failures:
`lifecycle_reflexive_refocus_does_not_require_engine_disable` and
`lifecycle_later_empty_factory_supersedes_before_or_after_target_binding`.
The strict-order negative passed. Discovery4.673s, execution5.686s, complete
remote action14.498s. SUMMARY SHA256
`006e8b6c53e5b4ff7a066368b6a88853fde0c281b11cdd9e12ff6456575d85bb`.
Command: `python3 scripts/dev-check.py check --target bin:lay-ibus-engine`.
Local provenance: `/home/ubu/.cache/lay/development/run-ctg9ducl/`.

Actual-client proof extension is scoped to an explicit additional lifecycle
scenario set, preserving the five original scenario bodies and cold/warm
schedules. It must exercise real type changes and refocus, report its own
denominator, and never replace the old cold restoration failure. No new
production options or artificial pauses are authorized by this test extension.

Held-Get production-callback RED: remote `run-CNJ15t/tests/SUMMARY.json`,431
selected/executed,428 PASS,3 FAIL. Exact suffixes:
`residual_held_compatibility_get_survives_early_key_as_empty_unknown_start`,
`residual_held_compatibility_get_survives_actual_reset_as_empty_unknown_start`,
`residual_held_compatibility_get_survives_actual_terminal_content_type_as_empty_unknown_start`.
All fail at the same assertion: the original request has disappeared after
word loss, before its held Get reply. Actual FocusIn/Reset/ContentType/key
handlers and the real zbus observer are used, not a simulated reducer.
The prior two pure-transition failures now pass with their repairs.
Remote action14.592s, SUMMARY SHA256
`40d3643ff7707d30479a842296c9327d5917eb7d492860403f352e33d587e755`;
local provenance `/home/ubu/.cache/lay/development/run-bp0njht5/`.
Same explicit focused command as above. Prior `run-ZJOG1m` stopped at formatting;
`run-ifMkj8` failed fixture setup before reaching the defect. Neither is a
causal RED receipt. Fixture setup was corrected by rebuilding the real open
word token before arming suppression; product expectations were not weakened.

Actual-client lifecycle baseline, same failing physical-candidate bytes:
`/home/e/projects/lay-development-runner/lifecycle-baseline-20260907-v1/receipt.json`.
Command: guarded remote `python3 scripts/proof/ime-client/run.py --remote-worker
--config /home/e/projects/lay-development-runner/ime-reset-final-rQnmd5/client-config.json
--output /home/e/projects/lay-development-runner/lifecycle-baseline-20260907-v1
--scenario-set lifecycle` (default immediate schedule).
Driver SHA256 `cd04de9791d26def51e61275fa40365f95c8ad5074525fb0828aba8241585def`;
its28 tooling tests passed before launch.0/3 lifecycle cases completed:
ordinary same-engine focus-out/refocus loses the previously admitted tail and
returns `passive:unknown-context`; no content-type or reset case was reached.
Candidate15728764 and copied IBus24338c0e identities match the earlier failed
physical candidate and installed IBus respectively. Elapsed1.579s including
artifact drain/cleanup; not production latency. Receipt SHA256
`30f37f874a626e9edcedb6ccb4d48624f73941ebd08819918dd5f11e8488a0bf`.
No private candidate remains; desktop was not touched. The initial description
of this as causal client RED is withdrawn: the optional witness incorrectly
expected word preservation across IBus's real intermediate fake context.
This receipt records a harness-oracle failure, not a demonstrated runtime
regression. Neither the content-type nor Reset case was reached.

Client-oracle correction (second and final bounded review pass): IBus
1.5.34-rc2 `bus/ibusimpl.c`, `bus_ibus_impl_set_focused_context`, substitutes
`ibus->fake_context` when focus becomes NULL with a global engine. The exact
upstream source at
`https://raw.githubusercontent.com/ibus/ibus/1.5.34-rc2/bus/ibusimpl.c`
and both private traces show A -> fake context -> A, not exact-context refocus.
The optimized candidate706b admits and settles keys39/40 with a fresh owner2;
its `passive:unknown-context` empty tail is the correct safe word discard.
Its first receipt is also an oracle failure, not lifecycle product FAIL.

Before editing that optional witness: assert unchanged client-visible text,
same engine and returned canonical client context, discarded word/receipt,
then exactly one real Space boundary and one admitted printable without retry.
Apply that boundary contract to all three real-refocus cases. The controlled
same-ContextKey no-Disable and held-Get source tests remain unchanged and are
the causal preservation proof. No runtime, deadline, wait, private-bus
configuration, original five restoration scenarios or production authority
changes. The narrower assertion prevents a false cross-field retention claim;
it does not demonstrate physical desktop recovery. Run old and new candidates
identically and label a common PASS only as lifecycle smoke coverage.
Independent reviewer: current oracle6/10, High0/Medium1; proposed corrected
witness9/10, High0/Medium0. No new runtime repair was requested or made.

### Integrated source acceptance

Final focused `run-c55BY8/tests/SUMMARY.json`:436 selected/executed,436 PASS,
0 failures. Discovery4.700s, execution6.066s, remote action14.879s. SHA256
`05c562fc86511ec45f0743131290f07d3f311d34f7c3551613132d8835778a8c`;
local `/home/ubu/.cache/lay/development/run-e63w6apo/`.
Includes held Get, reply-before-marker, early-key then Set, actual plain
FocusIn without Disable, predecessor binding/focus before and after successor
installation, malformed/unrelated typed Set, and prior post-revocation recovery.

Intermediate `run-Y7cnPQ` was a compile refusal (temporary borrowed Value;
fixed by binding the message body). `run-QErFaT` was stopped at an old unbounded
fixture waiting for a rejected marker; no PASS is claimed. Its delayed-Reset
fixture now has a3s bound and retains the original different-target recovery
assertions. The overbroad owner-path guard was replaced with current consumed
receipt provenance; canonical revocation retires that provenance, without
another controller or history. `run-AaMola` measured435/436 before that final
revocation cleanup; the unchanged old failing scenario is green in run-c55BY8.

Fresh-context independent review (`lifecycle_repair_review`, GPT-5.6 Sol/high):
9/10, High0, Medium0 on final frozen source. One bounded review/repair cycle;
The second bounded pass below addresses only the client-oracle correction.
The reviewer did not run tests independently.
The review covers source and witness design, not desktop or release acceptance.

Canonical discovery was regenerated remotely:2697 total identities,20 added
IME correctness tests relative to the previous2677;0 removed or reclassified.
Manifest SHA256 `44464aa603cabdb6dfda94c418a754ae0418567d483f811c35a81061fce7d372`.
The zero-known-failure contract was rebound to this exact manifest; its empty
exception list and historical observation remain unchanged. This grants no
new failure allowance. Affected/full checks and optimized actual-client results
were pending at this historical checkpoint; completed results follow.
No production action occurred.

### Completed affected checks and optimized client, 22:01 +03:00

`python3 scripts/dev-check.py check`:2671 selected/executed,2671 PASS
(2635 correctness +36 package); registry2697 =2671 selected +15 ignored
+11 performance exclusions. Tooling99 PASS with one explicit real-cgroup
integration skip. Remote `/home/e/projects/lay-development-runner/run-it9K5U/`,
local `/home/ubu/.cache/lay/development/run-hwu9s4v3/`. Tests SUMMARY SHA256
`cc3bcd1e3c399fd60d04cc63f287b5913d7e3c7ca775b184ffbc56e718e2f23e`;
source archive SHA256
`39e4fd178804dc49e6b346386e4e5a9522570df70d27c07df65a5b85fa249c4e`.
Discovery compilation23.640s; execution348.789s; remote action377.484s.
Daemon239 tests took141.732s; IME436 took6.077s. Targets are sequential and
test-threads1: build jobs20/CPU2000% does not mean every test uses20 CPUs.
This is affected correctness/package acceptance, not a new full release gate.

Remote `scripts/update-architecture-graph.sh` PASS43.49s; log SHA256
`90f2833dee36b30213ad787c7273632d5d17dce21733978b2c9579393e331fc1`.
Initial optimized build in the same resource guard:
`scripts/cargo-guard.sh build --release --locked --bin lay-ibus-engine`.
Cargo59.02s, timed command59.17s; no profile overrides. Root for this build,
architecture.log, build.log, client-config.json and all client receipts below:
`/home/e/projects/lay-development-runner/lifecycle-final-vwafpP/`.
Candidate `lay-ibus-engine` SHA256
`706b4d819e481feb0ccb4dab49751390745d7fe25c8683a8a0a6fac4471ae333`.
Nine-role dependency manifest remains
`/home/e/projects/lay-development-runner/boundary-deps-UjRxy5/dependency-manifest.json`,
SHA256 `ea6c07b1ddd0d504f87578a39a40b552f301c618195ab0e84df9d9acd2f21c76`.
Copied IBus24338c0e matches the installed IBus; no system-daemon substitute.

Corrected driver SHA256
`993ab7fc460281de1931ac76a2771daec1070f321d9f1551832392156fa2d26d`;
original five-case `run_cases` SHA256 remains
`eea5c141f6a8a10f6f9d6823fe7a526607c9a0f0e5db34dee4da9ad2ae1f98ab`.
28/28 harness tests PASS. An intermediate tooling assertion compared
`ast.unparse` formatting and failed on inserted parentheses; the assertion now
compares expression ASTs. Neither runtime nor client expectations changed in
that repair. Optional client runs use the existing guarded worker, exact
config, a fresh output, and `--scenario-set lifecycle`; restoration leaves
that switch at its default. The original52s deadline and cleanup remain.

| Output subdirectory / receipt.json | Candidate | Result | Receipt SHA256 |
| --- | --- | --- | --- |
| client-lifecycle-old-corrected |15728764|3/3 PASS|`eb3b7a388547846de3838c378efe2476d84569d4e68a0317faee7008b9b1b791`|
| client-lifecycle-new-corrected |706b4d81|3/3 PASS|`01999583b30bd74336e3c1bcdd88a4dc2eaceaddb3a33b37b09f2d42b9101bfe`|
| client-restoration-immediate |706b4d81|0/5 completed, first case FAIL|`d7a9d9f95ee43beb1ab49614a075bab024b0b89f286f5f5aa70df6782ae971a2`|
| client-restoration-post-exact-ready |706b4d81|5/5 PASS|`35e1d527314048aaa2410f5b756e46ad0e17ac33315074b218c52239d8cfb939`|

Systemd durations: old lifecycle1.597s, new lifecycle1.595s, immediate1.630s,
post-exact-ready2.618s. These include fixture/cleanup, not production latency.
Post-ready candidate warmup91,630us; diagnostic-file observation995,238us
includes trace flush delay. Immediate warmup97,379us was available eventually,
but the first correction emitted no DeleteSurroundingText and visible text
remained unchanged. The separate cold defect is retained; warm PASS cannot
replace it. Lifecycle smoke proves dummy-context liveness and safe word loss,
not exact-ContextKey preservation, the cached desktop callback route, physical
Double Shift, or general Wave quality. All private candidates were reaped.

The current runtime repair is source-verified; production authority has not
changed in this step. Installed95348e4a and its physical failure remain the
live baseline. New installation/restart/rollback still needs separate user
authority and keyboard-safe staging. TD-121/release acceptance is not DONE;
no new commit/push is claimed. Proof-only harness/document edits after this
build require the final architecture refresh; runtime Rust remains frozen.

Second-pass closure: reviewer accepted the corrected witness but found one
Medium in its tooling guard (8/10): missing explicit three-case success
denominator and remaining formatting-sensitive AST checks. The bounded repair
adds an exact ordered case-identity assertion before client PASS and structural
checks for each Space->printable sequence. It does not change the scenario's
input, waits, production binary or any restoration assertion. Recheck the small
tooling/client lanes only, retaining the cold failure.

Final closure,22:09 +03:00:29/29 remote tooling PASS; exact denominator guard
rejects missing/extra/reordered/duplicate identities. Final driver SHA256
`6546a3cd65368d694293a481082ee912060b15dc1219d8006ed9dc2f169c234e`.
Finding-only reviewer confirmation:9/10, High0, Medium0; static read-only,
no independent test execution. No third broad architecture audit was opened.
Both candidates again passed3/3 with this exact driver. Final artifact root:
`/home/e/projects/lay-development-runner/lifecycle-delivery-JCfGbv/`.

- `client-lifecycle-old-final/receipt.json`, old15728764,1.667s, SHA256
  `a37796895c213569c4bdee36d8b53a342aff8abc9a234e40e201fb624c21dc3e`.
- `client-lifecycle-new-final/receipt.json`, new706b4d81,1.656s, SHA256
  `3234c6a5c2bc8f5a212b878034c97f6c23f9fa09607d289487e76854b22fabb9`.
- `architecture-final.log` PASS, SHA256
  `e1d6b9b0b868553eb3d225abd6f68c8f6e21862797cffb4508ab9c36d82c7c24`;
  embedded/source architecture receipt SHA256
  `560c8d19146151d6cd1e0242749bca66f29c9f524393479c46f4a7d7a634740f`.
- `build-final.log`: same guarded release command, Cargo0.09s, total0.24s.
  Final binary remains byte-identical706b4d81; this was cache verification,
  not another59s compilation. Existing frozen runtime,436/436,2671/2671 and
  restoration evidence are therefore retained, not blindly reexecuted.
  The only new client branch is the lifecycle success-denominator assertion;
  it cannot run in the unchanged default restoration scenario set.

Last read-only production recheck: IBus4715 (Sep2), lay-daemon272240
(Sep7 14:20:59), installed IME2249560 (20:30:30) all unchanged; installed
SHA25695348e4a unchanged. InputState still fails with
`metadata observer cancelled`. All new private candidates cleaned up.
No local builds/tests, global IBus restart, keyboard-daemon restart, input
source change, physical key injection, install, rollback, commit or push.
Next authority-bearing step is a separately approved keyboard-safe temporary
IME preview; neither that preview nor cold-start remediation is silently
authorized by these source/test results. TD-121 remains IN_PROGRESS.

## Authorized lifecycle preview on the existing desktop IBus

The user responded to the explicit temporary-preview permission request with
«блять не останавливайся больше!!!!». Proceed with that preview, not a silent
global IBus/keyboard-daemon restart or permanent installation. The subsequent
question whether the old daemons are the cause is diagnostic, not restart
authority. Specific stale capability-cache contents remain unmeasured.

Pre-action live state: IBus4715, keyboard daemon272240, failed installed
IME2249560/SHA95348e4a, selected lay-ime-ru, Focused=true and InputState error
`metadata observer cancelled`. Sources remain exactly lay-ime-us/lay-ime-ru.
Candidate706b4d81 is the byte-identical remote release artifact accepted above.
Local preview root: `/home/ubu/.cache/lay/development/ime-lifecycle-hot-6LhiQT/`.
Reuse the previously used start-hot.sh procedure with only this cache root
changed: verified native xkb:ru::rus before stopping exact failed IME2249560;
temporary hash-named unit; exact process hash and Ping; select Lay only after
startup; InputState required when focused; native fallback before stopping the
candidate on failure. Installed files, source lists and both protected PIDs
remain untouched. No synthetic physical keys or private client in the desktop.
Startup liveness is not physical acceptance; actual user input is still needed.

Preview executed22:18:58 +03:00: exact failed IME2249560 stopped only after
native engine readback. New `lay-ime-hot-706b4d81.service`, PID2841233, runs the
verified706b4d81 from this cache; installed95348e4a was not overwritten.
`start-hot.log` records startup/no-focus; later Focused=true and
InputState=`passive:daemon-word-buffer`. IBus4715 and daemon272240/source list
are unchanged. Staging/launch took0.644s; no local compilation or key injection.

First live retained snapshot: plain compatibility FocusIn1, FocusInId0,
CreateEngine1;242/242 legacy callback admissions accepted and242/242
settlements accepted,0 denied. Four visible preedit updates were recorded.
Warmup available in142,677us. Space outcomes remain separately counted:
2 prefetch_not_ready,3 infrastructure/full_no_apply,11 rank/full_no_apply;
no successful replacement is proven by this snapshot. User text remains only
in the private cache trace, never copied into this journal. This proves that
the new IME can acquire and process the old IBus compatibility route without
restarting that daemon; it does not prove repeated refocus/Reset, all user
functions, absence of any cache issue, or physical acceptance. Requested
manual wrong-layout/Tab/Double-Shift confirmation is still pending.

The read-only cold-start follow-up identifies a different first loss. In the
private fixture's immediate trace, row113 exact preparation has no authority
snapshot/certificate/decision; rows129-131 retire the request as NotReady
(lookup3,575us), rows133-134 commit only the ordinary Space, and row142
records warmup completion97,379us. The post-ready control has exact proof
before Space and applies with3us lookup wait. Source confirms background
warmup after factory publication (`server.rs:98-113`), unchanged3,500us budget
(`committed_tail.rs:36-46`), pending-slot retirement
(`space_autocorrect_prefetch.rs:348-368`) and safe uncorrected fallback.
Not every worker phase has a timestamp, so an internal phase-cost attribution
is still unknown. No cold-start runtime change was made. In particular, moving
blocking lexical initialization before factory registration is not an admitted
quick fix: the current ordering explicitly protects keyboard availability at
login. It needs a separate bounded consequence check, not a larger Space wait.

Architecture after preview documentation: remote `architecture-preview.log`
under lifecycle-delivery-JCfGbv, PASS; SHA256
`1193b467402072c75cbe769a6b79f84b72d8cc7bb758f4dbd6fc9f2e6bb66ea9`.
Last live readback,about8.5min after launch: same2841233, InputState healthy
`passive:daemon-word-buffer`, still242/242 accepted callbacks (no new physical
input since the first snapshot). This is continued liveness, not additional
functional test coverage. User confirmation remains the next acceptance input.

## Sep8 physical rejection and opt-in diagnostic repair (superseded)

At01:09 +03:00 the user reported «не работает!». IME2841233 was still the exact
706b4d81 candidate, but InputState again failed with `metadata observer cancelled`.
The trace had reached exactly512,000 bytes. At01:11 the exact candidate was
stopped only after selecting/readback xkb:ru::rus; IBus4715/daemon272240 remain.
Private snapshot `ime-lifecycle-hot-6LhiQT/failed-physical-20260908-0111.jsonl`
under the local development cache has SHA256
`93799b6acab48298e98758165cb103ab31a4652fb27c46596c461a51de5e2958`.
Its surviving metadata contains669 refused key admissions and no lifecycle
event: normal500KiB tail compaction displaced the first failure. No exact new
reducer cause is claimed from that truncated evidence. The earlier242/242
accepted startup snapshot cannot establish long-lived physical functionality.

Subsequent installed-process restart was confirmed by the user: «это я
делал». The later observed daemon3757261 and installed IME3757358 (started
01:18:55 +03:00) are therefore an intentional user restart, not evidence of
another unexplained failure. IBus4715 was unchanged at that observation.
Resolve current PID/executable identities again before any preview; the
earlier daemon272240/native-XKB withdrawal snapshot is historical.

Read-only refresh at01:30:59 +03:00: IBus4715, daemon3757261, IME3757358;
selected engine lay-ime-ru; InputState returned `metadata observer cancelled`.
The process executable and installed file both hash to
`95348e4a81056a9a476c9aade72194978a4c359abe81ac6983a381989423a668`.
Unlike the overwritten706b4d81 preview tail, this installed-process log still
contained CreateEngine serial98: `factory_acquisition_state=transfer_revoked`
with owner5/activation5, followed by `observer_ingress_refused=denied`.
This localizes that installed-process refusal, not the cause of the separate
706b4d81 physical failure. Preserve all reducer acquisition statuses in the
optional metadata snapshot so a precedence-based summary cannot conceal them.
Private saved log:
`/home/ubu/.cache/lay/development/ime-installed-failure-xiKj6N/ibus-engine-trace.jsonl`,
SHA256 `2f3907aa3eef2a2a48699413372aed0474b0a28ac3c04d882e3d498328571cf2`.
No service action accompanied this refresh.

User authorization: «дебаг режим внедри!» and «потом отключим его или сделай
опцией!». Scope is optional metadata diagnostics, not another speculative
lifecycle repair, global restart, cold-start fix, SafetyGate change or release
acceptance. Default OFF; `LAY_IME_DEBUG=1` enables it per process. An optional
`LAY_IME_DEBUG_PATH` selects the report path; documented default is private.

### Designs and decision before production edits

- Increase the existing verbose rotating log:4/10. It can still lose the
  first failure under sustained traffic and retains sensitive key/text fields.
- Bounded first-failure recorder integrated with existing IME trace hooks:
  9/10, selected.64 recent metadata events, aggregate key counters, first
  terminal reason/callsite and pre-failure reducer/fence identities. Freeze on
  the first observer failure; use existing private atomic file primitives.
- Always-on general tracing framework/service:2/10. Excess scope, dependency,
  privacy and runtime cost for this concrete failure.

### Consequence/budget check

Diagnostic route: existing authenticated observer/callback -> metadata-only
trace hook -> bounded optional recorder -> one first-failure snapshot. It
cannot mint, retain, rank or revoke runtime authority; no model/lattice,
package/cache identity, learning/feedback, verifier, edit-plan, source selection,
deadline or fallback behavior changes. Existing denied/cancelled results and
side effects must be identical with the flag off/on. Do not restart or retry
the observer to make the debug check pass.

Default-off work must be only a cached option check with no recorder allocation
or file IO. Enabled steady-state work is bounded metadata/counters; no key
codes, decoded characters, input/preedit text, replacement text, raw D-Bus body,
arbitrary bus error text, screenshots or application/window titles. Use closed
event/reason vocabulary and numeric serial/owner/request/fence/generation IDs.
Timestamp/sequence and unavailable/dropped metadata must be explicit. Avoid
flooding the64-event window with every successful key; keep key counters.

Snapshot state reads use nonblocking/bounded access, never change lock order or
hold reducer/fence locks while serializing/writing. Ring state is diagnostics,
not another admission controller or source of truth. Freeze first failure
before cleanup/Drop can replace it with generic Cancelled. A single bounded
file replaces the previous process report (PID/start identity differentiates
runs); repeated errors within a process cannot overwrite its first failure.

The existing async verbose writer is unsuitable for the pinned failure alone:
it is gated by debug_action_log and silently drops a full channel. Reuse
private atomic file writing, allowing one short-lived writer only on first
fatal failure, with no persistent worker, timer, queue, polling or synchronous
filesystem wait in the key/observer callback. File/serialization/thread errors
must not change runtime results; expose a concise stderr failure diagnostic
when persistence fails. Bound serialized output and use mode0600. Do not turn
on the legacy verbose/text logger merely to make the new option work.

Regression obligations: flag unset/0 emits nothing; flag1 works independently
of legacy debug_action_log; fixed ring/counters and first failure survive
post-failure floods; JSON escaping/privacy and size/0600; inaccessible output
does not affect authority; real observer error/cancellation/Drop preserves the
first cause/callsite and pending identities. Existing focused IME contracts
must remain green. All execution remains remote/guarded; one connected code
owner, fresh-context review >=8/10 with no High/Medium, maximum two repair
passes. The known failed candidate/native input and unchanged installed bytes
are the rollback boundary; no production success or DONE claim is admitted.

## Sep8 minimal adapter refusal repair — decision before edits

User direction: read all2878 lines, stop overcomplicating, and make adapter.rs
reliable. The entire current file was read; SHA256
`ce11770a043747ba96651964b673b28a2b5680eb5d549e039ff0648a1884f69f`.
Its unfinished diagnostic delta adds352 lines over the2526-line pre-debug
snapshot, plus749 lines in trace/ime_debug.rs. Preserve that experiment outside
the source build and restore only those owned diagnostic edits, retaining all
prior lifecycle fixes. No blanket checkout/reset and no installed-file change.

Installed95348e4a matches the archived release artifact. The final source
snapshot run-stLTUo matches HEAD for both reducer and adapter: SHA256
291452d0ba96c2ffa06f81a05f0e6858bbe139dcf5607a5eb8007ee3a386673a and
0be41e1ae96f34799f57c682f7e3adccfaec15f1f61a22be4e1c6f825fb8ab02 respectively.
That reducer rejects an existing Revoked ticket as not Consumed; the retained
CreateEngine98 trace records transfer_revoked before the rejection. The adapter
converts a reducer refusal to a fatal observer error. Newer source already
retires one later revoked-transfer case, but the adapter still conflates any
factory admission refusal with loss of its metadata transport. The exact cause
of the separate706b physical failure remains unproved.

Options: clearing cancellation/restarting the observer after arbitrary Denied:
2/10 (discards ordering/provenance); distinguish factory refusal using existing
non-authorizing dispositions:9/10 (selected); new dispatcher/module split/general
tracing framework:3/10 (unnecessary scope). No file-length quota drives this.

Selected boundary: a correctly parsed and authenticated CreateEngine for which
the existing reducer declines a ticket remains a refused factory callback, but
does not terminate observation. Reuse the existing Passive disposition if its
consumer contract remains non-authorizing. Do not catch all Denied errors or
weaken sender validation, malformed-body checks, stamp publication, ordered
marker checks, disconnect handling, SafetyGate or verifier. Do not separately
change key overflow/duplicate handling without a discriminating proof: it can
indicate lost ordering, unlike a declined factory transition. No new controller,
state owner, timer, retry, queue, fallback or broad reducer rewrite is authorized.

Consequences: candidate/lattice/ranking, packages, caches, learning/feedback,
models and text-edit validation are unchanged. A refused event grants no owner,
transfer, text tail or admission token; only the existing fresh valid lifecycle
can recover. Stream availability changes, not authority. Maintain current lock
order, callback receive-order witnesses and bounded stores; no extra RPC, sleep,
disk IO or steady-state allocation. Existing5ms acquisition and3500us Space
deadlines remain unchanged. No epoch reset or acceptance of stale results.
Future package updates cannot alter this metadata-only distinction. Risk is
accidentally promoting Passive callbacks or masking real transport loss; tests
must assert refusal, stale-token rejection and subsequent valid recovery, not
just that the process remains alive. Maintenance cost is a small branch with a
why-comment and focused regressions in the existing harness.

Proof: controlled real production observer CreateEngine refusal is RED on the
pre-change adapter, then remains refused without cancellation after repair;
later valid factory/focus/key route recovers. Malformed body/wrong sender and
actual terminal transport paths remain fail-closed. Run the existing focused
IME target remotely with exact nonzero identities; full affected/release gates
remain separate. Fresh-context review>=8/10,H0/M0, at most two repair passes.
No new install, service restart, global IBus change, release/DONE or physical
acceptance is implied by source GREEN. Rollback is the saved pre-debug source
and unchanged installed95348e4a, with failed physical evidence retained.

Shelving completed with apply_patch; adapter and trace exactly match the
pre-debug436-test snapshot. Recoverable full copies of the three experimental
files: `/home/ubu/.cache/lay/development/ime-debug-shelved-gzw9VY/`.
No prior lifecycle source edit was reverted. The experiment was never built,
installed or used for a PASS claim. Existing diagnostics are documented in
DEVELOPMENT.md; LAY_IME_DEBUG is not a supported current-source option.

### Causal RED before the factory branch change

`python3 scripts/dev-check.py check --target bin:lay-ibus-engine` ran remotely
under the unchanged dedicated-20cpu/resource guards. Remote
`/home/e/projects/lay-development-runner/run-zxtrcc/tests/SUMMARY.json`, local
`/home/ubu/.cache/lay/development/run-id2ww8vm/`.
Selected/executed437:436 PASS and exactly one expected FAIL, the new
`context_admission::adapter::tests::word_scope::residuals::residual_declined_authenticated_factory_is_passive_and_later_factory_recovers`.
At residuals.rs:781 the production observer returned Denied on a well-formed
authenticated factory request overlapping the first pending ticket:
`a declined factory transition is not observer transport loss: Denied`.
This is controlled production-path RED for the remaining refusal-to-fatal
conversion, not a reconstruction of the unobserved706b desktop failure.
Snapshot SHA256346a42b74ee3065f550cdf26d9d598bf35a01124edc3cd2b23418922756e33d7.
Remote action14.850s; discovery4.770s; test execution6.004s. The canonical
manifest reports the one new identity without rewriting it. No runtime action.

### Focused GREEN after the minimal change

The only production delta relative to the pre-debug snapshot replaces one
`ticket.ok_or(Denied)?` with a let-else returning existing Passive, plus a
two-line why-comment. No other fatal path, reducer or deadline changed.
Adapter SHA256b1684bfcde1a2bd0c85593b74a47b7cabe3f980bcfa2a5488e8969bdf681f3ab;
regression file SHA2567f10567481cda2c7f489f59d9859503070fa7ad74c7d74a4cfb5c5f3dce60656.
The RED test assertions were unchanged for GREEN.

Same explicit remote focused command:437 selected/executed/PASS,0 failures;
3 performance tests excluded. Remote
`/home/e/projects/lay-development-runner/run-qV3Dqa/tests/SUMMARY.json`, local
`/home/ubu/.cache/lay/development/run-h6az0mbb/`.
GREEN SUMMARY SHA25608bda221349d2ffcb9327ce7fe64387181e4a81988c771c5e2979e5dad220cc9;
RED SUMMARY SHA2562bc5c6833583763e5fc376c06a482deaea4c9b9d4ab284f175a3b2819ce40461.
GREEN source archive d2625e45f48d20a39840e8d18b091276722ba361f03f980c389db09db8dfe661.
Remote action14.567s; discovery4.682s; tests5.761s. This proves scoped source
refusal/non-authority/continuation, not physical typing or release acceptance.
Independent review and remote architecture refresh were pending at this
checkpoint; their completed results follow. Installed95348e4a was unchanged.

### Independent review and exact optimized candidate, Sep8

Fresh-context reviewer `adapter_refusal_review` returned PASS9/10, High0,
Medium0; no repair pass was required. The review checked the frozen production
branch, unchanged RED/GREEN regression assertions, factory callback consumer,
new-engine initial state, key admission and transport termination. Passive
cannot bind the refused factory target or grant its callback an owner/token.
The object server can still construct an engine object; construction alone is
not context admission. The regression exercises the real p2p observer and key
handler, but does not directly execute object-server registration. That thin
consumer was checked statically. Physical input and release acceptance remain
outside this review verdict.

Remote architecture refresh PASS and optimized build PASS. Artifact root:
`/home/e/projects/lay-development-runner/adapter-refusal-yYeGQt/`.
Build command: `scripts/cargo-guard.sh build --release --locked --bin lay-ibus-engine`,
inside the unchanged dedicated-20cpu resource scope. Cargo reported59.06s.
The frozen `lay-ibus-engine` SHA256 is
`e596b50733819861a4620949b8003ee198b1903421ee37c7a6c12264575d6fae`.
`architecture.log` SHA256
`1d18004c082c161dab7aad6b3dd9c7e4fc53f6e8a83c310f86fce120d2f8f899`;
`build.log` SHA256
`b6ec9b258ea6d78d915b9d6406b69ff47e0b6f399b00b2a0ce27d4b8128b3a97`.
This artifact is preserved separately from Cargo target output and must not be
silently replaced by a later all-binaries build.

User subsequently authorized continuation through remote checks and controlled
desktop verification with rollback. No global IBus/keyboard-daemon restart,
input-source migration or synthetic physical key injection is included. Before
permanent installation, rerun required affected/release checks and actual-client
scenarios for these bytes, then obtain physical-input confirmation. Earlier
706b cold-start failure and unexplained physical rejection remain historical
failures, not accepted or relabelled by this source review. No install, release,
DONE, commit or push is claimed at this checkpoint.

### Broad-check finding and bounded oracle repair, Sep8

The canonical registry now includes the new declined-factory regression:
2698 identities =2636 correctness +36 package +15 ignored +11 performance.
Manifest SHA256342c0808843d366e8a6f462e3aa923ea18d5236047a1f20d7b923f1ffcd337f3.
The existing zero-failure contract was rebound to these exact bytes; its empty
exception list and historical observation are unchanged.

Remote `scripts/check-lay-changed.sh` executed2672 selected tests:2671 PASS,
one failure in the existing
`residual_later_factory_retires_revoked_transfer_and_recovers_source_free`.
The final verdict is BLOCKED_CONTRACT, not PASS. Saved output:
`/home/e/projects/lay-development-runner/adapter-prod-mOnOK5/affected-failed/`;
wrapper log `changed-device-fixed.log` in the same parent. IME failed at the
fresh pending-fence unwrap, residuals.rs:1047. The new factory-decline test
passed. An earlier wrapper attempt (`changed.log`) could not open /dev/null
inside an outer cache-mount namespace and selected no tests; its exit0 is NOT
accepted evidence. Further commands use the existing canonical cache via the
test runner's path resolution, without that extra namespace.

Read-only consequence analysis before the test-only repair: native FocusInId
returns after detaching `finish_native_activation`; creating reducer.request
does not guarantee `arm_fence` has run. The old half of this fixture receives
its outbound marker before inspecting pending, but the fresh half does not.
Production ordering is arm_fence before emit_signal. Selected design9/10:
receive and validate the fresh outbound marker, retain it without forwarding,
then keep all old-timer/old-marker/no-authority assertions unchanged and forward
the retained fresh nonce at the original completion point. A sleep or larger
budget is3/10; dropping the pending assertion is1/10; both are rejected.

This adds an actual transport witness using the existing bounded harness, not
a synthetic implementation or another runtime controller. No production code,
candidate/ranking/learning/package/cache behavior, owner/token rule, timeout,
keyboard output, or installed artifact changes. Missing marker, wrong identity,
None after the marker, or stale-event effects still fail. The observed broad
failure is retained; a new bounded focused run and fresh review must determine
whether the stronger witness resolves the fixture race or exposes a real
background publication failure. No retries-until-green are permitted.

Oracle repair completed in one pass. Residuals SHA256
`28dd8b76614de23e1fa1e42fc8a01284277bb4a924c10b0d4a354754edcba89b`.
Focused remote437/437 PASS,0 failures,14.757s; receipt
`/home/e/projects/lay-development-runner/run-Iq4qgM/tests/SUMMARY.json`,
SHA256c6369d9374b093f7e51e40c9bfc613d76040885b3e0e33ea764c4985f0f956f2.
Archive SHA256c13346c92a6dcbd0b306013d16fe79b9e14c81b5c7dca603b2cb7c22d64094fb.
Fresh oracle review PASS10/10,H0/M0, unchanged production code and budgets.
The broad failure remains historical; it is not rewritten into a full PASS.

Exact e596 candidate actual-client results under adapter-prod-mOnOK5:
`client-lifecycle/receipt.json`:3/3 PASS, SHA256
c37cdb9485d84985f15fde4e11ead5e375768a91ce2101522c483dcab346662b;
`client-post-ready/receipt.json`:5/5 PASS, SHA256
53cfd6bf79e3f88b5f4e7f9d023e5400f469e3b43cb6592f95457cf3ed1110df;
`client-immediate/receipt.json`:0/5 completed, first case FAIL, SHA256
50fe9e83c71988298ac2b3a6d4fb399805efd7692dfc1c2a9047fa9233ddcb56.
All private processes were reaped. Warm startup does not replace cold failure.

### Final lint finding: unused ready-state field

Remote `scripts/check-lay-lints.sh` refused one added dead-code diagnostic:
`ReadyActivationState.request` is never read. Log:
`/home/e/projects/lay-development-runner/adapter-prod-mOnOK5/lints.log`.
The earlier lifecycle repair made this ready-state copy redundant in production.
No known-warning allowance is added. The initial read-only search missed two
reads inside the test-only `take_ready_activation` helper; the rejected attempt
and corrected scope are recorded below.

Consequence check before production edit: remove the private field and its
assignment (9/10); retaining it with a warning allowance (2/10) or adding a
dummy read (1/10) preserves unused state without a consumer. Selected removal
does not change request validation, owner/outcome/fence identity, lock order,
timers, cancellation, candidate/ranking/SafetyGate/verifier, packages/reloads,
cache invalidation, learning/feedback or IME/daemon wire compatibility. There
is no serialized/public ABI for this struct. Storage can only shrink; no new
allocation, RPC, task, queue or fallback is introduced. Rollback is the two
removed lines; the prior e596 executable remains preserved. Because source
bytes change, final optimized build/client identity must be recorded anew.
Focused assertions and both existing lint feature scopes remain mandatory.

The initial two-line removal is REJECTED, never installed: focused discovery
failed to compile the test binary (E0609 at adapter.rs:1752). Independent review
also found the two remaining test-only identity reads:4/10,H1/M0. The exact run
is `/home/e/projects/lay-development-runner/run-WhMvQY/`; no test executed there.
Corrected minimal design before repair: keep the request identity under
`#[cfg(test)]` at both its declaration and constructor assignment. Its only
consumer already has that same condition. This removes the unused production
copy while preserving every test-side snapshot/consume identity check; no
assertion or production ownership rule changes. This is preferable to changing
the test identity comparison merely to fit a two-line deletion. The previous
storage/ABI/latency/authority consequence bounds still apply to production.
One finding-only review confirmation and fresh compilation/tests are required.

Final source adapter SHA256
`60d50d526d965efeb6a7f77bf69b0436754586f8d93c8a0aa301a5ac6f57d586`.
The field and its initializer now share `#[cfg(test)]`, with a why-comment;
the test-side identity comparisons are unchanged. Finding-only independent
confirmation PASS10/10,H0/M0. Focused remote437/437 PASS after this correction,
14.888s, `/home/e/projects/lay-development-runner/run-mCkKUs/tests/SUMMARY.json`;
local `/home/ubu/.cache/lay/development/run-egvvscyu/RESULT.json`.
Snapshot SHA2560338dbefd4b09e4b286adf8f529f3353203f50d90d08dfc4f05b1918d90c8075.
The final full release gate was run on that source after a fresh remote
architecture update. Its non-PASS outcome is recorded below; it is no longer running.

Resource clarification requested by the user: `CARGO_BUILD_JOBS=20` controls
compilation, not test execution. `run_non_timing` currently iterates targets
sequentially and the guard requires `RUST_TEST_THREADS=1`. The239 daemon tests
in the previous broad run took141.659s at approximately one core. This is a
test-runner throughput limit, not evidence that the remote machine lacks cores.
Parallel isolated target groups would need a separate bounded tooling change;
no20-core test-execution or development-speedup claim is made for this run.

### User stopped tests; fresh production IME installed, Sep8 03:05:52 +03:00

The full correctness/package lane completed2672/2672 with zero failures in
373.094s. The enclosing full-release script then exited1 during default clippy
inventory (`Cargo build-finished reported failure`). It did not reach its
all-binaries release build and is NOT a full-release PASS. Exact retained log:
`/home/e/projects/lay-development-runner/adapter-prod-mOnOK5/full-final.log`.
The user explicitly rejected further tests and requested production delivery,
then specifically required a fresh build of the latest working source.

Ready e596 was briefly installed at03:02 with a backup and native-input staging.
It was superseded at03:05:52 by a newly compiled optimized IME from adapter
60d50d52/residuals28dd8b76 and matching current runtime files. Only build ran:
`scripts/lay-resource-guard.sh -- scripts/cargo-guard.sh build --release --locked
--bin lay-ibus-engine -j20`, remote dedicated-20cpu scope,58.72s, four warnings.
No new test, lint, client-proof or broad check was launched after cancellation.

New artifact/installed/process SHA256:
`12db8c00ebd23d1dbc1603dc1fbba337ddfed0c0aa308258b669df8fbc8c95dc`.
Remote artifact and log under `adapter-prod-mOnOK5/`:
`lay-ibus-engine-latest`, `build-latest-prod.log`.
Permanent path `/home/ubu/.local/lib/lay/bin/lay-ibus-engine`; process147597,
unit `lay-ime-release-12db8c00.service`; selected engine `lay-ime-ru`.
Native `xkb:ru::rus` readback preceded both IME stops. Atomic replacement
preserved backups; IBus4715/start2261 and daemon3757261/start44087079 unchanged,
as was the user's input-source list. Ping responds, Focused=true, InputState
`passive:daemon-word-buffer`, not the previous observer-cancelled error.

This proves fresh build, permanent installation and basic interface readback,
not physical autocorrection/IME/Double-Shift acceptance or cold-first-word
restoration. No new commit/push or TD-121 DONE. See [CONTINUE.md](../CONTINUE.md)
for exact backup paths, current user constraints and the next scoped action.
