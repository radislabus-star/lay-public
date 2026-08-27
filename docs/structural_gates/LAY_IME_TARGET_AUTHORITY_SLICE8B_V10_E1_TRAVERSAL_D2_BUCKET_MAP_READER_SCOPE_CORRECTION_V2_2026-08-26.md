# D2 bucket-map reader scope correction V2

Date: 2026-08-26

## Supersession

V1 correctly bounded `objdump` to the three sealed ownership symbols, but its
retained `--line-numbers` flag still forced the remote binutils reader to index
the complete 318 MB LTO+DWARF image. The first bounded reader remained CPU
active for more than ten minutes without emitting output. It was terminated
before marker consumption; its parent removed the pre-marker stage. V1 remains
historical evidence but is not the effective reader command.

## Effective command split

`objdump` owns only instruction boundaries, machine bytes and assembly:

```text
/usr/bin/objdump --disassemble --demangle --wide \
  --start-address=<sealed-symbol-start> \
  --stop-address=<sealed-symbol-end> <D2-ELF>
```

The three exact symbol intervals remain:

```text
0x778320..0x7793ae  d1_enumerate_lane_prepared::<false>
0x926520..0x926643  V13DafsaView::edge
0x9266b0..0x926808  V13DafsaView::state
```

The separate frozen `addr2line --functions --inlines --demangle --addresses`
reader remains the sole source of source-line and inline-frame ownership for
every instruction in those intervals. Removing duplicate line lookup from
`objdump` therefore removes no map evidence and does not alter classification.

A benign read-only timing probe of the exact hot-symbol command completed in
0.69 seconds with maximum RSS 220,416 KiB. It produced no file, consumed no
marker, executed no ELF subject and opened no PMU event.

## Verdict

```text
D2_BUCKET_MAP_READER_SCOPE_REPAIRED_V2
V1 effective                 false
marker consumed              false
authoritative map generated  false
scientific route repeated    false
next admitted action         self-check, then one map publication
```
