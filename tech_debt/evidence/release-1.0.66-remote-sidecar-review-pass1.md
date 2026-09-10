# Lay 1.0.66 remote sidecar: independent review, pass 1

## Findings

**M1 — Receipt parsing silently accepts NUL-corrupted input.**

Staged controller `scripts/install-live-release-1.0.66.sh:218–225` parses the
receipt with `IFS= read -r line`. Bash discards NUL bytes in this mode; the local
`bash(1)` documentation explicitly states, “Other than the case where delim is
the empty string, read ignores any NUL characters in the input.” Consequently,
inserting a NUL inside an otherwise valid key or value, including
`compile_exit_status`, normalizes back to the accepted text. The receipt hash is
captured from the same corrupted file, so subsequent equality with that captured
hash does not reject it. This violates the required strict, malformed-input
rejection boundary. It does **not** establish a bypass of the independent
package/compiler/sidecar hashes or prove a wrong artifact can be installed.

Required correction: validate the receipt bytes as the admitted text encoding
before Bash line parsing, rejecting NUL and other disallowed bytes. Add a remote
negative fixture with an embedded NUL in an otherwise valid receipt and require
failure before mutation. Evidence here is source plus documented Bash semantics;
no local reproducer or test was executed.

**M2 — The added post-copy-failure test does not exercise a failure or the
controller's rollback dispatch.**

Staged harness `tests/test_release_live_install_controller.py:554–558` executes
`install_file_atomic` followed unconditionally by `rollback_files`. No failure is
injected, and neither the production forward subshell nor its `forward_rc`
handling is exercised. This can demonstrate the restoration helper's byte/mode
parity, but cannot satisfy the owning addendum's “injected post-copy failure”
denominator. The adjacent new forward test at lines 480–524 inspects source text
only; it does not dynamically verify the environment received by the generic
installer or that a failing post-copy check reaches rollback before runtime
activation. The retained donor rollback-routing tests remain useful but do not
close this new transaction boundary.

Required correction: extend the isolated remote harness to execute the actual
forward transaction and failure dispatch with mocked runtime boundaries. Observe
the installer skip/input bindings, inject a post-copy failure, require no later
forward activation, and assert the existing rollback route restores all seven
L2 files with their original bytes and modes. Keep the generic installer and
1.0.65 controller immutable.

## Verdict and execution scope

- **Score: 8/10. H = 0; M = 2.** The staged delta needs the two corrections above
  before its readiness claim is restored. No critical runtime defect was found
  in the reviewed sidecar copy or checksum binding itself.
- **Execution: PENDING.** This pass executed no tests, Cargo, installer,
  production binary, input operation, service command, or remote compilation.
  It grants no coding, installation, or production authority.
- Review scope is the new sidecar delta relative to the historical prepared
  1.0.66 artifact, not inherited donor architecture. The earlier three-version-
  substitution proof remains historical evidence and is not rewritten by this
  verdict.

## Verified source evidence

The controller delta is 162 added and 18 removed lines; the harness delta is 392
added and 5 removed lines. Static method counts are 26 before and 32 after; these
are source counts, not executed-test totals or a replacement for historical
combined-suite counts.

The new controller independently checks the canonical package pin, final
compiler, sidecar, and receipt before the forward transaction. Fixed adjacent
artifact/receipt paths and initialized expected values prevent caller-supplied
artifact identity from replacing those checks. Hash and size revalidation occurs
before the generic installer and again before the sidecar copy.

The generic installer receives `LAY_SKIP_EXACT_V13_SIDECAR=1` together with the
admitted package and explicit source/destination bindings. Its unchanged
implementation gates the compiler invocation on that flag. The controller's old
verification-time compiler invocation is removed, with no local compilation
fallback introduced. Atomic copy is followed by byte equality, regular-file and
non-symlink checks, digest, size, and mode 0644 readback; forward verification
repeats them and checks that the L2 file count remains seven.

The diff does not modify global IBus identity handling, the L1.1 lifecycle,
snapshot/rollback helpers, or the existing forward failure dispatcher. Git
inspection found no changes to the pinned 1.0.65 controller or generic installer.

The receipt binds transported bytes to package/compiler/artifact identities. It
does **not** independently prove remote execution. The release owner must still
inspect the guarded remote compile command, zero exit result, and log, then bind
the transferred files to that run. Compilation, controller tests, artifact
transfer, and live release/rollback remain separate pending denominators.

## Exact reviewed identities

Staged root: `/tmp/lay-release-1.0.66-remote-sidecar.Ppvoyt`.

| Artifact | SHA-256 |
| --- | --- |
| Staged controller | `2198b20a0050b2874fff8174e5b6d67720831b063129196ea5237c17ff43846a` |
| Staged harness | `1d6debf7e1fb861753c50941809b91eef49feaa80614ba9335635b85d46d8fc3` |
| Prepared controller, `/tmp/lay-release-1.0.66-controller-prep.5funTP` | `abe1d7f1b35bfc6b7456699326bb6137675bfd5c8eafe2e7c52fd034f0255ae9` |
| Prepared harness, same root | `2b98fc44a5654e92d32ae115f5e799daeb50aa034a609bc7df7081bce2727d3f` |
| Worktree `scripts/install-live-release-1.0.65.sh` | `962f4b7500758cf1cf421e72695b10695d5b49d29e696782bd4060144ea96e66` |
| Worktree `scripts/install-release-binaries.sh` | `3427ae85155f073cfaacf37b09eb901defb57ee8912977ad31b786d928b35c82` |
| Owning `release-1.0.66-remote-sidecar.md` at review | `447271b7cb1387d6da8917f303255e0b0629405773582deefe8a47b8b8bb820d` |

## Review method

Read the worktree `AGENTS.md`, owning consequence addendum, graphify skill, full
controller/harness delta, and relevant unchanged installer/resolver contracts.
The read-only graph query used observed vocabulary `release rollback receipt
install` with a 1400-token cap; it returned broad donor context and does not
contain the staged temporary artifact, so findings use current source lines.
No graph update or saved query was written under this read-only review scope.
Only this review report was created. Date: 2026-09-06.
