# Lay 1.0.66 controller adaptation: independent bounded review

Date: 2026-09-06

Verdict: PASS for the prepared version-only controller adaptation.
Score: 9/10. Findings: High 0, Medium 0, Low 0.
No blocking defect found in the reviewed delta. This verdict is NOT permission
to install and is NOT a full release PASS or a TD-120/TD-121 runtime verdict.

## Exact scope and artifact binding

Worktree: `/home/ubu/projects/lay-tech-debt-20260831`.
Staging root: `/tmp/lay-release-1.0.66-controller-prep.5funTP`.

Read scope:

- Worktree `AGENTS.md` and the complete
  `tech_debt/evidence/release-1.0.66-controller-preparation.md`.
- Complete staged `tests/test_release_live_install_controller.py`, lines
  1-1685, and its complete diff against the tracked current harness.
- Complete byte comparison and diff of staged
  `scripts/install-live-release-1.0.66.sh` against the tracked 1.0.65 donor.
  Donor semantic sections read: lines 1-340 and 648-1302; the remaining
  lifecycle section was checked through unchanged-byte identity and its
  harness coverage, not independently re-audited as new production code.
- Extraction boundary locations across the donor; current runtime-controller
  hash; donor hash from both the worktree and `git show HEAD:`; scoped Git
  status/diff. No Git write was performed.
- Read-only Graphify query `release rollback snapshot` under the repository
  instructions. Its broad results supplied no additional adaptation evidence;
  the verdict uses direct source comparison. No graph output was written.
- Remote proof file hashes, terminal log excerpts, and per-suite success counts
  over read-only SSH to `e@192.168.3.94`.

Independently verified SHA-256 values:

| Artifact | SHA-256 |
| --- | --- |
| Tracked donor `scripts/install-live-release-1.0.65.sh` | `962f4b7500758cf1cf421e72695b10695d5b49d29e696782bd4060144ea96e66` |
| Staged `scripts/install-live-release-1.0.66.sh` | `abe1d7f1b35bfc6b7456699326bb6137675bfd5c8eafe2e7c52fd034f0255ae9` |
| Staged `tests/test_release_live_install_controller.py` | `2b98fc44a5654e92d32ae115f5e799daeb50aa034a609bc7df7081bce2727d3f` |
| Tracked `scripts/lay-runtime-control.sh` | `746d077115277775acb275e077b57823ace9e416d2575eaf0bc53d3327d64e6d` |

The remote donor, new controller, and harness hashes match those exact local
artifacts. The staged new controller has mode `0755`. The tracked donor and
current tracked harness are unchanged at review time.

## Findings and reasoning

The controller diff contains exactly the three admitted substitutions:
`EXPECTED_VERSION=1.0.65` to `1.0.66`, `ROLLBACK_VERSION=1.0.64` to `1.0.65`,
and snapshot prefix `1.0.65-preinstall-` to `1.0.66-preinstall-`.
Independently applying these substitutions to the donor and comparing the
result with `cmp` yields identical bytes. No additional controller meaning is
introduced by this adaptation.

The new harness binds `CONTROLLER` to 1.0.66 and `DONOR_CONTROLLER` to 1.0.65
(lines 18-25). The strengthened delta test checks both fixed hashes, exactly
one occurrence of each donor admission, the reconstructed expected source,
new admission counts, and executable access (lines 167-191). The historical
1.0.65 artifact remains protected by its fixed donor hash.

Every dynamic source extractor reads `CONTROLLER`; none obtains executable
functions from `DONOR_CONTROLLER`. Their boundaries and dependency stubs are
unchanged from the current harness. The substitutions in lines 173-179 build
only the expected comparison value; they do not rewrite the controller under
test. Existing later `harness.replace` calls vary isolated scenarios rather
than redirecting tests to a historical controller. Direct self-test and
snapshot-fault invocations also execute 1.0.66.

The retained tests include real isolated file restoration and byte/mode
comparison, snapshot rejection before mutation, engine recovery blocked by
mixed bytes or IBus identity drift, exact process identity and argv checks,
package/tree fingerprint drift, cleanup uncertainty, and rollback ordering.
The forward rollback function remains extracted from production source and
preserves the failed forward exit status. Process stop must precede byte
restoration, and byte/projection verification must precede reactivation.

Some checks inspect source strings, and some function tests replace external
dependencies with stubs. Their claims remain confined to those checks and
isolated function behavior; they do not establish a complete real-desktop
installation or rollback. This distinction is preserved in the verdict.

## Independently inspected remote proof

Remote proof root:
`/home/e/projects/lay-1066-controller-proof-pYa18kUp`.

- Initial `controller-tests.log`: 33 tests, 32 passed, 1 pre-existing opt-in
  systemd integration skip, 2.171 seconds.
- Completed `controller-tests-integration.log`: 26/26 controller tests plus
  7/7 L1.1 process-guard tests passed, 33/33 total, zero skipped, 2.469 seconds.
- The integration log explicitly records
  `test_l11_spawn_survives_controller_parent_exit ... ok`.
- Integration log SHA-256:
  `f7d08f9c211f1bedd7d878ef84daf896ec1511a7e1f00f7d2dc45910193322f2`.
- The log records the active `dedicated-20cpu` resource guard, 20 jobs,
  one test thread, CPU quota 2000%, and the remote verification lease.

The owning agent executed both suites; this reviewer only inspected their
artifacts. The 33-test denominator includes seven separate process-guard
tests and must not be represented as 33 controller test methods.

## Limits and authority

This review neither ran tests or Cargo nor invoked services, live input,
installation, metadata changes, commits, pushes, graph rebuilds, or subagents.
Only this review receipt was written. Runtime authority did not change.

The preparation note is an earlier pre-execution snapshot; the remote results
above are later evidence. Release binaries, package/runtime compatibility,
full TD-120/TD-121 behavior, actual desktop forward installation and rollback,
and release publication were outside this review. The 9/10 score applies only
to adaptation correctness and retained bounded verification, not to those
unmeasured release properties.
