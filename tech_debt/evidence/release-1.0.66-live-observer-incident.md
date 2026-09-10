# Lay 1.0.66: live context observer failure, 2026-09-07

Status: OBSERVER_REPAIR_INSTALLED / LIVE_FAILURE_RECURRED.
Owner: [TD-121](../121-preserve-word-across-ime-layout-handoff.md).

**Superseding live result, 17:17 +03:00:** the installed repaired process again
returns `metadata observer cancelled`, now after `CreateEngine123` sees a
revoked transfer. All526 retained later key admissions fail. No stale binaries
were found. [Recurrence evidence and explicit bounded replan](release-1.0.66-factory-recurrence.md).
The17:03 readback below is historical startup evidence, not physical PASS.

Current checkpoint 17:03 +03:00: repaired IME PID1148501 is loaded from the
verified release binary `95348e4a...23a668`. The live bridge now returns
`passive:no-focus`, not `metadata observer cancelled` or an unavailable
adapter. This proves adapter availability after installation, **not** physical
autocorrection, suggestions or Double Shift acceptance. User confirmation is
requested and still pending. Only the IME binary/process changed; global
IBus4715, daemon272240, L3272161 and L1.1271400 are unchanged.

## Measured incident

At 16:10–16:15 +03:00, the user reported that autocorrection, IME suggestions
and Double Shift no longer worked after installation. The installed CLI is
1.0.66. Daemon 272240, IME 272498, L3 272161 and L1.1 271400 remained alive;
global IBus 4715 was unchanged, with engine `lay-ime-ru`. These process checks
do NOT establish functioning physical input.

The live `LayIme.InputState` call failed with `metadata observer cancelled`,
while `Focused` returned true. The retained admission log has 490 refused
legacy key callbacks and no accepted legacy key callbacks. Its earlier entries
show `FocusOut` serial 28, `FocusIn` 29, `Set` 32, `Reset` 34/35, `Set` 36,
`Reset` 38, then `observer_ingress_refused` for `FocusOut` 39, immediately
followed by `ibus_context_admission_observer: stopped / context admission denied`.
The rolling log also contains older-process entries: do not treat its historical
prediction successes as current 1.0.66 results. Events lack wall timestamps;
the observation time is not the exact failure time.

Private preserved log (contains user text; do not commit its contents):
`/home/ubu/.cache/lay/development/release-1066-incident-wbhHQr/ibus_engine_debug.jsonl`.
SHA256: `2c3c69864e4846e4c2446eef95d468a428861271a290e934f0010c77df79117c`.

## First broken mechanism

`ContextAdmissionReducer::focus_out` can refuse an already opened/revoked
handoff. `ContextAdmissionObserver::apply_ingress` converts that refusal to
`AdapterError::Denied`; `process_message` calls `fail_closed`, and `run` exits.
`Drop` permanently cancels the shared adapter. Subsequent ordinary keys can
still commit text, but lack context authority; bridge reads/manual actions fail.
`Disable` uses the same fatal mapping and belongs to the same failure class.
The exact internal ticket contents at live serial 39 were not logged; the
repeated/revoked-ticket explanation is a source-supported causal hypothesis to
reproduce, not a claimed memory dump.

## Options and decision

- Restart unchanged IME: 2/10. May restore it briefly but retains the defect.
- Temporarily restore snapshotted 1.0.65: 8/10 for immediate recovery. Offered
  to the user; no rollback is authorized merely by this document.
- Narrow lifecycle repair plus causal regressions: 9/10, recommended. A valid
  current-owner lifecycle callback that cannot transfer text must revoke that
  context's authority without killing observation of subsequent contexts.
- Catch every `Denied` and keep running: 1/10. Rejected: erases the distinction
  between a rejected handoff and a broken trust/transport contract.

## Consequence analysis and repair boundary

