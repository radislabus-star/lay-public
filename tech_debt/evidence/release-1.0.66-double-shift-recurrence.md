# TD-121: repeated Double Shift loses the committed tail, 2026-09-08

Status: `INSTALLED / PHYSICAL_CONFIRMATION_PENDING / COLD_START_OPEN`. The user explicitly reauthorized tests
and reported that Double Shift works once, then cannot convert back.

## Observed failure and scope

Production IME147597 is the installed12db8c00 candidate. Private trace snapshot:
`/home/ubu/.cache/lay/development/double-shift-frjcffof/ibus_engine_debug.jsonl`.
Do not commit that trace: it contains user text. The metadata sequence is one
terminal `ManualToggleV3` replacement, an admitted factory transition, source
FocusOut/Disable, then a target with no owner and an empty tail. Further Shift
callbacks are refused. `no_live_acquisition` is the factory's pre-event state,
not a factory refusal. The global IBus and keyboard daemon remain unchanged.

Code identifies an untested boundary: bridge output advances the shared tail
epoch through `publish_tail_handoff`, whereas only key-callback settlement
updates the context reducer. `seal_source` then correctly refuses unequal
epochs. The existing same-instance atomic roundtrip test cannot exercise this.
The next proof must fail on the old runtime at the real legacy bridge and
source seal, with no printable key between conversions to hide the gap.

## Design and consequence analysis before production changes

Baseline: keep the exact epoch check and reject the inconsistent handoff.
Design A: settle an already admitted bridge output in the existing reducer,
using its existing admission token and word lineage, before layout transport
can begin. Retain authenticated focus, exact epoch and single-use transfer
checks. This is the selected direction; exact placement is under independent
review. No additional detector, owner, queue, timer or fallback is admitted.
Design B: settle only after the bridge method returns. Rejected: its background
layout request can already have delivered CreateEngine/FocusOut by then.
Design C: relax the source epoch comparison. Rejected: this would admit stale
or unreported output and weaken the reason the handoff exists.

Candidate generation, lattice retention, ranking, verifier, packages/deltas,
learning and feedback are unchanged; there is no lexical operation in exact
manual conversion. No new model load, cache, RPC, retry or wait is needed.
Expected cost is bounded metadata validation under the existing reducer lock;
latency and RSS effects are estimates until checked, not measured speedups.
The admitted token must not survive owner/focus/revocation changes, an input
gap, or an intervening unsettled key. Failed output cannot settle text; stale
completion cannot advance a successor. Publishing metadata cannot grant
KnownStart or alter word lineage. Existing key and atomic settlement keep
their ownership; daemon detection and terminal single-frame output stay intact.
The same output publication and layout handoff must remain valid across
repeated conversions, including a trailing boundary and accepted completion.

Proof plan: production bridge/observer/callback regression with repeated exact
surface alternation and sealed transfer; adverse stale/unsettled cases;
focused IME discovery and affected contracts; independent review; mandatory
release checks and actual-client smoke before delivery. Record physical
keyboard confirmation separately. All execution is remote under the existing
dedicated-20cpu/resource/Cargo guards. Rollback is only this scoped source
change; preserve prior dirty work and installed bytes until delivery gates.

## Results

Independent audit also found the next deterministic loss: target ContentType
repeats the same `(10,0)` tuple, but both observer and handler revoke the word
unconditionally. The source and target tuple are equal in the private trace.
Record the tuple with existing admitted word settlement in the reducer; a
typed, authenticated unchanged observation for the current owner or exact live
transfer target is idempotent. This does not bypass same-context Get/marker,
seal, profile or ownership validation. A different tuple, absent provenance,
stale target or input gap retains the existing reset behavior. Source-free
activation and revocation clear that metadata. This adds one bounded tuple to
existing context state, not a metadata map or another authority controller.
Reject comparing only a new engine's default tuple, and reject treating any
Passive/failed observation as unchanged. The regression will include target
Set before and after marker plus changed/missing/stale negative controls.

