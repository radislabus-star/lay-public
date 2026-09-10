## Accepted development simplification — 2026-09-07

- The user subsequently prioritized TD-124 maintenance tooling before further
  release work. Follow DEVELOPMENT.md: scripts/dev-check.py is the remote-only
  development entrypoint. Its explicit focused PASS is not release acceptance;
  canonical full gates remain unchanged. No new runtime migration is implied.
- Keep release 1.0.66 scoped to TD-120/TD-121 and verified delivery. Wave
  quality is TD-123 / 1.0.67. Broad ownership migrations, research/runtime
  separation and removal of hot-upgrade compatibility are separate decisions,
  not new prerequisites silently added to 1.0.66.
- Start from the actual failing entrypoint and visible effect, then make the
  smallest justified change, rerun that scenario and its affected contracts,
  obtain independent review, and execute the mandatory release gates. A helper
  or atomic-path PASS cannot establish legacy-client or physical-input PASS.
- Reproduce concurrent failures with controlled event order and bounded waits
  using existing production reducers/adapters. Distinguish key-before-ready
  from key-after-ready. Do not substitute sleeps, retries-until-green, or a
  second simulated implementation for a causal proof; retain real IBus smoke.
- Targeted verification must establish the exact expected test identities,
  nonzero selection, actual entrypoint and asserted effects. Use the existing
  test manifest/discovery machinery. Prove a new regression detects the old
  failure or a narrowly controlled violation; record which kind was tested.
- Use existing guarded explicit --lib/--bin or test-lane routes for the inner
  loop. check-lay-changed.sh currently runs all correctness/package lanes for
  Rust changes; do not call it a component-only check. Keep full final gates,
  and use broader checks whenever the affected dependency closure is unknown.
- Reuse proof only when relevant source, dependencies, configuration, toolchain
  and environment identities still match. Do not rebuild accepted artifacts
  during installation or rerun unchanged evidence just to generate a receipt.
- Give one implementation owner a connected runtime change and one owner the
  remote heavy-execution lease. Other agents may independently review or work
  on disjoint files. Retain the two-pass review/repair limit; unresolved issues
  require explicit replanning, never weakened assertions or an unreported pass.
- Before adding an owner, generation, timer, cache, queue or fallback, identify
  the demonstrated failure, why existing mechanisms cannot suffice, and the
  replacement/removal boundary. File/line counts are warning signals, not
  arbitrary quotas. Reuse ContextAdmissionReducer rather than add another
  authority controller; preserve distinct GTK, terminal and atomic transports.
- Extend existing opt-in bounded diagnostics with causal refusal reasons and
  event/owner identities where needed. Avoid user text in new metadata traces;
  diagnostics must not grant authority, add polling/RPCs, or change deadlines.
- Keep one current owning task with exact log/receipt links; preserve historical
  evidence as historical. Measure reproduction time, focused-check time and
  repair rounds separately; do not claim speedup from test count or estimates.
- All builds, tests, training and architecture refreshes run only on the remote
  host under the existing resource/Cargo guards: dedicated-20cpu, build jobs 20,
  Rust test threads 1, CPU 2000%, MemoryHigh 24G, MemoryMax 28G, swap 1G,
  TasksMax 512, target <=12 GiB. Reading/editing sources locally is allowed.
- Never use the nanda-structural-gate skill or any of its bundled commands.
  Consequence analysis is ordinary owning-document text. Do not restart global
  IBus, change engine names or migrate user input sources without explicit
  separate user approval; accepting simplification is not that approval.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code or architecture documentation, run `scripts/update-architecture-graph.sh`. It performs the AST-only Graphify update, refreshes the source binding, publishes a receipt only for a `PASS` verdict, and runs the mandatory architecture check. A bare `graphify update .` is not a completed refresh.

## Cargo disk budget

