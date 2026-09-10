# Public issues 42–44 and release 1.0.70

Status: mandatory x86_64 gates, independent review and ARM cross-build/smoke
PASS; revised binaries installed and loaded. Publication targets public/main
and v1.0.70; exact completed remote refs belong to the publication receipt.
The user explicitly requested fixing the three GitHub issues and publishing
1.0.70. The previously installed 1.0.70 boundary fix is retained. Its physical
typing check is not silently treated as an observed success; the new request
authorizes publication after the additional fixes and required verification.

## Evidence and first failing owner

- [Issue 42](https://github.com/radislabus-star/lay-public/issues/42): the daemon
  admits ordinary `-`/`=` keys through `is_typing_key`, but the preceding
  `is_leading_non_word_symbol_key` branch starts a token-wide ignore latch.
  This drops the visible punctuation and following letters while retaining the
  preceding completed word. `WordBuffer::what_to_replay(1)` can therefore return
  a different suffix and erase count from the visible text. The first loss is
  buffer admission, before manual projection, output planning or verifier.
- [Issue 43](https://github.com/radislabus-star/lay-public/issues/43): the supplied
  ARM64 log reaches release compilation, contains compiler warnings, and ends
  with explicit Ctrl+C. It does not contain a compiler error or completed build.
- [Issue 44](https://github.com/radislabus-star/lay-public/issues/44): the AMD64
  log ends after compiler warnings and a shell prompt. No error, exit status,
  signal or OOM evidence is supplied. The failed cause is not established by
  this log. Both installer reports use the same source-build stage, which hides
  progress with `--quiet`, builds every research binary, and does not retain a
  build log or explain an unsuccessful return.
- GitHub at task start: exactly these three public issues are open, with no comments;
  the private repository has no pull requests, open issues or commit comments.
  Public `main` is `3bb30c7def7206d364317be1fafa1a82aa61b99c` / 1.0.59.
  The accepted private cleanup branch is `73724fa497a8d8e4b713277d4bf20d7e165bf3c2`;
  pushing that branch alone does not update the public install URL.

Raw issue JSON and complete attachment bytes are private, under
`/home/ubu/.cache/lay/development/github-issues-70-3ps08xs0/`.
`issue-evidence.json` records URLs, hashes, line counts and diagnostic excerpts.
The ARM log is 2089 lines / 79576 bytes; the AMD log is 74 lines / 4438 bytes.
The older warning flood is not an independently established architecture bug.

## Consequence analysis before production edits

Engineering scores compare bounded options, not measured quality or speed.

| Mechanism and route | Score | Consequences |
| --- | --- | --- |
| Keep the token-wide ignore latch | 2/10 | Preserves the reproducible visible-tail mismatch and empty-buffer refusal. |
| Clear the buffer at a leading symbol | 5/10 | Prevents replaying the previous word, but loses the symbol and multi-word scope; requires another special boundary. |
| Keep a separate punctuation exception or shadow tail | 6/10 | Can repair selected prefixes while adding another state/retention path. |
| Remove the obsolete leading-symbol latch; retain shortcut filtering | 9/10 | Selected. Every ordinary typing key follows the same existing buffer route; actual visible punctuation contributes to suffix identity and erase count. |
| Only remove Cargo `--quiet` | 7/10 | Exposes progress, but still compiles unused research tools and loses the failure transcript. |
| Publish a new binary distribution system | 5/10 | Could avoid user compilation, but adds ABI/architecture packaging, release assets and another install route before the reported build failure is known. |
| Build the existing installed binary set with visible progress and retained failure output | 9/10 | Selected. Reuse the current installer as the binary-name owner; preserve feature/dependency/profile choices and existing resource guards, expose the real exit status and log location. |

Issue 42 invariants and second-order effects:

- Candidate retention/ranking: this is manual physical-key projection, with no
  model/ranking call added. Preserve ordinary symbol keys in `WordBuffer`; do
  not special-case issue text, words, suffixes or case IDs. The whole fixed
  restoration proof remains required because newly admitted tokens also reach
  existing automatic typing assistance. Clean CLI/technical-token preservation
  must be checked, not assumed.
- Authority: keep Control/Alt/Meta shortcut filtering, the single daemon-owned
  Double Shift detector, exact replay scope, preedit/committed-tail ownership,
  existing snapshot/lease validation, SafetyGate and output verifier unchanged.
  No second gesture detector or fallback output is added.
- Timing/resources: remove a boolean latch and its save/restore/reset plumbing;
  no timer, RPC, cache, queue, owner, allocation policy or deadline is added.
  Admitted literal keys use existing buffer allocations. Work for formerly
  ignored text can increase; no latency/RSS improvement is claimed from line
  count or state removal. Retain compiled and native proof observations.
- Context/concurrency: delete the latch from both daemon and saved-window state
  so focus restoration cannot revive suppression. Preserve hard-boundary resets,
  pending-assistance cancellation, event counts, replay rearming and stale-result
  rules. Tests cover prefixes after a completed word and at an empty field,
  trailing spaces, shifted symbols, both layouts and shortcut release.
- Learning/packages: no model/package/dependency change or new learning
  permission. Preserve existing learning and automatic-undo reset semantics;
  inspect clean technical-token behavior through existing typing assistance.
- Compatibility/removal: change daemon buffer admission only. GTK, Kitty and
  atomic IME consumers keep their separate transports. Remove the producer and
  entire obsolete latch together; do not leave an always-false compatibility
  predicate or a second source of punctuation ownership.
- Failure/rollback: keep the installed first 1.0.70 binary set until the revised
  release passes. Rollback is that exact saved ten-binary manifest, not a rebuild.

Installer invariants and limits:

- Keep all ten installed binary targets and required `research-tools` feature,
  locked dependencies, release optimization and Cargo disk/CPU constraints.
  Select names from the existing binary installer rather than maintain a second
  divergent list. No daemon, model or quality gate is dropped to shorten a build.
- Preserve successful stdout/stderr and the compiler's progress. Save the build
  transcript under the user's state directory; propagate failure/signal and
  logging failures before any package or binary installation. No polling,
  background progress process, retry-until-success or hidden build fallback.
- This does not establish an OOM, kernel or ARM compiler defect in the supplied
  logs. Reproduce and report source/build/installer outcomes separately. Native
  x86_64 and ARM compatibility evidence must be distinguished; this worker is
  x86_64 and currently has no ARM compiler/emulator.
- Preserve update/stash behavior and current system/runtime installation
  ownership. Do not run a public bootstrap on the user's live desktop merely
  to test a build or mock success on package/model installation.
- Further source inspection found a concrete missing runtime prerequisite:
  `install-l3-context-phase.sh` invokes `jq` three times, while the base system
  package lists omit it. Add that existing prerequisite for supported package
  managers and exercise the declared dependency contract. This is a separate
  demonstrated clean-install defect, not an invented explanation for the
  incomplete ARM/AMD attachment logs.

## Planned proof and publication boundary

First add semantic regression cases against the unmodified production buffer
filter, record their causal failure, then repair the shared admission mechanism.
Tests must assert exact retained events, erase counts, target/reverse surfaces,
previous-word preservation and shortcut/automatic-assistance effects. Replace
the two old tests that explicitly encoded ignored symbols with the corrected
contract; retain their identities or record the manifest transition explicitly.

Exercise the actual installer build entrypoint with controlled Cargo success,
failure, interruption and logging failure, including exact target selection and
absence of later install effects on failure. Retain the existing public-issue,
update-preservation and installed-binary suites. Run focused daemon checks,
full mandatory release checks, the single-owner manual/native controls and
the unchanged fixed restoration proof under the remote resource guard. Obtain
an independent review, with at most two repair/review rounds for this change.

After the final source/bytes pass, install and verify the revised ten binaries,
versions and loaded process hashes while preserving global IBus. Publish the
verified release to the private cleanup branch and the public installation
branch; verify remote commit and public source versions. Audit the concrete
public export before publication so private development history/logs are not
accidentally carried into public history. Do not close an issue on the basis of
a source push alone or claim an unobserved user-machine installation success.

General TD-123 restoration quality, the compound «зуын» -> «push» case, and
the previous formal performance limits remain separate OPEN claims.

## Review round 1 and cancellation consequence check

The independent static review found two medium issues: cancelling the new
builder PID does not forward cancellation to the existing Cargo guard, and
the first clean-token test disabled automatic layout correction and used only
one word. Review: `github-issues-70-3ps08xs0/review-v1/review.md`.

The daemon focused run `run-qolkmuq1` / remote `run-veu42k` passed in 146.503 s,
including the 32-case exact suffix matrix. This does not prove production Nanda
preservation. Extend the existing test to both layout settings and all distinct
context scopes; record the test runtime's Nanda-disabled boundary explicitly.

For builder cancellation, retain the existing guard as the sole Cargo process
group and deadline owner. The logging wrapper must wait on the guard PID and
forward INT/TERM/HUP as TERM to that owner, close its transcript descriptor,
wait for the logger, and return the original cancellation status. A pipeline
without forwarding is rejected because guard and compiler can survive the
wrapper. A second compiler supervisor/timer is rejected because the existing
guard already terminates its Cargo group. A waited guard plus the existing
`tee` logger is selected; it adds no runtime daemon/model/candidate/cache/
learning authority. Failed logging still stops installation. The log descriptor
must close before waiting for `tee`, otherwise EOF and wrapper completion can
deadlock. Controlled tests must signal the wrapper PID with TERM/HUP and its
foreground group with INT, prove compiler and child exit, lease release,
preserved diagnostics and unsuccessful status. No live desktop install is used
for this experiment; failure cleanup kills only the test's private sessions.

The first controlled baseline failed all three cancellation variants in 18.313 s:
each reached the compiler/child readiness event and held the private lease,
then failed to close inherited output within six seconds after cancellation.
The test's `finally` cleanup terminated only those private sessions. Evidence:
`github-issues-70-3ps08xs0/signal-baseline.log`; no production service changed.

The first repaired-builder suite passed seven tests and both PID cancellation
variants, but timed out on group INT. Inspection found a harness condition:
the guarded runner can inherit ignored SIGINT, which Bash cannot restore with
a trap. The group-signal fixture must restore the foreground job's default
SIGINT disposition before exec. This correction changes no production code;
repeat the causal baseline and candidate under the corrected signal contract.
The first INT observation is not proof of terminal Ctrl+C behavior.

With the corrected foreground signal setup, the baseline fails TERM/HUP and
passes INT (12.443 s); the candidate passes all eight installer tests, including
all three cancellation variants, in 1.207 s. Each cancellation verifies the
compiler and child are stopped and the lease is available. Receipts/logs:
`github-issues-70-3ps08xs0/installer-check-v2.json`,
`installer-baseline-v2.log`, `installer-candidate-v2.log` on the remote runner.
The expanded daemon focused run `run-8qm_o5le` / `run-UEgchT` passes in 149.058 s.
The final full run additionally uses the actual configured context bounds 1–3.

## Final source validation and ARM compatibility scope

The frozen full run `run-j4e7w_0n` / remote `run-LKy6vh` passes in 891.255 s.
The explicit changed gate and full gate both execute all 2726 correctness/
package tests (2690 + 36), with zero admitted failures. All 2747 unrelated
manifest rows remain identical; two tests are renamed to the corrected symbol
retention contract and three tests are added. Fifteen ignored and eleven
performance tests are separate excluded denominators. Strict default/research
lint checks pass; dead-code inventory changes are byte locations only.
The actual public builder compiles its ten installed targets successfully.
All 698 Rust source hashes and ten copied release binaries are verified.

Receipt: `run-j4e7w_0n/github-issues-full-acceptance-completed-identity.json`;
both `changed-test-lanes/SUMMARY.json` and `full-test-lanes/SUMMARY.json`, plus
`full-artifact-fetch.json`, reside beside it. The new IME binary is still
`f4d3c8e256ab4f9164007482d828a7e171a9e42770846e9bdd210724d9fd270a`.
Independent review round 2 is 9/10, H/M/L = 0/0/0, static-only, with the exact
thirteen daemon source hashes and installation script hash bound in
`github-issues-70-3ps08xs0/review-v2/verdict.json`. It does not award an
unobserved physical keyboard or Ubuntu installation PASS.

ARM verification uses a separately extracted compiler/sysroot and static QEMU
on the Ubuntu 22.04 x86_64 worker. No host packages are installed and no binfmt
registration or production service is changed. The intended proof is a GNU
Linux ARM64 cross-build through the actual public builder, ELF machine 183,
ten `--help` starts, version, layout conversion and IME XML smoke under QEMU.
It cannot establish native ARM desktop or Ubuntu 26.04 installation behavior.
Use the same remote resource guard and total 12 GiB Cargo target budget.

The first C toolchain probe fails before any Lay ARM build: the extracted
assembler cannot locate its host `libopcodes-2.38-arm64.so`. This is a private
toolchain relocation defect, not a reproduced Lay compiler failure. Keep the
failed receipt under remote `github-issues-70-3ps08xs0/arm-release-proof/`;
the next probe supplies the extracted host-library directory only to the
compiler wrapper. No runtime source or accepted x86_64 artifact changes.

The second ARM probe compiles and executes its C program, installs the pinned
Rust ARM target, then stops with compiler status 101 because the extracted
archiver has the same host-library lookup problem (`libbfd-2.38-arm64.so`).
Receipt: remote `github-issues-70-3ps08xs0/arm-release-proof-v2/`, 21.778 s.
The public builder correctly preserves that exit status and full diagnostics.
Apply the extracted native-library search path to the complete private build
subprocess so both compiler and archiver use the same relocated toolchain;
leave the emulated program environment and system library configuration alone.

The third ARM run successfully builds all ten public targets and executes four
help commands. Its smoke wrapper then incorrectly expects `lay-test-input
--help` to return zero. This binary has a manual scenario parser: every non-list
input, including `--help`, must be refused without `LAY_TEST_INPUT_ARMED=1`.
It correctly exits 1 before creating a keyboard. Preserve that safety refusal;
do not arm synthetic input or change production code to satisfy the harness.
The completion check reuses the successful cross-build and four successful
starts, validates this exact refusal, and runs the remaining five help commands
plus version/layout/XML. Receipt: remote
`github-issues-70-3ps08xs0/arm-release-proof-v3/`, 231.441 s.

## Installation and evidence reuse

The reviewed installation transaction completed on 2026-09-10 at 01:07:29 UTC.
Ten installed binary hashes, CLI and extension version 1.0.70, and all four
loaded process hashes match the accepted manifest. Only `lay-daemon` changed:
`133a1f79e57b34293c496921be40d96d3bc56c5e03ea630d1d8f8939404920d5`.
The other nine binary hashes match the first installed 1.0.70, including IME
`f4d3c8e256ab4f9164007482d828a7e171a9e42770846e9bdd210724d9fd270a`.
The preserved global IBus is PID4715; installed processes at verification are
daemon2729844, IME2729852, L1.1 service2729596 and L3 service2729845.
All eight immutable model files, configuration and input sources are unchanged.
Runtime authority changed only at this installation, not during proof runs.

Receipt: `~/.cache/lay/development/run-j4e7w_0n/installation-1.0.70.json`.
Exact rollback snapshot:
`~/.local/state/lay/release-backups/1.0.70-td123-aeuwv34d/`.
The private installer binds final full/changed/builder PASS, thirteen reviewed
daemon files, the exact installation-script SHA, old native receipts and
current model hashes before mutation, and rechecks loaded bytes afterward.
Its twelve functions and entire mutation/rollback block match the accepted
first-1.0.70 installer byte for byte; there is no installation-time rebuild.

The previous fixed89 strict/normal/experimental results, native13 IBus/Readline
controls, retained-boundary 3/4 -> 4/4 pair and first-word GUI results are reused
for the identical IME/dependency bytes. All Rust outside the thirteen daemon
files matches that accepted source. This is unchanged IME evidence, not a new
measurement of daemon keyboard replay or production Nanda preservation for
newly admitted CLI prefixes. The daemon's fresh proof is the 32-case causal
buffer/replay matrix, shortcut controls, sixty clean-token configuration/
context checks and complete changed/full lanes including four explicit
`physical_double_shift_owner_` tests. Physical keyboard acceptance is PENDING.

## Completed ARM proof and publication boundary

Remote `github-issues-70-3ps08xs0/arm-smoke-completion/receipt.json` records
`PASS_CROSS_BUILD_AND_EMULATED_SMOKE`: all ten binaries are ARM64 ELF machine
183, nine help commands start, the synthetic-input helper preserves its exact
unarmed refusal, and version 1.0.70, `ghbdtn` -> `привет` and both IME XML
engine names pass under static QEMU. The successful cross-build from v3 is
reused without a rebuild; four help starts and the refusal are reused by exact
binary/log hashes. Five remaining help commands and three extra smokes are
fresh. Final smoke completion takes 0.427 s. This remains Ubuntu 22.04 x86_64
cross-compilation/emulation, not a native Ubuntu 26.04 desktop installation.

The public export contains the same 1375 source/document inputs as the frozen
development snapshot, with subsequently verified manifest/graph outputs and
final evidence documentation. Eight local graph query/history paths are
excluded. Its parent is the existing public 1.0.59 commit; private development
commits are not added to public history. Existing archived experiment removals
come from the already accepted cleanup; no active Rust or test source is
removed. The canonical L2 download remains pinned to the existing v1.0.18
asset, so the new source release does not change model identity or introduce
a second binary distribution route.

Exact public/private commits, tag, release URL and live source checks are
recorded after publication in
`~/.cache/lay/development/github-issues-70-3ps08xs0/publication.json`.
Issue 42 has a causal regression fix. For issues 43/44, the installed-target
build, progress/failure diagnostics and missing `jq` prerequisite are repaired
and checked; the causes of the supplied incomplete logs and a successful
installation on the authors' machines remain unobserved. Do not close those
two installation reports solely on cross-build or source-publication evidence.