Reuse `IngressDisposition::Revocation`; add no owner, retry, cache, timer or
fallback. Restrict the change to authenticated current-owner FocusOut/FocusOutId
and Disable reducer refusals. Invalid sender bindings, malformed payloads,
transport/disconnect/cancellation and observer integrity failures retain their
existing fail-closed behavior. A mismatching FocusOutId stays stale, not a revoke
of another context. Refusal must never yield a handoff grant or accept old text.

Revocation must invalidate old tokens, seals and ready results. The existing
fresh FocusIn/source-free acquisition is the recovery path: UnknownStart must
stay unknown until an observed boundary. Test later key/bridge admission as well
as mere observer survival, since keeping a useless task alive is insufficient.
Independent diagnosis also identified a consumer obligation: an authenticated
revoked FocusOut/Disable must still execute local focus/reset cleanup. Existing
callbacks otherwise skip cleanup when they cannot seal a handoff, leaving
`SharedState.active_path` stale. Reuse the typed Revocation stamp and the
existing separate `context_handoff_sealed` flag: callback success means apply
the lifecycle event, not permission to transfer text. Revoked lifecycle clears
that flag and word authority; stale/untrusted events cannot clear another
engine's focus. Assert focus cleanup and Disable reset effects explicitly.
Use the existing reducer and real zbus adapter/legacy callbacks with controlled
event order. No fixture-specific runtime condition; no new polling or sleeps.

Candidate generation, ranking, SafetyGate, deletion geometry, suppression,
physical Double Shift ownership, package reloads, learning and model caches
are unchanged. Their authority remains downstream of valid context admission.
No extra hot-key RPC, allocation queue, deadline or CPU/RSS budget is introduced;
the change runs on lifecycle messages. Main risk: retaining an obsolete lease
after lifecycle refusal, covered by stale-token/UnknownStart negatives. Preserve
the existing pending-before-reducer lock order and bounded stamp machinery.

## Verification and authority

Old-source causal RED is reproduced remotely at
`/home/e/projects/lay-development-runner/run-Umz8i4/tests/SUMMARY.json`:
414 selected IME tests, 411 passed and exactly three new lifecycle regressions
failed with `Denied` at the observer survival assertions (repeated FocusOut,
FocusOut after Reset/ContentType, Disable without a transferable ticket).
Build 4.53 s; test-target execution 5.727 s. Remote action 14.582 s excludes
archive/transport. An earlier formatter-only failed run
`/home/e/projects/lay-development-runner/run-seuj5s` executed no tests and is
not causal RED. Its two formatting differences were applied explicitly.

The source successor changes only adapter lifecycle-refusal classification and
the existing FocusOut/Disable local-cleanup consumer. A fourth regression keeps
bad-sender and malformed-payload observer termination intact. Source edits are
not installed; the live 1.0.66 process remains the original failing image.

Focused GREEN: `/home/e/projects/lay-development-runner/run-xehZP6/tests/SUMMARY.json`,
415/415 selected IME tests PASS, zero failures. SHA256:
`1d128ad12f2ceabad44686a2bf2cd9b74455ee6fa711fb525b309041517e5173`.
Build 4.19 s; target execution 5.728 s; remote action 14.254 s excluding
archive/transport. The four discovered regression identities are registered in
the canonical manifest (correctness 2614, package 36, performance 11, ignored 15).
These results prove controlled lifecycle recovery and preserved trust rejection,
not physical input or installation. Independent review and broader checks are
in progress; installation and rollback have not been executed for this repair.

Review pass 1: **7/10, High 0 / Medium 1, REPAIR_REQUIRED**. A retained typed
Revocation stamp can match the old engine-local owner after a different-path
successor becomes current. Completing old cleanup can then cancel the
process-global precognition worker. The existing shared-tail generation checks
do not protect that global cancellation. Minimum repair: `revocation_matches`
must also require the reducer's current owner; add a deterministic delayed-old
FocusOut/Disable test. This is the one additional repair pass. The owner's
point-in-time check does not claim to introduce a lease across all cleanup;
the pre-existing successful-lifecycle race is not redesigned in this patch.

