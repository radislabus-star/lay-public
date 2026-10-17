# L3 full-proof execution placement — 2026-10-07

Status: installed cold trainer; source, frozen parity and initial live resource
checks PASS. Keyboard runtime and text-edit authority unchanged.
Owner: `docs/l3-online-phase-field.md`. Source: commit
`69ce94bc9b2b7559769c9f75701f66c1df5781ad`, isolated branch
`codex/l3-proof-offload-20261007`.

## Measured failure and ownership

The installed trainer (SHA-256
`12a8e5e354a1e69ca19bb8b7cec25081e3de8f74b5bf340bcf52eb67cbc5b4fb`)
executes a full frozen transition differential synchronously after each
targeted PASS. The latest ten completed differentials visit 15,704 fragments
and take 18.76–23.77 minutes at the desktop's 25%-of-one-core ceiling. A fresh
four-second sample measured 25.0% of one core. Since October 4, nine attempts
completed and no cycle error was logged. This is expensive proof execution,
not evidence of an infinite failure retry. The 156 saved full receipts have
zero gained top-1 on the frozen nonregression set; the last admitted delta improves
two of two targeted rows. These are separate denominators and neither proves
organic input quality.

The first excessive-cost transition is targeted PASS -> local full proof.
The existing online owner continues to read feedback, select one relation,
compile it, run targeted proof, validate full proof, admit and compact. C01,
C02, C05, C08 and C10 remain conjunctive. No keyboard adapter changes.

## Designs and choice

1. Batch relations: amortizes the frozen pass, but changes the accepted
   single-relation unit and interference between deltas. Defer this design.
2. Reuse partial/full proof results: could reduce total work, but requires an
   independently proved affected-set boundary and identities for all L2/L3
   dependencies. Defer this design; no proof cache is introduced.
3. Execute the same full-proof CLI on the existing remote worker: chosen.
   Changes execution placement, retains the complete denominator and all
   regression counters, and uses the existing portable content-bound receipt
   admission. An opt-in runner replaces local execution for this installation.
   Other installations without a configured runner retain their existing CLI.

## Consequence analysis

- Candidate retention, ranking and false authority: no classifier, lattice,
  decision or scoring changes. Transfer the exact proof inputs and dependencies;
  use the same candidate binary and existing full differential CLI. All five
  zero-regression counters remain mandatory. A remote result grants no text
  editing or direct publication authority.
- Latency and CPU/RSS: network and worker availability affect learning delay,
  never synchronous input deadlines. Desktop quota remains 25%. Full evaluation
  moves under the worker's existing dedicated-20cpu guard and shared lease.
  Local compilation/targeted proof/compaction still cost resources; measure them
  separately and do not claim zero background CPU. Measured placement timings are recorded below.
- Identity/invalidation: no result cache. Snapshot manifest and every referenced
  base/delta, corpus, surface evidence, evaluator bytes and L2 inputs. Hash before
  transfer, verify remotely and again before return. Portable baseline/delta
  identities are checked by existing admission. Preserve remote receipt paths
  so same-path legacy compatibility cannot bypass portable content validation.
- Reloads and feedback: retain journal cursor, causal episode identities,
  ready thresholds, single relation selector and delta-free compact publication.
  No new input owner, learner or model source of truth. The remote executor
  only computes one cold receipt; only the existing local owner publishes.
- Concurrency/stale results: one synchronous request per existing online owner,
  one remote resource lease. Mutated input, evaluator mismatch, missing receipt
  or changed baseline refuses admission; no retry may publish stale bytes.
- Failure: configured remote execution never falls back to expensive local
  proof. Retain pending evidence on transport failure, lengthen the existing
  polling sleep with bounded backoff, and reset it after success. No queue,
  detached job, new timer or polling subsystem. Last admitted model remains
  available if the worker is unavailable.
- Compatibility/maintenance: only the trainer, its cold helper and service
  configuration change. IME/daemon bytes and processes remain intact. Keep SSH
  host, login and worker paths in private configuration. Remote temporary inputs
  are owned per attempt, permission-restricted and cleaned on completion;
  preserve compact local request/receipt evidence. No credentials in Git.
- Rollback: restore the saved trainer and service drop-in; restart only
  `lay-l3-online.service`. Keep admitted model and journal/state intact.

## Proof and promotion gates

Required: adapter rejection tests (missing/failed runner, wrong/stale content,
non-PASS/regression receipt); remote helper tests (transport quoting, hashes,
dependency binding, private cleanup); existing trainer and admission contracts;
mandatory affected/release source gates; independent review; a real remote
full differential with all five counters zero and matching full denominators;
guard/lease evidence; live post-install CPU/RSS and service identity. Preserve
exact IME/daemon hashes and PIDs. Physical client acceptance is not claimed by
this cold-executor change. Measured outcomes and receipt paths are recorded below.

