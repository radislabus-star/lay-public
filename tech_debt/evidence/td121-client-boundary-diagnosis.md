# Client process exit and replacement-boundary investigation — 2026-09-07

Status: DIAGNOSING, not release or runtime acceptance. TD-124 remains DONE.
Runtime authority changed: false. Production binaries and processes unchanged.

## Two separate observations

1. The portable private IBus client loses `io.github.radislabus_star.LayIme`
   after its first correction boundary. Existing receipt:
   `/home/e/projects/lay-development-runner/run-MXjXRH/client/receipt.json`,
   SHA-256 `e3bfa54bffbf38a7e70168f8f23e7ea393a101a896b8e5d418b11f4609f7e4e7`.
   Completed 0/5. The same candidate previously refused correction with
   `prefetch_not_ready`; that is not evidence for this new process disappearance.
   Existing stderr and kernel/user journal inspection did not explain the exit.
2. The user reports that variable-length autocorrection can eat the separator
   before the corrected word. A frozen production log contains
   `лово → слово ` at Unix timestamp 1788743507: old length 4, planned delete 4,
   replacement length 6 including its trailing Space. This does not support
   the hypothesis that the planner deletes the new word's length. It also does
   not prove what the terminal actually displayed after applying the command.

Private source log snapshot:
`/home/ubu/.cache/lay/replacement-boundary-aPcVzL/`.

- `recent_actions.jsonl`: SHA-256
  `21742c1c3107b92aad8c3ba8dfbd60f78fe26bd75ea9ce40633e363ebab9ffad`;
  75 nonempty lines, 74 valid JSON records.
- `ibus_engine_debug.jsonl`: SHA-256
  `1b785ffd71f3532ea05841dbbf6c89d9562ca8d994230f0ea89355ac3c1b7af2`;
  2739 nonempty lines, 2738 valid JSON records.
- Each rolling log begins with a truncated record; parse per line and report
  that exclusion. Do not copy the user's full text history into Git.
- The two candidate-before-apply rows are different observation points, not
  proof of two actual replacements. Later retained transport rows explicitly
  use `terminal_erase_commit`; the exact earlier transport rows have rotated.

## Route and smallest discriminating observation

Space → old committed token → verified edit → `replace_committed_tail` →
terminal DEL sequence plus CommitText, or a distinct surrounding-text adapter.
The old/new lengths, visible previous token, left separator, trailing boundary,
and cursor position must be asserted independently. A Python client that merely
appends DEL control characters is not a terminal-consumer proof.

Before changing any runtime geometry, capture why the existing private
candidate exits. Use its existing runner and exact candidate/dependencies,
adding only process/signal tracing inside the private namespace. Keep the
frozen driver's five cases, assertions, cleanup and deadlines unchanged.
This is a diagnostic instrumented run, never a latency or release PASS.

Options (engineering judgement, not measured quality):

| Option | Score | Decision |
|---|---:|---|
| Observe child exit/signal with the existing isolated harness | 9/10 | First step: discriminates crash, signal and intentional exit without runtime edits |
| Restore broad inherited HOME mounts speculatively | 4/10 | Changes several variables and weakens isolation before knowing the cause |
| Increase waits or retry the same client until green | 1/10 | Does not establish causality; rejected |

## Consequences and boundaries

The diagnostic launcher lives in the private evidence directory, not a new
runtime module or a second test framework. Existing remote machine validation,
heavy lease, CPU/memory/swap/process limits and 90-second service limit remain.
Candidate and fixture bytes, lexical packages, receipt mounts, scoring,
candidate retention, verifier, SafetyGate, cache identity, reloads, learning,
feedback and mutation ownership remain unchanged. No new fallback or polling
route. Process tracing perturbs timing and process count; an observed deadline
failure is not promoted to the original cause. Trace files contain only private
fixture process/signal events, not production input. Rollback is to use the
unmodified runner without the diagnostic wrapper; no installed state changes.

After exit diagnosis, extend actual-client coverage for shorter/equal/longer
replacements and a preserved left-context sentinel, including multiple spaces
and non-ASCII characters. Prove the actual terminal consumer separately from
surrounding-text behavior. Do not fix a missing separator by unconditionally
inserting a space, reducing every delete count by one, or weakening safety.
No claim yet that the user's visible failure has been reproduced or fixed.

## Process observation and next controlled comparison

One instrumented run used the unchanged runner SHA-256
`8758da61b6a3727b43e9e25f3d1a13b94707f646e2f57cdcb81ce1423a1a3212`:
`/home/e/projects/lay-development-runner/exit-diagnostic-3qM8Nq/client/`.
`process-exit.trace.19` identifies the exact candidate exec and SIGABRT;
`.32` identifies the aborting thread. Receipt SHA-256
`a407c6588d12e0c0792b6bc7b8f626028ff7b53059a135e1d441dbcfc2fe9ae8`.
The client failed 0/5 before cleanup. The process tracer subsequently kept
following the private L1.1 service; the unchanged 90-second service watchdog
cleaned up the namespace. That timeout is an instrumentation consequence,
not the original IME failure and not a product latency measurement.

Source inspection found a missing declared dependency:
`l2/surface.rs::surface_motif_memory` expects the lexical phase artifact.
`lexical_phase/runtime.rs` searches XDG data and the embedded build checkout;
both locations are absent in the portable sandbox. The old broad HOME mount
incidentally exposed the build checkout. Candidate build artifact:
`/home/e/projects/lay-td120-121-SUdh2I/data/lexicon/l2_lexical_phase_v2.bin`,
62,424,748 bytes, SHA-256
`3ff9de4d785aed1b547c56da67dd3cf27644af8968bd0e3bb55a252074cf0268`.
This is a strong hypothesis, not yet a completed causal comparison.

