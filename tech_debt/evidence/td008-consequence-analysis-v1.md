# TD-008 Consequence Analysis V1

Status: `READY_FOR_IMPLEMENTATION`

Base commit:

```text
b4f908e40eeb1a1faa90dd2b169e94061dd4caf0
```

## Scope

TD-008 classifies every row in the TD-005 dead-code baseline and removes only
items whose owner has no runtime, proof, compiler, test, serialized-format, or
compatibility consumer. It also repairs pre-existing non-dead Clippy failures
that prevent the canonical lint route from measuring the dead-code baseline.

The task does not add a feature split, move a package boundary, change a
serialized discriminant, or delete code merely because default Rust
reachability does not observe its proof route.

## Classification Contract

Each of the 368 baseline rows receives exactly one class:

- `live-unused`: part of a live owner but proven to have no consumer;
- `obsolete`: a retired implementation with no retained proof or format role;
- `proof-only`: consumed by tests, proofs, or immutable research routes;
- `compiler-only`: consumed by an explicit compiler or training target;
- `compatibility-api`: retained for wire, disk, schema, failure-taxonomy, or
  source-contract compatibility.

Only `live-unused` and `obsolete` rows are eligible for deletion in TD-008.
Grouped compiler diagnostics are deleted only when every named member has the
same proven ownership.

## Consequences

- Runtime behavior: no candidate birth, retention, ranking, edit plan, output
  mutation, Double Shift, IME, daemon, or backend route changes.
- Package and format identity: serialized fields, enum discriminants, checksum
  layouts, package readers, and compatibility projections remain unchanged.
- Proofs and compilers: retained and classified here; build isolation belongs
  to TD-104.
- Concurrency: no owner, lock, queue, worker, timer, generation, or stale-result
  protocol changes.
- Performance: source removal may reduce compile work or binary surface, but no
  runtime speed claim is made from that observation.
- Rollback: owner-scoped deletion batches remain mechanically reversible; the
  complete classification ledger records every retained and removed row.
- Installation: no release build, install, service restart, or live runtime
  authority is admitted by this task.

## Measurement And Proof

Before and after measurements run on the disposable mini-PC checkout, not on
the installed Lay tree. They record exact source commit, toolchain, cold and
warm lint wall time, rebuilt artifacts, warning inventory, and target size.

Required verification:

1. canonical lint inventory and Clippy contract;
2. affected owner tests for each deletion batch;
3. default and all-target compile routes;
4. package/format parity where a retained compatibility owner is adjacent;
5. independent review of classification and deletion evidence;
6. `git diff --check` and refreshed architecture graph.

## Authority Boundary

This task may delete only unreachable implementation residue. It cannot change
installed binaries, runtime configuration, package bytes, or production
authority. Any proposed edit that crosses that boundary is deferred to its
own task instead of being hidden inside dead-code cleanup.