Bridge placement selected after independent audit: a scoped existing admission
token while the bridge owns the engine write lock; each tail publication is
settled before a layout request can escape. RAII clears the witness on normal,
error and cancelled exits; failed/cancelled output revokes word authority.
Speculative clones cannot inherit this witness. No new asynchronous operation
or controller is introduced. Exact lineage and no unsettled received input
remain necessary, not just token equality.

Initial test development found and corrected a borrowed message-body lifetime
and the fixture's ManagedCommit expectation (cursor width11). These were test
setup failures, not causal runtime RED. Remote logs are retained in
`run-67nq2o` and `run-a9KdBa` under the development runner. Runtime authority
has not changed.

Focused checkpoint: causal RED at
`/home/e/projects/lay-development-runner/run-h1DqCR/tests/logs/bin-lay-ibus-engine.log`
refused the first source seal after successful real bridge output. Current
focused PASS441/441 is
`/home/e/projects/lay-development-runner/run-cihT4S/tests/SUMMARY.json`, local
`/home/ubu/.cache/lay/development/run-ce2qo2om/RESULT.json` (15.030s guarded
checks;6.187s selected execution). The regression completes24 bridge toggles
and factory transfers, boundary/no-boundary, matching ContentType before reply
or after ready publication. It checks exact CommitText, transferred surface,
KnownStart, and availability of a prediction input frame on every successor.
This is not proof of displayed/generated suggestions or a physical keyboard.
Three additional tests cover bridge scope cleanup/cancellation and adverse
metadata/settlement identities. The existing hints-change negative now sends
an actual `(0,0)->(0,1)` change; identical metadata is the new idempotent case.
Fixture-only UnknownObject replies from the detached zbus dispatcher are
consumed by exact serial/type/error; no arbitrary output is discarded.

Fresh-context independent review is in progress. The lint gate currently
stops on a positional dead-code baseline mismatch for unchanged preedit
methods after the added clone-witness reset line (added1/removed1, shifted
byte offsets). Log: `run-cihT4S/lints.log`. No full-release verdict or new
installation yet. Existing correctness/physical/cold-start limitations remain.

## Review repair 1: metadata remains authoritative on stamp failure

Fresh review:7/10, one medium finding, no other high/medium findings. The new
setter error return can drop an authenticated sensitive ContentType when the
existing1ms observer rendezvous fails. Word revocation alone neither records
sensitivity nor clears retained text. This is a demonstrated code-path defect;
an actual-setter regression will withhold the stamp deliberately.

Selected repair: restore baseline metadata application after failed observation
and revoke the word. Keep a successfully observed but stale equality witness
as a no-op. Dropping all failed observations was rejected because it loses
password/PIN/private/hidden-text handling; extending deadlines or replaying
callbacks was rejected because it adds scheduling and stale-callback hazards.
The existing setter still owns sensitivity, background invalidation, preedit
clearing and tail closure. No new route, owner, cache, wait or fallback is added.
Candidate retention/ranking may be cleared by sensitive metadata as required;
no lexical score, verifier, package/delta or learning contract changes. CPU/RSS
cost is the existing constant metadata work; no measured speed claim is made.
An unavailable stamp cannot preserve word authority, and an equality witness
cannot affect a successor. Rollback is the single error-branch repair and its
test. Proof covers the actual setter with all four sensitive metadata forms,
retained text, prediction authority and assistance, plus the existing complete
IME and repeated-transfer contracts. Installation remains pending.

Review repair RED: `run-xm5wYi/tests/logs/bin-lay-ibus-engine.log` fails the
actual password property assertion. PASS442/442 after repair:
`run-lZWA13/tests/SUMMARY.json`, local `run-b6_09yf6/RESULT.json`,15.051s.
Second and final review:9/10,H0/M0; no remaining runtime repair requested.

