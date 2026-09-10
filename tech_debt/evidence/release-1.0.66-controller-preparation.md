# Lay 1.0.66 live-install controller preparation

Date: 2026-09-06

Status: REMOTE_REGRESSION_PASS, staged preparation only. The controller has
passed remote regressions but has not been registered or used for a live
install. This document does not admit or declare release 1.0.66 ready.

## Consequence analysis recorded before script editing

### Facts

- The proven historical donor is
  `scripts/install-live-release-1.0.65.sh`, SHA-256
  `962f4b7500758cf1cf421e72695b10695d5b49d29e696782bd4060144ea96e66`.
- The donor has exactly three release-specific admission values relevant to
  the next controller: `EXPECTED_VERSION=1.0.65`,
  `ROLLBACK_VERSION=1.0.64`, and the exact allowed snapshot prefix
  `1.0.65-preinstall-`.
- The 1.0.66 preparation is bounded to a new controller, its existing Python
  regression harness, and this evidence note. Cargo/version metadata, Rust
  sources, services, live input, configuration, installation, and release
  publication are outside this preparation.

### Chosen route and exact semantic delta

Create `scripts/install-live-release-1.0.66.sh` as an exact donor copy with
only these substitutions:

1. `EXPECTED_VERSION=1.0.65` becomes `EXPECTED_VERSION=1.0.66`.
2. `ROLLBACK_VERSION=1.0.64` becomes `ROLLBACK_VERSION=1.0.65`.
3. The admitted snapshot prefix `1.0.65-preinstall-` becomes
   `1.0.66-preinstall-`.

The regression harness will pin both donor and prepared-controller hashes and
will reconstruct the expected 1.0.66 bytes from the donor using precisely
those three substitutions. Any generic controller rewrite or additional
semantic drift must therefore fail the scoped-delta assertion.

### Invariants and second-order consequences

- Candidate/lattice retention, ranking, false authority, learning, and
  feedback semantics are unchanged: the controller neither generates nor
  ranks correction candidates, and no runtime/model code changes here.
- Latency deadlines and tail behavior are unchanged: all timeouts, retry
  bounds, readiness checks, and tail/runtime behavior remain donor-identical.
- CPU, RSS, and allocation behavior are unchanged apart from comparing new
  version strings of the same size. No cache is added; cache identity and
  invalidation are unchanged.
- Package/delta reload behavior and source provenance are unchanged. The same
  release-tree, extension, L2, L3-unit, and L1.1 provenance checks remain in
  force.
- Concurrency and stale-result defenses are unchanged. The donor's captured
  process identities, hashes, argv, tree fingerprints, global IBus identity,
  and process-drift gates remain byte-identical.
- Failure behavior remains fail-closed. The rollback boundary advances only
  from installed 1.0.64 to installed 1.0.65, while the rollback mechanisms,
  ordering, verification, and exit behavior remain unchanged.
- IME and daemon compatibility contracts are unchanged. In particular, this
  preparation introduces no global IBus restart, no second input owner or
  mutation route, no fallback, no cache, and no source of truth.
- Long-term maintenance/removal cost is one versioned controller and three
  release-specific harness admissions. It is removable with the normal
  historical-controller retention policy and does not create a generic
  abstraction that later releases must maintain.

What can get worse is narrowly bounded to wrong release admission: a mistaken
version, rollback baseline, or snapshot prefix could admit the wrong tree or
reject the intended one. Exact donor/controller hashes, exact substitution
counts, snapshot fault tests, and remote execution of the full regression
harness are the controls. Future packages or online updates do not change this
conclusion unless they change controller structure; such a change must not be
smuggled into this version-only preparation.

### Alternatives considered

1. Selected: preserve every donor byte except the three release admissions.
   This retains the already-proven transaction, rollback, IBus, L1.1, process,
   and provenance behavior and gives the smallest review surface.
2. Parameterize one generic controller. Rejected for this preparation because
   it would rewrite controller ownership and admission semantics, enlarge the
   proof surface, and weaken the historical byte-pinned artifact boundary.
3. Copy the donor and update only `EXPECTED_VERSION`. Rejected because rollback
   would target 1.0.64 and the controller would admit 1.0.65 snapshot paths,
   contradicting the intended 1.0.66 transaction boundary.

