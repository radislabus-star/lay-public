# Exact English guard startup memory — 2026-10-24

The complete performance lane found a real budget failure in
`v27_exact_en_guard_controlled_resource_budget`: incremental RSS/PSS 15420 KiB,
against the unchanged RSS limit 15360 KiB and PSS limit 16384 KiB. Anonymous
growth was 7932 KiB and repeated warmup added zero. The receipt contained 139370
English entries, 312 protection entries, 3432552 logical resident bytes and
authority fingerprint 14251413822242752713. Receipt:
`weekend-release-gates/performance/performance/logs/lib-lay-exact_layout_authority--proof--v27_exact_en_guard_controlled_resource_budget.log`
under the remote KDE install-control root. The other ten performance checks pass.

The immutable bank previously allocated owned strings independently for two
overlapping dictionaries, then freed duplicates after merging. The allocator
can retain those pages. The change sorts/deduplicates borrowed ASCII tokens
before allocating the final unique bank. It preserves the existing filters,
case normalization, protected-word inclusion, sorted order and lookup function.
No thresholds, model data, authority owner or input route change.

First measurement after deduplicating borrowed tokens: all eleven performance
checks PASS. RSS/PSS delta 13800 KiB, anonymous delta 6568 KiB, repeated anonymous
growth 4 KiB. All entry counts, logical resident bytes and the complete authority
fingerprint equal the baseline. Receipt: `weekend-performance-bank-v2` in the
same control root. Resource test total elapsed was 0.988s versus 0.680s before;
this includes warmup/measurement and is not a per-key latency comparison.
One refinement normalizes owned dictionary buffers in place before bytewise
sorting, avoiding repeated case folding inside the comparator. Protected tokens
retain their prior bytes. Its parity and final measurements are pending.

Verification must compare every installed dictionary token and the complete
bank fingerprint with the previous construction, plus case/punctuation and
protected-token controls. Run the same fixed eleven performance tests and full
source gates. No re-fit or new lexical exceptions. The expected allocation
reduction is a hypothesis until the resource receipt is measured. Native client
latency remains untested for changed bytes. Runtime authority is unchanged.

Final bank-construction refinement: both complete-bank parity tests PASS and
all eleven performance tests PASS (`weekend-bank-parity-v3`,
`weekend-performance-bank-v3`). RSS/PSS delta 13672 KiB, Pss_Anon delta
6568 KiB, repeated Pss_Anon growth 4 KiB. Counts, logical bytes and authority
fingerprint still exactly match the baseline. Resource test total elapsed
0.688s; hot/cold readout budget controls pass. Graph refresh embeds a new
architecture receipt in src/generated; its source-identity change correctly
prevented reusing an older performance receipt. The final release pipeline
therefore reruns the fixed performance lane against that final source snapshot.
