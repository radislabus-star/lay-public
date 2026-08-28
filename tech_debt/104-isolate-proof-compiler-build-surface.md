# TD-104: Isolate Proof And Compiler Build Surfaces

Status: `DISCUSSION_REQUIRED`
Priority: `P2`
Class: complex build ownership
Size: `L`
Decision dependency: TD-008 measurements and TD-102 boundary decision

## Evidence Required

TD-008 must provide the exact retained proof/compiler warning inventory plus
clean/default check time, rebuilt crate count, and artifact-size measurements.
No feature split is justified by warning count alone.

## Proposed Outcome

Evaluate the smallest boundary that keeps admitted runtime code in default
builds while preserving every proof/compiler command in an explicit tested
lane. Prefer existing target ownership or one narrow feature over a workspace
rewrite.

## Decision Questions

- What measured local/CI build cost is removed?
- Can target ownership solve it without feature combinations?
- Which package/format types must stay shared and stable?
- How will every proof and compiler route remain discoverable and tested?
- Does this overlap TD-102 enough that only one proposal should be admitted?

## Non-Negotiable Constraints

- No crate-wide `allow(dead_code)`.
- No deletion based only on compiler reachability.
- No runtime algorithm, package byte, authority, or scoring change.
- Default, proof, compiler, and all admitted feature combinations must compile
  through `scripts/cargo-guard.sh`.
- Reject the proposal when the measured benefit is negligible.

## Completion Record

- Decision: pending
- Measured benefit: pending
- Review score: pending