- Run project Cargo commands through `scripts/cargo-guard.sh`; do not invoke a broad unscoped `cargo test` from this repository.
- Prefer `scripts/check-lay-changed.sh` or an explicit `--lib`/`--bin` route. Use `scripts/check-lay-full.sh` only for a release gate.
- The default `target/` budget is 12 GiB. The guard monitors a running Cargo process group and stops it when the budget is crossed.
- `target/` is disposable build cache. Installed release binaries live in `~/.local/lib/lay/bin` and are linked from `~/.local/bin`.
- Check current usage with `scripts/cargo-guard.sh --status` before and after an unusually broad build.

## Protected Double Shift route

- Physical Double Shift has exactly one detector: the `lay-daemon` trigger FSM.
  It invokes `ManualToggleV3`; that reply selects one typed execution route and
  never authorizes a second detector or mutation.
- Legacy IBus `ProcessKeyEvent` is observe-only for Shift press/release events.
  Never add local pair recognition or call a manual-toggle/text-edit method from
  that route. Two detectors produce the forbidden `x -> y -> x` reversal.
- `ProcessKeyEventAtomicV1` is an exclusive route and may retain its own atomic
  gesture detector because legacy processing is disabled there.
- The code boundary is `observe_daemon_owned_legacy_shift`, which intentionally
  has no `EngineOutput` argument. Keep Double Shift mutation capability only in
  `process_atomic_shift_gesture` and the daemon-owned `ManualToggleV3` route.
- Ordinary Double Shift is an exact physical-key projection only: for example,
  `а <-> f` and `привет <-> ghbdtn`. It must not call lexical correction,
  ranking, morphology, learning, or any model.
- After IME autocomplete acceptance, the accepted token is an
  `ImeCommittedTail`. `ManualToggleV3` must return the typed exact-tail
  disposition. The daemon then reads one exact `VisibleTailV2` snapshot from
  the IME; it must never reconstruct that suffix from `DaemonWordBuffer`.
- Every complete `press -> release -> press -> release` pair toggles once and
  rearms immediately. Do not add a burst latch, quiet-window rearm, pair
  debounce, or multi-tap delay. Four Shift taps are exactly two toggles.
- Legacy sequential `DeleteSurroundingText` plus `CommitText` is not the GTK
  committed-tail route: two sealed attempts proved that
  `RequireSurroundingText` does not produce a deterministic callback there.
- Exact SurroundingText committed-tail replay requires an active physical-input
  grab, two validations of the same source/focus/epoch/tail lease, one
  no-fallback GNOME `ActivateLayout`, exact shell and IBus readback, checked
  autocorrect suppression, and one bounded uinput delete-plus-replay. No
  polling, retry, second mutation, or generic layout reconciliation is admitted.
- An `ImeCommittedTail` with terminal purpose, no SurroundingText, and proven
  terminal erase geometry must execute inside `ManualToggleV3` as exactly one
  `terminal_erase_commit` output frame followed by one IME-owned layout sync.
  It must never delegate to daemon physical input, emit physical Backspace, or
  replay replacement characters through ordinary IME key processing. Preserve
  exact round trips such as `rjvvbn <-> коммит`, including a trailing boundary.
- Do not arm committed-tail suppression merely when `ManualToggleV3` delegates
  to the SurroundingText route. Arm it only after exact capture, both lease
  checks, and target-layout handoff succeed, immediately before the first
  Backspace. The terminal single-commit route does not use this suppression.
- The GNOME extension activates the target Lay input source exactly once.
  GNOME's input-source manager owns the resulting IBus transition; do not run a
  second `ibus engine` command from `activateLayoutId` or its
  `current-source-changed` callback.
- Printable IME input invalidates old candidate authority immediately but keeps
  the current preedit surface visible until the matching background result can
  replace or hide it once. Do not restore per-key `clear -> update -> show`
  blinking, and never let Tab accept the retained stale surface.
- Before any Double Shift release or installation, run
  `scripts/check-lay-changed.sh`; the explicit
  `physical_double_shift_owner_` tests must pass, followed by a single-owner
  client-visible smoke and one real-keyboard confirmation.

