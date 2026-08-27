# M3 Actual-Owner Sidecar Identity Correction V5

Date: 2026-08-27

Paper verdict: `M3_ACTUAL_OWNER_SIDECAR_IDENTITY_REPAIRED`

Execution authority: absent until the V5 structural route and a separate V5
implementation/execution admission both pass.

## Immutable Failure

V4 compiled successfully, reconstructed one V11 sidecar and stopped before the
first fixed owner request:

```text
V4 receipt verdict                 BLOCKED_PROVENANCE
V4 Cargo exit                      101
fixed owner requests started       0 / 764
historical V11 full SHA-256        5ebffb813ba0ca1e0080ec01756a2dafc51346297558d37cdd135abfde6acfaa
historical Phase 7D source SHA-256 d983d16169c7526d56e9f78299524ab80d3d4f3e67ff19770f07a7ae6e61045d
staged Phase 7D source SHA-256     16b6cc0128099e99c5a77037feac5cf49efd2ed088f6ddb2ade433da92241e5b
```

The sidecar header stores `phase7d_semantics_digest()` at bytes `112..144`.
That digest is the SHA-256 of the complete `typed_edit_traversal.rs` source.
The admitted structured certificate projection therefore changed the full
sidecar SHA even if the encoded DAFSA payload remained byte-identical. V4
incorrectly treated the historical full SHA as the identity of a reconstruction
made under the new source digest.

V3 and V4 remain immutable. V4 is terminal and cannot be retried.

## Corrected Identity Proof

The next proof performs one deterministic reconstruction and evaluates its
identity before any owner request:

```text
current V11 bytes
  -> exact size 2,460,144
  -> payload SHA in header == SHA-256(bytes[256..])
  -> package identity/counts/record widths/root/symbol digest exact
  -> bytes[112..144] == SHA-256(current typed_edit_traversal.rs)
  -> clone bytes in memory
  -> replace clone[112..144] with historical source SHA d983d161...
  -> SHA-256(projected clone) == historical V11 SHA 5ebffb81...
```

The final projection equality is the hard non-header identity gate. Subject to
the already frozen SHA-256 trust model, it proves that every byte other than the
declared source-bound semantics field is identical to the sealed historical V11
sidecar. It is not a relaxed checksum and does not accept a payload, package,
layout, count, root or symbol-table change.

The proof then validates the unmodified current bytes through the current
`V13DafsaView`, materializes exactly one typed generation and lets all fixed
requests borrow that generation. The projected historical clone is never
parsed, searched, written or published.

## Receipt Contract

The immutable owner receipt must publish:

```text
historical full SHA
historical semantics-source SHA
current full SHA
current semantics-source SHA
current payload SHA from header
current recomputed payload SHA
historical projection full SHA
projection changed byte range = 112..144 only
sidecar reconstructions = 1
sidecar files written = 0
owner requests completed = 764 or exact blocked count
```

Any size, header, payload, package, record-width, count, root, symbol digest,
current-source digest or historical-projection mismatch dispatches to
`BLOCKED_PROVENANCE` before owner semantics. Candidate, certificate, capacity,
owner-identity and semantic failures retain the V4 dispatch classes.

## Edit And Execution Boundary

The only newly admitted source edit is the ignored proof in
`src/nanda_wave/l2_field/v13_typed_peak.rs`. No production module, structured
certificate source, package, Cargo input or fixed proof input may change.

One new local Cargo invocation may run the exact ignored owner proof into a new
receipt namespace. It is not a V4 retry. Network, remote execution, perf, PMU,
installation, sidecar publication and production activation remain forbidden.

## Effective Tree

```text
V4 BLOCKED_PROVENANCE (immutable)
  -> V5 source-bound identity correction
  -> V5 structural PASS
  -> V5 execution admission
  -> one current reconstruction
  -> historical-header projection PASS
  -> current loader validation
  -> one typed materialization
  -> 382 forward + 382 reversed actual-owner requests
  -> M3_ACTUAL_OWNER_PARITY_PASS or terminal BLOCKED_*
```
