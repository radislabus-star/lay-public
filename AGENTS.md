## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

## Cargo disk budget

- Run project Cargo commands through `scripts/cargo-guard.sh`; do not invoke a broad unscoped `cargo test` from this repository.
- Prefer `scripts/check-lay-changed.sh` or an explicit `--lib`/`--bin` route. Use `scripts/check-lay-full.sh` only for a release gate.
- The default `target/` budget is 12 GiB. The guard monitors a running Cargo process group and stops it when the budget is crossed.
- `target/` is disposable build cache. Installed release binaries live in `~/.local/lib/lay/bin` and are linked from `~/.local/bin`.
- Check current usage with `scripts/cargo-guard.sh --status` before and after an unusually broad build.

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
- Prefer spending a substantial analysis budget, including roughly 20k tokens when the risk warrants it, over repeated speculative compile/test/deploy cycles. Implementation speed is measured by accepted systemic results, not by how quickly the first patch is produced.

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
