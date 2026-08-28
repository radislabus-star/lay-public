# TD-102: Separate Nanda Runtime And Research Tooling

Status: `DISCUSSION_REQUIRED`
Priority: `P2`
Class: workspace and package architecture
Size: `XL`
Decision dependency: finish TD-001 through TD-008

## Evidence

- `src/nanda_wave` contains 163,770 lines across 193 files.
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
