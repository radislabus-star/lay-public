# Build warnings: measured private proof/research boundaries

Status: DONE_SCOPED_BUILD. Priority: P2. Release group:1.0.101.
Owner: existing package/runtime/compiler contracts; extends TD-135 under the
user's explicit six-release implementation order. No new fit or input route.

## Root and measured alternatives

Same source100: installer research build259 dead-code diagnostics (243library,
16training executable); default library436 and lexical-compiler library242.
These are diagnostic records, not the count of unique functions. Items need
different boundaries: test-only proofs, compiler-feature consumers, duplicated
binary compiler support, and reserved shared data which must keep its shape.
Exact compiler spans and AST mapping are retained in CACHE101.

Recommended9/10: narrow existing cfg boundaries from the three measured
configurations, retain all tests, share compiler implementation and keep
explicit reserved data contracts. Delete everything named proof/legacy3/10:
would risk package compatibility and unadmitted research. Blanket warning
suppression1/10: no executable boundary or useful maintenance result.

## Implementation constraints

Gate private items only in measured unused configurations, with cfg(test)
preserved. A public CLI/compiler consumer must remain available. Do not change
stable enum tags or remove carrier fields to quiet the compiler: exact
field/variant expectations document reserved contracts, with no crate/module
allow or dummy runtime references. New reachability must make an old
expectation unfulfilled instead of silently hiding an obsolete annotation.

The training binary must reuse the existing compiler; prior direct-include
test modules keep their original identities. The offline output-journal
prototype keeps one implementation. Its original library module becomes a
test shim, and its existing microproof binary imports the implementation.
Protected text_edit.rs/gate.rs stay byte-identical. The architecture scope
guard changes only the exact moved owner path, under the explicit decision.

## Verification and limits

Require three configurations, all ten optimized installed programs, all CI
binary targets, unchanged source test identities/lanes, existing package
round trips, meaningful lints and no unexplained baseline additions. Record
actual diagnostics, artifacts, source/mode closure and short native controls.
Keep public model bytes, live learning data and native residuals unchanged.
Compilation/format parity does not prove improved restoration quality, RSS or
latency; measure those separately if a claim is made.


## Completion evidence

All release gates passed, installed build diagnostics259->0, compiler tiny
fixture bytes equal, all original3519 correctness/package and11performance
identities retained. Both public CI workflows pass on4c61cdb6. Exact source,
all10 artifacts and remote loaded-byte/native controls are in release101
verification/native receipts and the owning document. This closes the measured
build-warning slice; residual native failures and unadmitted research/quality
work remain OPEN. Publication/updater discovery is recorded after publication.
