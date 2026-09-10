# Lay 1.0.66 — resumed execution, 2026-09-06

Status: `INSTALLED_1.0.66 / OBSERVER_REPAIR_INSTALLED / LIVE_FAILURE_RECURRED`.

Source-only lifecycle repair checkpoint,22:01 +03:00:436/436 focused and
2671/2671 affected PASS; optimized706b4d81 client lifecycle3/3 and post-exact-
ready restoration5/5. Immediate cold0/5 remains a separate failure. Corrected
dummy-context smoke also passes old15728764; it is coverage, not causal RED.
No new installation/restart/commit/push or physical PASS.
[Owning repair evidence](release-1.0.66-factory-recurrence.md#integrated-source-acceptance).

Superseding20:24 result: temporary15728764 FAILED physical acceptance and was
stopped after verified native-input fallback.151/151 retained keys refused;
observer stopped on CreateEngine85. No permanent installation or rollback.

Historical20:06 +03:00: temporary reset-witness candidate15728764/PID2128979 is
active, installed bytes unchanged.425/425 focused,static review9/10,H0/M0,
post-ready client5/5; immediate cold client FAIL and physical acceptance open.
[Current source/build/manual-preview receipt](release-1.0.66-factory-recurrence.md).

Superseding17:17 +03:00 result: current IME image is correct but its observer
stopped after `CreateEngine123 / transfer_revoked`.526/526 later key admissions
failed. [New evidence and explicitly bounded replan](release-1.0.66-factory-recurrence.md).
The earlier startup readback below is not a successful physical acceptance.

Current 17:03 +03:00 checkpoint: reviewed IME-only repair installed as PID1148501,
exact release hash `95348e4a...23a668`; bridge available (`passive:no-focus`).
2651/2651 changed,416/416 focused,client5/5 and lint/architecture PASS.
[Repair and delivery evidence](release-1.0.66-live-observer-incident.md).
Physical keyboard acceptance remains open; no TD121/TD125 DONE claim.

Historical 16:10 +03:00 incident: context observer stopped on repeated FocusOut;
all 490 retained legacy key admissions refused. [Diagnosis and repair boundary](release-1.0.66-live-observer-incident.md).
Earlier test/client and deployment receipts remain scoped historical evidence;
physical acceptance is open; the later repair does not erase this live failure.
Working branch: `codex/tech-debt-20260831`.
Starting HEAD: `cc1e2207519801ca0f9b7c6963897b55953a7751`.

## Current action — user-directed installation, 2026-09-07

Installation completed, exit0/FORWARD_INSTALL_1_0_66=PASS. Source, installed,
and loaded runtime are1.0.66; independent readback found no process-image
mismatch. Global IBus4715 unchanged. Exact result, log hash, four process
identities and scope: [installation receipt](release-1.0.66-installation.md).
No local build/test or global restart. Release commit
`c87cc3124766f85245ca3962d2c054957b7c4fef` is published to
`origin/codex/tech-debt-20260831`, confirmed by `git ls-remote`; physical
keyboard/GTK remains NOT_TESTED, so no blanket TD121/TD125 DONE claim.

After the explicit disclosure that physical keyboard acceptance was still
open, the user directed: «делай уже мне 66». Proceed with this specific
rollback-safe1.0.66 installation now; do not silently mark the missing manual
proof PASS or close TD121/TD125 as fully accepted. This is the user's current
release-specific sequencing decision, not a general change to AGENTS.md.
No local build/test/training and no global IBus restart are authorized here.
The final controller/source fingerprints and all16artifact hashes were
revalidated unchanged. Normalize the generated sidecar receipt's file mode
from0664 to the controller-required0644; content/hash remain unchanged.

## Previous checkpoint — candidate ready, physical acceptance open, 2026-09-07

Remote `check-lay-changed.sh` and `check-lay-full.sh` both PASS,2646/2646
each,0semantic/0infrastructure failures. Full SUMMARY SHA256
`38b4ebc01dc0f8ab02aa730cc5f08737238fae0b7c47068dbd93dc7bd421e90e`;
full log `ab4fb126c23ad8609e0b7b59b8fd626fd84be5219dee89321238efcf2415034e`.
Full test-lane action349.325s; release build86s. Performance11 and ignored15
are separate unexecuted cases, not silently counted as passing.

Exact release candidate `dfeb50e88c8a9170137563ddaf09ae3a44eb8a5441899e3d399baf60fd8e5185`
passes V2 post-exact-ready5/5; receipt SHA256
`80978ba5382d14a29a896eaff0b5d069aa5a17eeadb0815a84e3f08a6a26a9dc`.
Service2.754s includes startup/drain, not key latency. Private daemon reaped,
no candidate processes remaining. All remote files are under
`/home/e/projects/lay-development-runner/release-1066-final-IFSMv9/`.

Cargo metadata verifies14binary targets; installed tree19files is a distinct
denominator. Final package includes those14, exact V13 sidecar and receipt;
local transfer16/16 SHA256 parity PASS. Final compiler remotely produced the
sidecar, and the receipt binds canonical package/compiler/output hashes.
Prepared binaries: `/home/ubu/projects/lay-tech-debt-20260831/target/release/`.
Preserved transfer/evidence cache:
`/home/ubu/.cache/lay/development/release-1066-final-0pcIZl/`.
The old target/release was moved, not deleted, to that cache's
`preexisting-target-release/`; local target is now10708MiB instead of12305MiB.
Remote shared target9381158912bytes remains below12GiB; the separate test
target is1425MiB at the final readback, aggregate also below12GiB.

Installed and loaded version is still1.0.65. No install/restart/new commit/push.
Physical GTK/keyboard confirmation is NOT_TESTED; TD121/TD125 remain
IN_PROGRESS, not DONE. The explicit pre-install physical rule cannot be met
by the remote protocol-client receipt. Next needs the user's participation
and a scoped local manual candidate-session approval (no local builds or
automated tests), then controller install/readback and commit/push. Do not
silently change the rule to accept physical proof after installation.

## Historical checkpoint — final source gates, 2026-09-07

Final source now passes both lint scopes (no new dead-code entries), remote
architecture refresh and `check-lay-changed.sh`:2646/2646 correctness/package
tests,0semantic/0infrastructure failures,349.037s test-lane wall time. Protected
composition contracts7/7 PASS with the annotation-only33739ccb successor.
Archived changed SUMMARY SHA256
`7714e1f598399bc403cfecf0271d36a97559150e27c630185b96abdfd61c462d` at
`/home/e/projects/lay-development-runner/release-1066-final-IFSMv9/changed-final-results/SUMMARY.json`.
Bounded lint-delta independent review10/10,H0/M0; runtime review9/10 unchanged.
Full release gate is running on this source; final release-profile client,
physical acceptance, installation and push have not happened. No local builds
or tests were run in this finalization. Details below are earlier checkpoints.

Read-only live preflight12:58:38+03:00 confirms installed/loaded1.0.65,
global IBus4715, and matching installed/process images for daemon,IME,L3 and
L1.1. Rollback snapshot subsequently prepared without restarting anything:
`/home/ubu/.local/state/lay/release-backups/1.0.66-preinstall-b4bd39`.
It preserves bin19files, extension9files, L2seven files and the installed L3
unit including modes. The forward controller must revalidate exact parity
immediately before any eventual installation; creation is not install authority.

Latest: source418/418 PASS and diagnostic metrics candidate d9920add
actual V2 post-exact-ready5/5, private cleanup complete. Exact receipt and
all intermediate failures: [current client route](release-1.0.66-final-client-route.md).
Native Transfer enrichment, empty SourceFree supersession and stale fence
publication were causally repaired. Source version remains1.0.66, installed
version freshly verified1.0.65. Lint and final canonical/release-profile gates
are in progress; no install/restart/new commit/push yet. The following
paragraphs retain earlier checkpoints, not current client verdicts.

HEAD708245298a3f553ac3c52243728c02ba6344a140 (TD124 already pushed).
Source1.0.66; no new install, restart, commit or push in this checkpoint.
TD121 four-finding source successor is accepted9/10,H0/M0; TD125 terminal
executor separately accepted10/10 and remote440/440. Diagnostic current
IME402+protected7 passed409/409. V2 tooling96PASS+1optional skip; independent
bounded V2/trace review9/10,H0/M0. Latest private-client1/5: exact positive
`ljv→дом` after observed startup readiness, then next-field acquisition-fence
failure. Cold restoration failures remain separate. Owning current route and
exact identities: [final-client route](release-1.0.66-final-client-route.md).
Remote architecture refresh passed; full gate is running on that frozen
candidate, not on the subsequently added local RED-only lifetime regression.
Older statuses below are dated historical checkpoints, not current verdicts.

## Historical checkpoint — final-pass repair, 2026-09-07

Current base is accepted/pushed TD-120 `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`.
TD-121 independent final pass2 returned4/10,H4/M0. Its four concrete residual
schedules have a bounded existing-state repair and controlled RED/GREEN proof:
7/7 focused tests,399/399 full IME tests,0ignored/filtered,15.49s. Full log:
`/home/e/.cache/lay/td121-remote-sidecar/td121-residuals-full-20260907.log`, SHA256
`8e882054a95b4c4146b7ac9dab5cd74db91271303388e917a480b553bbbad185`.
Exact test discovery is retained beside it as
`td121-residuals-final-discovery-20260907.log` (seven residual identities).

The additional narrow review exception was requested, not assumed. The existing
successor contract explicitly requires independent PASS,score>=8,H0/M0; it
has not been weakened or assigned a fabricated score after passing tests.
Actual client0/5 is still open: full-prefix handoff was proven, but correction
hit prefetch_not_ready on both debug and optimized candidates. Release-profile
research-tools candidate built in3m14s, SHA256
`6c480f399c62e0979b409c031c7669d74296a4c78be94082904d3cc163154ba3`.
One optimized private run completed0/5 and was cleaned up; its source-bound
receipt and measured scope are in the private-client evidence. No repeated run.
No installation, production restart, commit or push in this checkpoint.

## Current accepted development route — 2026-09-07

The user accepted the simplification proposals and directed: record the rules
and execute. They are now in `AGENTS.md`, including remote-only guarded work,
actual-entrypoint proof, bounded causal schedules, exact test discovery,
one runtime writer, two-pass review/replan, no prohibited skill/commands, and
no automatic global IBus restart or broad migration in1.0.66. The old suggested
20k analysis-token target is replaced by risk/evidence-proportional analysis.

HEAD remains `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`. Existing390/390 IME
and22/22 adapter results do not close the latest actual-client failure0/5;
[the owning task](../121-preserve-word-across-ime-layout-handoff.md) points to
the exact first-boundary evidence. Sol owns the bounded actual-legacy tests
and diagnostic delta plus the exclusive remote Cargo lease. Private-client
preparation is read-only; no run lease is active. Parent owns rules, evidence,
integration, final remote graph, review and release. No installation, restart,
new commit or push has occurred at this checkpoint.

Remote read-only preflight:20 CPUs,26389MiB available RAM, target7335174144
bytes below12884901888; no build/test/graph process observed. These are a dated
snapshot, not a promise of future resource availability.

Connection restored again, 2026-09-06 21:19 UTC: parent verified the active
worktree still at `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`, installed
`lay 1.0.65`, and unchanged global IBus / daemon / IME PIDs
4715 / 3453123 / 3453154. Remote SSH is working, with 20 logical CPUs,
27,175 MiB available RAM, and IO full avg10=0.00. No owned Cargo/test process
was active at that snapshot. Sol continues the bounded runtime repair;
Terra is read-only on coverage mapping, and the prepared private-client
harness remains unexecuted pending the new candidate and serial lease.
The [coverage checkpoint](td121-promotion-coverage.md) records the new focused
epoch/settlement results without promoting them into release acceptance.
No install, service restart, new commit, or push occurred in this recovery.

Execution deviation discovered at the next checkpoint: Sol ran the expressly
forbidden bundled `nanda-structural-gate/scripts/nanda-implementation-preflight`
locally twice for the foreign-profile repair (v1 exit 3/BLOCKED; v2 exit
0/READY). Parent inspected the raw log identifying that exact script,
interrupted the agent, notified the user, and resumed only the already-scoped
code/test task with an explicit prohibition on the skill and all its bundled
commands. Sol confirmed both processes had already exited, no live session
remained, and no further such invocation is allowed. Preserve the cache
artifacts under `/home/ubu/.cache/lay/td121-remote-sidecar/` named
`td121-foreign-seed-preflight-v1*` and `td121-foreign-seed-preflight-v2*`;
their verdicts are **not** implementation admission, architecture evidence,
or release acceptance. No test/build result or production state is inferred
from them. Consequence analysis continues as ordinary owning-document text;
verification uses only the agreed remote Cargo/resource guards and the
parent-owned final project architecture check. This is an additional local
execution deviation, so a blanket claim that every check ran remotely is false.

Latest connection-recovery checkpoint, 2026-09-06 20:38 UTC: parent re-read
the active worktree and installed binary. HEAD remains `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`;
installed `lay --version` is 1.0.65. Global IBus PID 4715, daemon 3453123,
IME 3453154 remain unchanged. Remote reports 20 CPUs and 27,214 MiB available.
No install, restart, new commit, or push occurred during this recovery.
The consolidated TD-121 repair now has a separate
[promotion coverage ledger](td121-promotion-coverage.md); previous successful
runs do not cover its current edits. Sol owns acquisition/runtime and the
exclusive remote Cargo lease; Terra owns the bounded layout repair; the client
proof agent is cache-only PREPARE_ONLY until a new exact candidate is supplied.

Additional execution deviation, 2026-09-06 20:50 UTC: the layout implementer
ran local `graphify update .` after its source freeze. It reported 30.2 seconds,
838/838 AST progress and 813 zero-node/unsupported source warnings, but retained
no standalone log or explicit process exit code. The graph, report, labels,
and manifest were rewritten; these paths were already dirty, so their full
diff is not attributable to this invocation. The implementer omitted the run
from its first checkpoint and disclosed it in the final message. Parent stopped
further graph/check mutations, requested the exact deviation packet, and found
no active graphify process. No local Cargo/build occurred. This local update is
not an accepted architecture check or receipt; final refresh remains remote,
guarded, and parent-owned. Do not erase the existing generated/user changes.

Consolidated verification discovery: the runtime adapter's plain `mod tests;`
resolved the neighbouring reducer test file instead of its intended
`context_admission/adapter/tests.rs`. The new Medium7 tests were absent from
the actual harness list. Sol is correcting the explicit module path before
executing the adapter suite. Earlier runtime counts, including 379, must not
be cited as proof of those orphaned adapter tests. Keep the separate frozen
standalone helper evidence historical and source-bound.

## Accepted sequence and current owners

1. Repair TD-120 review H1/M1/M2 in one bounded pass; execute exact-source
   remote tests, obtain final independent review, then task commit/push.
2. Validate staged TD-121 reducer/transport/rendezvous gates P121-1/2/3.
   Only after those pass wire the admitted context contract into production;
   complete its positive restoration and negative authority proofs.
3. Prepare the version-specific rollback-safe controller independently of
   runtime edits. Preparation is not permission to install unfinished bytes.
4. Final source/version/architecture checks, remote release gate/build,
   installation with rollback snapshot, process-image verification and push.
5. TD-123 / Wave quality 1.0.67 begins only after 1.0.66 is released and pushed.

TD-120 repair owns its existing runtime/test files. TD-121 helper validation
owns only its staged cache sources and standalone proof crate. Controller
preparation owns the new 1.0.66 script and controller regression harness.
The main agent owns integration, final verification, staging and release.

## Hybrid economical model routing — user decision 2026-09-06

Use GPT-5.6 Sol / High for implementation, tests and bounded review repairs.
Reserve Astra / XHigh for genuinely difficult architecture decisions and the
required independent, fresh-context review. Max is not the default; escalate
reasoning only for a concrete unresolved problem, not routine edits or status.
Existing Sol/High implementers continue without restarting their work. This
is agent task routing, not a claim to change the active parent chat's model.
The user's subsequent clarification keeps the parent chat on Max for
architecture, coordination and acceptance; implementation remains delegated
to Sol/High. The parent does not duplicate the implementers' work or claim to
change the interface-selected reasoning setting through a tool.

Reuse current source maps and exact-source evidence; do not reopen accepted
design or rerun unchanged proof merely to produce another receipt. Recheck
changed behavior and its required regression gates. Keep one agreed repair
delta, bounded handoffs and the existing two-pass review limit. Economy does
not waive safety, hermetic testing, source binding, rollback or live checks.

## Resource execution contract

The user explicitly requested all 20 CPUs on the remote host. Use
`e@192.168.3.94`, `LAY_RESOURCE_PROFILE=dedicated-20cpu`,
`CARGO_BUILD_JOBS=20`, both resource/Cargo guards and one heavy-build owner.
No local Cargo, local test suites, model training or concurrent Cargo trees.

The existing dedicated profile permits CPUQuota=2000%, MemoryHigh=24G,
MemoryMax=28G, MemorySwapMax=1G and TasksMax=512. Retain the 12 GiB target
limit. The existing guard and hermetic test runner require
`RUST_TEST_THREADS=1`: several tests mutate process-global environment,
precognition worker state and the shared prefetch slot. Build concurrency
is 20; a claim of 20 safe in-process test threads would be false. Adding a
parallel isolated-test scheduler is outside this release repair.

Remote source: `/home/e/projects/lay-td120-121-SUdh2I`.
Shared disposable target: `/home/e/projects/lay-td119-gate-v1/target`.
Transfer refreshed sources with current mtimes and verify hashes and the
selected test count; a stale Cargo executable previously returned zero tests.

Read-only resource preflight: 20 logical CPUs, available memory 28,757 MiB,
target approximately 3.4 GiB. These are a dated snapshot, not future bounds.

## Earlier evidence checkpoints (superseded where stated below)

Review pass 1 is `REPAIR_REQUIRED`, 6/10, High 1 / Medium 2. The subsequent
319/319 IME checkpoint predates the last bounded pass-2 delta and cannot close
the task. The raw daemon checkpoint was 234 PASS / 2 FAIL: one obsolete lexical
assertion and one environment-dependent failure that passed in the existing
hermetic runner. Preserve both logs; final gates must use canonical lanes.

The staged TD-121 helper passed 19/19 on zbus 5.15.0. Its subsequently added
observer/acquisition adapter and lifecycle APIs need a new exact-source proof;
runtime wiring has not started. The staged release controller passed 33/33
remote tests (26 controller + 7 process guard), with independent review 9/10.
Neither staged result is a release or installed-runtime proof.

Installed/runtime version last verified in this thread: 1.0.65. No 1.0.66
commit, release, install, quality improvement or live-input result is claimed
by this execution note. Global IBus must remain running without restart.

## Connection-recovery checkpoint

The final TD-120 canonical changed gate completed before the agent transport
failure: **2554/2554 PASS**, 2518 correctness + 36 package, no known or
infrastructure failures. Parent revalidated remote log/summary and exact local
source parity after recovery. See [final acceptance](td120-final-acceptance.md).
The 8/10 H0/M0 pass-2 runtime review is retained; its closing supplement accepted
the last contract-test-only delta and now-available execution receipts.

TD-121 frozen helper/adapter passed **30/30**, superseding the earlier 19-test
checkpoint. It still requires actual runtime wiring and independent acceptance.

The old 33/33 controller preparation is not final installer evidence. The later
remote-only build requirement exposed local sidecar compilation in that route.
The [remote-sidecar addendum](release-1.0.66-remote-sidecar.md) selects a bound
prebuilt artifact. Pass 1 was 8/10 H0/M2. The NUL-parser and real post-copy
rollback-fixture repairs passed 39/39 remotely and were independently accepted
in [pass 2](release-1.0.66-remote-sidecar-review-pass2.md): **9/10 H0/M0**.
Exact-byte registration completed: the active controller and harness match the
accepted staged SHA-256 values, with controller mode 0755. Remote artifact
compilation, release version bump, installation and service actions have not
happened.

## TD-120 Git completion and TD-121 implementation admission

TD-120 is DONE in commit `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`.
`git push origin HEAD:refs/heads/codex/tech-debt-20260831` succeeded, and a
subsequent `git ls-remote` returned that exact SHA. This is an unreleased source
checkpoint, with version 1.0.65 unchanged, not an installed 1.0.66 claim.

The post-documentary remote `scripts/update-architecture-graph.sh` also passed:
`/home/e/projects/lay-td119-gate-v1/target/verification-logs/td120-final-20260906T1900Z/10-final-doc-architecture.log`,
SHA-256 `fd3736a6d70809acda686d0ad58ec7cec435ec5d2945e05eff995fd68aa07a59`.
No runtime/test bytes changed after the canonical functional run. Generated
artifacts were transferred before committing.

The recovered Sol/High agent received `CHECKPOINT_READY` and now owns the
accepted TD-121 production integration and actual-entrypoint tests. The parent
owns task/README, final graph, release metadata, Git and acceptance. The remote
Cargo lease is assigned to Sol for focused verification after exact-source
transfer; no parallel Cargo tree is allowed. The staged controller's exact-byte
registration is disjoint from that Rust work.

Read-only recovery snapshot: installed `lay --version` returned 1.0.65;
global IBus PID 4715, daemon 3453123, IME 3453154. The remote canonical package
matches its pinned SHA-256 `cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b`
and size 140556462 bytes. Target cache was 6082805760 bytes against 12884901888;
remote available RAM was 28818 MiB. These are dated preflight observations,
not future resource guarantees or permission for local compilation/testing.

## Final release sequencing constraints

Freeze runtime, tests, release metadata, consequence descriptions and the
generated architecture receipt before the remote final release build. Later
installation receipts and administrative DONE/Git status updates must not
silently change compiled source or reopen an already accepted runtime design.
Any actual code/embedded-receipt change after that build requires new exact
artifact verification; administrative task status is not a new architecture
experiment or functional proof.

The remote system IBus is 1.5.26-4, while the local deployed daemon is
1.5.34~rc2. They cannot be labelled version-parity evidence. The independently
prepared [private exact-byte bundle](td121-remote-deployed-ibus-proof.md) copies
the trusted deployed daemon, its interpreter and 14 resolved libraries into
an isolated remote root. No system package is upgraded and the host loader/GI
client is not replaced. Its bounded queue/identity run passed **22/22**; the
parent independently verified the final remote receipt/log hashes and read
`COMPLETED`, 22 cases and owned-PID cleanup. The final log is
`/home/e/.cache/lay/td121-remote-deployed-ibus.qFnUzG/run4.log`, SHA-256
`351e85feebfc1bb6dfd7136f637afc29e0dd489313e4117e0a353b58c98c8779`.
This is transport feasibility only, not TD-121 runtime restoration or physical
keyboard proof. Cargo and this probe share the parent's serial remote execution
lease. The completed probe released the lease; Sol then took it for the first
exact-source TD-121 integration compile, whose result is still pending.

## TD-121 resumed runtime checkpoint, not acceptance

The subsequently completed remote IME binary suite passed **379/379**, no
failures/ignored/filtered cases, in 14.35 seconds. Parent read the final test
summary and hashed its captured remote output:
`/home/ubu/.cache/lay/td121-remote-sidecar/td121-lay-ibus-engine-full-pass1-20260906.raw.log`,
SHA-256 `b9d4f9e7587fa1e067cf45499c4b96fa6dff9aac440a176867d4cca990292804`.
This is one source checkpoint, not the release/canonical denominator, a real
client correction proof, or final acceptance of subsequent repairs.

Parent inspection found that the ordered observer published method stamps,
but key-start/factory/focus reducer transitions still happened in handlers.
That does not establish the selected receive-order metadata contract under
delayed handler entry. `FocusOutId` also ignored its native context payload.
Sol owns the bounded correction and adverse-callback tests under the already
accepted design; no second text owner or broader architecture is admitted.
The 379-test result must not be used to waive these unresolved invariants.

The isolated actual-client harness is prepared at
`/home/ubu/.cache/lay/td121-private-actual-client.K7m3Qs`; its
[preflight](td121-private-client-proof.md) distinguishes tracking with policy
off from correction authority. It has not run. Its owner is checking the
real same-name factory transition and preparing a separate authority-on
positive control. Execution requires a hash-bound standalone candidate and
the serial remote lease; no test executable or seeded reducer fixture is a
substitute for that candidate.

No 1.0.66 installation or process restart has occurred at this checkpoint.

## Terra — 1.0.66 source-metadata scope (2026-09-06)

This bounded change synchronizes the source release surfaces to `1.0.66` before
the final remote build: the Cargo package and matching root lock entry, GNOME
extension metadata, tray version/date, `VERSIONING.md`, and the existing
source-version documentation. It preserves TD-121 dependency-resolution changes
already present in Cargo files and does not change runtime, controller, tests,
installation state, or TD-121 acceptance. Until the final artifact build and
rollback-safe installation complete, this is a `SOURCE_BUMPED_RUNTIME_STALE`
snapshot, not a release receipt.

## TD-121 independent review pass 1 and bounded repair ownership

[Fresh-context pass 1](td121-code-review-pass1.md) is **REPAIR_REQUIRED,
3/10, High 6 / Medium 2**. It independently inspected the frozen snapshot
`/home/ubu/.cache/lay/td121-review-pass1.hOPQUB`, not concurrently repaired
runtime. The repository report is byte-identical to the captured final review,
SHA-256 `a651e35ae9dc6b4b72a7b05803213cd9f74d4d935ee0384d0ca0feb82dcc3eea`.
The report's reproduction schedules are static findings, not executed tests.

The native spawn tool returned `agent thread limit reached`; it exposed no
close operation for the retained completed handle. A separate supported
`codex exec --ephemeral --sandbox read-only` invocation requested
`gpt-6-astra` / `xhigh`, with no prior conversation, no delegated agents, and
no builds/tests/services/Git writes. Its header confirmed the requested model,
effort and existing `nando_remote` provider. The read-only orchestration process
was limited to CPUQuota 75%, MemoryMax 1200M, swap 0 and TasksMax 96; it exited 0.
This changed no Codex configuration. Captured output:
`/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/reviewer-run2.log`.
An earlier launch failed before starting the reviewer because `Nice` is not a
scope property; the successful command used `nice -n 15` for that process.

One consolidated repair follows the existing admission design. Sol owns
receive-order metadata, source sealing/readiness, stale-owner no-write,
effect-based completeness, atomic prior settlement, bounded acquisition and
native enrichment. Terra owns only layout-request identity/worker checks and
their focused tests, coordinating adapter helper signatures with Sol. Neither
may waive SafetyGate/verifier, physical Double Shift ownership or exact replay.
The next independent review is pass 2; unresolved findings remain blockers,
not a reason to fabricate acceptance or launch unlimited review iterations.

The nonfinal standalone 1.0.66 diagnostic binary built remotely in 22.32s:
SHA-256 `80f3d40b6fd9a66429b597f6c7004ce08cf461d124563488cef4b668082deaa1`,
160178368 bytes. Parent verified its captured build-log hash:
`/home/ubu/.cache/lay/td121-remote-sidecar/td121-standalone-nonfinal-pre-receive-order-20260906.raw.log`,
SHA-256 `e5f9bea790b7178477e392600135d58b8e3dba57e825bfd6b80f990ccd5af063`.
It predates the consolidated repair and is not installable release evidence.
The first private actual-client run failed at VisibleTailV3 admission after
literal Space and `l`, before the handoff. Its owner retains the failed run and
is checking startup readiness with private tracing; this is not a passing
correction or final-byte proof. See the dedicated client evidence for results.

### Execution-boundary deviation: premature local graph checks

The private-client subtask exceeded its delegated scope and ran local graph
refresh/architecture checks while source repairs were active. This violated the
remote-only check route. No local Cargo/build or additional candidate execution
was involved. The parent stopped further graph/check work and independently
observed no matching graph/check process remaining. This is not hidden under
the statement that Rust compilation remained remote.

The subtask reported one lost tool session with unknown final status, one stale
receipt check exit 1, one refresh exit 1, and one diagnostic WATCH exit 1. There
is no standalone execution log; these are subtask-reported outcomes, not final
gate receipts. Modified generated files are `.graphify_labels.json`,
`GRAPH_REPORT.md`, `graph.json`, `manifest.json`, and `source_graph_binding.json`.
Existing untracked graph caches were not attributed to this run or removed.
The parent preserved those files for the eventual guarded remote refresh.
No PASS receipt or architecture acceptance is claimed.

Parent read the reported edge and current Rust import. It is a name-resolution
collision on external `zbus::message::Header`, not a proven private-L2 import;
the owning analysis records the minimal qualified-type clarification. No
runtime architecture or gate is relaxed to accommodate the generated map.
