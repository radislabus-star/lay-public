# D2 bucket-map reader scope correction V1

Date: 2026-08-26

## Scope

This immutable overlay changes only the `objdump` read scope used to prepare
the sealed D2 machine bucket map. It does not change the D2 ELF, Build ID,
machine classification rules, bucket taxonomy, marker ledger, map join key,
failure taxonomy, parity, U/V/T routes, or scientific claim boundary.

## Observed implementation fact

The exact V4 full-ELF command

```text
/usr/bin/objdump --disassemble --line-numbers --demangle --wide <D2-ELF>
```

used one CPU for more than 1,800 seconds on the 317,706,232-byte LTO+DWARF
ELF and exceeded the local controller timeout. The controller had deliberately
placed all reader invocations before marker consumption. Therefore the failed
preparation attempt produced no authoritative map, consumed no marker, ran no
subject, opened no PMU event and left `bucket-map.available` unchanged. The
controller-owned orphan reader was terminated by exact PID/argv identity; its
parent removed the pre-marker stage.

## Effective reader command

The full read is superseded by three deterministic address-bounded reads over
the already sealed machine ownership closure:

```text
/usr/bin/objdump --disassemble --line-numbers --demangle --wide \
  --start-address=0x778320 --stop-address=0x7793ae <D2-ELF>

/usr/bin/objdump --disassemble --line-numbers --demangle --wide \
  --start-address=0x926520 --stop-address=0x926643 <D2-ELF>

/usr/bin/objdump --disassemble --line-numbers --demangle --wide \
  --start-address=0x9266b0 --stop-address=0x926808 <D2-ELF>
```

The ranges are pinned by the sealed symbol table:

```text
d1_enumerate_lane_prepared::<false>  0x778320 + 0x108e
V13DafsaView::edge                   0x926520 + 0x0123
V13DafsaView::state                  0x9266b0 + 0x0158
```

`nm`, `readelf`, `addr2line`, direct ELF byte hashing and full `.text`
complement coverage remain unchanged. The producer still proves instruction
closure for each owned symbol and hashes every published range directly from
the sealed ELF. Unread unrelated disassembly is assigned only to the
`OUTSIDE_TRAVERSAL` complement and receives no mechanism ownership.

## Verdict

```text
D2_BUCKET_MAP_READER_SCOPE_REPAIRED
marker consumed              false
authoritative map generated  false
retry of scientific route    false
next admitted action         repeat pre-marker self-check, then one map publication
```
