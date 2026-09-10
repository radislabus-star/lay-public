# TD-125 — Authorized-plan execution: actual RED/GREEN

Date: 2026-09-07. Verdict: PASS for the scoped executor/Readline regression;
not release acceptance, historical event identification, or live IBus proof.
Installed binaries and services were not modified. No candidate generation,
ranking, SafetyGate/verifier authority, timing budget or mutation owner changed.
Runtime output changed only to execute the existing authorized deletion count.

## Mechanism and observation

`ImeCandidateAccept` can turn a full logical request (`пров -> проверка`) into
an append-only physical plan (`delete=0, insert=ерка`). The terminal executor
combined request deletion4 with plan insertion `ерка`, while its internal tail
stored the full `проверка`. First mismatch is the emitted frame, before terminal
consumption. An additional full-token request then operates on a shorter actual
client word than the internal model expects.

Actual remote baseline failures:

- Expected commit `ерка `; observed `DEL DEL DEL DEL ерка `.
- Controlled completion then full-token executor call: expected Readline line
  `метка проверки §`; observed `мепроверки §`. `§` is a cursor marker injected
  after the frame, not part of the proposed correction. Left context and Space
  were actually removed by Readline, not by a Python imitation of deletion.

This sequence invokes the executor directly. It deliberately does not establish
that ordinary Space bypasses the suppression armed by explicit acceptance.
The historical user's event sequence and its actual terminal output are still
unknown. Full-token Space-shaped requests themselves passed growth/equal/shrink
tests even before the fix. Do not present that path as independently broken.

## Small correction and reproducible proof

The terminal DEL prefix in `state.rs::replace_committed_tail` now uses
`authorized_plan.backspaces`. Output-profile selection is unchanged after review;
the first GREEN below included a profile change that is superseded by repair.
The actual-output trace now reports the authorized physical deletion count;
logical request/candidate records retain their own full-token extent.
Logical tail replacement retains request deletion count and full target text.
GTK/surrounding deletion already used the plan. Cursor-moving plans retain
their SurroundingText requirement. No fallback or new mutation is added.

Command, run from `/home/ubu/projects/lay-tech-debt-20260831` for each distinct
source snapshot (never rerun an unchanged failure until green):

```sh
python3 scripts/dev-check.py check --target bin:lay-ibus-engine
```

| Measurement | RED, unchanged executor | GREEN, two-line fix |
|---|---:|---:|
| Discovered / selected | 404 / 401 | 404 / 401 |
| Passed / failed | 399 / 2 | 401 / 0 |
| Excluded performance / ignored | 3 / 0 | 3 / 0 |
| Entire remote workflow, including transfer | 38.068 s | 39.020 s |
| Worker commands | 14.385 s | 13.912 s |
| Test execution | 5.692 s | 5.667 s |

The initial five test identities in `td125_terminal_edit_tests` cover:

- exact authorized append and full internal logical tail; absent geometry was
  initially admitted, but the repair retains baseline refusal in a sixth test;
- completion followed by full-token edit consumed by Readline;
- nine combinations of grow/equal/shrink and one/two Spaces/Unicode left text;
- deliberate old-count-plus-one mutation producing `меткаслово §`, proving
  that the consumer oracle does not hide boundary loss;
- stale epoch refusal with zero effects and unchanged internal tail.

The helper starts real Bash `read -r -e` with a private PTY and INPUTRC binding
DEL to `backward-delete-char`; payload bytes are written unchanged after prompt
readiness. A separate pipe returns the actual NUL-framed Readline line. Child
executable identity, C.UTF-8 and inputrc hashes are recorded by its JSON result.
Ubuntu Bash embeds Readline: `nm -D /bin/bash` shows `readline`,
`rl_readline_name`, `rl_readline_version`; `ldd` shows no separate Readline DSO.
The helper correctly checks the child Bash executable, not a nonexistent DSO.

Scope exclusions: actual legacy IBus signal delivery, Kitty/Codex GUI, physical
keyboard, combining-mark/grapheme geometry, cold lexical availability and
aggregate Wave quality were not tested by this proof. The existing private
client's `prefetch_not_ready` remains open; its frozen driver was not modified.

## Evidence identities

RED local: `/home/ubu/.cache/lay/development/run-555dzngz/`.
RED remote: `/home/e/projects/lay-development-runner/run-WTAWbo/`.
GREEN local: `/home/ubu/.cache/lay/development/run-hde026w2/`.
GREEN remote: `/home/e/projects/lay-development-runner/run-JPEnUd/`.
Each request owns the complete source archive and file identities; each remote
`tests/SUMMARY.json` owns discovery, exact selected tests, exclusions and failures.