The existing private-client fixture disables precognition and consumes only
SurroundingText; neither can establish terminal Double Shift or displayed
suggestions. Add an opt-in manual-toggle lane to the same harness, keeping the
original scenario bodies and deadlines. Use actual InputContext callbacks and
the existing GNU Readline consumer for DEL frames. A private shell-interface
fixture forwards each ActivateLayout once into actual IBus; this tests the
transport/handoff but is not a GNOME or physical-keyboard proof. Observe
preedit signals with precognition explicitly enabled only in this lane.
No production source/configuration/authority changes arise from this fixture.

The lint gate also exposed a pre-existing `question_mark` diagnostic in
source-free tail epoch binding (`context_runtime.rs`). Replace the nested
checked-add Option branch with `?`, preserving the same early None return and
all binding checks. This is syntax-equivalent, with no authority, allocation,
deadline, package, prediction, feedback or rollback behavior change. No lint
suppression is added. The default/research dead-code baselines will be refreshed
only after both clippy passes succeed, and their actual diff reviewed.

## Candidate and actual client

Remote preflight `run-79ju1V/release-preflight.log`: default/research clippy
PASS, inventories535/359 unchanged except one preedit byte-span shift in each
baseline (+47bytes); manifest refreshed; bound architecture graph/check PASS.
Release IME build58.74s, SHA256
`fe643c10d65a84be412f36c6c18bfe4439227df768e55478d2bc8743a35ffce7`.
Artifact: `/home/e/projects/lay-development-runner/run-79ju1V/lay-ibus-engine`.
Target cache9397420032/12884901888bytes. Four accepted dead-code warnings;
no added diagnostic or lint suppression.

Exact-candidate private client, immediate startup:
`run-79ju1V/client-manual/receipt.json`. Two terminal cases PASS, eight manual
toggles each, one CommitText and one shell activation per conversion, exact
GNU Readline visible text with/without trailing boundary. The third case,
post-handoff preedit, FAILS: all observed materialization returns zero
candidates before lexical readiness, even though key admission and transferred
word remain valid. No retries or larger deadlines were introduced.

The existing paired `post-exact-ready` startup schedule distinguishes this
known cold-start limitation from handoff loss. Same candidate/config/dependencies
and scenario bodies: `run-79ju1V/client-manual-post-ready/receipt.json`,3/3 PASS,
including16 exact terminal conversions and actual visible UpdatePreeditText
`ерить` for committed `пров` after same-context engine recreation. The schedule
observes exact-authority readiness with trace batching; it does not directly
measure L2 readiness latency. Do not relabel immediate FAIL as PASS. Physical
keyboard, real GNOME/Kitty, cold first input and general candidate quality stay
open. Runtime authority on the workstation has not changed.

Mandatory changed/full release scripts are running under the same remote guard:
`run-79ju1V/{release-gates.log,changed.log,full.log}`. Installed bytes remain
`12db8c00` until delivery. Runtime review stays9/10,H0/M0; no candidate-ranking,
model, safety gate or verifier change is included in this repair.

Changed-script execution: all2677 selected correctness/package tests ran with
zero failures, but the command correctly exited1 on a stale known-failure
ledger binding after new test admission. Preserve
`run-79ju1V/changed.log` and its BLOCKED_CONTRACT summary; do not call that
invocation PASS. Rebind only `manifest_sha256` in the existing empty ledger.
No expected failure, exclusion, assertion or historical observation changes.
The final full script will revalidate the bound2677-test suite; execute the
changed script's remaining cargo-check/replay tail as well. Earlier helper
checks remain valid. This avoids rerunning that entire helper prefix in
addition to the mandatory full script. New binding has no runtime effect.

## Final automated gates and delivery

