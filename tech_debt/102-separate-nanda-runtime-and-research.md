# TD-102: Separate Nanda Runtime And Research Tooling

Status: `DONE`
Priority: `P2`
Class: workspace and package architecture
Size: `XL`
Decision dependency: finish TD-001 through TD-008

## Evidence

- `src/nanda_wave` contains 164,071 lines across 193 files at the TD-102
  decision commit.
- The main crate exports or internally exposes thousands of items and builds
  runtime readout, compilers, proofs, evals, and research kernels together.
- Large files include V13 typed peak, productive proof/runtime, L2 runtime,
  lexical proof/compiler, and L3/context phase.
- Dead-code and build noise are concentrated in this mixed surface.

## Proposed Outcome

Evaluate a small workspace split with stable contracts:

- `lay-runtime`: hot/readout and product-facing contracts;
- `lay-model-format`: shared immutable package types/codecs;
- `lay-research`: compilers, proofs, evals, and one-shot experiment support.

This is a proposal, not a predetermined three-crate rewrite. Fewer boundaries
are preferred if feature isolation from TD-008 is sufficient.

## Decision Questions

- Does feature isolation already solve compile and lint costs?
- Which types genuinely cross runtime/format/research boundaries?
- What is the measured clean build, incremental build, and binary-size benefit?
- Can package format parity be proven without duplicating types?

## Required Preflight If Admitted

- Public command and package-format inventory.
- Dependency-direction map and cycle check.
- Baseline build timings, binary sizes, and test lane timings.
- Move-only first pass with no algorithm or schema change.

## Rejection Condition

Reject if it creates circular crates, duplicated schemas, broad public APIs, or
less than a measurable build/navigation benefit.

## Completion Record

- Decision: `WORKSPACE_SPLIT_NOT_ADMITTED_BENEFIT_UNPROVEN`
- Current architecture: one crate retained; package schemas and commands
  unchanged
- Remote measurement: three clean repeats for `lay-daemon` and `--all-targets`
- Runtime median: 20.11 seconds
- All-targets median: 24.44 seconds
- Paired overhead: 3.91 / 4.21 / 4.41 seconds
- Raw evidence: 25/25 SHA-256 entries PASS
- Review: pass 1 `6/10`, pass 2 `8/10`; all requested fixes resolved
- Installed runtime: unchanged
- Next owner: TD-104, limited to measured same-crate cold-surface isolation

Evidence:

- `evidence/td102-build-surface-measurement-v1.json`
- `evidence/td102-build-surface-measurement-v2.json`
- `evidence/td102-remote-raw-v2/`
- `evidence/td102-boundary-decision-v1.md`
- `evidence/td102-independent-review-v1.md`
- `evidence/td102-independent-review-v2.md`
- `evidence/td102-completion-v1.json`
