# Lay 1.0.66 remote exact-V13 sidecar consequence addendum

Status: `REGISTERED_FROZEN_CONTROLLER_PROOF`
Date: 2026-09-06
Scope: frozen release-controller and release-test registration only; runtime
authority is unchanged.

## Known fact and current baseline

The prepared 1.0.66 controller was historically proved as exactly the pinned
1.0.65 controller plus three version substitutions. That remains valid evidence
for the prepared donor artifact, but it is no longer the complete proposed 1.0.66
delta.

The generic binary installer invokes the newly installed
`lay-nanda-wave-train --compile-v13-exact-sidecar` unless
`LAY_SKIP_EXACT_V13_SIDECAR=1`. The prepared controller's `verify_forward` also
invokes the release `lay-nanda-wave-train` to rebuild and compare the sidecar.
Both invocations would compile on the local production host. No local build,
training, sidecar compilation, or test execution is permitted for this release.

The current rollback snapshot owns seven L2 files, including the installed
`LAY-L2-RU-FULL-v13.dafsa`. The compact receipt described below is a transport
and provenance input beside `target/release`; it is not installed into the L2
tree and therefore does not change that seven-file snapshot contract.

## Alternatives, scored 1-10

1. **Prebuilt, strictly admitted remote artifact: 9/10 (selected).** Compile the
   exact sidecar on the leased remote worker with the final release
   `target/release/lay-nanda-wave-train` and canonical package, capture a compact
   receipt, transfer the sidecar and receipt beside the release binaries, and
   admit both before any service stop or byte mutation. This is the smallest
   change that preserves deterministic artifact equality without local work.
2. **Controller-managed remote RPC: 5/10.** Teach the production controller to
   authenticate, lease, upload, execute, and download from the worker. It can
   provide stronger end-to-end orchestration but introduces network credentials,
   remote state, retry policy, and a second release-transaction failure domain.
   That complexity is not justified for a one-release prerequisite.
3. **Permit local compilation: 1/10.** Preserve the current installer and
   verifier. This violates the explicit execution boundary and is rejected even
   though it is mechanically simple.

## Selected delta and provenance boundary

Remote preparation writes two fixed, non-symlink regular files next to the final
release binaries:

- `target/release/LAY-L2-RU-FULL-v13.dafsa`
- `target/release/LAY-L2-RU-FULL-v13.dafsa.release-receipt`

The receipt has one strict schema and records the 1.0.66 release version, the
literal successful compile operation, exit status zero, and name/byte-count/SHA-256
identity for all three material inputs or outputs: the canonical package, final
`lay-nanda-wave-train` compiler binary, and produced source sidecar. The live
controller accepts no expected hash, size, or artifact path from an argument or
environment variable. It parses the fixed adjacent receipt fail-closed, rejects
missing, empty, symlinked, malformed, duplicate, unknown, or mismatched fields,
and independently rehashes/recounts the package, compiler, and sidecar.

The receipt alone is not claimed as independent proof that compilation ran.
Remote execution proof is the guarded command result and exit status captured by
the release harness; the receipt binds the artifact transferred from that run to
the exact final compiler and package. The local controller proves admission and
copy/readback parity only. This separation avoids inferring compilation from
source identity.

## Exact transaction order

1. Before any service stop, extension reload, runtime command, or destination
   mutation, resolve the canonical package and capture package, compiler,
   sidecar, and receipt identities. Require the canonical package name, pinned
   canonical byte count and digest, final compiler regular-file identity and
   executable mode, nonempty regular sidecar, strict receipt fields, and exact
   receipt-to-observation equality.
2. Capture the admitted source sidecar digest and byte count. Immediately before
   copying, re-read both and fail on source drift.
3. Invoke the existing generic installer with
   `LAY_SKIP_EXACT_V13_SIDECAR=1` explicitly. There is no fallback to local
   compilation.
4. Install the already admitted sidecar to the existing L2 destination via
   `install_file_atomic ... 0644`.
5. Verify installed bytes against the admitted source, then independently check
   installed digest, byte count, regular-file/non-symlink status, and mode 0644.
   `verify_forward` repeats those identity/readback checks and never launches
   `lay-nanda-wave-train`.
6. Any later forward failure uses the unchanged atomic rollback route. Snapshot
   tree parity restores the original seven L2 files byte-for-byte and mode-for-mode.

## Consequences required by AGENTS.md

- **Candidate/lattice retention:** unchanged. The canonical package and exact
  sidecar destination and bytes are unchanged; no correction candidate producer,
  lattice limit, or retention rule changes.