`run-79ju1V/full.log`: complete `scripts/check-lay-full.sh` PASS. Exact bound
correctness/package2677/2677 (2641+36), zero semantic/infrastructure failures,
348.112s; summary `run-79ju1V/full-test-summary.json`, per-target logs preserved
in `run-79ju1V/full-test-logs/`. All four `physical_double_shift_owner_` tests
pass in their genuine daemon/IME targets. Tooling101 tests, one explicitly
optional real-cgroup integration skip. Performance11 and ignored15 remain
excluded by the established release configuration, not reported as executed.
Default/research lint inventories535/359 PASS, no non-dead diagnostics;
architecture, formatting, helper syntax, CLI smoke and all-release-binaries
build PASS. The changed script's remaining cargo-check tail also PASS.
Final external target9397444608/12884901888bytes. No cache cleanup.

Delivery dependency verification found one historical mismatch in the private
fixture: its L1.1 service was `f0af549b...`, while the installed and running
local service is `1cedb27c...`; all eight model/receipt files matched. Preserve
the old runs as historical. A separate immutable `run-79ju1V/local-deps/`
snapshot uses the current local service and a manifest SHA256
`75a0a69332cf26a7c41be570a0ffc496316726540289abd8e94e80663508a2dd`.
Same IME bytes, exact current dependencies, fresh private-client results:

- `manual-local-immediate/receipt.json`: two terminal cases PASS16 toggles;
  third preedit case still FAILS at cold startup.
- `manual-local-post-ready/receipt.json`:3/3 PASS, including16 toggles and
  visible post-handoff suggestion.
- `lifecycle-local/receipt.json`:3/3 PASS.
- `restoration-local-post-ready/receipt.json`:5/5 PASS, including same-context
  restoration, both layout directions and different-field/UnknownStart refusals.

Each path is under `run-79ju1V`; local copies and full-gate logs are under
`/home/ubu/.cache/lay/development/run-97gag00j/`. No failed cold case is relabelled.

Installed at2026-09-08 04:31:50 +03:00 using the earlier verified candidate;
no rebuild during installation. Only the IME file was replaced atomically:
`/home/ubu/.local/lib/lay/bin/lay-ibus-engine`. File and process SHA256 both
`fe643c10d65a84be412f36c6c18bfe4439227df768e55478d2bc8743a35ffce7`.
New process634706/start45244635; unit `lay-ime-release-fe643c10.service`.
Native `xkb:ru::rus` readback preceded the old IME stop; `lay-ime-ru` was then
restored. IBus4715/start2261, daemon3757261/start44087079 and L1.1 service271400
are unchanged. Input-source list remains both Lay engines. Initial bridge
Ping responds with `no-focus`; physical focus/input is not inferred from that.

Backup of12db8c00:
`/home/ubu/.local/state/lay/release-backups/1.0.66-ime-double-shift-51g4d9pn/lay-ibus-engine`.
Installation receipt: local `run-97gag00j/installation.json`. Workstation
runtime authority changed only through this explicitly authorized IME delivery.
Physical verification was requested from the user and remains pending. TD-121
is not DONE; no commit/push or general Wave-quality promotion is claimed.


## First word without leading boundary — 2026-09-08, diagnosis in progress

The user's physical acceptance failed: Double Shift does not work on the
first word. Production remains fe643c10/PID634706. The previous manual client
cases both began with a literal Space; their "no-boundary" name meant no
**trailing** boundary. They did not prove the reported first-word case.

Read-only diagnosis: bridge_actions.rs and shift.rs both reject UnknownStart
before manual planning. The live accepted callbacks retain the typed tail
while WordScope stays UnknownStart after Reset. Private trace snapshot:
`/home/ubu/.cache/lay/development/first-word-9_azzl8s/live-before.jsonl`.
No user text is copied into this document. This is mechanism evidence, not a
new physical PASS. The existing regression choreography is extended with a
source-free first word without Space; expected result is exact manual output,
repeatable factory transfers, and still no automatic word authority.

Implementation preflight remains pending until the observation watermark and
all manual consumers are audited. No production code has changed at this step.