## Infrastructure replan after the signal check

The actual signal scenario refused to start: the worker's user service was
delegated only `memory pids`. Scope memory/swap/tasks limits were present, but
`cpu.max` was absent although the existing guard log reported CPUQuota=2000%.
The new actual-envelope check correctly refused this environment. No assertion
or proof gate is relaxed. Systemd on this worker cannot change Delegate through
`set-property`; that attempt failed before modifying the unit.

The bounded repair is a user-instance-specific systemd drop-in adding `cpu` to
the existing `memory pids` delegation, followed by manager configuration reload
and an actual scope check. No service restart, parent resource ceiling or
unrelated workload throttle is authorized by this repair. Preserve the original
delegation receipt; rollback removes only the newly owned drop-in. The remote
executor and its ownership design remain unchanged. This replan addresses the
first observed infrastructure failure before installation, rather than another
model experiment.

The delegated controller also had to be enabled in the existing user/app
subtrees. The user manager retained its old controller-availability mask after
reload, so it was reexecuted in place; its MainPID stayed unchanged. A fresh
guarded scope then exposed `cpu.max=2000000 100000`, memory high/max
25,769,803,776 / 30,064,771,072 bytes, swap max 1,073,741,824 and tasks max 512.
No service was restarted by this infrastructure repair. The application slice
received explicit default CPUWeight=100 without a parent CPU ceiling.

Measured focused trainer check: 44 selected/executed/passed, zero failed,
33.4 seconds; receipt
`/home/ubu/.cache/lay/development/run-r6jpyhet/RESULT.json`.
Measured remote transport checks after infrastructure repair: 10/10 PASS,
including the actual SIGTERM worker/evaluator cleanup scenario; receipt
`/home/ubu/.cache/lay/l3-proof-offload-20261007/worker-build-ufe4nair/RESULT.json`.
Two independent source reviews completed; ordinary TERM/HUP, portable identity,
proof dependency binding, no fallback and publication ownership were reviewed.
Hard SIGKILL/OOM/power loss and an interrupted transfer before worker startup
do not have universal cleanup guarantees. These focused passes alone do not establish release or installed-runtime
acceptance; those results are recorded separately below.


## Release manifest binding repair

The full canonical discovery passed with 3,040 registered tests, exactly four
additions and no removed or changed test policies. All selected Rust tests
reported zero failures, then the known-failure ledger correctly rejected its
old manifest hash. Update only that hash to the new canonical manifest. The
ledger still contains zero exceptions, and its observation bytes and identity
are unchanged. The failed release receipt is
`/home/ubu/.cache/lay/l3-proof-offload-20261007/worker-build-bztj2_3s/RESULT.json`;
this is a metadata failure, not a passing release gate.

At shutdown the learner had already admitted generation 174 at 10:46 EEST.
The saved targeted/full receipts bind the previous compact base, SHA-256
`41f72d2e725cfb6fe257dda32c1168aa5f4cc8b017ff2dcca92dc5fa61b1f9be`;
the current compact base is
`cc58c4ba6ec17d0c4bbf373237a4da26f97f5b0fcc3a4881a7ad6797264ff0d8`.
For the preinstallation executor check, use a private manifest pointing to
that preserved previous base and the saved delta. This reproduces the original
full-proof inputs without changing or admitting anything in the live model.

## Real executor check and parity follow-up

The complete release gate passed: 3,014 correctness/package tests, zero known
or infrastructure failures, mandatory lints, syntax, architecture, Firefox
adapter checks and release build. Receipt:
`/home/ubu/.cache/lay/l3-proof-offload-20261007/worker-build-v1ei0h7m/RESULT.json`.
The real remote full proof passed on 15,704 fragments, 50,592 lattice transitions
and 41,064 compared transitions; all five regression counters are zero. The
worker pass took 202.074 seconds; the whole guarded transfer/evaluation/return
took 215.754 seconds. Evaluator SHA-256:
`f7cebe0037fdc5e399cb8a33caed9ae38c8d78ac950cd6a5b67cf68cd46a806c`.
Receipt: `/home/ubu/.cache/lay/l3-proof-offload-20261007/PREINSTALL.full-proof.json`.
Actual 20-core quota, 24/28 GiB, 1 GiB swap, 512 tasks and shared lease were
checked in the running scope. This is a nonregression proof, not organic quality.

