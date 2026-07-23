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
