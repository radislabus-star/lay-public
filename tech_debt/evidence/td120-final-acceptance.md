# TD-120 — final scoped acceptance

Date: 2026-09-06. Baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Source: `/home/ubu/projects/lay-tech-debt-20260831`.
Remote source: `e@192.168.3.94:/home/e/projects/lay-td120-121-SUdh2I`.

## Result and exact scope

The canonical changed gate passed for the final TD-120 runtime and test bytes.
The parent re-read the remote results after the connection recovery and compared
SHA-256 identities for every changed Cargo/runtime/contract-test/lane file, all
three added TD-120 IME test files and the composition successor binding. All
matched the local worktree. Generated graph, source binding and architecture
receipt also matched. No functional suite was rerun merely because an agent's
transport failed after successful execution.

The implemented contract is typed `CurrentWord | LegacyReplayV1 | ExactReplay`,
successful-output arming, ordinary-word incarnation retirement, shared-state
consumption and same-lock atomic admission/capture/reconciliation. It removes
the previous word's ordinary veto without granting correction authority.
SafetyGate, verifier, exact replay V2 and daemon-only physical Double Shift are
preserved. The production effect branches have narrow private injection seams;
they do not create alternate runtime decisions or protocols.

The independent [pass-2 review](td120-code-review-pass2.md) records 8/10,
High 0 / Medium 0 for the runtime, including its mechanical repair appendix.
Its original execution-PENDING statement is dated evidence: the successful
canonical run below happened later. Its bounded closing supplement separately
inspects the final two source-contract test amendments; it is not a third full
code-review pass.

## Canonical execution

All builds and tests ran remotely, using the existing resource and Cargo guards:

```sh
cd /home/e/projects/lay-td120-121-SUdh2I
env CARGO_TARGET_DIR=/home/e/projects/lay-td119-gate-v1/target \
  LAY_RESOURCE_PROFILE=dedicated-20cpu CARGO_BUILD_JOBS=20 RUST_TEST_THREADS=1 \
  scripts/lay-resource-guard.sh -- scripts/check-lay-changed.sh
```

Cargo jobs 20 is build parallelism, not 20 in-process test threads. The existing
limits remain CPUQuota 2000%, MemoryHigh 24G, MemoryMax 28G, swap 1G, TasksMax
512 and disposable target budget 12 GiB. No local build/test/training occurred.

Raw receipt root, retained on the remote host:
`/home/e/projects/lay-td119-gate-v1/target/verification-logs/td120-final-20260906T1900Z/`.
The directory's label is not the measured execution timestamp; individual
receipts retain their original timestamps.

| Proof | Result | Receipt |
|---|---|---|
| Canonical correctness | 2518 selected, PASS | `09-results/SUMMARY.json` |
| Canonical package | 36 selected, PASS | Same summary |
| Combined selected cases | **2554/2554 PASS**, known semantic failures 0, infrastructure failures 0 | Same summary; lanes elapsed 348.743 s |
| IME target | 319/319 PASS | `09-check-lay-changed-final.log` |
| Daemon target | 239/239 PASS | Same log |
| Library target | 1757/1757 PASS | Same log |
| TD-113 contracts | 7/7 PASS | `06-focused-contract-tests.log`, then canonical lane |
| Text mutation monopoly contracts | 16/16 PASS | `07-focused-contract-tests.log`, then canonical lane |
| Changed script, including fmt/check | PASS, terminal `== lay changed check OK ==` | `09-check-lay-changed-final.log` |
| Source-bound architecture wrapper | PASS | `08-update-architecture-graph.log` |

Per-target numbers are subsets of the selected lanes, not additional cases.
The discovered manifest contains 2580 tests in 37 targets: correctness 2518,
package 36, performance 11 and ignored 15. The last two categories are **not**
part of the 2554-pass denominator. Fifty TD-120 tests were added; no tests were
removed and no unrelated additions were introduced. `known_failures.json`
remains empty; only its manifest binding changed.

SHA-256 identities:

```text
09-check-lay-changed-final.log
6fd091055a369e0d4b038f6885a635e8785f3cd07a54338df41e6b3f39ad2259
09-results/SUMMARY.json
53e2932a9737e3627fa8190250e0d6017ad2a0bcfb2e433145e08e786ae47858
scripts/test-lanes/manifest.json
10074f804a4df9db3b8433560e2e45d2e20f804312c5d3c09d591f39938d7567
scripts/test-lanes/known_failures.json
80465585e57699c279359ab3dcfed279bd98770110773822d9c67edfcc4d35ac
src/bin/lay_ibus_engine/composition_commit.rs
bfee6bdabc148de1cccf44558d176e3d78c101f42b8912e4ffa25b9de131f713
tests/td113_hybrid_source_contract.rs
0a19679a438ba85e1fef467b73fa92bbe4e30b70314674cab498c543bdbf7fd2
tests/text_mutation_monopoly_contract.rs
3cf3fea05577b38eb0f87a438378d2b7e50816a3bf9baf8c4af53c61c3e6fd6c
tech_debt/evidence/td120-composition-mutation-successor.json
4768f19c1c1afa6651106aa8edd2c8a98996c6198c466d956f995b0ad1be9af0
```

The review includes the remaining runtime/source fingerprints. The successful
source-contract repair preserves the immutable TD-113 preflight and binds one
exact reviewed successor, not a blanket rebaseline. Replay assertions bind
authorization and actual effect order instead of private variable names. The
earlier three-contract RED in `05-check-lay-changed.log` remains retained.

## Denominators that must not be conflated

- Ordinary producer/lifetime, V1 caller/receiver compatibility, V1 known
  residuals, exact replay admission, atomic normal/guard-drift/owner-drift and
  feedback have separate fixtures detailed in the pass-2 review.
- Full-frame O6 covers 6 exact-layout correction frames and 3 protected
  no-correction frames across three safety profiles, plus 3 fresh-guard-absence
  checks. This is production managed-output evidence, not general asynchronous
  Nanda/Wave heldout quality or a physical keyboard result.
- The changed script's unsafe-edit scoreboard used the **remote host's**
  `/home/e/.local/share/lay/recent_actions.jsonl`. It does not verify the local
  user's current typing or desktop client.
- `LegacyReplayV1` delayed/before-only identity limitations remain explicit
  TD-122 debt. Canonical-context handoff is TD-121, not implicitly closed here.
- Performance/ignored lanes, new latency/RSS measurements, local client and
  physical input, release build, installation and process-image checks remain
  separate release work. Installed version at this checkpoint remains 1.0.65.

Only task metadata/evidence and regenerated architecture artifacts may change
between this functional run and the TD-120 commit. Any later TD-121 runtime
change needs its own acceptance and regression evidence. Task completion is
not release 1.0.66 completion; release 1.0.67 quality work remains deferred.