Next diagnostic changes exactly one dependency: bind that hash-checked file
read-only and select it using existing `LAY_L2_LEXICAL_PHASE_MEMORY`. Do not
change candidate/fixture bytes or expose the build checkout. No process tracer
in the comparison, so it cannot keep an auxiliary service alive. A separate
supplement records the added artifact; it does not claim the old eight-role
manifest covered it. Surviving the boundary establishes only the exit cause;
full correction, terminal boundary, latency and release proof remain separate.

Comparison completed: `exit-diagnostic-3qM8Nq/with-lexical/receipt.json`, SHA-256
`04d9da7144339569e138cac5880c34689462f788f418d02d244f27fb2ee2e510`.
The candidate remained alive after Space and until deliberate private daemon
cleanup; bridge snapshots succeeded. Its trace returned `prefetch_not_ready`,
with `space_lookup_wait_us=3535`; correction completed 0/5. Inner service
1.724 seconds, CPU 2.631 seconds. Diagnostic dependency supplement SHA-256
`8407372e5e4f0a5c4b672a76928c0b12d5af9b6af08eea898622619c917044b7`.
This confirms that declaring the missing artifact removes the observed abort
for this exact candidate/environment, not that all dependencies or client
behavior are now complete. No repeated unchanged runs were performed.

Durable fix admitted within this diagnostic scope: add one required
`l2_lexical_phase` role (`l2/l2_lexical_phase_v2.bin`) to the existing manifest
validation and expose it through the existing environment selector. Missing or
wrong-hash input must fail before the private client starts. Use a new nine-file
manifest/config; never modify the historical eight-file manifest or receipts.
This adds no runtime module, owner, cache, timeout or authority bypass. It is
a tooling follow-up, not another TD-121 runtime repair/review round.

An additional static test defect is now explicit: the frozen driver declares
only PREEDIT_TEXT and FOCUS, yet requires DeleteSurroundingText. With width11
and no surrounding support, `CommittedTailOutputProfile` selects TerminalErase.
Its `Client.on_commit` appends DEL literally, not as a terminal line editor.
Do not silently weaken the frozen assertions. A versioned successor needs
separate surrounding-text and real terminal consumers before positive product
acceptance or the user's left-separator reproduction can be claimed.

## Durable runner patch and verification

Runner change: four added lines, one required role and its existing env selector.
Frozen driver unchanged. No Rust runtime source changed in this investigation.

- Remote RED through `python3 scripts/dev-check.py self-test`: local
  `run-f630u1u1`, remote `run-NMU1wu`; 84 tests,4 errors,1 optional skip.
  Errors identify the absent role/env interface, not an executed runtime abort.
  RESULT SHA-256
  `d7de04ca0983119e48e8abb0206442de2254baeb3c36c48221b6151ce99ba399`.
- Remote GREEN, same public command: local `run-9vbafi4m`, remote `run-qSG3Si`;
  85 discovered,84 passed,1 optional real-cgroup skip,0 failures,1.019s unittest.
  Missing file, absent role in a newly hash-pinned old-eight-file manifest,
  and same-size hash corruption fail before output creation or process launch.
  Same-size corruption tests cover both lexical phase and existing L2-v13.
  RESULT SHA-256
  `258ceddacbf5f111b2256cc9fc10617e869a429b9129d05a9e285fdca35b27e0`.
- New immutable dependency directory:
  `/home/e/projects/lay-development-runner/boundary-deps-UjRxy5`.
  Nine-role manifest SHA-256
  `ea6c07b1ddd0d504f87578a39a40b552f301c618195ab0e84df9d9acd2f21c76`.
  Old dependencies/manifests/receipts were not overwritten.
- Initial config attempt `run-21156spd`/`run-RiHRvA` refused before client
  launch: the manifest filename must be `dependency-manifest.json`. Corrected
  the newly prepared config/file naming, not the runner's path validation.
- Final ordinary entrypoint:
  `python3 scripts/dev-check.py client --client-config
  /home/e/projects/lay-development-runner/boundary-deps-UjRxy5/client-config-admitted.json`.
  Local `run-imzbx7hc`, remote `run-FPUQW2`; candidate survives Space,
  cleanup confirms it remains alive until private daemon stop and none remain.
  Inner service1.757s. Client still0/5, explicit `prefetch_not_ready`, observed
  lookup wait3643us. Do not describe this as correction/budget acceptance.
  Client receipt SHA-256
  `94f75fbb4883ddc1c7917122552eaa676eb042acb26db430bae3c428e7cd0be6`.

Final patch identities before architecture refresh:

- `scripts/proof/ime-client/run.py`:
  `56de62c231a56f9449af3ae68704898f4114106a73c550d75678699257d48423`.
- `tests/test_ime_client_harness.py`:
  `624a1e63487f402f66da999355364602cc5a387162b99d131b7985f40cb6cbb4`.
- Unchanged frozen `driver.py`:
  `9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750`.

Independent source diagnosis was performed by Sol/High agents. It is NOT a
fresh-context final code review. Two attempts to start that reviewer were
refused by the session's agent-thread limit. No review score or DONE/commit/push
is claimed for this follow-up. Next session can review the three-file patch
without inheriting implementation context; then continue the explicitly
separate terminal-consumer proof and existing cold-authority investigation.

Production process IDs rechecked unchanged: IBus4715,daemon3453123,IME3453154.
No install, restart, training or local build/test execution occurred.

Architecture refresh route: existing guarded remote
`scripts/update-architecture-graph.sh` from the source-matched development
mirror. Log destination:
`/home/e/projects/lay-development-runner/exit-diagnostic-3qM8Nq/architecture.log`.
The generated architecture receipt and that log own the actual verdict;
AST/document coverage must not be described as product correctness evidence.
