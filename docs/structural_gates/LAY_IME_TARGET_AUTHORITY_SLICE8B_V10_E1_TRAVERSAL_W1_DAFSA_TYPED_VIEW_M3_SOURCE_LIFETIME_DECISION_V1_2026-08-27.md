# DAFSA Typed View M3 Source And Lifetime Decision V1

Date: 2026-08-27

Status: `M3_TEST_ONLY_TYPED_VIEW_SELECTED`

## Scope

This paper consumes the positive M3 terminal result only. It decides how the
proved typed representation may leave the disposable experiment source. It does
not grant production authority, change the runtime, compile another ELF, repeat
an M3 route, or claim an end-to-end latency result.

Immutable predecessors:

```text
M3 experiment contract SHA-256
  55ab0bb2bcda695bde3653fabad74243edb6230f4e4072264a5a33a807ba04be
M3 implementation receipt SHA-256
  b4461145fcffa760c904e93838dace7770933a5fd8bab7231623bc2caa3cc4a9
M3 execution admission SHA-256
  bd77fc3d568a20cd21d108db20eb57995b32cfa320056ab37aebaca6bfec9119
M3 terminal receipt SHA-256
  a84355e42bad335d45b379c7e76d2b353bed6c23c30593e1c721be0c0058f324
M3 terminal verdict
  W1_DAFSA_TYPED_VIEW_PASS
```

The experiment contract identity above is the SHA-256 of
`LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_V1_2026-08-27.md`.

## Measured Facts

The exact test-only source built one prevalidated typed view from the sealed
byte view before the measured region. Full forward and reverse parity covered
`81,128` states and `226,341` edges with zero field or query mismatches.

```text
                              byte view          typed view
traversal thread CPU/edge     26.032472651 ns    22.961831598 ns
cycles/edge                  103.702024253       92.152445235
instructions/edge            361.199825441      307.739808260
effective frequency            3.791382262 GHz    3.791263675 GHz

CPU gain                      11.795426%
cycle gain                    11.137274%
instruction delta            -14.800676%
frequency delta                0.003128%
```

The typed payload contains one 12-byte record for every state and edge:

```text
states                         81,128
edges                         226,341
typed payload               3,689,628 B
construction wall          1.627..1.678 ms
construction thread CPU    1.629..1.679 ms
```

Construction was outside the traversal denominator. The result therefore
proves a traversal representation gain and records, but does not amortize, its
construction and residency cost.

## Decision

The prevalidated typed view is selected as the sole M3 implementation candidate
for future test-only V13 integration. Repeated byte-slice decoding and its error
plumbing account for `3.070641053 ns/edge` of the measured W1 traversal, within
the exact pinned workload and host envelope.

The byte-view baseline remains the immutable comparison and format validator.
It is not a second scientific candidate and M3 may not be rerun. The rejected
M2 fused-minimum candidates remain rejected.

No additional W1 instruction-level experiment is admitted from this result.
The next performance claim must be an end-to-end single-request measurement in
the authority route that actually owns the exact candidate search. A lower
`ns/edge` value alone cannot promote a diagnostic executor.

## Ownership And Lifetime

Future integration must provide exactly one typed-view owner per exact sidecar
identity. The owner must be created only after byte-format validation and must
be invalidated atomically with the package/sidecar generation that produced it.
No query key, target label, request-local cache, fallback representation race or
independent reload path is allowed.

The selected first integration shape is:

```text
validated immutable sidecar generation
        -> one safe typed materialization
        -> one immutable typed-view owner
        -> all exact-search requests borrow that generation
```

This shape is selected for a future test-only source integration because it is
the mechanism actually measured by M3 and preserves safe indexed access. It is
not yet a production residency decision.

## Rejected And Deferred Designs

1. A query-local or per-request typed materialization is rejected. It would add
   roughly `1.6 ms` and `3.69 MB` of transient allocation to each request and
   invalidate the measured traversal interpretation.
2. Reinterpreting the byte sidecar as native Rust records is rejected. Alignment,
   endianness, record validity and aliasing were not proved, and M3 admitted no
   `unsafe` or unchecked format view.
3. A new native on-disk sidecar is deferred. It changes package compatibility,
   delta/reload semantics and the format source of truth, none of which M3
   tested.
4. Keeping independently reloadable byte and typed authorities is rejected. It
   creates identity drift and stale-result races.

## Consequence Boundary

The current `v13_typed_peak` executor is compiled only under `#[cfg(test)]`.
The production `TypingAssistWorker`, candidate/lattice retention, ranking,
SafetyGate, edit plans, feedback semantics, daemon queueing and runtime package
authority were not changed or tested by M3.

Before any production edit, a separate consequence/preflight paper must close:

```text
candidate and certificate parity in the actual authority path
single-request p99 and total-material p99 on the fixed proof
RSS budget for the original sidecar plus the 3,689,628-byte typed payload
single owner and generation identity across package and delta reloads
concurrent readers and stale-result cancellation
allocation failure and rollback to the prior generation
package compatibility and removal/replacement plan
all fixed heldout quality classes and false-authority gates
```

Until then:

```text
test-only typed source selection       admitted
production source edit                 not admitted
runtime authority change               false
install / restart / deployment         forbidden
new remote performance route           not admitted
```

## Next Tree

```text
M3_TEST_ONLY_TYPED_VIEW_SELECTED
        -> test-only source integration consequence/preflight
        -> exact candidate/certificate parity in owning route
        -> end-to-end single-request latency and RSS proof
        -> production authority decision

Any missing consequence or failed gate
        -> STOP without production promotion
```