### First-word consequence analysis — selected before runtime edits

Scope: the live failure is TERMINAL purpose10, managed literal commits without
SurroundingText. Only this executable terminal route gains explicit manual
projection of a fully observed suffix. KnownStart keeps all existing routes;
UnknownStart continues to deny prediction, correction, Tab and generic bridge
replacement. Ordinary Double Shift remains exact physical-key projection.

Alternatives, design judgments rather than measured scores:

1. **9/10: bounded observed-suffix scalar count in existing WordLineage.** It
   travels through the existing settled key, source seal and same-context
   TransferGrant. It is zero for KnownStart and all source-free/reset/revoke
   constructors. UnknownStart grows it only on an exact mirrored append,
   bounded by retained tail length; an exact trailing Backspace decrements it.
   A command, navigation, gap, boundary crossing or context reset clears it.
   This retains local text evidence without granting whole-word authority.
2. **6/10: clear the committed-tail mirror on every input gap, then admit its
   exact new suffix manually.** Viable only after auditing every mutation and
   transfer consumer; it discards useful retained context and expands the
   regression surface. Rejected in favor of one scalar provenance fact.
3. SurroundingText can prove an actual field start in supporting clients, but
   the reported terminal publishes none. Treating FocusIn/Reset/empty local
   memory as field-start proof is invalid and does not solve this safely.

The count is provenance, not another owner, timer, queue, text cache, fallback,
model or source of truth. WordCompleteness remains UnknownStart. Existing
bridge fences, owner/activation/revocation/lineage checks and terminal erase
geometry still gate output; both bridge entry and direct manual method apply
the same narrow predicate. No lexical rule, ranking, SafetyGate, edit validator
or verifier changes are admitted. Generic VisibleTail/ReplaceTail/suppression
remain KnownStart, so a manual success cannot grant automatic authority.

Consequences: candidate/lattice retention and ranking are unchanged because
unknown words still create no prediction frame; known-word token equality
must remain unchanged (count0). False manual authority could arise from a
retained prefix after a gap, so the complete planned token must fit inside the
new contiguous observation count. Count scalar effects, not arbitrary key
presses, and never increase it for a key without a mirrored text effect.
Bounded tail scans are at most the existing160-scalar retained buffer; no new
RPC, wait, retry or deadline is introduced. Key settlement does not allocate;
manual admission uses bounded temporary token extraction as corrected below. CPU/RSS impact is one
bounded scalar field copied in existing lineage structures, no new dynamic
memory. Exact observed cost is unmeasured until proof; no speedup claim.

Invalidation and concurrency use the existing reducer/token and final key
settlement. Atomic speculation/rollback must copy/discard the same lineage
fact with its existing state; a cancelled or stale callback cannot publish a
fresh count. Same-context transfer preserves it, new context/Reset/revoke
starts at zero, and a terminal projection preserves scalar count and
UnknownStart. Packages, reloads, deltas and learning do not gain any authority;
unknown-word feedback remains disabled. Future layout packages still have to
satisfy the existing exact projection and edit geometry contract. Maintenance
cost is the additional WordLineage field and shared predicate; removal belongs
to any future replacement of this same context-admission owner.

GTK remains a separate unproven route: no-SurroundingText terminal admission
must not open generic snapshots or exact daemon replay. Read-only review found
that exact manual leases and ready activation installation also need a
no-intervening-key GTK consumer proof. No repair of those mechanisms is silently
included here, and no GUI/physical PASS follows from terminal/private proof.

Required proof: new first-word callback/bridge RED on pre-fix runtime; repeated
exact visible output across both layout directions and same-context factory
handoffs; Unicode scalar and Backspace checks; retained-prefix, command/gap,
reset, different-context and sensitive-field negatives; UnknownStart still
blocks automatic frames/Tab/correction. Run affected focused contracts, fresh
independent review with at most two passes, required changed/full release
checks and actual private IBus/GNU Readline smoke without leading Space. Real
keyboard acceptance remains separate. Rollback is the existing installed
fe643c10 artifact; only a reviewed new IME may be installed, preserving global
IBus/daemon, model services and input-source list. No runtime authority has
changed at this preflight step.