- **Ranking and false authority:** unchanged. No runtime score, gate, verifier,
  source ID, or automatic-apply authority changes. A wrongly admitted sidecar
  could affect ranking broadly, so the package/compiler/artifact triple binding
  and readback are mandatory rather than a sidecar-only checksum.
- **Latency deadlines and tail behavior:** unchanged after installation. Release
  preflight adds bounded hashing of large artifacts; runtime hot paths and tail
  latency are not modified.
- **CPU, RSS, and allocation:** local release CPU/RSS decrease because both local
  compiler invocations are removed. Remote preparation bears that bounded cost.
  Runtime allocation and steady-state resources are unchanged.
- **Cache identity and invalidation:** the admitted identity is the tuple of
  release version, canonical package bytes/digest, final compiler bytes/digest,
  and sidecar bytes/digest. Any drift invalidates admission before copy. There is
  no cache and no reuse under a partial tuple.
- **Package and delta reloads:** canonical package resolution and pin remain the
  source of truth. A future package or compiler build invalidates the receipt and
  requires remote recompilation. No delta or online reload route is added.
- **Learning and feedback semantics:** unchanged; no learning state, feedback,
  corpus, or package format is modified.
- **Concurrency and stale-result races:** source digest/size are captured at
  admission and checked again immediately before atomic copy. Installed readback
  detects post-copy mismatch. A concurrent source replacement therefore fails
  closed rather than installing under stale authority.
- **Failure and rollback:** artifact or receipt missing/empty/symlink/drift fails
  before service stop and mutation. Failures after installation enter the
  existing rollback. The receipt stays outside the L2 tree, and rollback retains
  exact seven-file byte/mode restoration.
- **IME and daemon consumers:** global IBus identity, selected-engine transaction,
  daemon/L3/L1.1 lifecycle, and process-parity checks are unchanged. No new
  runtime owner, route, fallback, cache, or source of truth is introduced.
- **Maintenance and removal:** after a release pipeline natively transports a
  signed or content-addressed sidecar bundle, remove the release-local admission
  functions and restore a single generic artifact-install path. Until then, the
  fixed receipt schema is intentionally release-local rather than a generic API.

## What could get worse and invalidation rules

Release staging now requires two additional transferred files and strict receipt
generation. A missing transfer, truncated artifact, compiler rebuild, canonical
package change, or receipt formatting drift aborts the release. Hashing the
140,556,462-byte package adds preflight time. These are accepted fail-closed costs.
The design must be invalidated if the sidecar compiler becomes nondeterministic,
the package contract changes, the release binary is rebuilt after remote
preparation, the sidecar destination leaves the seven-file L2 snapshot, or remote
execution evidence cannot show the exact compiler command returning zero.

## Proof denominators and verdict scope

The fixed isolated harness must separately report:

- historical donor pin: pinned 1.0.65 hash and prepared three-substitution donor
  hash remain checked as historical evidence;
- staged delta: exact allowed additions/removals relative to the prepared 1.0.66
  artifact, with no weakening of donor tests;
- preflight rejection: missing sidecar, empty sidecar, sidecar symlink, malformed
  or mismatched receipt, compiler/package/artifact drift, all before any mocked
  stop or mutation;
- forward behavior: explicit installer skip flag, no local train launch, atomic
  source-to-destination equality, digest/size/mode readback, and unchanged L2
  count seven;
- rollback behavior: injected post-copy failure restores all seven L2 files with
  exact bytes and modes;
- unchanged contracts: global IBus projection and L1.1 lifecycle coverage remain
  in the complete controller harness.

Passing this harness proves the staged controller contract only. The later remote
run must report remote compilation, controller tests, artifact transfer identity,
and live release/rollback evidence as separate denominators. No runtime authority
changes in this addendum, and no 1.0.67 Wave-quality claim is in scope.

## Repair-pass consequence addendum after review pass 1

Review pass 1 scored the stage 8/10 with H0/M2 and is not accepted. This bounded
repair addresses only its two findings.

1. Before Bash `read` parses the receipt, a byte-level gate will require every
   byte to be printable ASCII or LF, require a final LF, and reject CR, NUL, and
   all other control/non-ASCII bytes. The existing fixed schema, duplicate and
   unknown-key rejection, and independently observed package/compiler/sidecar
   identities remain authoritative. This closes Bash's NUL-stripping
   normalization without adding a second parser or caller-controlled bypass.
2. The prior unconditional helper rollback fixture will be replaced by an
   isolated execution of the production forward subshell, `forward_rc`
   dispatch, and existing `run_forward_rollback`. Only external service,
   process, installer, and runtime boundaries will be mocked. A post-copy
   verifier failure must preserve its nonzero status, skip later activation,
   enter the real rollback function, and restore all seven L2 files with exact
   bytes and modes.