The wider pre-repair run at
`/home/e/projects/lay-development-runner/run-GeJKt5/tests/SUMMARY.json`
executed all 2650 correctness/package tests with no per-target failures, then
failed the known-failure ledger's manifest-hash binding (the four new test
identities had been registered, but that binding was still old). Its overall
gate is **FAIL**, not PASS. Update only the empty ledger's manifest hash after
the final regression registration; add no allowed failures or exclusions.

Repair-pass RED: `/home/e/projects/lay-development-runner/run-JIv6Uq/tests/SUMMARY.json`,
415/416 tests passed; the delayed-old-Revocation test failed because FocusOut
cleared `old-local-stale` after a different-path successor had installed.
Build 4.34 s, target 5.504 s. The final source adds the current-owner equality
predicate to `revocation_matches`; no consumer may accept an old owner's typed
revocation merely because its local engine still remembers that owner.
Canonical manifest now registers five new tests: correctness 2615, package 36,
performance 11, ignored 15. Its SHA256 and the empty known-failure ledger binding
are `6549cfffc83431cc4821ba9638977287e89895f65d6c105bd98624f5e6343789`.
The allowed-failure set remains empty, and the historical observation is unchanged.

Final focused GREEN: `/home/e/projects/lay-development-runner/run-stLTUo/tests/SUMMARY.json`,
**416/416 PASS**, zero failures. SHA256:
`58486f13848019ad53181add7aa3af35a6d99866fd62b4e78089b2b863b124e6`.
Build 4.43 s, target execution 5.737 s, remote action 14.479 s excluding
archive/transport. This is the final reviewed source, not the earlier 415-test
candidate. Independent fresh-context review pass 2: **9/10, High 0 / Medium 0,
PASS**. M1 is resolved by the current-owner predicate and the causal delayed-old
FocusOut/Disable regression, including successor state and token validity.
The two-pass review/repair budget is complete; no further source repair is
requested. Architecture/changed gates, release build, deployment and physical
acceptance are separate evidence, not covered by the review score.

Final remote architecture and changed gates are PASS at
`/home/e/projects/lay-development-runner/release-1066-observer-1u2gDB/`:
`scripts/update-architecture-graph.sh` followed by `scripts/check-lay-changed.sh`.
Correctness/package denominator **2651/2651**, zero semantic/infrastructure
failures; test-lane action 349.183 s. Performance 11 and ignored 15 are excluded,
not counted as passing. Architecture source fingerprint:
`ca70e36fa310cbbf36ecb3d07c29945d96af159341e879af357d4cc14e82a48d`.
`architecture.log` SHA256:
`dcdc2942be878e4442ffd2bced64303b8d4e10b200af654020a6676f33d49b3e`;
`changed.log` SHA256:
`51144b981a04c0cf133256fef80645c9b42263dd0b56189b3198b2e697b5bc69`.
Graphify and source-binding outputs were retrieved as tracked artifacts only;
the portable graph root remains `.` and unrelated local graph memory is
preserved. Historical replay over the remote host's 200 records is not live
input acceptance on the user's workstation. The prior full-release receipt is
historical; this patch's canonical changed gate is not relabelled a full run.

Verified source requirements: old-source RED; repaired repeated FocusOut with Reset/ContentType,
ordinary and repeated Disable; recovery through fresh focus and visible boundary;
old-token/foreign-context rejection; existing sender/transport failures; focused
IME plus affected contracts, independent review, then release/client gates.
All builds/tests remain remote under the existing resource guards. A private
client PASS is not physical GTK/terminal/Double Shift acceptance. The prior
2646/2646 and old five-case PASS remain historical scoped receipts, not evidence
against the reported live failure. Deployment of the repaired source follows.

## Exact release candidate and delivery

