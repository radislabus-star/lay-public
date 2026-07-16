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