Candidate/lattice retention, ranking/false authority, runtime latency and tail
behavior, CPU/RSS/allocation, package and delta reloads, learning/feedback,
concurrency ownership, IME/daemon consumers, global IBus identity, and L1.1
lifecycle are unchanged. The byte scan is bounded to the small release receipt;
it adds no cache or runtime route. Cache identity/invalidation becomes stricter:
malformed receipt bytes invalidate admission before parsing or mutation.
Rollback production behavior is not changed; only its new sidecar failure path
is exercised through the existing dispatcher. Maintenance cost is one small
ASCII-byte predicate and one transaction-specific isolated fixture. A future
receipt encoding beyond printable ASCII plus LF invalidates this schema and
requires a versioned parser, not relaxation. Remote proof denominators added by
this repair are one embedded-NUL pre-mutation rejection and one actual
post-copy-failure dispatch with exit-status, no-later-activation, and 7/7 L2
byte/mode restoration assertions. No local execution is authorized.

## Repair execution receipt, 2026-09-06

Status: `STAGED_CONTROLLER_PROOF_PASS`; runtime authority remains unchanged.

The bounded repair changed only the staged release controller and its isolated
controller harness. The controller now performs its printable-ASCII-or-LF byte
gate with a portable AWK field variable before Bash parses the receipt. The
harness has separate embedded-NUL key and value mutations. Its post-copy fault
fixture executes the production forward subshell and `forward_rc` dispatcher,
uses the real atomic copy and production installed-sidecar verifier, corrupts
the destination only after the real copy has proved source bytes and mode 0644,
and then uses the existing `run_forward_rollback`. External installer,
service, extension, runtime-control, and L1.1 process boundaries are isolated
fixtures. It asserts preserved forward failure status 1, no forward extension
reload or L1.1 transition, explicit no-local-train installer bindings, and
exact byte/mode parity for all seven restored L2 files.

The exact staged source identities at the passing run were:

- controller: `d8a8e8b82fc9ec94f7b985e310f5831684461f536a53c0c68ea4d1e40726129a`
- harness: `05c554b687b7d019047acb86729e75f35757e5db6f9d5e5d41b8cdb06472f4fd`
- transfer tar: `7fe834fff734ea2f4f9abcd4d83e773a767ecfd8ea9a079bf7cde91254a26a44`

The tar was made with `--mtime=now`, then rehashed after extraction into the
independent remote proof tree
`/home/e/projects/lay-1066-controller-proof-pYa18kUp`. Remote-only command:

```text
LAY_RESOURCE_PROFILE=dedicated-20cpu CARGO_BUILD_JOBS=20 RUST_TEST_THREADS=1 \
LAY_L11_SYSTEMD_INTEGRATION=1 scripts/lay-resource-guard.sh -- \
python3 -m unittest -v tests.test_release_live_install_controller \
tests.test_release_l11_process_guard
```

The final guarded rerun is `PASS`: 39 tests passed, 0 failed, 0 skipped, in
3.649 seconds. Raw final log:
`/tmp/lay-release-1.0.66-remote-sidecar-repair-20260906-rerun2.log` on the
remote proof worker. The previous raw log is retained separately as
`/tmp/lay-release-1.0.66-remote-sidecar-repair-20260906.log`; it ran 39 tests
and recorded 5 failure records plus 1 error before the portable-awk and
fixture-directory corrections. Neither run invoked Cargo, compiled a sidecar,
installed a release, or operated live services.

This receipt proves only the staged controller/harness contract, including M1
and M2. Remote artifact compilation, transported release artifact identity,
and live release/rollback evidence remain separate pending denominators.

## Active-worktree frozen-byte registration

The accepted pass-2 bytes were registered unchanged into this active worktree:

- `scripts/install-live-release-1.0.66.sh`: SHA-256
  `d8a8e8b82fc9ec94f7b985e310f5831684461f536a53c0c68ea4d1e40726129a`,
  executable mode `0755`.
- `tests/test_release_live_install_controller.py`: SHA-256
  `05c554b687b7d019047acb86729e75f35757e5db6f9d5e5d41b8cdb06472f4fd`.

This registration makes no internal modification to either accepted artifact.
It relies on the already-recorded remote guarded proof of 32 controller plus 7
L1.1 guard tests (39 pass, 0 fail, 0 skipped); no test, Cargo, service,
installer, graph, version, or Git action was performed for the byte-identical
registration. The pass-2 verdict remains bounded to M1/M2 and grants no
artifact-compilation, transfer, release, or runtime authority.
