# Lay 1.0.66 remote sidecar: independent review, pass 2

Verdict: `PASS` for the bounded M1/M2 repair. **Score: 9/10; H = 0; M = 0.**
Date: 2026-09-06. Both pass-1 findings are closed for the exact staged identities
below. This is the second and final bounded review pass, not release acceptance.

## Scope and method

Reviewed only strict receipt-byte admission (M1) and the new sidecar post-copy
failure through the production rollback dispatcher (M2), against
`release-1.0.66-remote-sidecar.md` and `release-1.0.66-remote-sidecar-review-pass1.md`.
The prepared baseline remains `/tmp/lay-release-1.0.66-controller-prep.5funTP`;
the reviewed stage is `/tmp/lay-release-1.0.66-remote-sidecar.Ppvoyt`.
Controller and harness line references below are relative to that staged root.
Inherited donor architecture, TD-120, TD-121, and 1.0.67 quality are outside scope.

Read worktree/ancestor AGENTS instructions, the graphify skill and query reference,
then ran read-only `graphify query "install release manifest rollback" --budget 1100`
using observed graph vocabulary. Its broad donor context does not establish the
temporary staged implementation; the findings below use direct source inspection.
Reviewed the controller delta, targeted harness/rollback source, and remote logs;
independently calculated local/remote SHA-256 identities and log test counts.
No tests, builds, compilers, installers, production binaries, service operations,
Git mutations, graph refresh/save-result, or runtime edits were executed by this
review. Only this report was written; runtime authority is unchanged.

## M1: CLOSED — reject malformed bytes before Bash parsing

`scripts/install-live-release-1.0.66.sh:205-222` requires a nonempty regular,
non-symlink receipt with final byte LF. Its `od -An -v -tu1` stream preserves
every byte numerically; the AWK predicate admits only byte 10 or bytes 32-126.
NUL, CR, other controls, DEL, and non-ASCII bytes therefore fail this predicate.
The non-reserved AWK field variable avoids the failed run's portability defect.
The controller enables `pipefail`, so read/scan failures cannot be hidden by the
last pipeline command. The gate is called at line 237, before `IFS= read -r`
at line 238. The fixed required fields and independent artifact identities remain
in place. Production admission exits at lines 1355-1358 before the subsequent
service/engine mutation routes.

`tests/test_release_live_install_controller.py:464-490` introduces two distinct
negative subtests: NUL inside `compile_exit_status` and NUL in its value. Both
mutate an otherwise valid receipt, call the production admission helpers, require
nonzero completion, and assert that the subsequent mutation marker is absent.
The successful exact-admission/copy test at lines 368-382 prevents an always-reject
implementation from satisfying this evidence. The inspected remote log reports
both enclosing test methods as passing. These NUL cases are subtests within one
method, not two additional members of the 39-test denominator. Rejection of the
other disallowed byte classes and missing final LF is established here by source
inspection; this pass does not claim dedicated executed fixtures for every byte.

## M2: CLOSED — real post-copy failure enters production rollback

`tests/test_release_live_install_controller.py:164-195` extracts the actual
`run_forward_rollback`, forward subshell plus `forward_rc` dispatcher, release
file helpers, and installed-sidecar verifier from the pinned staged controller.
Only the verifier function's name is changed to retain its production body beside
the fault-injection wrapper. The fixture at lines 556-715 uses those extracted
functions, with installer, service, engine, extension, and process boundaries
mocked inside temporary paths.

The production forward subshell (`scripts/install-live-release-1.0.66.sh:1424-1452`)
performs source revalidation, invokes the mock generic installer, and executes the
real atomic sidecar copy. The verifier wrapper first proves exact copied bytes and
mode 0644 and records a copy marker; only then does it corrupt the destination
and call the unchanged production verifier body (`tests/...:648-654`). That body
fails on the actual byte mismatch. The subshell's `errexit` exits before the next
L1.1 transition; the actual dispatcher at controller lines 1453-1466 passes the
failure to the actual `run_forward_rollback` at lines 1191-1279. Its real
`rollback_files` and `restore_tree_atomic` restore and compare the L2 tree.

