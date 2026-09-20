# L2 Canonical Architecture Above L1.1

Status: canonical L2 ownership is closed for the live local IME/daemon route:

```text
L1.1 bounded lattice
-> standalone V13 CanonicalL2Field
-> one Winner | Tied | Abstain readout
-> L3
-> TransitionDecisionCore
-> verifier
```

Last source audit: 2026-08-01.

Runtime authority: unchanged by the ownership cleanup. The live default is
`CandidateReadoutRoute::CanonicalL2Field`; `FullWave` remains a compare-only
reference. The immutable V13 package is loaded directly and was not rebuilt.
There is no executable same-lemma or near-neighbor donor fallback in
`src/nanda_wave/l2_field/bridge.rs`.

Sections that describe `L2FieldShadow`, morphology donors, or near-neighbor
donors are retained below as dated implementation history. They do not describe
the current executable owner after 2026-08-01.

## 0. Canonical Live Owner Closure, 2026-08-01

The current code ownership is:

```text
CorrectionRequest
-> CandidateReadoutRoute::CanonicalL2Field
-> canonical_text_readout()
-> bounded L1.1 seed surfaces
-> StandaloneL2Field::readout() over immutable V13
-> CanonicalL2FieldReadout { candidates, authority }
-> one shared candidate lattice
```

Measured facts:

- installed V13 bytes: `135121803` (`128.86 MiB`);
- installed V13 SHA-256:
  `bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e`;
- package SHA before and after the code cutover: identical;
- warmed before/after semantic snapshot: `8 / 8` inputs identical;
- candidate records compared: `86` before and `86` after;
- semantic snapshot SHA-256 before and after:
  `08e25753179ff608ef96ab968f8585803e337afc0d3701337fee69160ae1f418`;
- selected-surface divergence: `0 / 8`;
- selected-gate divergence: `0 / 8`;
- standalone V13 fixed proof remains the quality authority:
  same-lemma false authority `0`, near-neighbor false authority `0`;
- focused `nanda_wave::l2_field` proof: `26 passed / 0 failed` after removal
  of eight test-only donor tests and their dead implementation.
- remote 20-job release build: `110.43 s`, average CPU `203%`, peak RSS
  `1563256 KiB`, swaps `0`;
- remote focused test build/run: `26 / 26` passed in `10.96 s`, average CPU
  `156%`, peak RSS `1412604 KiB`.

The before/after snapshot compares replacement, error class, gate action,
winner/none, scoreboard, candidate count, and candidate ordering. Diagnostic
route names, source IDs, and reason strings are intentionally renamed from
`Shadow` to `Canonical`; they are not field geometry.

What was not tested in this ownership-only change:

- no L1.1 or L2 package was recompiled;
- no new L2 quality training was run;
- no L3/L4 behavior was promoted;
- the pre-existing environment-sensitive tests for `звгрузи` and IME
  transposition authority remain separate test debt; the same failure was
  reproduced from baseline commit `a5188a5`.

Verdict: `PASS_canonical_live_owner`.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_LIVE_OWNER_CUTOVER_2026-08-01.json`

Runtime authority changed: `false`.

## 25. 2026-08-10 Productive Worker Cache V32

V32 kept V30 semantics and moved the geometry cache from one lemma invocation
to one Rayon worker. It also precomputed family suffix lanes once per source and
read encoded family specificity without decoding the suffix payload again.

```text
configuration                    workers   generated birth p50 / p99
V30 per-lemma geometry                 1                3.008 / 7.705 ms
V30 per-lemma geometry                20              12.731 / 73.085 ms
V32 worker-local geometry              1                2.820 / 6.866 ms
V32 worker-local geometry             20              10.084 / 71.872 ms

V32 workers=20 wall / peak RSS                         6.15 s / 377,432 KiB
V32 workers=1 wall / peak RSS                          6.65 s / 337,012 KiB
```

Class SHA, directional counters and false-authority counters remain unchanged.
Verdict: `PASS_optimization_FAIL_latency`. Runtime authority changed: `false`.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WORKER_CACHE_V32_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WORKER_CACHE_V32_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WORKER_CACHE_V32_BUILD_2026-08-10.time.txt
```

## 26. 2026-08-10 Productive Normalized Geometry V33

V33 removes repeated Unicode lowercase work and the second normalized
`String` allocation while scoring generated surfaces. Character and keyboard
geometry are computed directly from the already normalized generated surface.
The bounded `256 / 256 / 16 / 32 / 196608` frontier is unchanged.

Measured on the same fixed `13 x 10` proof:

```text
configuration                         workers   generated birth p50 / p99
V32 worker-local geometry                   1                2.820 / 6.866 ms
V32 worker-local geometry                  20              10.084 / 71.872 ms
V33 normalized-surface geometry             1                2.838 / 6.987 ms
V33 normalized-surface geometry            20              13.923 / 65.649 ms

V33 workers=20 wall / peak RSS                              6.18 s / 382,252 KiB
V33 workers=1 wall / peak RSS                               6.67 s / 336,852 KiB
generated top-16, every class                                             100%
generated unique top-1, mean / worst                              85.385% / 70%
directional target wins / reverse false supports                       69 / 0
false authority / false singleton                                      0 / 0
```

The complete class, directional, safety, sidecar and frontier summaries are
byte-identical to V32. V33 improves the concurrent p99 by `8.7%`, but regresses
the one-worker p99 by `1.8%` and still misses the `<=5 ms` gate. It is retained
as a semantics-preserving implementation simplification, not accepted as the
final speed solution. Verdict: `PASS_parity_FAIL_latency`. Runtime authority
changed: `false`.

Not tested by this experiment: the larger fixed proof, clean/ambiguity
preservation, live L3 selection, daemon/IBus latency and installed clients.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_NORMALIZED_GEOMETRY_V33_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_NORMALIZED_GEOMETRY_V33_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_NORMALIZED_GEOMETRY_V33_BUILD_2026-08-10.time.txt
```

## 27. 2026-08-10 Rejected Per-Rule Bound V34

V34 tested exact bounded top-32 retention inside each selected productive
family range. A rule skipped geometry only when its rank with perfect geometry
could not enter the current top-32. Quality and safety remained exactly equal
to V33, but latency regressed:

```text
configuration                      workers   generated birth p50 / p99
V33 normalized geometry                  1                2.838 / 6.987 ms
V33 normalized geometry                 20              13.923 / 65.649 ms
V34 per-rule bound                       1                2.970 / 7.649 ms
V34 per-rule bound                      20              12.302 / 69.901 ms

V34 workers=20 peak RSS                                      384,148 KiB
V34 workers=1 peak RSS                                       336,696 KiB
false authority / false singleton                                  0 / 0
```

Direct sidecar inspection explains the rejection: `1,268,069` exact family
ranges have mean `1.0001`, p99 `1` and maximum `3` rules; no exact family range
contains more than `32` rules. The large dimension is one level higher:
`1,183` source-target transitions contain mean `1,072`, p99 `20,301` and maximum
`28,437` family variants. The per-rule container optimized an absent inner
fanout and added comparison overhead. V34 code was removed. Verdict:
`REJECT_wrong_bound_level`. Runtime authority changed: `false`.

Not tested: a lemma/slot-level upper bound, the larger fixed proof, L3 handoff,
daemon/IBus latency and installed clients.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_EXPANSION_V34_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_EXPANSION_V34_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_EXPANSION_V34_BUILD_2026-08-10.time.txt
```

## 28. 2026-08-10 Rejected Lemma Bound V35

V35 moved the exact upper bound from individual rules to whole lemma
hypotheses. It generated a sequential seed top-32, then retained every
remaining lemma whose perfect profile and geometry could still meet the seed
joint-evidence cutoff. The `256 / 256 / 16 / 32 / 196608` frontier and final
quality remained unchanged.

```text
configuration                      workers   generated birth p50 / p99
V33 normalized geometry                  1                2.838 / 6.987 ms
V33 normalized geometry                 20              13.923 / 65.649 ms
V35 lemma upper bound                    1                3.304 / 7.939 ms
V35 lemma upper bound                   20              12.927 / 71.205 ms

V35 workers=20 peak RSS                                      380,964 KiB
V35 workers=1 peak RSS                                       336,852 KiB
false authority / false singleton                                  0 / 0
```

The seed serialization cost exceeded any work removed by the bound. V35 code
was removed. Verdict: `REJECT_seed_serialization`. Runtime authority changed:
`false`.

Not tested: function-level CPU profile, larger fixed proof, L3 handoff,
daemon/IBus latency and installed clients.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BOUND_V35_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BOUND_V35_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BOUND_V35_BUILD_2026-08-10.time.txt
```

## 24. 2026-08-10 Productive Warm Repeat V31

V31 added a proof-only environment switch that evaluated all fixed cases once
before the measured pass in the same process. It warmed canonical-source
`OnceLock`s, decoder blocks, mmap pages and generated rule paths without
changing packages, candidates or authority.

```text
configuration                     workers   generated birth p50 / p99
V30 normal cold process                 1                3.008 / 7.705 ms
V31 full warm repeat                    1                2.957 / 7.727 ms
V30 normal cold process                20              12.731 / 73.085 ms
V31 full warm repeat                   20             14.620 / 172.221 ms
```

The warmup itself took `1.164 s` with one worker and `0.656 s` with twenty.
Quality counters remained unchanged with zero false authority/singleton.

Verdict: `REJECT_cold_source_hypothesis`. The one-worker tail is unchanged and
the concurrent warm repeat increases contention. Embedding all canonical
sources into the productive sidecar is therefore not justified as a latency
fix. The proof-only warmup must not enter the runtime route.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WARM_REPEAT_V31_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WARM_REPEAT_V31_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_WARM_REPEAT_V31_BUILD_2026-08-10.time.txt
```

Runtime authority changed: `false`.

## 23. 2026-08-10 Productive Geometry Cache V30

V30 restored the accepted V26 decoder cache and changed only duplicate work
inside productive generation:

```text
one lemma basin
-> generated surface
-> geometry computed once per distinct surface
-> reused across competing morphology slots
-> hash dedup
-> unchanged deterministic total sort and form_limit=32
```

No package, frontier, evidence, rank tuple or authority rule changed.

```text
configuration                    workers   generated birth p50 / p99
V26 decoder cache                      1                3.218 / 8.336 ms
V26 decoder cache                     20              16.115 / 73.670 ms
V30 geometry cache                     1                3.008 / 7.705 ms
V30 geometry cache                    20              12.731 / 73.085 ms

V30 workers=20 wall / peak RSS                         6.18 s / 367,324 KiB
V30 workers=1 wall / peak RSS                          6.69 s / 337,016 KiB
false authority / singleton                                             0 / 0
exact target wins / reverse false supports                             69 / 0
```

Class and directional summary SHA values are byte-identical to V26. V30 is
accepted as a lossless local optimization, but not as promotion evidence:
one-worker p99 remains `1.54x` over budget and concurrent p99 remains `14.62x`
over budget.

Verdict: `PASS_optimization_FAIL_latency`. Runtime authority changed: `false`.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_WORKERS20_13X10_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_WORKERS1_13X10_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_CACHE_V30_BUILD_2026-08-10.time.txt
```

## 22. 2026-08-10 Associative Cache V28 And Parallelism Probe V29

V28 retained the V27 O(1), read-only hit contract but restored full
associativity inside every shard through `HashMap block -> slot` and bounded
round-robin eviction on misses.

```text
configuration                       workers   generated birth p50 / p99
V26 associative LRU                       1                3.218 / 8.336 ms
V26 associative LRU                      20              16.115 / 73.670 ms
V28 indexed associative                   1                3.346 / 9.637 ms
V28 indexed associative                  20             13.512 / 107.674 ms
```

V28 preserved the V26 class/directional SHA values, `69/0` directional
wins/reverse supports and `0/0` false authority/singleton. It nevertheless
regressed both p99 modes, so the O(1) cache-index hypothesis is rejected.

V29 kept V28 code and forced `RAYON_NUM_THREADS=1` to test ownership of
parallelism:

```text
outer workers / inner Rayon       generated birth p50 / p99   process CPU
20 / 1                                      287.544 / 366.065 ms       105%
 1 / 1                                       14.435 / 34.380 ms         99%
```

The inner Rayon lane is necessary for current per-request latency. Merely
moving parallelism to outer proof workers serializes generation through the
single global Rayon lane and does not reduce core work.

Verdict: `REJECT_V28_V29`. V26 remains the best measured storage baseline. The
next optimization target is repeated productive rule expansion, generated
surface construction, profile allocation and geometry calculation before the
final bounded readout. Package/frontier/evidence must remain unchanged.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_ASSOCIATIVE_CACHE_V28_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_ASSOCIATIVE_CACHE_V28_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_ASSOCIATIVE_CACHE_V28_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_PARALLELISM_V29_OUTER20_INNER1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_PARALLELISM_V29_OUTER1_INNER1_13X10_2026-08-10.json
```

Runtime authority changed: `false`.

## 21. 2026-08-10 Productive Direct Decoder Cache V27

V27 tested a bounded direct-mapped decoder cache. Each of the `16` shards held
`128` fixed slots behind an `RwLock`; a cache hit used O(1) slot lookup and did
not mutate recency state. Package bytes, productive evidence and all frontiers
were unchanged.

```text
configuration                    workers   generated birth p50 / p99
V26 associative LRU                    1                3.218 / 8.336 ms
V26 associative LRU                   20              16.115 / 73.670 ms
V27 direct mapped                      1               3.648 / 10.118 ms
V27 direct mapped                     20             14.949 / 218.109 ms

V27 workers=20 wall / peak RSS                         6.21 s / 359,748 KiB
V27 workers=1 wall / peak RSS                          6.86 s / 336,744 KiB
false authority / singleton                                             0 / 0
exact target wins / reverse false supports                             69 / 0
```

The V27 class and directional summary SHA values are byte-identical to V26.
The direct slot mapping nevertheless causes collision-thrashing under the
multi-request access pattern. Its p99 is `3.0x` worse than V26 and `43.6x`
above the accepted `5 ms` gate.

Verdict: `REJECT_direct_mapped_cache`. The accepted next direction is a bounded
fully-associative shard with O(1) block-to-slot lookup, read-only cache hits and
replacement only on misses. Runtime authority changed: `false`.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_WORKERS20_13X10_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_WORKERS1_13X10_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECT_CACHE_V27_BUILD_2026-08-10.time.txt
```

## 20. 2026-08-10 Productive Decoder Cache V26

### What Was Tested

V26 tested one storage-level latency hypothesis without changing morphology
evidence, readout, package bytes or bounded frontiers:

```text
compact decoder block miss
-> decode outside shard Mutex
-> recheck shard cache after decode
-> insert only when still absent
```

The decoded-form cache capacity changed from `16 x 32 x 32 = 16,384` to
`16 x 128 x 32 = 65,536` forms. The fixed proof retained the canonical limits:

```text
broad lemma frontier              256
active lemma frontier             256
features per lemma                 16
form lattice                       32
atom relation budget          196,608
```

No L1.1, canonical L2 or productive sidecar package was recrystallized. The
sidecar remained `81,688,382 B`, with `1,268,215` profiles and `7,191`
directional pairs.

### Measured Facts

Remote release build on `e@192.168.3.94` completed with exit `0`:

```text
build wall                       2:16.52
build peak RSS             1,928,024 KiB
```

The unchanged fixed `13 x 10` proof compared V25 and V26:

```text
configuration                 workers   generated birth p50 / p99
V25 source cache                    1                3.569 / 7.335 ms
V25 source cache                   20              17.403 / 93.280 ms
V26 decoder cache                   1                3.218 / 8.336 ms
V26 decoder cache                  20              16.115 / 73.670 ms
```

V26 workers=20 completed in `6.17 s`, used `268%` average CPU and peaked at
`358,136 KiB` RSS. V26 workers=1 completed in `6.74 s`, used `264%` average CPU
and peaked at `337,016 KiB` RSS. The apparent CPU above one core in the
workers=1 proof includes package/runtime helper work; it is not twenty proof
workers.

Quality and directional evidence did not move:

```text
class summary SHA        d4aec55925b462c54a6a1004e1e3faba0f2366a85e306f4f2c6bd2b5cfa0dcdf
directional summary SHA  72fbbbd2b9205a7cc895a432e789b219cbc7b5daab35d2f373c15a7ec2d307f0
exact target wins        69
reverse false supports   0
false authority          0
false singleton          0
```

### Verdict Scope

Verdict: `FAIL_latency`. Moving decompression outside the mutex reduced the
20-worker p99 by `21.0%`, but `73.670 ms` remains `14.7x` above the accepted
`5 ms` gate, while single-worker p99 regressed from `7.335` to `8.336 ms`.
V26 is therefore not a promotable final configuration.

Tested: fixed generated-form quality parity, directional parity, cold package
load, proof wall time, CPU, RSS and generated-birth latency for one and twenty
workers.

Not tested: larger `13 x 100` or `13 x 20,000` proof, clean and ambiguity
retention after live integration, L3 final selection, daemon/IBus latency or
physical application behavior.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_WORKERS20_13X10_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_WORKERS1_13X10_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DECODER_CACHE_V26_BUILD_2026-08-10.time.txt
```

Runtime authority changed: `false`.

## 19. 2026-08-10 Productive Directional NH Gate V20-V22

### What Was Tested

The directional morphology relation received its own independent denominator.
The existing `13 x N` restoration proof samples `H` rows and therefore cannot
prove relations trained from `NT`. The new gate streams all `NH` rows, resolves
only requested competitor surfaces through `F` rows inside the target lemma,
and excludes cross-lemma competitors from morphology authority.

```text
NT real competitor observation
-> exact and neighbor pair evidence
-> NH independent context
-> same-lemma target slot versus competitor slot
-> target win | tied/no-evidence | reverse false support
```

The productive V2 sidecar was reused unchanged. No L1.1, canonical L2, or
productive package was recrystallized.

### Rejected V20 And V21 Readouts

V20 allowed the first available exact or neighbor relation to settle a slot:

```text
same-lemma comparisons                 3,483
pair coverage                          1,202  34.510%
target directional wins                  892
reverse false supports                    244
verdict                                  FAIL
```

V21 inspected exact, left-neighbor, and right-neighbor lanes independently and
required both neighbor lanes to agree:

```text
pair coverage                          1,479  42.463%
target directional wins                  189
reverse false supports                     24
verdict                                  FAIL
```

This rejects neighbor agreement as morphology authority. Two frequent lexical
neighbors can agree on the wrong form; changing a support multiplier cannot
turn that evidence into an independent grammatical observation.

### Accepted V22 Ownership

V22 gives directional authority only to an independently observed exact
competitor scene. Neighbor relations remain retention evidence and preserve a
`Tied` lattice for L3. This is an ownership boundary, not a word, suffix, or
manually weighted exception.

```text
NH rows                               42,195
competitor surfaces                  78,082
same-lemma competitor surfaces        2,451
same-lemma comparisons                3,483
pair evidence covered                 1,479  42.463%
exact target directional wins            69
reverse false supports                    0
tied pair evidence                    1,410
no pair evidence                      2,004
reverse invariant violations              0
directional verdict    PASS_shadow_directional_nh
```

The old `H`-row `13 x 10` class counters remained byte-for-byte equivalent to
the V19 baseline after canonical JSON normalization:
`333d30e07db6a2d11a45d5f8fa9cd06e8f9bfeee46b487d25023776063e5d5a8`.
Thus this experiment changed directional evidence interpretation without
masking the existing restoration denominator.

### Performance And Scope

```text
workers                                  20
directional NH scan                 2.247 s
whole proof wall                    6.50 s
average proof CPU                      307%
proof max RSS                      349,172 KiB
generated birth p50 / p99     77.731 / 133.857 ms
release build wall                  2:12.78
release build max RSS            1,923,864 KiB
```

The directional gate passes, but overall promotion remains `FAIL`: the existing
generated unique top-1 and `<=5 ms` latency gates still fail. Generated forms
remain `SuggestOnly`; no daemon or IBus authority was installed or restarted.

Not tested in V22:

- a larger fixed `13 x 100` or `13 x 20,000` generated-form denominator;
- clean and ambiguity retention after live generated-candidate integration;
- L3 final sentence-level selection and physical apply authority;
- installed daemon and IBus latency.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_RAW_V20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_TWO_LANE_V21_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_EXACT_V22_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_DIRECTIONAL_NH_EXACT_V22_13X10_2026-08-10.time.txt
```

Runtime authority changed: `false`.

## 18. 2026-08-10 Productive Morphology V7 Lemma-Basin Pareto

### What Was Tested

V7 restricted productive Pareto dominance to candidates inside the same
`lemma_id`. Different lemma basins remained in the bounded `Tied` lattice for
independent L1.1/L3 resolution. The V5 birth operator, corpus, heldout split,
limits, and all 130 proof scenes remained unchanged.

### Measured Facts

```text
cases                                      130
damage classes                              13
generated top-16, every class           100.0%
raw generated unique top-1 range         50-100%
cross-lemma target retention             100.0%
readout selected-target range            90-100%
Winner / Tied / ABSTAIN                 0 / 129 / 1
false singleton                               0
false authority                                0
debug training                          103.749 s
generated p99                            801.219 ms
peak RSS                           1,326,744 KiB
wall                                    128.82 s
average CPU                                155%
```

The only non-selected target was the deliberate `ABSTAIN` already identified
in V6: the damaged input was itself a valid generated form, while the target
remained present in the underlying top-16 lattice. Thus V7 closes unsafe
cross-lemma collapse on this micro denominator, but it does not close the
unique top-1 or production latency/RSS gates.

### Verdict And Scope

Overall promotion verdict remains `FAIL`: every final quality dimension is
still conjunctive, and raw unique top-1 is below `>95%` in several classes.
The narrower result is `PASS_shadow_retention`: productive L2 safely births and
retains unseen forms, then exposes unresolved lemma basins to L3 instead of
inventing authority.

Not tested: larger denominator, clean/ambiguity retention, live L3 handoff,
release latency, compact sidecar size, daemon or IBus behavior.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEMMA_BASIN_PARETO_V7_DEBUG_13X10_2026-08-10.json`.

Runtime authority changed: `false`.

## 17. 2026-08-10 Productive Morphology V6 Global Pareto Readout

### What Was Tested

V6 left the V5 morpheme birth unchanged and added a Pareto
`Winner | Tied | ABSTAIN` readout over independent lemma-atom,
damaged-surface geometry, and set-valued context-compatibility axes. A valid
generated input surface forced `ABSTAIN`; no generated candidate received live
authority.

### Measured Facts

```text
cases                                      130
damage classes                              13
generated top-16, every class           100.0%
raw generated unique top-1 range         50-100%
Pareto target retention range            90-100%
classes with 100% Pareto retention         11/13
Winner / Tied / ABSTAIN                108 / 21 / 1
false singleton                               1
false authority                                0
debug training                          103.308 s
generated p99                            696.428 ms
peak RSS                           1,326,524 KiB
wall                                    128.41 s
average CPU                                155%
```

The single false singleton exposed an ownership error rather than a missing
morpheme: global Pareto comparison allowed one lemma basin to erase another
using only lexical geometry plus positive-only context compatibility. The one
`missing letter` readout loss was a deliberate `ABSTAIN`, because the damaged
input was itself a valid generated form while the target remained in the
underlying top-16 lattice.

### Verdict And Scope

Verdict: `FAIL`. Global cross-lemma Pareto collapse is rejected. Productive L2
may settle morphology slots inside one lemma basin; cross-lemma authority
requires independent L1.1 or L3 evidence. V7 therefore preserves different
lemma basins as `Tied` while retaining the existing within-lemma Pareto
readout.

Not tested: larger denominator, clean/ambiguity retention, live L3 handoff,
release latency, compact sidecar size, daemon or IBus behavior.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_PARETO_READOUT_V6_DEBUG_13X10_2026-08-10.json`.

Runtime authority changed: `false`.

## 16. 2026-08-10 Productive Morphology V5 Edge Morphemes

### What Was Tested

V5 generalized the productive transform from suffix-only replacement to a
bounded `prefix + retained stem + suffix` operator. The direct suffix path
remains the fast path; the bounded edge search is used only when direct stem
retention cannot express the observed paradigm relation.

### Measured Facts

```text
cases                                      130
damage classes                              13
target lemma retention, every class     100.0%
generated top-16, every class           100.0%
generated unique top-1 range            50-100%
false singleton                              16
false authority                                0
admitted profiles                      1,267,969
debug training                          104.809 s
generated p50 / p99              336.953 / 774.631 ms
RSS after training                 1,280,036 KiB
peak RSS                           1,327,088 KiB
wall                                    130.30 s
average CPU                                156%
```

The edge operator added `21,644` admitted profiles over V4 and removed every
generated-surface top-16 loss on this micro denominator. This establishes the
productive birth contour for unseen Russian forms. It does not establish
unique authority.

### Remaining Readout Problem

The remaining failures are close competing lexical basins. Their independent
evidence axes disagree: one surface has stronger lemma evidence, another has
stronger damaged-surface geometry, and some damaged inputs are themselves valid
surfaces. A weighted scalar winner is unsafe here. The next readout is a
Pareto-evidence `Winner | Tied | ABSTAIN`: unique authority requires dominance
across independent axes; conflicting evidence remains a lattice for L3.

### What Was Not Tested

- Pareto tied/abstain coverage;
- a larger fixed denominator;
- release latency and compact sidecar size;
- live IME authority.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_EDGE_MORPHEME_V5_DEBUG_13X10_2026-08-10.json`.

Runtime authority changed: `false`.

## 15. 2026-08-10 Productive Morphology V4 Compatibility Lattice

### What Was Tested

V4 stopped treating other positive labels as anti-evidence. For each lemma an
observed context formed a set of compatible morphology-slot projections;
frequency inside that set did not affect authority. The generated readout used
`lemma x suffix-profile x damaged-surface geometry`, while context acted only
as a bounded compatibility selector.

### Measured Facts

```text
cases                                      130
damage classes                              13
target lemma retention, every class     100.0%
generated top-16 range                  90-100%
generated unique top-1 range            60-100%
false singleton                              17
false authority                                0
debug training                           98.491 s
generated p50 / p99              320.435 / 672.383 ms
peak RSS                           1,171,400 KiB
wall                                    123.48 s
average CPU                                157%
```

Compared with rejected V3, false singleton fell `50 -> 17`, the minimum
top-16 rose `60% -> 90%`, and the minimum top-1 rose `40% -> 60%`. The
set-valued compatibility interpretation is therefore retained as an
architectural result, but the configuration is not promoted.

### Remaining Shared Mechanisms

The remaining top-16 surface losses are concentrated in productive edge
rewrites, not in the 13 damage operators. A suffix-only rule cannot synthesize
the comparative prefix/suffix transformation `ходячий -> походячее`; a second
family loses the exact reflexive gerund surface while retaining its lemma and
slot. Other misses are close competing lexical basins and require calibrated
`Tied | ABSTAIN`, not corpus-item exceptions.

V4 also proved that positive-only coverage cannot be a permanent hard negative
gate. An unobserved slot is unlabeled, not contradictory. Future explicit
anti-centers must be trained from actual competitor observations.

### What Was Not Tested

- bounded prefix plus suffix morpheme transforms;
- explicit context-slot anti observations;
- ambiguity calibration for generated surfaces;
- a larger denominator or release latency;
- live runtime authority.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_COMPATIBILITY_V4_DEBUG_13X10_2026-08-10.json`.

Runtime authority changed: `false`.

## 14. 2026-08-10 Productive Morphology V3 Posterior Rejection

### What Was Tested

V3 streamed all explicit five-column `T` scenes without materializing the
corpus, projected morphology features without POS/aspect, built a marginal
context-slot posterior, applied Laplace smoothing to suffix profiles, and
ranked generated candidates by the product of lemma, context, profile and
damaged-surface geometry evidence.

### Measured Facts

```text
cases                                      130
damage classes                              13
target lemma retention, every class     100.0%
generated top-16 range                  60-100%
generated unique top-1 range            40-100%
false singleton                              50
false authority                                0
streamed train context rows              15,922
excluded context rows                         0
context modes                               105
context slots                               227
debug training                           98.994 s
generated p50 / p99              479.736 / 927.801 ms
peak RSS                           1,184,804 KiB
wall                                    125.25 s
average CPU                                171%
```

The zero excluded context rows are expected for this corpus split: selected
target names occur only in `H`, not in `T`. The explicit exclusion path was
nevertheless exercised by the focused unit proof.

### Rejected Mechanism

`V3` is rejected. It regressed V2 despite preserving every target lemma. The
teacher contexts are multi-label rather than mutually exclusive. For example,
`они _` legitimately contains past plural, present third-person plural and
short-adjective plural slots. Therefore `context total - target support` is not
independent anti-evidence, and marginal label frequency is not grammatical
authority. Multiplying that biased marginal into the rank allowed common slots
to override stronger L1.1 lemma and damaged-surface evidence.

The next contour must treat positive morphology slots as a set-valued
compatibility lattice. A context may reject an unobserved slot when grounded
contradictory evidence exists, but frequency differences among multiple
observed-compatible slots cannot manufacture authority. Explicit anti support
must come from actual competitor evidence, not from other positive labels.

### What Was Not Tested

- explicit morphology-slot anti scenes;
- unseen-context generalization;
- release latency;
- larger fixed denominators;
- live IME authority.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_CONTEXT_POSTERIOR_V3_DEBUG_13X10_2026-08-10.json`.

Runtime authority changed: `false`.

## 0.1 Productive Morphology Leave-Lemmas-Out V1, 2026-08-10

The first generative L2 prototype was measured separately from the live owner.
Its contour was:

```text
damaged surface
-> compositional lemma lattice
-> context-ranked MorphologySlot
-> suffix-family transform learned from other lemmas
-> generated surface
-> ShadowUnverified
```

The fixed micro proof selected `40` target lemmas from real `H` context rows,
excluded every selected lemma from productive-profile training, and evaluated
`13 x 10 = 130` damaged surfaces. The normal exact-bank denominator and the
exact-masked generated denominator were recorded separately. The immutable V13
package was not rebuilt; exact target lookup was disabled only in the generated
birth route.

Measured facts:

- train/heldout lemma overlap: `0`;
- train lemmas admitted: `93,632`;
- productive profiles admitted: `180,912`;
- context target-slot retention: `90–100%` by damage class;
- unseen generated top-16 retention: `60–90%`;
- unseen generated unique top-1: `40–90%`;
- false singleton: `40 / 130`;
- false authority: `0` because every generated birth remained
  `ShadowUnverified`;
- exact annotation leaks: `0`;
- profile training: `11.272 s`;
- generated birth p50 / p99: `980.946 / 1,635.043 ms`;
- peak RSS: `756,084 KiB` (`738.36 MiB`);
- whole proof wall time: `22.66 s`, average process CPU `643%`, swap growth `0`.

The first shared failure mechanism was not a list of individual words. The
runtime scanned every admitted rule for a source/target slot, admitted weak
generic suffix profiles into the same frontier, and ranked damaged-surface
geometry before morphology-family evidence. Context-slot birth was already
mostly retained, but the morpheme readout was both too broad and too slow.

Verdict: `FAIL`, rejected for promotion. The next experiment must replace the
slot-wide scan with longest-supported suffix-family postings and rank lemma,
context slot, family/profile, and surface geometry as distinct evidence stages.

What was not tested:

- a physical V13 package rebuilt without heldout target surfaces;
- clean preservation and ambiguity retention;
- live generated-candidate integration;
- L3/L4/DecisionCore authority;
- daemon or IBus latency.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LEAVE_LEMMAS_OUT_MICRO_13X10_2026-08-10.json`

Runtime authority changed: `false`.

## 1. Why This Document Exists

The current live route is now a mixed live-owner flip:

```text
deterministic candidates
+ L2FieldShadow local field
+ internalized L1.1 seeded birth
+ shadow same-lemma morphology donor
+ shadow near-neighbor donor
-> one correction lattice
```

That is good enough for continued development, but it is not a clean final
architecture. Right now:

- `/home/ubu/projects/lay/src/nanda_wave/lexical_grokking/restoration.rs`
  owns true `L1.1` lexical restoration readout;
- `/home/ubu/projects/lay/src/correction_core/candidate_sources.rs`
  still owns the live candidate merge, but the live route now resolves through
  `CandidateReadoutRoute::L2FieldShadow`;
- `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/field.rs`
  is a shadow morphology teacher, not the live canonical `L2`;
- `/home/ubu/projects/lay/src/nanda_wave/l2_candidate_phase.rs`
  is a transition-phase package, not the final `L2` above `L1.1`.

The new `L2` must become one explicit owner:

```text
L1.1
  restores damaged token signal

L2
  owns local competition between restored forms

L3
  owns broader phrase / semantic pressure

verifier
  owns destructive edit authority
```

## 2. Current Live Reality

The factual live route on 2026-07-26 is:

```text
CorrectionRequest
-> deterministic_text_candidates()
-> nanda_text_candidates()
   -> shadow_text_candidates()
      -> bounded lexical birth
      -> internalized L1.1 seeded birth
      -> same-lemma donor
      -> near-neighbor donor
      -> one local readout
-> unified candidate lattice
-> TransitionDecisionCore
-> verifier
```

Important consequences:

1. `L1.1` is already real as bounded lexical restoration evidence, but it is
   not yet a standalone fully packaged lexical owner.
2. `L2FieldShadow` is now the live candidate-field route for local IME/daemon
   correction.
3. Morphology and transition-phase learning exist, but they are still
   side-teachers rather than the canonical owner above `L1.1`.

This document defines how that mixed route must close into one real `L2`.

Implementation status on 2026-07-26:

- `CandidateReadoutRoute::live_default()` now resolves to
  `CandidateReadoutRoute::L2FieldShadow`;
- `CandidateReadoutRoute::compare_reference()` now resolves to
  `CandidateReadoutRoute::FullWave`;
- the new route lives under
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/`;
- `L2FieldShadow` no longer requires an injected
  `L2CorrectionPeakContext` for candidate birth;
- `L2FieldShadow` now self-prepares its local lexical / boundary contour
  directly from the input text;
- `L2FieldShadow` now internalizes bounded `L1.1` restore surfaces into that
  same local field instead of emitting a separate shadow-side `L1.1` sidecar
  candidate;
- `L2FieldShadow` now also contains one narrow internal morphology donor for
  same-lemma / morphology-slot competition;
- that donor is shadow-only, limited to already-born Cyrillic local surface
  candidates, and only activates when exactly one same-lemma cohort exists
  inside the bounded shadow field;
- the donor is backed by the existing 462k-form morphology package through
  `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/runtime.rs`;
- short low-entropy tokens of length `<= 3` currently bypass `L1.1` seeded
  birth and stay on the plain lexical field to preserve abstain parity on
  ambiguous local signals;
- IME boundary-owned Space/autocorrect mutations now surface from
  `L2FieldShadowBoundary` on the live route rather than `BoundaryCell32`;
- the local donor winner multiplier is now explicit as
  `SHADOW_DONOR_WINNER_WEIGHT = 5` in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- it is still donor-based and currently reuses the existing lexical-phase,
  boundary, layout, and `L1.1` donor packages rather than a standalone
  canonical `L2` package;
- the public CLI candidate-route surface now exposes only
  `l2-field-shadow` and `full-wave`;
- no `CompactL2`, `compact-l2`, `uses_peak_context`, or
  `L2LexicalPhaseCell32` matches remain in `src/`, `tests/`, or `src/bin/`;
- runtime authority changed for the live local route, but not for standalone
  package promotion.

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib correction_core`: failed
  (`95 passed / 14 failed`);
- `scripts/cargo-guard.sh check --bin lay`: passed;
- `scripts/cargo-guard.sh check --bin lay-nanda-wave-eval`: passed;
- `scripts/cargo-guard.sh check --bin lay-daemon`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_uses_shadow_surface_source_ids`:
  passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_self_prepares_l11_candidate_without_peak_context`:
  passed;
- `scripts/cargo-guard.sh test --lib live_l2_field_shadow_route_births_nanda_candidates_without_full_wave_authority`:
  passed;
- `scripts/cargo-guard.sh test --lib hidden_state_blocks_live_known_form_drifts_from_logs`:
  passed;
- `scripts/cargo-guard.sh test --lib ambiguous_known_to_known_swap_requires_relation_proof`:
  passed;
- `scripts/cargo-guard.sh test --lib gate_authorizes_same_transition_behind_unchanged_right_context`:
  passed;
- `scripts/cargo-guard.sh run --bin lay -- --help`: shows
  `--candidate-route <l2-field-shadow|full-wave>`;
- `scripts/cargo-guard.sh run --bin lay -- --help`: shows
  `--compare-candidate-routes`;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2939`,
  `records_used = 134`,
  `surface_diverged = 18 / 134`,
  `gate_diverged = 18 / 134`,
  `provenance_diverged = 32 / 134`,
  `reference_apply = 27 / 134`,
  `shadow_apply = 29 / 134`,
  `user_target_match.reference = 7 / 134`,
  `user_target_match.shadow = 8 / 134`,
  `user_target_match.both = 5 / 134`.

What was not tested in this step:

- fixed heldout `L2` proof;
- live IME/daemon authority flip;
- latency, RSS, and cold-load budget of a real standalone `L2` package.
- formal batch-time / RSS receipt for the self-owned replay path;
- resolution of the 14 broader `correction_core` failures from the broad lib
  run.

Verdict scope:

- the new route compiles and is wired as the only executable live local-field
  owner contour;
- `L2FieldShadow` now owns its own candidate-birth input contour instead of
  consuming a prebuilt legacy lexical route;
- the old `CompactL2` route and `L2LexicalPhaseCell32` source path are removed
  from executable/public route selection;
- on the measured 134 real correction-log inputs, the current live owner no
  longer has full selected-surface or selected-gate parity with `FullWave`;
- this is not yet evidence of a finished standalone canonical `L2` package.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_LIVE_OWNER_IME_DAEMON_GATE_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_LEGACY_ROUTE_REMOVAL_2026-07-26.json`

Runtime authority changed:

- `true`

Historical note:

- remaining `CompactL2` and `L2LexicalPhaseCell32` mentions below refer only
  to earlier compare baselines and receipts; they do not describe current
  executable route selection.

## 2.1 First Internal Morphology Donor Inside `L2FieldShadow`

What was tested for this code step:

- `scripts/cargo-guard.sh check --lib`: passed;
- `scripts/cargo-guard.sh test --lib same_lemma_`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2938`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 26 / 134`,
  `compact_apply = 36 / 134`,
  `shadow_apply = 36 / 134`,
  `user_target_match.compact = 6 / 134`,
  `user_target_match.shadow = 6 / 134`,
  `user_target_match.both = 6 / 134`.

Measured implementation facts:

- the morphology donor now lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- it calls
  `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/runtime.rs`
  lazily through `shadow_same_lemma_surface_readout(...)`;
- it only evaluates already-born shadow surface candidates;
- it only runs for Cyrillic local candidates;
- it only acts when exactly one same-lemma cohort is present;
- on a `Winner`, it filters losing surfaces from that cohort and retags the
  promoted shadow candidate with `L2FieldShadowMorphology`.

What was not tested in this step:

- fixed heldout `L2` proof for same-lemma competition;
- real-log examples where the donor emits `Tied` or `Abstain`;
- live IME authority change;
- latency and RSS of the morphology donor under daemon load.

Verdict scope:

- `L2FieldShadow` now contains its first real internal donor above the input
  contour: same-lemma / morphology-slot competition;
- this donor remains shadow-only and did not change runtime authority;
- on the measured 134 real correction-log inputs, it preserved selected surface
  parity and selected gate parity with `CompactL2`;
- this is not yet proof of a full standalone canonical `L2` local field.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_SAME_LEMMA_MORPHOLOGY_2026-07-26.json`

Runtime authority changed:

- `false`

## 13. 2026-08-16 Immediate-Space Delivery Boundary

The fixed managed-IME replay disproved the assumption that scheduling the
correction projection first was sufficient. The Productive V90 field is
single-flighted, but deterministic candidate material still runs outside that
reusable field on one non-preemptive correction worker.

Measured facts from the fixed `4` warmup plus `20` eligible replay:

```text
eligible NotReady                            20 / 20
eligible correction projections              4 / 20
eligible display projections                20 / 20
late edits after fallback                         0
Space p99                                  8.245 ms
printable p99                              0.194 ms
late final correction runs        108.974-125.444 ms
late final DecisionCore portion          0.311-0.588 ms
late final correction L3 portion         0.020-0.067 ms
late final L1.1 / Productive V90                 0 / 0
```

Thus the first loss is before `DecisionCore`: deterministic candidate
materialization occupies the correction worker. A later character supersedes
the desired generation but cannot preempt the already running generation.
When Space arrives, the final generation is either still queued or itself far
over the deadline. Literal fallback correctly prevents a delayed mutation, but
the correction is lost.

A remote optimized focused profile separated this stage without changing
runtime authority:

```text
deterministic candidate total             309.176 ms
Boundary birth                              0.001 ms
primary typing-rule pass                  262.995 ms
composite candidate pass                   46.160 ms
experimental_layout_ru_to_en              186.075 ms
single_letter_substitution                 25.014 ms
```

The number is a cold focused-unit observation, not a physical latency
percentile. Its architectural result is nevertheless conclusive: Boundary
generation is not the bottleneck. Layout protection expands Russian typo
evidence, the normal typo lane repeats those searches, and the composite lane
performs another word-only expansion. These pure word-repair results must be
materialized once and reused by typed operation identity.

The canonical ownership boundary is therefore refined to:

```text
immutable reusable material
├── Productive V90 field material
└── pure deterministic token/local-structure candidate material
            |
            v
per-frame current-context projection
-> L3/L4/DecisionCore
-> PreparedCorrectionLease(InputFrameIdentity)
-> verifier
-> committed-tail mutator
```

Reusable material may be retained after GUI-frame supersession because it has
no mutation authority. The following objects remain strictly per-frame and
must be discarded on supersession: selected winner, online context scores,
`AuthorizedEdit`, exact GUI identity, feedback state and mutation permission.

The bounded worker may preserve one final queued request as `material_only`
after literal fallback. It may populate the pure material memo, but its
superseded generation cannot publish a lease. A later printable replaces this
single queued item; no per-key thread or unbounded backlog is permitted.

Rejected consequences:

- extending the Space wait would move the lag into the input thread;
- one worker per printable would multiply stale dictionary work and CPU tails;
- caching final decisions would freeze online context and create stale
  authority;
- a Boundary-only bypass would remove morphology and lexical competitors;
- literal token exceptions would repair fixtures rather than the mechanism.

The next proof must report hot reuse and cold first-touch separately. A hot
PASS does not prove a cold unknown token can finish inside `8 ms`.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice5/immediate-space-replay-attempt2.json`

Runtime authority changed:

- `false`

## 13. 2026-08-16 Unified Token Field Slice 2

The live Productive V90 route now has one bounded immutable field owner:

```text
exact scene + ordered L1.1 seeds + package generation
-> CanonicalTokenKey
-> bounded single-flight cache
-> Arc<PreparedCanonicalTokenField>
-> separate text or IME materialization
-> request-time L3/L4
-> DecisionCore
-> verifier
```

The cache holds at most `128` ready fields and `32` computing keys, with at
most `8` waiters on one key. A producer computes outside the mutex. Transient
failure is removed rather than cached as lexical truth. Package reload advances
the generation, clears ready entries and rejects an old producer publication.

One rejected intermediate exposed an identity-cost defect. The structural key
correctly included Productive package SHA-256, but the accessor recomputed the
digest over the complete `16.5 MiB` mmap twice on every cache lookup. A cache
hit therefore took up to `160.398 ms` even though candidate materialization was
about `0.3 ms`. The package view now computes and stores its exact 32-byte
digest once during checked immutable admission. Explicit reload remains the
only generation transition and invalidates dependent caches.

Measured release facts:

```text
format/cache/live focused tests                    17 / 17 PASS
baseline/new semantic projections                    4 / 4 PASS
ordered candidate/authority mismatches                       0

morphology hot p99                              4.643 -> 4.287 ms
glued hot p99                                   4.046 -> 3.083 ms
damaged-prefix hot p99                          8.060 -> 8.187 ms

ready request cache disposition              1 producer + 199 hits
release benchmark binary                              11,030,088 B
standalone benchmark peak RSS                 316,004-319,368 KiB
runtime authority changed                                    false
```

The general `<=5 ms` gate is still open because the damaged-prefix route is
`8.187 ms` p99. This is reported separately from the accepted single-flight
mechanism. Slice 2 permits the shared GUI identity and scheduling work in Slice
3; it does not authorize deployment or claim full quality/latency promotion.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice2-single-flight-reuse.json`

## 13. 2026-08-16 Productive V90 Active-Binding Rebuild

### What Was Tested

The frozen Productive V90 induction was copied to a new isolated remote work
root and resumed against the active L1.1 V9 package and unchanged canonical L2
V13 package. Shared-support recovery, evidence reduction, calibration and final
package compilation completed without touching the installed Lay runtime.

### Measured Facts

```text
L1.1 V9 SHA-256                      bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
canonical L2 V13 SHA-256             cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
Productive package bytes             17,309,944
Productive package SHA-256           40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
recovery sidecar bytes                2,123,112
recovery sidecar SHA-256             de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e
resume wall                               97.37 s
resume peak RSS                         627,440 KiB
manifest entries / H                  1,300 / 1,280
frozen payload SHA-256               2f54844d7f7900734049d2ed2ae53150eead60da3223c0efc4256ba804b7f89b
runtime authority changed                         false
```

The compiler verdict is `PASS_shadow_suggest_only_package`, with
`authority_blocked_by_target_loss=true`. Package construction therefore does
not prove morphology quality or authorize installation.

### Proof Identity Boundary

The old frozen manifest names Productive `9fd8c950...` and L1.1 `47fa757a...`.
The new package names Productive `40fb6a9f...` and L1.1 `bf5a1619...`. Proof
code must admit these only as two exact atomic generation rows while retaining
the unchanged spool, canonical package, axis schema, manifest payload, 1,300
case identities, `H=1,280`, and oracle bindings. Cross-generation pairs and
unknown digests remain rejected. The frozen manifest is not rewritten.

### What Was Not Tested

- fixed `13 x 100 x 2` Productive quality and safety proof;
- installed Productive admission;
- daemon or IBus latency and physical input;
- the later unified-token-field runtime slices.

### Verdict

- package build: `PASS_shadow_suggest_only_package`;
- quality/safety parity: `EXACT` against the accepted semantic V90 baseline;
- automatic latency gate: `FAIL_measured_shadow_gates` at `5.286 ms` versus
  the frozen `5.000 ms` threshold;
- deployment: approved only as a fingerprint-only rebind under the existing
  receipt-scoped `5.317 ms` V90 exception;
- runtime authority changed: `false`.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/resume-build-receipt.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/resume-build.time.txt
```

### Fixed Proof And Semantic Identity

The first full replay accidentally selected `shared_replay_owner=legacy`. Its
raw top-1 `272` and `2,526` base-projection failures are retained as
non-normative route-error evidence. They do not characterize Productive V90.

The corrected semantic proof measured `H/B/S0=1280/1280/1280`, semantic/base
raw top-1 `1109/267`, zero base-projection failures, zero false singleton and
zero integrity errors. The clean one-worker proof produced exact old/new
quality and safety parity across `2,600` cases and all `2,600` independent
probe comparisons. New maximum class p99 is `5.286 ms`, compared with the
already accepted old V90 checkpoint of `5.317 ms`.

The formal automatic verdict remains `FAIL_measured_shadow_gates`; the frozen
`5.000 ms` threshold was neither edited nor reported as passing. The deployment
decision reuses only the 2026-08-12 receipt-scoped V90 exception because the new
generation is the same semantics bound to active L1.1 V9 and is faster than the
accepted generation.

Byte comparison proves the scope:

```text
Productive payload after 256-byte header
  SHA-256  6a959bf04e5011b576c333b87cd00a0c400d5b735581a259c1af89e0fc03aeb8
  verdict  byte-identical

recovery payload after 256-byte header
  SHA-256  ad3d1c03d3a48fc81838d63644f3be51fc0d7d2405e0850f73a71ad5730a31ed
  verdict  byte-identical
```

Only the Productive header L1.1 fingerprint/checksum and the recovery header's
base-package SHA-256 differ. No morphology payload, candidate ordering,
authority rule or runtime route changed.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/productive-v90-active-v9-v13-full-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/productive-v90-active-v9-v13-semantic-full-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/productive-v90-active-v9-v13-semantic-normative-clean-workers1-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_2026-08-16/baseline-v90-semantic-normative-clean-workers1-13x100.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_DEPLOY_DECISION_2026-08-16.json
```

### Live Active-Binding Deployment

The V9-bound pair was installed over the incompatible generation after a full
rollback snapshot. The installed `1.0.33` loader admitted the exact pair before
managed runtime reload. Only `lay-daemon` and the managed `lay-ibus-engine`
were restarted; global `ibus-daemon` remained PID `3702`.

```text
status                                      ready_live_owner
Productive SHA-256                          40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
recovery SHA-256                            de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e
daemon PID / package maps                   1447582 / 2
managed engine PID / package maps           1447607 / 2
global IBus PID                             3702
rollback                                    /home/ubu/.local/lib/lay/rollback/1.0.33-pre-v90-v9-20260816-064416
```

Installed direct-query measurements exposed a route mismatch:

```text
legacy helper cold query                    1.542-2.113 s
legacy helper cached field p99              0.799-2.690 ms
legacy helper whole p99                     7.078-9.631 ms
legacy helper samples                       4 x 200 hot
four-process stable PSS                     756,319 KiB
```

`--query-live-l2` still calls the legacy
`standalone_surface_field_readout_with_productive_limit()` route. Current live
display and correction paths instead call
`canonical_owned_text_candidates() -> Productive V90`. These numbers are
therefore non-normative route-drift evidence and neither pass nor fail live
latency. Cold live traces measured `3.077-3.523 s` including first package
admission, while the inner Productive stages were sub-millisecond. Package
owner restoration is a PASS; installed hot latency remains `UNKNOWN` until a
dedicated same-process live-owner benchmark exists.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_ACTIVE_BINDINGS_LIVE_DEPLOY_2026-08-16.json`

## 46. 2026-08-16 Unified IME Token Field Plan

The `данорм -> да норм` physical failure was narrowed to a delivery failure,
not a literal-word or Boundary-birth defect. The full correction route can
produce the Boundary candidate, but the matching Space decision may not be
ready inside the existing 8 ms wait budget.

The paper design selects one immutable canonical L1.1 -> Productive V90 token
field with two typed projections:

```text
one canonical token identity
-> one bounded single-flight L2 field owner
   -> display projection, no mutation authority
   -> correction projection, full candidate competition
-> exact Space lease
-> verifier
-> one committed-tail mutator
```

The design deliberately does not serialize both projections in one worker.
Display and correction have different deadlines and feedback semantics; one
serial executor would create head-of-line blocking. They may use separate
background consumers, but they must share one immutable field computation and
must not create a second correction authority route.

Measured prerequisite status at design time:

```text
Productive V90 status     unavailable
reason                    package fingerprint mismatch
runtime authority changed false
```

Therefore implementation remains `BLOCKED_BEFORE_CODE` until a Productive V90
package matching the active L1.1 and canonical V13 hashes is admitted and
warmed. Fingerprint validation must not be weakened, and a Boundary-only fast
return is forbidden because it would hide the failed morphology owner and
remove candidate competition.

Full consequence analysis, state machines, file slices, rollback policy and
promotion gates:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/ime-unified-token-field-implementation-plan-2026-08-16.md`

Design route contract:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/preflights/LAY_IME_UNIFIED_TOKEN_FIELD_ROUTE_2026-08-16.json`

Exact structural receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_ROUTE_DESIGN_2026-08-16.json`

### 46.1 Implemented GUI identity and Space lease

The design prerequisite was later restored without weakening package
fingerprints. Slices 1 and 2 extracted and single-flighted the immutable field;
Slice 3 now implements the GUI ownership boundary:

```text
managed printable commit
-> publish tail_epoch
-> capture one InputFrameIdentity
-> schedule prepared Space correction first
-> schedule passive display from the same identity

Space
-> take exact single-consumer lease
-> cancel display generation only
-> revalidate path/focus/tail_epoch/exact tail/layout/config
-> verifier + one committed-tail mutator
   | ready and authorized -> replacement + exactly one Space
   | absent/stale/blocked -> exactly one literal Space, no late edit
```

`PrecognitionIdentity` and `SpaceAutocorrectKey` no longer exist as competing
partial identities. Backspace, focus/reset, layout and config transitions
invalidate stale work. Display cancellation alone does not invalidate a
matching correction lease.

The first complete IBus test run exposed one independent adapter leak: a
multi-token Boundary replacement was shown as passive preedit. The repair is
operation-shaped, not lexical: whitespace-bearing replacements stay available
to the verified Space route but are excluded from passive display. The final
remote target is `202 / 202 PASS`; installed runtime remains `1.0.33` and was
not changed.

Exact measured receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice3-shared-gui-identity.json`

What was tested:

- structural separation of correction preparation, display refresh, ready
  Space apply, literal-Space fallback, observation and proof routes;
- one owner per typed role inside each named event;
- exact static path cardinality for the declared design.

Measured structural result:

```text
verdict                              PASS
nodes                                  20
edges                                  33
issues                                  0
warnings                                0
ready_for_implementation_preflight   true
safe_to_edit                         false
```

What was not tested:

- source parity of the proposed design;
- implementation correctness;
- candidate-quality parity;
- latency, CPU or RSS after the proposed refactor;
- physical GUI behavior.

Runtime authority changed:

- `false`

### 46.2 Implemented observability and source-bound route

Slice 4 removes telemetry ambiguity without moving semantic authority. The
canonical field now returns typed timing and single-flight evidence to each
caller. Display and correction project that evidence independently, while
Space records only consumption of the prepared correction lease:

```text
InputFrameIdentity
-> correction worker -> canonical single-flight field -> correction readout
   -> PreparedCorrectionLease
   -> correction projection receipt
-> display worker -> same canonical single-flight field -> display readout
   -> preedit renderer
   -> display projection receipt

Space
-> exact lease lookup
-> ready: verifier -> backend authorizer -> committed-tail mutator
-> miss/stale/blocked: literal user-Space mutator
-> lease outcome receipt
```

The ready correction and literal fallback have different physical mutation
functions and are mutually exclusive event scenarios. Calling both one source
callsite would be false. Each event still has exactly one mutation owner; the
literal fallback is not a second committed-tail replacement authority.

The field projection trace contains worker generation, tail epoch, engine path,
field producer count, cache disposition, field generation, L1.1 time,
Productive V90 time, display/semantic/correction L3 time and decision totals.
The Space lease outcome is a closed typed set:

```text
ready | not_ready | stale | unauthorized | applied
```

The obsolete direct `canonical_ime_candidates()` wrapper and the obsolete
non-observed `into_resolution_with_peak_context()` entry were removed only
after their observed replacements preserved the semantic wrappers. Ranking,
candidate bytes, verifier behavior and runtime authority were not changed.

Measured software evidence:

```text
remote lay-ibus-engine tests            205 / 205 PASS
observed-source nodes / edges              26 / 38
observed-source routes                          15
source evidence                          64 / 64 PASS
issues / warnings                             0 / 0
installed Lay                                  1.0.33
runtime authority changed                        false
```

The first full run was `204 / 205` because a static test still required the
removed legacy readout wrapper; only that test contract changed. The first
source-bound packet had `65 / 65` valid markers but returned `VETO` because the
paper packet mislabeled the lease as a producer after a rank owner. Reclassifying
the lease as an orchestrator and ending correction authority at the readout
resolved the paper defect without touching runtime code.

Not yet proved:

- actual physical per-frame projection cardinality;
- fixed immediate-Space eligible `NotReady` rate;
- Space and printable latency percentiles;
- aggregate restoration, false authority, glued-word recall and false-split
  quality;
- physical WeChat, Telegram and browser behavior;
- release build, deployment and rollback.

Exact receipts:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice4-observability-and-duplicate-removal.json`

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_UNIFIED_TOKEN_FIELD_2026-08-16/slice4-observed-source-route.json`

### 46.3 Canonical-only correction route and physical-token ownership

The active hybrid correction route is rejected. With Nanda enabled, the source
route must contain one candidate producer chain:

```text
active input
-> one canonical token extractor
-> bounded L1.1 lattice
-> Productive V90
-> one final candidate lattice
-> one DecisionCore
-> verifier
```

`DeterministicThenNanda` and its ordered pair of candidate producers have been
removed from the source. `DeterministicOnly` remains a disjoint explicit mode
only when Nanda is disabled; it is not a fallback after a canonical abstention,
timeout, unavailable package, or empty L1.1 lattice.

The first canonical-only proof exposed a token-ownership defect before L1.1:

```text
physical IME token       current bridge token       result
ytn                      ytn, then rejected          UnsupportedInput
cj,frf                   frf                         wrong query identity
```

The L1.1 V9 package itself is not the source of those two losses. A direct
bounded query against the admitted V9 package measured:

```text
ytn, limit 32            contains нет at rank 2
cj,frf, limit 8          contains only собака
врмея, limit 8           contains время at rank 1
звгрузи, limit 8         empty lattice
```

Therefore the next source change is one systemic extractor, not another
candidate lane. It must preserve the complete physical layout surface,
including layout-letter punctuation, and admit only one of these typed inputs:

```text
Cyrillic lexical token
ASCII physical-layout surface
```

URLs, CLI options, technical identifiers, protected ASCII tokens, and known
English words fail closed before L1.1. The accepted token bytes must be reused
unchanged by L1.1, the Productive V90 cache key, field preparation, field
materialization, and candidate transition classification. A layout-derived
Russian surface must retain `CandidateOrigin::Layout`; it must not be relabeled
as a generic L2 lexical mutation.

`звгрузи -> загрузи` is a separate measured empty-seed mechanism at L1.1. It is
not permission to restore `deterministic_text_candidates()` to the active
route. Its class must be measured and repaired at the wave contour or retained
as `ABSTAIN` until that independent gate passes.

What was tested at this checkpoint:

- the admitted V9 package was queried directly for the four surfaces above;
- the active source callers were statically inspected after hybrid-mode
  removal;
- the physical-token loss was localized to
  `canonical_owned_text_candidates_observed()` before the L1.1 socket call.

What was not yet tested:

- the new extractor implementation;
- Productive V90 materialization from layout-derived L1.1 seeds;
- aggregate layout, lexical, boundary, clean-preservation, false-authority,
  false-split, latency, or double-Shift rollback gates;
- release build, deployment, or physical GUI behavior.

Verdict scope: route diagnosis and design only. Runtime authority changed:
`false`; installed Lay remains `1.0.33`.

Design contract:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/preflights/LAY_IME_CANONICAL_LAYOUT_TOKEN_OWNERSHIP_ROUTE_V1_2026-08-16.json`

The exact design receipt and direct V9 query receipt are written under:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_CANONICAL_LAYOUT_TOKEN_OWNERSHIP_2026-08-16/`

### 46.4 Typed contour field after hybrid removal

The first implementation checkpoint used the compatible V9-bound Productive
package rather than the stale package from the initial remote replay:

```text
Productive p2m SHA-256  40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
package bytes           17,309,944
IME focused proof       14 / 36 PASS
failed tests            22
stale producer checks    3
behavioral failures     19
installed Lay           1.0.33
runtime authority changed false
```

The three stale checks require the historical `glued_phrase` producer ID even
though the same boundary operation is now emitted by
`CanonicalL2FieldBoundary`. They must assert typed origin, verdict, transition
proof, and safety effect instead of restoring the old producer.

The 19 behavioral failures reduce to seven shared mechanisms:

1. exact RU-to-EN keyboard targets such as `тфтвф -> nanda` and
   `цусрфе -> wechat` are present but are relabeled as generic `L2Surface`, so
   they lose layout authority;
2. some exact reverse-layout targets such as `зва -> pdf` are absent because
   only the raw contour is sent to L1.1;
3. mixed-script surfaces are rejected before the field, including one wrong-key
   prefix and one duplicate-prefix-plus-typo shape;
4. short and internal-layout-key surfaces do not share the canonical field,
   leaving `yt -> не` absent and `ye;ty -> нужен` exposed to unrelated lexical
   noise;
5. bounded L1.1 does not retain several exact package forms that are one inverse
   edit from the observation, including insertion, deletion, and edge recovery;
6. some retained targets remain non-authoritative because their own contour
   provenance is unavailable at materialization;
7. a boundary proposal can win while a stronger canonical lexical target is
   missing from the final lattice, as in the general `пер хвачу` false-split
   mechanism.

The shared defect is the field-level `physical_layout: bool`. Layout is a
relation between one observed contour and one candidate, not a property of all
candidates in a request. The accepted replacement is a typed contour field:

```text
observed token
-> canonical token extractor
-> bounded contour set
   + Identity
   + ExactLayout
   + LayoutThenTypo
   + InverseGeometry
-> one merged L1.1/canonical grounding field
-> Productive V90
-> one candidate lattice
-> common L3
-> one DecisionCore
-> verifier
```

`Identity` is the raw L1.1 observation. `ExactLayout` is a byte-exact keyboard
projection. `LayoutThenTypo` is a candidate recovered from that projected
contour after additional damage. `InverseGeometry` is a bounded exact-form
lookup through the canonical form index and is not allowed to impersonate an
independent L1.1 observation.

The cache identity must bind the ordered contour surfaces, relation tags,
bounded L1.1 lattices, inverse form references, scene bytes, and all three
package hashes. A candidate origin is derived from its own surface and lemma
provenance. No field-wide layout flag remains.

The separate `short_layout_candidates()` donor is removed after one-symbol and
two-symbol layout contours enter the same field. URLs, CLI options, technical
identifiers, protected ASCII, and known clean English still fail closed before
the field. `deterministic_text_candidates()` remains unreachable from a
Nanda-enabled event and is not executed after unsupported input, empty lattice,
timeout, package failure, tie, or abstention.

Measured evidence retained for this checkpoint:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TYPED_CONTOUR_FIELD_2026-08-16/hybrid-removed-compatible-package.log
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TYPED_CONTOUR_FIELD_2026-08-16/compatible-package-debug.log
```

What has not been tested at this checkpoint:

- typed contour implementation;
- focused `36 / 36` parity;
- aggregate restoration and per-class quality;
- clean preservation, false authority, false split, latency, RSS, rollback, or
  physical GUI behavior;
- release build, installation, or deployment.

Verdict scope: measured failure classification plus pre-implementation design.
Runtime authority changed: `false`.

## 13. 2026-08-16 Nonblocking IME Precognition (`1.0.32`)

### 13.1 Observed Failure

The installed `1.0.31` IME performed full candidate materialization on the
printable-key path. The local trace retained 339 valid timing records with:

```text
p50                         2.508 ms
p90                        94.499 ms
p99                      2408.008 ms
max                      2823.807 ms
events >=100 ms                  34
events >=1 s                     30
```

The first shared mechanism was package admission, not legitimate L3 work. The
installed productive V90 package fingerprint does not match the active L1.1 V9
and canonical L2 V13 packages. Before this fix, every uncached letter retried
package discovery, hashing, and failed admission.

### 13.2 Accepted Runtime Route

```text
printable key
-> commit or publish the visible character
-> capture exact path + focus + tail epoch + context + token identity
-> replace the one latest-only background work slot
-> full unchanged L2/L3 candidate materialization
-> reject a superseded generation
-> reject any identity mismatch
-> publish preedit candidates
```

The Space route remains separate:

```text
Space
-> DecisionCore
-> verifier
-> authorized mutation or fail closed
```

No candidate source was removed and no steady-state candidate limit was
narrowed. The IME remains a display/output adapter; the shared candidate core
still owns ranking and the verifier still owns mutation admission.

### 13.3 Stable Failed Admission

Productive package admission is now a process-generation result:

- a successful runtime is cached;
- a failed admission and its exact error are cached;
- ordinary input reads either cached result in O(1);
- only explicit package reload can replace the cached generation;
- a failed explicit reload clears dependent candidate caches and remains
  fail-closed.

This does not make the incompatible V90 package compatible. It prevents that
known incompatibility from becoming repeated keyboard-path IO. A compatible
package still requires its own package proof and explicit reload.

### 13.4 Measured Software Proof

Remote host: `e@192.168.3.94`, 20 logical CPUs.

```text
full IME tests                     195 / 195 PASS
authority contract                  20 / 20 PASS
mutation monopoly                   16 / 16 PASS
input-gate tests                      6 / 6 PASS
transition replay false applies              0
unsafe-edit gate failures                    0

warmed candidate materialization n         140
p50                                      11 us
p90                                      18 us
p99                                      19 us
max                                      20 us

largest observed cold readout stage      310.643 ms
```

The cold readout number is intentionally not claimed as key latency: it is now
background work. The new `ibus_printable_key_timing` and existing
`ibus_space_key_timing` records are the post-install proof owners for perceived
key latency.

### 13.5 Scope Boundary

Tested:

- failed-admission caching;
- superseded-generation rejection;
- exact focus/tail/token identity rejection;
- renderer/readout ownership separation;
- full IME and changed-source release gates;
- transition replay and unsafe-edit scoreboards;
- remote warmed and cold candidate-materialization timings.

Not tested at this software-only checkpoint:

- post-install physical typing in Telegram, WeChat, browser, and terminal;
- live printable-key `p99 <=5 ms` and `max <20 ms`;
- live Space latency during an applied autocorrection;
- physical double-Shift rollback after autocorrection;
- a compatible productive V90 package;
- aggregate L1.1 per-class quality, package-size, and RSS gates, which this
  scheduling change does not alter.

Runtime authority changed at this checkpoint:

- source route: `true`;
- installed runtime: `false`;
- candidate ranking owner: `false`;
- mutation/verifier owner: `false`.

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_NONBLOCKING_PRECOGNITION_SOFTWARE_PROOF_2026-08-16.json`

## 2026-08-15 IBus Space Boundary And Physical Layout Recovery

This change closes two first-mechanism failures in the committed-token route.
It does not add literal runtime exceptions for the reported strings.

### 13.1 Failure Mechanisms

For glued words, `BoundaryCell32` could select a valid split such as
`вотслов -> вот слов`, but the committed-tail structural adapter rebuilt a
generic action and substituted the selected winner only after verification.
The candidate surface survived while its exact DecisionCore action authority
did not.

For physical-layout text, a leading punctuation key such as `,` reset the fast
preedit token before the following ASCII letters arrived. The complete physical
token `,kznm` therefore fragmented into the visible partial `,лять`. At the same
time, the exact layout projection and the broader `layout_then_typo` producer
could compete for the same input.

### 13.2 Implemented Route

```text
printable physical input
-> one live preedit token, including valid layout-letter symbols
-> one visible candidate readout
-> schedule an exact path + epoch + tail + layout prefetch
-> BoundaryCell32 or exact whole-phrase layout birth
-> one DecisionCore winner and its selected EditAction

Space
-> take only the matching prefetched generation, bounded by 8 ms
-> verify snapshot + epoch + from + to + allow_apply + exact plan
-> one backend authorizer
-> one committed-tail mutator
-> one outcome trace
```

The selected action is now carried inside `LatentTextTransitionCandidate` and
accepted only when it is byte-for-byte and plan-for-plan identical to the
structural verifier result. A mismatch returns `UnsafeEdit`; there is no late
winner substitution.

The layout route keeps a leading physical-layout symbol in `PreeditFastState`
but does not expose it as a candidate until an ASCII letter follows. Known
multi-token physical input can then project atomically, while an exact physical
projection suppresses the partial `layout_then_typo` lane.

### 13.3 Measured Facts

- focused remote proof: `15/15 PASS`, `0` failed;
- full remote `scripts/check-lay-changed.sh`: `PASS`;
- observed-source route gate: `PASS`, `37/37` source evidence checks;
- static execution cardinality: one visible-input path, one prefetch path, one
  Space-apply path;
- isolated hot precognition generation: p50 `11 us`, p90 `19 us`, p99 `20 us`,
  max `24 us`;
- unsafe replay: `200` records, `0` gate failures, `0` observed risk, `0` slow
  output;
- existing Space prefetch wait budget remains `8 ms`.

Exact receipts:

- `docs/structural_gates/receipts/LAY_IME_SPACE_BOUNDARY_LAYOUT_RECOVERY_PROOF_2026-08-15.json`;
- `docs/structural_gates/receipts/LAY_IME_SPACE_BOUNDARY_LAYOUT_RECOVERY_ROUTE_DESIGN_2026-08-15.json`;
- `docs/structural_gates/receipts/LAY_IME_SPACE_BOUNDARY_LAYOUT_RECOVERY_ROUTE_OBSERVED_2026-08-15.json`.

### 13.4 Scope Boundary Before Deployment

Not yet proven at this point in the change:

- physical GUI application of `вотслов + Space -> вот слов `;
- physical GUI application of `b ,kznm -> и блять`;
- double-Shift rollback and stuck-key behavior with the release binary;
- live Space latency and `prefetch_not_ready` frequency across applications;
- broad glued-word recall and false-split percentages.

Runtime authority changed:

- `false`; the installed `1.0.30` binary is still byte-identical to its
  pre-change image. Source promotion and physical verification follow in the
  release step.

## 13. 2026-08-12 Productive V90 Live Ownership Handoff

### Canonical live route

The source worktree now has one candidate and authority route for Russian
single-token morphology:

```text
IBus / daemon Space boundary
-> CandidateReadoutRoute::CanonicalL2Field
-> complete bounded L1.1 seed lattice, limit 32
-> Productive V90 L2
   -> canonical L2 is read-only lemma/form identity storage
   -> grounded L1.1 lane remains protected
   -> productive morphology lane adds compatible forms
-> one composite L2 lattice
   -> one productive surface may retain a calibrated L2 Winner
   -> more than one productive surface is always Tied
   -> unresolved productive candidates remain SuggestOnly
-> common live L3 phrase field inside TransitionDecisionCore
-> DecisionCore apply admission
-> structural transition verifier
-> one physical mutation route
```

`src/nanda_wave/l3.rs::run_l3()` is a trace and evaluation API. It is not a
second live owner beside `TransitionDecisionCore`. The live correction route
evaluates the complete composite candidate set through
`src/nanda_wave/l3_phrase_gate.rs` inside `TransitionDecisionCore` and accepts
a `SuggestOnly` productive form for authority evaluation only after a directed
L3 pairwise certificate or an exact positive L4 transition.

The historical standalone canonical-L2 candidate/readout path remains only in
the explicit cold-probe and diagnostic query APIs. It is not called by
`canonical_owned_text_candidates()` and therefore is not a parallel live
decision route. Two unused retained diagnostic fields, the copied L1.1 seed
vector and its unreported timing value, were removed without changing the
diagnostic algorithm.

### IME feedback contract

The first visible completion is retained until the token boundary and is
classified as follows:

```text
exact attested completion              -> confirmed_attested
same morphology identity, new ending  -> ending_changed / edited_ime
unrelated continuation                 -> censored
unattested or unobserved outcome       -> censored
```

Not pressing `Tab` is not negative evidence. `ending_changed` requires either
a shared canonical-L2 lemma identity or independently bounded completion-edit
geometry; a common typed prefix alone is insufficient.

### What was tested

- focused IME feedback classification after canonical morphology identity was
  connected;
- a typed productive-lattice invariant where two surfaces from separate slots
  of one lemma force `Tied` and both remain `SuggestOnly` even if the packaged
  V90 verdict names one internal Winner;
- an end-to-end live authority invariant where a V90 `Tied` lattice reaches
  the common L3, one candidate receives the unique directed pairwise context
  certificate, `TransitionDecisionCore` selects only that candidate, and the
  structural verifier passes;
- the complete repository-selected changed suite, including IME latency,
  authority monopoly, input gate, shadow replay, and unsafe-edit gates.

### Measured facts

```text
IME feedback focused gate                         1 passed / 0 failed
V90 multi-surface Tied gate                       1 passed / 0 failed
V90 -> L3 -> DecisionCore -> verifier gate        1 passed / 0 failed
text mutation monopoly contract                  15 passed / 0 failed
input gate                                        6 passed / 0 failed
shadow replay records                                         57
shadow false applies / missed good candidates                0 / 0
shadow unverified transitions                                   0
unsafe-edit gate failures                                        0
hot IME candidate generation p99 / max                    64 / 69 us
changed-suite verdict                                           PASS
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_LIVE_OWNER_HANDOFF_2026-08-12.json
```

### What was not tested

- the current source worktree against the complete remote V90 fixed proof;
- package fingerprint compatibility after building the current release
  binaries;
- installed daemon and physical IBus behavior with Productive V90 loaded;
- live Space-path latency and RSS with the real V90 package mapped;
- double-Shift rollback after the final release installation.

### Verdict scope and authority

The source-level ownership migration is `PASS_local_live_authority_contract`.
The currently installed Lay runtime and package authority are unchanged. Live
installation is forbidden until the remote fixed proof, package integrity,
warmup, Space latency, and physical IBus gates pass.

### 13.1 Productive V90 live deployment

Release `1.0.21` installs the accepted Productive V90 package as the live L2
owner. No V91 package was built and no morphology corpus was recrystallized.
The release was built from the isolated remote source snapshot:

```text
/home/e/projects/lay-v90-live-source-20260812
```

The fixed `13 x 100 x 2 cohorts` proof over the release source produced:

```text
evaluated                                      2,600
H / B / S0                      1,280 / 1,280 / 1,280
H -> B / B -> S0 losses                       0 / 0
base / semantic raw top-1                 267 / 1,109
minimum class top-16                            97.0%
false singleton / integrity errors             0 / 0
base projection failures                            0
maximum class p99                            13.139 ms
previous accepted-proof maximum class p99    16.635 ms
```

All non-timing proof data match the accepted V90 receipt. The automatic proof
verdict remains `FAIL_measured_shadow_gates` because the frozen maximum-class
budget is `5 ms`. That gate was not changed or reported as passing. Deployment
used the user's explicit acceptance of the separately measured `5.317 ms`
V90 runtime result as a release exception.

Installed artifacts:

```text
/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m
bytes        17,309,944
sha256       9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438

/home/ubu/.local/share/lay/nanda_wave/l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r
bytes         2,123,112
sha256       44a20f7aaf7578a960477fbfb1d30c9828b9d71f037e3ad1b2d57bc7fa5568c4
```

Post-install status is `ready_live_owner`, both files are mmap-backed, and both
the daemon and managed IBus engine map the V90 package. The measured checkpoint
was:

```text
lay version                                      1.0.21
lay-daemon RSS                               401,748 KiB
lay-ibus-engine RSS                          367,588 KiB
constant productive cache per process         12,927,216 B
global ibus-daemon PID before / after          3,702 / 3,702
GNOME extension loaded version                       1.0.21
```

The installed runtime route is now:

```text
L1.1 bounded lattice
-> Productive V90 L2
-> one composite lattice
-> common live L3
-> DecisionCore
-> verifier
-> one physical mutation route
```

The physical ordinary-input and double-Shift rollback checks remain explicitly
pending user confirmation. The rollback snapshot is:

```text
/home/ubu/.local/lib/lay/rollback/1.0.20-pre-v90-20260812-130206
```

Exact deployment receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_LIVE_DEPLOY_2026-08-12.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V90_LIVE_DEPLOY_FIXED_PROOF_2026-08-12.json
```

## 14. 2026-08-12 Atomic IME Readout Publication

The Productive V90 route already rebuilt a non-empty bounded field after every
printable continuation. A physical trace showed `12` candidates for both
`пров` and `прове`; therefore this defect did not originate in L1.1, Productive
L2 candidate birth, or common L3 acquisition.

The first faulty ownership boundary was the IBus output adapter:

```text
old: ShowPreeditText -> UpdatePreeditText
new: UpdatePreeditText(visible=true) -> ShowPreeditText
```

Both inactive committed-tail completion and active composition now publish
through `LayIbusEngine::publish_preedit_payload`. A fresh candidate cannot be
exposed as an empty or stale intermediate frame while replacing the previous
suffix.

Measured gate on `e@192.168.3.94`, `20` logical CPUs:

```text
focused candidate rebirth                  1 / 1 PASS
focused publication order                 1 / 1 PASS
previous target invalidation               1 / 1 PASS
full sequential lay-ibus-engine          183 / 183 PASS
cargo fmt check                                  PASS
Cargo target                     9,191,882,752 B
Cargo target budget             12,884,901,888 B
```

Not tested by this software gate:

- physical client rendering after installing the release;
- whether phrase context should rank a noun surface over an infinitive surface
  for a particular prefix. Candidate ranking and runtime authority were not
  changed.

Runtime authority changed: `false`. The single L1.1 -> Productive L2 -> common
L3 -> DecisionCore route remains intact; only its IBus publication lifecycle
changed.

Release `1.0.22` is installed. The remote release build completed in `186.62 s`
with peak RSS `2,372,324 KiB`, zero swaps, and exit status `0`. The global
`ibus-daemon` retained PID `3702`; only the managed Lay daemon and engine were
restarted. Both active processes still mmap Productive V90 `.p2m/.p2r`.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_PREEDIT_ATOMIC_REBIRTH_2026-08-12.json`

## 17. Unified Live IME Candidate Field, 2026-08-12

### 17.1 Accepted route

The live IME no longer runs completion and correction as two untyped ranking
routes. The accepted route is:

```text
IBus active token
-> one CandidateGate
   -> exact-prefix lexical birth
   -> one-edit damaged-prefix birth
   -> exact layout projection
   -> layout plus lexical repair
   -> Productive V90 morphology surfaces
   -> bounded boundary evidence
   -> L3 context birth and score
   -> L4 signed evidence
-> one TransitionDecisionCore display readout
   -> Completion(suffix)
   -> Replacement(full surface, display only)
-> explicit Tab
-> verified ImeCandidateAccept edit plan
-> IBus mutation
```

`Space` autocorrect remains a separate typed operation over the same correction
and verifier core. A displayed replacement has no automatic mutation authority.

### 17.2 Typed lattice contract

The shared field carries explicit lanes:

```text
ExactCompletion
CorrectedPrefixReplacement
LayoutReplacement
BoundaryReplacement
GeneralReplacement
```

The display limit remains `12`. Material is bounded to `64`. Lane reserves
preserve operator diversity before final ranking, but do not grant apply
authority. Corrected-prefix diversity is preserved by:

```text
MorphologySlotIdentity {
    domain: CanonicalFeature | ProductiveV1,
    lemma_id,
    slot_id,
}
```

The identity crosses the Productive/canonical morphology, L2 candidate, live
proposal and DecisionCore boundaries. It is topology metadata only: it cannot
mint a Winner or bypass the verifier.

### 17.3 Active and settled states

An exact lexical surface does not close an active IME trajectory. While the
user is still editing the token, grounded morphology continuations may change
the ending. After a word boundary, the clean committed token is settled and a
weak lexical extension is suppressed. Failure to press `Tab` is censored
feedback, not negative evidence.

### 17.4 Explicit replacement contract

A completion appends a suffix. A replacement is rendered as a full typed
surface and may be applied only by explicit `Tab`. `ImeCandidateAccept` builds
an edit plan from the current visible token, validates the full-token or
surface-preserving boundary transition, and then passes through the common
text-edit gate. Left-context rewrites remain forbidden.

### 17.5 Systemic latency fixes

Two general hot-path defects were removed:

1. Cyrillic input no longer runs broad Cyrillic-to-English settlement on every
   key unless an exact layout projection is independently known.
2. Boundary evidence is evaluated only after the cheap exact-completion count.
   With two or more grounded prefix continuations, the expensive split scan is
   not executed.

The second fix reduced the isolated unique `остан` cache miss from
`303,646 us` to `9,902 us`. The six-prefix isolated debug gate measured:

```text
пол    35,426 us
цел    17,864 us
рас    24,742 us
оста   16,997 us
дост   20,356 us
остан   9,902 us
max    35,426 us <= 50,000 us debug gate
```

The focused damaged-prefix route measured approximately `3.1-3.4 ms` hot,
compared with approximately `549 ms` before removing the duplicate heavy
settlement route.

### 17.6 Proof and baseline comparison

All Cargo work was run on `e@192.168.3.94` through
`scripts/cargo-guard.sh`.

```text
CandidateGate                            27 / 27 PASS
L2 IME readout                            6 / 6 PASS
lay-ibus-engine                         185 / 185 PASS
typing transition authority             20 / 20 PASS
text mutation monopoly                  15 / 15 PASS
input gate                                6 / 6 PASS
changed-code gate                              PASS
transition replay false applies                  0
unsafe edit gate failures                        0
```

The full library suite was compared with an untouched `1.0.22` worktree under
the same remote environment:

```text
baseline 1.0.22     1,282 pass / 66 fail / 1 ignored
unified candidate  1,287 pass / 64 fail / 1 ignored
new failure names                                  0
removed failure names                              2
```

The remaining `64` failures are baseline-known package/service/global-state
tests in this environment. They are not claimed as passing and are not hidden
by the focused gate.

### 17.7 Verdict scope

Runtime authority changed for live IME candidate display and explicit Tab
acceptance. Runtime authority did not change for automatic `Space` correction,
double-Shift rollback, `SafetyGate`, or verifier admission.

Not tested before installation:

- physical rendering and Tab behavior in a real client;
- physical double-Shift rollback after the new binary is installed;
- morphology top-1 correctness for arbitrary sentence contexts.

### 17.8 Installed release result

Release `1.0.23` was built on `e@192.168.3.94` and installed on 2026-08-12.

```text
remote release elapsed          204.18 s
remote release max RSS       2,381,764 KiB
remote release average CPU          399%
remote Cargo target         2,097,106,944 / 12,884,901,888 B
lay-ibus-engine SHA-256     4d598579bd894c903326482b0333a9107...
```

The final LTO/codegen stage is not parallel across all 20 logical CPUs; the
build still stayed within the Cargo disk and memory budgets.

Installed CLI, extension metadata, and the loaded GNOME extension all report
`1.0.23`. Global `ibus-daemon` PID remained `3702`; only Lay-managed processes
were restarted. Both `lay-daemon` and `lay-ibus-engine` mmap the accepted
Productive V90 `.p2m/.p2r` packages.

Measured main-contour RSS immediately after startup:

```text
lay-daemon          399,952 KiB
lay-ibus-engine     386,364 KiB
lay-l1.1-serve      306,504 KiB
lay-l3-online         4,384 KiB
total             1,097,204 KiB
```

Post-deploy focused gates passed for IBus (`185/185`), authority (`20/20`),
mutation monopoly (`15/15`), and InputGate operator (`2/2`). The
installed-package/shared-state Space contract retains two baseline-known
failures under `known_current_word_surface_drift`; they remain within the
already reported 64-failure baseline remainder and were not repaired by
literal fixtures.

Software and runtime promotion are complete. Physical `Tab`, morphology
rerank, responsive `Space`, and double-Shift rollback remain user-observed
gates. Rollback:

`/home/ubu/.local/lib/lay/rollback/1.0.22-pre-1.0.23-20260812-194801`

### 17.9 Physical smoke result

After installing `1.0.23`, the user reported that real typing works very well
overall. This closes the broad installed-runtime physical smoke gate as
`PASS`. The report did not enumerate explicit `Tab`, double-Shift, morphology
rerank, or `Space` latency scenarios separately, so those detailed gates remain
unrecorded rather than being inferred from the broad positive result.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_UNIFIED_CANDIDATE_FIELD_2026-08-12.json`

## 20. 2026-08-10 Productive Geometry And Slot Preparation V33-V38

### What was tested

The productive morphology runtime was profiled and optimized without changing
the canonical L1.1 or L2 packages, without reducing the bounded
`256 / 256 / 16 / 32 / 196608` frontier and without admitting generated forms
to runtime authority.

The V33 `perf` profile over the fixed `13 x 100` proof identified:

```text
damerau_levenshtein_rows                         16.31% CPU cycles
allocator family                       approximately 22-24%
context_slot_evidence_for                         4.02%
decoder block cache                               3.16%
bounded_context_key                               2.30%
```

V36 introduced reusable worker-local geometry storage and delayed bounded
keyboard distance until after the character score. V37 tested and rejected a
worker-local morphology-slot cache. V38 instead prepares one immutable slot map
once per request before parallel lemma expansion.

### Measured facts

All accepted measurements below use the normal stripped proof binary. V33,
V36 and V38 have byte-identical fixed `13 x 100` class summaries and
directional summaries:

```text
configuration                         p50       p99       peak RSS
V33 perf baseline                   3.018     6.970 ms   336,612 KiB
V36 reusable geometry scratch       2.812     6.403 ms   337,016 KiB
V38 request-level slot map          2.328     5.885 ms   336,692 KiB
gate                                            <=5.000 ms
```

V38 preserves the complete quality and safety denominator:

```text
evaluated cases                                  1,300
generated top-16, worst                            94%
readout target retention, worst                    91%
generated unique top-1, worst                      61%
false authority / singleton                       0 / 0
directional same-lemma comparisons                3,483
directional pair coverage                       42.463%
directional target wins / reverse false            69 / 0
steady / peak RSS                      315,644 / 336,692 KiB
productive sidecar bytes                    81,688,382
```

The V37 micro proof preserved quality but regressed one-worker p99 to
`8.165 ms`; its cache was removed. The current V38 twenty-worker `13 x 10`
sample measured `9.974 / 85.832 ms`, so the concurrent tail is not promoted.

### What was not tested

- the larger fixed productive denominator;
- clean preservation and ambiguity retention after live generated-candidate
  integration;
- grounded L1.1 lattice preservation at the final live L3 handoff;
- daemon, IBus and physical multi-client latency;
- physical apply authority for generated forms.

### Verdict scope

- V36: `PASS_lossless_geometry_optimization`;
- V37: `REJECT_worker_local_slot_cache`;
- V38: current accepted source baseline and `PASS_quality_parity`;
- overall productive promotion: `FAIL` because p99 is `0.885 ms` above budget,
  generated top-16/readout retention still fail in individual classes, and
  strict generated unique top-1 remains below `>95%` in every class;
- runtime authority changed: `false`; productive births remain `SuggestOnly`.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V33_PERF_SELF_2026-08-10.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V33_PERF_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V33_PERF_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_SCRATCH_V36_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_SCRATCH_V36_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GEOMETRY_SCRATCH_V36_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SLOT_CACHE_V37_WORKERS1_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SLOT_CACHE_V37_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SLOT_CACHE_V37_BUILD_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_WORKERS1_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_WORKERS20_13X10_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_GLOBAL_SLOT_CACHE_V38_BUILD_2026-08-10.time.txt
```

## 21. 2026-08-10 Productive Small-Range Dedup V39

### What was tested

V39 changed only three lossless operations inside generated-form geometry:

1. exact normalized surface equality returns before vector and keyboard work;
2. Damerau-Levenshtein keeps the shorter input on the allocated row axis;
3. generated family results use linear `Vec` dedup with the unchanged
   `productive_birth_rank` instead of a per-call `HashMap`.

The third change is grounded in the packaged sidecar: exact selected family
ranges have p99 `1`, maximum `3` rules and no range above `32`. No frontier,
score, weight, package or authority rule changed.

### Measured facts

V39 is byte-identical to V38 for class and directional summaries on fixed
`13 x 10` and `13 x 100` proofs. False authority and false singleton remain
`0 / 0`. Four sequential one-worker `13 x 100` runs produced:

```text
run                           p50       p99       peak RSS
1                           2.207     6.092 ms   336,852 KiB
2                           2.292     7.027 ms   336,648 KiB
3                           2.244     5.711 ms   337,012 KiB
4                           2.236     5.819 ms   337,016 KiB
V38 reference               2.328     5.885 ms   336,692 KiB
gate                                  <=5.000 ms
```

The V39 median p99 is approximately `5.96 ms`; the twenty-worker `13 x 10`
probe measured `12.976 / 59.952 ms`. P50 improved consistently, but neither
single-client nor concurrent p99 is promoted.

### What was not tested

- larger productive quality denominator;
- clean/ambiguity and grounded L1.1 preservation at live L3 handoff;
- daemon/IBus physical multi-client behavior;
- generated-form apply authority.

### Verdict scope

- `PASS_quality_parity` for exact class/directional equivalence and `0 / 0`;
- `FAIL_latency` because every measured p99 remains above `5 ms`;
- V39 remains a profiling candidate only;
- runtime authority changed: `false`; generated forms remain `SuggestOnly`.

Exact receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_SMALL_DEDUP_V39_2026-08-10/`.

The unstripped V39 profile measured the next self-costs as:

```text
damerau_levenshtein_rows                         19.78%
Unicode conversion lookup                         9.52%
compact decoder block cache                       4.81%
generate_forms_prepared                           4.21%
bounded Damerau rows                              4.11%
UTF-8 conversion                                  3.85%
RU key mapping                                    3.60%
allocator family                         approximately 15%
```

## 22. 2026-08-10 Productive Lowercase Key Units V40

### What was tested

V40 tested whether the metrics-profile Unicode and keyboard cost owned release
p99. The canonical keyboard mapper, rather than L2, gained:

- a direct path for already-lowercase RU characters;
- reusable encoded key-unit output without an intermediate `Vec<KeyEvent>`.

Generated geometry consumed that API. No package, frontier, transform, score,
weight, readout or authority rule changed.

### Measured facts

Local keyboard/compositional/productive/format tests passed `36 / 36`. Remote
class and directional hashes remained byte-identical to V38/V39 on fixed
`13 x 10` and `13 x 100`; false authority/singleton remained `0 / 0`.

```text
run                           p50       p99       peak RSS
1                           2.282     6.054 ms   337,016 KiB
2                           2.256     6.213 ms   336,968 KiB
3                           2.236     6.331 ms   336,852 KiB
4                           2.285     6.228 ms   336,692 KiB
V39 median                  ~2.240    ~5.956 ms
gate                                  <=5.000 ms
```

The twenty-worker V40 `13 x 10` probe measured `15.114 / 97.993 ms`.

### What was not tested

- larger quality denominator and live L3 handoff;
- daemon/IBus physical multi-client latency;
- generated-form apply authority.

### Verdict scope

- `PASS_quality_parity` and `0 / 0` safety;
- `REJECT_lowercase_key_units_no_release_tail_gain` because release p99 did not
  improve despite the metrics-profile hotspot;
- V40 code is removed;
- runtime authority changed: `false`.

Exact receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_LOWER_KEY_UNITS_V40_2026-08-10/`.

## 23. 2026-08-10 Productive Common-Edge Damerau V41

### What was tested

V41 removed equal prefix and suffix units before exact/bounded OSA DP while
retaining the original normalized-similarity denominator. An exhaustive local
test compared the optimized exact and bounded distance against the untrimmed
heap reference for every pair of ternary strings through length `4`.

No package, frontier, score denominator, readout or authority rule changed.

### Measured facts

All local compositional/productive/format tests passed. Remote `13 x 10` and
`13 x 100` class/directional hashes remained byte-identical to V38/V39; false
authority/singleton remained `0 / 0`.

```text
run                           p50       p99       peak RSS
1                           2.247     6.092 ms   337,016 KiB
2                           2.228     5.834 ms   337,000 KiB
3                           2.277     6.334 ms   336,868 KiB
4                           2.286     6.532 ms   336,856 KiB
V39 median                  ~2.240    ~5.956 ms
```

### What was not tested

- larger quality denominator or live L3 handoff;
- daemon/IBus multi-client behavior;
- generated-form apply authority.

### Verdict scope

- `PASS_quality_parity` and exact exhaustive OSA parity;
- `REJECT_common_edge_trim_no_release_tail_gain` on release latency;
- V41 code is removed;
- runtime authority changed: `false`.

Exact receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_COMMON_EDGES_V41_2026-08-10/`.

## 24. 2026-08-10 Productive Paradigm Field Design Boundary

V42 tested bounded Rayon lemma chunks without changing any package, frontier,
transform, score, or authority rule. Class and directional summaries remained
identical to V39 and false authority/singleton remained `0 / 0`, but release
latency rejected the change:

```text
workers=1, 13 x 100 p99     6.602 / 6.343 / 7.735 / 8.179 ms
workers=20, 13 x 10 p99                              70.105 ms
gate                                                <=5.000 ms
```

Verdict: `REJECT_bounded_lemma_chunks_no_tail_gain`. The result closes task
granularity as the active architectural direction. V39 remains the source
baseline.

The next canonical kernel must remove repeated per-surface representation
rather than redistribute it. The detailed design is:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-canonical-design.md`.

The build-ready paper implementation, including typed records, exact algorithms,
calibration, package format, delta protocol, and proof denominators, is:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-implementation.md`.

Paper review and defect-closure matrix:

`/home/ubu/projects/lay/docs/l2-productive-paradigm-field-paper-review-2026-08-10.md`.

Its fixed route is:

```text
L1.1 bounded lemma lattice
-> LemmaParadigmBinding
-> learned ParadigmCenter
-> context-conditioned MorphologySlot phase field
-> implicit productive prefix trie
-> shared exact character + keyboard geometry traversal
-> evidence-calibrated Winner | Tied | ABSTAIN
-> L3
-> verifier
```

V1 deliberately excludes convergent suffix-minimized FST traversal because OSA,
atom, phase, length, and decoder state depend on the complete emitted prefix.
The grounded L1.1 lane and productive top-32 lane remain physically separate;
generated forms cannot evict a grounded L1.1 candidate.

The design grants no authority. Base L1.1 and canonical L2 remain immutable;
generated forms remain `SuggestOnly` until the strict per-class quality,
candidate parity, latency, clean/ambiguity, and physical multi-client gates all
pass.

Exact V42 receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_BOUNDED_LEMMA_CHUNKS_V42_2026-08-10/`.

## 13. 2026-08-10 Productive Context-Axis Backoff V8

### 13.1 Accepted Mechanism

The productive Russian morphology contour now transports typed evidence through
the complete local route:

```text
L1.1 bounded lexical lattice
-> L2 LemmaCenter
-> exact form bindings + productive form births
-> exact scene evidence
-> learned nearest-left / nearest-right context backoff
-> part-of-speech-scoped morphology basin
-> context-controlled feature axis
-> relative L1 geometry preserved inside that axis
-> Winner | Tied | ABSTAIN
-> L3
-> DecisionCore
-> verifier
```

The implementation does not contain literal word or preposition exceptions.
Backoff projections are learned from the same `T` rows as exact scenes. Exact
scene evidence has priority; nearest-neighbor evidence is used only when that
exact scene was not observed.

Context evidence is scoped by part of speech. For noun and pronoun candidates,
the context-controlled axis is case. Number remains controlled by lexical
geometry and wider context. This prevents a context such as `нет _` from
inventing singular-number authority while still allowing it to support the
genitive case basin.

Morphology still cannot create cross-lemma authority. It only redistributes an
existing lemma-basin budget. When several forms share the supported case, the
budget lift preserves their prior L1 geometry difference instead of flattening
them into an artificial tie.

Generated forms remain `SuggestOnly`.

### 13.2 Measured Micro Result

Remote host:

```text
e@192.168.3.94
/home/e/build/lay-l1-shadow
```

Final compact sidecar:

```text
/home/e/build/lay-l1-shadow/artifacts/l2-productive-sidecar-v4-axis-context-backoff-2026-08-10/package.bin
bytes             79,424,614 B, approximately 76 MiB
sha256            ae4e2764febead3517329d7532f504a8e4701e6abef89d4631d5a540bffacd52
known contexts    194
context slots     416
format verdict    PASS_format_roundtrip
```

Direct live DecisionCore queries with the final code and sidecar:

| Input | First candidate | Rank milli | Morphology result |
|---|---|---:|---|
| `без документы` | `без документа` | 639 | same-lemma support |
| `к документы` | `к документу` | 632 | same-lemma support |
| `о документы` | `о документе` | 632 | same-lemma support |
| `нет документовы` | `нет документов` | 993 | same-lemma support, plural geometry retained |
| `на документы` | `на документ` | 669 | not applicable; ambiguous government did not create authority |

All candidates in this micro remained `SuggestOnly`; no candidate was selected
for automatic application.

### 13.3 Fixed Proof Result

The compact sidecar fixed proof used:

```text
13 classes x 100 cases = 1,300 cases
workers                           20
broad lemma frontier             256
active lemma frontier            256
features per lemma                16
form lattice                      32
atom relation budget         196,608
```

Measured facts:

```text
false authority                    0
false singleton                    0
status violations                  0
exact annotation leaks             0
generated top-16, mean        97.923%
generated top-16, worst       94.000%  sparse multi-omission
readout retention, mean       97.154%
readout retention, worst      93.000%  sparse multi-omission
unique top-1, mean            84.385%
unique top-1, worst           61.000%  sparse multi-omission
proof compute                 50.480 s
end-to-end command            71.17 s
peak RSS                     349,016 KiB
verdict                         FAIL
```

Per-class unique top-1:

| Class | Percent |
|---|---:|
| adjacent transposition | 87% |
| double substitution | 77% |
| extra letter | 93% |
| layout projection | 90% |
| letter substitution | 90% |
| missing letter | 87% |
| non-adjacent transposition | 84% |
| omission + transposition | 76% |
| prefix truncation | 85% |
| punctuation suffix | 94% |
| repeated fragment | 88% |
| sparse multi-omission | 61% |
| suffix truncation | 85% |

The fixed proof remains a fail because every required dimension is conjunctive
and strict `>95%` per class is not met. The productive L2 readout therefore may
not replace L1.1 as the restoration owner.

### 13.4 Latency Scope

The query benchmark explicitly reported `cache_mode=uncached_each_iteration`:

```text
whole shadow route p50 / p99     93.629 / 105.309 ms
productive projection p50 / p99 41.561 / 52.179 ms
```

These values fail the `<=5 ms` live budget, but they are not an installed-daemon
measurement because each benchmark iteration reconstructs the uncached field.
They are retained as a performance blocker, not presented as live IME latency.

### 13.5 Receipts And Verdict Scope

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_CONTEXT_AXIS_BACKOFF_V8_FORMAT_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_CONTEXT_AXIS_BACKOFF_V8_MICRO_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_CONTEXT_AXIS_BACKOFF_V8_DEBUG_13X100_2026-08-10.json
```

Tested:

- typed morphology evidence transport for exact and generated forms;
- exact-scene then nearest-neighbor backoff precedence;
- part-of-speech isolation;
- noun case projection without number authority;
- ambiguous same-neighbor contexts retaining competing slots;
- relative geometry preservation inside one supported case basin;
- compact sidecar format roundtrip;
- five direct live queries;
- fixed `13x100` productive sidecar proof on 20 workers.

Not tested:

- installed daemon and IBus latency with this sidecar;
- automatic apply authority;
- full-sentence semantic disambiguation beyond bounded local context;
- a passing standalone productive top-1 gate.

Verdict scope:

- `PASS_mechanism_shadow` for context-conditioned Russian ending rank inside a
  retained lemma basin;
- `PASS_safety` for zero false authority and zero false singleton in the fixed
  `13x100` proof;
- `FAIL_promotion` for per-class quality and latency;
- runtime authority changed: `false`.

## 13. 2026-08-10 Productive Morphology V2 Family Index

### What Was Tested

The real compact V13 package was evaluated with a deterministic
leave-lemmas-out `13 x 10` proof after replacing the slot-wide suffix scan with
family-indexed longest-supported suffix lookup. All `40` selected target lemmas
were excluded from productive suffix-profile training.

### Measured Facts

```text
cases                                      130
damage classes                              13
target lemma retention, every class     100.0%
generated top-16 range                  50-100%
generated unique top-1 range            30-100%
false authority                              0
admitted suffix profiles             1,246,325
debug profile training                  97.319 s
RSS after training                 1,124,004 KiB
peak RSS                           1,182,320 KiB
generated p50 / p99              328.969 / 707.402 ms
```

The family index removed the unbounded slot-wide suffix-rule traversal from
each generated birth. Its timing cannot be compared directly with the V1
release receipt because V2 used a debug binary.

### First Shared Failure Mechanism

Lemma birth is not the current loss point: the target lemma remained in both
the broad and active lattice in `130 / 130` cases. The loss occurs in context
slot and generated-surface selection.

For one exact lexical context, every compiled `SlotPhaseCenter` stores the same
`scene_wave`. `slot_center_score()` therefore gives each positive slot the same
coherence `1000`; support is capped at `16 * 8`, so sufficiently observed but
mutually incompatible slots all saturate at `1128`. This is positive-only
support, not a posterior against alternatives, and cannot reliably choose a
Russian ending.

### Verdict Scope

`FAIL`. Family indexing is retained as a bounded lookup improvement, but V2 is
not promoted. The next experiment must train positive and anti support for a
context-projected morphology slot from streamed `T` rows, exclude heldout lemma
names, and rank generated surfaces by independent joint evidence.

### What Was Not Tested

- release-build latency parity;
- a denominator larger than `13 x 10`;
- generalization to previously unseen lexical contexts;
- a compact reloadable morphology sidecar;
- live IME authority or automatic application.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_FAMILY_INDEX_V2_DEBUG_13X10_2026-08-10.json`.

Runtime authority changed: `false`.

## 19. 2026-08-10 Incremental DAFSA Completion Accumulator

This experiment fixes the remaining cold decoded-completion bottleneck without
removing candidates, narrowing the field, changing a score, or recompiling the
L1.1/L2 packages.

The diagnosed final decoder contains:

```text
decoder states                         142,470
decoder arcs                           368,025
decoded surfaces                     1,600,423
states visited by measured prefixes    226-1,190
```

The DAFSA traversal itself was already bounded. The repeated work happened
after reaching a terminal: as many as `576` decoded surfaces independently
rebuilt the same prefix byte 4-grams, phase cells, and atom-center keys, then
cloned, sorted, and deduplicated each complete key vector.

The accepted route is:

```text
typed prefix
-> compute prefix byte-gram accumulator once
-> enter decoded DAFSA state
-> for each edge
   -> checkpoint accumulator
   -> append character
   -> add only newly completed byte 4-grams
   -> recurse
   -> restore checkpoint
-> at terminal
   -> add boundary atoms temporarily
   -> read phase, atom count, and unique query overlap
   -> restore checkpoint
-> unchanged candidate sort and readout
```

Unchanged production bounds:

```text
result candidates                         96
material candidates                      576
maximum decoder visits                24,000
maximum completion suffix                  8 characters
candidate sources removed                   0
```

The previous full-rescan implementation remains test-only as an exact reference.
The remote release benchmark used the final package and six distinct cache-miss
prefixes in one sequential test process:

```text
old complete rescan                    8,427 us
incremental accumulator                6,239 us
saved                                  2,188 us
improvement                               25.97%
candidate count/order/scores/evidence     exact parity
test result                               1 / 1 PASS
```

Related gates already completed on the same source state:

```text
incremental accumulator parity          PASS
lexical phase runtime completion         9 / 9 PASS
sequential product candidate gate       26 / 26 PASS
product candidate gate wall time          224.18 s
```

Remote release build facts:

```text
release                                  1.0.17
Cargo jobs                                    20
build wall time                            133.16 s
build average CPU                              321%
build peak RSS                       1,818,692 KiB
build swaps                                      0
```

What was tested:

- incremental feature extraction against complete-surface extraction;
- exact old/new decoded candidate parity for the final-package prefix set;
- release-optimized execution on `e@192.168.3.94`;
- preservation of all `96 / 576` candidate limits and score/evidence fields.

What was not tested:

- the fixed L1.1 `13 x 20,000` quality proof, because package bytes, candidate
  sources, scores, and authority did not change;
- L2/L3 package recompilation;
- physical key-to-frame latency or multi-day cache churn.

Verdict scope:

- `PASS_EXACT_PARITY_RELEASE_BENCH`;
- this is a runtime work-reduction result, not a quality-promotion claim;
- runtime authority changed: `false`.

Installed cutover:

```text
remote -> staging SHA parity              10 / 10 PASS
staging -> installed SHA parity           10 / 10 PASS
CLI / daemon / IME / L3 version                  1.0.17
GNOME extension                                  1.0.17
daemon service                                    active
L3 online service                                 active
L1.1 sidecar                                      ready
L1.1 terminals                                  852,582
active engine                                lay-ime-ru
global IBus PID                            3702 -> 3702
global IBus restarted                            false
recent service warnings                              0
```

Only the managed Lay processes were replaced. The canonical L1.1 and L2 package
bytes and SHA-256 remained unchanged.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_DAFSA_INCREMENTAL_COMPLETION_1_0_17_2026-08-10.json
```

## 13. 2026-08-10 Canonical Live Hot Deployment Gate

The current canonical L2 worktree was synchronized byte-for-byte to
`e@192.168.3.94`, compiled with the optimized Cargo `metrics` profile on `19`
build jobs, and measured in three independent processes on CPU set `4-11`.
The complete bounded lattice remained enabled.

Measured live correction-route latency for `50` hot samples per process:

```text
run    p50       p90       p99       max
1      1.802 ms  1.944 ms  1.988 ms  1.988 ms
2      1.834 ms  1.999 ms  2.069 ms  2.069 ms
3      1.852 ms  1.987 ms  2.141 ms  2.141 ms

gate   p99 <= 5.000 ms, max <= 10.000 ms
result PASS
```

The L1.1 server RSS during the run was `349,008 KiB`. The immutable packages
were:

```text
L1.1  190,139,182 B  sha256 47fa757acac03b0f76e5397e965b9127884e245e9845ce0f1ca0896fb40f33e9
L2    140,556,462 B  sha256 cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
```

The earlier `12-15 ms` diagnostic values were produced by Cargo's explicitly
`unoptimized` test profile. They remain useful for function-level diagnosis but
are rejected as deployment latency evidence. No runtime rewrite or lattice
reduction was justified by that debug-only result.

This experiment did not test diverse first touch, the fixed 13-class quality
proof, or the physical multi-client matrix. Runtime authority did not change.
Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_CANONICAL_LIVE_HOT_METRICS_3X50_2026-08-10.json`.

## 14. 2026-08-10 Canonical L2 Fixed Retention Proof

The canonical V13 V2 packages were evaluated on the complete fixed heldout
matrix: `13` damage classes times `20,000` real heldout forms. The proof used
`20` workers and the production bounds below without reducing the L1.1 lattice:

```text
broad lemma frontier        256
active lemma frontier       256
features per lemma           16
form lattice                 32
atom relation budget    196,608
geometry          exact bounded Damerau
```

Measured class results are deliberately reported with both retention and
readout top-1. The gate in this experiment is target retention, not lexical
restoration authority:

```text
damage class                  target retained   active lemma   form top-16   readout unique top-1
adjacent transposition               99.980%        99.980%        99.980%                96.545%
double substitution                  98.835%        98.875%        98.835%                75.870%
extra letter                        100.000%       100.000%       100.000%                97.675%
layout projection                    99.995%        99.995%        99.995%                99.145%
letter substitution                  99.970%        99.970%        99.970%                89.890%
missing letter                       99.960%        99.960%        99.960%                92.450%
non-adjacent transposition           99.530%        99.665%        99.530%                77.330%
omission + transposition             99.505%        99.535%        99.505%                87.015%
prefix truncation                    99.950%        99.950%        99.950%                82.140%
punctuation suffix                  100.000%       100.000%       100.000%                99.775%
repeated fragment                   100.000%       100.000%       100.000%                93.255%
sparse multi-omission                95.415%        95.350%        95.415%                78.300%
suffix truncation                   100.000%       100.000%       100.000%                93.185%
```

Measured aggregate and resource facts:

```text
evaluated                         260,000
false authority                        0
verdict            PASS_shadow_retention
proof compute                  549.797 s
wall time                      562.760 s
average CPU                       1,514%
proof peak RSS                597,476 KiB
compositional index          77,182,508 B
L2 package                  140,556,462 B
lemma birth p50 / p99       3.183 / 9.675 ms
form birth p50 / p99       15.947 / 399.790 ms
readout p50 / p99           0.489 / 1.238 ms
```

What was tested:

- typed broad-lemma birth from real heldout damaged forms;
- learned context reduction, exact bounded form expansion, and target retention;
- all three required retention gates strictly above `95%` in every class;
- false authority equal to zero.

What was not tested:

- clean preservation, owned by the separate fixed L1.1 restoration proof;
- live L1.1 winner authority transfer;
- L3, L4, DecisionCore, daemon, or IBus final apply authority;
- deployment latency. The proof-only form-birth tail includes exhaustive
  heldout expansion and is not substituted for the independent hot deployment
  gate in section 13.

Verdict scope:

- canonical L2 preserves a grounded target through its bounded compositional
  field above `95%` in every fixed damage class;
- `PASS_shadow_retention` does not claim strict unique top-1 restoration;
- runtime authority did not change.

Exact receipt:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_CANONICAL_FIXED_RETENTION_13X20000_2026-08-10.json`
- SHA-256 `9323ba9d65d9bc85a59eda07ab4fd0892286bcb23bcd5b2fe238919929e5c157`

## 18. Compositional Exact-Form Birth V2 Micro-Proof, 2026-08-09

The tested route extends the canonical field without word-specific runtime
conditions:

```text
damaged surface
-> character + keyboard n-gram wave code
-> banded exact nearest-lemma search
-> exact forms of the retained lemmas
-> normalized character or physical-key Damerau similarity
-> bounded 16-form lattice
-> composition-only authority guard
```

Compact V2 embeds the hot lemma-wave index. Its exact-search contract is:

```text
128-bit code = 8 bands x 16 bits
probe radius 2 -> complete for total Hamming distance <= 23
probe radius 3 -> complete for total Hamming distance <= 31
otherwise      -> exhaustive exact fallback
```

Focused tests proved that banded ranking is identical to exhaustive ranking and
that all centers inside the `23 / 31` bounds enter the candidate set.

Full V13 V2 format facts:

```text
forms                                      1,875,032
lemmas                                        93,672
morphology bindings                        3,255,785
stored lemma wave centers                    428,929
band postings                              3,385,217

reference package                        135,121,803 B
compact V1 package                        63,544,178 B
compact V2 package                        86,794,442 B = 82.77 MiB
V2 embedded index                         23,250,264 B
V2 format build                                3.25 s
V2 build average CPU                              765%
V2 build peak RSS                         549,812 KiB
reference/V2 exact section parity                  PASS
```

The first fixed proof used exactly `100` L2-only forms in each of the same 13
damage classes used by L1.1, for `1,300` damaged cases. It did not reuse or copy
the class implementations.

Measured micro result:

```text
clean unique birth top-1                    90.0256%
clean top-16 retention                      90.0256%
false authority                                    0

class                          unique top-1   top-16
adjacent transposition                 44%       49%
double substitution                    27%       30%
extra letter                           68%       68%
layout projection                      20%       21%
letter substitution                    56%       68%
missing letter                         45%       55%
non-adjacent transposition             27%       29%
omission + transposition               10%       11%
prefix truncation                      46%       48%
punctuation suffix                     90%       90%
repeated fragment                      60%       67%
sparse multi-omission                  11%       14%
suffix truncation                      30%       90%

birth p50 / p99                   5,266 / 7,154 us
composition readout p50 / p99       223 /   536 us
L2 cold load                               578,314 us
RSS after L2 load                          146,044 KiB
process peak RSS                           306,136 KiB
```

The clean failure localizes the first shared mechanism before damage-class
ranking: logarithmic multimodal compression retained only `428,929` centers for
`1,875,032` exact forms. A clean form whose code was omitted can lose its own
lemma before exact form scoring. Therefore this experiment is rejected as a
quality configuration; the low damaged scores must not be repaired class by
class.

Next experiment:

- retain every distinct exact-form wave code inside its lemma;
- preserve the same exact band search and authority guard;
- rerun the same fixed micro-proof before any full `20,000 x 13` proof;
- accept the larger index only if the complete package remains below `195 MiB`.

What was not tested:

- sentence-context ranking between valid morphology cells;
- final L3/L4/DecisionCore apply authority;
- daemon or IBus latency;
- the full `20,000 x 13` denominator, because the micro-proof already failed.

Verdict: `FAIL_center_compression_target_retention`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_COMPOSITIONAL_V2_MICRO_100_2026-08-09.json
```

Runtime authority changed by this experiment: `false`.

### 18.1 Exact-Form Centers And Lemma-Frontier Matrix

The next experiment removed logarithmic center loss by retaining every distinct
exact-form code inside each lemma. No damage-class-specific branch was added.

Measured package facts:

```text
stored exact-form wave centers              1,924,190
band postings                              13,559,110
package bytes                             151,414,190 = 144.40 MiB
embedded index bytes                       87,870,012
RSS after L2 load                          272,308 KiB
process peak RSS                           457,408 KiB
format build                                    3.73 s
format build average CPU                          680%
format build peak RSS                     815,444 KiB
reference/V2 exact section parity                  PASS
```

This fixed the first mechanism exactly:

```text
clean unique birth top-1       90.0256% -> 100.0000%
clean top-16 retention         90.0256% -> 100.0000%
false authority                                  0
```

It did not close damaged-surface birth. The same fixed `100 x 13` sample was
then evaluated with larger lemma frontiers without recompiling or changing any
score:

```text
lemma frontier       4       8      16      32      64
clean top-1       100.0   100.0   100.0   100.0   100.0
worst top-1         9.0    13.0    20.0    26.0    34.0
worst top-16        9.0    14.0    21.0    27.0    37.0
birth p99, us    45,867  49,609  58,035  59,785  78,036
```

The worst class in every row was `omission_transposition`. Increasing only the
frontier is rejected: even `64` lemmas is far below the strict quality gate and
is already far above the `5 ms` hot-path budget.

The shared remaining failure is the lossy retrieval representation itself.
One 128-bit SimHash code is not a sufficient primary index for sparse omission,
transposition, and combined damage. The next canonical experiment replaces it
as primary birth with typed sparse n-gram postings aggregated by lemma:

```text
typed character and physical-key atoms
-> compact atom -> lemma postings
-> weighted overlap frontier
-> exact form reconstruction and Damerau verification
-> SimHash only as bounded fallback
```

What was tested:

- all exact-form centers over the complete `1,875,032`-form V13 field;
- exact reference/V2 section parity;
- unchanged fixed `100 x 13` damage sample;
- lemma-frontier ablation `4 / 8 / 16 / 32 / 64`;
- package bytes, index bytes, cold RSS, peak RSS, and birth latency.

What was not tested:

- typed sparse n-gram posting quality or size;
- full `20,000 x 13` proof;
- sentence context, L3/L4 apply authority, daemon, or IBus.

Verdict: `FAIL_lossy_simhash_primary_birth`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_COMPOSITIONAL_ALL_CENTERS_MATRIX_2026-08-09.json
```

Runtime authority changed by this experiment: `false`.

### 18.2 Decoded-Surface Runtime Materialization A/B

This experiment tested one physical runtime change only. The compact package on
disk and every score stayed unchanged. During cold validation, all front-coded
UTF-8 surfaces were retained as one raw byte bank plus one `u32` offset per
form, so exact-form expansion could borrow a decoded surface instead of
reconstructing its decoder block on each lookup.

Direct A/B used the same package, fixed `100 x 13` contextual sample, `20`
workers, `256` broad lemmas, `128` active lemmas, `16` forms, and `65,536`
atom relations:

```text
metric                         compact on-demand    materialized       delta
form birth p50                         43,444 us        41,482 us     -1,962 us
form birth p99                        107,688 us       111,485 us     +3,797 us
proof                                  3,445,740 us     3,452,131 us  +6,391 us
L2 cold load                             797,840 us       818,536 us +20,696 us
peak RSS                                 414,676 KiB       462,904 KiB +48,228 KiB
minimum active-lemma retention                  94%               94%         0
minimum form-top16 retention                    94%               94%         0
minimum readout retention                       94%               94%         0
false authority                                   0                 0         0
package bytes                           140,556,462       140,556,462         0
```

The same materialized runtime was also measured with all `256` broad lemmas
kept active:

```text
form birth p50 / p99                    66,409 / 179,497 us
proof wall / average CPU                       8.44 s / 1,184%
peak RSS                                          467,000 KiB
minimum broad / active lemma retention               96% / 96%
minimum form-top16 / readout retention                95% / 95%
false authority                                              0
verdict                                                   FAIL
```

Per-class `active=256` result:

```text
class                         broad  active  form16  readout  readout top-1
adjacent transposition          100     100     100      100             94
double substitution              98      98      97       97             78
extra letter                    100     100     100      100             98
layout projection               100     100     100      100             97
letter substitution             100     100     100      100             88
missing letter                  100     100     100      100             96
non-adjacent transposition       99      99      99       99             83
omission + transposition        100     100     100      100             92
prefix truncation               100     100     100      100             86
punctuation suffix              100     100     100      100            100
repeated fragment               100     100     100      100             86
sparse multi-omission            96      96      95       95             80
suffix truncation               100     100     100      100             90
```

Measured conclusion:

- materialization did not change quality or false authority;
- direct p99 became `3.526%` slower while peak RSS grew by `48,228 KiB`;
- the apparent gain inferred from different active widths was not real under a
  controlled same-width A/B;
- keeping all `256` lemmas avoids premature context narrowing, but expanding
  all their exact forms is still far outside the runtime latency budget and
  sparse multi-omission remains exactly at `95%`, not strictly above it.

The materialization code was reverted. The compact on-demand decoder remains
canonical. The next experiment must reduce repeated form work structurally,
without narrowing the grounded lemma lattice and without another full raw
surface copy.

What was tested:

- controlled same-width decoder A/B at `active=128`;
- wide `active=256` quality, latency, RSS, CPU, and false authority;
- all thirteen damage classes on the fixed micro denominator.

What was not tested:

- the final `20,000 x 13` denominator;
- daemon or IBus latency;
- L3/L4/DecisionCore apply authority;
- clean L1.1 preservation, owned by its separate fixed proof.

Verdict: `REJECT_materialized_decoder_no_p99_gain`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_DECODER_MATERIALIZATION_AB_REJECTED_2026-08-09.json
```

Runtime authority changed by this experiment: `false`.

### 18.3 Exact-Bounded Frontier And Atom-Relation Matrix

The first full `20,000 x 13` proof localized the remaining loss before form
expansion. With `256 / 256 / 16 / 32` and `65,536` active atom relations, the
target lemma was absent from both lemma frontiers for `7.01%` of sparse
multi-omission cases. Form expansion and readout then changed the result by only
`+0.065` percentage points:

```text
sparse multi-omission broad lemma retention       92.990%
sparse multi-omission form top-16 retention       93.045%
sparse multi-omission readout retention           93.055%
false authority                                         0
evaluated cases                                   260,000
verdict                                                FAIL
```

This rejects form-score, readout, and per-class patches as the first repair
site. The shared mechanism is insufficient postings evidence before the fixed
lemma frontier.

Two bounded alternatives were compared on the same deterministic
`1,000 x 13` sample with `20` workers. Wider broad frontiers recovered the
target but introduced a separate contextual reduction over discarded lemmas:

```text
broad -> active   atom relations   sparse readout   context p99   form p99
256   -> 256              65,536           94.0%          0 us     30,947 us
512   -> 256              65,536           96.4%     14,623 us     30,832 us
1,024 -> 256              65,536           96.9%     29,468 us     31,164 us
512   -> 256             131,072           97.8%     17,794 us     36,078 us
1,024 -> 256             131,072           98.0%     38,569 us     41,773 us
```

The wide-frontier route is rejected. The canonical field remains one
`256 -> 256` lemma frontier, so contextual reduction is an identity operation.
The postings budget was then varied without changing package bytes, lemma
width, feature width, form width, geometry, or scores:

```text
atom relations   sparse readout   lemma p99   form p99   false authority
65,536                    94.0%       4,382 us   30,947 us                0
98,304                    94.9%       4,936 us   31,245 us                0
131,072                   95.9%       5,985 us   34,981 us                0
196,608                   96.8%      10,228 us   44,697 us                0
262,144                   97.0%      10,888 us   37,788 us                0
```

The p99 values above are concurrent proof measurements, not single-client IME
latency. `196,608` was selected for the full denominator: `131,072` had only a
`0.9` percentage-point micro margin, while `262,144` bought only another `0.2`
points over `196,608`.

What was tested:

- one fixed `13 x 1,000` sample for every matrix row;
- broad/active retention, form/readout retention, false authority, CPU latency,
  RSS, and unchanged package size;
- only global bounded capacities; no word, suffix, phrase, source ID, or damage
  class entered runtime conditions.

What was not tested by the matrix:

- the full denominator, recorded separately in section 18.4;
- single-client daemon or IBus hot latency;
- final L3/L4/DecisionCore apply authority;
- L1.1 clean preservation or L1.1 per-class top-1.

Verdict: `SELECT_atom_relations_196608_for_full_proof`; wider lemma frontiers
are `REJECT_context_reduction_cost`.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_EXACT_BOUNDED_REL65536_FULL_REJECTED_2026-08-09.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_EXACT_BOUNDED_ATOM_RELATION_MATRIX_2026-08-09.json
```

Runtime authority changed by these experiments: `false`.

### 18.4 Canonical Exact-Bounded Full Retention Proof

The selected contour was evaluated over the complete fixed denominator:

```text
damaged surface
-> typed atom postings, at most 196,608 relations
-> 256 broad lemmas = 256 active lemmas
-> 16 morphology features per lemma
-> exact-bounded Damerau geometry
-> 32-form lattice
-> Winner | Tied | ABSTAIN readout
```

Package identity:

```text
forms          1,875,032
package bytes  140,556,462 B = 134.05 MiB
SHA-256        cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
```

Fixed heldout result:

```text
class                         form top-16   readout retained   unique top-1*   false authority
adjacent transposition             99.980%            99.980%          96.595%                 0
double substitution                98.855%            98.855%          75.865%                 0
extra letter                      100.000%           100.000%          97.660%                 0
layout projection                  99.995%            99.995%          99.185%                 0
letter substitution                99.970%            99.970%          89.905%                 0
missing letter                     99.960%            99.960%          92.475%                 0
non-adjacent transposition         99.625%            99.620%          77.380%                 0
omission + transposition           99.535%            99.535%          87.075%                 0
prefix truncation                  99.950%            99.950%          82.155%                 0
punctuation suffix                100.000%           100.000%          99.765%                 0
repeated fragment                 100.000%           100.000%          93.280%                 0
sparse multi-omission              95.415%            95.420%          78.310%                 0
suffix truncation                 100.000%           100.000%          93.190%                 0
```

`*` Unique top-1 is diagnostic here. Every case remained `ABSTAIN`; this proof
promotes target retention for the downstream contextual field, not standalone
apply authority. It therefore does not replace the separate strict L1.1
per-class unique top-1 proof.

Measured execution facts:

```text
evaluated                                    260,000 cases
selected target forms                         74,252
scanned morphology rows                    5,857,714
workers                                            20
wall time                                      341.73 s
average CPU                                      1465%
internal peak RSS                           552,136 KiB
swap growth                                          0
lemma birth p50 / p99                 3,740 / 12,371 us
form birth p50 / p99                 16,101 / 54,578 us
readout p50 / p99                       630 / 4,351 us
false authority                                      0
verdict                            PASS_shadow_retention
```

What was tested:

- `20,000` real heldout cases for every one of the thirteen fixed damage
  classes, `260,000` total;
- target retention at broad lemma, active lemma, form top-16, and readout;
- false authority, cold load, concurrent stage latency, RSS, package identity,
  and deterministic bounded sampling;
- strict `>95%` retention for each class, including `95.420%` for sparse
  multi-omission.

What was not tested:

- single-client daemon and IBus latency;
- final L3/L4/DecisionCore application authority;
- clean L1.1 preservation and the separate L1.1 unique top-1 contract;
- deterministic recompaction and release installation, which remain promotion
  gates after this proof.

Verdict: `PASS_shadow_retention`. The canonical runtime capacities are
`256 / 256 / 16 / 32 / 196,608`; product promotion remains pending.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_EXACT_BOUNDED_REL196608_FULL_20K_2026-08-09.json
```

Runtime authority changed by this proof: `false`.

### 18.5 Mmap-Backed Compositional Index View, 2026-08-10

This experiment changes only immutable L2 package ownership and index storage.
Candidate birth, geometry, scores, calibration, frontier limits, and
`Winner | Tied | ABSTAIN` readout are unchanged.

```text
StandaloneL2Field::load
-> RuntimeL2Package::Compact
-> immutable Linux mmap of the 140,556,462-byte V2 package
-> CompactPackageView
-> embedded compositional sections exposed as CompactLemmaWaveIndexView
-> keep only the derived atom degree table as private runtime memory
```

The package remains the exact V2 package for `1,875,032` forms. No section is
discarded: the typed atom keys, offsets, postings, surface wave codes, wave-band
offsets, wave-band postings, and lemma ranges remain addressable through the
bounded view. Reverse reconstruction is performed only for the active bounded
frontier; the complete relation field stays available in the mmap.

Measured storage comparison against the previous owned compositional index:

```text
metric                                      owned V2       mmap view        delta
package bytes                            140,556,462     140,556,462            0
compositional index private bytes         77,182,508         170,224  -77,012,284
embedded zero-copy view bytes                      0      77,012,284  +77,012,284
full-proof RSS peak, internal receipt          597,476 KiB     481,844 KiB  -115,632 KiB
full-proof RSS peak reduction                                           19.353%
RSS immediately after L2 load                              176,576 KiB
L2 cold load                                                  734,284 us
package mmap-backed                                                true
index source                                      compact_v2_mmap_view
```

The `170,224 B` private allocation is the derived degree table:
`42,556 atom degrees x 4 B`. It preserves rarity-ordered lemma birth. A variant
without this table was rejected because lemma-birth p50 regressed to
`11,462 us`; the retained table restores it to `2,944 us` in this run.

Fixed `13 x 100` micro-proof over the same corpus and capacities
`256 / 256 / 16 / 32 / 196,608`:

```text
evaluated                                      1,300 cases
workers                                                 20
wall time, complete process                           6.12 s
average CPU                                            796%
lemma birth p50 / p99                         2,944 / 9,161 us
form birth p50 / p99                       15,773 / 449,355 us
readout p50 / p99                              433 / 1,087 us
readout target retention, 12 classes                 100%
sparse multi-omission readout retention                98%
false authority                                          0
verdict                              PASS_shadow_retention
```

The complete fixed `13 x 20,000` proof then evaluated the same deterministic
`260,000` cases as the canonical owned-index baseline:

```text
class                         broad lemma   form top-16   readout retained   false authority
adjacent transposition             99.980%         99.980%            99.980%                 0
double substitution                98.875%         98.835%            98.835%                 0
extra letter                      100.000%        100.000%           100.000%                 0
layout projection                  99.995%         99.995%            99.995%                 0
letter substitution                99.970%         99.970%            99.970%                 0
missing letter                     99.960%         99.960%            99.960%                 0
non-adjacent transposition         99.665%         99.530%            99.530%                 0
omission + transposition           99.535%         99.505%            99.505%                 0
prefix truncation                  99.950%         99.950%            99.950%                 0
punctuation suffix                100.000%        100.000%           100.000%                 0
repeated fragment                 100.000%        100.000%           100.000%                 0
sparse multi-omission              95.350%         95.415%            95.415%                 0
suffix truncation                 100.000%        100.000%           100.000%                 0
```

All quality fields, including complete per-class counters, failures, false
authority, sampling denominators, capacities, gates, and verdict, were
canonicalized after removing only latency, cold-load, memory, and sampling-time
fields. The owned and mmap JSON values are byte-identical:

```text
normalized quality SHA-256  73d95a71f6ccb8abe863990f0b3b28fe56d87c702d4d5a82d4c18bdcf795a4c4
quality counters            EXACT MATCH
```

Full-proof execution facts:

```text
evaluated                                      260,000 cases
selected target forms                            74,252
workers                                                20
complete wall time                                10:44.44
average CPU                                          1487%
internal proof time                              631.008 s
internal peak RSS                              481,844 KiB
external peak RSS                              480,100 KiB
lemma birth p50 / p99                       3,396 / 10,547 us
form birth p50 / p99                      17,925 / 467,537 us
readout p50 / p99                            524 / 1,295 us
false authority                                          0
verdict                              PASS_shadow_retention
```

The concurrent proof's form-birth p99 values are not single-client IME latency
measurements and are not promoted as such.

What was tested:

- exact reference, owned compact, and mmap-view readout parity in focused
  runtime tests;
- all `61/61` focused L2 field tests and `25/25` L3 online tests on the
  isolated remote gate with `20` test threads;
- mmap ownership, zero-copy index access, private index bytes, cold load, RSS,
  and all thirteen retention classes on both the micro and complete fixed
  denominators;
- exact equality of every quality counter against the canonical owned-index
  baseline;
- `0` false authority with unchanged runtime capacities.

What was not tested by this experiment:

- single-client daemon and IBus latency or settled PSS;
- final L3/L4/DecisionCore apply authority;
- clean L1.1 preservation, owned by its separate fixed proof.

Verdict: `PASS_mmap_full_storage_and_retention`. The storage implementation is
accepted without a quality or authority change; product promotion still
requires the daemon/IME and release-package gates.

Exact receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_MMAP_ZERO_COPY_13X100_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_MMAP_ZERO_COPY_13X100_2026-08-10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_MMAP_ZERO_COPY_FIXED_RETENTION_13X20000_2026-08-10.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_MMAP_ZERO_COPY_FIXED_RETENTION_13X20000_2026-08-10.time.txt
```

Installed daemon and managed-IBus runtime measurement:

```text
process                  settled PSS   target PSS   result
lay-daemon                242,166 KiB   204,800 KiB  WATCH +37,366 KiB
lay-ibus-engine           206,849 KiB   204,800 KiB  WATCH  +2,049 KiB
lay-l1.1-serve            275,002 KiB   358,400 KiB  PASS
lay-l3-online               1,339 KiB             -  PASS
complete runtime          725,356 KiB   768,000 KiB  PASS -42,644 KiB
```

The complete runtime was stable at `725,356 KiB PSS` at `0`, `30`, `60`, and
`120` seconds. Settled totals were `933,340 KiB RSS`, `463,060 KiB`
PrivateDirty, and `0 KiB` swap. Both daemon and managed IBus mapped the same
`140,556,462 B` package as one read-only private file region. The managed engine
changed from PID `4020744` to `4135074`, retained `lay-ime-ru`, and the global
`ibus-daemon` remained PID `3702`.

This closes the conjunctive complete-runtime PSS gate but not the two
per-process memory targets. Those remain explicit optimization debt rather than
being hidden by the aggregate PASS.

Exact installed-runtime receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_MMAP_LIVE_RUNTIME_2026-08-10.json
```

Runtime authority changed by this experiment: `false`.

## 17. Compact Exact-Surface Format V1, 2026-08-09

This experiment changes only the physical representation of the immutable V13
field. It does not change candidate birth, scores, calibration, readout, or live
runtime authority.

```text
existing V13 L2FieldPackage
-> compact exact codec
   -> FormCenterRef                 16 -> 8 bytes
   -> UTF-8 surfaces               front-coded blocks of 32 forms
   -> feature_mask                 248-entry dictionary
   -> MorphBinding                 16 -> 9 bytes
      lemma_center_id              implicit from LemmaCenter range
-> exact decode to existing L2FieldPackage
```

The surface language remains exact and materialized. This step does not assume
that `lemma + feature_mask` uniquely determines a form: multi-lemma surfaces and
multiple surface variants in one feature slot survive the round trip.

Full V13 measured result:

```text
forms                                      1,875,032 / 1,875,032
lemmas                                        93,672 / 93,672
morphology bindings                        3,255,785 / 3,255,785
feature dictionary entries                       248
decoder blocks                                58,595
forms per decoder block                            32

reference bytes                          135,121,803 = 128.86 MiB
compact bytes                             63,544,178 =  60.60 MiB
saved bytes                               71,577,625 =  52.97%
compact/reference ratio                    47.027331%

FormCenterRef section                     15,000,256 bytes
decoder block offsets                        234,380 bytes
decoder front-coded payload                9,775,225 bytes
feature dictionary                              992 bytes
LemmaCenter section                        2,997,504 bytes
MorphBinding section                      29,302,065 bytes
LocalContextMode section                     671,472 bytes
SlotPhaseCenter section                       17,100 bytes
NeighborCoupling section                     382,128 bytes
CompetitionEdge section                    5,162,904 bytes
TieCalibration                                    24 bytes
header                                            128 bytes
```

Decoder block selection was measured over all `1,875,032` sorted surfaces:

```text
block forms      encoded decoder MiB      average sequential decode steps
8                              13.529                                  4.5
16                             10.873                                  8.5
32                              9.546                                 16.5
64                              8.883                                 32.5
128                             8.551                                 64.5
256                             8.385                                128.5
```

Block `32` is canonical for V1: moving to `64` saves only about `0.66 MiB` while
doubling bounded random reconstruction work.

Exact parity proof:

```text
all package sections equal after decode                 PASS
reference bytes equal after compact decode/re-encode    PASS
reference SHA-256
bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
compact SHA-256
353cd9526429b35ec5c7846b81b06fc16f06b6ea262ffad1005630fcf9bff9b1

compact encode + internal round trip      1.18 s, 485,540 KiB peak RSS
independent exact parity                   1.08 s, 732,908 KiB peak RSS
```

What was tested:

- full V13 format conversion on the remote 20-core build host;
- exact record parity for every package section;
- exact recovery of the original deterministic V13 bytes;
- UTF-8 surfaces, a surface owned by multiple lemmas, and multiple surfaces in
  one lemma-feature slot in focused tests;
- checksum rejection of a corrupted compact package.

What was not tested:

- direct mmap/zero-copy readout from the compact representation;
- compact-package cold startup, steady RSS, or hot p50/p99;
- any new L1.1 damage-class or L2 heldout quality proof;
- live daemon or IME installation.

Verdict scope:

- `PASS_format_roundtrip` and `PASS_exact_parity`;
- not a quality promotion by itself;
- the immutable reference V13 quality receipt remains authoritative;
- runtime authority changed: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_COMPACT_FORMAT_V1_2026-08-09.json
```

### 17.1 Compact direct-backed runtime V1, 2026-08-09

The compact representation is now a runtime storage backend, not only an
offline codec:

```text
StandaloneL2Field
-> RuntimeL2Package
   -> Reference(L2FieldPackage)
   -> Compact(CompactPackageView)
      -> retain one 63,544,178-byte package buffer
      -> read FormCenterRef, MorphBinding, and exact surfaces from that buffer
      -> materialize the smaller lemma/context/competition sections
-> one unchanged index builder
-> one unchanged score and Winner | Tied | Abstain readout
```

V1 owns the compact bytes in a `Vec<u8>` loaded by `std::fs::read`; it is not
yet an mmap implementation. The heavy form, decoder, and binding sections are
not expanded into their reference-width vectors. Exact surfaces and all
multi-lemma and same-slot variant bindings remain available on demand.

Standalone load measurement with the same release binary:

```text
metric                            reference V13       compact V1
package storage                  reference_v2_owned  compact_v1_direct
package backing bytes               135,121,803          63,544,178
process peak RSS, status path        266,400 KiB         107,296 KiB
process peak RSS                       260.16 MiB          104.78 MiB
RSS saved                                                   59.72%
status wall time                           0.56 s             0.56 s
forms                                  1,875,032          1,875,032
morphology bindings                    3,255,785          3,255,785
```

The status path reports both `package_storage` and `package_backing_bytes`, so
a loaded reference package can no longer be mistaken for the compact backend.

The complete fixed V13 heldout proof was then rerun against the compact package
with 20 workers:

```text
same-lemma total                                  2,501,613
same-lemma target coverage                        99.998081%
same-lemma false authority                                 0

noun target coverage                              100.000000%
adjective target coverage                         100.000000%
pronoun target coverage                           100.000000%
verb target coverage                               99.986490%

near-neighbor total                                  42,195
near-neighbor target coverage                      100.000000%
near-neighbor false authority                               0

compact cold load                                  611,508 us
compact hot p50 / p99                                38 / 183 us
hot p99 gate                                             5,000 us
proof wall time                                        26.02 s
proof CPU                                                 649%
proof peak RSS                                  3,670,024 KiB
```

After removing only package path, package byte count, and timing fields, the
complete reference and compact proof JSON files have the same normalized
SHA-256:

```text
61b57a2522d3173d061668759d9ac25063d3a94dc65ec20617c12de5790f4efc
```

Therefore all reported denominators, winners, ties, abstentions, failures,
per-feature counts, per-POS counts, target coverage, and false-authority counts
are identical. Focused runtime tests additionally compare complete readout
objects for reference and compact backends across Winner, Tied, and Abstain
routes.

The compact runtime trades bounded decode work for memory. Compared with the
original reference proof, cold load changed from `477,477` to `611,508 us` and
hot p99 from `97` to `183 us`; both remain below their gates, while standalone
RSS fell by `159,104 KiB`. This is an accepted resource trade, not a latency
improvement claim.

What was tested:

- direct compact-backed runtime over the complete V13 package;
- exact reference/compact record parity plus complete readout-object parity in
  focused Winner, Tied, and Abstain tests;
- the full `2,501,613 + 42,195` fixed heldout proof;
- target coverage and false authority for every reported POS and field class;
- standalone load RSS, package backing bytes, cold load, hot p50/p99, wall time,
  and proof peak RSS.

What was not tested:

- mmap-backed file ownership or page-fault behavior;
- the separate L1.1 thirteen-damage-class proof, because L1.1 bytes and scoring
  did not change in this experiment;
- daemon or IME installation and multi-day live stability.

Verdict scope:

- `PASS_compact_runtime_full_v13`;
- compact runtime quality counters are exactly equal to the reference V13
  receipt;
- package and standalone RSS budgets pass;
- runtime authority changed: `false`;
- no daemon or IME cutover was performed.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V13_COMPACT_RUNTIME_V1_2026-08-09.json
```

## 13. 2026-08-09 L2+L3 Live Prediction Axis

The canonical live prediction route is now:

```text
typed token prefix
-> exact L2 completion field
   + one-pass one-edit DAFSA field
   + bounded online L3 context-birth reserve
-> shared L3 context scoring
-> TransitionDecisionCore admission and ranking
-> IME suffix or display-only replacement
```

The IME remains a renderer. It does not create, rank, or authorize candidates.

### 13.1 Bounded configuration

```text
one-edit DAFSA node frontier       8,192
one-edit corrected-prefix basins      16
decoded reserve per fuzzy basin        8
online words per context center        32
L3 context-birth reserve per call       4
live L2 material cap                    64
```

One Damerau dynamic-programming traversal replaces generated typo-prefix
enumeration. All surviving corrected-prefix basins enter one ranked field.
Compact decoder continuations are retained per basin so inflected surfaces are
not lost behind a hotter terminal-only basin.

Online accepted usage updates the bounded context frontier incrementally:

```text
accepted/confirmed usage event
-> context n-gram IDs
-> context center -> at most 32 target words
-> prefix-filtered context birth
-> decoder/attestation verification
```

This path does not require recompiling the lexical package.

### 13.2 Admission contract

- corrected-prefix candidates are L2-grounded but remain display-only
  replacements;
- a token that is already an exact lexical state cannot be extended by lexical
  geometry or broad L3 similarity alone;
- extension of a complete token requires an exact context-born target or an
  independently accumulated context-to-target usage relation;
- `TransitionDecisionCore::select_live_completions` now directly removes every
  proposal rejected by `admit_live_completion`; L4 cannot accidentally revive
  a rejected proposal;
- duplicate surfaces and duplicate non-empty suffixes are removed after ranking
  with stable seen sets, independent of score adjacency.

### 13.3 Measured facts

Pre-change live log for the observed phrase:

```text
o / о       -> оставить was top, 2.694 ms total
ось         -> осьмых was visible, 4.877 ms total
предскз    -> no useful candidate, 5.392 ms total
```

The persisted confirmed event for `теперь нужно улучшить -> ось`
produced a three-token context center with support `9`. In a live-memory route
probe, prefix `о` produced `ось` as the only admitted candidate with source
`L3ContextBirthCell32`; complete token `ось` produced no weak `осьм*`
continuation.

Focused checks passed for:

- one-edit missing-letter basin `предскз -> предсказ*`;
- decoder morphology retention `переспективн -> перспективнее`;
- short-prefix context birth;
- complete-token extension rejection;
- shared DecisionCore admission enforcement;
- stable candidate deduplication;
- 20/20 typing-transition authority contracts;
- 15/15 text-mutation monopoly contracts.

What was not tested in this architecture experiment:

- a broad fixed IME hit-rate corpus;
- physical GUI acceptance after release installation;
- the fixed 13-class L1.1 restoration proof, which is a separate contract.

Verdict scope: targeted L2+L3 prediction-axis pass. This is not a claim of a
broad IME-quality pass.

Exact receipt:
`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_L3_LIVE_PREDICTION_AXIS_2026-08-09.json`.

Runtime authority changed:

- `true`

## 17. 2026-08-06 Nonblocking Layout Handoff After Autocorrection

The live log exposed a second synchronous owner on the physical Space route
after the `DecisionCore` prefetch work had already been moved off that route.
The observed sequence was:

```text
Tcnm
-> autocorrection commits "Есть "
-> process-level switch to lay-ime-ru blocks
-> switch command times out after 204 ms, ok=false
-> the switch completes later
-> the first key of the next word is decoded under the old layout
-> nакой
-> the next Space repairs it to "такой "
```

Measured pre-fix facts from
`/home/ubu/.local/share/lay/ibus_engine_debug.jsonl`:

```text
prefetch for Tcnm                         486 us
CommitText for "Есть "                    49 us
replacement state                    204,383 us
replacement total                    204,435 us
physical Space total                 204,588 us
ibus_layout_sync target=ru            ok=false
```

The `1.0.13` runtime contract is now:

```text
authorized layout autocorrection
-> commit corrected surface and one Space
-> immediately set this LayIbusEngine decoder to the target layout
-> publish committed-tail handoff
-> schedule one latest-only background process-level IBus switch
-> return from physical Space
```

The background state is bounded to one worker and one replaceable desired
request. A newer desired layout replaces a request that has not started yet.
The worker emits the final `ibus_layout_sync` result, while the hot path emits
`ibus_layout_sync_requested`. The external IBus command and its timeout no
longer belong to autocorrection's physical Space latency.

Manual double-Shift remains on the blocking layout synchronization route. That
operation explicitly asks for a completed user-visible layout transition and
is not part of this Space-only ownership change.

What was tested:

- release compilation of `lay`, `lay-daemon`, and `lay-ibus-engine`;
- installation of Lay `1.0.13` and GNOME extension runtime `1.0.13`;
- restart of only `lay-daemon` and `lay-ibus-engine`;
- global `ibus-daemon` retained PID `3702`.

What was not tested:

- post-install physical GUI Space latency percentiles;
- a repeated live `Tcnm -> Есть такой` interaction after installation;
- quality impact on the fixed L1.1 or L2 heldout proofs.

Verdict scope:

- the measured `204.588 ms` is a pre-fix fact and is not presented as a
  post-fix result;
- code ownership and the installed runtime changed so that autocorrection no
  longer waits for the process-level IBus switch;
- live behavioral confirmation remains pending user typing.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_AUTOCORRECT_LAYOUT_HANDOFF_NONBLOCKING_2026-08-06.json
```

Runtime authority changed:

- `true`

## 18. 2026-08-08 Shared L3 Scene On The IME Preedit Path

The live IME trace exposed a latency outlier while constructing a display-only
completion:

```text
token                         сдела
returned candidates               8
full precognition             83,652 us
L2 material                     889 us
L3 context                   82,290 us
DecisionCore                     27 us
visible suffix                     ть
```

This was not a Space stall and did not apply an edit. It blocked the printable
key path while L3 constructed the preedit candidate field.

The redundant computation was inside
`ContextPhasePackage::score_candidates_with_mode_and_pair_views`. Before
`1.0.14`, one batch built the same context scene once for the batch and then
rebuilt it again inside `candidate_relation_vector` for every candidate:

```text
old: context scene builds = 1 + frontier size
new: context scene builds = 1
```

The live trace records eight returned candidates, but did not record the raw L3
frontier used before final admission. Therefore this experiment does not claim
an exact old scene-build count for that sample. For any raw frontier size `N`,
the count moves from `1 + N` to `1`. Each candidate still clones that scene and
adds its own semantic relation vector before the existing positive, anti,
signature, pairwise, and DecisionCore readout.

The optimization is result-preserving:

- no L2 or L3 candidate limit changed;
- no positive, anti, hard-negative, signature, semantic, or pairwise bank was
  disabled;
- no score, threshold, authority, `Tied`, or `ABSTAIN` rule changed;
- only repeated construction of an identical intermediate vector was removed.

Measured facts:

- pre-fix live outlier: `L3 = 82.290 ms`, total preedit `83.652 ms`;
- post-change debug hot readout over 1,200 iterations:
  `p99 = 1.812 ms`, `max = 1.943 ms`, debug gate `<=5 ms` passed;
- release hot context-phase readout over 1,200 iterations:
  pre-change `p99 = 165 us`, `max = 664 us`;
  post-change `p99 = 164 us`, `max = 182 us`;
- release full sentence readout with 14 pair views and 12 candidates over 1,200
  iterations: `p50 = 444 us`, `p99 = 628 us`, `max = 688 us`;
- immediately after installing `1.0.14`, while graphify and build work were
  still running, the GUI trace still contained L3 outliers: `91.933 ms` for
  token `с` and `64.348 ms` for token `служ`; these are post-fix observations,
  so the physical GUI latency gate remains open even though the isolated full
  sentence readout stays below `1 ms`;
- the wider unique-prefix candidate-gate test remains above its historical
  `1.5 ms` release budget: observed maximum `6.323 ms`, with L2 material up to
  `4.500 ms` and L3 context up to `1.614 ms`; this experiment does not declare
  the complete preedit latency gate closed;
- context-phase behavioral suite: `83/83 PASS` on 19 test threads.

What was not yet measured at the time of this architecture entry:

- post-install physical GUI p50/p95/p99 under an idle development workload;
- recurrence rate of scheduler or page-fault outliers during multi-day input;
- fixed L1.1 restoration proof, which is outside this result-preserving L3
  intermediate-vector change.

Verdict scope:

- the identified duplicate L3 scene construction is removed;
- the context-phase maximum improved in the focused release measurement, while
  the wider candidate gate still fails its existing latency budget;
- post-install loaded-system telemetry still has outliers and prevents a live
  PASS claim;
- verdict is `WATCH`: clean physical typing telemetry remains the final latency
  confirmation.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_L3_SHARED_SCENE_PREEDIT_2026-08-08.json
```

## 19. 2026-08-08 Leading-Pronoun Boundary Recovery

The live input logs exposed `мнесбросили`: Space itself was committed, but the
next autocorrection left the preceding and current words glued. Before this
change the installed end-to-end diagnostic returned `chosen: none` for
`мнесбросили `.

The missing owner was the L2 boundary field. `light_boundary_replacement`
could identify a short leading pronoun, but accepted the right side only from
the narrow surface-motif bank. At the same time the broad morphology bank could
mark the complete glued surface as known and suppress the split. Thus the
stable lexical center `сбросили` was visible elsewhere in L2 but could not
support boundary birth.

The canonical structural rule is now:

```text
short Russian pronoun (<=3 letters)
+ stable right lexical/morphology center (>=4 letters)
-> L2 boundary candidate may beat a broad form-only whole-surface hit
```

This is not a phrase or word exception. Common, protected, and strict clean
whole-word surfaces still return before the override. The same right-center
predicate is used both when the candidate is born and when it competes with a
broad whole-surface morphology hit.

Measured facts:

- focused positive test:
  `мнесбросили -> мне сбросили`, `1/1 PASS`;
- focused clean-word guard:
  `мнение` does not produce `мне ние`, `1/1 PASS`;
- end-to-end debug CLI:
  `CanonicalL2FieldBoundary`, `Eligible/class_allows_apply`, selected output
  `мне сбросили `;
- the broader `boundary_cell_` filter reported `15/17 PASS`; the unrelated
  existing `boundary_cell_scans_split_pair_inside_tail` case also failed and
  is not claimed fixed by this experiment.

What was not tested:

- physical GUI typing after installing the new release;
- a broad clean-word false-split corpus;
- the fixed 13-class L1.1 restoration proof.

Verdict scope:

- the exact L2 birth/readout/admission route for `мнесбросили` passes;
- runtime authority changes because this candidate moves from no selection to
  an eligible boundary correction;
- broader boundary quality remains a separate gate.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_LEADING_PRONOUN_BOUNDARY_2026-08-08.json
```

Runtime authority changed:

- `false`

## 14. 2026-08-06 Short-Function Boundary Shift And Space Timing

### Observed Input Shape

The live input

```text
какие документ ыим
```

is not a missing committed Space. The physical key sequence placed Space before
the final `ы`, so the committed tail contained two surfaces:

```text
документ | ыим
```

The existing `moved_prefix_pair` producer correctly emitted the structural
boundary-shift candidate:

```text
какие документ ыим
-> какие документы им
```

The candidate was rejected later by
`boundary_shift_unstable_token_mass`, because the structural veto required both
result tokens to contain at least four characters.

### Canonical Structural Gate

A boundary shift still cannot change letters. It may only redistribute the
existing tail characters across the last two token boundaries. Each resulting
token must have independent lexical support.

For a token shorter than four characters the additional contract is:

```text
length >= 2
AND known Russian phrase part
AND known short Russian function word
AND exact surface phase center
```

This is a class-level rule, not a word-specific exception. It admits supported
short pronouns and function words while keeping arbitrary short fragments
blocked.

### Space Hot-Path Measurement

The new `ibus_space_key_timing` and `ibus_space_autocorrect_timing` events split
the physical Space route into setup, DecisionCore, replacement and commit time.
The live trace for `склееватся` measured:

```text
Space total                 217876 us
autocorrect DecisionCore    217769 us
Space commit                    67 us
status                  no_decision
```

Additional live outliers reached `224644 us` and `408007 us`. Therefore the
remaining freeze owner is the synchronous committed-token DecisionCore call on
the IBus Space hot path. Boundary commit and replacement are not the dominant
cost. Version `1.0.11` adds exact telemetry and the short-function
boundary-shift admission, but does not claim that the Space latency gate has
passed.

### Evidence Scope

What was tested or measured:

- the live physical sequence and committed-tail surfaces were read from
  `/home/ubu/.local/share/lay/ibus_engine_debug.jsonl`;
- `moved_prefix_pair` produced the expected boundary-shift candidate in the
  diagnostic route;
- release binaries `1.0.11` were built, installed and the Lay runtime was
  restarted without restarting the global `ibus-daemon`;
- global `ibus-daemon` PID remained `3702` during installation.

What was not tested:

- post-install physical GUI confirmation of `документ ыим -> документы им`;
- latency p50/p95/p99 after removing synchronous DecisionCore work from Space;
- fixed heldout L1.1 or L2 quality proof;
- wider boundary-shift corpus coverage.

Verdict scope:

- `1.0.11` is installed with the generalized short-function boundary-shift
  gate;
- physical behavior remains user-verification pending;
- Space latency remains a measured open defect, not a PASS.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_SPACE_BOUNDARY_SHIFT_TIMING_2026-08-06.json`

Runtime authority changed:

- `true`, limited to boundary-shift candidates whose short side satisfies all
  independent lexical and exact-phase checks above.

## 15. 2026-08-06 Nonblocking Space Autocorrect Prefetch

### Rejected Runtime Shape

The `1.0.11` live trace proved that the physical Space handler synchronously
owned the complete correction calculation:

```text
Space key
-> DecisionCore, up to 249579 us in the observed post-install trace
-> commit Space, 690 us
```

This ordering is forbidden. A correction calculation may be expensive, but it
must never delay or suppress the user's physical word boundary.

### Canonical Runtime Shape

Version `1.0.12` uses one process-wide latest-only prefetch worker:

```text
printable committed character
-> publish exact (engine path, tail epoch, tail, layout) key
-> background DecisionCore calculation

physical Space
-> exact completed key available: consume its decision
-> missing, pending or stale key: commit Space immediately
```

The worker stores at most one desired request and one completed result. New
input replaces pending desired work. A result is published only if its
generation is still current. The Space route accepts a result only when engine
path, tail epoch, complete committed tail and active layout all match.

Therefore:

- Space contains no synchronous DecisionCore call;
- stale correction output cannot be applied to newer text;
- a calculation that is not ready may skip that one autocorrection, but cannot
  delay or consume the physical Space;
- the existing `AuthorizedEdit`, structural verifier and exact one-trailing-
  Space contract still own any prefetched correction that is applied.

### Evidence Scope

Measured input fact that caused the change:

```text
post-1.0.11 Space total       250388 us
autocorrect DecisionCore      249579 us
Space commit                     690 us
```

What was verified in this step:

- release compilation of `lay`, `lay-daemon` and `lay-ibus-engine` succeeded;
- installed CLI and GNOME extension report `1.0.12`;
- `lay-daemon` and `lay-ibus-engine` restarted;
- global `ibus-daemon` PID remained `3702`;
- the executable Space path no longer calls
  `decide_active_composition_autocorrect` synchronously.

What was not tested:

- physical GUI latency distribution after installation;
- prefetched correction hit rate during real typing;
- correction quality impact when Space arrives before prefetch completion;
- fixed heldout L1.1/L2 proof.

Verdict scope:

- architectural blocking owner removed from the Space hot path;
- installed runtime awaits physical user verification;
- quality and latency gates are not promoted until live telemetry supplies
  denominators.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IBUS_SPACE_NONBLOCKING_PREFETCH_2026-08-06.json`

Runtime authority changed:

- `true`; a not-ready prefetch now fails open for the physical Space and closed
  for the autocorrection.

## 13. 2026-08-05 Atomic Space And Nonblocking L3 Refresh

### Candidate Birth And Blocking Points

```text
physical Space
-> committed-token autocorrect decision
-> AuthorizedEdit replacement
-> exactly one committed word boundary

next printable key
-> bounded L2 candidate birth
-> read current immutable L3 composite
-> live candidate readout
```

Two coupled runtime defects were observed in the same typing sequence:

- an autocorrect replacement and its triggering physical Space did not have an
  explicit executor-level contract requiring exactly one trailing boundary;
- `with_default_memory()` synchronously loaded a changed L3 composite manifest
  on the hot preedit thread before scoring the next token.

The canonical runtime contract is now:

- a successful Space autocorrect must carry exactly one trailing ASCII Space in
  the authorized replacement; an invalid boundary fails closed and the managed
  route commits the physical Space normally;
- manifest polling may detect a new L3 generation on the readout path, but one
  bounded background worker owns package loading;
- live readout continues against the previous immutable `Arc<L3CompositeMemory>`
  while the worker loads the new generation;
- the worker swaps the ready composite under the write lock; candidate scoring,
  L3 weights and text-edit authority do not change.

### Measured Facts

- live pre-fix trace for token `ош`: total `777948 us`, L2 material `2358 us`,
  L3 context `775051 us`;
- additional live pre-fix examples included `ту`: L3 `83588 us`, and `пу`: L3
  `96816 us`;
- post-change debug cache-miss probe over six distinct prefixes: maximum total
  `34338 us`, maximum L3 stage `11998 us`;
- committed-tail focused tests: `8/8 PASS`;
- one-Space autocorrect sequence tests: `3/3 PASS`.

### Scope And Gate

What was not tested at this point:

- post-install physical GUI p50/p99 under a newly admitted online L3 delta;
- application-specific surrounding-text behavior in every GTK, Chromium and
  WeChat surface;
- full L1.1 thirteen-class restoration proof.

The wider `ime_correction::tests` gate is not green in the current checkout:
sequential execution produced `17 PASS / 14 FAIL`. The failures include stale
source-owner expectations (`personal_phrase` versus `glued_phrase`) and missing
live decisions. They are recorded as a separate existing gate and are not
reported as proof of this focused executor/latency change.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_ATOMIC_SPACE_NONBLOCKING_L3_2026-08-05.json`

Runtime authority changed:

- `false`

## 14. 2026-08-05 Bounded Typo Plus Boundary Repair

### What was tested

- live canonical route for
  `Готовь докуентыдля -> Готовь документы для`;
- `BoundaryCell32` candidate birth for a damaged glued current token;
- proposal admission through the existing
  `current_token_boundary_split_or_repair` contract;
- preservation of the known-word split guards for `уровне` and
  `на уровне`.

### Measured facts

- before the change, both `full-wave` and `canonical-l2-field` produced no
  applicable candidate for `Готовь докуентыдля`;
- the deterministic route could describe `Готовь документы для`, but it was
  `SuggestOnly/boundary_operator_changes_surface`;
- the live canonical route now has `17` candidates: `1` applicable and `16`
  suggest-only;
- the selected candidate is `Готовь документы для` from
  `Nanda:CanonicalL2FieldBoundary`;
- the selected gate is `Eligible/class_allows_apply`;
- the edit is verified as a bounded current-token `GluedWords` operation;
- no phrase-specific replacement table was added.

### General contract

```text
damaged current token
-> BoundaryCell32 proposes known lexical parts
-> current_token_boundary_split_or_repair
   requires unchanged left context
   requires one damaged current token
   requires one or two added word boundaries
   requires known replacement parts
   rejects an already-known original token
   requires Damerau-Levenshtein distance <= 2
-> BoundaryMergeSplit verifier
-> common L2 readout
```

### What was not tested

- broad glued-token recall and false-split percentages;
- physical GUI behavior outside the installed IME probe;
- the fixed L1.1 thirteen-damage-class heldout proof.

### Verdict scope

`PASS_targeted`: the canonical live L2 owner can apply a verified typo repair
and boundary split on the current token. This is not a broad boundary-quality
claim.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_BOUNDED_TYPO_BOUNDARY_REPAIR_2026-08-05.json`

Runtime authority changed:

- `true`, limited to candidates already proved by
  `current_token_boundary_split_or_repair`

## 13. 2026-08-04 Standalone L2 First-Space Warmup

### What was tested

- installed `LAY-L2-RU-FULL-v13.bin` first touch through
  `lay --explain-correct "ЕланаПросит "` with `LAY_L2_FIELD_TRACE=1`;
- a second canonical L2 readout in the same process;
- the existing verified boundary case `Еленапросит -> Елена просит`;
- focused boundary and correction-core tests plus `scripts/check-lay-changed.sh`;
- installation and restart of only the managed Lay runtime processes.

### Measured facts

- installed standalone L2 package: `135,121,803` bytes (`128.86 MiB`);
- cold standalone field load/readout: `379.144 ms`;
- second standalone field readout in the same process: `1.297 ms`;
- second complete canonical L2 materialization: `8.116 ms`;
- the old cold load happened synchronously on the first boundary readout and
  was therefore visible as a pause after Space;
- `warm_up_l2_for_ime()` now loads and indexes the standalone L2 field on its
  existing background warmup thread before candidate memory is published as
  ready;
- candidate birth, scoring, boundary authority, package format, and package
  contents did not change;
- installed Lay runtime PIDs changed from daemon `1853387` / engine `1853423`
  to daemon `1938013` / engine `1938039`;
- global `ibus-daemon` remained PID `3702`.

### What was not tested

- a broad latency distribution across physical GUI applications;
- cold startup on hardware without a warm Linux page cache;
- broad glued-word recall or false-split rate;
- the fixed L1.1 thirteen-damage-class heldout proof.

### Verdict scope

`PASS_targeted`: the measured `~379 ms` package first touch was moved out of
the first-Space hot path into background IME warmup. The measured post-load
canonical L2 path remains single-digit milliseconds for this probe. This is a
latency lifecycle result, not a broad quality claim.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_FIRST_SPACE_BACKGROUND_WARMUP_2026-08-04.json`

Runtime authority changed:

- `false`

## 13. 2026-08-04 Two-Content Glued-Word Boundary Birth

Canonical route added by this experiment:

```text
one glued Cyrillic token
-> enumerate internal boundaries
-> require independent left and right lexical/form centers
-> require at least one strong L2 surface center
-> reserve at most 2 Boundary candidates in the canonical L2 field
-> common L2/L3 lattice
-> TransitionDecisionCore and verifier
-> Winner | Tied | Abstain
```

The generic two-content route requires at least `4 + 4` characters. The
earlier `3 + 3` experiment was rejected because it admitted the false split
`поспорта -> пос порта`.

Clean whole-surface authority is conjunctive safety evidence. If the original
token already has a known Russian word/form center, generic two-content birth
is suppressed. A known whole surface may be split only when the existing
contextual boundary operator independently confirms the same replacement.
This preserves contextual `у насесть -> у нас есть` without allowing
`улетели -> улет если`.

Measured facts:

- `Еленапросит -> Елена просит` is selected by the live correction core;
- source is `CanonicalL2FieldBoundary`;
- class is `GluedWords`;
- gate is `Eligible/class_allows_apply`;
- the explain readout contained `13` candidates: `1` applicable Boundary
  candidate and `12` one-word candidates retained as `SuggestOnly`;
- boundary reserve is `2` candidates;
- L2 unit birth, canonical bridge reserve, live correction-core selection,
  known-whole preservation, multi-letter-preposition safety, and contextual
  known-glue preservation passed in focused sequential tests.

What was not tested:

- broad heldout glued-word recall and false-split rate;
- latency distribution under a live typing workload;
- physical application through the installed IBus engine;
- the fixed L1.1 thirteen-damage-class proof, which is not a boundary proof.

Known separate boundary debt:

- the pre-existing reverse operation `тако й -> такой` currently fails to
  birth in `boundary_scan_candidates`; this two-content glued-token experiment
  does not claim to fix two-token merge recovery.

Verdict scope:

- `PASS_targeted` for generic two-content glued-token birth and live canonical
  L2 selection;
- broad boundary quality is `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed: `true` in release `1.0.7`;
- global `ibus-daemon` PID stayed `3702` during the managed runtime restart.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_TWO_CONTENT_GLUED_BOUNDARY_2026-08-04.json`.

## 14. 2026-08-04 Class-Conditioned Sparse-Omission Reserve

The canonical live bridge keeps a bounded class-conditioned reserve instead of
using one undifferentiated top-N cut:

```text
L1.1 lattice seeds: 16
-> general L2 material frontier: 8
-> sparse-internal-multi-omission reserve: at most 2 additional surfaces
-> one canonical L2 local field
-> L3 and common admission
```

This reserve changes candidate retention only. It does not mint Winner
authority. Reserved candidates still obey the L2 local verdict and remain
`SuggestOnly` under `Tied` or `Abstain`.

Measured facts for the live-log case:

- input: `на компанию Хунлу можем подврдить `;
- L1.1 had `подтвердить` as seed `16/16`, score `1813`;
- the former general frontier retained `8` candidates and discarded that seed;
- after the reserve, correction-core candidate count changed from `10` to
  `11`;
- `подтвердить` now reaches the common lattice as
  `SparseInternalMultiOmission`;
- both `подтвердить` and the competing `подводить` remain `SuggestOnly` under
  `canonical_l2_field_local_tie`;
- final action remains `keep`, so the change fixes candidate visibility without
  reintroducing the observed false autocorrection to `подводить`.

What was tested:

- focused canonical bridge retention test for a sparse omission below the
  general frontier;
- source explain for the exact live-log phrase;
- two existing sparse-omission correction-core contracts passed.

What was not tested:

- fixed heldout sparse multi-omission percentages;
- broad false-candidate cost of the two-slot reserve;
- sentence continuation replay after the following word becomes available;
- physical installed IME behavior at receipt creation.

Known separate failure:

- the existing `переподлчаю -> переподключаю` authority test currently births
  the expected candidate but selects no transition. This is an L2 authority
  baseline failure, not evidence that the new reserve regressed candidate
  retention.

Verdict scope:

- `PASS_targeted` for class-conditioned candidate retention;
- automatic semantic restoration is `NOT_CLAIMED`;
- broad sparse-omission quality is `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed: `true` in release `1.0.7`;
- installed hot readout retains `подтвердить` and returns `Tied/ABSTAIN` with
  no selected transition;
- global `ibus-daemon` PID stayed `3702` during the managed runtime restart.

Cold fail-closed follow-up:

The first request immediately after a managed restart exposed a separate
authority leak. When the `12 ms` L1.1 socket request timed out, L2 inverse
lookup could still birth `подводить` and promote it as a lexical winner despite
having no L1.1 seeds. The canonical ownership contract is now explicit:

```text
no confirmed L1.1 seeds
-> no standalone L2 lexical field
-> no inverse-only Winner authority
-> keep / ABSTAIN
```

This is a general fail-closed rule. It does not special-case `подврдить` or
`подтвердить`; it prevents every cold, unavailable, or timed-out L1.1 request
from being replaced by autonomous L2 lexical authority.

Installed verification in release `1.0.7`:

```text
release SHA-256              3387bfc4f4716853ee632868d4866d35d833fdbb745a8e1abd4fa3b3d57c29e4
cold first Nanda candidates  0
cold first Nanda selection   none
hot Nanda candidates         11
hot target                   подтвердить / SuggestOnly
hot wrong competitor         подводить / SuggestOnly
hot selection                none
glued-word regression        Еленапросит -> Елена просит
lay-daemon PID               1830167
lay-ibus-engine PID          1830194
lay-l1.1-serve PID           1830227
global ibus-daemon PID       3702 -> 3702
```

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_SPARSE_OMISSION_RESERVE_2026-08-04.json`.

## 15. 2026-08-04 Reference-Backed Short-Participle Ambiguity

Observed live failure:

```text
input                       подлючен
wrong live winner           подлечен
expected visible candidate  подключен
```

The installed L1.1 package has no `подключен` or `подключён` surface. Its
top-16 field contains noun forms such as `подключение`, while L2 one-edit
inverse lookup independently births the valid but contextually wrong
`подлечен`. System Hunspell confirms that both `подключен` and `подлечен` are
real short passive participles, so deleting or globally suppressing either
surface would be incorrect.

Canonical route added by this experiment:

```text
one-letter omission geometry
-> derive bounded candidate surfaces
-> require an explicitly attested long participle in the reference lexicon
   (for example подключенный -> подключен)
-> reserve at most 2 reference-backed short forms without authority
-> combine with the ordinary L1.1/L2 cohort
-> unresolved equal-distance forms force Tied/ABSTAIN
-> sentence context may resolve them later
```

Measured source facts:

```text
candidate count before      15
candidate count after       16
подключен                   missing-letter / SuggestOnly
подлечен                    letter-substitution / SuggestOnly
local verdict               Tied
selected transition         none
```

Safety reasoning:

- no surface string is hardcoded;
- candidate birth requires exact long-form reference evidence;
- the reference donor cannot grant Winner authority;
- the rule only preserves one-edit ambiguity and therefore cannot rewrite an
  unrelated token;
- a real sentence-level context remains responsible for choosing between two
  valid meanings.

What was tested:

- long-form backing for masculine and inflected short participles;
- rejection of a fabricated unbacked short form;
- exact source explain for `подлючен`;
- both valid candidates survive as `SuggestOnly` and no transition is selected.

What was not tested:

- broad short-participle recall and false-ambiguity rate;
- sentence contexts that should resolve `подключен` versus `подлечен`;
- fixed L1.1 thirteen-class restoration proof, because the package is unchanged.

Verdict scope:

- `PASS_targeted_source` for preventing the false singleton;
- automatic semantic restoration is `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed: `true` in release `1.0.7`.

Installed verification:

```text
release SHA-256             5a53ef90a1e47007176a13e41b2c241db85eb7bb60db6ecd1b621d7cd791178f
hot candidate count        16
подключен                  missing-letter / SuggestOnly
подлечен                    letter-substitution / SuggestOnly
selected transition        none
glued-word regression      Еленапросит -> Елена просит
sparse reserve regression  подтвердить retained; selected none
lay-daemon PID             1853387
lay-ibus-engine PID        1853423
lay-l1.1-serve PID         1853447
global ibus-daemon PID     3702 -> 3702
```

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_REFERENCE_BACKED_SHORT_PARTICIPLE_AMBIGUITY_2026-08-04.json`.

## 13. 2026-08-04 Internal Layout-Key Projection Contract

Observed live failure:

```text
typed surface                 ye;ty
exact physical projection    нужен
live Space result             unchanged
```

The IME trace proved that all five characters, including `;`, remained in the
committed-tail field. The failure was therefore not character loss or a split
replacement range. The final source-level root cause was a second settlement:
the layout lane first proved the exact known projection `ye;ty -> нужен`, then
context-free L2 morphology moved that result to the same-lemma neighbour
`нужна`. The verifier correctly abstained when `нужен` and `нужна` conflicted.

Canonical rule:

```text
ASCII token with internal layout-letter key
-> exact full-token keyboard projection
-> known opposite-layout word/form
-> keep exact layout candidate eligible
-> exact projection is lexical authority
-> L2 morphology may settle only unknown/noisy projections
```

This is class-based, not a word exception. The internal-key set is the existing
layout alphabet (`;`, `[`, `]`, `,`, `.`, `'`, and their shifted variants).
Known English words and technical surfaces such as `pdf`, URLs, CLI options and
brand tokens retain their protection.

What was tested:

- `ye;ty -> нужен` through the committed-tail manual-toggle planner:
  `5` deleted characters, exact replacement `нужен`;
- `ye;ty -> нужен ` through the live Space decision with active English layout;
- `pdf` remains unchanged with active English layout;
- the candidate constructor retains exact `нужен` instead of settling it to
  `нужна`;
- debug explain emits one accepted layout candidate, `нужен`, and no `нужна`
  competitor;
- all focused tests passed when run independently through
  `scripts/cargo-guard.sh`.

Measured facts:

```text
exact projection tests       4/4 PASS
manual delete span           5 characters
false protected pdf apply    0
accepted layout candidates   1
morphology competitors       0
debug output                 нужен
```

What was not tested at this point:

- aggregate L1.1 heldout quality, because this change does not alter the L1.1
  package or its readout.

Installed runtime facts:

```text
release                         lay 1.0.7
installed explain              ye;ty -> нужен
confidence                     SingleCandidate
second candidate               none
lay-daemon.service             active
active engine                  lay-ime-ru
global ibus-daemon PID         3702 -> 3702
changed-file gate              PASS
```

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/IME_INTERNAL_LAYOUT_KEY_PROJECTION_2026-08-04.json`

Runtime authority changed at this documentation point:

- `true`; release `1.0.7` was installed and the managed Lay runtime was
  restarted without restarting the global IBus daemon.

## 14. 2026-08-04 Typo-Tolerant IME Completion Lane

What was tested:

- Russian IME completion after one insertion, deletion, substitution, or
  adjacent-transposition error in an unfinished prefix;
- the observed `переспектив...` family;
- separation from same-size full-token repair, which remains owned by the
  Space/autocorrect route;
- IME rendering as a full-token replacement accepted only by explicit `Tab`;
- hot-path latency and the existing IME latency budget suite.

Measured facts:

```text
damaged prefix                 переспектив
returned family candidates    12
examples                      перспективный, перспективна,
                              перспективно, перспективней
cold targeted readout         7,867 us
hot cache readout                  6 us
existing IME latency suite    p50 26 us / p90 36 us / p99 46 us / max 62 us
```

The corrected-prefix lane starts at `7` Cyrillic characters, admits at most
`2` corrected prefix basins, and reserves at most `8` L2 candidates plus one
final display slot. Exact-prefix candidates are retained. Early ambiguous
states such as `пересп` are not forced to `перспективнее`, because real
`переспать...` and `преспокойных...` basins still compete there.

What was not tested:

- aggregate IME hit-rate over a fixed heldout typo-prefix corpus;
- physical `Tab` acceptance after installing the new binary;
- typo-tolerant ASCII completion;
- full L1.1 13-class restoration proof, because package data and boundary
  restoration were not changed.

Verdict scope:

- targeted corrected-prefix family coverage: `PASS_targeted`;
- existing hot latency suite: `PASS`;
- broad IME quality promotion: `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed at this checkpoint: `true` (`lay 1.0.6`).

Installed runtime facts:

```text
global ibus-daemon PID       3702 -> 3702
managed engine PID        432941 -> 464498
lay-daemon PID            432906 -> 464453
active engine                       lay-ime-ru
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_IME_TYPO_TOLERANT_COMPLETION_2026-08-04.json
```

## 13. 2026-08-04 Single-Edit Inverse Lane And Tied Readout

What was tested:

- canonical standalone L2 restoration for `переспективнее`, `отвликайся`,
  `переделаем`, and the ambiguous observed surface `наденный`;
- package-indexed inverse lookup for every one-step Damerau operation:
  insertion, deletion, substitution, and adjacent transposition;
- tied-cohort authority for length-changing versus shape-preserving repairs;
- preservation of valid Russian verb forms through reusable ending relations,
  without word-specific runtime rules.

Measured facts:

```text
переспективнее -> перспективнее
отвликайся     -> отвлекайся
переделаем     -> переделаем
наденный       -> наденный
```

The inverse lane remains bounded to `16` package form references and performs
direct package index lookups. It does not scan the complete L2 field. When the
L2 readout remains tied, insertion/deletion candidates are `SuggestOnly`;
substitution/transposition may retain independently verified authority.

What was not tested:

- fixed heldout percentages for all L1.1 damage classes;
- full-corpus L2 recompilation or package-format changes;
- weak IME preedit coverage beyond the four direct smoke probes;
- physical typing after installation.

Verdict scope:

- bounded single-edit inverse lane: `PASS_targeted`;
- observed false-authority containment for `наденный`: `PASS_targeted`;
- broad language-quality promotion: `NOT_CLAIMED`;
- runtime authority changed in source: `true`;
- installed runtime authority changed at this checkpoint: `true` (`lay 1.0.5`).

Installed smoke facts:

```text
global ibus-daemon PID       3702 -> 3702
managed engine PID        4002343 -> 432941
lay-daemon PID            4002297 -> 432906
active engine                       lay-ime-us
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_SINGLE_EDIT_INVERSE_AND_FEEDBACK_SANITATION_2026-08-04.json
```

## 13. 2026-08-03 Inverse Length Birth and Tied-Cohort Authority

### Tested change

The standalone L2 runtime now performs a bounded inverse lookup for forms that
are exactly one insertion or one deletion from a damaged token:

```text
damaged token
-> bounded one-length-edit variants
-> binary search in the existing sorted DecoderGraph
-> at most 16 additional lexical seeds
-> existing L2 context/competition readout
-> Winner | Tied | Abstain
-> existing transition verifier
```

This is candidate birth only. It does not scan the field, add word-specific
rules, recompile V13, or grant apply authority by itself. Exact one-edit forms
enter with the same lexical energy as the strongest L1 seed because they are
alternative explanations of the same damaged signal. Learned context and
competition remain responsible for separating them.

`L2FieldAuthority::Tied` now carries the tied surfaces. A tie cannot promote an
L2 surface, but it also cannot veto an independently verified candidate that is
already a member of that tied cohort. Foreign candidates are still demoted to
`SuggestOnly`.

### Measured facts

- synthetic package inverse lookup:
  `окное -> [окне, окно]`,
  `перхвачу -> [перехвачу]`,
  clean `окне -> []`;
- focused standalone L2 tests: `29/29 PASS`;
- focused IME regression:
  `клавиатурой не перхвачу -> клавиатурой не перехвачу`, `1/1 PASS`;
- installed V13 readout for `перхвачу`:
  `Tied(первачу=2038, перехвачу=2038)`;
- final correction-core selection for that case:
  verified deterministic `missing_letter -> перехвачу`;
- installed V13 readout for `у меня в окное`:
  `Abstain`; the lattice contains `окне` and `окно`, but neither receives
  authority;
- debug-process timing after initialization was about `36 ms` for the
  `перхвачу` probe. This is not a release latency measurement.

### Rejected experiment

An exact-key suffix backoff
`у меня в _ -> меня в _ -> в _` was tested against installed V13. It produced
no non-zero slot, neighbor, or competition evidence for `окное`, so it was
removed from the code. Verdict: `NO_EFFECT_NOT_RETAINED`.

### Not tested

- fixed heldout per-error-class percentages;
- release hot p50/p99 after installation;
- physical WeChat typing after binary replacement;
- a trained L2/L3 contextual winner for `у меня в окное -> у меня в окне`;
- full IME module parity, whose current environment-dependent baseline still
  has unrelated pre-existing failures.

### Verdict and authority

- verdict: `PASS_TARGETED_PERHVAHU_WATCH_OKNOE`;
- package changed: `false`;
- L1.1 restoration authority changed: `false`;
- L2 tied-cohort authority handling changed: `true`, narrowly for independently
  verified members of the reported tied cohort;
- exact receipt:
  `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_INVERSE_LENGTH_TIED_AUTHORITY_2026-08-03.json`.

### Installed state

- release build: remote `20`-CPU host, `CARGO_BUILD_JOBS=20`;
- installed version: `1.0.1`;
- installed binaries: `lay`, `lay-daemon`, `lay-ibus-engine`;
- active engine after replacement: `lay-ime-ru`;
- global `ibus-daemon` PID before/after: `3702/3702`;
- installed explain route confirms
  `клавиатурой не перхвачу -> клавиатурой не перехвачу`;
- installed explain route confirms `у меня в окное` remains `Abstain`.

## 16. 2026-07-31 Live Input Log Feedback Gate

What was inspected:

- `/home/ubu/.local/share/lay/recent_actions.jsonl`;
- `/home/ubu/.local/share/lay/nanda_wave/word_usage_events.jsonl`;
- `/home/ubu/.local/share/lay/ibus_engine_debug.jsonl`;
- `/home/ubu/.local/share/lay/nanda_wave/l3-online/state.json`.

Measured facts:

- `142` valid recent action records;
- `2,708` valid usage events;
- `871` manually completed visible prediction matches;
- `12` explicit completion accepts and `134` completion rejects;
- `2` double-`Shift` auto-undo rejections;
- the online L3 reader consumed `511,831` source bytes but still had
  `generation = 0` and no pending relation.

Three concrete runtime failures were separated by mechanism:

- `40 000 р -> 40 000 h` and `Екб -> Tr,` were short Cyrillic-to-ASCII
  layout candidates that received apply authority and were then explicitly
  undone by the user;
- `cnjq` had the correct raw projection `стой` in the IME prediction path,
  while the after-space correction path performed a second typo pass and
  applied `сотой`.

Canonical correction:

- a one-to-three-character Cyrillic-to-ASCII layout candidate is
  `KeepOriginal`; learned state may not promote it to an automatic edit;
- once the raw layout projection is an established Russian surface, it
  settles before any secondary typo repair.

What was not tested at the time of recording:

- repair of the full correction-core baseline;
- a post-install multi-hour live input window.

Verification update:

- focused structural gate:
  `short_cyrillic_to_ascii_layout_is_never_applyable_from_logs` -> `PASS`;
- focused correction-core gate:
  `short_russian_word_does_not_autoswitch_to_ascii_from_logs` -> `PASS`;
- focused raw-projection gate:
  `stable_layout_projection_precedes_secondary_typo_repair_from_logs` -> `PASS`;
- source-built probes:
  `40 000 р -> None`,
  `Екб -> None`,
  `cnjq -> стой`;
- route contracts:
  `typing_transition_authority_contract = 20/20`,
  `text_mutation_monopoly_contract = 15/15`,
  `input_gate = 6/6`;
- the wider sequential correction-core run was `84/105 PASS`; its remaining
  `21` authority failures are therefore recorded as `WATCH`, not hidden by
  the focused result;
- two representative failures,
  `deterministic_mode_corrects_multiword_wrong_layout_tail` and
  `unique_transposition_certificate_repairs_short_word`, also failed against
  the unchanged `0.2.333` source in an A/B control. The wider red set is
  baseline debt; it is not promoted to PASS by this experiment.

Exact receipt:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_LIVE_INPUT_LOG_FEEDBACK_2026-07-31.json`.

Runtime authority changed:

- `true`, limited to release `0.2.334`:
  short Cyrillic-to-ASCII candidates are kept original and stable raw layout
  projections settle before secondary typo repair.

Installation verification:

- installed `lay 0.2.334`;
- `lay-daemon`, `lay-l3-online`, the GNOME extension and the L1.1 service are
  active;
- the IBus daemon retained PID `3793`;
- the Lay engine changed from PID `3236279` to PID `2989683`, matched the
  release SHA-256 and answered its D-Bus health probe;
- the previous engine mode `lay-ime-us` was restored;
- installed probes remained:
  `40 000 р -> None`,
  `Екб -> None`,
  `cnjq -> стой`.

## 13. Standalone Full-Neighbor V13, 2026-07-30

V13 closes the cold standalone `L2` package over the final global `L1.1`
field. It does not recompile `L1.1` and does not store a second lexical
restorer. The package binds existing `L1.1` terminal identities to a larger
materialized morphology field and local context competition.

```text
L1.1 bounded lattice
-> StandaloneL2Field
   -> terminal/surface form binding
   -> same-lemma expansion
   -> morphology-slot centers
   -> document-split near-neighbor couplings
   -> directional competition edges
-> Winner | Tied | Abstain
-> L3
-> verifier
```

The context teacher was built from a public Russian literature corpus with an
80/20 document-level split. A surface is admitted to the neighbor proof only
when an independent heldout document exists. No surface, lemma, product or
phrase-specific runtime rule was added.

Measured package facts:

```text
source unique surfaces                  1,875,032
L1.1-bound forms                          517,257
L2-materialized forms                   1,357,775
lemma centers                              93,672
morphology bindings                    3,255,785
context modes                              41,967
slot centers                                  225
neighbor couplings                         15,922
directional competition edges             215,121
train scenes                                58,117
heldout scenes                           2,543,808
package bytes                         135,121,803
package size                              128.86 MiB
package SHA-256
bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
```

Fixed heldout proof:

```text
same-lemma total                         2,501,613
same-lemma target coverage              99.998081%
same-lemma false authority                       0

noun target coverage                    100.000000%
adjective target coverage               100.000000%
pronoun target coverage                 100.000000%
verb target coverage                     99.986490%

near-neighbor total                        42,195
near-neighbor target coverage            100.000000%
near-neighbor false authority                      0
near-neighbor tied                         41,832
near-neighbor correct winners                 363

cold load                                  477,477 us
hot p50 / p99                                22 / 97 us
proof workers                                     20
```

The high tied count is intentional: an unseen local scene does not acquire
fake authority merely because several forms share one lemma. The proof gate is
target retention plus zero false authority, not winner count in scenes that
remain linguistically underdetermined.

Product query:

```text
context                 сокольим глазком _
L1.1 seed               посмотреть, evidence 1000
L2 form                 посмотри, evidence 760 + explicit competition 486
L2 local score          1246
readout                 Winner(посмотри)
```

This result comes from corpus evidence keyed by context mode and morphology
features. The executable contains no `посмотреть -> посмотри` branch.

Cold build measurements:

```text
corpus preparation       69.41 s, 99% CPU, 1,972,860 KiB peak RSS
package compile          20.72 s, 332% CPU, 3,557,868 KiB peak RSS
fixed proof              20.03 s, 489% CPU, 3,928,852 KiB peak RSS
```

What was tested:

- full final package decode and standalone status;
- complete same-lemma and near-neighbor heldout denominators;
- per-POS target coverage and false authority;
- exact `L1.1` package fingerprint binding;
- bounded runtime latency;
- a context-driven same-lemma form movement from `посмотреть` to `посмотри`.

What was not tested:

- every possible semantic distinction in unrestricted Russian text;
- multi-day live daemon stability with V13;
- broad discourse meaning beyond the local L2 context window.

Verdict scope:

- `PASS_standalone_field` for packaged local morphology/context competition;
- runtime authority did not change during the cold experiment;
- release installation still requires the common daemon/IME verifier smoke.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RUSLIT_FULL_NEIGHBORS_V13_2026-07-30.json
```

### 13.1 Release cutover 0.2.333

The proven V13 package and matching binaries were installed atomically after
an isolated installation check. Only `lay-daemon.service` and
`lay-l3-online.service` were restarted. The GNOME extension was reloaded to
show the new version; the managed IBus engine was not restarted.

```text
installed version                         0.2.333
installed package        LAY-L2-RU-FULL-v13.bin
installed SHA-256
bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
installed package status                    ready
IBus PID before / after         3236279 / 3236279
tray reported version                     0.2.333
daemon / L3-online service          active / active
daemon cgroup memory after reload           331 MiB
daemon process RSS                      161,668 KiB
IBus process RSS                        130,340 KiB
```

Installed live probes:

```text
Нужно ... Apple b  -> Apple и       selected
Apple b            -> no selection
в коде             -> no correction-core selection
врмея              -> время         selected
```

The double-Shift exact autocorrect rollback contract passed both its static
authority contract and daemon pending-undo runtime test. No physical keyboard
event was injected during release verification.

Runtime authority changed in release `0.2.333`: `true`, only through the
existing L2 local readout, L3 context and transition verifier chain.

### 13.2 Public V13 package distribution, 2026-08-01

GitHub issue `radislabus-star/lay-public#40` exposed a release-distribution
defect rather than an L2 field defect. The source installer required a local
canonical package under `data/l2/`, but that 128.86 MiB artifact is not stored
in the public Git checkout. A clean installation therefore built every Rust
binary and then failed before installing the user service.

Release `0.2.341` makes the proven V13 artifact an immutable GitHub Release
asset and pins its complete contract in the installer:

```text
artifact               LAY-L2-RU-FULL-v13.bin
bytes                                      135121803
SHA-256  bbe67a772b684e0f187483796fca248ac0b10576195b1aa524f0b2bde0f6601e
release URL   .../releases/download/v0.2.341/LAY-L2-RU-FULL-v13.bin
cache                    ~/.cache/lay/models/
install       ~/.local/share/lay/nanda_wave/l2/
```

The resolver accepts, in order, a verified explicit/source artifact, the
already installed artifact, or the verified cache. Only when none exists may
it download over HTTPS. Byte count and SHA-256 are checked before any release
binary is installed and checked again on the atomic package copy. Offline
updates reuse an already verified installation. Missing or corrupt input stops
before a partial binary installation.

Measured release checks:

```text
clean-checkout fixture download and install          PASS
offline reuse of installed package                   PASS
corrupt package rejection                            PASS
no binary installed on package failure               PASS
public install/update/uninstall regressions          PASS
real local V13 bytes and SHA-256                      MATCH
remote release build, 20 Cargo jobs                  PASS, 1m59s
public anonymous HTTPS asset download                PASS, 23.27s
download resolver peak RSS                           13,920 KiB
public downloaded bytes                              135121803
public downloaded SHA-256                            MATCH
isolated cache-to-install route                      PASS
isolated installed version                           0.2.341
local installed version                              0.2.341
local daemon / L3-online                      active / active
local GNOME extension version                        0.2.341
local IBus PID before / after               1630206 / 1630206
```

What was not tested at this checkpoint:

- a completely blank operating-system installation including dependency
  installation, service activation and a new desktop login;
- any new L2 quality behavior, because package bytes and runtime authority are
  intentionally unchanged.

Verdict scope: `PASS_public_install`. Runtime authority changed: `false`.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/PUBLIC_INSTALL_CANONICAL_L2_V13_2026-08-01.json
```

## 13. Pairwise Context Witness Boundary

The canonical live local route is:

```text
L1.1 bounded lattice
-> L2 candidate field
-> L3 directed pair certificate
-> L4 witness resolution
-> transition verifier
-> one selected edit or ABSTAIN
```

L2 context support and an L3 pairwise certificate are different signals. L2
may keep several candidates alive; only the directed L3 certificate identifies
which contextual relation won. L4 must preserve that distinction instead of
merging both signals into one boolean support flag.

The certificate does not manufacture text and does not grant direct apply
authority. It only removes losing semantic classes from the already bounded
L2 lattice. The verifier remains the sole owner of whether the selected edit
may mutate visible text.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_LAYOUT_RUNTIME_CLOSURE_2026-07-30.json
```

## 13. Canonical V7 full-lemma package, 2026-07-30

The final L1.1 base contains one seed for every admitted Russian lemma. The
canonical L2 compiler then materializes non-L1 wordforms inside L2 instead of
requiring L1.1 to duplicate the complete morphology surface set.

```text
L1.1 WordCenter                           852,582
morphology source bindings             3,255,785
unique morphology surfaces             1,875,032
lemma centers                              93,672
unseeded lemmas                                  0
L1-bound forms                            517,257
L2-materialized forms                   1,357,775
competition edges                          54,407
context modes                                 123
package bytes                         130,595,163
package SHA-256  436b2b8cc99f16c48f240f5fbeef0a64dc2ccb7c84b898e948d34f0adaf3e41e
compile wall                                20.42s
compile peak RSS                       3,514,936 KiB
compile swap                                     0
```

Full heldout:

```text
evaluated scenes                        2,501,613
unresolved                                      0
target lattice coverage                   99.9977%
winner top-1                              45.8284%
false authority                                  0
hot p50 / p99                         21 / 97 us
proof workers                                   20
proof wall                                  17.77s
proof peak RSS                         3,881,396 KiB
```

| POS | Cases | Target coverage | Winner top-1 | False authority |
|---|---:|---:|---:|---:|
| adjective | 1,592,125 | 100.000% | 38.812% | 0 |
| noun | 554,148 | 100.000% | 80.545% | 0 |
| pronoun | 46 | 100.000% | 52.174% | 0 |
| verb | 355,294 | 99.984% | 23.124% | 0 |

The low winner percentage is not an error hidden by aggregate coverage. L2
keeps morphologically valid alternatives tied where local evidence cannot
choose safely; L3 and the verifier remain responsible for wider context and
edit authority. Near-neighbor proof is `20/20`.

The old morphology shadow runtime and same-lemma donor are removed from the
live ownership graph. The executable route is:

```text
L1.1 bounded lattice
-> StandaloneL2Field V7
-> one Winner | Tied | Abstain readout
-> L3
-> verifier
```

Tested: complete source corpus, all lemma reachability, full heldout,
per-POS denominators, near-neighbor field, package latency and zero false
authority. Not tested here: broad semantic sentence understanding or a global
IBus restart. Runtime authority did not change during the remote proof.

Code verification:

```text
lexical_grokking unit tests                 103/103
l2_field unit tests                          30/30
context_phase unit tests                     70/70
new/changed owner tests                       5/5
typing transition contracts                 20/20
text mutation monopoly contracts            15/15
IBus committed-tail and double-Shift        18/18
daemon full-undo preservation                 1/1
```

The broad correction-core comparison remains `WATCH`: clean `0.2.329` and this
change fail the same 22 test names in the same remote environment. Baseline was
`89 passed / 22 failed`; this change is `90 passed / 22 failed` because its new
semantic-drift owner regression passes. No new wide failure was introduced.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU_FULL_V7_ALL_LEMMAS_2026-07-30.json
```

## 13. 2026-07-29 Standalone RU L2 V6 Evidence Authority

### 13.1 Kernel Ownership

The accepted standalone package route is:

```text
L1.1 bounded terminal lattice
-> up to 4 evidence-ranked lemma hypotheses
-> L2-owned generated-form decoder
-> global morphology-slot phase
-> lemma-specific neighbor pressure
-> directional competition
-> Winner | bounded Tied lattice | Abstain
-> L3
-> verifier
```

The package stores generated UTF-8 surfaces itself. A form absent from L1.1 has
`l1_terminal_id = u32::MAX` and a valid `decoder_ref` in the L2 decoder. L1.1
therefore owns lexical seed birth, while L2 owns morphology materialization.

Competition provenance is part of the evidence contract:

- ordinary morphology competition may settle forms inside one lemma;
- only an explicit near-neighbor teacher edge may independently authorize a
  cross-lemma competition transition;
- global morphology-slot evidence identifies a grammatical slot, but is not
  independent evidence for lexical lemma identity;
- if cross-lemma evidence is insufficient, the readout preserves the bounded
  candidate lattice instead of manufacturing a singleton;
- finite verb forms with the same person and number remain tied across
  underdetermined tense or mood when no lemma-specific evidence separates them.

No word-specific exception list or target surface rule was added.

### 13.2 Compiled Package

Measured on `e@192.168.3.94`:

```text
source morphology bindings       3,255,785
source unique surfaces           1,875,032
source lemmas                       93,672

admitted lemma centers              76,500
unseeded lemmas                     17,172
admitted forms                   1,410,190
L1.1-bound forms                   500,085
L2-materialized forms              910,105
morphology bindings              2,405,261
context modes                         123
slot centers                          225
neighbor couplings                  11,847
competition edges                   40,491
decoder bytes                   31,824,107

package bytes                   96,594,655
package MiB                          92.12
compile wall seconds                  19.09
compile average CPU                  351%
compile peak RSS KiB             3,511,952
compile swap bytes                       0
```

Artifact:

```text
/home/e/build/lay-l1-shadow/artifacts/l2-v6-evidence-authority-2026-07-29/LAY-L2-RU-FULL-v6.bin
SHA-256 b9b0d43c17dfd55562a42d325ff529d5d070c571dd1ca046ca5135f8b7f0093d
```

### 13.3 Fixed Heldout Proof

Proof artifact:

```text
/home/e/build/lay-l1-shadow/artifacts/l2-v6-evidence-authority-2026-07-29/proof-final-zero-authority.json
```

Measured facts:

```text
heldout scenes available          2,501,613
evaluated with at least one seed  1,847,790  73.863943%
unresolved without any L1 seed      653,823  26.136057%

resolvable target coverage          99.997078%
resolvable winner top-1             46.463072%
resolvable false authority                   0
resolvable abstain                          51

POS          evaluated     target coverage     false authority
noun           554,148        100.000000%                     0
adjective    1,041,226        100.000000%                     0
verb           252,370         99.978603%                     0
pronoun             46        100.000000%                     0

near-neighbor scenes                    20
near-neighbor top-1               100.000%
near-neighbor false authority             0

cold load                           347.807 ms
hot p50                                  20 us
hot p99                                  92 us
proof workers                                20
proof wall seconds                       16.31
proof average CPU                         431%
proof peak RSS KiB                   3,785,392
proof swap bytes                              0
```

The proof passes the decision contract on the resolvable domain. It does not
prove the `17,172` lemmas that have no L1.1-bound form. Those lemmas cannot be
born from the current L1.1 terminal lattice and remain an explicit corpus
boundary, not a hidden failure inside the evaluated denominator.

### 13.4 Rejected Safety Experiments

The following experiments were rejected:

1. seed-count readout majority:
   removed false singletons but also blocked legitimate context-driven
   lower-support lemma transitions;
2. global slot-only tie:
   removed false authority but collapsed winner top-1 to `0.339054%`;
3. treating ordinary within-lemma competition as cross-lemma authority:
   produced `25` false-authority winners on the first full V5 proof.

The accepted rules are structural evidence rules. They do not reference
individual words from the failure set.

### 13.5 Verdict And Authority

```text
standalone package build                         PASS
resolvable per-POS target coverage >=99%         PASS
resolvable false authority = 0                   PASS
near-neighbor top-1 and false authority          PASS
package format and size                          PASS
hot-path latency                                 PASS
all-source-lemma reachability                    WATCH 17,172 unseeded lemmas
isolated full-route V6 compare                   PASS 0 false authority
source default package changed                   true
running IME/daemon authority changed             false
```

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU_FULL_V6_EVIDENCE_AUTHORITY_2026-07-29.json
```

### 13.6 Full L1.1 -> L2 -> L3 -> Verifier Replay

What was tested:

- the complete isolated correction route with the installed L1.1 V8 package,
  canonical RU L2 V6 package, L3, decision core, and verifier;
- both `FullWave` reference and `L2FieldShadow` live-owner routes over every
  correction receipt that contains `lay_from`;
- package discovery through the normal installed path, without an explicit
  `LAY_L2_PACKAGE` override;
- parallel deterministic replay with `20` workers;
- persisted usage-memory rebuild after changing the accepted-event projection.

Measured facts:

```text
correction log records seen                  2,945
records with a replayable lay_from             975
workers                                          20

reference eligible applies                      70
reference applies matching user target          68
reference false authority                        2

L2 V6 owner eligible applies                    28
L2 V6 owner applies matching user target        28
L2 V6 owner false authority                      0

selected surface divergences                    71
selected gate divergences                       76
selected provenance divergences                101

wall time                                    9.77 s
average CPU                                  1,569%
peak RSS                                  543,608 KiB

targeted release tests                         72 PASS
targeted release failures                       0
wide nanda_wave current                    541 PASS / 8 FAIL
wide nanda_wave HEAD baseline              529 PASS / 8 FAIL
```

The final two false-authority cases were not repaired with word-specific
conditions. They exposed a derived-cache compatibility error:

1. automatic `autocorrect` and `layout` applies were already excluded from new
   positive feedback;
2. old schema-13 usage snapshots still contained counts compiled before that
   exclusion;
3. usage snapshot schema `14` invalidates those derived counts and rebuilds them
   from the raw event log;
4. signed-memory state and target IDs cover the complete normalized phrase,
   preserve case and punctuation, and therefore do not collapse unrelated
   scenes onto the last token.

Concrete surfaces from the failure log occur only in regression tests. The
production rule checks event provenance and signed state identity; there is no
word allowlist, denylist, or hardcoded replacement.

What was not tested:

- a restart of the user's global IBus engine or running desktop daemon;
- runtime behavior for the `17,172` source lemmas with no L1.1 seed;
- a claim that the smaller number of eligible V6 applies is a complete
  correction-quality improvement outside the measured user-target receipts.

The wide `nanda_wave` gate is not green, but it did not introduce a new failing
test. The same eight test names fail on `HEAD 0.2.328` and on this change. They
cover stale tracked L3 schema fixtures, historical `LayoutWordCell32` ownership,
legacy FullWave trace expectations, one language-quality fixture, and two
environment-sensitive completion checks. They remain a separate `WATCH`; the
focused L2, signed-memory, transition-identity, and usage-projection tests pass
`72/72`.

Verdict scope:

- the isolated installed-package route passes the zero-false-authority gate on
  all `975` replayable real correction receipts;
- source and release discovery may select L2 V6 by default;
- the already running desktop authority is unchanged until a separate safe
  daemon/IBus restart;
- the package remains bounded by the standalone V6 proof in section 13.3.

Exact receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_V6_LIVE_OWNER_SIGNED_FEEDBACK_2026-07-29.json
```

Remote replay evidence:

```text
/home/e/build/lay-runtime-replay/v6-default-full-owner-schema14.json
/home/e/build/lay-runtime-replay/v6-default-full-owner-schema14.time
/home/e/build/lay-runtime-replay/baseline-0.2.328-nanda-wave.log
/home/e/build/lay-runtime-replay/current-0.2.329-nanda-wave.log
```

## 13. 2026-07-28 IBus L2 Cache Budget

The initial attribution to the L2 lexical cache alone was wrong. The compact
`62,424,748 B` lexical phase package is mmap-backed, but the process also
loaded the L3 composite and retained bounded lexical completion readouts.

```text
initial L2 preload prefixes                 1,536
initial preload material limit                 96
live IME material limit                         48
initial maximum cache entries                1,536
L3 runtime manifest deltas                        0
```

The L2 preload used a different material-limit key from live IME requests and
therefore produced few useful cache hits. More importantly, the zero-delta L3
manifest still passed its complete base package through the shard reducer.
That no-op reduction created the large anonymous runtime regions.

Baseline after warmup:

```text
RSS                           245,812 KiB
PSS                           215,609 KiB
anonymous PSS                 177,772 KiB
file PSS                       37,837 KiB
swap                                0 KiB
```

Three intermediate configurations were rejected:

```text
256 prefixes, material 32, cache 256
  8-second RSS                  110,128 KiB
  2-minute RSS                 217,720 KiB
  verdict                      REJECT: early-only reduction

bootstrap only, material 48, cache 64
  cold "п"                         78,727 us
  cold "пров"                      55,898 us
  cold sentence ending "д"         80,742 us
  verdict                      REJECT: cold latency regression

78-prefix warmup, RU 192 / EN 96, cache 128
  8-second RSS                  106,284 KiB
  2-minute RSS                 118,500 KiB
  5-minute RSS                 226,904 KiB
  5-minute PSS                 196,794 KiB
  5-minute anonymous PSS       159,028 KiB
  verdict                      REJECT: delayed allocator growth
```

The accepted configuration and loader behavior are:

```text
bootstrap preload prefixes                     2
Russian bootstrap prefix                    "пр"
English bootstrap prefix                    "ex"
preload mode                      CompletionOnly
Russian preload/live cache key                192
English preload/live cache key                 96
maximum cache entries                         128
zero-delta L3 manifest              direct base load
L3 shard reduce when deltas == 0          disabled
```

Russian preedit requests `24` candidates and therefore uses
`24 * 2 * 4 = 192`; English requests `12` and uses `12 * 2 * 4 = 96`.
Those exact cache keys are unchanged. Only speculative startup materialization
was narrowed: rare prefixes still traverse the complete optimized DAFSA lane
on first use, and no candidate, posting, or decoded-surface frontier was cut.
A non-empty L3 delta list still uses the existing composite reducer.

Measured on the same T480 after a managed child-engine restart:

```text
metric                      baseline      16 sec       2 min       5 min
RSS                     245,812 KiB  105,376 KiB  105,376 KiB  105,376 KiB
PSS                     215,609 KiB   75,374 KiB   75,373 KiB   75,375 KiB
anonymous PSS           177,772 KiB   40,580 KiB   40,580 KiB   40,580 KiB
file PSS                 37,837 KiB   34,795 KiB   34,793 KiB   34,795 KiB
swap                           0 KiB        0 KiB        0 KiB        0 KiB
```

At five minutes this is `-57.1%` RSS, `-65.0%` PSS, and `-77.2%` anonymous
PSS against the original warm baseline. More importantly, the delayed
five-minute rebound of the rejected 78-prefix configuration did not recur.

The first timing table below was produced by an unoptimized debug test. It is
retained as diagnostic evidence, not presented as production latency:

```text
hot samples                                      140
hot p50 / p90 / p99 / max       29 / 36 / 43 / 53 us
cold "п"                                      50,307 us
cold "пр"                                      2,322 us
cold "пров"                                   43,678 us
cold "file"                                    1,035 us
cold sentence ending "д"                      55,616 us
```

The cache-key mismatch and cold DAFSA path were corrected in `0.2.327`.
Production release measurements for the final two-prefix bootstrap on the same
fixed samples:

```text
sample                         before       final
Russian "п"                    7,990 us    6,908 us
Russian "пр"                     455 us      829 us
Russian "пров"                 8,077 us    6,726 us
English "file"                   116 us      152 us
English sentence ending "d"    2,690 us    2,237 us
Russian sentence ending "д"   11,431 us    8,067 us
Russian long context "при"     1,944 us    2,344 us
hot p99                           22 us       10 us
```

The remaining `6.7-8.1 ms` rare cold cases are genuinely new decoded-form
basins, not cache-key misses. They retain the complete `1,152`-surface material
lane. The runtime now visits atoms without allocating one `Vec<u8>` per byte
n-gram, computes phase and center keys in one pass, carries DAFSA character
depth through recursion, and reuses the terminal character count.

Tested:

- `precognition_candidate_generation_stays_under_budget`: PASS;
- lexical cache projection regression: PASS;
- streaming atom summary parity against materialized atoms: PASS;
- lexical phase runtime completion tests: `9 / 9` PASS;
- zero-delta L3 composite fast-path regression: PASS;
- `scripts/check-lay-changed.sh`: PASS;
- release `lay-ibus-engine 0.2.327` built and loaded;
- live process PID `3236279` used the installed release;
- managed child-engine restart retained an `xkb:ru::rus` fallback and did not
  restart the global IBus daemon;
- no IBus daemon restart and no swap.

Not tested in this checkpoint:

- multi-day cache churn at the 128-entry bound;
- end-to-end physical key-to-GNOME-frame latency;
- full L2/L3 quality proof;
- memory with one or more admitted L3 delta packages.

Verdict scope:

- `PASS_runtime_memory_5m`;
- no scoring, candidate-birth, settlement, package, or authority coefficient
  changed;
- this is not a restoration-quality promotion claim.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_IBUS_CACHE_BUDGET_2026-07-28.json`.

Runtime authority changed:

- `false`.

## 13. 2026-07-27 Full Russian Package And Live Lattice Boundary

What was tested:

- compiled the complete Russian L2 teacher corpus
  `data/morphology/lay_ru_l2_full_pos_v3.tsv`;
- repeated the package build and compared SHA-256;
- ran the fixed full heldout proof with `20` proof workers;
- installed the package and exercised the real
  `L1.1 lattice -> standalone L2 -> correction core` route for
  `звгрузи -> загрузи`;
- measured the standalone field separately from the complete socket route.

Measured package facts:

- package: `data/l2/LAY-L2-RU-FULL-v4.bin`;
- SHA-256:
  `1980f89ca2930dfb4abdba489ebc83313b4e0c1851bd5a29e65b25e838a95108`;
- deterministic repeat: identical SHA-256;
- package size: `23,359,260 B`;
- admitted forms: `500,085`;
- lemma centers: `76,500`;
- morphology bindings: `770,261`;
- slot centers: `221`;
- neighbor couplings: `5,280`;
- competition edges: `18,337`.

Measured fixed heldout facts:

- evaluated scenes: `651,029`;
- unresolved teacher scenes: `1,850,584`;
- target coverage: `99.9963%`;
- winner top-1: `81.8406%`;
- false authority: `0`;
- near-neighbor: `18 / 18`;
- standalone cold load: `121,895 us`;
- standalone hot p50 / p99: `6 / 19 us`.

Per-POS target coverage:

- noun: `100.0000%`;
- verb: `99.9047%`;
- adjective: `99.9923%`;
- pronoun: `100.0000%`.

Live boundary correction:

- `Restore` remains the final L1.1 `Winner | Tied | Abstain` contract;
- a separate `Lattice` service request now exposes the bounded L1.1 frontier
  before final authority classification;
- the canonical L2 route consumes this lattice and no longer depends on the
  collapsed L1.1 winner;
- for `звгрузи`, the lattice contains `загрузить`; standalone L2 expands the
  same lemma and the focused live route selects `загрузи`;
- focused live route test: `1 / 1 PASS`.

Measured integration limitation:

- complete socket-route latency for the focused case was approximately
  `39.5 ms` p50 and `39.7 ms` p99;
- this exceeds the accepted live p99 budget of `5 ms`;
- standalone L2 is not the bottleneck; the remaining cost is L1.1 lattice
  materialization and socket/decode work.

Build CPU observation:

- the canonical production release profile uses `codegen-units=1` with LTO,
  so the final crate/link stage is inherently close to one-core;
- the integration build with `codegen-units=20`, LTO disabled and
  `CARGO_BUILD_JOBS=20` reached approximately `1200% CPU`;
- full proof readout is parallel, while parsing the `435 MB` teacher TSV
  remains single-threaded technical debt.

## 14. 2026-07-27 L1.1 Service CPU And Lattice Transport Experiment

What was tested:

- measured why the remote 20-thread machine stayed at low CPU under concurrent
  L1.1 lattice requests;
- compared candidate-birth atom and posting budgets on the full L1.1 package;
- replaced the diagnostic Lattice socket payload with the compact typed
  `terminal_id + surface + authority + score_milli` transport;
- replaced one new OS thread per socket connection with a fixed reusable
  20-worker pool and a bounded queue;
- tested a smaller 64-candidate phase frontier and checked the complete
  `звгрузи -> загрузи` route.

Measured service facts:

- the old thread-per-connection service processed 20,000 requests at
  `937.1 req/s`;
- the fixed 20-worker pool processed the same 20,000 requests at
  `5,582.9 req/s`, a `5.96x` gain;
- a 100,000-request run sustained `5,931.3 req/s`;
- a separate 40,000-request CPU sample used `110.09` service CPU-seconds in
  `6.415` wall-seconds, or an average of `17.16` CPU cores;
- all `100,000 / 100,000` long-run probes retained the required L1 seed
  `загрузить`;
- resident memory after warming 20 reusable scratches was approximately
  `2.0 GiB`.

Measured posting-budget facts for `звгрузи`:

- one birth atom per channel selected 10 atoms and 10,219 postings, touched
  7,493 centers, but lost `загрузить`;
- two birth atoms per channel selected 20 atoms and 28,094 postings, touched
  20,199 centers, and retained `загрузить` at rank 5;
- a 20,000 global posting budget selected 18 atoms and 17,374 postings, touched
  13,489 centers, and retained `загрузить` at rank 5.

Compact transport facts:

- a 16-seed response is 1,410 bytes;
- a 64-seed response is 5,573 bytes;
- with the unchanged full 128-candidate phase frontier, limit-16 hot latency
  measured p50 `4,711 us` and p99 `5,443 us`;
- this still misses the strict `5,000 us` gate by `443 us`.

Rejected experiment:

- reducing only the L2-facing phase frontier from 128 to 64 produced p50
  `4,018 us` and p99 `4,810 us`;
- L1 still retained `загрузить` at rank 5;
- the complete canonical L2 route lost `загрузи`;
- verdict: `REJECT_quality_regression`; the phase frontier remains 128.

New canonical L2 blocker:

- the teacher corpus contains
  `F загрузить загрузи verb:imp_excl:sg:imp:perf`;
- the L1.1 and L2 package fingerprints match;
- the current L1.1 package does not contain an exact `загрузи` WordCenter;
- direct `Restore("загрузи")` returns `ABSTAIN`, surface `загрузки`,
  geometry distance 1;
- canonical L2 can only emit terminal IDs materialized by the L1.1
  DecoderGraph, so it cannot output a surface missing from L1.1;
- current L2 uses `strongest_lemma_count`, so the larger noun seed cohort
  `загрузка` suppresses the weaker verb lemma seeded by `загрузить`;
- the readout becomes `Abstain` over noun forms and never births `загрузи`;
- two general corrections are required: admit the morphology surfaces required
  by canonical L2 into the L1.1 corpus, then replace count-majority lemma
  selection with a bounded evidence-weighted multi-lemma settlement;
- neither correction may add a word-specific exception.

What was not tested:

- no fixed heldout per-damage-class proof was run for the experimental 20,000
  posting budget;
- no full ambiguity or false-certainty proof was run for the compact transport;
- no canonical L2 heldout proof was run after replacing
  `strongest_lemma_count`;
- no production daemon latency was measured after installation.

Verdict scope:

- fixed worker pool: `PASS_throughput_and_retention_probe`;
- compact typed transport: `PASS_protocol`, `FAIL_latency`;
- 20,000 posting budget: `PASS_probe_only`, not promoted;
- 64 phase frontier: `REJECT`;
- complete canonical L2 route: `FAIL`;
- runtime authority changed: `false`.

Exact receipt:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L11_L2_SOCKET_POOL_AND_LATTICE_BUDGET_2026-07-27.json`

Exact receipts:

- `docs/structural_gates/receipts/L2_CANONICAL_FULL_COMPILE_V7_2026-07-27.json`;
- `docs/structural_gates/receipts/L2_CANONICAL_FULL_PROOF_V7_2026-07-27.json`.

What was not tested or promoted:

- the complete protected IME live gate after installing a release containing
  the new `Lattice` protocol;
- the full correction-core suite still has `82 PASS / 21 FAIL`, inherited from
  the old live-owner assumptions and separate transition-phase authority;
- the `5 ms` complete-route latency gate has not passed;
- no global IBus restart was performed.

Runtime authority changed:

- `false`; package and functional route are proven, but promotion remains
  blocked by complete-route latency and protected live regressions.

## 13. 2026-07-27 Canonical Noun Package Full Safety Proof

What was tested:

- deterministic rebuild of
  `/home/ubu/projects/lay/data/l2/LAY-L2-RU462K-NOUN-v1.bin`;
- fixed heldout readout over every available noun scene;
- per-feature winner, tied-target coverage, abstain, and false-authority
  denominators;
- cold load, hot p50/p99, package size, and peak compile/proof RSS.

Measured facts:

```text
forms                         462,314
lemmas                         47,766
morph bindings                633,016
train scenes                    1,548
heldout scenes                554,148
context modes                      59
slot centers                       60
neighbor couplings              1,548
competition edges               6,144
package bytes              19,244,056
package SHA-256  db8087fb642d29fe270133b5eb08dac12828db9679e64899dddf691ea3b86be6

evaluated                 554,148 / 554,148
winner correct                   450,772
winner top-1                     81.3450558%
tied contains target             103,376
target coverage                 100.0000000%
abstain                                0
false authority                        0

second locative target coverage 100.0000000%
second locative false authority          0

compile wall                         12.82 s
compile peak RSS                   664,392 KiB
proof wall                            ~56 s
proof peak RSS                     746,572 KiB
proof cold load                    852,717 us
proof hot p50 / p99                 68 / 152 us
```

The earlier `68` false-authority cases had two structural causes:

- the context identity retained only the nearest preposition, collapsing
  distinct governors such as `лежит на _` and `сосредоточен на _`;
- an ambiguous surface could borrow pressure from an unrelated homonymous
  lemma and become a false singleton despite equal same-lemma slot evidence.

The canonical correction is:

- context mode and lexical anchor now cover the same bounded two-token window
  used by the scene wave;
- pressure is accumulated inside one lemma and selected across alternative
  lemmas without additive homonym amplification;
- equal positive slot evidence inside one lemma produces `Tied`, never an
  artificial `Winner`.

The reduction from `81.7952966%` to `81.3450558%` winner top-1 is not hidden:
`2,496` additional syncretic or homonymous cases moved to a target-containing
tied lattice. Target coverage improved to `100%` and false authority fell to
zero. This is the required safety trade: ambiguity remains explicit instead of
being reported as false certainty.

What was not tested:

- near-neighbor competition, because the current fixed corpus contains only
  same-lemma morphology scenes;
- verbs, adjectives, pronouns, and auxiliaries;
- installed live package behavior and daemon/IME regression;
- standalone runtime promotion.

Verdict scope:

- canonical Russian noun same-lemma/morphology-slot safety gate: `PASS`;
- full canonical L2: `NOT COMPLETE`;
- runtime authority changed: `false`.

Exact receipts:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU462K_NOUN_COMPILE_CONTEXT_V2_2026-07-27.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2_CANONICAL_RU462K_NOUN_PROOF_FULL_V6_2026-07-27.json`

## 14. 2026-07-27 Full Russian POS Teacher And Cross-Lemma Contract

What was implemented:

- the existing noun feature values remain binary-compatible;
- the free bits in the 32-bit feature mask now encode verb, adjective,
  pronoun, number, gender, person, tense, mood, aspect, and POS-specific form
  kind;
- infinitives, finite verbs, imperatives, gerunds, full/short adjectives,
  participles, comparatives, pronouns, and auxiliary forms can share the same
  fixed-width `MorphBinding` package section;
- `LemmaCenter.primary_pos` is derived from admitted bindings instead of being
  hard-coded to noun;
- typed `NT` / `NH` teacher rows represent bounded cross-lemma competition;
- cross-lemma edges are indexed from both endpoint lemma families so runtime
  field birth can reach the relation from either active L1.1 seed;
- proof now has an explicit near-neighbor denominator and failure examples
  instead of reporting `tested=false`.

Cold-teacher generation facts on the remote 20-core build machine:

```text
noun visible form centers                  462,314

adjective/participle lemmas                 48,294
adjective/participle bindings            2,240,679
adjective/participle scenes              1,600,679

verb/infinitive/gerund lemmas                21,772
verb/infinitive/gerund bindings             381,936
verb/infinitive/gerund scenes               337,418

pronoun lemmas                                   24
pronoun bindings                                160
pronoun scenes                                  122

typed near-neighbor relations                    20
corpus bytes                            432,122,626
generation wall                              52.39 s
generation peak RSS                      1,495,744 KiB
```

The corpus artifact is:

`/home/e/projects/lay-l2-build/data/morphology/lay_ru_l2_full_pos_v1.tsv`

Measured facts do not yet imply package/runtime promotion. At this point:

- corpus generation: `PASS`;
- feature/parser/compiler microtests: `PASS`;
- full package compile: running;
- full per-feature and near-neighbor heldout proof: not yet measured;
- runtime authority changed: `false`.

## 2.2 Second Internal Donor: Near-Neighbor Lexical Competition

What was tested for this code step:

- `scripts/cargo-guard.sh check --lib`: passed;
- `scripts/cargo-guard.sh test --lib near_neighbor_`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2938`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 26 / 134`,
  `compact_apply = 36 / 134`,
  `shadow_apply = 36 / 134`,
  `user_target_match.compact = 6 / 134`,
  `user_target_match.shadow = 6 / 134`,
  `user_target_match.both = 6 / 134`.

Measured implementation facts:

- the near-neighbor donor also lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- it runs after same-lemma morphology filtering and before unified candidate
  materialization;
- it only inspects already-born shadow lexical candidates;
- it only runs on Cyrillic local competition;
- it builds one bounded near-neighbor cohort around the current lexical leader;
- it only acts when the current leader also wins the internal near-neighbor
  strength readout by a conservative margin;
- on a `Winner`, it filters weaker near-neighbor competitors from that cohort
  and retags the promoted shadow candidate with `L2FieldShadowNearNeighbor`.

What was not tested in this step:

- fixed heldout `L2` proof for near-neighbor competition;
- replay examples where the donor should return explicit `Tied` or `Abstain`;
- live IME authority change;
- latency and RSS of the near-neighbor donor under daemon load.

Verdict scope:

- `L2FieldShadow` now contains a second real internal donor above the input
  contour: bounded near-neighbor lexical competition;
- this donor remains shadow-only and did not change runtime authority;
- on the measured 134 real correction-log inputs, it preserved selected surface
  parity and selected gate parity with `CompactL2`;
- this is still not yet proof of a full standalone canonical `L2` local field.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_NEAR_NEIGHBOR_LEXICAL_2026-07-26.json`

Runtime authority changed:

- `false`

## 15. 2026-07-30 One-symbol layout birth for L3 context

`L2FieldShadow` now births the complete bounded one-symbol layout lattice
instead of asking the multi-character layout helper for one winner. For a
single ASCII key, `short_token_candidates` supplies the exact keyboard
projection and configured visual alternatives. The bridge preserves trailing
whitespace and sends every surviving surface through ordinary transition
admission.

```text
one-symbol surface
-> L2 short-token candidates
-> bounded competing surfaces
-> transition admission
-> L3 sentence-context pressure
-> Winner | Tied | ABSTAIN
-> verifier
```

No one-symbol candidate receives authority from its birth order. When two
layout alternatives remain close and L3 has no pairwise certificate or strong
phrase evidence, transition admission keeps the result unresolved. This
prevents an arbitrary first candidate from becoming an autocorrection.

The same candidate constructor is used by the cold L3 probe, so learning and
runtime no longer disagree about the `b` lattice. The visual replacements are
the existing configured lexical surfaces, not a product or sentence-specific
branch.

Measured:

```text
generic candidate birth test                    PASS
unknown-context abstain test                     PASS
L3 context-phase tests                         74/74
targeted L3 relation proof                       PASS
full 80k differential L3 proof                   PASS
new false authority                                 0
```

What was not tested:

- every one-symbol visual ambiguity;
- multi-day live service behavior;
- physical input in every toolkit.

Runtime authority changed during this experiment: `false`.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L3_LAYOUT_PAIRWISE_DELTA_2026-07-30.json
```

## 2.3 Local Readout Safety Gate Inside `L2FieldShadow`

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib l2_field`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- targeted route spot checks on
  `докурчиват`, `ЯДРА`, `ене`, `смеа`, `сделам`, `сли,`, `вошеьные`:
  selected surface parity restored;
- `scripts/cargo-guard.sh run --bin lay-nanda-wave-eval -- --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2939`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 16 / 134`,
  `compact_apply = 25 / 134`,
  `shadow_apply = 25 / 134`,
  `user_target_match.compact = 7 / 134`,
  `user_target_match.shadow = 7 / 134`,
  `user_target_match.both = 7 / 134`.

Measured implementation facts:

- the generic local readout shell still lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`;
- the near-neighbor donor is now explicitly prevented from collapsing the field
  when its internal winner is not the current lexical leader;
- the same donor may still return `Tied` or `Abstain`, but no longer upgrades a
  nonleader surface into a singleton shadow winner;
- the local readout now keeps the bounded lexical surface field intact on the
  measured real regressions where compact `L2` remained tied or selected a
  different surface through the shared lattice;
- the route-level tests now lock parity on those cases through
  `/home/ubu/projects/lay/src/correction_core/candidate_sources.rs`;
- runtime authority did not change.

What was not tested in this step:

- fixed heldout `L2` proof for local readout winner/tie calibration;
- live IME/daemon authority promotion;
- standalone `L2` package latency, RSS, and cold-load budget;
- broader donor families beyond same-lemma morphology and near-neighbor lexical
  competition.

Verdict scope:

- `L2FieldShadow` now has a safer internal local readout shell above the
  self-born lexical field;
- on the measured 134 real correction-log inputs, selected surface parity and
  selected gate parity with `CompactL2` are restored after tightening this
  winner admission;
- provenance still diverges by design on selected Nanda surfaces because the
  shadow route uses `L2FieldShadowSurface` instead of `L2LexicalPhaseCell32`;
- this is still shadow-only evidence and not yet a promotion to runtime
  authority.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_LOCAL_READOUT_GATED_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_TARGETED_NONLEADER_CASES_2026-07-26.json`

Runtime authority changed:

- `false`

## 2.4 First Owner-Swap Pass: `L1.1` Seeded Birth Inside One Local Field

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib l2_field`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `scripts/cargo-guard.sh run --bin lay -- --compare-candidate-routes --candidate-route l2-field-shadow 'врмея '`
  kept selected surface parity and selected gate parity while collapsing the
  shadow route to one local readout candidate;
- `scripts/cargo-guard.sh run --bin lay -- --compare-candidate-routes --candidate-route l2-field-shadow 'пку '`
  restored abstain parity on the short ambiguous token after adding the short
  seeded-birth guard;
- `scripts/cargo-guard.sh run --bin lay-nanda-wave-eval -- --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_seen = 2939`,
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 17 / 134`,
  `compact_apply = 26 / 134`,
  `shadow_apply = 26 / 134`,
  `user_target_match.compact = 7 / 134`,
  `user_target_match.shadow = 7 / 134`,
  `user_target_match.both = 7 / 134`.

Measured implementation facts:

- the seeded birth merge lives in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs` inside
  `shadow_surface_seed_candidates(...)`;
- `L2FieldShadow` no longer emits a separate shadow-side `L2FieldShadowL11`
  candidate during route materialization;
- authoritative `L1.1` restore output is now internalized into the same local
  `L2` field as bounded surface evidence and may surface as
  `L2FieldShadowSurface` or `L2FieldShadowReadout`;
- existing lexical candidates receive only bounded `L1.1` score/overlap boosts;
- seed-only insertion is bounded to at most two authoritative surfaces and only
  for token length `>= 4`;
- token length `<= 3` bypasses seeded birth entirely, which restored parity on
  the measured `пку` short-signal regression;
- on `врмея`, the compact route still exposes a wider 7-candidate lattice while
  `L2FieldShadow` now settles the same selected surface into one local readout
  candidate with no surface/gate divergence;
- runtime authority did not change.

What was not tested in this step:

- fixed heldout `L2` proof for the seeded-birth route;
- live IME/daemon runtime promotion;
- latency and RSS budget of the per-request `L1.1` seed request path under
  sustained daemon load;
- broader seeded-birth replay beyond the measured 134 real correction-log
  inputs and the targeted `врмея` / `пку` probes.

Verdict scope:

- this is the first real owner-swap pass toward
  `L1.1 bounded lattice -> one real L2 local field -> one local readout -> L3 -> verifier`;
- inside `CandidateReadoutRoute::L2FieldShadow`, `L1.1` now feeds the local
  field as formal bounded seed birth rather than as a separate route-level
  sidecar candidate;
- on the measured 134 real correction-log inputs, selected surface parity and
  selected gate parity with `CompactL2` are preserved;
- provenance still diverges by design because the shadow route now reports its
  own internal field ownership instead of `L2LexicalPhaseCell32`;
- this remains shadow-only evidence and is not yet a runtime promotion.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_L11_SEEDED_BIRTH_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_VRMEYA_L11_SEEDED_BIRTH_2026-07-26.json`
- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_PKU_L11_SEEDED_BIRTH_2026-07-26.json`

Runtime authority changed:

- `false`

## 2.5 Live Owner Flip For IME And Daemon Local Correction

What was tested for this code step:

- `scripts/cargo-guard.sh test --lib ime_correction`: passed, `25 / 25`;
- `scripts/cargo-guard.sh test --lib l2_field`: passed, `9 / 9`;
- `scripts/cargo-guard.sh check --bin lay`: passed;
- `scripts/cargo-guard.sh check --bin lay-daemon`: passed.

Measured implementation facts:

- `/home/ubu/projects/lay/src/candidate_contract.rs` now makes
  `CandidateReadoutRoute::live_default()` return `L2FieldShadow`;
- the live local IME route under
  `/home/ubu/projects/lay/src/ime_correction.rs` now expects boundary-owned
  Space/autocorrect authority as `L2FieldShadowBoundary`;
- `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs` no longer carries
  temporary `LAY_DEBUG_SHADOW_L2_FIELD` logging;
- `/home/ubu/projects/lay/src/ime_correction.rs` no longer carries temporary
  `LAY_DEBUG_IME_CORRECTION` logging;
- the shadow local donor winner multiplier is explicit and fixed as
  `SHADOW_DONOR_WINNER_WEIGHT = 5`;
- tied and abstain donor bonuses remain bounded and unchanged; only the winner
  multiplier was made explicit.

What was not tested in this step:

- fixed heldout `L2` proof for the live local owner route;
- standalone `L2` package latency, RSS, and cold-load budget;
- broader `L2` donor families beyond same-lemma morphology and near-neighbor
  lexical competition;
- promotion of `L2FieldShadow` from donor-reusing owner contour to a fully
  standalone packaged `L2`.

Verdict scope:

- the live local IME/daemon correction route is now owned by one real
  `L2FieldShadow` local field above bounded `L1.1` evidence;
- the route now reads as
  `L1.1 bounded lattice -> one real L2 local field -> one local readout -> L3 -> verifier`
  for live local correction;
- this is a runtime authority change for the local route;
- this is not yet proof that the final standalone canonical `L2` package is
  complete.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_LIVE_OWNER_IME_DAEMON_GATE_2026-07-26.json`

Runtime authority changed:

- `true`

## 2.6 Divergence Buckets On Real Corrections Window

What was tested for this code step:

- `scripts/cargo-guard.sh run --bin lay-nanda-wave-eval -- --l2-route-compare-report --limit 200 --examples 200`
  on `/home/ubu/.local/share/lay/corrections.jsonl`.

Measured implementation facts on Monday, July 27, 2026:

- the same `134` usable records in the `200`-line window now reran as
  `surface_diverged = 19 / 134`,
  `gate_diverged = 19 / 134`,
  `provenance_diverged = 33 / 134`;
- `reference_apply = 27 / 134`;
- `shadow_apply = 30 / 134`;
- user-target exact-normalized matches on this rerun were:
  `reference = 7 / 134`,
  `shadow = 8 / 134`,
  `both = 5 / 134`;
- the divergences are not one amorphous problem; they split into five concrete
  buckets:
  - `8` cases: shadow false apply or false suggest after reference abstain;
  - `3` cases: shadow found the user target while reference abstained;
  - `3` cases: shadow missed a user-target hit that reference found;
  - `4` cases: reference picked an off-target winner while shadow abstained;
  - `1` case: both routes committed to different off-target winners.

Operational interpretation:

- the main unfinished `L2FieldShadow` problem is now explicit: `8 / 19`
  divergent cases are unsafe local-field winner births where the route should
  emit tied lattice or abstain instead of selecting a local winner;
- the next recall problem is narrower: `3 / 19` cases where `L2FieldShadow`
  abstains but `FullWave` actually hits the target;
- the route also already has `3 / 19` positive wins that should be preserved
  while tightening the unsafe bucket.

What was not tested in this step:

- fixed heldout proof after any bucket-specific field/readout changes;
- full IME/daemon replay of these exact `19` cases;
- runtime latency or RSS impact after tightening local readout.

Verdict scope:

- this rerun converts the vague “18/134” or “19/134” discussion into a real
  `L2` work queue;
- the priority is no longer abstract parity with `FullWave`, but the concrete
  `8` shadow false-apply/false-suggest cases that should collapse into tied or
  abstain readout;
- this is measurement only; runtime authority did not change in this step.

Receipt path:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_DIVERGENCE_BUCKETS_CORRECTIONS_200_2026-07-27.json`

Runtime authority changed:

- `false`

## 3. Ownership

### 3.1 L1.1 Ownership

`L1.1` owns only:

- one damaged token at a time;
- lexical restoration of that token;
- bounded candidate lattice for that token;
- `Winner / Tied / Abstain` over lexical restoration evidence;
- lexical evidence such as geometry, positive phase, backward reconstruction,
  anti evidence, ambiguity shells, and crystallization state.

`L1.1` does not own:

- phrase-local ending choice;
- same-lemma form competition;
- context-driven candidate reordering;
- neighbor-governed morphology slot choice;
- multiword competition;
- destructive edit authority.

### 3.2 L2 Ownership

`L2` owns:

- the first local field above the bounded `L1.1` lattice;
- same-lemma form competition;
- near-neighbor lexical competition;
- local morphology-slot inference;
- preposition / particle / auxiliary / agreement cues;
- phrase-local tied vs winner vs abstain readout;
- candidate evidence attribution for local decisions.

`L2` is the owner of decisions such as:

```text
посмотреть / посмотри / посмотрим
дом / дома / домом
времени / время / временами
посмотри / просмотри / подсмотри
```

These are local-field decisions, not `L1.1` decisions and not `L3` decisions.

### 3.3 L3 Ownership

`L3` owns only broader context pressure:

- wider phrase memory;
- semantic suppression or support;
- longer-range preference shifts;
- maintaining ties when local evidence is not decisive.

`L3` must not become a substitute for the missing real `L2`.

### 3.4 Verifier Ownership

The verifier remains the sole owner of destructive edit authority:

```text
selected local winner
-> structural verification against visible snapshot
-> AuthorizedEdit or no-op
```

No `L1.1`, `L2`, or `L3` object may bypass this boundary.

## 4. Critique Of The Current Live Shape

The current live shape is useful, but architecturally wrong in three ways:

1. lexical restoration and local phrase competition are still split across
   separate candidate producers;
2. morphology knowledge exists, but does not yet sit inside one canonical local
   field above `L1.1`;
3. the old `CompactL2` route is no longer executable, but the live
   `L2FieldShadow` route still reuses donor packages and is not yet a
   standalone packaged canonical `L2`.

That means the current runtime still behaves like:

```text
several candidate birth routes
-> merge
-> decide
```

instead of the desired:

```text
L1.1 bounded lexical lattice
-> one real L2 local field
-> one local readout
-> L3 pressure
-> verifier
```

The purpose of the new `L2` is to remove that ownership drift.

## 5. Canonical L2 Memory

The canonical `L2` package should be centered on stable IDs, not raw string
heuristics.

### 5.1 Main Memory Objects

```text
L2 Field Package
|
+-- FormCenterRef
|   stable reference to an existing L1.1 visible form
|
+-- LemmaCenter
|   lexical identity shared by several visible forms
|
+-- MorphBinding
|   FormCenterRef <-> LemmaCenter <-> slot/features
|
+-- LocalContextMode
|   bounded phrase-local scene identity
|
+-- SlotPhaseCenter
|   learned local evidence for a slot or form family
|
+-- NeighborCoupling
|   support/repel relation from nearby token classes or surfaces
|
+-- CompetitionEdge
|   candidate-vs-candidate local suppress/support relation
|
+-- TieCalibration
|   honest thresholds for Winner / Tied / Abstain
|
+-- DecoderRef
    materializes visible UTF-8 output from FormCenterRef
```

### 5.2 FormCenterRef

`L2` must not duplicate the lexical surface memory that already belongs to
`L1.1`. It should reference it.

Minimal shape:

```text
FormCenterRef
  l1_terminal_id
  script_flags
  length_bucket
  decoder_ref
```

Meaning:

- `L1.1` keeps the visible form identity;
- `L2` points at that identity and competes using local context.

### 5.3 LemmaCenter

`LemmaCenter` is the local family owner for several visible forms:

```text
LemmaCenter
  lemma_id
  primary_pos
  form_range
  local_context_profile_range
  competition_edge_range
```

For example one `LemmaCenter` can bind:

```text
посмотреть
посмотри
посмотрим
посмотрел
посмотрела
```

`L2` then chooses between these forms from local scene evidence.

### 5.4 MorphBinding

`MorphBinding` binds visible form to lemma and slot:

```text
MorphBinding
  form_center_ref
  lemma_center_id
  feature_mask
  support
  flags
```

For Russian the slot must encode at least:

- part of speech;
- case;
- number;
- gender where relevant;
- person;
- tense;
- mood;
- aspect;
- infinitive / finite / imperative distinction.

The existing shadow teacher in
`/home/ubu/projects/lay/src/nanda_wave/morphology_phase/field.rs` is the
starting donor for this layer.

### 5.5 LocalContextMode

`LocalContextMode` is a compact identity for phrase-local scene features:

```text
left function token class
+ right function token class
+ punctuation boundary class
+ local position / adjacency mode
+ optional neighboring lexical class anchors
```

This object must stay bounded and cheap. `L2` is not a full sentence semantic
graph.

### 5.6 SlotPhaseCenter

`SlotPhaseCenter` is the learned local scene pressure for a slot or tight form
group:

```text
scene
-> positive subcenters
-> anti subcenters
-> score for slot/form family
```

Examples:

- imperative scene;
- infinitive-governed scene;
- noun after preposition requiring one case;
- adjective-noun agreement scene;
- plural noun scene;
- finite verb after pronoun scene.

### 5.7 NeighborCoupling

`NeighborCoupling` carries short-range local support or repulsion:

```text
neighbor pattern
-> supports candidate family
-> or repels candidate family
```

Examples:

- `в + noun(prepositional)`
- `к + noun(dative)`
- `не + imperative / finite contrast`
- adjective agreement cues;
- stable local two-word motifs.

### 5.8 CompetitionEdge

`CompetitionEdge` is explicit candidate-vs-candidate local pressure:

```text
candidate A
candidate B
scene key
support delta
anti delta
tie-allowed flag
```

This is the core object that prevents the field from collapsing into one global
string score.

### 5.9 TieCalibration

`TieCalibration` must be learned from evidence, not hard-coded around one
example:

```text
minimum positive
minimum margin
tie window
abstain window
false-authority ceiling
```

The important principle is honest local uncertainty:

- if one same-slot candidate family wins clearly, emit `Winner`;
- if several candidates remain locally valid, emit `Tied`;
- if the scene is too weak, emit `Abstain`.

## 6. Canonical L2 Runtime Path

The runtime path must become:

```text
input token
-> L1.1 lexical restoration
-> bounded L1.1 lattice
-> L2 field birth
-> same-lemma expansion
-> near-neighbor expansion
-> local slot scoring
-> pairwise competition
-> Winner | Tied | Abstain
-> L3 broader pressure
-> TransitionDecisionCore
-> verifier
-> AuthorizedEdit or no-op
```

### 6.1 L2 Field Birth

`L2` begins from the full bounded `L1.1` lattice, not only its top-1.

For each `L1.1` candidate:

1. map `terminal_id -> FormCenterRef`;
2. expand to its `LemmaCenter`;
3. add same-lemma alternate forms that are legal for the bounded local scene;
4. add explicit near-neighbor competitors already linked by local competition
   edges;
5. keep source attribution from `L1.1`.

### 6.2 Same-Lemma Expansion

If `L1.1` restores a lexical family correctly but not the local form,
`L2` must be able to walk within that family:

```text
L1.1 surface winner = посмотреть
local scene = imperative
L2 family walk = {посмотреть, посмотри, посмотрим, посмотрел, ...}
L2 local winner = посмотри
```

This is the main reason `L2` cannot be replaced by raw lexical restoration.

### 6.3 Near-Neighbor Expansion

`L2` must also carry local competition between geometrically close but
context-distinct families:

```text
посмотри
просмотри
подсмотри
досмотри
```

These edges must be explicit and bounded. `L2` should not brute-force the full
lexicon at runtime.

### 6.4 Local Readout

Local readout must use:

- `L1.1` evidence floor;
- slot evidence;
- same-lemma pressure;
- neighbor couplings;
- pairwise competition edges;
- tie / abstain calibration.

`L2` readout emits:

```text
ordered local lattice
+ local verdict
+ evidence attribution
+ tie/abstain reason
```

### 6.5 IME And Daemon Readout

IME and daemon must consume the same `L2` readout.

IME remains only:

- display backend;
- commit backend;
- accepted-completion source.

IME must not own a separate lexical or morphology brain.

## 7. Proposed Code Ownership

The target code layout should converge to:

```text
src/nanda_wave/l2_field/
|
+-- model.rs
+-- compiler.rs
+-- runtime.rs
+-- format.rs
+-- proof.rs
+-- teacher.rs
+-- bridge.rs
+-- mod.rs
```

Proposed responsibilities:

- `model.rs`
  core records: `LemmaCenter`, `MorphBinding`, `CompetitionEdge`,
  `LocalContextMode`, `TieCalibration`;
- `compiler.rs`
  build package from corpora, logs, shadow teachers, and calibrated evidence;
- `runtime.rs`
  field birth, bounded expansion, local competition, local readout;
- `format.rs`
  deterministic binary package format;
- `proof.rs`
  fixed heldout, per-class local decision proof, tie honesty, latency, RSS;
- `teacher.rs`
  cold teacher import from morphology/package builders and future corpora;
- `bridge.rs`
  the one adapter from current correction-core route into the new `L2`.

Existing donors:

- `src/nanda_wave/lexical_grokking/restoration.rs`
  donor for the `L1.1 -> lattice` boundary;
- `src/nanda_wave/morphology_phase/field.rs`
  donor for morphology-slot field concepts;
- `src/correction_core/candidate_sources.rs`
  current live merge route that the new bridge must replace;
- `src/nanda_wave/l2_candidate_phase.rs`
  separate transition-phase donor, but not the canonical local `L2`.

## 8. Proof Contract

Promotion of the new `L2` requires a fixed proof that measures the local-field
job directly, not only lexical restoration.

### 8.1 Required Proof Families

The fixed `L2` proof must contain at least:

1. same-lemma form choice;
2. local morphology slot choice;
3. near-neighbor lexical competition;
4. tie honesty on ambiguous scenes;
5. abstain honesty on underdetermined scenes;
6. zero direct mutation authority bypass;
7. hot-path latency and bounded RSS.

### 8.2 Required Scoreboard

Every run must report:

```text
same-lemma top-1
same-lemma tie coverage
same-lemma false authority

morphology-slot top-1
morphology-slot authority
morphology-slot false authority

near-neighbor top-1
near-neighbor tie coverage
near-neighbor false authority

ambiguous tied accuracy
abstain honesty

package bytes
cold load
steady RSS
hot p50 / p99
```

Aggregate winners are not enough. Per-class denominators are required.

### 8.3 Runtime Promotion Gate

The standalone packaged `L2` may replace donor-reusing `L2FieldShadow` only
when:

1. fixed local proof passes;
2. live shadow route shows no unsafe regression;
3. `TransitionDecisionCore` behavior remains verifier-safe;
4. IME and daemon both read the same emitted lattice;
5. evidence attribution remains inspectable.

## 9. Cutover Plan

The cutover should happen in five explicit stages.

### 9.1 Stage A: Package Build

Compile a standalone `L2` package from:

- `L1.1` terminal identities;
- morphology bindings;
- local context scenes;
- competition edges;
- tie calibration.

No runtime authority yet.

### 9.2 Stage B: Reference Compare Readout

This stage is complete for live ownership. The remaining compare shape is now:

```text
CandidateReadoutRoute::FullWave
CandidateReadoutRoute::L2FieldShadow
```

`L2FieldShadow` already owns the live local route; `FullWave` remains the
reference compare path.

### 9.3 Stage C: A/B Receipts

On fixed corpora and selected live logs compare:

- `FullWave` reference;
- live `L2FieldShadow`;
- later standalone packaged `L2`.

The comparison must show where the new route wins, ties, abstains, or regresses.

### 9.4 Stage D: Runtime Flip

This stage is complete for the live local route:

```text
live route
L2FieldShadow
-> one local readout above bounded L1.1 input
```

`L1.1` is no longer a separate live sidecar on that route; it is already
internalized as bounded lexical input to the local field.

### 9.5 Stage E: Remove Ownership Drift

After standalone package promotion:

- remove the remaining donor-reuse ownership drift inside `L2FieldShadow`;
- keep the old lexical route only in historical receipts, not in executable
  code paths;
- keep morphology and transition-phase teachers as teachers, not hidden live
  owners.

## 10. Forbidden Behaviors

The canonical `L2` must not:

- re-implement raw lexical restoration already owned by `L1.1`;
- collapse the `L1.1` lattice to one candidate before local competition;
- brute-force the whole lexicon at runtime;
- depend on IME-only state as its main evidence source;
- silently replace `Tied` or `Abstain` with fake certainty;
- bypass `TransitionDecisionCore` or the verifier;
- hide its local winner without evidence attribution.

## 11. Canonical Summary

The clean target is:

```text
damaged token
-> L1.1 restores lexical basin
-> L2 chooses locally valid form and neighbor winner
-> L3 adds broader phrase pressure
-> verifier decides whether edit may happen
```

The main correction to the current runtime is simple:

`L1.1` must stop being only an extra candidate source.
It must become the formal lexical base of one real `L2`.

## 12. 2026-07-27 Local Readout Safety Tightening

What was changed in this step:

- `L2FieldShadow` local readout now demotes tied/abstained local surface cohorts
  from `Eligible` to `SuggestOnly` instead of leaving the correction core to
  pick an arbitrary surface winner;
- short dense growth clusters now emit `l2_field_shadow_local_tie` or
  `l2_field_shadow_local_abstain` for affected local surface candidates;
- live double-`Shift` rollback of layout-only autocorrect was restored in the
  daemon correction memory path.

Measured facts from direct route compares on 2026-07-27:

- `смеа `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored;
- `докурчиват `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored;
- `сли, `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity preserved;
- `слои `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored after demoting the short growth tail
  (`соли`, `слови`, `слоги`, `сложи`, `сломи`) to suggest-only;
- `ене `:
  `FullWave.selected = None`,
  `L2FieldShadow.selected = None`,
  parity restored; `пение ` remains suggest-only under
  `short_sparse_multi_omission_requires_tie_or_context`;
- `сделам `:
  direct live compare currently gives
  `FullWave.selected = "сделай "`,
  `L2FieldShadow.selected = "сделай "`,
  so it must stay in live parity coverage rather than the abstain-only bucket.

What was tested:

- direct `lay --compare-candidate-routes` probes for
  `смеа `, `докурчиват `, `сли, `, `слои `, `ене `, `сделам `;
- focused unit coverage in
  `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`
  for dense missing-letter and long-form tie clusters;
- focused daemon/lib undo checks for double-`Shift` rollback memory.
- exact receipt:
  `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_SHORT_GROWTH_GATES_2026-07-27.json`.

What was not yet completed:

- refreshed full 134-record divergence bucket receipt after this tightening;
- wider replay over the remaining divergence buckets after the short-growth fix;
- promotion of this safety pass into a finished live-owner compare gate.

Runtime authority changed:

- `false`

## 13. 2026-08-17 Candidate-Specific Target Authority

The current canonical correction route still has an authority ownership defect,
not merely a ranking defect:

```text
canonical_text_readout_observed
├── L1.1 -> Productive V90 -> field-wide authority
└── old Boundary projection -> independently admitted candidate
```

`PreparedCanonicalTokenField` currently converts every field with more than one
surface into a common L3 tie. Correct lexical/geometric targets are therefore
often demoted to `SuggestOnly`. In parallel, the historical Boundary producer
can emit an `Eligible` split and beat the correct whole-token target. L3 cannot
honestly repair this because context may select among grounded targets but may
not invent lexical grounding.

Revision 5 separates cacheable material, frame settlement and event authority;
the earlier draft had combined them:

```text
context-neutral `TokenContour | BoundaryWindowContour`
-> MaterialTargetIdentityV1 + bounded semantic witnesses + completeness
-> exact frame span/projection/replay
-> CandidateState: Born | Grounded | Rejected
-> exact edit-footprint conflict cohort
-> CohortVerdict: Winner | Tied | ABSTAIN
-> L2Certified or calibrated ContextCertified
-> DecisionCore -> one transaction protocol -> one state-correct event mutator
```

The owning paper now compares the current dual route, widened-frontier,
context-first and separate-fast-mutator alternatives against the selected
route across retention, false authority, latency/RSS, cache/reload, online
learning, concurrency, rollback and removal cost. The selected route accepts a
possible reduction in automatic coverage when evidence is incomplete; that is
reported as abstention rather than hidden as a quality win.

`MaterialTargetIdentityV1` contains no focus, tail epoch, replacement span,
surrounding context, context score or selected winner. Those belong to
`FrameTargetIdentityV1` and `SettledFrameEvidenceV1`. A frame rejection can never be
cached as lexical material.

`BoundaryWindowContour` binds both exact tokens and the exact separator. A
split uses `CompositeBoundaryGroundingV1` to ground every emitted part and its
segmentation; a merge binds the merged lexical center while clean two-token
preservation remains a separate veto. The token-only identity from Revision 3
could not represent this contract.

This corrects a second current-source defect: Productive V90 currently receives
the two-left-token scene while constructing `PreparedCanonicalTokenField`, and
`CanonicalTokenKey` caches `scene_bytes`. That can hide an alternative before
the cohort is formed. The selected architecture requires context-neutral target
membership; different left contexts over the same lexical input may change only
frame settlement, never target birth, grounding or completeness. If Productive
cannot enumerate that bounded set without context, promotion stops at the
Productive contract instead of approximating a singleton.

The decision vocabulary remains:

```text
CandidateState
  Born | Grounded | Rejected

CohortVerdict
  Winner | Tied | ABSTAIN

AuthorityCertificate
  L2Certified | ContextCertified
```

Target rejection, lexical settlement reasons, absolute authority blockers,
original preservation and context observations are separate types. Context
never changes `CandidateState` or cohort membership. A calibrated context
selector may resolve an explicitly admitted complete morphology/lexical tie,
but never overflow, incomplete enumeration or multiple edit components.
`Tied` contains at least two known grounded targets; an incomplete field with
zero or one retained target is
`ABSTAIN(IncompleteEnumeration)`, not a one-member tie.
A conflicting `Born` target from an incomplete or failed grounding lookup is an
unresolved alternative: it is not a grounded tie member, but it blocks a
grounded singleton until the lookup becomes complete or rejects it. Cohorts are
canonically ordered by exact footprint, target, evidence and completeness bytes
before hashing; producer arrival, score and provenance cannot affect context
identity.

Each exact target retains a deterministic bounded set of at most four
independent relation/grounding/geometry witnesses. A target and witness
overflow blocks automatic authority instead of silently pruning proof. The
original input is a separate preservation/default state, never a replacement
candidate or target grounding.

Witness independence is defined by relation, canonical operator, exact
grounding, package generations and derivation root. Producer/source provenance
is merged diagnostic metadata and cannot manufacture a second witness. The
24-byte witness ceiling is achieved only with typed references into immutable
prepared-field tables; all dereferenced bytes and table storage remain part of
the retained-memory gate.

The selected automatic contract is:

```text
exact target identity
+ grounded typed relation witness
+ exact replay geometry
+ complete conflict-cohort enumeration
+ one compatible target
+ no preservation veto, hard contradiction or overflow
-> L2Certified

multiple compatible grounded targets
-> Tied
-> bounded context selection
-> ContextCertified or ABSTAIN
```

Score-margin authority is disabled. `ContextCertified` is also disabled for
live emission until a separate fixed heldout context-authority calibration
passes with zero false authority. Current context is exactly two left tokens;
it is not sentence understanding. Until that gate, a lexical tie remains
`ABSTAIN` and context may only rank display or shadow output.

An online overlay generation is `DisplayOnly`, `ShadowSettlement` or
`AuthorityEligible(proof_receipt_hash)`. Merely binding a generation ID does
not authorize it. New online learning can affect display, but cannot change
automatic correction until the exact overlay bytes pass their own promotion
gate.

Boundary split/merge becomes a typed hypothesis inside the same prepared field
and competes with the whole-token target before cohort settlement. Source IDs,
lane kinds and raw surface cardinality remain provenance/readout metadata; they
never grant authority.

The L1.1 seed-service lattice can contain 128 members, while the typed
restoration readout retains at most 32 plus explicit `TiedOverflow`. The stored
Productive target envelope then remains `32 + 32 + 8 + 2 = 74`; it is not an
enumeration-completeness claim. L1.1 `TiedOverflow`, Productive, contour or
Boundary overflow is preserved explicitly and blocks singleton authority. In
particular, retaining two Boundary hypotheses cannot hide a third conflicting
split and then call the first one a winner.
The same distinction applies to computation: deterministic posting, replay,
grounding and operator-work ceilings must be frozen before Slice 2. Exhaustion
is `Overflow(WorkBudgetExceeded)`, never `Complete`; the 74-target storage bound
does not authorize unbounded context-neutral enumeration.

Display, explicit Tab, committed-tail Space, active-composition Space, stale
publication and double-Shift rollback are separate event routes. They share
immutable prepared target material and exact identities, but each has one
event-specific rank/authorization/mutation owner. Tab consumes a visible user
selection receipt; automatic Space requires `L2Certified` or
an independently promoted `ContextCertified`; rollback consumes the exact
autocorrection receipt. Deferred rollback is frame-bound, finite, never
auto-retried and may be retried only by one later explicit gesture while valid.
Physical active-composition Space remains a separate raw commit route from
library APIs that merely calculate hypothetical active-composition corrections.

Tab, committed Space, active-composition Space and rollback share one explicit
authorization/transaction protocol, not one physical mutator. Corrected/raw
Space and rollback use the committed-tail mutator; Tab and active Space use the
active-composition mutator. Committed Space selects exactly one of
`CorrectAndAppendSpace | RawSpace` before final authorization and never re-enters
selection after refusal or output start. A backend refusal may restore a
pending rollback only when zero output is proved. Every other zero-output
attempt is `AttemptedNoEffect` only after complete effect-vector equality and a
durable terminal state. Once any output was emitted, the event enters bounded
`RecoveryRequired`; raw Space partial output is included, and no compensation
runs until the exact full effect snapshot is observed.
Rollback deadlines bind a monotonic boot/process epoch and are invalid after an
epoch change.

Journal-required output uses the Revision-8 `OrderedGroupCommitV1` paper
strategy: exact intent is durable before effect, the prior terminal state may be
co-committed with the next prepare, and one `SameLineageStateBarrierV1` blocks
both Lay and native state changes until terminal durability. Two independent
foreground durability waits per steady-state event are rejected before runtime
integration. A non-overlapped terminal state may still cause a separate
background storage sync; actual sync calls, bytes, queueing and I/O time remain
measured costs, and any inherited next-event wait is foreground latency.
`RecoveryQuarantined` is nonterminal and native-only; explicit
reset preserves the incident, rotates journal generation, establishes a new
baseline and emits no correction learning.
Startup reconciliation first uses a zero-mutator exact observation route. Only
an observed compensable state may enter the separately authorized recovery
mutator; both paths persist exact terminal settlement before opening the
lineage barrier. Reset has its own durable receipt, while emergency key cleanup
has an independent idempotent effect proof and cannot settle text output.

Measured diagnostic baseline before implementation:

```text
focused exact-layout admission        5 / 5 PASS
immutable full canonical gate         18 / 36 PASS
immutable deep representative series   0 / 12 PASS
immutable baseline parity             46 / 49 PASS
remote execution receipts             97 / 97 valid
future source-absence proof            59 / 59 PASS
Slice 0 baseline freeze                    PASS
measured current-route latency           7 / 7 PASS
complete EventRuntimeBudgetV1     FAIL_COVERAGE, not PASS
runtime authority changed                  false
```

The complete evidence model, state machine, operator matrix, cache and learning
semantics, proof denominators, migration slices and rollback boundary are in:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/ime-canonical-target-authority-paper-2026-08-17.md`

Source-bound review result:

```text
current observed correction route                 VETO
source markers                                  21/21
event-route design, event-only scope                PASS
event route size                     33 nodes / 51 edges / 12 routes
material/frame/context-neutral design               PASS
material route size                   23 nodes / 44 edges / 7 routes
output transaction design                              PASS
output route size                    49 nodes / 94 edges / 57 routes
structural design issues                                  0
durable diagnostic raw logs                       13/13
frozen event/Boundary case definitions            108
baseline parity cases                              49
future-contract cases assigned to owning slices    59
immutable remote execution integrity             PASS, 97/97 valid
immutable assertion result                       FAIL, 64/97 PASS
frozen baseline parity                           FAIL, 46/49 PASS
future-contract source absence                   PASS, 59/59
Slice 0 baseline freeze                          PASS only in frozen scope
measured current-route latency                   PASS, 7/7 implemented strata
complete EventRuntimeBudgetV1                    FAIL_COVERAGE, four Slice 7 routes absent
external historical partial manifest             PRESENT, superseded/non-promotable
final Slice 0 repository results manifest         PRESENT, baseline-only PASS
Slice 1 implementation preflight V7               SUPERSEDED, placeholder hashes
Slice 1 implementation preflight V8               PASS, exact immutable manifests
Slice 1 evidence vocabulary                       PASS, 13/13 contracts
Slice 1 semantic output parity                     PASS, 49/49 exact
Slice 1 remote RSS delta                           PASS, 1,280/5,120 KiB
Slice 2 deterministic work budgets       PASS, exact V90/V9/V13 tuple
Slice 2 material/frame shadow             PASS, runtime unchanged
Slice 2 upstream incomplete fields        877/1,300, authority blocked
Slice 7 durability microproof             UNEXECUTED BLOCKER
Slice 9 numeric context risk policy       UNFROZEN BLOCKER
runtime authority changed                           false
deployment authorized                               false
```

The previous broad `READY_TO_IMPLEMENT` and Revision-3 Slice 1 receipts are
superseded: they covered
multiple behavior-changing slices with one source baseline and did not model
the contracts above. The Revision-3 material route also placed context evidence
before cohort construction. Each source-mutating slice now requires a new preflight.
The 36-case IME denominator is frozen into disjoint first-loss subsets for
birth/retention, lexical authority and context settlement. Slice 8 cannot claim
final `36/36` when a nonempty context subset remains for Slice 9.
An intermediate `LexicalOwnerRelease` may claim only its birth/retention and
lexical-authority subsets; only `FullTargetAuthorityRelease` may claim final
`36/36`.
The migration has Slice 0 plus twelve implementation/release slices: immutable
rerun, vocabulary, context-neutral material/frame binding, candidate state,
conflict cohort, missing target birth, Boundary internalization, crash-safe
event transaction, lexical live readout, separately calibrated context
authority, compatibility-route removal, performance/failure proof, and then a
versioned physical release. The exact ordering and gates live in the paper
linked above. This paragraph describes the pre-Slice-1 checkpoint; the bounded
vocabulary-only implementation result is recorded below.

The 2026-08-20 immutable attempt-3 run provides complete execution identity,
not a promotion result. Its exact external evidence root is:

```text
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820
```

The source archive contains 2,214 hash-valid files and is bound to 97 exact
remote invocations. All 97 produced one valid receipt; 64 assertions passed and
33 failed. Series results are `18/36`, `0/12` and `46/49`. The three baseline
failures are `false-split-ambiguous-short-shift`,
`false-split-non-boundary-source` and `split-authority-binds-target`. All 59
future required-test names are byte-absent from frozen `src/` and `tests/`.
The failures are frozen behavior debt for their owning later slices, not a
blocker to the vocabulary-only Slice 1; requiring behavior repair first would
contradict Slice 1's parity-only contract.

Their first shared mechanism is not a word list. Boundary birth, target
binding, proposal admission and DecisionCore currently infer separate surface-
shape booleans instead of carrying one exact typed Boundary evidence object.
This allows verifier-valid shape to act as authority, loses one admitted birth
path during target binding, and applies a Boundary-specific downgrade to a
non-Boundary producer before preservation. The selected Slice 1 vocabulary must
make observed contour, exact split target, segmentation, operator and
completeness one value consumed by both IME and full correction; the verifier
remains safety-only.

The authoritative local V3 private-owner probe measures the exact installed
engine through the outer D-Bus request/reply clock. Printable, committed Space,
explicit accept, rollback, layout, refusal and owner-handoff strata pass their
individual limits with p99 values `0.347`, `8.566`, `0.864`, `0.967`, `0.772`,
`0.401` and `0.289 ms`. Rollback restores `128/128` distinct frozen inputs;
14,546 trace rows are contiguous with zero typed failures. Active-composition
Space, repeat identity, durability prepare/co-commit and the same-lineage
barrier are absent from the installed runtime and belong to Slice 7. Therefore
current-route latency is `PASS 7/7`, while complete `EventRuntimeBudgetV1`
remains `FAIL_COVERAGE`. Missing Slice 7 routes do not block the vocabulary-only
Slice 1, but they block Slice 7 exit and final promotion.

The repository artifacts are exact and machine-readable:

```text
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/immutable-rerun/source-at-execution.sha256-manifest.json
  SHA-256 45389cef6c5843473799a5e2df0c066c12ac77e43b5b598d2c3d91158f5af511
docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/immutable-rerun/results-manifest.json
  SHA-256 986ea9be4a89c64c3e29b4102d9d554fb63843bd32dd0baa79de812a573c97da
```

The results manifest claims `PASS_BASELINE_FREEZE_ONLY`; it preserves remote
assertion quality as `FAIL 64/97` and complete latency as `FAIL_COVERAGE`.
Neither deployment nor a runtime-owner change follows from it.

Exact first-loss receipt:

```text
/home/ubu/projects/lay-immutable-evidence/ime-target-authority-slice0-20260820/baseline49-boundary-first-loss-analysis.json
```

Exact active design receipts:

- `docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_EVENTS_ROUTE_DESIGN_PASS_V4_2026-08-17.json`
  proves pre-output event/command/gesture identity with 33 nodes, 51 edges and
  12 routes;
- `docs/structural_gates/receipts/LAY_IME_TARGET_MATERIAL_FRAME_ROUTE_DESIGN_PASS_V6_2026-08-17.json`
  proves preservation-first material/frame order with 23 nodes, 44 edges and 7
  routes;
- `docs/structural_gates/receipts/LAY_IME_TARGET_OUTPUT_TRANSACTION_ROUTE_DESIGN_PASS_V8_2026-08-17.json`
  proves the durability strategy, three state-specific output mutators,
  same-lineage dispatch/hold barrier, nonterminal quarantine, explicit reset,
  split observed/compensating startup recovery and proved key cleanup with 49
  nodes, 94 edges and 57 routes;
- `docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_BASELINE_2026-08-17/event-and-boundary-cases-v7.json`
  freezes 108 definitions: 49 executable baseline cases and 59 future contracts
  assigned to their first required migration slice;
- `docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE1_IMPLEMENTATION_PREFLIGHT_V7_BLOCKED_2026-08-17.json`
  reports 39 pinned baselines, 10 source scans, 13 mapped tests and 11 invariants
  but is superseded because its two immutable-rerun entries carry impossible
  placeholder hashes;
- `docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE1_IMPLEMENTATION_PREFLIGHT_V8_2026-08-20.json`
  pins the real immutable manifests and passed before the isolated source edit;
- `docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE1_EVIDENCE_VOCABULARY_2026-08-20/final-receipt.json`
  proves the vocabulary-only Slice 1 implementation and no runtime authority or
  deployment change. It does not authorize Slice 2.

### Slice 1 Evidence Vocabulary Result, 2026-08-20

`src/typing_transition/target_evidence.rs` now owns one bounded vocabulary for
semantic witness roots, exact material/frame identities, enumeration
completeness, proof-scoped overflow, prepared-material leases, conflict cohorts
and future certificates. Legacy L2 and live replacement enums project into it
only at named adapters. Common evidence is computed on demand and is not yet a
rank, admission, display, cache, mutation or verifier owner.

```text
remote builds                              4/4 PASS
focused contract groups                    5/5 PASS
frozen executions                         49/49 valid
normalized semantic logs                  49/49 exact
baseline status differences                         0
adapter fault injections                    2/2 rejected
TargetWitnessV1                                  24 B
TargetEvidenceSetV1                            128 B
evidence payload per prepared field          9,472 B
complete PreparedTargetMaterialV1           11,376 B
160-field retained delta                  1,820,160 B
remote median RSS delta                       1,280 KiB
RSS delta ceiling                              5,120 KiB
runtime authority changed                           false
deployment actions                                      0
```

The payload and complete-object ceilings are distinct: 74 evidence sets consume
exactly `9,472 B`; target identities and the material envelope bring the object
to `11,376 B`, still below the `12,288 B` total retained-delta ceiling.
Malformed accelerators are reconstructed from exact semantic roots before
retention. A narrow completeness claim requires a non-zero exhaustive
partition-proof reference. Scope mismatch becomes an order-independent
whole-field integrity failure. Compile-time exhaustive destructuring keeps
frame-bound state outside cacheable prepared material.

The three frozen baseline failures remain unchanged at `46/49`; Slice 1 proves
parity, not repair. The next gate is a new Slice 2 preflight that freezes
deterministic per-producer and aggregate enumeration-work budgets before
context-neutral prepared material and exact frame binding are implemented.

### Slice 2 Deterministic Work Measurement Result, 2026-08-20

Proof-only instrumentation now measures complete per-field work before the
material/frame split changes any runtime owner. It counts posting visits,
relation replays, grounding lookups, generated logical targets and operator
steps independently for canonical grounding, cold binding and Productive
traversal, then reconstructs an exact aggregate. The measurement is disabled
unless `LAY_PRODUCTIVE_WORK_MEASUREMENT` is present and changes no candidate,
rank, verdict, package or authority.

The remote release binary consumed the exact active package tuple:

```text
Productive V90  40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44
L1.1 V9         bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7
canonical L2    cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
axis schema     b5b24f952e83e1e9738db0f89a9d2e9e16eaf7af754990114a562d42be3c060b
frozen manifest 13e2db3470006303de3d87b2f999988db1661157760fec447e54ac37d5b495ea
```

The `13 x 1` smoke passed `13/13`. The fixed `13 x 100` proof passed
`1,300/1,300` unique measurement samples with exact aggregate reconstruction
and no counter errors. The largest measured field required `229` grounding
lookups, `88,130` posting visits, `103,803` aggregate relation replays, `6,348`
generated logical targets and `382,340` aggregate operator steps. Cold binding
owns `90,225,248 / 92,746,251` total operator steps, so future preparation must
cache context-neutral binding material rather than replay it during frame-only
readout.

Frozen budgets for the exact V90/V9/V13 generation are:

```text
producer                  posting   relation   grounding   generated   operator
canonical_grounding             0          0         256           0          0
cold_binding               131,072    131,072           0           0    524,288
productive_traversal             0      8,192           0       8,192     16,384
aggregate                  131,072    131,072         256       8,192    524,288
```

Each non-zero ceiling is the smallest power of two not below the corresponding
fixed-proof maximum. Zero producer dimensions remain forbidden rather than
receiving speculative capacity. The aggregate budget is checked independently;
it is deliberately not the sum of producer ceilings. Budget exhaustion must
produce explicit incomplete material and block automatic authority.

Safety and semantic parity remained unchanged:

```text
evaluated semantic comparisons             2,600
H / H -> B / B -> S0                 1,280 / 0 / 0
false singleton / integrity errors          0 / 0
probe structural parity                2,600/2,600
non-latency normalized parity SHA-256
905ce2d6ad7cb5c28e852fa0e603927feabb5e2afde031ab7826c8c51f256b4b
measurement sample digest
31e49ea32a391580f2f9b5c56256a5aecf1b2a25a0d39a7072c228d713e62ea8
```

The instrumented proof measured `19.47 s`, `660%` CPU and `391,408 KiB` peak
RSS. This is proof throughput evidence only; stage telemetry changes timing and
the host was not an isolated hot-path benchmark. No latency promotion follows.

Tested: exact package identity, complete fixed work denominator, deterministic
producer/aggregate accounting, semantic non-latency parity and fail-closed
measurement integrity. Not tested: the actual context-neutral material/frame
implementation, exhaustion behavior in that implementation, live cache leases,
daemon/IBus performance, multi-client applications or automatic authority.
Runtime authority changed: `false`. Deployment actions: `0`.

Exact receipts:

```text
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_WORK_MEASUREMENT_2026-08-20/final-receipt.json
/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_WORK_MEASUREMENT_2026-08-20/slice2-work-full-13x100.json
```

Verdict: `PASS_WORK_BUDGET_FREEZE_RUNTIME_UNCHANGED`. This closes only the
numeric blocker. The next source mutation requires a new implementation
preflight for context-neutral prepared material, exact frame binding and
shadow-only readout.

### Slice 2 Context-Neutral Material/Frame Result, 2026-08-20

The prepared-material boundary is now implemented in the isolated
`codex/l1-exact-peak-search` worktree. Context-neutral Productive enumeration
owns lexical target formation. `PreparedTargetMaterialV1` owns deterministic
target/evidence storage and completeness. `ExactInputFrameV1` owns volatile
source-window, caret, selection, preedit, case and punctuation identity. A
bounded lease registry prevents use after field generation reuse and limits
retention to 32 fields with at most 8 consumers each.

```text
L1.1/canonical/Productive evidence
-> context-neutral bounded enumeration
-> PreparedTargetMaterialV1 + exact digest
-> PreparedMaterialLeaseV1
-> ExactInputFrameV1
-> digest + generation + UTF-8 frame validation
-> proof-only shadow replay
```

The fixed remote denominator passed all semantic and integrity contracts:

```text
material pairs / unique pairs       1,300 / 1,300
work-budget passes                   1,300 / 1,300
context comparisons                         3,900
frame bindings                       3,864 / 3,864
stale-frame accepts                    0 / 3,864
H -> B / B -> S0                            0 / 0
false singleton / integrity errors          0 / 0
semantic non-latency gate                   PASS
```

Completeness remains a separate authority dimension: `423` materials are
`Complete`, while `877` are `UPSTREAM_INCOMPLETE`. The latter are preserved for
diagnostics and later upstream work but cannot become automatic authority.

Measured proof resources were `20.72 s` wall, `636%` CPU and `392,048 KiB`
peak RSS on 20 workers. Instrumented maximum class p99 was `19.258 ms`, so no
hot-path latency or promotion PASS is claimed. Runtime authority changed:
`false`; deployment actions: `0`; installed version remains `1.0.33`.

Receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE2_MATERIAL_FRAME_2026-08-20/final-receipt.json`

The next gate is Slice 3 candidate-state implementation preflight. It must
consume the material completeness state without weakening it and must not add a
parallel live owner.

### Slice 3 Candidate-State Shadow Result, 2026-08-20

`CandidateStateV1` is now derived only after material lease, exact frame,
replacement span and projected-target replay validation. The state owner is
`productive_v1/candidate_state.rs`; it has no API for context, scoring, ranking,
display, admission or mutation.

```text
BoundFrameTargetV1
-> per-witness geometry assessment
-> complete target-grounding namespace check
-> Born | Grounded | Rejected(reason)
-> absolute authority blocker set
```

Target and field completeness are intentionally distinct. Incomplete target
grounding remains `Born`. Field-level `UPSTREAM_INCOMPLETE` does not erase a
valid exact grounding for a retained target, but its blocker prevents every
future Winner or certificate until the complete conflict field is proven.
Original preservation is a separate frame-bound result outside target storage.

The fixed `13x100` proof measured:

```text
candidate-state derivations               3,864 / 3,864
Born / Grounded / Rejected                0 / 3,864 / 0
false grounding / cross-context mismatch          0 / 0
stale candidate-state accepts                     0
original-preservation comparisons          3,900 / 3,900
H -> B / B -> S0                                  0 / 0
probe parity                                2,600 / 2,600
false singleton / integrity errors                0 / 0
```

The proof consumed `19.77 s` wall, `674%` CPU and `392,048 KiB` peak RSS on 20
workers. Instrumented Productive traversal still puts maximum class p99 at
`16.181 ms`, so live latency and promotion remain failed and unclaimed. Slice 3
is a shadow-only semantic PASS; runtime authority, packages, installed version
`1.0.33`, daemon and IBus were unchanged.

Receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE3_CANDIDATE_STATE_2026-08-20/final-receipt.json`

Next gate: Slice 4 complete conflict-cohort construction and deterministic
`Winner | Tied | ABSTAIN` shadow verdict.

### Slice 4 Conflict-Cohort Shadow Result, 2026-08-20

`productive_v1/conflict_cohort.rs` now owns the single post-validity lexical
cohort. It binds exact edit footprints, merges semantic duplicates, constructs
conflict components and consumes original preservation before deriving a
context-free `Winner | Tied | ABSTAIN` verdict. L3/L4, scores, rank and live
admission do not participate.

The fixed `13x100` proof derived `3,900/3,900` cohorts with zero context/hash
mismatch, incomplete Winner, false singleton, lost grounded target,
multiple-component authority or preservation bypass. The measured verdicts
were `0 Winner`, `1,050 Tied` and `2,850 ABSTAIN`; `877/1,300` fields remain
explicitly upstream-incomplete and therefore cannot issue authority.

This is a shadow-only semantic PASS. Productive maximum class p99 remains
`14.566 ms`, above the `5 ms` promotion gate. Runtime ownership, installed
packages, daemon/IBus and version `1.0.33` were unchanged.

Receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE4_CONFLICT_COHORT_2026-08-20/final-receipt.json`

Next gate: Slice 5 missing-target birth and retention shadow.

## TD-118 Process-Cold Package Identity Reuse, 2026-09-05

### What Was Tested

TD-118 measured the optimized installed-package TD-117 test route in fresh
processes, then removed only duplicate full-file identity scans admitted by the
baseline. The test host retained the installed Lay 1.0.63 package tuple. Seven
baseline and seven candidate processes were paired and interleaved, with the
order reversed in alternating pairs. Page-cache state was uncontrolled and
warmable; every process began with empty Rust generation cells.

The implemented admission route is:

```text
validated L1.1 receipt
-> retain verified artifact bytes + SHA-256

compact Canonical mmap
-> one combined checksum + full-backing SHA-256 traversal
-> retain LoadedPackageIdentity
-> existing V13 identity validation
-> existing Productive L1.1 + Canonical identity validation
```

Reference/non-compact Canonical packages still use pathname metadata and
SHA-256. The service-side L1.1 admission remains independent. No digest is
claimed for the separate service process's loaded backing.

Focused loader, corruption, cache, lifecycle, and prefetch verification passed
`46/46`. Exact witness replay passed `7/7`; TD-117 passed `28` with one intended
ignore; TD-007 passed `10` with two intended ignores. The optimized installed-
package E2E passed in every one of the `14/14` paired processes with unchanged
semantic results:

```text
плозо          -> плохо       complete Winner, capability 1, one verified edit
рабоает        -> работает    complete Winner, capability 1, one verified edit
востанавливать -> two targets complete Tied, capability 0, no edit
```

### Measured Facts

```text
metric                              baseline       candidate       delta
process-cold Wave median          3,707,823 us    2,841,853 us    -23.355%
process-cold Wave max             4,221,946 us    3,222,062 us    improved
process wall median                    7.40 s          6.58 s    improved
warm Wave median                    32,873.5 us      27,912.5 us  -15.091%
warm Wave max                       55,411 us        34,425 us    -37.873%
peak RSS median                    515,112 KiB      516,524 KiB    +0.274%
peak RSS max                       516,644 KiB      516,772 KiB    +0.025%
```

All seven paired cold deltas favored the candidate. All paired runs recorded
zero major faults. The stage attribution moved rather than disappeared:
Canonical median rose from `1,098,217 us` to `1,814,327 us` because it now owns
the cryptographic traversal of its retained mmap; V13 fell from `674,215 us` to
`32,177 us`; Productive fell from `1,791,988 us` to `879,893 us`.

Paired `openat` traces established the intended mechanism:

```text
artifact / owner                         baseline opens   candidate opens
Canonical package, complete process                   3                 1
L1.1, independent service/ensure process              2                 2
L1.1, Productive process                              2                 1
Productive P2M / P2R                                1 / 1             1 / 1
```

The final candidate release-test ELF is `21,913,768 B`, SHA-256
`3085371ebbcaa1c00bae973855ed18b272160174a4aa77c55baf403df01ec549`.
It was compiled with Rust `1.97.1` under the guarded 20-CPU profile. No candidate
binary was installed. Installed Lay remained `1.0.63`, the existing L1.1 PID
remained `2201795`, and no Lay or global IBus process was restarted.

### What Was Not Tested

- physical preedit rendering or user-perceived first-Space latency;
- process-start-to-ready publication;
- storage-cold behavior after dropping host caches;
- population p95/p99 guarantees from seven pairs;
- a service-loaded backing digest handshake;
- a 1.0.64 build, installation, or rollback transaction.

### Verdict Scope

The process-cold performance gate passes: median admission improved by
`865,970 us` (`23.355%`), warm median/max improved, and peak-RSS median/max stayed
within the 5% ceiling. Candidate generation, lattice retention, ranking,
authority, verifier, SafetyGate, mutation, learning, lifecycle, package format,
and public protocol are unchanged. Runtime authority changed: `false`.
Deployment actions: `0`.

Exact receipt:

`tech_debt/evidence/td118-process-cold-admission-verification-v1.json`

## TD-123 cold V90 admission: implementation preflight, 2026-09-08

Measured: exact installed IME with correctly routed nine dependencies computes
the complete cold lexical frame, field generation2, then marks its correction
superseded with no newer input. `actual-ime-v90` retains the2500ms prepared
timeout FAIL. The separate `actual-ime-v90-evaluated` matrix completes seven
cases: damaged2/6 and clean1/1; known first full computation is superseded,
later complete frames are prepared. Tied/Abstain versus Winner separates the
remaining authority refusals. These diagnostic examples are not heldout quality.

Source fact: scheduling captures cache epoch1; the first successful V90 loader
clears all dependent caches and advances it to2. The publishing predicate
requires exact epoch equality. Initial admission is mutex-cached and no V90
field can exist before that sole loader returns. Unavailable results are not
cached as lexical truth. The exact concurrent first-loader thread is unknown;
causal before/after proof must retain the same client order and package bytes.

Selected design (9/10, design assessment): initial admission fills the existing
epoch, matching the documented contract. Remove only the first-success clear.
Explicit reload still replaces the cached success/error and invalidates all
dependents. No new owner, timer, cache, fallback, retry or public API.
Alternative (5/10): clear inside admission mutex closes the peer-read window
but still invalidates the already stamped request. Alternative (4/10): preload
before stamping moves model IO onto input handling, or background warmup merely
hides the race. Restamping a result after evaluation is rejected: it would
assert an unproven material identity.

Consequence analysis: candidate lattice, ranking, cohort evidence, L3/L4,
DecisionCore, SafetyGate and verifier are unchanged. Initial cache retention
cannot retain an earlier V90 field because none exists before the sole load;
all model hashes remain in the cache key. Explicit successful or failed reload
continues epoch invalidation, including in-flight work; publication, Space
lookup and apply mismatch guards stay exact. No new package/delta discovery or
learning/feedback path. A previously discarded eligible cold correction may
now be applied through the existing verifier, so this is a behavioral fix and
requires actual-client proof, not a claim of changed model quality.

Expected CPU/RSS/allocation effect is removal of one initial clear; no search
expansion, allocation budget or deadline change. Cold admission IO remains in
its current worker/warmup paths. Warm latencies and candidate projections must
be compared separately; the key-before-ready failure is not closed by a
key-after-ready PASS. UnknownStart automatic-edit refusal remains an explicit
limit, and model Tied/Abstain failures remain open. Other consumers still see
the same generation on first admission and a new one on explicit replacement.
The existing epoch controller is retained; maintenance removes an unnecessary
transition. Rollback is this small source hunk plus rebuilt verified IME; no
model or desktop state migration.

Proof plan: preserve actual old-binary cold FAIL; run the same client/order
after rebuild with all dependency identities and V90 consumption checked;
retain all seven matrix outcomes. Run focused lib/IME contracts, independent
review, mandatory changed/full release gates and architecture refresh before
installation. Quality dimensions remain conjunctive and no aggregate/per-class
restoration, clean, false-certainty, lattice, package/RSS or latency gate is
waived. This patch claims only initial publication repair, not >95% actionable
restoration or the completion of TD-123. Runtime authority is unchanged until
a separately evidenced installation within the user's existing repair scope.

## TD-123 generic L4 feedback versus current authority: superseded narrow preflight2026-09-08

Measured first refusal on exact candidate c0be96cd plus9 admitted files,12
persisted model/usage/config inputs and5 exact system dictionaries: the supplied
phrase retains and ranks the final target first, L3=720/support/pairwise=true,
posterior0.752,risk0.286. Both ordinary lanes refuse it with
`latent_l4_negative_transition_memory`: signed=-577, state_specific=false,
transition attract/repel counts0. Model-only input parity is proven by consumer
maps, L3 loaded statistics and a separate file-read trace. The strace run's
cleanup failed because the tracer became the parent; its enclosing private
unit terminated all processes. It is read-set evidence, not client acceptance.

A diagnostic ablation removes ALL generic rejected-word/context maps from a
private copied feedback snapshot, preserving every transition map and every
other input. The same five-token client changes from dirty0/2,clean3/3 to
dirty1/2,clean3/3. No production feedback was edited. Causal paths:
`autocorrect-live-ojoasco5/phrase-inputs-candidate/` and
`phrase-generic-feedback-neutral-candidate/`; this does not establish general
quality improvement or authorize dropping learned data.

Source facts: DecisionCore already admits a SuggestOnly candidate with actual
L3 directional pair certification; valid frame-bound lexical admissions form
another independent existing route. Signed-memory readout combines generic
word feedback into a negative signal. Final generic-negative veto protects
operator consensus and verified layout only; it never receives the current
L3 pair certificate or validated lexical admission. Numeric context support
is not that certificate and must remain insufficient. Slice9 ContextCertified
emission is still absent and is not needed to repair this existing route.

Selected design9/10: repair the existing signed-memory-to-transition adapter.
A negative transition requires repel_count > attract_count. Exact state-specific
negative retains its existing veto; actual generic transition fallback retains
the existing -450 calibration. Word/context-only negative continues to influence
ranking/Bayes but cannot manufacture transition evidence. This follows the L4
contract in `l4-causal-transition-memory-plan.md`: absence of transition evidence
is not negative evidence. `add_rejected_fix_sources` intentionally records word
rejection with record_transition=false; no learned data is corrupt merely
because this consumer promoted its scope incorrectly.

Alternative6/10: propagate L3/frame-bound authority to bypass generic veto;
rejected for this patch because it changes handling of genuine nonzero generic
transition evidence and repairs later than the first scope error. Alternative
2/10: erase negative feedback; loses valid history and does not fix its consumer.
No new authority bit, argument, producer, source label, threshold or owner is
needed. SafetyGate, verifier and all admissions remain byte-identical.

Consequence check: bounded lattice, rank scores, positive/negative priors and
model evidence remain unchanged. The corrected negative flag also restores a deterministic Eligible contender
to the existing owner-gravity competition if its action is verified, distance
is no greater and rank is close enough. Thus the selected surface or abstention
may change despite unchanged numeric scores; this needs a separate semantic
competition control and a fixed before/after replay. Candidates still need
existing producer/L3/lexical authority and every later
policy/structural/safety/verifier check. No evidence or equal/positive transition
counts cannot assert negative experience even if generic word prior is negative.
Exact negative dominates unrelated positive word priors as before. False-apply
risk is accidental weakening of real negative transition history, so test
state-specific negatives and genuine generic fallback separately. No new owner,
cache, queue, timer, allocation or package format; CPU/RSS change is one scalar
comparison. Deadlines, publication/apply epochs, delta reload and stale-result
rules stay byte-identical. Learning writers, feedback files and usage snapshots
are unchanged; future updates supply the same separately typed count fields.
IME and daemon share this adapter. Rollback is the source hunk and verified
binary; no data rollback or desktop migration.

Proof plan: semantic regression uses an existing real common-L3 report plus
controlled word-only/exact/generic-transition negative signals. Baseline must
fail the word-only case. Assert target, independently obtained L3 certification,
action proof, final admission and retained negative controls. Numeric support
alone stays insufficient; existing Strict/verifier contracts rerun unchanged.
Then replay the native phrase with ORIGINAL feedback bytes and all other inputs
unchanged, plus original seven-case matrix and affected fixed proof. Reuse L1
quality proof only for unchanged L1 algorithm/package identities, reporting all
aggregate/per-class dimensions and scope separately from DecisionCore admission.
L4 historical negative replay must preserve negative_false_apply=0. Focused
remote checks, independent review, mandatory changed/full/client gates and
architecture refresh remain required before install. Missing targets, isolated
ties, immediate Space and UnknownStart remain separate. Preflight changed no
installed runtime or model authority.

## TD-123 L4 evidence domains: revised consequence check2026-09-08

The narrow adapter revision above is REJECTED, not accepted by its passing
tests. Independent review pass2 is7/10,H0/M1/L0. Its new competition control
(`run-mVVxki`, local `run-5totqn_r`, lib1758 PASS,153.252s) proves that restoring
a word-only-negative deterministic contender to owner gravity can exclude a
real L3-certified target. The target, lattice, verifier and controlled equal
ranks stay fixed. This is a semantic authority regression; no production
false-apply or final competing surface is inferred from that control. No
installation occurred. This starts an explicit revised design, not a third
repair of an allegedly accepted narrow patch.

Selected9/10: distinguish the two existing consumers. Typed transition-negative
evidence requires actual net rejected transition counts, as above. The existing
owner-precedence check separately consumes generic preference rejection: its
predicate must remain exactly the old `exact_negative OR (!exact_positive AND
signed<=-450)`, with exact_positive/negative derived from state specificity and
the respective net attract/repel balance. The initial shorthand here omitted
the exact-positive exemption; independent design review caught this BEFORE
implementation. Frozen baseline `run-v6szovnz/request.json` binds decision.rs
SHA05527a928596828c8a499093802697f20ab4aefdaedd2a271b5c4763110ca3d5,
identical to HEAD source. The full expression above is the required parity
contract. This withdraws a competitor's automatic source precedence;
it neither grants nor refuses that competitor's independent final admission.
The candidate remains in the lattice and may win ordinary admitted ranking.
Generic word/context weights thus preserve their prior ranking/competition
effect while losing only their invalid promotion into hard transition evidence.
Use one named signal projection in CandidateDecisionSignals and the existing
owner-gravity consumer; do not add an owner, stage, type or fallback.

Alternative6/10: remove producer-based owner gravity and rank all independently
admitted candidates. This is a viable broader policy redesign but changes
competition for every L2/L3 candidate, needs a new full calibration comparison
and cannot be attributed to the demonstrated negative-domain bug. Alternative
5/10: require successful final admission of the deterministic rival before it
may suppress another candidate. Direct recursion is invalid; a nonrecursive
two-stage resolution would need a separately specified cycle/tie policy. The
selected route preserves the existing competition predicate for all signal
tuples and bounds this fix to the incorrectly promoted negative witness.

Consequence check: target birth/retention, all numeric ranks/Bayes/word priors,
positive L3/L4/lexical evidence, producer permissions, SafetyGate, edit planning
and verifier stay unchanged. Exact negative and genuine generic transition
negative still refuse the target. Positive or equal transition counts cannot
be relabeled as net rejected experience. Owner competition must match baseline
at the -450 boundary, both specificity states, and negative/equal/positive
count balances. A wrong signal projection at that second consumer could again
exclude the certified target; a real negative mislabeled as word-only could
increase false applies. These are separate semantic controls, followed by a
fixed actual-client before/after comparison with original feedback bytes.

No model/delta format, learning writer, history, config, cache generation,
publication race, deadline, queue, lock, timer or allocation changes. CPU/RSS
effects are bounded scalar comparisons; latency gain is not inferred. Future
model or online updates retain distinct counts and preference scalar and must
not collapse their domains. Both IME and daemon use the shared DecisionCore.
Rollback removes the two signal projections and restores the verified baseline
binary without data conversion. The cold V90 publication fix is independent.
Revised target-preservation invariant RED: remote run-XzEA6D/local
run-2qtegy9g,1758 selected, sole expected word_only_competitor failure,152.968s.

Proof before installation: change the rejected competition expectation into
the required target-preservation invariant and record its RED on the narrow
revision; GREEN must preserve the exact target and verifier receipt under
word-only preference, and retain genuine-negative refusal. Check the old
competition predicate's full piecewise boundary domain independently. Repeat
the original phrase and seven-case actual-IME matrices, plus a frozen set of
existing correction/clean fixtures with aggregate and per-class results; do
not label those existing/training fixtures as unseen heldout quality. Historical
2466 L4 replay inputs are absent; their old result cannot become a fresh PASS.
Relevant existing full contracts, independent review of this revised design,
changed/full release gates, exact client smoke and architecture refresh remain
mandatory. L1 package/algorithm quality dimensions are unchanged and remain a
separate conjunctive contract. No production runtime authority changed here.

Revised implementation review pass1:9/10,H0/M0/L0; focused2206 PASS
(448IME+1758lib),162.798s, remote run-UgV9rU/localrun-c1abdlhw. Runtime source
frozen after review. The exact old precedence expression is preserved, and
the semantic test covers its exact-positive exception as well as the separate
hard-negative domain. Final native selection/false applies are not inferred
from target admission receipts. Fixed client comparison manifest ae1bf941
contains89 existing rows (47dirty/42clean), selected before build by a stated
lowercase Cyrillic/space route;33 excluded rows and every source hash are
recorded. It is not an unseen heldout or full-profile quality proof. The first
16-case baseline pilot stopped at the existing byte-tail JSONL rotation; its
product outcomes remain evidence. The corrected diagnostic parser skips only
the partial first record at the512000byte cap, without extending deadlines or
retrying Space. Runtime code and installed authority did not change for it.

## TD-123 revised L4 candidate: measured build and client comparison2026-09-08

Tested candidate SHA995b609343aa7b3bc9fad628cc80df09ea1b2125753c4d62c42b3b0790d50fb0,
7,728,288 bytes, contains the cold V90 publication fix and revised L4 signal
projection. Exact697 Rust source hashes match both local owning checkout and
remote build mirror; aggregate7024f54c663004a7e05822651a368aa54d6a1baf137dcb768158c1f5927ba7c8.
The independent source projection proof preserves the COMPLETE old owner
precedence predicate, including the exact-positive exemption, for all scalar
inputs. This is predicate parity, not a quality measurement.

All preflight shell/build checks passed; Rust release build2m55s, target
9,397,616,640/12,884,901,888 bytes. The subsequent metadata recorder FAILED:
it incorrectly required the entire generated architecture receipt JSON inside
the optimized IME. That receipt is used only by test callers and LTO removes
it. The failed assertion remains recorded; no rebuild or runtime workaround
was used. candidate-identity.json instead binds every Rust file and labels the
receipt generated_source_architecture_receipt, architecture_receipt_embedded_in_ime=false.
Mandatory source architecture and release gates remain required separately.
Exact build evidence: /home/ubu/.cache/lay/development/run-c1abdlhw/{preflight.json,preflight.log,candidate-identity.json};
remote /home/e/projects/lay-development-runner/run-UgV9rU/.

Actual private IBus/GNU Readline uses original feedback, all9 core roles,
12 persisted inputs,5 system dictionary/ngram inputs and verified loaded
V90/L3 identities. Supplied phrase: dirty0/2 ->1/2; clean3/3 ->3/3. The final
missing-letter token restores; the composite token still abstains. Original
seven-case matrix: dirty2/6 ->2/6,clean1/1 ->1/1. These are key-after-preparation
observations, not physical-keyboard or immediate-Space latency acceptance.

Fixed89 existing fixtures,47dirty/42clean, retain manifest ae1bf941 and original
source-row denominators. Before/after actual outputs:

| Source class | Count | Before correct/preserved | After correct/preserved |
|---|---:|---:|---:|
| restoration_regressions |24dirty|9/24 (37.50%)|9/24 (37.50%)|
| missing_letter |8dirty|5/8 (62.50%)|5/8 (62.50%)|
| repeated_letter |4dirty|0/4 (0%)|0/4 (0%)|
| transposition |5dirty|5/5 (100%)|5/5 (100%)|
| context_fixture |6dirty|0/6 (0%)|0/6 (0%)|
| clean_missing_letter_control |4clean|3/4 (75%)|3/4 (75%)|
| clean_repeated_letter_control |4clean|3/4 (75%)|4/4 (100%)|
| clean_valid_word |34clean|32/34 (94.12%)|32/34 (94.12%)|

Aggregate correct restoration19/47 (40.43%),abstention25/47 (53.19%),wrong
output3/47 (6.38%) are unchanged. Clean preservation38/42 (90.48%) ->39/42
(92.86%);3 clean changes remain. Verdict NO_REGRESSION_OBSERVED is limited to
this ordered native comparison. Learning remains enabled within each of five
fresh shards initialized from identical bytes. Auto-layout state can persist
after Reset; one next-case pre-Space surface differs because prior native
behavior differs, while physical key scripts and final correct surface agree.
This is a closed-loop comparison, not stateless or unseen heldout quality.

Two failed parser pilots remain evidence: each stopped at a partial first
JSON record after existing512000-byte log compaction. Final reader skips only
that partial non-object prefix, checks every complete record and still requires
current engine+epoch within unchanged deadlines. All10 final shards completed
with candidate/input/cleanup parity. No runtime tracing code changed.

Residual composite refusal first lacks independent authority: L1.1 does not
retain the target; L2 adds it; common L3 receives it but exact learned profile
is absent. Existing sentence-recurrence certification requires distance1;
composite distance2 is outside that contract. Missing-letter final L3 support
comes from the current-sentence recurrence certificate, not a new learned
phase profile. No demonstrated L3 class filter or permission to widen this
certificate follows from these observations.

Not tested: all-profile heldout improvement, general >95% per-class quality,
new L1 quality/package/RSS/latency gates, physical input or immediate-Space
restoration, and fresh historical2466 L4 replay (original inputs absent).
L1 algorithm/package/calibration remain unchanged; all conjunctive dimensions
retain their previous scope. No model training, feedback editing, new owner,
SafetyGate/verifier weakening or production installation occurred. Candidate
runtime source remains frozen after review. Mandatory release gates are next.
Exact native evidence: /home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/phrase-domain-after/receipt.json,
/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/matrix-domain-paired-summary.json,
/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/fixed-fixtures-v3-comparison.json and fixed-fixtures-v3-{before,after}-0..4/.

## TD-123 candidate995b6093 installed2026-09-08

Mandatory gates for the exact reviewed candidate passed: changed2684/2684
(2648correctness+36package),414.006s; full2684/2684,651.982s. The13 actual
private client cases passed (first-word US1/RU1,manual3,lifecycle3,restoration5),
with exact candidate/dependency/driver identities and completed cleanup. Four
physical_double_shift_owner_ tests passed; this is code evidence, not a new
physical-keyboard observation. Before/after697-file Rust identity is unchanged.
Target9,397,526,528/12,884,901,888 bytes. Source architecture gate passed42.929s.
Exact receipts: /home/ubu/.cache/lay/development/run-c1abdlhw/{gates.json,changed-test-summary.json,full-test-summary.json,
physical-owner-test-identities.json,client-proof-validation.json,budget-after.log}.

Installed the SAME tested file at 2026-09-08T12:28:55.649410+00:00; file and running
/proc executable SHA995b609343aa7b3bc9fad628cc80df09ea1b2125753c4d62c42b3b0790d50fb0.
IME PID3983217/start49187106,unitlay-ime-release-995b6093.service. The previous
6dc95148 binary is saved at /home/ubu/.local/state/lay/release-backups/ime-autocorrect-20260908-71oibez0/lay-ibus-engine.
Selected Lay RU and the two configured input sources were restored/verified.
IBus4715,daemon3757261,L1.1service271400 retained exact process start identities.
Only the IME process was replaced. D-Bus Ping proves liveness only (no focus
at installation). Physical normal-tempo phrase confirmation was requested
and remains PENDING. Exact installation receipt: /home/ubu/.cache/lay/development/run-c1abdlhw/installation.json.

Installed runtime authority changed=true, bounded to cold V90 publication
and corrected L4 transition-negative interpretation in the IME. Package/model
permissions, learning data, SafetyGate and verifier remain unchanged; daemon
binary is unchanged. The private89-fixture result remains19/47 dirty correct
and39/42 clean preserved, with3 wrong dirty outputs and3 clean changes.
General Wave quality, immediate-Space behavior, all safety profiles and the
historical combined two-field/profile failure remain OPEN. This installation
does not close TD-123 or establish release1.0.67 quality/commit/push acceptance.

Read-only follow-up: current compiler::calibrate_l11 samples every fifth
resolved surface against its first anchor and takes a geometry quantile.
The admitted package still carries geometry0/positive990/backward663. Current
source inspection does not prove the historical compiler inputs that created
those values. Any recalibration must separately measure target retention,
readout authority, false certainty, clean preservation, per-class quality,
package/RSS and latency on frozen independent inputs; blindly increasing the
geometry limit is not an admitted repair. No further runtime/model edit made.


## Sep8 continuation after cleanup — timed native input diagnosis

The user requested that autocorrection continue after the new-branch cleanup.
The cleanup is complete at `ce064fa51a4d42c4b85e0e9e2c33710cebd5366c`; all697
Rust sources match the installed995b candidate. The current product baseline
is unchanged: fixed existing native fixtures19/47 dirty correct and39/42 clean
preserved; physical normal-tempo confirmation is pending.

The next diagnostic changes only the private input schedule. Existing
`autocorrect-phrase-domain-runner.py` and the real private IBus/GNUReadline
consumer retain candidate995b, nine dependency roles, additional persisted
L3/feedback inputs, exact system dictionaries, fixture text/order, startup
control, cleanup and resource envelopes. Three fresh runs use requested key
cadences0,80,150ms. They observe actual start/RPC timings and send Space at the
same scripted cadence as other characters, without awaiting a prepared
correction or retrying a failed token. Initial `post-exact-ready` startup is
still separately labelled; this does not prove immediate cold startup or
physical GNOME/Kitty input. Frozen initial persisted inputs are the earlier
private snapshot, not a clone of production's current in-memory learning.

Alternatives: (A) repeat only the after-prepared control, low diagnostic value
because it excludes arrival-before-ready; (B) fixed timed native input plus
existing trace identity/outcome, chosen9/10 for distinguishing remaining
readiness from completed refusal; (C) change runtime wait/deadline or widen
model authority before measurement, rejected because the mechanism is not
established. Timing is a scenario input, not causal proof of a race: any later
concurrent repair still requires controlled event-order proof in real owners.

Proof denominators remain separate: five phrase words/run, dirty2,clean3;
UnknownStart versus known-tail; field-request/producer/publication state;
Space ready/not-ready/stale/no-apply; actual visible correct/unchanged/wrong
outputs. An executed diagnostic is not product PASS or unseen heldout quality.
The baseline after-prepared phrase result is dirty1/2,clean3/3. No expected
output, old failed receipt or fixed89 denominator is changed.

Consequence boundary: no runtime/candidate lattice, ranking, verifier,
SafetyGate, model/package/calibration, reload, feedback rule or ownership
change. Learning occurs only within each fresh private process from the same
frozen inputs. No new production timer/cache/queue/fallback or stale-result
route. Script cadence can mask or expose timing sensitivity, so report actual
intervals and retain0ms and150ms outcomes together; do not promote a slower
success into a deadline or authority relaxation. CPU/RSS remain the existing
private CPU200%/1536M/90s envelope inside the remote dedicated20CPU guard.
Rollback is deletion of the private diagnostic output; installed IME and
production input sources remain unchanged. Independent source review examines
the existing readiness/publication/refusal evidence before any runtime edit.

Exact private outputs will be `autocorrect-ojoasco5/phrase-cadence-{0,80,150}/`
on the remote worker; copied receipts belong under the existing private
`autocorrect-live-ojoasco5` cache. No runtime code has been edited in this step.

Measured results: all three fresh native runs completed all five words and
reaped their private processes. Dirty restoration was 0/2,0/2,1/2 at requested
0,80,150ms respectively; clean preservation was 3/3 in each run. Actual median
key intervals were 5,304/81,420/151,890us. First-word UnknownStart remains
distinct from the four known-tail frames. Candidate, nine dependencies,
private configuration, driver, launcher, additional persisted inputs and
resource envelope identities matched across all runs. The full fixed89
denominator was not rerun: runtime and model bytes have not changed.

The composite frame completed at both80ms and150ms and returned rank NoApply;
readiness does not explain that refusal. For the final missing-letter frame,
80ms Space returned NotReady after3,525us; its later correction:superseded
is consistent with the lookup retiring that generation. At150ms the same
frame prepared and applied, with8us lookup wait. Source order takes the
correction lease BEFORE cancelling display work; display cancellation is not
evidence of a correction-generation race. Existing traces lack enqueue/start
timestamps, so queue delay and the exact pending state at lookup remain
unmeasured. Timing alone is not a controlled concurrency proof.

Final-frame core timing, in microseconds:

| Cadence | Full core | Candidates | Deterministic | L1.1 + V90 | Outer decision | Inner decision |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
|80ms|83,349|54,833|41,655|12,751|28,514|19,442|
|150ms|85,965|59,076|51,418|7,322|26,886|18,926|

These are nested timers, not additive columns. Deterministic typing rules
took27,362/35,650us and composite generation13,865/14,554us. Substitution took
16,060/22,431us within typing. Outer decision includes preparation of legacy
L2 peak context before the inner DecisionCore timer. These diagnostic runs
enabled verbose field/decision output (roughly0.8MiB stderr at80/150ms), so
they cannot establish ordinary-runtime latency or production tail percentiles.

Next discriminating control is exactly one fresh run per0/80/150ms with the
five optional stderr debug/timing variables absent. Keep native bounded
trace, identical driver/startup/input/material and all failed outcomes. This
changes observation cost only; candidate authority, deadlines, learning and
production runtime are unchanged. No retry-until-green or performance PASS
is permitted. Outputs: `phrase-cadence-quiet-{0,80,150}/` under the same cache
and remote run root. Independent review is inspecting semantic-preserving
ways to remove repeated candidate work before any production implementation.

Exact evidence: `/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/cadence-summary-v1.json`
and `phrase-cadence-{0,80,150}/{receipt.json,run-metadata.json,ibus-engine-trace.jsonl,engine.stderr}`.
The initial stdin batch transport executed no scenario; its retained record
is `cadence-batch-stdin-not-executed.json`, not a PASS or product failure.
Runtime authority changed=false for this diagnostic milestone. Physical
keyboard, unseen quality, all safety profiles and cold-start acceptance were
not tested. The95%-per-damage-class conjunctive L1 gate remains unchanged.

Quiet control results: the first `phrase-cadence-quiet-*` attempt removed the
three timing flags but the derived driver re-enabled the two verbose flags.
All15 executed outcomes are retained; `cadence-quiet-control-v1-rejected.json`
labels that observation control NOT_ESTABLISHED. The corrected private driver
removes only those two assignments; input text, ordering and schedules match.
`phrase-cadence-quiet-v2-{0,80,150}/engine-environment.json` proves all five
flags absent in the actual executable. Stderr fell to344/509/509bytes; all15
words and cleanup completed. Outcomes remain dirty0/2,0/2,1/2 and clean3/3
each. Exact evidence is `cadence-quiet-v2-summary.json` in the same cache.

At quiet80ms, final-frame core time was76,007us, yet Space still returned
NotReady after3,654us and the result later reported superseded. At quiet150ms
core82,480us prepared and applied. Thus stderr cost does not explain the
failure; core elapsed alone does not measure time from scheduling to ready.
Queue/start and publication intervals are the first remaining timing gap.
The canonical `productive_v90_us` also includes grounding/materialization;
a ready_hit does not mean a second V90 field production. No fixed89 or
per-class quality result changed in these source-identical diagnostics.

## TD-123 preflight: observe the existing correction worker queue

Selected next step is bounded observation, before a performance patch. Add
an optional enqueue Instant to existing DesiredWork only when tracing is on.
At the existing worker's evaluation and publication boundaries, record queue,
evaluation and publication intervals in one metadata event with the existing
engine path, epoch and generation. Keep this outside the locked publication
section and omit all token/context text. No new worker, owner, queue, cache,
deadline, polling or RPC. The timestamp is not consumed by any decision.

Alternatives: (A) continue interpreting core time as end-to-end, rejected by
quiet80ms; (B) add these missing intervals to the existing bounded opt-in trace,
chosen9/10 for discriminating the next mechanism; (C) optimize exact bounded
geometry first, viable for semantic parity but its fraction of total delay is
unmeasured; (D) share raw lexical lanes before separate limit-dependent
reserves, viable and potentially larger work removal but much broader risk.
Neither C nor D is authorized as an implementation conclusion by timing alone.

Consequence check: candidate/lattice/rank/authority, calibration, packages,
reload and learning/feedback expressions are unchanged. Store no material or
outcome in the new timestamp; stale generations keep the existing retirement
and publication predicates. Opt-out work carries None. Opt-in work adds one
Instant to the existing bounded desired slot and a constant-size metadata row
per evaluated generation; the current bounded writer/rotation stays owner.
Small clock/formatting cost can perturb timing, so this is diagnosis rather
than a performance claim. Do not log under the worker state lock. Allocation
is only existing enabled trace formatting, with no retained per-token history.
IME and daemon protocols and single Double Shift owner remain unchanged.
Rollback removes timestamp and event together; no data migration or new
consumer requirement. Future package/online-update timing remains observable
without allowing the observation to affect stale-result acceptance.

Verification: existing IME worker identity/material/publication/timeout and
trace contracts, plus a metadata timing-boundary assertion without user text;
build a private candidate through the remote guarded development route, then
repeat the fixed three quiet native schedules once. Record source/binary and
input parity; no install or general quality promotion in this diagnostic
step. Any later concurrent repair still requires controlled event-order proof
through the actual owners. Independent review precedes production edits.

Design review9/10,H0/M0/L1. Resolved the low timing-description issue:
publication_us ends at the observation immediately AFTER dropping the state
lock; it can include descheduling and is not the exact instant the lease
became available. queue_us excludes admission and inline exact preparation.
Check trace configuration before acquiring worker state. Off-to-on requests
without an enqueue timestamp emit no row; on-to-off may suppress the row.
Missing rows and superseded-before-dequeue work mean UNKNOWN, never zero.
Formatting may slightly delay the next dequeue. The pure metadata test uses
7/11/13us intervals,31us total, None and superseded generation identity. Real
rounded component sums may differ from total by at most2us.

Implementation review pass1:9/10,H0/M0/L0. First explicit IME development
check stopped at rustfmt only (2.619s), before test compilation/execution:
`/home/ubu/.cache/lay/development/run-c5qnf70s/RESULT.json`, remote run-0249bu.
Applied exactly its two expression-wrapping changes; no semantic repair.
The next fresh focused check retains this failed formatting receipt.

Queue observation completed. Focused449/449 passed in14.390s; the new
`trace::tests::correction_prefetch_trace_separates_intervals_without_text`
is present in discovered, selected and executed manifests. Fresh-context
independent implementation review9/10,H0/M0/L0. Architecture refresh41.201s
PASS; private release build59.023s PASS. Candidate
`9f8e99e0fa3836485a785344e06b02d816812b0c936b65919f4f224d32453361`
is at remote `/home/e/projects/lay-development-runner/run-YMQqMd/lay-ibus-engine-timing`.
Source697 hashes match the reviewed snapshot. This file is NOT installed.
Receipts: `/home/ubu/.cache/lay/development/run-by567ovy/queue-candidate-identity.json`,
`queue-focused-summary.json`, `queue-focused-discovered.json` and
`queue-graph-copy.json`; source architecture receipt was copied and verified.

The three quiet native schedules completed15/15 words, private cleanup and
candidate/input identity checks. Outcomes remain dirty0/2,0/2,1/2 and clean3/3
each at0/80/150ms. New timing events7/27/28 match existing correction records
by engine path, epoch, generation and outcome. Exact evidence:
`/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/cadence-queue-summary-v1.json`
and `phrase-cadence-queue-{0,80,150}/`.

At80ms the previous prefix (epoch35,generation6) computed for127,614us and
finished superseded. The final frame (epoch36,generation7) waited46,778us,
computed73,615us, observed publication in<1us: total120,394us. Space returned
NotReady after3,535us. At150ms the final queue was76us, computation94,583us,
publication2us,total94,662us; the lease applied. The completed composite frame
remains rank NoApply at both cadences. These source-bound observations prove
that existing serial evaluation of the older request contributes the queue
delay. They do not prove a stale-result authority race or guarantee current
computation below80ms even after removing the queue. No controlled cancellation
or scheduling change was tested. Opt-in overhead, trace on/off transitions,
writer saturation, production physical input and release gates were not tested.

Next repair alternatives (design scores, not measured speedups): (A) cancel
already superseded work using the existing request generation,8/10; removes
obsolete work but94.6ms current computation gives no guarantee of meeting80ms
through cancellation alone; its sufficiency has not been tested; (B) remove
repeated exact lexical/context computation while preserving
all candidate surfaces, ordering, scores, proof and authority,9/10 and the
recommended first direction; (C) add concurrent workers,4/10 because it adds
resource/synchronization cost and may worsen the existing CPU envelope.
The specific operation removed by B still needs measured attribution before
production implementation. The obsolete prefix spends32,565us in L3 and
59,498us in inner DecisionCore; these are nested parts of127,550us core.
Candidate preparation, shared-context recomputation and geometry are the next
bounded source inspection, not permission to prune the lattice or bypass L3.
Runtime authority changed=false for all queue diagnosis work.

### Sep 8 TD123: repeated L3 pair features — preflight

Measured boundary: the quiet native 80ms run spends46.8ms queued behind a
superseded127.6ms calculation; L3 inside that calculation takes32.6ms.
Source inspection finds `score_candidates_with_mode_and_pair_views` calculating
the same token hash and L2/morphology signature for both candidates on every
anchored pair, then again while building the pair lattice. The unary scene is
already shared and is not the proposed repair. Attribution to the repeated
signature calculation is still a hypothesis until the operation count and
private native before/after timing are measured.

Alternatives, design scores rather than measured speedups:

- Baseline: preserve repeated reads; correct semantics but quadratic expensive
  signature work when no anchored pair is found.
- A,9/10,recommended: remember each exact pair feature only for this scoring
  invocation, indexed by the unchanged candidate position and calculated on
  first use. Preserve the existing pair search and short circuit. Prefer a
  local optional vector and ordinary mutable closure if that is sufficient;
  no persistent cache or new owner is justified.
- B,8/10: eagerly calculate the same feature for every candidate once. This is
  simpler but can add signature work to early-hit/disabled/empty pair routes.
- C,4/10: persist a surface-feature cache across calls. Broader identity,
  invalidation and memory policy would be required without evidence of need.

Consequence analysis before production edits: retain every candidate and every
pair, the existing MAX_PAIR_CANDIDATES boundary, stable sorting, all scores,
hard/soft evidence and authority predicates. No spelling literal or fixture
identifier becomes runtime logic. The exact feature depends on the candidate,
the borrowed package's signature schema, and the existing immutable L2 surface
memory; verify that lifetime before changing reads. Memo entries die on return:
package/delta reload and feedback between calls must get new calculations.
No shared lock, worker, generation, deadline, timer, cancellation or fallback
is introduced. Publication and stale-result rejection remain the same owners.
Expected CPU benefit is fewer identical calculations, not a promised Space
deadline. Temporary allocation grows linearly with the input frontier; two
u64 values plus an Option tag are about24bytes per candidate on this64-bit
target (about1.5KiB for64); do not cap the frontier to bound this allocation.
Initialize storage only on first pair use if possible, so skipped routes keep
their allocation behavior. Check early anchor hits and all ablation modes.
Future mutable L2 readout would invalidate the constancy argument and require
an explicit snapshot; no current reload mechanism is to be changed here.
IME, daemon, learning and serialized package protocols remain unchanged.
Rollback removes only this call-local reuse and its diagnostic-only test hook;
there is no data migration or compatibility route to retain.

Verification order: independent design review; test-only scoped thread-local
signature counter around the real scoring entrypoint; fixed generated Cyrillic
cohorts with two exact views and no matching profiles. Collect all cohort counts
before asserting a linear ceiling, so the unoptimized code must fail for the
actual repeated-work defect. Record complete readouts and candidate order for
before/after equality, all ten modes, empty/early-return and duplicate/order
contracts; existing pair tie/cycle/authority tests remain required. No test-only
counter is compiled into a release. Then the smallest production change, the
explicit library/IME checks, private release and matching three quiet native
schedules, and the entire fixed89 client fixture set including per-class and
clean preservation. Mandatory changed/full release gates and fresh-context
review precede installation. Counts/parity alone do not prove restoration
quality or physical input acceptance. Retain failed measurements and report
their scope separately. Owning task remains TD123, not general tech debt.

Design review completed9/10,H0/M0/L0. Confirmed immutable L2
DEFAULT_MEMORY OnceLock and the borrowed L3 package lifetime. Selected ordinary
FnMut with a lazy optional vector, preserving lookups inside the old inner
pair loop and preserving anchored probes in NoPhase/NoPairwise modes.
First check run-vhob4qvs/run-HzTf6s stopped at rustfmt (2.675s); fixed only
wrapping. Baseline run-g0ojw_ge/run-bkIYLX selected/executed1760 library tests:
1759 passed, exactly the new scaling test failed (154.642s total). Actual
signature calls for N=0,1,8,16,32,64 were0,3,136,528,2080,8256, matching
2N^2+N for N>=2. This is a real unoptimized failure, not a injected violation.
The second new test passed all fields across40 schema/mode combinations and
three pair banks (compiled,early exact hit,cycle), reversed input and duplicates.
An exact two-test observation using the existing discovery and sandbox runners
with --show-output recorded all126 readout digests in2.361s; expected RED, not
quality PASS. Receipt: remote
`/home/e/projects/lay-development-runner/run-bkIYLX/pair-memo-exact-baseline/receipt.json`;
focused summary: local run-g0ojw_ge/pair-memo-baseline-summary.json.
These results authorize the selected minimal implementation under the written
preflight; they do not yet establish native latency or restoration improvement.

Implementation result: focused library1760 + IME449 =2209/2209 PASS in161.114s
(`run-xwqqoymn`, remote `run-Nvm8AU`). Exact two-test observation1.031s:
all126 baseline/candidate readout digests match; calls for N=0,1,8,16,32,64
are now0,3,24,48,96,192. Fresh-context code review9/10,H0/M0/L0,pass1/2,
no repair required. These are component parity and operation-count results.
Receipts: local run-xwqqoymn `pair-memo-focused-summary.json`,
`pair-memo-exact-candidate.json`, `pair-memo-component-comparison.json`;
baseline exact receipt copied to run-g0ojw_ge/pair-memo-exact-baseline.json.

Private candidate3707394809f35920728415fa008db9c97d375099d833534a01409eeeb7eecc8c
is remote run-Nvm8AU/lay-ibus-engine-pair-memo. Architecture41.373s and
release176.122s PASS; all697 Rust file hashes match the reviewed source, Cargo
budget PASS. Six generated graph artifacts were copied and hash verified.
Identity/command logs and `pair-memo-graph-copy.json` are in local run-xwqqoymn.
The installed engine remains995b6093; no production installation occurred.

Native outcome: all15 words executed, clean3/3 at each schedule; dirty0/2 at0ms,
0/2 at80ms and1/2 at150ms. Optional stderr diagnostics absent, driver/config,
nine dependencies, additional inputs and resource envelopes match the earlier
queue candidate. Private processes reaped. Every new queue timing row matches
the correction row's engine/epoch/generation/outcome; no trace truncation.
At80ms old-prefix L3 fell32,565→2,193us and old evaluation127,614→100,314us.
The final request queued20,680us and evaluated71,290us,total91,971us,still
NotReady at Space. At150ms final queue30us,evaluation65,890us,total65,922us;
the correction applied. These single schedules show reduced work but do NOT
close the80ms deadline or establish production percentiles/physical typing.
Private evidence: autocorrect-live-ojoasco5/cadence-pair-memo-summary-v1.json
and phrase-cadence-pair-memo-{0,80,150}/.

Whole fixed89 verdict: REJECT_CANDIDATE_FOR_PROMOTION. All five shards completed
with matching initial inputs, candidate identity and cleanup. Dirty results
remain19/47 correct(40.43%),25 abstentions,3 wrong outputs. Clean preservation
falls39/42(92.86%)→36/42(85.71%). Every source class is retained below; these are
existing regression/training fixtures, not an unseen heldout acceptance proof.

| Fixed source/error class | Before | Candidate |
| --- | --- | --- |
| missing_letter, restored | 5/8,62.50% | 5/8,62.50% |
| repeated_letter, restored | 0/4,0% | 0/4,0% |
| transposition, restored | 5/5,100% | 5/5,100% |
| restoration_regressions, restored | 9/24,37.50% | 9/24,37.50% |
| context_fixture, restored | 0/6,0% | 0/6,0% |
| clean_missing_letter_control, preserved | 3/4,75% | 3/4,75% |
| clean_repeated_letter_control, preserved | 4/4,100% | 1/4,25% |
| clean_valid_word, preserved | 32/34,94.12% | 32/34,94.12% |

The three newly changed clean strings are exact keyboard-layout projections.
All are failures, even though these small fixtures are not ordinary words.
Initial read-only review found baseline L1.1 query failures and smaller candidate
cohorts in two of these cases; baseline preservation therefore is not evidence
of a settled full-field refusal. Do not infer a ranking change from output
differences before comparing actually admitted fields. The third case already
differs before the final Space, so its prior layout/context transition must be
traced separately. Further performance/cancellation code is paused while the
shared false-authority boundary is identified. Runtime production authority
changed=false; the private candidate remains unaccepted and is not installed.

Correct comparison: private
`autocorrect-live-ojoasco5/fixed-fixtures-v4-pair-memo-comparison-v2.json` and
`fixed-fixtures-v4-pair-memo-{0..4}/`. The first local comparison incorrectly
read this driver's `observed` fixture-input field as output; it is sealed as
`fixed-fixtures-v4-pair-memo-comparison.rejected-input-field.json` with
REJECTED_ANALYZER. V2 reads actual `after.visible` and independently recomputes
and checks all89 recorded verdicts on both candidates. No scenario was retried
or discarded to obtain this result.

### Sep 8 TD123: shared short-layout authority rule — preflight

First shared defect: deterministic English autoswitch already rejects targets
with at most3 ASCII letters unless the existing technical-word dictionary gives
independent support. ProductiveL2 Layout proposals use a broader known_en test
and then receive live authority solely from their origin, even when the field
is Tied/Abstain. Exact physical projection proves how to change the text, not
that a short dictionary abbreviation was the intended language. The frozen
English dictionaries contain BB/DD/dd; lowercasing is confirmed, not guessed.
These are valid lexical centers and must not be deleted from the dictionary.
The old baseline's query failures hid two false applications; the third clean
change was a retained decoder layout after an earlier apply. Full source/trace,
dictionary and178 actual-status checks: private
fixed89-v3-v4-layout-authority-evidence.json, SHA256
2b84d4dfe65a5e82539e8380c9ff844bc46eb3fb78a0bde957b5fb71ed5e78df.

Options: A,9/10,recommended, extract the existing short-target predicate and
reuse it at the current productive live-authority override as well as in the
deterministic generator. B,7/10, put a stronger guard in common proposal
admission; viable only with explicit grounded-winner/context exceptions and
wider proof because a hard KeepOriginal can erase valid evidence. C,4/10,
filter dictionary abbreviations; loses legitimate centers and still fails to
distinguish valid lowercase short words. Scores are design judgments.

Consequences before code: A adds no threshold, word/suffix/source-id exception,
owner, candidate-generation path, cache, deadline, queue or calibration change.
Preserve the exact existing ASCII-alphabetic count and case-normalized technical
lookup, including digits/hyphens not increasing the count. Apply the ordinary
layout restriction only to ASCII targets. A protected grounded L1.1 Winner or
a matching actual L2 Winner retains authority before this ordinary allowance;
the generic short-target uncertainty cannot downgrade either. Keep all lattice
surfaces and scores. Uncertain nontechnical short Layout candidates become
SuggestOnly through the existing override, retaining independent positive L3
pair/exact L4 and frame-bound admission routes. No change to SafetyGate,
edit-plan validation, verifier, ClosedExact or daemon/manual Double Shift.

What may worsen: fewer immediate short-word autoswitches without context;
alias merging must not restore authority from a provider that bypasses the
same rule. Check duplicate ordinary/canonical lanes and unrelated winners.
Existing legitimate technical short words and longer English targets must keep
their route; Cyrillic targets are outside this extracted English predicate.
No package/delta identity changes; a future technical dictionary update affects
both consumers equally. No new feedback event or learning condition, though
preventing a wrong apply also prevents its downstream layout/learning cascade.
CPU adds one bounded target scan/technical lookup only to ordinary Layout
authority; reuse the existing lowercased target where possible, with no extra
correction-generator call. No retained allocation/RSS or synchronization.
Stale-result publication and IME/daemon protocol ownership stay unchanged.
Rollback restores the two call sites and removes the extracted predicate;
there is no migration or permanent compatibility wrapper.

Proof before promotion: baseline RED through real materialize_live_candidates,
retain surfaces/provenance and verifier proof, reject automatic apply without
positive context, preserve explicit grounded/actual winners and independent
context support. Cover actual Tied/Abstain and overflow; Unavailable is not
produced by live_authority and needs only the existing helper's enum coverage.
Use general short dictionary/technical/long/Cyrillic controls, not a runtime
fixture list. Then the minimal patch, affected library/IME tests, fresh-context
review, all fixed89 actual outputs/per-class/clean results and real native
layout/manual controls. Repeat timing only after quality regression is resolved.
No full release, model-quality or physical acceptance follows from helper PASS.
Independent fresh-context design review: 9/10, H0/M0/L0, first pass. The ordinary
short-target restriction also covers Winner(other surface); matching Winner
and protected grounded Winner take precedence. Unavailable is covered only at
the existing helper because live_authority does not return it. No production
authority code changed at this preflight stage.

RED proved the actual old defect: remote run-R64JF0, local
`/home/ubu/.cache/lay/development/run-djbc5viq/short-layout-baseline-summary.json`
and `short-layout-baseline-lib.log`. Exactly1762 selected/executed:1760 passed,
only the two new authority regressions failed. The materialization test reached
the real DecisionCore: `ии → bb`, L2 Abstain, valid physical edit proof, no L3
pair certificate or exact-positive L4, selected_index=0 and a transition receipt.
The helper separately exposed unconditional Layout authority for Unavailable.
Remote check153.806s, including discovery/compilation and existing isolated tests.

The private patch now extracts `english_layout_target_requires_context` and
reuses it in deterministic generation and productive ordinary Layout authority.
It removes the two superseded local length predicates; explicit winners still
precede the ordinary rule. Tests cover30 real materialization combinations and
the helper's case/ASCII-letter-count/Unavailable controls. Full-field verifier,
L3-pair/L4/frame-bound contracts are included in the affected library check.
Actual results follow; production authority changed=false.

### Sep 8 TD123: short-layout implementation and whole-proof result

GREEN2211/2211 (lib1762,IME449), no failures,160.673s remote check. Exact
baseline RED above and fresh-context implementation review9/10,H0/M0/L0,
pass1, confirm the authority distinction rather than a fixture-only workaround.
All30 real materialization combinations pass; existing L3 pair, exact-positive
L4 and frame-bound admission/alias contracts pass in the same library run.
Canonical manifest drift explicitly reports5 added tests; it was not rewritten.

Private candidatefa15601fe2289adcd168e553a0b8ef1afa6d720179caf32bddf376fbc331fdf4:
architecture41.343s,release176.313s,budget PASS,total217.925s. All697 Rust files
match the reviewed local tree. No runtime install, package/delta/config or
daemon/manual source change. Build and focused/native receipts are in
`/home/ubu/.cache/lay/development/run-xqb_96ka/`, notably
`short-layout-candidate-identity.json`, `short-layout-focused-summary.json`,
`short-layout-native-controls.json` and its five client directories.
The unchanged versioned native driver passes13/13: US first-word1,RU first-word1,
manual3,lifecycle3,restoration5. Every private daemon was reaped and no candidate
process remained. These are private IBus/Readline cases, not physical key proof.

Whole fixed89: all outputs match installed995b's frozen baseline. Compared with
rejected37073948, only the three new clean errors disappear, including the
retained-US-layout cascade. Dirty19/47=40.43%,25 abstentions,3 wrong; clean
39/42=92.86%,3 changed. Per-source classes on the new candidate:

| Class | Correct/total | Percent |
|---|---:|---:|
|missing_letter|5/8|62.50%|
|repeated_letter|0/4|0.00%|
|transposition|5/5|100.00%|
|restoration_regressions|9/24|37.50%|
|context_fixture|0/6|0.00%|
|clean_missing_letter_control|3/4|75.00%|
|clean_repeated_letter_control|4/4|100.00%|
|clean_valid_word|32/34|94.12%|

Every class equals995b; no aggregate or per-case regression in this scope.
Actual `after.visible` independently recomputes267 statuses across995b,
37073948 andfa15601f. Driver, dependency/content hashes, profile, initial learned
inputs, resource envelope and loaded L3 content/counts match. The initial
comparison stopped on different copied-file manifest_stamp values; normalization
excludes only that mtime/length-derived field, retaining content SHA checks.
No case or product run was retried. Exact comparison:
`/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/fixed-fixtures-v5-short-layout-comparison.json`
and five `fixed-fixtures-v5-short-layout-{0..4}/` directories.

Verdict: the demonstrated short-layout authority bypass is repaired and the
previous whole-proof clean regression is resolved. Fixed89 is an existing,
experimental, ordered, key-after-preparation set with learning inside each
shard; it does not establish heldout/general restoration quality, all safety
profiles, physical input, the conjunctive L1 gate or normal-tempo readiness.
Resume timing work now; three unchanged quiet cadence scenarios onfa15601f
are running before any further performance change. TD123 remains ACTIVE.

Quiet fa15601f result: dirty0/2,1/2,1/2 at0/80/150ms; clean3/3 each,15/15 words
executed and private cleanup complete. At80ms the previous prefix evaluated
in73,473us and finished before the next request; the final request queued29us,
evaluated62,759us,total62,792us and applied. At150ms its total was84,590us.
This single80ms success follows an earlier37073948 miss at91,971us and cannot
establish timing stability or attribute the difference to the authority patch.
Exact fresh inputs/driver: private `phrase-cadence-short-layout-{0,80,150}/`.

Next timing preflight, no production code: exactly8 paired fresh80ms runs of
queue baseline9f8e99e0 and currentfa15601f. Reuse the identical five-word quiet
driver, frozen dependencies/config/initial learning/dictionaries and two-client
maximum under the same guarded envelope. Fixed denominator:40 words per
candidate (24 clean,8 missing-letter,8 composite); no retries or discarded
failures. Verify real input intervals, candidate/input identities, all output
statuses, queue/evaluation/publication and cleanup. A bounded readiness result
requires8/8 missing-letter restores and24/24 clean preservation on the new
candidate; report composite refusal separately. Even that result is limited to
this native matrix and does not replace physical or general-quality acceptance.

### Sep 8 TD123: fixed eight-pair readiness result

All16 predefined fresh native runs completed in47.484s without retries or
discarded cases. Protocol completion is PASS; the current candidate's predefined
readiness gate is FAIL. Each candidate executed40 words:24 clean controls,
eight repeats of the same missing-letter word and eight composite refusals.

| Native80ms denominator | Baseline9f8e99e0 | Currentfa15601f |
|---|---:|---:|
| Missing-letter correction delivered by Space |0/8|7/8|
| Composite correction, separately ready-NoApply |0/8|0/8|
| Clean words preserved |24/24|24/24|

This7/8 measures timely delivery on repeated identical input, not recognition
accuracy on eight independent words. Final-frame total median108,251.5us became
74,705.5us; current totals65,555..102,853us. In current iteration2 the final
frame queued22,262us, evaluated80,589us and published in1us; trace total102,853us
includes clock-boundary rounding. Space lease was NotReady after3,534us; the
late result was superseded. The same calculation reports L1.1 readout2,533us,
V90 readout7,289us and DecisionCore10,886us (L3's1,716us is inside DecisionCore).
Do not infer what the discarded final decision would have selected.

Actual press intervals are comparable: baseline77,215..86,554us, median81,586.5;
current77,378..85,619us, median81,443. Driver/config/content identities, initial
learning, resource envelopes and all80 output statuses verified. All five
optional stderr diagnostic flags absent. Every private process was reaped.
Exact receipt:
`/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/cadence-stability-summary-v1.json`;
raw evidence in `phrase-cadence-stability-{queue,short-layout}-{0..7}/`, including
each receipt, actual-input, trace, run metadata and actual engine environment.

Architecture implication: FullField live IME still runs deterministic candidate
generation, canonical Wave readout and common DecisionCore in sequence. A
crystallized lexical package does not remove other work or the publication
deadline. This timing experiment neither proves a crystallizer quality failure
nor establishes its quality. Existing fixed89 per-class/clean results above
remain the latest separate output comparison; the conjunctive L1 contract,
physical keyboard acceptance and general heldout quality are not tested here.
Installed runtime authority changed=false; no model/package/config change.
Currentfa15601f is not accepted for installation. Continue measured latency
repair, then the separate ready-result authority path; TD123 remains ACTIVE.

Next diagnostic preflight: one fresh five-word80ms run of unchangedfa15601f,
same frozen quiet driver/inputs/envelope, enabling only the existing
LAY_DETERMINISTIC_CANDIDATE_TIMING flag. This attributes rule costs; stderr
instrumentation makes it ineligible as quiet timing acceptance. No source,
cache, worker, candidate-generation or authority change. Existing near-surface
cache keys include query limit, and raw lane reserves depend on that limit;
cross-limit reuse must not be implemented by truncating a larger final result.

Rule-profile result: all five words executed, three clean controls preserved,
missing-letter applied, composite refused; these instrumented outputs are not
a replacement for the failed quiet gate. The last two deterministic batches
took37,027/37,190us; their substitution rules19,016/11,561us and composite
steps9,700/13,572us. Ten frozen input metadata fields equal the quiet baseline;
only the requested existing stderr flag is present, cleanup complete. Exact
local evidence: private `phrase-cadence-rule-profile/`, including
`rule-cost-summary.json`, raw stderr, trace, receipt and engine environment.
The stderr records lack frame identities, so these are last-batch costs, not
independently joined per-frame subspans.

Next bounded diagnostic: separate near-surface retrieval from substitution
filter/rank under that same existing opt-in flag. Options: direct two-span
measurement9/10; infer from rule totals4/10; general profiler6/10 because the
available release is stripped and remote perf access restricted. Scores are
diagnostic judgments. Use a private instrumented build, one identical80ms run,
then remove this temporary attribution code before implementing a repair.
No candidate order/limits/predicates or model/learning changes; this private
experiment cannot authorize runtime promotion or quiet timing acceptance.

Substitution-cost diagnostic completed. Privatebbcff164f195dcc78978adf6e23eb47a1dab879d4188f00b8952c340cc12a12c
has697 Rust source hashes verified; IME check PASS31.316s, architecture41.125s,
release176.120s, Cargo budget PASS. The one native80ms diagnostic executed all
five words: clean3/3, dirty0/2; cleanup and ten frozen input metadata fields
verified. Nineteen cost records separate retrieval from filter/rank. The last
two autocorrect calls spend27,607/20,621us retrieving64 near surfaces versus
252/410us filtering/ranking them. Their same-limit proposal-only calls retrieve
the cached surfaces in6/5us. This identifies retrieval as the dominant measured
substitution cost. It does not measure the internal share of each retrieval
lane. Exact local build identity: `run-sppzlrbv/substitution-cost-candidate-identity.json`;
private `phrase-cadence-substitution-cost/substitution-cost-summary.json`, raw
stderr/trace/input/receipt/environment; exact attribution patch retained as
`substitution-cost-private-instrumentation.patch`. Temporary attribution code
was then removed and substitution.rs restored byte-for-byte to its baseline.
Installed authority/model/packages unchanged; no promotion verdict.

### Sep 8 TD123: repeated decoder reconstruction — preflight

Mechanism: near-surface requests use64 and32 final slots, calling the lexical
surface reader with512 and256. That reader repeats decoder reconstruction for
the same surface and derivational bases. Existing near-surface caches separate
limits correctly; each raw reconstruction performs its complete bounded DAFSA
walk, scores and sorts candidates, and only then truncates to the caller limit.
The walk/result before truncation depends solely on immutable LexicalPhaseMemory
bytes, exact input text and fixed code constants. Encoder, scoring and sorting
have no layout, learning, safety-profile or runtime-authority input. The
percentage of retrieval cost in this lane remains to be measured by the RED
component observation; total native benefit is not yet established.

Options, design judgments: A9/10, recommended, bounded per-memory reuse of that
complete sorted reconstruction before truncation; B7/10, cooperative cancellation
of obsolete work, reduces queue but leaves current retrieval cost and spans more
layers; C3/10, remove candidates/legacy rules, changes restoration behavior before
independent quality acceptance. Shared final-result truncation across near-surface
limits remains invalid because surface-reader lane reserves differ by limit.

Consequences before code for A: keep the existing reconstruction function's
order, limits, DAFSA visit cap, phase scores and all candidate fields. Store only
the raw sorted reconstruction, never a selected winner, authority, context or
edit plan. Bind reuse to the immutable memory instance and exact query bytes;
different packages have separate stores. Each caller still truncates its own
raw lane and executes unchanged prefix/lane reserve selection. All L1.1/L2/L3/L4,
DecisionCore, SafetyGate/verifier, exact/manual/daemon and stale-publication
contracts remain unchanged. No new word/suffix/fixture/source-id condition,
model threshold, package-format change, learning event or feedback condition.

Use a maximum16 retained entries and a2MiB accounted payload budget per memory
instance. Charge candidate Vec capacity times element size, every word String
capacity, and the owned query-key capacity; container/Arc metadata is separately
bounded by16 entries. Compute this accounting and allocate the query key outside
the mutex. This is a resident-store bound: evicted Arcs held by readers and cold
computations/cloned outputs consume additional transient memory. Measure RSS
separately; never label the2MiB store budget a total RSS bound. Oversized results
bypass reuse without dropping candidates. Eviction loses saved work only.
Share immutable candidate storage through Arc; clone just the requested prefix
outside the mutex. Under the mutex, clone only the Arc, recheck an existing key
after concurrent misses, and detach evictions into storage prepared outside
the lock. Destroy detached values after unlocking. Never hold the cache mutex
during reconstruction, size accounting or caller work. Poisoned access falls
back to normal computation. Concurrent first readers may duplicate computation;
this design adds no single-flight wait protocol. Artifact binding relies on the
existing immutable mapped-package contract, rather than copying package bytes.
Cold misses allocate retained raw material and return a cloned prefix; measure
this cost and retained bytes instead of assuming every workload improves.
Rollback removes the per-instance store and returns the current truncate path;
no persistent migration or compatibility wrapper.

Proof plan: baseline RED must observe repeated real decoder work across limits
and retain a complete before-change candidate-field digest matrix. Cover several
damage classes, clean/unknown/short/prefixed surfaces and limits, including full
surface_candidates reserve behavior. Candidate fields, order and counts must
match exactly after reuse. Test eviction, oversized bypass, package separation,
poisoned access and concurrent consumers holding evicted Arcs through the real
reader; do not make runtime fixture
conditions. Then affected library/IME checks, fresh-context review, whole fixed89
actual-output/per-class/clean comparison and the fixed eight-pair quiet native
gate. No installation on component-only PASS; TD123 and its later ready-refusal
work remain ACTIVE. Fresh-context design review pass1:8/10,H0/M2/L0. Both medium
contract gaps are addressed above: exact allocation accounting versus transient
RSS, and accounting/destruction outside the mutex with insertion recheck and
poison coverage. The review found no smaller demonstrated equivalent boundary.
Design review pass2/2:9/10,H0/M0/L0, accepted within this scope.

Baseline RED: remote run-mH0NZT, local run-r164go50,162.120s. Exactly1764 library
tests selected/executed;1763 PASS and only the new repeated-work invariant FAIL.
Across caller limits, three fixture queries perform8 raw reconstructions rather
than1; the prefixed query16 rather than2. No production cache implementation yet.
The exact two-test observation independently records4 work rows and224 complete
candidate-field/count/order hashes plus source identity; EXPECTED_RED,9.806s.
Raw reconstruction consumes5,515,208us of8,045,188us over that component matrix.
These are unoptimized test timings, not native latency percentiles. The package
SHA3ff9de4d785aed1b547c56da67dd3cf27644af8968bd0e3bb55a252074cf0268,
245,116 source words, matches the native frozen62,424,748-byte dependency exactly.
Evidence: `/home/ubu/.cache/lay/development/run-r164go50/reconstruction-baseline-summary.json`
and `reconstruction-exact-baseline.json`; canonical manifest drift reported,
not rewritten. Added counters are per-instance and cfg(test) only.

Implementation result: a per-instance ReconstructionCache owns full sorted raw
decoder material; surface/prefix readers retain all existing limit/reserve logic.
Resident payload accounting uses actual capacities; metadata storage is reserved
in the constructor. Size accounting, query/eviction storage allocation, result
cloning and detached Arc destruction occur outside the mutex. Replaced concurrent
entries and oversized/poisoned accesses preserve the same raw readout.

Initial GREEN:2217/2217 (lib1768,IME449), remote run-Lerbmg/local run-pgepa9wm,
144.733s. This snapshot predates only constructor metadata preallocation and
the subsequently strengthened byte-pressure test. Final-source verification:
IME scope PASS31.004s in run-l9zRRP/local run-bcgsiu73, followed by six exact
library reader/resource tests PASS27.986s including compilation. Fresh-context
code review pass1:9/10,H0/M0/L1; the missing byte-pressure test was added. Final
review pass2/2:9/10,H0/M0/L0, runtime.rs SHA
9b1bdf23b84f1be22f75dd085912210b6f13112db74d985ca9564195973f6d5d.
The final controls cover count/byte eviction, held retired Arcs, actual retained
allocation accounting, key uniqueness, oversized bypass, poisoned fallback,
separate packages and concurrent real surface readers. No source-ID assertions.

Exact comparison: all224 candidate-field/order/count digests and package identity
match baseline. Raw evaluations147→21; raw computation5,515,208→798,736us;
whole component8,045,188→3,326,151us. Four work controls are1/1,1/1,2/2,1/1
first/final computations across changing limits. Peak retained store16 entries,
342,518 accounted payload bytes. This is component allocation accounting, not
whole-process RSS. Timings are unoptimized test observations, not native IME
percentiles or a recognition-quality percentage.
Exact final receipts:
`/home/ubu/.cache/lay/development/run-bcgsiu73/reconstruction-exact-candidate.json`
and `reconstruction-component-comparison.json`; initial broad summary is
`run-pgepa9wm/reconstruction-first-green-summary.json`.

Installed runtime authority changed=false; no package/model/configuration or
learning mutation. Private release build is in progress. Before any promotion,
run all fixed89 outputs/per-class/clean controls,13 existing native lifecycle/
manual/restoration cases, then exactly eight quiet80ms pairs of parentfa15601f
and the cache candidate. Reuse the same frozen five-word driver, inputs and
envelope, at most two simultaneous clients; no retries/discards. Require8/8
timely missing-letter delivery and24/24 clean preservation on the new candidate,
report composite ready-NoApply separately. This does not close the later TD123
authority/quality, general heldout, resource or physical-keyboard gates.

Private releaseeea32f44be9ab8f2ea6b2c06101af5ae3a881676235b69312500c5a768dd16ee
completed: architecture40.906s, release176.157s, budget PASS,total217.320s.
All697 final Rust file hashes match the reviewed local tree. Exact identity and
three stage logs: local run-bcgsiu73/reconstruction-cache-candidate-identity.json
and reconstruction-cache-{architecture,release,budget}.log. The fixed89 matrix
has started with this exact binary. No installation or model/authority promotion.

Whole fixed89 result on eea32f44: NO_REGRESSION_ON_FIXED89. All89 actual outputs
equal parentfa15601f and installed995b's frozen baseline. Dirty19/47=40.43%,
25 abstentions,3 wrong outputs; clean39/42=92.86%,3 changed. Every source class:

| Class | Correct/total | Percent |
|---|---:|---:|
|missing_letter|5/8|62.50%|
|repeated_letter|0/4|0.00%|
|transposition|5/5|100.00%|
|restoration_regressions|9/24|37.50%|
|context_fixture|0/6|0.00%|
|clean_missing_letter_control|3/4|75.00%|
|clean_repeated_letter_control|4/4|100.00%|
|clean_valid_word|32/34|94.12%|

All five fixed shards completed18/18/18/18/17 cases, with frozen driver/content,
initial learning, authority inputs and resource envelopes equal across versions.
All267 statuses independently recomputed from actual after.visible; no changed
output or regression against either comparison version. Only the already
documented mtime-derived L3 manifest_stamp is excluded from identity comparison;
content identities stay checked. Private processes reaped. Exact local evidence:
`autocorrect-live-ojoasco5/fixed-fixtures-v6-reconstruction-cache-comparison.json`
and five `fixed-fixtures-v6-reconstruction-cache-{0..4}/` raw directories.
This existing experimental ordered post-ready set proves bounded non-regression,
not general/heldout/all-profile quality, the conjunctive L1 gate or physical input.

All13 unchanged native controls PASS: US first-word1,RU first-word1,manual3,
lifecycle3,restoration5. Timings2.188/2.186/3.633/1.961/2.999s; all candidate/
697-source/driver identities verified before/after, private cleanup complete.
Exact local report: run-bcgsiu73/reconstruction-cache-native-controls.json and
five native directories containing each receipt, run metadata and run.log.
Manual cases are private client protocol proof, not physical Double Shift.
The predefined eight quiet80ms pairs are now running. Installed authority=false
for this work; remaining restoration and physical acceptance stay OPEN.

The predefined eight-pair quiet80ms gate completed in47.609s: all16 runs,
80 words, zero retries/discards, exact input/content/envelope parity, all five
optional stderr flags absent and complete private cleanup. Bounded readiness
verdict PASS on the cache candidate; protocol completion is recorded separately.

| Native80ms denominator | Parentfa15601f, this new series | Cacheeea32f44 |
|---|---:|---:|
| Missing-letter delivery by Space |5/8|8/8|
| Composite correction, separately ready-NoApply |0/8|0/8|
| Clean words preserved |24/24|24/24|

Final-frame totals on the cache candidate52,725..64,536us, median55,154.5us;
queue2..479us. Parent totals65,899..114,318us, median68,439us; queue3..31,268us.
Actual press intervals: parent78,655..85,036us, median81,651; cache78,328..84,476us,
median81,543.5. Parent5/8 belongs to this new paired series; the historical7/8
series above remains unchanged. Neither denominator is independent-word accuracy.
The eight-pair80ms delivery criterion is satisfied; this does not establish
latency percentiles, physical-keyboard behavior, other cadences or model quality.

Exact local evidence:
`autocorrect-live-ojoasco5/cadence-reconstruction-stability-summary-v1.json`,
`cadence-reconstruction-stability-gate-v1.json` and all16 raw
`phrase-cadence-reconstruction-stability-{parent,cache}-{0..7}/` directories.
All160 known raw artifacts were copied. Full fixed89 output/per-class results
and13 native controls above remain the separate non-regression evidence.
No installation/model/authority promotion. Continue TD123 with ready-result
refusals through the complete L1.1→L2→L3→L4→DecisionCore→verifier path.

### Sep 8 TD123: ready-result first-loss audit and bounded authority preflight

Measured source: private reconstruction candidateeea32f44, unchanged fixed89
and its exact 9 dependencies, 12 additional inputs, configuration and driver.
No new production code, package, installation or authority change in this audit.
The 89 native outputs remain19/47 dirty correct and39/42 clean preserved.
Of25 dirty ready-NoApply rows,23 retain the **full expected replacement** in
DecisionCore:21 are SuggestOnly without L3 directional or exact L4 support;
two are Eligible but rejected by an authoritative hidden-state ambiguity.
Two boundary targets never reach DecisionCore. Each evaluated row was joined
to one stderr decision using its prepared decision/l3 timing tuple and full
replacement bytes, including the actual retained context.88 joins succeeded;
the remaining clean row has explicitly zero decision work and preserves input.
The original ordered-shard/native learning semantics are unchanged.

An independent private service run queried all89 final lexical tokens plus the
two supplied phrase tokens, with the identical L1.1 service1cedb27c, V9 artifact
bf5a1619, proof49839304 and dependency manifest75a0a693. A second run queried
all65 distinct clean expected targets. Both completed and reaped their private
service;1.522s and1.363s, inside the existing CPU200%/1536MiB/no-swap envelope.
These are service observations, not a new IME run or a full heldout proof.
Four boundary edits are excluded from the one-token dirty denominator43.

| Original fixture group, one-token scope | Cases | Target in L1.1 lattice | Target authority |
|---|---:|---:|---:|
|restoration regressions|20|8/20,40.00%|0/20,0%|
|missing letter|8|6/8,75.00%|0/8,0%|
|repeated letter|4|0/4,0%|0/4,0%|
|transposition|5|3/5,60.00%|0/5,0%|
|context fixtures, final token only|6|6/6,100%|0/6,0%|
|dirty aggregate|43|23/43,53.49%|0/43,0%|
|clean missing-letter controls|4|0/4,0%|0/4,0%|
|clean repeated-letter controls|4|3/4,75.00%|1/4,25.00%|
|other clean controls|34|20/34,58.82%|18/34,52.94%|
|clean aggregate|42|23/42,54.76%|19/42,45.24%|

The43 dirty readouts are40 ABSTAIN plus3 Winners preserving the observed token,
with zero target Winners. Of43 expected targets,28 survive their separate clean
service query;15 do not. Of those28, five disappear for the damaged query and23
remain without authority. A missing clean-query target is an observed service
result, **not** independently decoded proof of absence from the package's
terminal table. For the42 clean queries, readouts are19 Winner,4 Tied,19 ABSTAIN;
this does not change the separately measured native clean preservation39/42.
The supplied missing-letter target survives without authority; the composite
target is absent from L1.1 but is generated and retained by later layers.

The installed full V9 receipt's SHA matches the frozen dependency. Its
`unique_top1=208739/210974=98.9406%` uses the **objective-unique** denominator;
its total260000 damaged readouts separately contain32063 Winner,302 Tied and
227635 ABSTAIN. Per-class rank/retention data remain in that exact proof and
in the audit packet. This is the existing ranking gate, not an automatic-edit
rate. The receipt verifies calibration geometry0/positive990/backward663;
`typed_basin/settlement.rs` still passes that calibration to restoration
classification. This audit has not established the historical compiler inputs
that produced it. No arbitrary threshold increase or package rebuilding follows
from a ranking PASS. All L1 conjunctive proof dimensions remain required.

First demonstrated authority-transfer defect: one current frame has a complete
canonical exact-partition lexical Winner and both ordinary and canonical target
lanes are subsequently rejected as `ambiguous`. The competing operation changes
a token boundary. There are no exact L4 transition counts or directional L3
certificate for either choice in this recorded frame. Current
`L4PhaseWitnessReadout.supported` means that the bank contains any center;
`estimate_hidden_typing_state` lets that presence make an unresolved choice
blocking. `candidate_has_apply_authority` receives the validated frame capability
but its early hidden-state rejection ignores it. This is different from the21
retained SuggestOnly targets, for which sufficient authority was never present.
The other native ambiguity has a Tied lexical frame and cannot receive the same
repair merely because its expected fixture output is known.

Alternatives before edits, scores are engineering judgments:

1. **9/10, selected:** in the existing authority adapter, let a currently validated
   frame-bound lexical capability survive **only advisory L4 ambiguity**. The
   exception is unavailable in the presence of independent L3 Support/Suppress,
   an exact L4 transition observation, or a verified operator-consensus witness
   in the evaluated lattice. Explicit rejection, a selected witnessed rival,
   malformed certificates, missing/expired/wrong-frame capabilities and the
   existing downstream safety/edit-verifier checks remain effective. Generic
   phase pressure continues to rank candidates and does not become a new veto.
2. **8/10:** carry the capability as a new typed lexical witness through the L4
   active-hypothesis certificate protocol. Semantically reasonable, but requires
   changes to probe identities, certificate replay and more data structures.
   No need for that larger protocol migration to repair this bounded transfer.
3. **4/10, rejected:** disable hidden ambiguity generally. This could admit
   unrelated boundary or lexical alternatives without a valid lexical winner.

Consequence check: no changes to L1.1/L2 candidate generation, bounded retention,
raw scores, model files, calibration, common L3, learned feedback, profiles,
SafetyGate, edit-plan validation, final verifier, ClosedExact or manual Shift
ownership. The positive exception requires the issuer/projector/consumer's
existing event-specific capability validation, not source names or candidate
rank. No word, suffix, fixture ID or manually weighted exception is allowed.
Expected native benefit is a hypothesis until the complete fixed89 rerun;
the21 missing-authority cases, two missing boundary targets and the composite
phrase are not claimed fixed by this step.

Operational consequences and removal boundary, before production edits:

- The extra evidence check scans the already evaluated, bounded lattice only
  when the current candidate has a validated frame capability and the refusal
  is `ambiguous`. It performs no allocation, reconstruction, field traversal,
  model access or locking. CPU cost is linear in that lattice per qualifying
  candidate; no additional resident state or cache is introduced. A negligible
  deadline/tail effect is an estimate, to be checked by the same native cadence
  protocol; bounded scans and the existing memory envelope are not RSS proof.
- Existing capability validation still binds query, event, owner, epoch,
  generation, configuration, package identity and lease. The worker's publish
  and stale-result checks remain authoritative. No new generation, timer,
  cache key, invalidation rule or concurrent writer is introduced. Package or
  delta reloads retain their current invalidation path. A mismatched or expired
  capability cannot enter the exception; failure remains a refusal.
- Ranking and candidate retention stay unchanged, but admitting a previously
  blocked edit can change subsequent learned history. This is a second-order
  behavior change even though learning code is untouched. Preserve ordered
  fixture shards and compare their complete outputs. Future L3 directional
  observations, exact L4 transition observations or verified operator witnesses
  anywhere in the lattice disable the exception; future packages do not inherit
  authority from an old frame. False acceptance or any required error-group
  regression rejects the patch rather than relaxing a gate.
- IME and daemon consumers keep their existing typed receipt, verifier,
  SafetyGate and edit-application contracts. There is no new execution route,
  fallback, public protocol or owner. Strict-profile refusal, invalid-certificate
  refusal and selection of a witnessed rival remain explicit regression checks.
- Rollback removes this one adapter exception; before installation it has no
  production effect. If later installed and rejected by physical acceptance,
  restore the last verified installed artifact through the existing procedure.
  A future typed lexical witness in L4 should replace and remove this adapter
  exception, with the same proof obligations, rather than become a second
  authority owner beside it. Until then the bounded evidence predicate and its
  cross-candidate negative controls are the maintenance obligation.

TDD/proof plan: obtain a real issuer→projection→consumer capability using existing
test fixtures; preserve that field and vary L4 evidence. Reproduce advisory
ambiguity refusal, then require the exact surface, authorized transition receipt
and verifier effects to survive. Cover missing/expired/mismatched capability,
exact negative and rival positive evidence, L3 evidence, invalid resolution
certificate and all three safety profiles. Do not inject a bare true authority
boolean as the sole proof. Run scoped lib/IME checks, fresh-context design/code
reviews, unchanged native fixed89 with per-group results,13 native controls and
the frozen eight-pair80ms readiness gate. Reject any required-group regression.
Full release, RSS and physical ordinary-keyboard acceptance remain later gates.

Independent design review completed in two passes: final 9/10, H0/M0/L0,
accepted. The first regression reuses the real lexical issuer/projection and
DecisionCore evaluations, then supplies an unresolved domain through the real
L4 resolver. It is a controlled authority-adapter probe, separate from the
recorded native first-loss evidence; it does not recreate the evaluator.
Production code is still unchanged at the RED snapshot.

RED completed on the unchanged adapter: 1,769 selected/executed library tests,
1,768 passed and only `frame_bound_lexical_authority_survives_only_advisory_l4_ambiguity`
failed. All 12 profile/phase observations were evaluated. The no-phase control
admits Normal/Experimental and refuses Strict; adding zero, negative or equally
positive phase pressure incorrectly refuses both permitted profiles, while
Strict remains refused. Source snapshot86816729705633803c5e3d237ba72845c99937c065f9c9eb37c1d4446ac24538;
remote run-nONUHN, local `~/.cache/lay/development/run-dy6ahi11/` contains
`authority-adapter-red-{test-summary.json,discovered-manifest.json,lib.log}` and
`RESULT.json`. Guarded check136.331s, focused harness109.767s. An earlier 2.611s
format failure executed no tests and is not the RED behavior proof. The canonical
manifest reports additions without mutation. Implementation additionally requires
a valid L4 certificate and no selected witnessed rival before the exception.

First GREEN: library1,772/1,772 plus IME449/449, total2,221/2,221, no failures;
guarded check145.036s, discovery25.362s and execution116.495s. Exact local
`~/.cache/lay/development/run-72639mlo/ready-authority-test-summary.json` and
`RESULT.json`, remote run-E5hXNy. Code review pass1:9/10,H0/M0/L1. The sole
test-evidence issue was that a tampered replacement invalidates both the frame
and the edit, so it cannot isolate the verifier. The same test now also obtains
a real capability for a protected-token candidate: frame validation must remain
true, the real action verifier must fail, and advisory ambiguity must not yield
an authorized transition under Experimental. No production code changed during
that review repair. Final scoped verification and review closure are pending;
the first GREEN does not claim execution of the strengthened control.

Final scoped check includes that strengthened control:2,221 selected/executed,
2,221 passed,0 failed (library1,772 and IME449),141.998s guarded total;
discovery23.719s, execution115.090s. Code review pass2:9/10,H0/M0/L0, accepted.
Exact local `~/.cache/lay/development/run-i4ruo2vh/{RESULT.json,ready-authority-final-test-summary.json}`,
remote run-1E8BoD. All697 Rust source hashes match the reviewed checkout;
admission SHA553c9a17cddb626cee037357d653705672953cf6195a601375f4ad161c357b23,
test SHA c7e70fc822993acb6b6c5dc7540ec3f8dff5c23a39b6808ebb13ded2cb3d074a.
The canonical manifest's added identities are explicit drift, not silently
updated release acceptance. A private candidate build and graph refresh are
running from that exact snapshot. Native benefit, cadence, RSS, full release
and physical acceptance remain unmeasured for this adapter. Installed runtime
authority changed=false; the candidate's authority change is the bounded
adapter exception described above.

Resource observation protocol for the next13 native controls: an external
private observer will match only the candidate executable's device/inode and
PID/start-time, sample kernel VmRSS/VmHWM and its private client cgroup memory
peak every20ms, and retain sample counts and coverage. It will not alter the
versioned driver, engine flags, client limits, model inputs or application
deadlines. These are sampled process/cgroup high-water observations; unobserved
exit intervals are not a claim of exact final process peak. The separate quiet
eight-pair timing gate has no resource sampler. The enforced1536MiB client
memory envelope and cache-accounted payload remain separate measurements.

Exact private artifacts under `/home/ubu/.cache/lay/development/autocorrect-live-ojoasco5/`:
`fixed89-ready-path-audit-v1.json`, `l11-and-native-first-loss-v1.json`,
`l11-fixed-audit/{l11-fixed-audit-results.json,run-metadata.json,run.log}`,
`l11-known-target/{l11-known-target-results.json,run-metadata.json,run.log}`,
their frozen input manifests and private helper source. The installed package
proof is the exact4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d
receipt under `~/.local/share/lay/nanda_wave/l1.1/`. Runtime authority changed=false.

### Sep 8 TD123: bounded authority transfer native result

Private candidate7a58535c9b1e9e00e4e79918b5d58b3cd70103f526c8948167389b76dc13d663
was built from the reviewed697-file Rust snapshot: architecture41.407s,
release176.187s, total217.846s, Cargo budget PASS. All six generated graph
artifacts were copied back with SHA/size verification. Later owning-document
updates still require the final architecture refresh. There is no installation.

The unchanged five ordered shards executed all89 fixed fixtures and cleaned up.
All267 before/parent/candidate statuses were recomputed from complete visible
outputs. Driver, nine dependencies, additional/system inputs, initial learned
data, configuration and authority identities match; only the previously justified
mtime-derived L3 manifest_stamp is excluded. Correct dirty restoration increases
19/47→20/47; clean preservation stays39/42. One actual output changes from a
retained typo to the exact expected lexical correction. No previously correct
case is lost, no new WRONG_OUTPUT appears, and no required group's correct or
false-output count regresses. Remaining dirty outcomes are24 refusals and3
wrong outputs; clean has3 wrong outputs.

| Fixed fixture group | Parent correct | Candidate correct |
|---|---:|---:|
|Restoration regressions|9/24,37.50%|10/24,41.67%|
|Missing letter|5/8,62.50%|5/8,62.50%|
|Repeated letter|0/4,0%|0/4,0%|
|Transposition|5/5,100%|5/5,100%|
|Context fixtures|0/6,0%|0/6,0%|
|Dirty aggregate|19/47,40.43%|20/47,42.55%|
|Clean missing-letter controls|3/4,75%|3/4,75%|
|Clean repeated-letter controls|4/4,100%|4/4,100%|
|Other clean controls|32/34,94.12%|32/34,94.12%|
|Clean aggregate|39/42,92.86%|39/42,92.86%|

All13 unchanged native controls pass: US1/RU1/manual3/lifecycle3/restoration5,
with exact driver/source/candidate validation and cleanup. The external resource
observer saw all five engine processes and client cgroups,413 samples total.
Largest observed process VmHWM/VmRSS is360,992KiB,352.53MiB; largest observed
cgroup memory.peak is363,495,424B,346.66MiB. These kernel accounting domains differ
and are not summed or substituted for each other. All sampled client limits
are1,610,612,736B/no swap; observed swap/oom/oom_kill counts are0. Sampling does
not establish an exact final peak after the last observation or physical RSS.
It is distinct from the cache's accounted payload and the enforced envelope.

The frozen quiet eight-pair80ms protocol also passes on these bytes. Both parent
and candidate restore the missing-letter token8/8 and preserve clean24/24;
both leave the composite token ready-NoApply0/8. All16 runs/80 outcomes and
absence of all five optional stderr flags were independently checked after
copying raw artifacts; no resource sampler, retries or discarded runs.
Series time47.623s. Candidate final-input-to-publication totals are59.554–75.144ms,
median66.788ms; parent50.149–66.449ms, median53.030ms. The observed total median
is higher, so no cost-neutrality or speedup claim follows from the gate PASS.
DecisionCore medians are8.866ms candidate and9.098ms parent; the measured total
difference is not localized to the edited adapter by these stage observations.
Candidate queue peaks at8.840ms. Eight runs do not establish p99 or other cadences.

Verdict: the demonstrated lexical-capability transfer defect is repaired without
fixed-set regression, and the bounded readiness/control checks pass. This does
not complete TD-123, establish full heldout/all-profile quality, explain every
remaining refusal, or constitute release/physical acceptance. A fresh audit of
the89 candidate traces now finds24 dirty refusals,22 retained targets:21
SuggestOnly and one Eligible ambiguity, plus two absent boundary targets.
The91 independent L1.1 query inputs are byte-for-byte unchanged, so their frozen
service/package observations remain reusable; no new L1 authority was created.
Further work must identify the first shared missing authority mechanism before
another patch. Model/package/installed runtime authority changed=false.

Exact private receipts under `~/.cache/lay/development/autocorrect-live-ojoasco5/`:
`fixed-fixtures-v7-ready-authority-comparison.json`, all five corresponding raw
shards, `fixed89-after-ready-authority-path-audit-v1.json`,
`cadence-ready-authority-stability-{summary,gate}-v1.json`, all16 raw cadence
directories and `ready-authority-cadence-stage-observations.json`.
Build/source/reviewed-test/native/resource receipts are under local
`~/.cache/lay/development/run-i4ruo2vh/`, remote run-1E8BoD, including
`ready-authority-{candidate-identity,native-controls,native-resources,graph-copy}.json`.

### Sep 8 TD123: remaining exact-field refusal diagnostic preflight

The current89-case audit contains24 dirty refusals. Some retained targets have
an L2 IntegrityFailure before L3, while other fields are complete with a real
multi-target tie. The existing error merges raw storage overflow and invalid
structural-work proof. Source inspection shows that raw Phase-7D certificates
are capacity-checked before the supported authority partition is replayed.
It is a hypothesis, not a measured cause, that unsupported roots consume this
capacity. Neither a larger limit nor wider authority is authorized by that fact.

The smallest discriminating observation will call the existing exact V13
generation on all91 frozen token inputs (the89 fixtures plus the two cadence
tokens), with the same canonical package and sidecar. A temporary cfg(test)
observation immediately before the completed-proof guards will record raw and
supported target/root counts, certificate replay errors, the actual capacity
and work-budget predicates, and the unchanged producer result. Expected targets
are joined only after execution. This measures an earlier package/query stage;
it does not itself prove frame grounding, ranking, edit authority or native
output quality. Genuine supported ties and unsupported relation classes remain
separate from capacity failures.

Consequence check: only the private frozen remote test snapshot is instrumented;
release runtime code, candidate generation, limits, ranking, verifier, cache
identity, leases, learning, package reloads and IME/daemon ownership stay on the
reviewed candidate. Diagnostic replay/JSON allocation increases test CPU/RSS and
latency, so these timings are not native performance evidence. The existing
remote resource/Cargo guard owns the sole heavy lease. The harness remains
filesystem/network sandboxed. Exact source/dependency/test identities and91
actual observations are required; no retries or example selection. The two
temporary source edits will be saved as a private patch and restored byte for
byte after execution, including failure. No installed process or model changes.
Production repair options and their consequences will be assessed only after
this observation distinguishes raw capacity, supported capacity and work limits.

#### Measured diagnostic result and capacity-scope repair preflight

The guarded private observation executed exactly91/91 inputs in14.713s
(compile23.54s,total38.921s). The nine dependencies and input bytes match, all697
Rust sources were restored byte for byte, and all original runtime control flow
was retained. There are55 raw-complete and36 raw-storage-overflow queries;
all91 pass the existing structural-work predicate and certificate replay. Raw
target counts range0–364, max3 roots/target. Every supported partition fits the
existing limits: max19 targets and2 roots/target, versus limits74 and4. The36
overflow records are15 dirty and21 clean. Among the24 native dirty refusals,
exactly14 have this earlier overflow; the other10 have raw-complete fields.
Nine of those complete native fields are actual supported ties and one has no
supported target. The cadence composite token likewise has one raw candidate
and zero supported candidates. Capacity repair is not a solution for an
unsupported relation or independently unresolved competition.

Root cause is the completeness scope: `from_candidates` marks the untruncated
raw set over capacity; `with_completed_search_proof` rejects that marker before
replaying the supported partition. `authority_partition` repeats the same
blanket rejection. Search work was complete in all measured overflow cases.
However, merely removing these checks is insufficient: common material and
the display lattice require all exact births to be mandatory and would then
reject the raw74+ set downstream. Both consumers must keep their original bound.

Options, scored for this task:

1. **9/10, selected:** keep the existing raw enumeration and completed search
   proof; validate the supported74/4 partition independently. Feed the common
   material/lattice a bounded, explicitly incomplete view only when raw storage
   overflow occurs. It includes every supported exact target, keeps the raw
   count/digest/overflow, and cannot carry a completed authority proof itself.
   The original untruncated enumeration alone owns the authority partition.
2. **7/10:** introduce separate raw-search and bounded-partition container types
   through the producer/bridge/live/diagnostic APIs. The type distinction is
   clearer, but widens this repair into an API migration and duplicated stored
   material when the existing proof/scope boundary can express the invariant.
3. **3/10, rejected:** increase the common74-target or4-root limit. It consumes
   more memory/work, still conflates scopes, and future packages can exceed it.
   Filtering raw candidates at the search producer is also rejected because it
   would silently narrow complete search/display evidence for ordinary queries.

The selected change must allow a completed search proof past raw StorageCapacity
only; every other incompleteness, invalid work proof, wrong package/observed/
sidecar/semantics binding, missing retrieved form, or unreplayable certificate
still fails closed. A supported set over either74 targets or4 roots/target
must remain non-authoritative with no truncation before certification. Existing
counts/digests include all supported competitors. Raw diagnostics stay raw and
overfull. The common-field view is made only after the original authority
material is prepared: on raw overflow it carries the complete bounded supported
subset but preserves raw logical count/digest and StorageCapacity, with no
search proof. Whole-field completeness therefore never becomes a substitute
for the independent relation-partition proof. Grounded L1.1 candidates retain
their mandatory lane; supported targets enter the existing exact lane. If the
union of mandatory grounded and supported targets exceeds74, the existing
capacity refusal remains; this patch does not invent a truncation preference.

Consequences and boundaries:

- **Retention/rank/false authority:** previously lost supported roots become
  available, including rivals. No score, tie rule, relation class, acceptance
  threshold, SafetyGate, edit verifier or consumer capability check changes.
  The larger valid field may turn an apparent singleton into a tie and must
  never certify from a truncated prefix. Common display candidates can change
  in the36 previously failed queries; whole fixed89 target retention, complete
  visible outputs, false outputs and each error class must not regress.
- **Latency/CPU/RSS/allocation:** measured supported sizes are small but future
  packages may reach the existing limits. Raw search already materializes the
  full set; successful replay and common material now add work where an early
  error used to stop it. BTree count/replay/cloning work is bounded by existing
  search limits and74/4 partition limits, not estimated as free. No new hot-path
  tracing or second package lookup is added. Native13 sampled RSS and the quiet
  eight-pair80ms gate must be repeated after the full fixed set; no p99 claim.
- **Cache/reload/concurrency:** use only the immutable per-query enumeration and
  current package tuple. No new cache, owner, fallback route, memoization or
  mutable state. Existing generation invalidation, lease checks, result discard,
  package/delta reloads and frame/epoch binding remain. Current raw checksums and
  supported replay are required again after a changed package; future capacity
  overflow stays closed. The diagnostic patch is already removed.
- **Learning/feedback:** no learning rule changes, but any additional accepted
  output can change later history. Preserve frozen initial learned bytes,
  ordered fixture shards and complete metadata checks; report changed output
  and downstream evidence separately. Newly learned rivalry is not waived.
- **Failure/compatibility/rollback/removal:** IME and daemon consume the same
  prepared-field APIs; no format, package schema, installed service or source
  migration. Invalid common material still refuses; a valid authority view does
  not bypass final frame/target verification. Roll back this scoped capacity
  separation without reverting the accepted timing/authority adapter work.
  Keep raw/common/authority handling within the existing enumeration/live
  owners; a future explicit container-type migration replaces this view method
  rather than adding a parallel authority owner.

Meaningful TDD must first reproduce failure on raw>74 but supported singleton
through the real issuer/projector/consumer fixture, then test supported ties,
true supported target/root overflow, missing/stale/corrupt proof, wrong frame,
and retention of raw overflow in common material. A repeat of the full91 query
observation must preserve every raw candidate/certificate count and supported
set; only eligibility of the completed bounded partition may change. These
component observations are not quality proof. Final native89 percentages and
new false outputs remain separate from field completeness and admission counts.
Fresh-context design/code review follows the requested two-pass maximum.

Exact private diagnostic receipt:
`~/.cache/lay/development/autocorrect-live-ojoasco5/exact-authority-capacity-probe-v1/receipt.json`,
with raw `probe.log`, `test-only-observation.patch`, discovered manifest and
`compact-summary.json`. Installed runtime authority changed=false. The selected
production repair and its TDD are pending at this preflight.

Design review pass1:8/10,H0/M2/L1. Before implementation the selected route is
made explicit at three downstream boundaries. The lattice carries a distinct
exact-storage-overflow marker with its raw target lower bound and digest; root
overflow must be representable even if target count is at most74. Common
completeness combines this marker without relabelling it productive overflow.
Its presence also requires common L3 settlement even if the retained view has
zero or one surface, preventing an apparent productive singleton from gaining
eligibility. `live_authority` still checks a grounded L1.1 Winner first; no generic
incompleteness downgrades that Winner or removes its candidate.

An empty supported view of a nonempty raw enumeration must still contribute the
raw digest to common material. A held empty birth list is not evidence that no
exact enumeration occurred. Both production and test completeness must retain
the raw lower bound, StorageCapacity and digest for this observed empty shape.
The completed-proof issuer and authority partition will validate the raw count
and raw digest against the held untruncated births, in addition to their existing
bindings and replay. A bounded view with raw metadata therefore cannot attach
or reuse a completed proof. This uses the existing immutable identities and
avoids a new ownership flag or another authority container.

Add discriminating regressions for productive-singleton deferral, grounded
Winner preservation, the supported-empty overflow/digest/no-proof shape,
attempted proof reattachment, mandatory grounded-plus-exact target overflow,
and supported-plus-anchor witness-root overflow. Neither separate lane fitting
nor absence of a proof at construction time is sufficient evidence for the union
or for non-issuability. No SafetyGate, final verifier or L1.1 contradiction rule
is changed to satisfy these tests. Production code is still unchanged pending
design pass2 and the recorded TDD RED.

Design pass2 closed at9/10,H0/M0/L0. TDD RED then selected and executed1773
library tests:1772 passed and only the new
`raw_exact_overflow_preserves_bounded_partition_and_common_incompleteness`
failed at the old completed-proof rejection. Its fixture generates75 distinct
unsupported double-substitution surfaces using the real Phase-7D oracle, adds
the actual canonical singleton, and reaches the intended StorageCapacity failure;
this is a behavioral RED, not compilation or formatting failure. Total135.770s,
discovery23.432s, execution109.135s. Local evidence:
`~/.cache/lay/development/run-aa87o1gr/exact-capacity-red-{test-summary.json,discovered-manifest.json,lib.log}`;
remote run-BFvwsr has the original `tests/{SUMMARY.json,DISCOVERED_MANIFEST.json,logs/lib-lay.log}`.
Canonical test manifest reports only additive drift and was not rewritten.

The scoped capacity repair is implemented in the existing material, composite
and live-field owners. Raw count/digest validation prevents a bounded common
view from receiving or reattaching a completed search proof. The original raw
enumeration can prove a complete supported partition under the unchanged74/4
bounds. Common material retains the raw StorageCapacity marker and digest,
including an empty supported view. That marker requires common L3 settlement;
the grounded L1.1 Winner route remains first. Mandatory grounded/exact target
unions and supported/anchor witness-root unions still fail closed on overflow.
There is no new owner, cache, threshold, package change or verifier bypass.

The first scoped run executed2227 tests with2226 passing. Its sole failure was
an invalid new test expectation: a different frame supplied at issuance defines
a fresh context. The corrected test issues and projects once, retains the same
settlement/capability, then replaces the current frame. It proves positive
admission, stale-use denial, expiry denial and final no-edit with a passing edit
verifier. No production change was needed for that test correction.

The final scoped run passed2227/2227 (1778 library,449 IME), including all six new
capacity regressions. Total142.392s; discovery23.639s, execution115.549s. All697
Rust files match the frozen checked snapshot. Source review closed after two
passes at9/10,H0/M0/L0. The canonical manifest has21 additions only and remains
unmodified; its normal refresh and full release gates are still required.
Exact receipts: `~/.cache/lay/development/run-q5pd3id5/` contains the first
scoped FAIL; `~/.cache/lay/development/run-vx8clfk5/{RESULT.json,exact-capacity-final-test-summary.json,exact-capacity-final-discovered-manifest.json}`
contains the final PASS, remote run-X9Migg. The private source delta and identities
are `~/.cache/lay/development/autocorrect-live-ojoasco5/exact-capacity-code-v2.patch`
and `exact-capacity-code-v2-identities.json` in the same directory; the corrected,
formatted frame-test SHA is5a070bc3c227a6080a34bfd844be1d1c7cadda8a5e2630f7ae8095de12ec7a40.

Native89 quality, the same91 raw/supported query observation, sampled native13
resources and eight quiet80ms pairs remain pending. Component correctness and
review do not establish those results. Installed runtime authority changed=false.

The private candidate is now1bceb86970f44c38e1eb39457f88efd2acac962570a5a2db3b2737b6aa942fca,
remote run-X9Migg/lay-ibus-engine-exact-capacity. Architecture41.124s and
release176.312s passed, total217.706s including the Cargo budget check. All697
Rust hashes match the reviewed/checked sources. Six generated graph artifacts
were copied with exact SHA/size validation; later local result documentation
still requires the final graph refresh. Local identity/graph/build receipts
are under run-vx8clfk5 with the `exact-capacity-` prefix.

The same91-query private observation passed again: every raw count, supported
surface/root set, replay outcome and recorded structural-work counter matches
the baseline. All91 completed producer calls return the original raw diagnostic
scope, and the real post-issuer authority partition validates in91/91, including
empty partitions. A valid empty partition grants no target capability. This
comparison records raw counts, not all raw surface/certificate content. No query
producer, package or search lane changed. Test15.320s, compile23.43s, total39.408s;
all temporary test instrumentation was removed with697/697 source hashes
restored and all dependencies/input bytes unchanged. This is component proof,
not native latency or output quality. Exact local receipt and raw observation:
`~/.cache/lay/development/autocorrect-live-ojoasco5/exact-authority-capacity-probe-v2/`.
Native89, native13 resource sampling and quiet80ms measurements follow.

#### Exact capacity native verdict and next missing mechanism

All89 fixed fixtures completed under the same Experimental profile, frozen
inputs, ordered learning shards and key-after-ready schedule. The comparison
independently recomputed267 full visible outputs against installed-baseline995b,
parent7a58535c and candidate1bceb869. No previously correct case, required group
or false-output count regressed. The only change against the parent preserves
the clean input `важно`, which previously became `важна`.

| Fixed group | Parent correct | Capacity candidate correct | Candidate wrong outputs |
| --- | ---: | ---: | ---: |
| All damaged |20/47 (42.55%)|20/47 (42.55%)|3/47 (6.38%)|
| All clean |39/42 (92.86%)|40/42 (95.24%)|2/42 (4.76%)|
| Restoration regressions |10/24 (41.67%)|10/24 (41.67%)|2/24 (8.33%)|
| Missing letter |5/8 (62.50%)|5/8 (62.50%)|0/8|
| Repeated letter |0/4 (0%)|0/4 (0%)|1/4 (25%)|
| Transposition |5/5 (100%)|5/5 (100%)|0/5|
| Context fixtures |0/6 (0%)|0/6 (0%)|0/6|
| Clean missing-letter controls |3/4 (75%)|3/4 (75%)|1/4 (25%)|
| Clean repeated-letter controls |4/4 (100%)|4/4 (100%)|0/4|
| Other clean words |32/34 (94.12%)|33/34 (97.06%)|1/34 (2.94%)|

The complete first-layer audit joins88 actual decision traces and the single
zero-evaluation clean control. All24 dirty refusals now have a complete final
L2 relation partition:20 Tied and4 Abstain, compared with9 Tied,14 Failed and
one complete Abstain before this repair. This removes the measured earlier
capacity failure, but does not independently resolve those competitions.
Final target gates remain21 SuggestOnly, one Eligible and two absent whole-word
boundary targets. All91 lexical query identities are unchanged, so the existing
independent L1.1 readouts remain applicable. Display/request errors within a
trace window are not relabelled as final-frame failures.

The unchanged13 native controls passed. Read-only resource sampling observed
five candidate processes and five private-client cgroups in414 samples. Maximum
observed VmHWM358340KiB (349.94MiB); maximum sampled cgroup memory.peak361132032B
(344.40MiB). These are different accounting domains, not additive measurements.
All cgroups retained1536MiB memory.max and zero swap.max; sampled swap, OOM and
OOM-kill counts were zero. Final process-exit intervals can be unobserved.

The first cadence attempt is an incomplete harness run, not a candidate timing
result: the derived launcher admitted a `ready` label while its batch passed
`capacity`, so all eight candidate starts failed before engine initialization.
Eight parent runs and all eight failure logs are preserved in series-v1. Only
the private mode contract and distinct output names were repaired; binary,
driver, inputs, flags and80ms cadence stayed unchanged. The new fixed series-v2
completed all16 runs/80 words in47.601s, with no within-series retry or discard.
Both variants restored the missing-letter case8/8, preserved clean text24/24,
and had zero wrong outputs. Composite restoration remains0/8 ready-NoApply.
All five optional stderr flags were absent. Candidate final preparation ranged
56.797–66.989ms, median61.794ms; parent52.880–73.162ms, median65.237ms. Actual
input-interval medians were81.562ms and81.553ms respectively. This is a bounded
readiness PASS, not a p99, faster-cadence or causal speedup claim.

Exact native evidence in `~/.cache/lay/development/autocorrect-live-ojoasco5/`:
`fixed-fixtures-v8-exact-capacity-comparison.json`, five complete raw shards,
`fixed89-after-exact-capacity-path-audit-v1.json`,
`exact-capacity-final-refusal-field-summary.json`,
`cadence-exact-capacity-stability-{summary,gate}-v2.json` and all16 raw directories.
The original incomplete series is `cadence-exact-capacity-stability-summary-v1.json`
with eight parent directories and eight candidate launcher logs. Native13 and
resource receipts are `~/.cache/lay/development/run-vx8clfk5/exact-capacity-native-{controls,resources}.json`
with five raw control directories. Installed runtime authority changed=false.
General heldout quality, all-profile release acceptance, installation and the
physical keyboard remain open. Next analysis separates missing supported edit
geometry from independent context evidence needed to settle complete ties.

### TD123 next analysis: geometry and independent context

Source facts: the shared Phase-7D authority predicate admits only single
substitution, adjacent transposition, layout and missing/extra-letter relations;
other enumerated certificate families remain display evidence. Current-sentence
recurrence independently certifies only Damerau distance1. Neither observation
alone authorizes wider geometry or a lower safety threshold. None of the fixed89
failures has its expected word repeated in the current sentence according to
the preliminary text audit; the supplied cadence sequence does. These are
different proof scopes and must not be conflated.

The next minimal diagnostic retains the same frozen91 queries but records the
existing raw `diagnostic_json` candidate/certificate contents on disk. Expected
targets are joined only afterwards to classify absence, unsupported geometry
and real supported competition across the complete denominator. No production
change, package change or promotion is proposed yet. A private cfg(test) patch
uses the reviewed immutable snapshot and is restored byte for byte on exit,
under the same sole remote resource/Cargo guard and sandbox. Additional JSON
and replay work is observation overhead, not native performance evidence.
This creates no runtime owner, cache, route, reload, learning or authority change.

The full geometry observation completed91/91 and restored all697 sources
(test15.355s, total39.175s). Expected-token shapes:38 supported,23 absent and
30 raw-only. All30 raw-only expected tokens are clean controls; no damaged
expected target is present merely behind the supported-geometry filter.
Ten fixed damaged cases plus the supplied composite have no expected token in
the raw exact field. Therefore widening that filter alone cannot repair these
measured missing targets. The supplied composite retrieves a different raw
double-substitution surface, while its intended target needs a deletion plus
insertion. Its exact-field competitor must not be promoted merely because it
is the only raw surface. Exact receipt and complete post-execution join:
`autocorrect-live-ojoasco5/exact-authority-geometry-probe-v1/{receipt.json,target-geometry-summary.json}`.

Next, rerun the existing five-word post-ready native diagnostic on the same
private candidate and frozen inputs, with its unchanged detailed trace flags.
This locates the retained intended target and the current context/authority
path in the actual consumer. It is a bounded causal observation, not a latency
measurement, new fixture denominator, or permission to widen authority.

#### Bounded current-context recurrence: implementation preflight

The unchanged five-word post-ready native diagnostic completed5/5, with4/5
expected outputs and verified cleanup. Four actual known frames join uniquely
to DecisionCore timing. The composite's intended full replacement remains in
the outer bounded lattice as SuggestOnly (rank0.970, L3 NotApplicable, no
pairwise witness). Its final canonical relation partition is complete/empty,
not an invalid frame. In the same sentence, the one-edit control gets the
existing L3 recurrence certificate (support720, pairwise=true) and is restored.
Full raw provenance: `autocorrect-live-ojoasco5/phrase-domain-exact-capacity/`
and `supplied-phrase-after-exact-capacity-path-audit.json`. Detailed tracing
makes these preparation timings unsuitable for the quiet performance gate.

Options relative to the private1bceb869 baseline:

1. **9/10, selected for bounded implementation/proof:** extend the existing L3
   current-sentence recurrence issuer to a conservative two-edit RU case. Keep
   its one-edit behavior first and unchanged. Two-edit issuance requires an
   already-retained candidate, the exact target in the current sentence, the
   same Cyrillic script, an observed length of at least five letters, no
   clean/protected source certificate, no target L3
   suppression, and one unique matching surface across the complete nearby
  current-sentence vocabulary, including identity and nearer alternatives.
   The current canonical field is not fabricated or promoted by this evidence.
2. **7/10, viable but deferred:** represent that same recurrence evidence with
   a new typed certificate carried through L3/L4/DecisionCore. This makes its
   geometry explicit at additional interfaces but expands the producer/consumer
   migration. The existing event-local report and final verified transition can
   carry the bounded evidence without introducing another owner or route.
3. **3/10, rejected for this measured gap:** broaden the supported Phase-7D
   predicate. No damaged expected target in this91-query observation is merely
   hidden behind that predicate; it would admit different competitors while
   leaving the measured missing candidates absent. A full primary-geometry and
   package extension is separate research, not a small version of this fix.
   A bare global distance1-to2 change is also rejected: it lacks clean-source
   preservation and complete current-context competitor checks.

Chosen invariants and consequences, to be reviewed before production edits:

- **Candidate retention and ranking:** no generation, removal, lattice cap,
  lexical score or source-ID exception. Existing one-edit witness success/tie
  behavior within a real current sentence is evaluated first. The shared
  sentence parser must find delimiters before trimming preceding whitespace:
  review found that an immediately preceding newline currently disappears.
  Fix this boundary for both one- and two-edit evidence, with explicit controls.
  Only when the one-edit branch has no witness may a retained
  two-edit candidate obtain the existing positive recurrence report. Every
  distinct same-script sentence token at edit distance0,1 or2 competes for
  this new certificate, even if missing from the candidate lattice. Identity
  or a different nearer/tied context surface prevents new issuance. Duplicate
  aliases of one target share evidence; they are not separate competitors.
  For this new branch, enumerate Cyrillic competitor runs also behind internal
  punctuation, which the existing whitespace tokenizer retains inside tokens.
  A comma-joined rival missing from the lattice must block apparent uniqueness.
  Positive recurrence still requires an exact whole target token in the current
  sentence and a retained replacement; a split fragment cannot supply it.
- **False authority and safety:** reject a two-edit source attested by the
  existing `has_clean_russian_surface_certificate`, whose purpose is input
  preservation and which includes dictionary, morphology and protected words.
  Its exact dictionary starts at length5 and does not cover every bare short
  noun. Therefore the new branch requires observed length>=5; shorter input
  never obtains this new certificate. This covers the reader's exact-source
  range without adding another direct short-dictionary lookup. Existing form
  checks inside the preservation reader can still cold-load that dictionary.
  Short damaged restoration
  is outside this extension, and independent short clean nouns must stay clean.
  Absence of that certificate alone is never positive evidence. The exact
  current-sentence target supplies the independent positive observation.
  This does not prove every unknown input is damaged: clean confusables,
  unprotected names, competing recurrences and missing dictionary coverage
  remain material risks to measure. Existing Suppress, technical/protected
  class filtering, SafetyGate, edit-plan validation, posterior/margin checks,
  exact negative transition evidence and frame/lease verification stay intact.
  Strict still requires its separate evidence domains; no profile is weakened.
- **Latency, CPU and RSS:** reuse the existing issuer and lookup, with one
  source-preservation check only if the new two-edit branch is actually needed.
  Use the existing bounded edit-distance(max2) for its additional comparisons.
  Current sentence and candidate arrays remain bounded by the existing event
  and lattice. The added distinct-context scan and distance work allocate only
  within this call; there is no new persistent cache or background work.
  Existing dictionary/form caches may be cold and their first-use cost is not
  claimed free. Scoped checks, native13 resources and the unchanged quiet80ms
  eight-pair gate must establish the practical bound before acceptance. No
  deadline or resource limit will be raised to pass that gate.
- **Cache, reload and concurrency:** no new cache identity, state, lease, owner
  or synchronization. Certificate inputs are the current immutable event and
  full candidate array; final editing uses the existing bound transition.
  Dictionary/protected-word queries use existing OnceLock process snapshots,
  not live reloadable readers. Changed vocabulary takes effect in a new process;
  an in-process reload does not invalidate those snapshots. This is an existing
  lifetime limit, not a newly implemented refresh guarantee. Future data is not
  assumed equivalent to the frozen proof; current package/worker generations
  and stale-result rejection retain their existing, separate scope. No new
  cache or reload owner is introduced to claim broader protection.
- **Learning and compatibility:** the learning algorithms and data formats
  stay unchanged, but an additional accepted output can affect later in-session
  history. Preserve ordered fixed shards and compare complete outputs, not
  isolated suffixes. Both existing L3 callers use the same issuer, including
  IME/daemon consumers. Current sentence boundaries and script constraints
  remain; nothing is learned from a rejected or stale proposed certificate.
- **Failure, rollback and maintenance:** no certificate means the existing
  decision path continues. No fallback edit or second mutation is introduced.
  Roll back this small issuer extension independently of the accepted timing,
  authority-transfer and capacity repairs. If a later typed recurrence
  certificate replaces it, replace this issuer behavior rather than layering
  another authority route beside it.

TDD must first fail on the old issuer through an actual final consumer result,
then cover no recurrence, prior-sentence recurrence, two witnessed rivals,
nearer/identity context competitors absent from the lattice, clean/protected
source, suppressed target, mixed scripts, more than two edits, alias/order
invariance, all three profiles and actual verifier failure. Also record an
independent bounded family of damaged and clean-confusable context controls;
the supplied example alone is insufficient. The whole fixed89, all existing
scope tests, native13/resource and quiet80ms gates must not regress. Native
composite gains, clean preservation, per-class percentages and unmeasured
generalization remain separate. Fresh-context design/code review follows the
requested maximum of two passes. Production code is unchanged at this preflight.

Design review closed after two passes:8/10,H0/M1/L1 then9/10,H0/M0/L0.
The complete-vocabulary and lookup concerns are resolved in the preflight
above; that review does not establish runtime quality or performance.
Behavioral RED:1779 selected/executed lib tests,1777 PASS and exactly two
failures on the old issuer. The new final-consumer test refuses the intended
two-edit replacement under Normal despite a valid edit verifier; the expanded
boundary test incorrectly issues a positive one-edit certificate across an
immediately preceding LF. This is a behavior failure, not a compile/fmt failure.
Total137.100s, discovery/build23.40s, execution110.254s. Exact raw receipts:
`~/.cache/lay/development/run-6ueix8ux/context-recurrence-red-{summary,discovery}.json`
and `context-recurrence-red-lib.log`; remote snapshot `run-MWB3LK`, archive
`09fddac71e39ed2adba5d87b6736d692e5c63060e5f0f684b06246ca6624c022`.
No installed runtime authority changed. The following implementation remains
subject to the declared scope, whole-proof, native and resource gates.

The bounded implementation is confined to the existing L3 recurrence issuer
and shared sentence parser. The old one-edit branch runs first; the new branch
requires the reviewed complete context and preserved-input conditions, with
one deferred source check. Suppression on any alias of the proposed two-edit
target prevents the new certificate on all aliases. No report is modified
until every new issuance condition passes. The existing report strengths and
all downstream authority/verifier code are unchanged.

Verification additions cover two independent damaged surfaces under all three
profiles, exact selected surfaces and bound receipts; aliases/order;15 negative
context/source controls;28 sentence-boundary combinations; and suppression on
either alias. Actual final-consumer negative controls include clean sources
and absent-lattice competitors. Separate adversarial prefix edits preserve
normalized tokens (and receive the real L3 certificate) but fail the actual
edit verifier and must never produce a final transition. A ProtectedToken
control also retains the actual verifier veto. These scoped checks are pending
execution; they do not replace the frozen native or heldout proof.

Scoped GREEN completed2233/2233 (lib1784,IME449), exactly2233 selected and
executed,2257 discovered,0 failures. All697 Rust source hashes match the frozen
request. Total143.965s, discovery25.177s, execution115.590s. Exact raw evidence:
`~/.cache/lay/development/run-lsqh9s6e/context-recurrence-final-{summary,discovery}.json`
and `context-recurrence-final-lib.log`, remote `run-SJ5HuB`. Canonical manifest
drift is27 additive identities,0 changed/removed; it has not been silently
rewritten. Fresh-context code review9/10,H0/M0/L0 closed in one pass, with no
source repair required. Native and release/heldout acceptance remain open.

Before native execution, a separate16-case context family was frozen:8
predeclared two-operation long-word damages (five construction families) and8
clean context-confusable sources, including short and protected input. Each
variant gets the same complete ordered list, same initial packages/config,
and the same actual terminal/key-after-Ready driver mechanics. Only its fixture
binding/count/scope metadata are adapted; original fixed89 and drivers remain
unchanged. Receipt: `autocorrect-live-ojoasco5/context-recurrence-family-v1.json`,
SHA256 `616bd78fd4f8117225f0ca6e7728a3423d54e4772b30093577d55a1654dd9421`.
Compare all32 outcomes, candidate retention and target certificates against
private1bceb869. Report damage and clean denominators separately; no lost
correct result or new wrong output is acceptable. This family is prospective
bounded evidence, not an unseen heldout or a substitute for fixed89.
An external observer will record candidate process/cgroup memory across its
fresh process and subsequent cases, separately from native13 and quiet80ms.
First-use versus later process execution is observable; reference-reader cache
coldness is not inferred without evidence. No production installation occurs.

#### Bounded recurrence native verdict

Private candidate `1dc3a28315824360d503babeb0e7125317b9f97832463e6caa619a550eaea9d4`
was built from the reviewed697-source snapshot, with architecture41.380s,
release176.553s, budget PASS and total218.206s. All six generated graph artifacts
were copied and SHA/size verified. Later owning-document changes still require
the final architecture refresh. No package, config, driver, deadline or installed
runtime authority changed. Scoped2233 tests and fresh code review9/10 passed.

The complete frozen89 native outputs are byte-for-byte identical to private
1bceb869. All267 status classifications across baseline995b, parent1b and new1dc
were independently recomputed, with full dependency/config/driver identity
parity; only copied L3 manifest mtime is excluded as before. No lost correct
output, new wrong output or class regression occurred. Current measured classes:

| Fixed89 group | Correct / denominator | Percent | Wrong outputs |
|---|---:|---:|---:|
| all damaged |20/47|42.55%|3|
| all clean |40/42|95.24%|2|
| restoration regressions |10/24|41.67%|2|
| missing letter |5/8|62.50%|0|
| repeated letter |0/4|0.00%|1|
| transposition |5/5|100.00%|0|
| context fixtures |0/6|0.00%|0|
| clean missing-letter controls |3/4|75.00%|1|
| clean repeated-letter controls |4/4|100.00%|0|
| clean valid words |33/34|97.06%|1|

The separately predeclared16-case context family improves damaged restoration
0/8→6/8 (0→75%) while clean preservation remains8/8 (100%); wrong outputs0
and lost correct results0. By construction class: deletion+insertion0/2→2/2,
double deletion0/2→2/2, double substitution0/1→1/1, transposition+deletion0/1→1/1,
double insertion0/2→0/2. These tiny denominators are bounded controls, not
generalization or L1 heldout percentages. All16 candidate frames were audited:
the six restored targets were retained as SuggestOnly and received actual L3
support720/pairwise=true. The two remaining failures have prepared candidates0,
field producers0, correction-L3 time0 and DecisionCore time0. Thus no final
decision over a retained target occurred there; upstream birth/retention needs
its own full-path diagnosis before any further change.

All13 unchanged native controls pass:US1,RU1,manual3,lifecycle3,restoration5.
The external observer saw all5 processes/cgroups with414 samples, maximum
VmHWM354316KiB (346.01MiB) and cgroup peak356438016B (339.93MiB). The separate
family observer saw the candidate process/cgroup with736 samples, maximum
VmHWM431788KiB (421.67MiB) and cgroup peak449478656B (428.66MiB). These are
different accounting domains and workloads, not additive values or a memory
growth diagnosis. The1536MiB client limit and zero swap remain; sampled swap,
OOM and OOM-kill counters are0. Final exit intervals can be unobserved. Fresh
process and later cases are covered; exact reference-cache coldness and isolated
CPU cost of the new branch remain unmeasured. No observer ran during quiet80.

The unchanged eight-pair80ms cadence completed16 runs/80 words in47.551s,
without retries, discarded runs or any of the five optional stderr trace flags.
Both versions restore the missing-letter case8/8 and preserve clean24/24.
The supplied composite changes0/8→8/8; new candidate therefore restores both
damaged words in every repetition. Full visible outputs were independently
rechecked against raw receipts; wrong outputs0 for both variants. The existing
readiness/preservation gate and separate composite8/8 gate both PASS.
Missing-letter final preparation median61.964→63.909ms; new range49.900–71.433ms,
queue2–3344us. Actual requested80ms intervals for new candidate were
77.013/81.582/85.553ms (min/median/max). This establishes only the fixed native
cadence gate, not p99, a speedup, or physical-keyboard acceptance.

Exact receipts under `~/.cache/lay/development/autocorrect-live-ojoasco5/`:
`fixed-fixtures-v9-context-recurrence-comparison.json`, all five raw shards;
`context-recurrence-family-comparison-v1.json`, both raw family directories and
`context-family-recurrence-authority-audit-v1.json`;
`cadence-context-recurrence-stability-{summary,gate}-v1.json`, all16 raw runs.
Build/tests/native/resource receipts are under `run-lsqh9s6e/context-recurrence-*`.
The bounded recurrence change is accepted for further validation. General
quality, complete heldout/per-class L1 gates, full release, installation,
physical typing and TD123 DONE remain OPEN. The next task is the remaining
shared failure mechanisms; neither this phrase gain nor its cadence repeats
can replace those proof dimensions.

#### Remaining quality: terminal coverage diagnostic preflight

The next read-only observation separates target availability from search and
authority. Existing91 service queries are insufficient to call a missing clean
target absent from the package. The pinned service already exposes `Decode`
over terminal IDs; no Rust patch, alternate decoder or model rebuild is needed.
Use one fresh private instance of the same service/package and its existing
CPU200%/1536MiB/zero-swap/90s envelope, inside the sole guarded remote execution
lease. Query all91 original tokens plus all16 predeclared family tokens with
the existing lattice/restore calls, then enumerate the complete health-reported
terminal-ID range in fixed512-ID Decode batches. Bound the scan by80s and keep
the existing cleanup deadline; no retries or resource increases.

Record every requested range, returned/missing ID count, response hash and
complete decoded stream privately on disk. Join expected/observed surfaces only
after exact Decode responses. Record host stats so tombstones/overlays, if any,
cannot be mislabeled as raw package absence. Verify all frozen source/config/
service/package identities and cleanup. Recheck old91 readout parity because
the new coverage observation broadens the diagnosis; classify all107 cases,
not only failures. This adds no candidate, rank, certificate, cache, owner,
learning or runtime authority. Decode/JSON work is observation overhead, not a
typing latency result. A missing terminal and a calibrated abstention require
different fixes; no threshold change is authorized by this diagnostic alone.


#### Complete loaded L1.1 terminal coverage: measured result

The pinned private service decoded every ID in its health-reported domain:
852582/852582 nonempty surfaces,1666 fixed512-ID batches,0 missing IDs.
Independent local streaming verification checked consecutive IDs, every batch
hash, the complete decoded-stream SHA and every expected-surface membership.
Host stats report852582 exact surfaces,0 overlays,0 tombstones and0 query
failures; absence here is therefore absence from the loaded base terminal table,
not a tombstone or overlay effect. Package77962328 bytes, manifest generation0.
The107 lattice/restore queries complete; original91 responses are unchanged,
with identical frozen service/package/config/resource dependencies. New16 inputs
are the previously frozen context family. Cleanup reaped the private service.

| Lexical scope | Cases | Expected in decoder | Expected in damaged/clean query lattice | Target Winner authority |
|---|---:|---:|---:|---:|
| fixed single-token damaged |43|28/43,65.12%|23/43,53.49%|0/43,0%|
| restoration regressions, single token |20|10/20,50%|8/20,40%|0/20,0%|
| missing letter |8|7/8,87.50%|6/8,75%|0/8,0%|
| repeated letter |4|2/4,50%|0/4,0%|0/4,0%|
| transposition |5|3/5,60%|3/5,60%|0/5,0%|
| context fixtures, final token |6|6/6,100%|6/6,100%|0/6,0%|
| fixed clean |42|23/42,54.76%|23/42,54.76%|19/42,45.24%|
| separate context family damaged |8|8/8,100%|6/8,75%|0/8,0%|
| separate context family clean |8|8/8,100%|8/8,100%|7/8,87.50%|

The43-token damaged denominator excludes the same four boundary fixtures from
fixed47. It contains15 targets absent from this decoder,5 present but missing
from the damaged-query lattice, and23 retained without target authority. Three
damaged readouts are Winners for their observed surface; they are not correct
target Winners. The two family double-insertion targets exist in the decoder
but have empty damaged-query lattices. These disjoint lexical observations do
not prove the first failure in the full IME path: L2 and personal material can
already supply a target missing from L1.1. Match the current fixed89 consumer
frames before selecting a systemic fix. No word-list supplementation or
calibration increase follows automatically from these measurements.

Driver elapsed19.339s (scope19.417s, CPU19.359s); these include observation work,
not typing latency. Same CPU200%/1536MiB/zero-swap/90s envelope, with80s scan
bound, no retries. No new RSS measurement or full heldout proof is claimed.
Exact local receipts under `~/.cache/lay/development/autocorrect-live-ojoasco5/`:
`l11-terminal-membership-v1/{l11-terminal-membership-v1-results.json,decoded-terminal-rows.jsonl.gz,decode-batches.jsonl,run-metadata.json,run.log,l11-service.log}`,
`l11-terminal-membership-v1-analysis.json` and the pinned input/helper identities.
All source, model, configuration and installed runtime authority are unchanged.
This closes the bounded membership diagnostic; general quality remains OPEN.


#### Current fixed89 consumer audit and full acceptance preflight

All89 current1dc3a283 native cases were joined anew against their own raw
receipts/traces:88 uniquely identified DecisionCore frames and one prepared
zero-evaluation clean case. Of24 damaged refusals,22 retain the exact complete
expected replacement:21 are SuggestOnly without directional L3/exact L4
permission; one Eligible target is rejected by an ambiguous operation choice.
Two boundary-merge targets are absent as full edits, despite related lexical
surfaces. The three damaged wrong outputs retain their expected alternatives
as SuggestOnly. Two clean wrong outputs remain. Cached field observations can
omit a new frame line; a nearby diagnostic line is not independently sufficient
to certify the current cached frame. The consumer audit does not relabel generic
uncertainty as contradictory evidence or complete ties as lexical Winners.
Exact receipts: `autocorrect-live-ojoasco5/fixed89-after-context-recurrence-path-audit-v1.json`
and `audit-fixed-after-context-recurrence-path.py`, with all five source receipt,
trace and candidate hashes. Actual outputs equal the earlier fixed89 comparison.

The work now enters the original full-proof stage. No new production algorithm
or model change is selected from these residual examples. Retained ambiguity,
lexical coverage and missing independent contextual evidence remain explicit
quality limitations; two-operation birth and boundary merges are separate
mechanisms. This stage may validate or reject the accumulated bounded fixes;
it does not silently declare all remaining restoration debt solved.

First refresh the canonical test inventory through its existing guarded
write-manifest command. Compare against the frozen canonical file and the last
scoped discovery: precisely27 already-reviewed additions, no removed/changed
identity or fixture/toolchain/command drift is allowed. Then create a fresh
remote-only development snapshot, run the mandatory full release script with
its serialized performance lane, and preserve exact binaries/source/graph and
all executed test identities. The Cargo target limit remains12GiB; check it
before and after broad builds. All execution retains the sole dedicated20CPU
remote lease; private native clients retain their previous two-client envelope.
No production install, service restart or input-source mutation is part of
this preflight.

Full quality requires all fixed13 L1 damage classes and clean/coverage/false
certainty/package/RSS/latency dimensions, not only rank or format parity.
Recover exact frozen proof inputs/parameters and compare all required class
metrics. Existing accepted L2 semantic13x100x2 evidence has a separately scoped
historical latency exception; do not convert its formal5.000ms FAIL into PASS
or silently apply that exception to new measurements. Native fixed89 must also
be measured separately across the three existing correction-safety profiles;
only that profile may differ within each paired comparison. The native
Experimental89, independent context16,13 transport/manual controls and quiet80ms
remain distinct denominators. Final release bytes require their own identity
binding to applicable native proof. Physical typing and TD123 DONE stay OPEN.


Full acceptance attempt1 used remote `run-laoCUH`, local `run-x14za786`.
Its architecture refresh passed and all32 nonempty correctness/package targets
completed2711 selected tests with0 failures (2675 correctness,36 package).
The release script nevertheless exited1: the empty TD007 known-failure ledger
still referenced the previous canonical test-manifest SHA. This is a test
metadata contract failure, not a passing full release and not a failed Rust
assertion. Exact raw evidence remains in the remote run's
`td123-full-acceptance-full.log`, `full-test-lanes/` and
`td123-full-acceptance-candidate-identity.json`; total404.974s.
The repair changes only the ledger's manifest SHA. Its0 exclusions and original
historical zero-failure observation are preserved; no new exception is admitted.
A fresh snapshot must pass the mandatory full release route after this repair.

The existing test-lane contract explicitly separates the opt-in performance
lane from ordinary release authority and records historical8/11 PASS,3 blocked
performance assertions. Execute that unchanged lane separately so its full
results remain visible without preventing mandatory lint/build checks. Keep
all thresholds unchanged and never convert performance FAIL to PASS. Any
product/L1-specific budget still requires its own applicable evidence.

The original L1 corpus was recovered locally at
`~/.cache/lay/l1-peak-search-baseline-2026-08-13/corpus-852582.txt`:
14262280 bytes, SHA256
`abb9d71495acc27b7619b24621568ea72263f9bd40201aab9fffa1188fbff8f6`.
All852582 lines match the independently captured pinned-service decoder stream
in exact ID order,0 differences. Use this original corpus for full Gate C;
there is no need to reconstruct, extend or retrain the model inputs.
Final fresh-context combined code review closed in one pass:9/10,H0/M0/L0,
all19 changed Rust files, diff SHA256
`5a214c187c1ac7627d6b115778285b364ef27b9ae01af8d1f523a0ec26ec7028`.
Source review alone does not close heldout/resource/release/physical acceptance.


Final TD123 acceptance now binds the source version1.0.67 before building.
Only Cargo package/lock, extension version/date and VERSIONING publication
metadata change; all697 Rust source hashes and the closed19-file code review
remain unchanged. State is `SOURCE_BUMPED_RUNTIME_STALE`: installed995b remains
the rollback baseline, and no release or runtime authority is claimed. The
mandatory full route uses its default performance opt-out; all11 performance
assertions will run separately with unchanged thresholds. Private source-bump
receipt: `autocorrect-live-ojoasco5/td123-source-version-bump-v1.json`.


### TD123 final 1.0.67 mandatory build acceptance, 2026-09-09

The final immutable IME is `13db86237314df92458259ee439baa0a906e55af8c7657e28c1028ce8f06241e`.
All 697 Rust files match the reviewed implementation; only release metadata
was bumped to 1.0.67. The complete ten-binary release set is saved under remote
`run-tovK3y/release-binaries/` with a SHA-256 manifest. Its CLI reports
`lay 1.0.67`; installation and physical acceptance remain pending.

All 2,711 correctness/package tests passed in the original full invocation.
That invocation exited 1 at the dead-code inventory: nine existing default
diagnostics had moved byte locations, with no change to their messages,
subjects or targets. Both default and research inventories were refreshed
using the existing lint command and independently compared after removing
only byte locations; their 535/359 entries and every other field are unchanged.
No warning, failure exclusion, threshold or production gate was added.

Mandatory acceptance was completed in two evidence segments. The original
prefix contains fmt, architecture, harness self-tests, manifest validation
and the complete 2,711-test PASS. A private continuation executes the exact
unchanged suffix of `scripts/check-lay-full.sh` from its lint stage: both lint
scopes, extension and helper syntax, CLI smoke, all release binaries and diff
check passed. A final default-feature IME build also passed. The failed original
full invocation remains preserved; it is not relabelled as exit 0. Test reuse
is limited to the identical Rust, Cargo, test-manifest, fixture, toolchain and
environment identities after the byte-location-only inventory repair.

Exact evidence: local `~/.cache/lay/development/run-o0dqrl5c/`, remote
`/home/e/projects/lay-development-runner/run-tovK3y/`: `full-test-lanes/SUMMARY.json`,
`td123-full-acceptance-candidate-identity.json` (original FAIL),
`full-acceptance-continuation/` and
`td123-full-acceptance-completed-identity.json` (completed mandatory gates).
Original attempt 426.506 s; continuation including both release builds
472.561 s. Neither is an IME latency measurement. Runtime authority changed:
false. Full heldout, separate performance, three safety profiles, final native
client and physical acceptance are still open.


### TD123 full fixed model and performance evidence, 2026-09-09

Current source: 1.0.67, immutable IME `13db8623`, all 697 reviewed Rust files.
The private proof CLI was built with `lexical-compiler`; no model was trained or
rewritten. All 13 frozen input hashes were verified before and after execution.
The original corpus contains 852,582 exact decoder surfaces; Gate C uses
20,000 damaged cases per class plus every clean surface.

L1 verdict: **PASS_C_QUALITY**, exact quality parity in every one of 13 classes.
Unique top-1 is 208739/210974 (98.940628%)
within the independently unique objective subset. Across all 260,000 damaged
cases the same numerator is 80.284231%; this is a separate denominator.
Complete target retention is 100%; bounded-lattice coverage is
99.868077%. Typed readout: 32,063 Winner,
302 Tied, 227,635 ABSTAIN. Typed false authority and
false singleton are both zero. The legacy compatibility observer separately
reports four false-authority observations; these are not typed terminal or IME
application counts. Clean preservation is 852,582/852,582 (100%),
with zero mutating Winners. The false legacy-V8/A2 flag is inapplicable to this
direct V9 artifact; the overall conjunctive V9 Gate C verdict is PASS.

| L1 damage class | Cases | Unique objective | Unique top-1 | Top-1 % on unique objective | Lattice % | False authority |
|---|---:|---:|---:|---:|---:|---:|
| adjacent_transposition | 20000 | 16786 | 16781 | 99.970213 | 99.970000 | 0 |
| double_substitution | 20000 | 19093 | 18679 | 97.831666 | 100.000000 | 0 |
| extra_letter | 20000 | 19335 | 19190 | 99.250065 | 99.990000 | 0 |
| layout_projection | 20000 | 19196 | 19118 | 99.593665 | 99.215000 | 0 |
| letter_substitution | 20000 | 18304 | 18295 | 99.950830 | 99.980000 | 0 |
| missing_letter | 20000 | 14269 | 14187 | 99.425328 | 99.695000 | 0 |
| non_adjacent_transposition | 20000 | 18426 | 17883 | 97.053077 | 100.000000 | 0 |
| omission_transposition | 20000 | 15107 | 14889 | 98.556960 | 99.620000 | 0 |
| prefix_truncation | 20000 | 12684 | 12492 | 98.486282 | 99.995000 | 0 |
| punctuation_suffix | 20000 | 19998 | 19998 | 100.000000 | 100.000000 | 0 |
| repeated_fragment | 20000 | 19714 | 19507 | 98.949985 | 100.000000 | 0 |
| sparse_multi_omission | 20000 | 14097 | 13777 | 97.730013 | 99.830000 | 0 |
| suffix_truncation | 20000 | 3965 | 3943 | 99.445145 | 99.990000 | 0 |

All class numerators, objective denominators, full retention and terminal
readout counts equal the admitted `49839304...` proof. This proves preserved
packaged lexical quality; it does not assert improved IME Apply rate.

L2: all 2,600 fixed cases, 26 cohort/class groups, one worker. All shared
non-latency class metrics exactly match the accepted semantic V90 baseline.
H/B/S0 remain 1280/1280/1280, raw top-1 1109/2600 (42.653846%),
base raw top-1 267/2600 (10.269231%). All 2,600 independently executed
probed/unprobed readouts match; false singleton, integrity, base-projection
failures and H-to-B/B-to-S0 losses are zero. This is the conditional packaged
productive proof with grounded target/competitor identities, not live IME
accuracy. Formal verdict remains **FAIL_measured_shadow_gates**: maximum class
p99 is 5.276 ms versus the unchanged 5.000 ms threshold. The dated baseline
was 5.286 ms; no new waiver or stronger latency claim is introduced.

| L2 cohort / damage class | Cases | Top-1 % | Top-16 % | False singleton | p99 us |
|---|---:|---:|---:|---:|---:|
| LEMMA_HELDOUT::adjacent_transposition | 100 | 89.00 | 98.00 | 0 | 4360 |
| LEMMA_HELDOUT::double_substitution | 100 | 83.00 | 100.00 | 0 | 4382 |
| LEMMA_HELDOUT::extra_letter | 100 | 98.00 | 98.00 | 0 | 4400 |
| LEMMA_HELDOUT::layout_projection | 100 | 99.00 | 99.00 | 0 | 4224 |
| LEMMA_HELDOUT::letter_substitution | 100 | 90.00 | 98.00 | 0 | 4352 |
| LEMMA_HELDOUT::missing_letter | 100 | 77.00 | 98.00 | 0 | 4284 |
| LEMMA_HELDOUT::non_adjacent_transposition | 100 | 93.00 | 99.00 | 0 | 4334 |
| LEMMA_HELDOUT::omission_transposition | 100 | 82.00 | 99.00 | 0 | 4950 |
| LEMMA_HELDOUT::prefix_truncation | 100 | 99.00 | 99.00 | 0 | 4368 |
| LEMMA_HELDOUT::punctuation_suffix | 100 | 98.00 | 98.00 | 0 | 4710 |
| LEMMA_HELDOUT::repeated_fragment | 100 | 96.00 | 98.00 | 0 | 5276 |
| LEMMA_HELDOUT::sparse_multi_omission | 100 | 78.00 | 100.00 | 0 | 4317 |
| LEMMA_HELDOUT::suffix_truncation | 100 | 27.00 | 97.00 | 0 | 4109 |
| SEEN_EXACT::adjacent_transposition | 100 | 90.00 | 100.00 | 0 | 706 |
| SEEN_EXACT::double_substitution | 100 | 95.00 | 100.00 | 0 | 611 |
| SEEN_EXACT::extra_letter | 100 | 100.00 | 100.00 | 0 | 702 |
| SEEN_EXACT::layout_projection | 100 | 98.00 | 99.00 | 0 | 717 |
| SEEN_EXACT::letter_substitution | 100 | 97.00 | 100.00 | 0 | 719 |
| SEEN_EXACT::missing_letter | 100 | 92.00 | 100.00 | 0 | 704 |
| SEEN_EXACT::non_adjacent_transposition | 100 | 97.00 | 100.00 | 0 | 751 |
| SEEN_EXACT::omission_transposition | 100 | 95.00 | 100.00 | 0 | 531 |
| SEEN_EXACT::prefix_truncation | 100 | 100.00 | 100.00 | 0 | 644 |
| SEEN_EXACT::punctuation_suffix | 100 | 100.00 | 100.00 | 0 | 535 |
| SEEN_EXACT::repeated_fragment | 100 | 100.00 | 100.00 | 0 | 742 |
| SEEN_EXACT::sparse_multi_omission | 100 | 88.00 | 99.00 | 0 | 550 |
| SEEN_EXACT::suffix_truncation | 100 | 33.00 | 99.00 | 0 | 654 |

The separate performance lane executed all 11 contracts: **10 PASS, 1 FAIL**.
The remaining unique-prefix cold-materialization test measured 301.738 ms
against its debug-build 50.000 ms bound; its release bound is 1.500 ms and
was not measured by this debug test. No retry or threshold change was used.
The other ten contracts, including canonical L2 p99 (2.050 ms), exact-English
RSS, L3 hot context, preedit and nonblocking key-thread submission, passed.
The existing test-lane document already records this cold-prefix route as
blocked performance debt and separates the lane from ordinary release
authority. This result remains red and is not converted into a full performance
PASS. The actual final native80ms gate and physical typing remain separate.

Package: 77,962,328 bytes (74.351 MiB),
unchanged SHA `bf5a1619...`; all package/dependency gates applicable to V9 pass.
The proof process, not serving runtime, measured L1 peak RSS 768,440 KiB and
L2 peak RSS 285,968 KiB. Proof CPU averages were 1643% (20-worker L1) and 99%
(one-worker L2). Wall times were 418.50 s and 69.21 s; these are throughput
measurements, not per-keystroke latency. Cargo target ended at 11,354,185,728
bytes below its 12,884,901,888-byte limit. Final native process/cgroup measurements
are still pending. Runtime authority changed: false; no package or receipt
was installed or modified.

Exact local evidence: `~/.cache/lay/development/run-o0dqrl5c/`:
`td123-full-model-proof-comparison-v1.json`, `full-model-proof/receipt.json`,
`full-model-proof/l1-full.json`, `full-model-proof/l2-semantic-full.json`,
the corresponding `.time.txt` files and `full-model-proof/performance-test-lanes/`.
Remote owner: `/home/e/projects/lay-development-runner/run-tovK3y/`.


### TD123 final native profiles and installed runtime, 2026-09-09

The final acceptance candidate is IME SHA256
`13db86237314df92458259ee439baa0a906e55af8c7657e28c1028ce8f06241e`, version
1.0.67. It matches all 697 reviewed Rust source hashes and the completed
mandatory build identity recorded above. All native runs retain the frozen
fixtures, driver inputs, resource envelopes, package identities, key deadlines
and learning order. The final service binary changes from frozen `1ced...` to
`db825d2f...`; all other eight dependency identities are byte-identical.
`td123-final-candidate-inputs-v1.json` binds the two service manifests explicitly.

**Fixed89 profiles.** Each paired profile compares installed baseline `995b6093`
with final `13db8623` using the same frozen L1.1 service. Only the selected
`correction_safety` differs across profiles. The 89 fixtures contain 47 dirty
and 42 clean cases. The independent comparator recomputed 178 statuses per
profile, 534 total, and found no lost correct output, new false output or fixture
group regression. Fixture-manifest SHA256:
`ae1bf941a5fa3126fc3548bc9de613977f074a8c438d0c498bcbba800607532e`.
Actual per-engine configuration hashes were also read back from the launched
sandbox; a profile label alone was not accepted as configuration proof.

These are ordered native Cyrillic fixtures with Space after the prepared
result, not the L1 heldout damage-class population, Latin/mixed-input coverage
or an ordinary-cadence proof. The aggregate rows below overlap the group rows;
do not sum them into a larger denominator. Existing wrong outputs remain
visible. A class-level no-regression verdict is not universal correctness.

| Profile | Fixture group | Cases | Baseline correct % | Final correct % | Final correct | Final wrong output |
|---|---|---:|---:|---:|---:|---:|
| strict | all_dirty | 47 | 31.914894 | 31.914894 | 15 | 0 |
| strict | all_clean | 42 | 97.619048 | 97.619048 | 41 | 1 |
| strict | clean_missing_letter_control | 4 | 75.000000 | 75.000000 | 3 | 1 |
| strict | clean_repeated_letter_control | 4 | 100.000000 | 100.000000 | 4 | 0 |
| strict | clean_valid_word | 34 | 100.000000 | 100.000000 | 34 | 0 |
| strict | context_fixture | 6 | 0.000000 | 0.000000 | 0 | 0 |
| strict | missing_letter | 8 | 37.500000 | 37.500000 | 3 | 0 |
| strict | repeated_letter | 4 | 0.000000 | 0.000000 | 0 | 0 |
| strict | restoration_regressions | 24 | 29.166667 | 29.166667 | 7 | 0 |
| strict | transposition | 5 | 100.000000 | 100.000000 | 5 | 0 |
| normal | all_dirty | 47 | 40.425532 | 40.425532 | 19 | 3 |
| normal | all_clean | 42 | 92.857143 | 97.619048 | 41 | 1 |
| normal | clean_missing_letter_control | 4 | 75.000000 | 75.000000 | 3 | 1 |
| normal | clean_repeated_letter_control | 4 | 75.000000 | 100.000000 | 4 | 0 |
| normal | clean_valid_word | 34 | 97.058824 | 100.000000 | 34 | 0 |
| normal | context_fixture | 6 | 0.000000 | 0.000000 | 0 | 0 |
| normal | missing_letter | 8 | 62.500000 | 62.500000 | 5 | 0 |
| normal | repeated_letter | 4 | 0.000000 | 0.000000 | 0 | 1 |
| normal | restoration_regressions | 24 | 37.500000 | 37.500000 | 9 | 2 |
| normal | transposition | 5 | 100.000000 | 100.000000 | 5 | 0 |
| experimental | all_dirty | 47 | 40.425532 | 42.553191 | 20 | 3 |
| experimental | all_clean | 42 | 92.857143 | 95.238095 | 40 | 2 |
| experimental | clean_missing_letter_control | 4 | 75.000000 | 75.000000 | 3 | 1 |
| experimental | clean_repeated_letter_control | 4 | 100.000000 | 100.000000 | 4 | 0 |
| experimental | clean_valid_word | 34 | 94.117647 | 97.058824 | 33 | 1 |
| experimental | context_fixture | 6 | 0.000000 | 0.000000 | 0 | 0 |
| experimental | missing_letter | 8 | 62.500000 | 62.500000 | 5 | 0 |
| experimental | repeated_letter | 4 | 0.000000 | 0.000000 | 0 | 1 |
| experimental | restoration_regressions | 24 | 37.500000 | 41.666667 | 10 | 2 |
| experimental | transposition | 5 | 100.000000 | 100.000000 | 5 | 0 |

**Retained harness failure and bounded repair.** The original final batch is
`EXECUTED_WITH_FAILURES`, not PASS: experimental candidate shard 1 stopped after
12 of 18 cases when its concurrent JSONL snapshot ended inside a UTF-8 codepoint.
The other 24 shards completed. The driver already discarded incomplete final
JSONL records, but decoded the bytes too early. This was a reader failure,
not an observed runtime restoration regression.

Before editing the private reader, two options were compared: discard bytes
only after the last newline before strict decoding (9/10, selected) or retry
a full snapshot (5/10, timing-dependent). No production or original driver was
changed. The new private driver reconstructs its predecessor byte-for-byte
when the helper and two call-site changes are reversed. The remote proof
checked 103 byte cuts, reproduced 12 old-reader failures, preserved every
complete record, and still rejected complete invalid UTF-8 and invalid JSON.
Only the incomplete 18-case shard was executed again, once. The replacement
comparator admits this proven reader normalization and the pre-existing
mtime-only L3 manifest stamp normalization; fixtures, handlers, learning and
all behavior-affecting identities remain unchanged. The original failed
receipt and partial observations are retained; no retry-until-green occurred.
A preceding shell/guard stdin attempt executed no child and produced no receipt;
it was verified empty before the file-based recovery executed the proof.

**Final native transport and cadence.** All 13 unchanged private IBus controls
passed against the final IME and final L1.1 service: first-word US 1, RU 1,
manual toggle 3, lifecycle 3, restoration 5. All private-client cleanup checks
passed. This proves those client paths; real keyboard Double Shift remains a
separate acceptance item.

The quiet 80 ms experiment used eight paired runs with the same final IME and
frozen versus final L1.1 service. Each service variant restored the two supplied
dirty fixtures 8/8 each and preserved the three clean fixtures 24/24. All 80
case outputs exactly equal expected surfaces. The ordered phrase and learning
state are fixed; these are repeated executions, not 16 distinct error words.
The original key cadence, post-exact-ready startup, all additional inputs and
absence of all five detailed stderr flags were verified. Native13 and quiet
cadence elapsed 13.086 s and 47.744 s in the pipeline; these are run durations,
not individual correction latency. The native phase did not retrain models.

**Resources.** Only the separate 13-control run was sampled every 20 ms.
Five IME process identities and five client cgroups were observed. Maximum
process VmHWM was 359504 KiB (351.078 MiB); maximum cgroup memory.peak was
362614784 bytes. The sum of observed cumulative cgroup CPU usage was
14781445 microseconds; these cgroups also contain client work and are not an
IME-only CPU denominator. Observed swap, OOM and OOM-kill counters were zero.
The observer can miss final exit intervals; these are observed high-water
marks, not an upper-bound guarantee. The quiet cadence run had no observer
sampling overhead. The two earlier red performance measurements remain red.

**Verified local installation.** At 2026-09-08T23:27:24.228290+00:00 (local
2026-09-09), the complete ten-binary set from remote `run-tovK3y/release-binaries/`
was installed from its identical local copy, without rebuilding. The existing
installer verified the already canonical L2 package; it copied the same bytes.
The transaction backed up all ten old binaries and extension/cache files,
verified a temporary native-RU handoff, restarted only the four Lay-owned
processes and restored the selected `lay-ime-ru` source. It preserved global
ibus-daemon PID 4715 and start ticks 2261. Input sources remain exactly
`[('ibus', 'lay-ime-us'), ('ibus', 'lay-ime-ru')]`.

| Loaded component | PID | Start ticks | SHA256 prefix |
|---|---:|---:|---|
| IME | 3291789 | 53137705 | 13db8623 |
| L1.1 service | 3291614 | 53137510 | db825d2f |
| Daemon | 3291774 | 53137703 | 4e01703e |
| L3 watcher | 3291775 | 53137704 | c75f0944 |

All ten installed files and four `/proc/PID/exe` hashes match the release
manifest. Source/CLI/loaded GNOME extension report 1.0.67. Config SHA256 remains
`5887b077e716357cd0a622d16feda7147ff50c1f5ad2bee136f970fa095a9479`;
Experimental and precognition settings were preserved. All eight immutable
model/admission dependencies remain unchanged. Service health reports ready,
852582 terminals and 77962328 package bytes. Ping/Focused/InputState are recorded
as liveness only and do not prove the physical behavior.

Runtime authority changed: **true for this installation**; the preceding
private proofs and final graph refresh do not themselves change authority.
Rollback is the complete snapshot at
`/home/ubu/.local/state/lay/release-backups/1.0.67-td123-1p1e47xu/`.
Status is **INSTALLED_VERIFIED_PHYSICAL_PENDING**. The user has already been
asked to type the ordinary-tempo phrase and confirm physical Double Shift;
no response has been received. Do not repeat the installation or promote this
bounded improvement to general TD-123 quality/DONE/publication acceptance.
General unseen-word restoration, full native Latin/mixed-input coverage,
physical behavior and the two red performance obligations remain unproven or
open as specified above. No blanket waiver is introduced.

Exact evidence, local root `~/.cache/lay/development/`:

- `autocorrect-live-ojoasco5/td123-final-candidate-inputs-v1.json` and
  `td123-fixed-profile-{strict,normal,experimental}-comparison-v2.json`;
- `autocorrect-live-ojoasco5/td123-reader-repair-inputs-v1.json`,
  `td123-reader-repair-proof-v1.json`, `autocorrect-fixed-driver-v11.py` and
  `fixed-fixtures-v11-profiles-candidate-experimental-1/`;
- `run-o0dqrl5c/final-native-acceptance/receipt.json` (original failed batch),
  `run-o0dqrl5c/td123-release-native-controls.json`,
  `run-o0dqrl5c/td123-release-native-resources.json` and
  `autocorrect-live-ojoasco5/td123-release-cadence-summary-v1.json`;
- `run-o0dqrl5c/installation-1.0.67.json` and `release-binaries/`.

The corresponding remote roots are
`/home/e/projects/lay-development-runner/run-tovK3y/` and
`/home/e/projects/lay-development-runner/autocorrect-ojoasco5/`.
Raw typing logs stay private. Source documentation and the architecture graph
are updated after these measurements; accepted binaries are not rebuilt for
prose-only changes.

### TD-121 C17 retained-output Clippy diagnosis (2026-09-13)

Tested: the default-feature `cargo clippy --locked --all-targets` contract from
the frozen C16 source, with warnings denied and dead-code diagnostics excluded.
Measured fact: Clippy reported one `clippy::question_mark` mechanism at
`src/nanda_wave/l2/layout_adapter.rs:377`; the duplicate rendered diagnostic is
the same library source compiled for the library and library-test targets.
Replacing the `let Some(memory) = surface_motif_memory() else { return None; }`
form with `let memory = surface_motif_memory()?;` preserves the identical
`None` propagation when motif memory is unavailable. It does not change
candidate authority, rank, evidence, safety effects, or any deadline.

Not tested by C17: manifest discovery, semantic tests, research-feature Clippy,
full release acceptance, clients, installation, or physical delivery. Verdict
scope: **ACCEPTED MECHANICAL LINT CORRECTION ONLY**. Exact receipt:
`/home/ubu/.cache/lay/development/release-1.0.72-td121-prepared/c17-diagnostic/RESULT.json`
(SHA-256 `70687be0554d15a050e2868874a7017bec5e0234f0a73728a8f8a6e76af63423`).
Runtime authority changed: **false**.

### TD-121 C20 terminal-delivery release and installation (2026-09-13)

Tested: the final C20 source through the complete 1.0.72 release runner, four
isolated final-byte client cells, an owned GTK entry smoke, and the reviewed
transactional installer. Measured facts: all 13 release commands passed; changed and full gates each passed 2,807/2,807
with 11 intentional performance skips; compiled-receipt verification passed. The
preceding focused 519/519 proof on the same runtime source passed separately. The
GTK surfaces were exactly `привет`, `ghbdtn`, and layout projection `ghjdthrf` after autocomplete with expected manual-toggle counts
1/2/1. The extra-toggle case forwarded zero queued keys, spaces, or boundaries.

C20 contains two GTK `SurroundingText` replay fixes. First, valid exact-replay
scope is captured before Reset clears context. That captured scope suppresses only
the redundant epoch publication; negative, expired and stale scopes retain their
rejection. Fresh matching `SurroundingText` then permits every intended Backspace.

```text
valid exact-replay scope -> capture before Reset -> suppress redundant epoch
-> fresh matching SurroundingText -> intended Backspaces accepted
```

Separately, after completed replay `VisibleTailV3` may report the qualified
observed `UnknownStart` suffix even after the handoff flag is consumed. The queued
`ManualToggleV3` then settles and toggles once.

```text
completed replay -> qualified observed UnknownStart suffix
-> VisibleTailV3 -> queued ManualToggleV3 -> one settled toggle
```

Terminal single erase/commit is a distinct route and has no GTK quarantine.

The release result SHA-256 is
`02458047a539fb85be82b301fd1cf38b19af1f71241f34260713cf5c5dd90534`.
The GTK receipt is
`/home/ubu/.cache/lay/development/release-1.0.72-td121-c20-20260913/gui-smoke-owned-c20/RECEIPT.json`,
SHA-256 `27bf83fccd5a15e552dabd9afeada8f1bacfd9038b8f9a0046f2cb3d0fcd1eaf`.
Its `case_results_sha256` is the canonical v3 projection hash over selected case
identity, exact output and semantic trace fields; it is intentionally distinct
from the raw receipt-file hash.

Release 1.0.72 is installed. All ten installed artifacts and the four loaded
owners match the C20 manifest, and the loaded extension reports 1.0.72. The
transaction preserved global IBus identity, configuration, input sources, model
dependencies, journals and learner state. Installation receipt:
`/home/ubu/.cache/lay/development/release-1.0.72-td121-c20-20260913/installation-1.0.72.json`,
SHA-256 `ca7b0cb622f862cdb9a51678e640d27953fe798f3b37f291bdebce4f5e4735a4`.
Runtime authority changed: **true**, by installation of the accepted C20 bytes.

Not tested: human physical-keyboard behavior after installation, general heldout
answer quality, or TD-123 per-class quality gates. Verdict scope:
`INSTALLED_VERIFIED_PHYSICAL_PENDING`; those open denominators are not inferred
from release, client, or GTK delivery success.