| Artifact | SHA-256 |
|---|---|
| RED `state.rs` | `d8c147bdcedaa2eeb40fa92550bc274e4b1bfc4dc625680b692412a1ad09b056` |
| GREEN `state.rs` | `e8562d03200adfba34ea17f106b1ded75c60ca40176e0ed300e559d5e478113b` |
| Tests, unchanged RED/GREEN | `5f05ec104be64c5c59ea84c9d8b95c89b6d2046b86d3d7dc01b42459b67f9277` |
| Readline helper, unchanged RED/GREEN | `fb70ceaeb79caeb252d59fc1f63551350fddeee37fe1841b40a6767a7674ca5c` |
| RED `tests/SUMMARY.json` | `90846f3d4834f1c20c9f8d59e2bd0a8a58d307d7e289cfbbd5aa66bb726ac11d` |
| GREEN `tests/SUMMARY.json` | `7d726c5d46440442d004fd77e59239fb7f83fbe6753845841b8a0c3fcc025c11` |
| RED `tests/logs/bin-lay-ibus-engine.log` | `2ab37106b61eafe370c88019f0918680a4f51ff7268b9bb3546da07c37d2dd4c` |
| GREEN `tests/logs/bin-lay-ibus-engine.log` | `27fbc4588db11ba3c7f576432e26d10182e6e6a64f316cae523347b02d16ba15` |

Resource envelope: remote `e@192.168.3.94`, dedicated-20cpu, jobs20, test
threads1, CPU2000%, MemoryHigh24G/Max28G, swap1G, Tasks512, Cargo target12GiB;
single existing heavy lease. No local test/build execution or service restart.

## Independent review and bounded repair

Fresh-context `td125_plan_execution_review`, Sol/high: initial7/10, High0,
Medium1, Low2. Dependency-manifest repair separately10/10, no findings.
Reviewer identified that changing profile selection can lose SurroundingText
admission for cursor-moving zero-delete plans. The repair drops that profile
change entirely instead of introducing a broader route-selection API. Only
the terminal physical deletion and diagnostic output count change. The exact
no-backend refusal has a new regression. Evidence was updated, addressing the
documentation finding. One repair and final review completed: **10/10, High0,
Medium0, Low0**, scoped TD-125 executor/tests and dependency repair. Broader
release/historical/live-route acceptance remains excluded.

Four selected integration targets ran remotely:
`ime_space_boundary_contract`, `td113_hybrid_source_contract`,
`text_mutation_monopoly_contract`, `typing_transition_authority_contract`.
44/45 passed. The failure is
`td113_unsuperseded_protected_artifacts_match_the_v4_preflight_baseline`:
TD-121's protected successor remains `PROPOSED`, while that acceptance gate
requires `ACCEPTED`. No protected receipt/status/assertion was changed to make
this green. Exact receipt:
`/home/e/projects/lay-development-runner/run-2jS1Sq/tests/SUMMARY.json`, local
`/home/ubu/.cache/lay/development/run-kt2hr6in/`. This gate prevents broader
acceptance; the focused executor PASS does not override it. One earlier
pre-transfer snapshot was correctly refused after concurrent documentation
editing (`run-1cwv0sb0`); it executed no tests and is not a runtime RED.

## Final repaired union — authoritative current result

```sh
python3 scripts/dev-check.py check --target bin:lay-ibus-engine \
  --target test:ime_space_boundary_contract \
  --target test:text_mutation_monopoly_contract \
  --target test:typing_transition_authority_contract
```

PASS: **440/440**, composed of IME402, Space1, mutation16, typing-authority21.
Performance3 excluded, ignored0. Formatting passed. Entire workflow38.878s,
worker14.225s, tests5.740s. This union deliberately excludes the known failing
TD-113/TD-121 protected-successor acceptance test above; it is not an all-gates
PASS. The refusal is still open and must not be lost in the green denominator.

Local: `/home/ubu/.cache/lay/development/run-xu9f9sp6/`.
Remote: `/home/e/projects/lay-development-runner/run-BgJ1c1/`.
Final `tests/SUMMARY.json` SHA-256:
`1e4c7615de11ef5925ce78ab2b6b5af117085f632ab0c8f0cef05b3227a454b4`.
Final `state.rs` SHA-256:
`6146f2f0c3fcb3eba4336abfa48e7c8bc1f4e732ceeab14aeea8dd40918e1fc3`.
Final test file SHA-256:
`c64121a20560d3624d0481e156eb006010ed2fc3293012e0d59e46626d756213`.
Readline helper unchanged from baseline. The old failing positive-geometry
payload and real-consumer sequence assertions remain; new sixth test preserves
baseline no-backend refusal. Review/pass count: initial review plus one repair
and final verification. No further generic TD-121 review round is claimed.

Architecture refresh ran remotely under the same resource envelope after
syncing the final owning documentation into the runner mirror:
`scripts/update-architecture-graph.sh`, exit0, `lay architecture check OK`.
Log: `/home/ubu/.cache/lay/development/run-xu9f9sp6/architecture.log`.
File-length warnings remain navigation signals, not additional runtime defects
or grounds for a new refactor. Generated graph/binding/receipt must be synced
back into the local worktree; this is not the unresolved protected-successor
acceptance check and does not override its failure.
