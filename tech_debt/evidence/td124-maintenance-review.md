# TD-124 independent reviews

Scope: development tooling only. These scores do not accept TD-121, product
restoration or release 1.0.66. Runtime authority changed: false.

## Specification — fresh-context Sol/High

Reviewer: `td124_spec_review`. Pass 1: 8/10, H1/M2.

- Specify exact-target Cargo discovery and scoped registry checks, rather than
  full compilation followed by execution filtering.
- Include cross-target source contracts and define known-failure scope.
- Bind complete snapshot provenance, including deletes/renames/untracked paths.

One clarification pass: development checks freeze live-discovered identities,
report canonical drift and reject every failure. This deliberately does not
require a full canonical-manifest rebuild just to add a unit test. Existing
release equality/known-failure contracts remain unchanged. IME automatic
selection includes every integration target; unknown/shared scope broadens.

Pass 2: **9/10, H0/M0**, specification accepted for implementation.

## Code — fresh-context Astra/XHigh

Reviewer: `td124_code_review`. Pass 1: **7/10, H1/M2**.

1. H: configured workspace/output lived below `.cache/lay`, which the existing
   test sandbox masks. Actual discovery built the IME target in21.62s, then
   harness execution failed. Fix the directory, not sandbox isolation.
2. M: HEAD/changed-file/planner inputs could drift between initial planning
   and snapshot start, leaving a narrow plan attached to a broader snapshot.
3. M: standalone private-client API checked remote identity but not the
   existing resource-owner/lease markers; a label was not a held lease.

The reviewer verified normalized driver parity, unchanged fixture inputs and
budgets, explicit target selection, scoped registries, stable source path,
owned-mirror refusal, failure propagation and unchanged release contracts.

Consolidated repair: task-owned `projects/lay-development-runner`, masked-path
validation/tests; HEAD/changed-set checks before and after snapshot; required
guard contract on the standalone client API.

Pass 2: **9/10, H0/M0**, all three reported findings resolved; no further
repair requested. Code-review acceptance is not runtime/product acceptance.
The independent reviewer pinned:

- `scripts/dev-check.py`: `a5aac1d186968c575664bff8b2ee3184cf1ce79f608a645e977226840bb9a9f1`
- `scripts/proof/ime-client/run.py`: `1416efd0a57fd4a954a046e3ee5dc04f8438972b9a808bf897dcdf12bd435a8d`

Remote post-repair self-test through the public command:
`python3 scripts/dev-check.py self-test`,82 discovered,81 passed,1 explicit
optional live-cgroup test skipped,0failures. Log:
`/home/ubu/.cache/lay/development/run-pfo3ti2a/run.log`; remote result:
`/home/e/projects/lay-development-runner/run-keb8Ia/RESULT.json`.
Unittest1.023s; subprocess1.124s. Focused/client/architecture gates remain
separately recorded in the task's execution evidence.

## Last narrow correction — process observation, not runtime

The actual client run exposed an inherited substring PID matcher: the new
explicit candidate bind makes bwrap's arguments contain the candidate path.
The observer therefore falsely counted wrapper PID1. A remote RED test proves
both bwrap and Python wrappers were misidentified; the exact executable-token
predicate then passed12/12 harness tests. Paths:
`/home/e/.cache/lay/td124-tool-tests.6nNQvg/pid-red.log` and `pid-green.log`.

This is the second and final corrective pass. The same independent reviewer
reviewed only this delta: **9/10,H0/M0**. Earlier rest-of-scope approval remains
applicable to unchanged hashes. There was no third general audit.

The driver now has exactly two normalized substitutions: configured candidate
path and the precise PID predicate. Reversing those substitutions reproduces
the original frozen hash. Five scenarios, assertions and timing code remain
unchanged. Final worker SHA:
`8758da61b6a3727b43e9e25f3d1a13b94707f646e2f57cdcb81ce1423a1a3212`;
driver SHA:
`9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750`.
