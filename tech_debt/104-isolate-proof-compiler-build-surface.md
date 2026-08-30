# TD-104: Isolate Proof And Compiler Build Surfaces

Status: `DONE`
Priority: `P2`
Class: complex build ownership
Size: `L`
Decision dependency: TD-008 measurements and TD-102 boundary decision

## Evidence Required

TD-008 must provide the exact retained proof/compiler warning inventory plus
clean/default check time, rebuilt crate count, and artifact-size measurements.
No feature split is justified by warning count alone.

## Proposed Outcome

Inventory all unconditional cold compiler, proof, and eval roots across lexical
grokking, L2, L3/context, L4, and top-level eval surfaces. Evaluate the smallest
boundary that keeps admitted runtime code in default builds while preserving
every proof, compiler, and eval command in explicit tested lanes. Prefer existing target
ownership or one narrow same-crate feature over a workspace rewrite; the
existing `lexical-compiler` feature alone is not assumed to cover the surface.

## Decision Questions

- What measured local/CI build cost is removed?
- Can target ownership solve it without feature combinations?
- Which package/format types must stay shared and stable?
- How will every proof, compiler, and eval route remain discoverable and tested?
- Does this overlap TD-102 enough that only one proposal should be admitted?

## Non-Negotiable Constraints

- No crate-wide `allow(dead_code)`.
- No deletion based only on compiler reachability.
- No runtime algorithm, package byte, authority, or scoring change.
- Default, proof, compiler, eval, and all admitted feature combinations must
  compile through `scripts/cargo-guard.sh`.
- Reject the proposal when the measured benefit is negligible.

## Completion Record

- Decision: evaluate a narrow same-crate `research-tools` feature; no workspace split
- Measured benefit: accepted; default daemon cold-check median `20.80 -> 19.42 s`
  (`-6.63%`) and median peak RSS `1,295,896 -> 1,224,580 KiB`
  (`-5.50%`) across three fresh-target mini-PC runs
- Review score: `4/10 REVISE -> 9/10 PASS`; all five first-pass findings were
  corrected or closed with current evidence, and the final review found none

## Admitted Scope

The implementation candidate is intentionally smaller than the complete cold
surface inventory. It may isolate only roots with a closed research-only call
graph:

- top-level eval cases plus Nanda eval, L3 metric, and self-teacher modules;
- L3 context compiler, proof, and stream helpers;
- L4 cross-scene compiler, updater, proof, and usage-adapter helpers;
- morphology proof and lexical-phase compiler helpers;
- the offline eval, train, dataset, morphology, n-gram, and output microproof
  binary targets.

Mixed lexical-grokking and L2-field modules remain in the default crate. Their
proof and compiler files share runtime package types, typed certificate
semantics, productive runtime material, or test-owned helpers. Splitting those
files is a separate source decomposition, not part of this build-boundary task.
The L3 surface field and sentence representation also remain shared because
the live online/context package route owns them directly.

The candidate is accepted only if three clean mini-PC measurements show a
repeatable default `lay-daemon` cold-check reduction while the explicit
research lane, full hermetic tests, lints, release build, and package commands
remain available. Otherwise the source change is rejected and only this
measured decision is retained.

## Implemented Boundary

`research-tools` is a presence-only same-crate feature. It owns the closed
offline eval, compiler, proof, stream, and administration roots listed above,
plus the six offline binary targets that consume them. `lexical-compiler`
includes `research-tools`; it retains its narrower lexical behavior switch.

The default crate retains every live package/format/readout type, L3 sentence
and surface representation, runtime refresh owner, lexical runtime, exact V13
traversal, daemon, IBus engine, and installer-owned runtime binary. No package
bytes, scoring rule, candidate authority, or runtime algorithm changed.

The one integration test that directly calls the cold L3 feedback compiler is
feature-owned as well. With `research-tools` it retains the same manifest target
and test identity; without the feature its empty test crate allows default
`--all-targets` to remain valid.

## Measured Outcome

All measurements used Rust/Cargo `1.97.1`, `CARGO_INCREMENTAL=0`, offline mode,
and one fresh target directory per run on `e-MEGA-MINI-M1-13th`.

| Route | Baseline median | Candidate median | Delta |
| --- | ---: | ---: | ---: |
| default `lay-daemon` cold check | 20.80 s | 19.42 s | -6.63% |
| default `--all-targets` cold check | 24.41 s | 23.83 s | -2.38% |
| explicit research trainer cold check | 21.02 s | 20.66 s | -1.71% |

The daemon median peak RSS fell by `5.50%`. Default all-target rustc command
lines fell from `266` to `254`; the explicit research route remained `218 ->
218`, proving that the feature hides cold ownership from default builds rather
than deleting the tooling.

## Verification Scope

Tested:

- default daemon, default all-targets, lexical compiler, and research target
  compilation on the mini-PC;
- exact feature/target metadata and all `2,383` test-manifest identities;
- all `2,359` correctness/package executions with zero semantic or
  infrastructure failures;
- default and `research-tools` check/Clippy routes, with `366` exact research
  dead-code entries and zero non-dead diagnostics;
- release build of all 14 binaries with `research-tools`;
- public installer regression and release-package contracts;
- clean-install and self-teacher source fallbacks statically pinned to the
  required feature.

Not tested by TD-104:

- the serialized runtime performance lane;
- `direct-llm` and all-features compilation;
- live desktop/IME behavior;
- an actual installation, service restart, or runtime package mutation.

Verdict scope: `TD104_RESEARCH_TOOLS_BOUNDARY_ACCEPTED`. It is a build-ownership
and developer-feedback improvement only. Runtime authority and installed Lay
state did not change.

Evidence:

- `evidence/td104-boundary-decision-v1.md`
- `evidence/td104-boundary-correction-v2.md`
- `evidence/td104-boundary-correction-v3.md`
- `evidence/td104-integration-test-boundary-correction-v4.md`
- `evidence/td104-remote-baseline-v1/BASELINE.json`
- `evidence/td104-remote-candidate-v4/CANDIDATE.json`
- `evidence/td104-remote-validation-v1/VALIDATION.json`
- `evidence/td104-zero-failure-observation-v1.json`
- `evidence/td104-independent-review-v1.md`
- `evidence/td104-independent-review-v2.md`
- `evidence/td104-SHA256SUMS`