### Hypotheses, estimates, and unverified assumptions

- Hypothesis: remote regression execution will preserve the donor's existing
  behavior because the executable logic is unchanged outside the three string
  admissions.
- Estimate: runtime cost is unchanged to observable precision; no runtime
  measurement is part of this preparation.
- Unverified: the 1.0.66 release artifacts, metadata, remote test result, live
  baseline, and installation state. No release-readiness claim is authorized.

### Proof denominators and deferred work

- Static preparation denominator: exact donor SHA, exact new-controller SHA,
  exactly three admitted substitutions, executable mode, shell parse, Python
  parse, and scoped diff inspection.
- Behavioral denominator: the full
  `tests/test_release_live_install_controller.py` suite, including its opt-in
  systemd integration case where the remote environment supports it.
- Tests and all live/remote authority are deferred to the agent holding the
  remote lease. The prepared remote command is:

  ```bash
  python3 -m unittest -v tests/test_release_live_install_controller.py
  ```

- The architecture graph refresh is deferred to the owning integration phase:
  this bounded task cannot modify the already-dirty shared graph artifacts
  outside its three-file ownership.

## Preparation result

The controller and harness are staged outside the repository until TD-120 has
its own source-bound graph and commit:

- `/tmp/lay-release-1.0.66-controller-prep.5funTP/scripts/install-live-release-1.0.66.sh`
  - SHA-256:
    `abe1d7f1b35bfc6b7456699326bb6137675bfd5c8eafe2e7c52fd034f0255ae9`
  - mode: `0755`
- `/tmp/lay-release-1.0.66-controller-prep.5funTP/tests/test_release_live_install_controller.py`
  - SHA-256:
    `2b98fc44a5654e92d32ae115f5e799daeb50aa034a609bc7df7081bce2727d3f`

Static checks performed locally:

- donor SHA-256 rechecked and unchanged: PASS;
- controller diff against 1.0.65 contains exactly the three declared release
  admissions: PASS;
- `bash -n` on the staged controller: PASS;
- Python AST parse of the staged harness: PASS;
- tracked `scripts/install-live-release-1.0.65.sh` remains byte-identical;
- tracked `tests/test_release_live_install_controller.py` remains untouched.

At the preparation checkpoint, no tests, remote work, services, installation,
version bump or Git writes had run. The later parent-run result is recorded
below; it does not change that historical scope.

## Parent remote execution — 2026-09-06

The exact staged controller and harness were overlaid on a separate remote
proof tree, not the TD-120 checkout or installed production tree:
`e@192.168.3.94:/home/e/projects/lay-1066-controller-proof-pYa18kUp`.

Command from that tree:

```sh
env LAY_RESOURCE_PROFILE=dedicated-20cpu CARGO_BUILD_JOBS=20 \
  RUST_TEST_THREADS=1 LAY_L11_SYSTEMD_INTEGRATION=1 \
  scripts/lay-resource-guard.sh -- python3 -m unittest -v \
  tests.test_release_live_install_controller tests.test_release_l11_process_guard
```

Result: **33/33 PASS**, comprising 26 controller tests and 7 L1.1 process-guard
tests, zero skips, 2.469 seconds. The first run without the integration opt-in
had 32 PASS / 1 skipped, 2.171 seconds; it is preserved as `controller-tests.log`.
Final log: `controller-tests-integration.log`, SHA-256
`f7d08f9c211f1bedd7d878ef84daf896ec1511a7e1f00f7d2dc45910193322f2`.

The opt-in case creates only its own temporary remote bash/systemd service,
verifies survival after its launching parent exits, and stops it in cleanup.
Post-run process/unit inspection found no remaining fixture process or running
`lay-l11-release-*.service` on the remote host. No working desktop IBus, actual
Lay service, installed binary, package or user configuration was changed.

Independent fresh-context review: 9/10, High 0 / Medium 0, scope restricted to
the version-only adaptation and regression harness. See the separate
`release-1.0.66-controller-review.md`; neither record is a full release proof.
The two files remain staged outside the working source until TD-120's checkpoint.
