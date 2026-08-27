# D2 bucket-map reader scope correction V3

Date: 2026-08-26

## Supersession

V2 correctly made all three remote `objdump` reads address-bounded and removed
duplicate line lookup. The remaining remote GNU binutils 2.38 `addr2line`
reader nevertheless spent more than twelve minutes processing the 318 MB
LTO+DWARF ELF without producing output. It was terminated before marker
consumption; the controller removed its pre-marker stage. V1 and V2 remain
historical evidence, while this V3 overlay is the effective reader route.

## Effective ownership split

The remote producer remains authoritative for:

```text
sealed remote ELF identity and direct byte hashes
three bounded objdump instruction and machine-byte streams
nm symbol boundaries
readelf ELF and PT_LOAD geometry
address-list reconstruction
marker consumption
map construction, validation and publication
```

Only source and inline-frame resolution moves to the local byte-identical
sealed ELF copied by the one admitted build transaction:

```text
docs/structural_gates/receipts/
LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUILD_V1_2026-08-25/
REMOTE_EVIDENCE/d2-test-elf
```

The local controller verifies mode `0555`, size `317706232`, ELF SHA-256
`bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178`,
reconstructs instruction starts from the same three sealed virtual-address
intervals, and runs one cached GNU addr2line 2.46 invocation:

```text
/usr/bin/addr2line --exe=<sealed-local-D2-ELF> \
  --functions --inlines --demangle --addresses <exact-address-list>
```

Two independent local preparations produced the same closed evidence:

```text
instruction addresses             1064
address-list SHA-256              fca1804a3d0af34ae462938f970fd181706846da6e42dbb725c5ef1775581c58
addr2line output bytes            697799
addr2line output SHA-256          8b9b4767557a3ea019bbaebb280d1a56ab2180f34ad1e05aed9c2affb4c8a9e6
```

The remote producer must independently reconstruct the address list from its
own bounded disassembly, require the exact address-list SHA, verify the local
ELF and output identities carried in the payload, parse every address exactly
once, and seal the transferred output plus producer metadata. Any mismatch
fails before marker consumption.

## Unchanged boundaries

This correction changes no D2 ELF, Build ID, machine range, classification
rule, bucket taxonomy, map join key, failure taxonomy, marker ledger, parity,
U/V/T route, runtime authority or scientific claim. Local `addr2line` is a
read-only interpretation of already sealed bytes; it does not execute the ELF,
open PMU events or repeat a scientific route.

## Verdict

```text
D2_BUCKET_MAP_READER_SCOPE_REPAIRED_V3
V1 effective                 false
V2 effective                 false
marker consumed              false
authoritative map generated  false
scientific route repeated    false
next admitted action         static self-check, live marker recheck, one map publication
```
