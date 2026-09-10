# TD-124 execution evidence

Scope: reproducible maintenance tooling, not product restoration, full-release
acceptance, installation or production restart. Runtime authority changed:
false. Source worktree has unfinished TD-121 edits; the task-only Git change
must exclude them. Every run request owns its complete source snapshot.

## Public command and resources

All execution below used `python3 scripts/dev-check.py ...` from
`/home/ubu/projects/lay-tech-debt-20260831` and the configured SSH worker
`e@192.168.3.94`. Local activity was source inspection/snapshot transfer only.
Heavy execution: existing host lease, dedicated-20cpu, jobs20, Rust test
threads1, CPU2000%, MemoryHigh24G/Max28G, swap1G, Tasks512, target<=12GiB.
Private client retained CPU200%,1536M,swap0,Tasks128,90s.

The actual worker is Python3.10. Initial implementation was adjusted before
acceptance to use streaming SHA-256 and conservative Cargo-layout inspection,
not Python3.11-only APIs or a new TOML dependency.

## Tooling tests

- Initial root-runner RED: absent implementation is an import failure, not a
  semantic regression claim. Remote `td124-tool-tests.6nNQvg/red.log`.
- Root runner GREEN16/16; after plan-binding repair19/19 in0.013s.
- Public aggregate after first correction:82 discovered,81passed,1 explicit
  optional real-cgroup integration skip,0failures, unittest1.023s.
  Local `run-pfo3ti2a/run.log` SHA
  `543366e163b5720a67b650d6187ed4da593228a9f8d727e583f977749710d5ef`.
  `RESULT.json` SHA
  `2007690d831937188dd0bf03068bd8805d4abf43ab09da5963cb5df3cd0f0004`.
  Full command25.661s, not1.023s: snapshot/transfer/setup are separate costs.
- Final PID observer RED: bwrap and Python argument matches both fail the
  new negative assertion. GREEN12/12 harness tests after the exact-token fix.

Local run prefix: `/home/ubu/.cache/lay/development/`.
Remote accepted run prefix: `/home/e/projects/lay-development-runner/`.

## Actual focused Rust check

Command: `python3 scripts/dev-check.py check --target bin:lay-ibus-engine`.
This is an explicitly scoped inner-loop check, NOT automatic dependency-closure
or all-integration acceptance.

Initial attempt: `run-a8neu9el` / remote cache `run-8zOYji`. Cargo compilation
21.62s; harness could not run because `.cache/lay` was masked by existing
isolation. Retained as FAIL. Repair moved the owned mirror/results into
`projects/lay-development-runner`; the sandbox was not weakened.

Accepted first run: local `run-j_xprutp`, remote `run-9xiiLg`:

- Discovered399, selected396, executed396, passed396, failed0.
- Excluded3 performance tests,0ignored; exact names in discovered manifest.
- Canonical drift reported, canonical manifest unchanged; no known failures
  tolerated by the development route.
- Discovery/build21.898s, execution5.280s, focused total28.113s.
- Remote sequence including fmt31.219s; full workstation-to-result55.461s.
- Remote `tests/SUMMARY.json` SHA
  `2b58cbe6546447620502e81d9433875e2d68895b68b38f3a16c15d738753351e`.

A repeated unchanged-Rust run measures warm cache reuse separately; do not
compare its time to the old399-test run (different exclusions/environment).

## Actual repository-owned client

Command: `python3 scripts/dev-check.py client --client-config
/home/e/.cache/lay/td124-client-config.json`.
Candidate release binary SHA:
`6c480f399c62e0979b409c031c7669d74296a4c78be94082904d3cc163154ba3`.
Dependency manifest SHA:
`7f7343b6b3ffa8b7cfc924fd5c556e86f1073b677f6bc1fcd87b79e97b0b51d5`.

First migrated run: local `run-2iv2hyxg`, remote `run-2zS5rT`. The real client
preserved ` l -> ljv` with its leading boundary and emitted literal Space, but
the following bridge query failed `NameHasNoOwner`. The inherited observer
incorrectly included the bwrap wrapper in its cleanup report. No success was
claimed; that false process identity was repaired with RED/GREEN evidence.

After observer correction: local `run-530ufbic`, remote `run-MXjXRH`:

- Client status FAILED,0/5 completed, D-Bus `NameHasNoOwner` after Space.
- Private daemon reaped; remaining candidate process list empty.
- Receipt SHA
  `e3bfa54bffbf38a7e70168f8f23e7ea393a101a896b8e5d418b11f4609f7e4e7`.
- Driver0.406s, inner service471ms, wrapper child0.732s.
- Run uses private namespaces; no installed binary or global IBus changed.

This confirms the repository entrypoint executes and propagates failure, with
separate unit/parity/isolation evidence. It is NOT five passing client cases.
Cause of bridge disappearance is UNKNOWN: neither runtime nor changed harness
environment is exonerated by the identical driver scenario. Do not label this
as the old `prefetch_not_ready` without new trace evidence. TD-121 acceptance
and physical keyboard proof remain OPEN. No sleeps, retries-until-green or
runtime patch were introduced to change this verdict.

## Architecture / completion

Warm-cache run: local `run-b95w_1fk`, remote `run-QUdgAh`, same399 discovered /
396 executed and passed /3 performance exclusions. Discovery0.337s,
execution5.313s, focused total6.606s, remote sequence9.693s, complete command
33.246s. Rust input did not change; the stable owned source path reuses Cargo
artifacts. First-vs-warm timings are not an old-vs-new workflow benchmark.

Final remote gate command (inside the same existing guarded profile):
`scripts/check-lay-tests.sh self-test && scripts/update-architecture-graph.sh`.
Executed from `/home/e/projects/lay-development-runner/workspace`.

- Final tooling aggregate83discovered,82passed,1optional real-cgroup skip,
  0failures,0.945s. Architecture-script suites26/26 and2/2 passed separately.
- AST graph23827nodes,59394edges,1092communities; architecture verdict PASS
  and `lay architecture check OK`.
-814 sources yielded no nodes (mostly JSON/documents); AST refresh is not
  semantic documentation coverage. Existing file-size advisories remain.
- Gate log `/home/ubu/.cache/lay/td124-final/td124-final-gates.log`, SHA
  `db15e60c6135ee64a83e414e9e23661be1a2378b8839a62ba55a2e471add15ad`.
- Cargo cache7,709,732,864bytes of12,884,901,888 budget after focused checks.
- Local production PIDs remain IBus4715,daemon3453123,IME3453154.

The updated graph/receipt describes the whole dirty runtime working tree and
is retained there; it is NOT staged into the task-only tooling commit, which
must not publish unfinished TD-121 runtime proof. No Rust production source
or existing release gate is changed by the tooling checkpoint. Its hash is
the Git commit containing the TD-124 completion record; push is checked
against origin/codex/tech-debt-20260831.

No end-to-end development speedup ratio is claimed without a comparable prior
baseline. The delivered benefit is an executable repeatable route, explicit
scope, reusable harness and evidence collection without rebuilding commands
from historical cache paths.