Absolute supports/top-1 are 1,853/1,801 for both baseline and candidate, versus
1,875/1,821 in the historical receipt. Its old mutable usage snapshot was not
recorded; today's usage events/counts also changed during the experiment.
Do not infer algorithm regression, parity or quality improvement from that
unmatched comparison. Before installation, run the original installed trainer
and candidate on one new, identical frozen dependency snapshot using the same
remote transport. Require equal input identities, denominators and all numeric
differential counters. This follow-up has no model publication authority.


## Final frozen parity and installation — 2026-10-07

The previously installed trainer reports version 1.0.78; the candidate is
1.0.80. This byte/version distinction was checked before installation. A
controlled paired run used one frozen snapshot of every model/corpus/usage
input. Both evaluators ran through the same guarded remote executor. All input
identities, package hashes, dimensions and numeric differential counters match:
15,704 fragments; 50,592 lattice transitions; 41,064 compared transitions;
1,853 supports and 1,801 top-1 for both baseline and delta; all five regression
counters zero. Original and candidate both PASS. Receipt:
`/home/ubu/.cache/lay/l3-proof-offload-20261007/parity-o84k3p0h/COMPARISON.json`.
The historical 1,875/1,821 result cannot be compared as identical-input evidence:
its usage snapshot is unavailable. The paired result removes the candidate
algorithm/transport regression concern for this exact fixed experiment.
It does not measure organic correction quality or every damage class.

Measured remote evaluator durations: original 200.463 s,
candidate 133.082 s on the paired snapshot; the first
candidate pass took 202.074 s. These are placement measurements on this worker,
not a language or algorithm-only benchmark. Existing desktop passes took
18.76–23.77 minutes at 25% of one core. No corpus or comparison denominator was
reduced. Worker evaluator and unchanged resource guard now live in private,
versioned durable storage, independent of disposable build/run directories.
SSH connection data remain exclusively in the private worker configuration.

Installation replaced only the cold trainer and helper, installed a private
0600 worker configuration and added the L3 service environment drop-in. Only
`lay-l3-online.service` was started; no global IBus, IME, daemon, input source,
extension or keyboard adapter was restarted or replaced. Installed trainer
SHA-256: `f7cebe0037fdc5e399cb8a33caed9ae38c8d78ac950cd6a5b67cf68cd46a806c`.
Exact installation/rollback receipt:
`/home/ubu/.cache/lay/l3-proof-offload-20261007/INSTALLATION.json`.
The source build snapshot is recorded in
`/home/ubu/.cache/lay/l3-proof-offload-20261007/worker-build-v1ei0h7m/request.json` and its retained source archive.
The final documentation/generated graph refresh does not rebuild those accepted
installed bytes.

Fresh whole-service sample at 2026-10-07T12:23:28.796641+03:00:
10.000 s; 0.004936 CPU-seconds;
0.0494% of one core. The previous kernel
ceiling remains `cpu.max=25000 100000`. Trainer RSS
65,796 KiB; PSS
31,510 KiB; virtual size
77,032 KiB. Current service cgroup charge is
3,227,648 bytes; it is a separate kernel accounting metric,
not the process RSS. No new cycle errors occurred. IME and daemon PIDs/hashes
still match the before receipt. Generation 174, admitted
updates 156 and pending relations
128 remain; the live base hash is unchanged and the
manifest remains delta-free. Receipts:
`/home/ubu/.cache/lay/l3-proof-offload-20261007/POSTINSTALL.json` and `/home/ubu/.cache/lay/l3-proof-offload-20261007/POSTINSTALL_CHECKS.json`.

### Verdict scope and remaining limits

- Focused trainer: 44/44 PASS. Transport: 10/10 PASS.
- Complete mandatory source/release gate: PASS, 3,014 Rust tests, zero exceptions.
- Real remote full differential and paired frozen evaluator parity: PASS.
- Installed trainer identity, initial idle CPU/RSS, unchanged keyboard runtime:
  PASS for the recorded sample, not a multi-day guarantee.
- Runtime authority changed: false. Publication remains local and retains the
  targeted proof, full differential, all regression checks, admission and compact
  delta-free publication. There is no local full-proof fallback or proof cache.
- Not tested: a new organically triggered live admission after installation,
  long-term cadence/RSS, changed physical-client bytes (none were installed),
  hard-kill/OOM/power-loss and interrupted-transfer cleanup before worker startup.
- The worker must be reachable; failure keeps pending evidence and the last
  admitted model and backs off up to five minutes. Future trainer updates must
  provision the matching worker bytes/configuration; mismatches refuse admission.
- Owned experiment input copies are removed after successful checks. Raw private
  proof inputs are not committed. Failed source/build records remain local for
  provenance; disposable remote source runs can be removed after graph refresh.