### First independent review — 7/10, one M and two L, repair required

Focused runtime passed446/446 at
`/home/e/projects/lay-development-runner/run-GoJXcP/tests`; harness30/30 passed
in that run's `harness-tests.log`. On the exact installed fe643c10, the new
private first-word lane reproduced the same no-op: immediate0/2, reply(0,false),
`/home/e/projects/lay-development-runner/run-eNkWJR/first-word-baseline/receipt.json`.
The observed first-word regression helper has12 UnknownStart handoffs and12
with a trailing Space, separately from the existing24 known-start handoffs.

Review found a previously retained tail after atomic receipts5/1/4. Uncertain
submission may have changed the client; refusal of the atomic vector can
return handled=false and permit native key delivery (atomic V2 contract§9);
focus-lineage termination cannot prove the old tail still applies. The new
count must not survive any of these outcomes, including when a prefix was
already observed. The first receipt test starting from empty was insufficient.
A new real-AtomicDriver regression begins with acknowledged `a`, proposes `b`,
then consumes each receipt on Shift; both UnknownStart and KnownStart must
lose word/manual authority while the rejected proposal produces no edit.

Selected repair before editing production: use existing revoke_context_word
for non-submitted receipts after consuming their pending callback. Keep the
old literal tail as historical mirror, but revoke its count, token lineage,
prediction/learning authority. No replay, retry, new owner or deadline is
introduced. A subsequent key may reestablish only new observed suffix evidence;
KnownStart requires the existing actual boundary. This also makes the same
existing atomic uncertainty contract conservative for KnownStart. A competing
option to retain receipt1 was rejected because zero atomic effect does not
exclude native delivery. Rollback remains the old installed artifact; repeat
focused/affected checks and the second independent review after repair.

A numeric UnknownStart token had no manual plan but was mapped to daemon exact
delegation. Restrict only this new admission's no-effect result to NotHandled;
keep KnownStart outcome mapping unchanged. Test the actual bridge result and
use a proof-only transport marker to assert zero output, without a wait/retry.

Allocation correction: key settlement introduces no allocation; explicit
manual admission reuses bounded last-token String extraction, including the
existing terminal-executor predicate. These are temporary allocations bounded
by the retained160-scalar buffer, not a new persistent cache. Their timing and
allocation count are unmeasured. No optimization is required for this repair.

### Review repair causal proof

The new448-test selection on pre-repair runtime failed exactly the two new
regressions: retained count1 instead of0 after receipt5, and numeric bridge
reply(3,false) instead of(0,false). Exact log:
`/home/e/projects/lay-development-runner/run-YVebPE/tests/logs/bin-lay-ibus-engine.log`;
local wrapper `run-s4d4y5pc`,15.402s. The preceding run-OQAgHj stopped at fmt
and is not causal evidence. With the two scoped repairs, run-TcpGeO/tests
passed448/448,15.463s; local wrapper `run-ccp6ciuz`. The regression exercises
receipts5/1/4 for both starting completeness states and observes no new output.
Second independent review is pending. These are development proofs only;
the installed fe643c10 and production processes are unchanged at this point.

### Second independent review and exact release candidate

Pass2 accepted the implementation9/10,H0/M0/L0. It independently checked all11
scoped source/baseline identities, both repairs, suffix settlement/seal/transfer,
terminal-only admission and continued generic/automatic refusal. The reviewer
did not execute tests or claim physical/client acceptance;448/448 is the
execution owner's separate result. The two-pass review is complete.

