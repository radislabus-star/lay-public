# TD-104 Narrow Build Boundary Decision V1

Date: `2026-08-30`

Base commit: `307efa8f57d764ff9390d5c84359001866ea3db4`

## Decision

Evaluate one same-crate `research-tools` feature. Do not reopen the TD-102
workspace split. The feature changes Cargo reachability only; it does not
change runtime logic, package bytes, scoring, authority, or installed state.

The default route is the directly requested runtime target:

```text
cargo check --bin lay-daemon
  -> shared runtime modules
  -> no closed cold roots
```

Research, proof, and release-maintenance routes explicitly enable the feature:

```text
research-tools
  -> closed cold roots
  -> six offline/research binaries

lexical-compiler
  -> research-tools
  -> existing lexical compiler proofs
```

The existing hermetic test manifest and lint baseline continue to cover the
full surface by enabling `research-tools`. A default all-target check remains a
separate route and may skip targets whose Cargo metadata declares
`required-features = ["research-tools"]`.

## Weighted Budget

```text
+5 behavior preservation
+5 route coherence
+4 runtime hot-path build isolation
+4 proof/runtime separation
+3 smaller targeted smoke checks
+3 public command preservation in explicit lanes
-3 feature/configuration surface
-2 Cargo/script maintenance
--------------------------------
+19, no hard veto
```

This score admits a measured candidate, not automatic acceptance. The source
change is retained only when the mini-PC candidate shows a repeatable default
daemon cold-check improvement and every explicit research/test/release route
still passes. A negligible or noisy result closes TD-104 as rejected.

## Non-Goals

- No split of mixed `lexical_grokking` or `l2_field` internals.
- No changes to installed Lay or user services.
- No claim about runtime latency or binary execution speed.
- No reduction of proof coverage.
