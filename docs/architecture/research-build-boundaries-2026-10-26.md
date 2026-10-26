# Measured build boundaries for private research/proof code

Owner: existing nanda_wave compiler/package/runtime contracts and the offline
text_edit journal proof. Decision:2026-10-26-measured-research-build-boundaries.
Stage: original fourth release group, candidate1.0.101. No new model fit,
schema, input owner, delivery route, verifier authority or quality promotion.

## Baseline and choice

Same100 source closure9e1486e38397f45e824ffc4471649ce8e90f1b8fbc09abc0bf45f2347f1a426e:
default library436 dead-code diagnostic records; research library243 plus16
training binary; lexical-compiler library242. These records may name several
items or repeat a location across targets. Exact spans and AST/config mapping
are retained in CACHE101. No executable size/RSS claim follows from a warning.

Chosen9/10: existing test/feature boundaries, shared compiler facade and exact
reserved data expectations. Deleting everything named proof/legacy3/10 loses
unadmitted research or compatibility; blanket lint allowance1/10 creates no
useful boundary. Source changes use the original implementations.

## Implemented boundary

Private functions and data compile only in measured applicable configurations;
cfg(test) retains all original tests. Entire cold/test-only modules use one
parent declaration where supported by actual callers. Reserved enum tags and
carrier fields stay unchanged; item/config-specific expect(dead_code) records
their intentional presence and must become unfulfilled when usage changes.
No crate/module allow or dummy runtime use is added.

Training production code calls the existing library compiler through a
research-tools facade. Existing direct-include binary test modules remain
cfg(test), preserving identities and their characterization coverage. The
output-journal prototype implementation is byte-identical:
cc36c4e8a7b9bc8da91602f74bed342cd5f6db7fae8fa061de402710084244d2.
Its original module is a test shim; the existing microproof imports the same
implementation directly. Protected text_edit.rs/gate.rs stay byte-identical.
The exact OpenOptions owner binding moves with the unchanged implementation.

## Experiments retained

CROOT/release101-boundary-probe1: default11errors,research3,lexical3. Import
bindings and an extension impl did not yet match their new definition scopes;
the delta header also remains required by its codec macro despite lacking a
production constructor. Preserve that fixed wire type with an exact expectation,
align caller/import/module gates, and do not change the codec or suppress errors.

Probe2: all three all-target configurations compile, with25 research unused-
import diagnostics and extra default/lexical imports still to align. A separate
production library research check emits ZERO warnings. Probe3 aligns imports
and whole-module gates: all three all-target configurations compile with only
pre-existing test-support dead-code diagnostics (117default,129research,
50lexical). Full lint, test, optimized artifact and native gates remain pending.
Raw CROOT/release101-{boundary-probe1,boundary-probe2,boundary-probe3,production-check2}.

## Proof required before publication

Keep every original3519 correctness/package and11performance identity/lane.
The lint writer must remain strictly monotonic; only the unchanged prototype's
exact source-path identities may be rebound under this move, with no wildcard
or unexplained new diagnostic. Verify all ten optimized programs, all CI bins,
three feature configurations/MSRV, tiny offline old/new package byte parity,
exact loaded bytes and applicable short native controls. Keep100 full native
FAIL/BLOCKED and WezTerm visual failures separate; no universal native claim.

Quality admission remains in the authoritative syntax-agreement roadmap.
Format parity and source tests do not satisfy heldout/per-damage-class quality.