Assertions at harness lines 692-715 require exit status **1**, the successful-copy
marker, all ten observed installer environment values (including skip-sidecar=1,
offline=1, package identity and source/destination paths), absence of the later
forward extension reload and L1.1 transition, exactly seven L2 files, and equality
of every relative path, byte sequence, and mode to the snapshot. The sidecar's
snapshot mode is 0640, so this also detects failure to restore its original mode
after forward installation at 0644. Only the sidecar is deliberately corrupted;
the asserted denominator is whole-tree 7/7 parity after that fault, not seven
independently injected corruption cases. This closes the unconditional-helper
test gap identified in pass 1.

## Independently verified execution evidence and identities

Read-only SSH target: `e@192.168.3.94`. Remote proof tree:
`/home/e/projects/lay-1066-controller-proof-pYa18kUp`.
Remote raw log `/tmp/lay-release-1.0.66-remote-sidecar-repair-20260906-rerun2.log`
contains the active resource-guard header, **32 controller + 7 L1.1 guard = 39
passing test methods, 0 failures, 0 skips**, followed by `Ran 39 tests in 3.649s`
and `OK`. No test was rerun during this review.

| Artifact | Independently observed SHA-256 |
| --- | --- |
| Staged controller; identical remote proof copy | `d8a8e8b82fc9ec94f7b985e310f5831684461f536a53c0c68ea4d1e40726129a` |
| Staged harness; identical remote proof copy | `05c554b687b7d019047acb86729e75f35757e5db6f9d5e5d41b8cdb06472f4fd` |
| Final remote passing log | `d61d7a630622b8833e36ccdd8c7267c872302c65141ee464bab401daa522ad7d` |
| `/tmp/lay-release-1.0.66-remote-sidecar-repair-20260906.tar`, local and remote | `7fe834fff734ea2f4f9abcd4d83e773a767ecfd8ea9a079bf7cde91254a26a44` |
| Prepared baseline controller | `abe1d7f1b35bfc6b7456699326bb6137675bfd5c8eafe2e7c52fd034f0255ae9` |
| Prepared baseline harness | `2b98fc44a5654e92d32ae115f5e799daeb50aa034a609bc7df7081bce2727d3f` |
| Pinned 1.0.65 controller, worktree and remote proof copy | `962f4b7500758cf1cf421e72695b10695d5b49d29e696782bd4060144ea96e66` |
| Generic installer, worktree and remote proof copy | `3427ae85155f073cfaacf37b09eb901defb57ee8912977ad31b786d928b35c82` |
| Owning sidecar addendum at review | `7a8df6cd2b61b8ee384402053e2ae58fef17a9d4d22758891ed5bfdbafaa92e2` |
| Pass-1 review | `a8fd0af3417919d9c417f073741e16368b767c9575a0d6ddc79ecd292d73f435` |

The previous remote log remains at
`/tmp/lay-release-1.0.66-remote-sidecar-repair-20260906.log`, SHA-256
`8ab6032544d32da2a8b419477005bdd9422908fb6919e472e9501db9f9133d04`.
It records 39 methods, five failure records and one error; it has not been
presented as a passing run. The current controller is 182 added/18 removed lines
against the prepared baseline. Read-only Git comparison found no worktree changes
against HEAD for the pinned 1.0.65 controller or generic installer.

## Verdict boundary

The remote result proves the isolated staged controller/harness contract and
closes M1/M2. The fixture compiler is a script returning 97, its package and
sidecar are small synthetic byte strings, and its receipt is fixture-generated
(`tests/test_release_live_install_controller.py:209-275`). Neither that receipt
nor the 39 passing methods proves execution of the real sidecar compiler.

Real remote artifact compilation and its zero exit evidence, identity of the
transported release package/compiler/sidecar, and production install/rollback
verification remain separate **PENDING** denominators. Matching controller-source
transfer hashes do not close those artifact-transfer or runtime denominators.
No remaining H/M finding was identified within the two-finding repair scope;
this verdict grants no live release or runtime authority.
