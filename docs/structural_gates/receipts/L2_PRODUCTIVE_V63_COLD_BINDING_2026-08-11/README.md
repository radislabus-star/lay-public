# L2 Productive V63 Cold-Binding Diagnostic

V63 tested complete cold-lemma binding and cross-length paradigm execution
without rebuilding the immutable L1.1 or canonical L2 packages.

## Artifacts

```text
reinduce-receipt.json
reinduce.time.txt
reinduce.log
fixed-shadow-13x100-receipt.json
fixed-shadow-13x100.time.txt
fixed-shadow-13x100.log
```

Remote package:

```text
/home/e/projects/lay-productive-v1-build-20260811/out/LAY-L2-PRODUCTIVE-PARADIGM-V1-SHADOW-V63.p2m
bytes       17,309,944
sha256      5b80513cb33d3b82b4b9829742ecab6e4fc3248694f215d252901b630b122238
```

## Measured Result

```text
cohort          cases    L lemma    S exact    top-1    top-16    readout    empty
SEEN_EXACT      1,300    100.00%     99.77%    35.00%    99.77%     99.77%       0
LEMMA_HELDOUT   1,300     96.08%     92.08%     3.08%    90.38%     92.08%      51
```

Relative to V62, `LEMMA_HELDOUT` lemma birth moved from `9.00%` to
`96.08%`, exact-surface birth from `7.77%` to `92.08%`, top-16 from
`7.69%` to `90.38%`, and empty lattices from `1,183` to `51`.
`SEEN_EXACT` exact birth moved from `64.08%` to `99.77%`.

Safety and resources:

```text
verdict                          FAIL_measured_shadow_gates
Winner / Tied / ABSTAIN          0 / 0 / 2,600
false singleton                 0
integrity errors                0
unsupported false authority     NOT TESTED
runtime authority changed       false
cold mmap load                  121.505 ms
proof RSS / peak RSS            226,780 / 226,780 KiB
maximum class p99               144.976 ms
proof elapsed                   59.30 s
reinduce elapsed                38:31.45
reinduce peak RSS               611,204 KiB
```

## Verdict Scope

V63 validates the cold-binding mechanism and compact package representation,
but it does not pass promotion. The first shared measured loss in
`LEMMA_HELDOUT` is after lemma birth and before exact target generation. The
worst row is suffix truncation: `L=93%`, exact generation `76%`, top-16 `63%`.
An independent `B=true paradigm retained` counter is absent, so the proof
cannot distinguish binding loss from execution/generation loss.

All 2,600 readouts abstained. The proof carries one valid target per event but
does not measure the `MULTI_LABEL` or `UNSUPPORTED` partitions, so it cannot
justify forcing morphology-only top-1 or claim zero false authority. No later
version was named, implemented, compiled, or launched from this result.
