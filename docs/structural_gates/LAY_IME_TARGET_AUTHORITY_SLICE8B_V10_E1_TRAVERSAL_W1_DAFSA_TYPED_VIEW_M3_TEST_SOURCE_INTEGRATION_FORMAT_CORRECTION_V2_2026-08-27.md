# DAFSA Typed View M3 Test Source Integration Format Correction V2

Date: 2026-08-27

Paper verdict: `M3_TEST_SOURCE_INTEGRATION_FORMAT_REPAIRED`

Implementation authority: absent until a superseding code-route gate and
implementation preflight both pass.

## Defect In V1

The V1 integration paper and its implementation preflight pinned the sealed M3
sidecar as the fixed byte-view input:

```text
path     LAY-L2-RU-FULL-v13.dafsa
size     3,689,884 B
SHA-256  a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
magic    LAYV13D2
records  state=12 B / edge=12 B
```

The current source baseline is byte-identical to the archived active V11 source
and implements a different validated format:

```text
source SHA-256  d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b
magic           LAYV13D3
records         state=8 B / edge=8 B / symbol=4 B
```

Consequently, the current `V13DafsaView` cannot validate the sealed V10/M3
sidecar. A test that silently added a second V10 parser would violate the V1
rule that the current byte view remains the format validator. This is a real
post-preflight provenance defect, not an implementation detail.

V1 and both implementation preflight receipts remain immutable history. The
V2 `READY_TO_IMPLEMENT` receipt is superseded before any source edit.

## Corrected Fixed Input

The current source already owns deterministic V11 sidecar construction. The
sealed V11 Gate A receipt pins the exact output for the same canonical package:

```text
receipt SHA-256  6631b4fb2d0ba7d47008ab577801ee2f4bf6e2b6facc5c99b79fff7f7c2680e9
Gate A            PASS
sidecar size       2,460,144 B
sidecar SHA-256    5ebffb813ba0ca1e0080ec01756a2dafc51346297558d37cdd135abfde6acfaa
states             81,128
edges              226,341
symbols            34
root               81,127
record widths       8 / 8 / 4 B
full roundtrip      1,875,032 / 1,875,032
rank mismatches     0
```

The V11 receipt has overall verdict `FAIL_V11_A_B_C`; this correction consumes
only its independently passing Gate A format and roundtrip evidence. It does
not inherit V11 Gate B, Gate C or promotion authority.

The corrected proof performs exactly one in-memory reconstruction:

```text
exact V13 package
  -> existing current compile_sidecar
  -> assert 2,460,144 B and SHA-256 5ebffb81...
  -> current V13DafsaView::from_bytes validation
  -> one typed materialization
  -> all fixed searches borrow that typed generation
```

No sidecar file is written or published. The sealed V10/M3 sidecar remains
byte-identical historical evidence and is not passed to the current loader.

## Typed Payload Boundary

Encoded V11 records remain `8/8/4` bytes. Safe decoded materialization stores
the M3 hot-path fields as typed records:

```text
typed state  first_edge / suffix_count / edge_count / flags       12 B
typed edge   symbol / target / rank_delta                         12 B
payload      (81,128 + 226,341) * 12 = 3,689,628 B
```

The typed payload identity therefore remains the measured M3 logical layout,
while the validated input encoding changes to the current V11 format. No
native reinterpretation or unsafe cast is admitted.

## Lifetime And Failure

For one focused proof invocation:

```text
sidecar reconstructions  1
byte-view validations    1
typed materializations   1
fixed cases              382 forward + 382 reversed
```

Any package, reconstructed SHA, byte length, format, decoded field, count,
root, symbol digest, rank, candidate, certificate, completeness or work mismatch
is terminal `BLOCKED_PROVENANCE` or `BLOCKED_PARITY`. There is no fallback to
the old V10 parser and no second construction.

## Preserved Scope

All other V1 boundaries remain effective:

```text
source edits       only v13_typed_peak.rs + typed_exact.rs
production source  byte-identical
runtime authority  unchanged
network/remote     none
install/restart    none
perf/PMU           none
performance claim  none
next positive step actual-owner consequence paper only
```

The M3 `11.7954%` gain remains scoped to the sealed M3 ELF. This integration
proves semantic and lifetime transfer into current source, not codegen or
end-to-end latency transfer.

## Effective Tree

```text
V1 design + preflight V2 READY
  -> format incompatibility discovered before source edit
  -> V2 READY superseded
  -> format correction V2
  -> corrected code-route gate
  -> corrected implementation preflight
  -> test-only implementation
  -> one focused local proof
  -> M3_TEST_SOURCE_INTEGRATION_PASS or terminal BLOCKED_*
```