Remote preflight under the existing dedicated20CPU guard passed211.561s in
run-TcpGeO: lint inventories535 default/359 research unchanged, only preedit
byte locations shifted11; no new diagnostic identities. Test manifest now has
correctness2647+package36=2683 required tests, performance11 and ignored15
separate. The existing zero-failure ledger was rebound only to manifest
894702fa971a67d7390c443d17de821a61b24cfcf4e25e56e0858edc27f57c2f;
no allowed failures were added. Bound architecture graph PASS and exact release
IME build PASS58.71s. Candidate SHA-256:
`6dc951483c0e3f9122c995bd9b474bba1967d5e60dd4dba5f19f27b9fa7f4982`.
Target cache9397604352/12884901888 bytes. Exact log is
`/home/e/projects/lay-development-runner/run-TcpGeO/release-preflight.log`;
local candidate/provenance is `/home/ubu/.cache/lay/development/run-ccp6ciuz/`.
All9 installed dependency files match the private-client manifest again.
Mandatory changed/full gates and final exact-client matrix are still pending;
the candidate has not yet been installed at this checkpoint.

### Explicit replan after actual-client matrix failure — production frozen

The changed gate passed2683/2683 in386.324s. The exact6dc95148 combined
first-word lane completed first_word_us with8 exact GNU Readline round trips,
then failed before ANY text in first_word_ru: InputState reported context
admission denied. Original receipt remains FAIL,1/2:
`/home/e/projects/lay-development-runner/run-TcpGeO/first-word-immediate/receipt.json`.
No full gate or installation followed that failure.

Independent read-only diagnosis confirms CreateEngine184, source FocusOut186
installation/seal and Disable187 were accepted. The first unproven transition
is target activation after FocusIn189; Set193 is stale, Shift194/195 refused.
The trace does not reveal which target-acquisition predicate failed. Factory,
source-free, installation and seal functions match pre-first-word baseline;
no manual method or suffix append runs between the new empty owner and failure.
This suggests a separate lifecycle mechanism, but is not baseline reproduction.
The fixture also accepts any new Barrier nonce from the common engine owner;
the checkpoint spanning focus_in and profile selection can consume markers
from the preceding source-free activation. An old marker is not target proof.

Replan: keep the reviewed production implementation and accepted changed
runtime proof frozen. Retain the combined cross-field/profile FAIL separately.
Use a no-text actual-client control on baseline and candidate to check whether
that transition depends on the first-word patch. Add opt-in initial-US and
initial-RU first-word selections, each in a fresh cold private process from
the existing foreign seed. Each must retain all8 exact output/round-trip,
single-activation and UnknownStart assertions. These two independent starts
do not close the combined lifecycle failure. No runtime retry, new detector,
authority weakening, wait or deadline is authorized by this replan. Harness
selector/identity checks and final full release gate still apply; production
review remains the completed two-pass review, with no third runtime patch.

### Revised first-word proof and affected client contracts

The additional baseline initial-RU lane on exactfe643c10 fails0/1 at the first
manual call, reply(0,false). On exact6dc95148, two separate immediate cold
processes pass1/1 each: first-word-us-cold-v2,8 exact GNU Readline outputs,
2.238s; first-word-ru-cold-v2,8 outputs,2.186s. Aggregate for these independent
initial-layout starts is2/2 cases and16/16 exact conversions. Neither starts
with or ends in Space; both retain all original single-output, single-layout
activation, inverse-output and UnknownStart generic-snapshot assertions.

Other candidate/current-dependency lanes pass independently:
manual-post-ready3/3 (16 terminal conversions plus visible preedit),3.590s;
lifecycle3/3,1.955s; restoration-post-ready5/5,3.009s. Their prefixes, startup
schedule and denominators are unchanged. Cold preedit, GTK and the physical
daemon gesture remain outside these positive results. All9 local dependency
files were verified against the immutable private dependency manifest again.