## Architecture evidence discipline

- After every architecture experiment, update the owning architecture document in the same change. Do not leave the result only in terminal output or a receipt.
- Record separately: what was tested, measured facts, what was not tested, verdict scope, exact receipt path, and whether runtime authority changed.
- Compression/format parity is not a quality proof. Quality claims require aggregate and per-error-class percentages from the fixed heldout proof.
- Keep estimates, hypotheses, measured facts, and promotion gates visibly distinct.
- Never dismiss or rhetorically rank L1 proof dimensions against each other. Per-class restoration, clean preservation, lattice coverage, false certainty, package/RSS budgets, and latency form one conjunctive contract; all must be reported and all required gates must pass.
- The accepted L1 working gate is strict `unique top-1 > 95%` for every damage class. Aggregate top-1 cannot hide a failing class.

## Consequence analysis before code changes

- Before every code change, perform a consequence check scaled to its risk. For any runtime hot-path, authority-bearing, stateful, cached, concurrent, model/package, or architectural change, stop and write the analysis before editing production code.
- The written analysis must cover at least: candidate/lattice retention, ranking and false authority, latency deadlines and tail behavior, CPU/RSS/allocation effects, cache identity and invalidation, package/delta reloads, learning and feedback semantics, concurrency and stale-result races, failure/rollback behavior, compatibility with IME and daemon consumers, and long-term maintenance/removal cost.
- Analyze second-order effects, not only the reported example. State explicitly what can get worse, which existing invariant could be violated, how future packages or online updates change the conclusion, and whether the proposal creates another owner, route, cache, fallback, or source of truth.
- Compare the current baseline with at least two viable designs for a nontrivial change. Record why the selected design wins and why the rejected designs fail. A fast local result is not sufficient if it narrows the candidate field, weakens competition, changes authority, or increases future route count.
- Keep facts, hypotheses, estimates, and unverified assumptions separate. If a required consequence cannot yet be bounded, remain in analysis and gather evidence; do not start production implementation.
- Before code starts, record the chosen route, invariants, expected regressions, rollback boundary, proof denominators, and removal/replacement plan in the owning architecture document or implementation preflight. A preflight without an explicit consequence analysis is incomplete.
- Scale analysis to demonstrated risk and missing evidence, not a fixed token target. State the first unresolved mechanism and gather the smallest discriminating proof before another patch. Implementation speed is measured by accepted systemic results, not by how quickly the first patch is produced.

## Systemic wave-contour fixes

- Never repair a failing L1.1/L2/L3/L4 case by adding a literal word, phrase, suffix, test name, source ID, or case-specific branch to runtime code.
- Never work through a failing example list one item at a time. Group failures by their first shared mechanism, fix that mechanism once, and rerun the whole fixed proof set.
- A fixture may prove a general invariant, but fixture text must never become a runtime condition or a manually weighted exception.
- For restoration regressions, inspect the complete authority path first: `L1.1 Winner | Tied lattice | ABSTAIN -> L2 -> L3 -> L4 -> DecisionCore -> verifier`. Record the first layer where target retention, rank, or authority is lost.
- A grounded L1.1 candidate must remain in the bounded lattice. Higher layers may add candidates and apply calibrated positive or contradictory evidence, but generic uncertainty, an unrelated basin, or a second mutation-oriented veto must not erase it.
- A grounded L1.1 Winner may be downgraded only by explicit independently measured contradictory evidence. A generic L2/L3/L4 tie or abstention is not contradictory evidence.
- Do not weaken `SafetyGate`, edit-plan validation, or verifier authority to compensate for a candidate-generation, target-retention, ranking, or authority-transfer defect.
- Semantic tests must assert surfaces, verdicts, proofs, and safety effects. Do not make runtime ownership migrations fail only because an old producer `source_id` changed.
- If a change improves selected examples but the aggregate fixed proof or any required error class regresses, reject the change as a systemic failure.