Remote optimized IME build: 59.00 s, 7,707,088 bytes, SHA256
`95348e4a81056a9a476c9aade72194978a4c359abe81ac6983a381989423a668`.
Candidate path: `release-1066-observer-1u2gDB/lay-ibus-engine` under the remote
runner root above. Shared target ended at 9,381,269,504 / 12,884,901,888 bytes.
Both existing lint scopes PASS, no new dead-code entries, zero non-dead-code
diagnostics. Remote `lints.log` SHA256:
`f2caf0369949c5b88aa97ce431dc8bfdd30c32295884c05d9c39c1663fe4adf5`.
`build.log` SHA256:
`daeef057e8b80625af988fefb28b7a4e6681424c5b19aa9f7946ddea79d66e24`.
Archived `changed-results/SUMMARY.json` SHA256:
`64d68bd1e9e1abe098f26455b81406ea7ce041dfa5c3ddc7127b8ccde116bd12`.

The same hash passed all five unchanged V2 private IBus scenarios with the
explicit `post-exact-ready` startup schedule: whole-token ` ljv` → ` дом `,
both mixed-token handoff directions, foreign-field refusal and UnknownStart
refusal. Private service runtime 2.723 s is not key latency. Private IBus was
reaped and no candidate process remained. Receipt:
`release-1066-observer-1u2gDB/client-results/receipt.json`, SHA256
`0b91daf4054e947b76a85520852f4d13ef92ef1b293c14e8ac01052237ae26c0`.
This harness does not add physical Shift detection, prove mixed correction
quality, or exercise repeated-focus physical desktop behavior. Those claims
must not be inferred from its five-case PASS.

Local transfer matched the candidate hash. Delivery cache:
`/home/ubu/.cache/lay/development/release-1066-observer-delivery-P0Lqte/`.
Original 1.0.66 IME is preserved with its mode at
`/home/ubu/.local/state/lay/release-backups/1.0.66-observer-7LOUmA/lay-ibus-engine`.
This is not a rollback to 1.0.65, which was not authorized.

First transaction at 17:00:44 replaced the binary and started PID1137330 but
failed live bridge validation: `context admission unavailable`. It atomically
restored the original binary and started PID1137543. Source inspection plus
live metadata identified the bootstrap failure `No global engine`: killing
the active Lay engine leaves no GlobalEngine for the bootstrap query. The
original binary also failed that bootstrap after rollback. This is distinct
from the reviewed lifecycle refusal; do not hide it or call the first install
PASS. First transaction log `install.log` SHA256:
`470a82f27e0aa68f62cbe641972b809b7c1a9217e407e9d931e6b1906201c32c`.

The second transaction at 17:02:52 reused the existing release controller's
native-head staging sequence: verify `xkb:ru::rus`, atomically replace only the
IME file, terminate only the exact old managed IME, then restore the captured
`lay-ime-ru` engine. No input-source list migration or global IBus restart.
Exit0 / FORWARD_IME_REPAIR=PASS. Candidate = installed = process SHA256;
new PID1148501; bridge `passive:no-focus`; global IBus4715 and other Lay
processes unchanged. `install-native-head.log` SHA256:
`bd6835554614ce43bef67f0d11dfc25556fff229d0d334680297c4c63b763720`.

Runtime authority changed only by installing the reviewed lifecycle repair.
SafetyGate, verifier, model/package files, daemon and user configuration are
unchanged. No local compilation, test suite or training. Version remains
1.0.66; the exact hash distinguishes this repair from the original release.
Bootstrap with no GlobalEngine is not repaired by this source change: future
delivery must preserve a native head as the existing controller does. Any
general startup redesign is a separate decision, not a hidden third repair.
Physical acceptance remains NOT_TESTED until the user's keyboard confirmation;
TD121 and TD125 remain open. The established user-directed 1.0.66 delivery
sequence does not convert missing keyboard proof into PASS.
