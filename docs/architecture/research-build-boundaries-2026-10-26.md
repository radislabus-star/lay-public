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


First complete source run:3518PASS/1FAIL of3519, raw
CROOT/release101-final-gates/source.log. The only failure is
peak_oracle::tests::oracle_is_unreachable_from_production_source_owners,
whose final string still requires the broader test-or-lexical-compiler gate.
The new module is test-only. Its source guard now requires that stricter
boundary; all original runtime/service/peak_search no-oracle assertions remain.
No test identity/denominator or input guard is removed. Record this explicit
source-guard migration in the same decision and rerun the complete proof.


## Final 1.0.101 proof and scoped verdict

CROOT/release101-verified-gates/result.json: all20 gates PASS,3519/3519
correctness/package tests and11/11 performance,419Python(10skipped), lints,
both MSRV1.88 configurations and10 optimized programs. Release build215.690s.
The installer build emits0 Lay warnings (same100 build emitted259). The tiny
old/new compiler fixture is byte-identical; no model/quality promotion follows.
All original test identities/lanes remain unchanged. Source/modes1289files:
8c25e15e9c6f9c481a0b91e5da1b59227749f19cfd6c2db187c8bf7ee27f1a9f.
Local exact closure matches. CI40394811146 andSecurity40394811162 SUCCESS on
functional4c61cdb6a7e5f7344e8bc9e78ccbf5d1bbdc9c22.

Retained artifacts installed only on private remote KDE, engine3293136
SHAf7b8c6cd6d0c04195c63d6a13420f07c1a0ae56abb133e634d69fa14fe903b0a,
all10 binaries checked, original IBus/desktop preserved. Frozen r18 smoke20 each:
Firefox12P6F2B,Chrome15P3F2B. Two previous PASS losses andonegain versus100short
are retained; new ChromeSpace first refusal stale_lease, remains OPEN.
Kitty5P,Konsole5P,WezTerm2P3F; new visuals NOT_TESTED. Raw terminal static
version label100 is corrected by companion101 exactSHA/PID evidence without
rewriting rawresults. Full440/GNOME native not tested on101. Detailed records:
tech_debt/evidence/release-1.0.101-{verification,native}.json; raw CACHE101/verified.
Verdict: build scope VERIFIED, native residuals OPEN, not regression-free.
Runtime ownership, edit/verifier permission and model/learning authority unchanged.
