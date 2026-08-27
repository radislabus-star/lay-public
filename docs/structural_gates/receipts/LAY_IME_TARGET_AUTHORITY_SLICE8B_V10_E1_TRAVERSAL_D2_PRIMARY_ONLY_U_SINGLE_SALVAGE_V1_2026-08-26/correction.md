# D2 U-SINGLE Worker Sentinel Correction V1

## Scope

The immutable U-SINGLE execution receipt remains `BLOCKED_SEMANTIC`. Its only
selected violation was `component worker coverage mismatch`: the controller
expected worker ID `0`, while all 7,640 records carried worker ID `255`.

That predicate was not part of the frozen U validity contract. It was an
implementation validator defect. The exact sealed D1 fragment defines:

```rust
const D1_SINGLE_WORKER_SENTINEL: u8 = u8::MAX;
```

and `d1_run_component_single()` passes that sentinel to every single-route
component sample. The sealed D1 C-SINGLE sample stream also contains worker ID
`255` in every record. The D2 U-SINGLE stream reproduced the same representation.

## Immutable History

```text
historical execution verdict       BLOCKED_SEMANTIC
historical receipt SHA-256         46d52ac863e25da861f803096a6918a47a1f4b7138c0167c1f4724ad7b26dac8
u-single marker                    consumed-before-exec
U-SINGLE subject invocations       1
retry permitted                    false
```

This correction does not rewrite the historical receipt or state, recreate the
marker, or authorize another U-SINGLE execution.

## Offline Recovery

One independent reader-only salvage may verify:

```text
exact historical receipt and complete SHA256SUMS
exact D1 fragment SHA-256 bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665
exact D1 C-SINGLE sample stream
exact D2 U-SINGLE sample stream
record width 118 bytes
records 7,640
D1 worker IDs [255]
D2 worker IDs [255]
errors / unresolved 0 / 0
structure SHA-256 90d24adee563be803c390b41b18b41624b999db37b34c26650cb362f03d06712
thermal throttle drift empty
CPU/edge delta 0.3674526002160794 percent, at most 5 percent
live original state and marker projection unchanged before and after
```

It must execute no D2 ELF, Cargo, rustc, perf, PMU, parity, U, V or T route and
must perform no remote write.

## Effective Interpretation

If every offline check passes, the derived receipt may state:

```text
U_SINGLE_RECOVERED_FROM_SEALED_EVIDENCE
effective route state U_SINGLE_PASS
next action U-FIXED only
```

Future controllers must pin the derived receipt, preserve the historical
`BLOCKED_SEMANTIC` state, and accept the overlay only for this exact sentinel
predicate. Any other historical violation, byte drift, incomplete evidence or
live projection change remains terminal `BLOCKED_PROVENANCE`.

