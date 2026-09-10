# TD-120 runtime implementation — Sol/High

Worktree `/home/ubu/projects/lay-tech-debt-20260831`, baseline cc1e2207.
User requests implementation through release 1.0.66 and push. Implement this
bounded TD-120 task; parent owns remote checks, TD-121, review, commit/release.
Do not stop at a plan. Use apply_patch. No local Cargo, services/config/binary
changes, input devices, /root files, nanda-structural-gate, git writes or agents.
Read AGENTS.md, owning TD-120 and its full suppression-admission-analysis.
Read required skill instructions yourself, but do not scan unrelated evidence.

The selected design is FINAL: typed CurrentWord/LegacyReplayV1/ExactReplay,
ordinary lifetime with shared authoritative incarnation, existing transport
semantics, scoped atomic base-revision/owner/tail settlement. All consequences
are already written before this production admission. Implement that design
without a generic transaction framework, new protocol/worker or TD-121 observer.
README is updated. TD-122 remains an explicit Stage 2 residual.

Baseline test-only run on guarded 20-CPU remote: test module SHA
2cec08d82a40d7a4f551998911e4f40b49f1a9c8e6f971d786734d973356f787,
9 executed, 3 pass/6 fail, 0 ignored; compile 2.39s, tests 0.56s. Raw receipt
remote `/home/e/projects/lay-td120-121-SUdh2I-td120-red-v2.log`.
Five failures reach stale-suppression assertions: full deletion, left context,
same-text retype, undo boundary, duplicate. Candidate-boundary sixth failure
is a TEST BUG: insertion optimization emits no deletion. Correct test output
expectation to the legitimate minimal effect (commit exactly once, delete only
where needed), not runtime output to satisfy the helper.

Before runtime edits, fix these test-construction issues and freeze the named
new tests in your small implementation evidence note:

- Ordinary isolated fixtures should bind focus through real existing method;
  do not weaken owner validation for unbound tests. Avoid prebinding the exact
  target helper before its intended target FocusIn; use separate helper.
- Duplicate fixture starts `a  ` and after first deletion yields `a `, a CLOSED
  token. Correct first guard expectation to false for this case. Also add an
  open-result duplicate fixture to prove duplicate neither creates nor erases
  an existing valid guard. Do not treat closed-word old expectation as parity.
- Exact tests may change representation access but MUST retain source/path/
  epoch/deadline/admission/revoke semantics and temporary empty-tail protection.

Implement all ordinary producers/consumers and retirement, not only Backspace:
successful active manual/candidate, committed manual/candidate/undo/ReplaceTail;
failed/no-op/duplicate no new guard; partial/cursor edits/layout punctuation;
full current token deletion with left history; Space/Enter/terminal boundaries;
existing admitted handoff transfers same incarnation with rebind, old owner
cannot consume/clear it. No actual edit success inferred merely from Ok(()).
Preserve exact V2 and named V1 compatibility, no unchanged shadow booleans.

Atomic reconciliation is mandatory in this patch. Capture BASE revision and
owner/tail/focus/lease from same cloned snapshot; compare and replace under one
live lock. Guard-only drift preserves newest complete live suppression/handoff
block, including None and local mirror. Owner/tail drift rejects stale engine
AND shared assignment, keeps new state, retires stale local authority, no
deferred layout/feedback or repeated output. Next key uses existing refusal.
Cover equal final numeric increments, V1/V2 arm/revoke, sensitive ContentType,
shared-only exact cancellation, foreign engine and real ABA callbacks.
Do not assert unreachable same-engine mutator races. Public atomic wire
formats, exact replay safety and physical Double Shift ownership unchanged.

Add required focused tests from O1–O6/E1/A1–A3/C1; characterize V1 residual
without repairing its protocol. Use actual engine/output/callback methods and
surfaces/verdict/effect counts. Existing exact/atomic tests must keep semantic
coverage after field migrations. Don't drop assertions, add ignores, broaden
lexical authority or add fixture-specific runtime branches.

Only format touched Rust sources (avoid recursively formatting unrelated files),
run static/diff checks. No Cargo remotely either: parent is sole build owner.
Write a concise evidence note with changed mechanisms, exact test manifest,
known residuals, and unverified checks. Return implementation files and ready
for remote test; never claim compilation/PASS/DONE without evidence.