The no-text two-field/profile diagnostic passed2/2 on BOTH baseline and
candidate (empty-context-baseline-v2 / empty-context-candidate-v2). Therefore
this empty control did not reproduce the compound failure after8 manual
handoffs; baseline parity for that compound failure is still UNKNOWN.
Its original1/2 FAIL remains open, not replaced by the independent-start PASS.

The first selector revision completed every requested8-step conversion but
failed its final hardcoded two-case-name assertion; the derived empty control
had the same wrapper-name mismatch after2 successful setups. Those v1 receipts
remain FAILED, not runtime regressions or retrospective PASS. The corrected
selector keeps exact expected case identities and publishes a one-case
completion/behavior status. Final driver SHA-256:
`7deb6767f4a89e7b2c3dc96deec57fe173c620d5d88f965b9aa27e1ff9a5ac5e`.
Harness unit tests30/30 PASS after this change. No Rust source changed after
the completed second runtime review and the accepted changed gate.

Receipts and runner logs remain under
`/home/e/projects/lay-development-runner/run-TcpGeO/`; compact aggregation is
`client-final.json`. Local copied receipt/metadata/actual-output proof is in
`/home/ubu/.cache/lay/development/run-ccp6ciuz/client-proof-summary.json` and its
sibling case directories. The original combined matrix and v1 wrapper failures
retain their own artifacts. The final full code gate is running; installation
has not yet occurred at this checkpoint. This is a scoped terminal first-word
repair, not completion of the entire TD-121 activation contract.

### Final full code gate and scoped installation — 2026-09-08 11:50:25 +03:00

The full code gate passed495.756s: correctness2647+package36=2683/2683,
no semantic/infrastructure failures. It includes current harness self-tests,
fmt, architecture check, manifest/empty-ledger contract, lints, Python/JS/shell
checks, CLI explain and release builds. Performance11 and ignored15 were not
executed. All4 physical_double_shift_owner identities passed in their actual
binary targets. Exact final logs/summary: run-TcpGeO/full.log,
full-test-summary.json, full-final.json, full-test-logs/; target cache
9397608448/12884901888 bytes. This full-code PASS does not change the retained
combined-client1/2 FAIL or substitute for physical input acceptance.

Before installation, all697 Rust source hashes matched the accepted candidate
identity, all9 installed dependencies matched client proof, five positive
client lanes matched candidate6dc95148 and driver7deb6767, and the previous IME
file/process identity matched fe643c10. The current IME was backed up at
`/home/ubu/.local/state/lay/release-backups/1.0.66-ime-first-word-lz6lxv6e/lay-ibus-engine`.
Native `xkb:ru::rus` selection/readback succeeded before stopping only
lay-ime-release-fe643c10.service. The candidate was installed atomically and
started as lay-ime-release-6dc95148.service; selected lay-ime-ru restored.

Installed and running SHA-256 both equal
6dc951483c0e3f9122c995bd9b474bba1967d5e60dd4dba5f19f27b9fa7f4982;
PID2836850/start47876044. IBus4715/start2261, daemon3757261/start44087079 and
L1.1 service271400 were preserved, along with the two Lay input-source entries.
Ping/InputState initially reported no-focus, proving bridge liveness only.
Installation receipt: `/home/ubu/.cache/lay/development/run-ccp6ciuz/installation.json`.
Physical first-word round-trip confirmation was requested after installation
and remains pending. No physical keys were synthesized, no commit/push made.

Runtime authority changed only for explicit terminal manual projection of a
fully observed UnknownStart suffix, with existing erase geometry and context
fences. Generic edits, prediction/learning admission, model packages, ranking,
SafetyGate and verifier authority were not promoted. The final graph refresh
follows this saved installation entry through scripts/update-architecture-graph.sh;
its exact log is run-TcpGeO/graph-final.log. Retain the accepted binary instead
of rebuilding it for document metadata, and compare the compiled receipt's
Rust source fingerprint and check/status projection with the final graph.
