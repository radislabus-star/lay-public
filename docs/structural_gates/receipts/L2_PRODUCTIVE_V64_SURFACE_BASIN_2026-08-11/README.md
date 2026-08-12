# L2 Productive V64 Surface-Basin Diagnostic

Status: `PASS_MECHANISM`, `FAIL_PROMOTION`.

V64 tested one paper-approved mechanism against the frozen V63 artifacts:
coalesce generated candidates before the global top-32 by
`(lemma_id, target_slot_id, normalized_surface)`. Different morphology slots
remain different basins even when their surface text is identical. The physical
limits remain `16 / 32`; coefficients, authority thresholds, SafetyGate,
verifier, raw corpus, and transition induction were not changed.

## Frozen Inputs

```text
corpus sha256    85d9b5493e22c96569e3b331cc0059ae80853bd98e976c626ca8f791e75f22a6
corpus bytes     434,934,248
work directory  /home/e/projects/lay-productive-v1-build-20260811/work/full-v1-v63-reinduce
workers         19
V63 package     /home/e/projects/lay-productive-v1-build-20260811/out/LAY-L2-PRODUCTIVE-PARADIGM-V1-SHADOW-V63.p2m
V63 sha256      5b80513cb33d3b82b4b9829742ecab6e4fc3248694f215d252901b630b122238
```

The V64 resume receipt confirms reuse of the complete raw pass, external sort,
ownership reduce, transition induction, context replay, and context sort.

## Package

```text
path             /home/e/projects/lay-productive-v1-build-20260811/out/LAY-L2-PRODUCTIVE-PARADIGM-V1-SHADOW-V64.p2m
bytes            17,309,944 (16.51 MiB)
sha256           9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
mmap backed      true
constant cache   124 B
bindings         94,151
paradigms        2,099
programs         77,854
operations       270,449
terminals        72,104
trie nodes/arcs  82,346 / 80,247
authority        shadow SuggestOnly
```

Release build took `2:58.30`, used `244%` average CPU, and reached
`2,195,220 KiB` peak RSS. Frozen-artifact resume took `64.00 s` and reached
`620,480 KiB` peak RSS. A second independent resume produced a byte-identical
package with the same SHA-256.

## Fixed Proof

Denominator: `13 classes x 100 x 2 cohorts = 2,600`. Probed and ordinary
readouts were executed independently. Full structural parity was
`2,600 / 2,600`, with zero mismatches.

```text
cohort          cases    H     B    S0    S1    S2    S3   top-1  top-16     R  empty
SEEN_EXACT      1,300    -     -     -     -     -     -      455   1,297  1,297      0
LEMMA_HELDOUT   1,300 1,280 1,219 1,219 1,219 1,219 1,219      267   1,218  1,219     51
```

The exact `LEMMA_HELDOUT` first-loss decomposition is:

```text
outside target-POS train-learned hypothesis H     20
H covered, no oracle-compatible target-blind B    61
B retained, target slot absent at S0               0
S0 present, exact execution absent at S1            0
S1 present, lost by per-binding slot bound S2       0
S2 present, lost by global surface-basin bound S3   0
S3 present, lost by final readout R                  0
R retained but below display top-16                  1
```

Per-class `LEMMA_HELDOUT` counts out of 100:

| Class | H | B | S0 | S1 | S2 | S3 | R | top-16 | top-1 | p99 ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| adjacent transposition | 98 | 93 | 93 | 93 | 93 | 93 | 93 | 93 | 25 | 45.161 |
| double substitution | 100 | 95 | 95 | 95 | 95 | 95 | 95 | 95 | 20 | 75.365 |
| extra letter | 98 | 94 | 94 | 94 | 94 | 94 | 94 | 94 | 21 | 48.538 |
| layout projection | 99 | 92 | 92 | 92 | 92 | 92 | 92 | 92 | 24 | 81.651 |
| letter substitution | 98 | 95 | 95 | 95 | 95 | 95 | 95 | 95 | 24 | 43.281 |
| missing letter | 98 | 93 | 93 | 93 | 93 | 93 | 93 | 93 | 21 | 45.090 |
| non-adjacent transposition | 99 | 95 | 95 | 95 | 95 | 95 | 95 | 95 | 22 | 71.876 |
| omission + transposition | 99 | 95 | 95 | 95 | 95 | 95 | 95 | 95 | 15 | 61.913 |
| prefix truncation | 99 | 94 | 94 | 94 | 94 | 94 | 94 | 94 | 24 | 66.772 |
| punctuation suffix | 98 | 95 | 95 | 95 | 95 | 95 | 95 | 95 | 30 | 45.364 |
| repeated fragment | 98 | 95 | 95 | 95 | 95 | 95 | 95 | 95 | 18 | 65.700 |
| sparse multi-omission | 100 | 93 | 93 | 93 | 93 | 93 | 93 | 93 | 16 | 86.001 |
| suffix truncation | 96 | 90 | 90 | 90 | 90 | 90 | 90 | 89 | 7 | 55.264 |

V63 to V64 movement on the same 1,300 heldout cases:

```text
metric                         V63       V64      delta
exact surface born          1,197     1,219        +22
exact in top-16             1,175     1,218        +43
raw unique top-1               40       267       +227
readout target retained     1,197     1,219        +22
empty lattice                  51        51          0
maximum class p99          144.976    97.519 ms  -47.457 ms
package bytes            17,309,944 17,309,944          0
```

All `2,600` verdicts remained `ABSTAIN`. False singleton and integrity errors
were zero. Proof self-reported RSS/peak RSS was `228,912 / 228,976 KiB`; the
instrumented proof elapsed `88.76 s` at `605%` average CPU. The longer total
proof time is not a runtime regression denominator because V64 additionally
executes the read-only probed route; per-call latency is measured around the
ordinary unprobed call only.

## Verdict

The surface-basin hypothesis is accepted. V64 recovered all measured losses at
`S1 -> S2 -> S3 -> R` without increasing package size or weakening authority.
The old V63 `L -> S` aggregate had conflated true compatibility/slot loss with
physical duplicate crowding.

V64 is not promotion eligible. The remaining owner is before generation:
`H -> B`; corrected target-POS ownership proves `B == S0`. Of the `61` binding
losses, `59` are absent from remaining source-slot postings and `2` fail exact
exposed-form reconstruction. Raw top-1 and every required per-class gate remain below the
strict contract, maximum class p99 remains above `5 ms`, and unsupported,
multi-label, slot-heldout, integrated L1.1/L3/L4/verifier, queue-inclusive, and
physical product gates were not tested. No later version is authorized by this
receipt.

## Receipts

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/resume-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/determinism-repeat-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/fixed-shadow-13x100-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/s0-intersection-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/hbs0-pos-diagnostic-13x100-receipt.json
```

The first two proof receipts are historical definitions. The target-POS
diagnostic receipt is authoritative for `H/B/S0`; package bytes and runtime
semantics are identical in all three runs.
