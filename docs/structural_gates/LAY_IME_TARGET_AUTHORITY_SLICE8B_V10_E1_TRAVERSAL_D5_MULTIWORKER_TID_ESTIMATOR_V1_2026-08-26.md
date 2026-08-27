# D5 - multiworker TID estimator V1

Date: 2026-08-26

## Scope and immutable history

D5 is a new multiworker measurement namespace. It follows the terminal D4
single-worker result and does not reopen D2, D3, or D4:

```text
D2 terminal receipt              75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608
D3 terminal receipt              7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299
D4 terminal receipt              f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b
D4 verdict                       D4_SINGLE_ESTIMATOR_PASS
D4 U3/T3 markers                 consumed; never reused
D4 accepted samples              66543
D4 sampled CPU/edge              25.20277605266102 ns
D4 paired delta                  3.343809548379481%
```

D4 established valid single-worker attribution. It did not measure the exact
twenty-worker fixed or reversed component routes and therefore did not explain
the frozen concurrency inflation. D5 creates only these new routes:

```text
U4-FIXED
T4-FIXED
U4-REVERSED
T4-REVERSED
```

No historical marker, state, receipt, ELF, bucket map, package, schedule,
sysctl, installed runtime, or production source may be modified. D5 grants no
optimization or deployment authority.

## Reused exact evidence

```text
D2 ELF SHA-256
  bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178
D2 ELF size                      317706232
D2 ELF mode                      0555
D2 Build ID
  eb951f1a7526a9f1cb365040c10989aa5d3fc50f
D2 bucket-map SHA-256
  2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846
D2 bucket-map size               390324
D2 bucket-map mode               0444
D2 build audit                   4e19e2e806e2b3f04f8c0286a63f64631e7cf6ff143d9a90da4036654bc8339c
D2 bucket-map audit              8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7
D2 parity                        d2519797c6b21976dc6246830e187391a344acb9ec63a7b5a7816161cf306f74
D4 U3 receipt                    db2ba1b3d4e11ac2c4edb24e382f93d282b1b5d605fcbb1636f5e346030dc000
D4 T3 receipt                    dd4e3b7bb49d368fe1461c36fda0968af629293e801500770ca9dc3715a96f09
D4 terminal audit                f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b
```

The target remains `e-MEGA-MINI-M1-13th`, CPUs `0-19`, core PMU `0-11`, atom
PMU `12-19`, and `perf_event_max_sample_rate=8000`. D5 does not tune them.

## Markerless bootstrap

D5 repeats the corrected D4 admission architecture for its disjoint parent.
The namespace is first created without scientific markers. The parent and
state ancestors are root-owned `0755`; the exact future subject directory is
owned by UID `e`. An actual helper running as `e` must traverse every ancestor,
read and hash the exact D2 ELF and map, create/write/fsync/rename/reopen/read/
unlink fixed probe bytes, and publish sealed evidence. An independent local
auditor must copy the exact bootstrap with real `scp` as `e`, verify manifests
byte-for-byte, and repeat a live read-only projection.

Any UID, ancestor, SSH, SCP, evidence-mirror, or immutable-input failure is
`BLOCKED_BEFORE_MARKERS`. Only the independent bootstrap PASS admits one
atomic creation of four exact `0400` markers. A second independent read-only
audit must pass before U4-FIXED. Marker creation is never repeated.

## Exact subject envelope

All four routes execute the audited D2 ELF and exact test:

```text
nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty
--ignored --nocapture --test-threads=1
```

The subject uses `LAY_V10_D1_CPUS=0,1,...,19` for FIXED and
`LAY_V10_D1_CPUS=19,18,...,0` for REVERSED. It must report:

```text
workers                         20
queries                         382
warmup bursts                    1
measured rounds                 20
start/end barriers              21 / 21
worker affinities               exact singleton mapping
worker migration deltas          0 for all 20
errors / unresolved              0 / 0
component records             7640
examined edges/round       25145756
```

The process is whole-process wrapped. No `--pid`, attach delay, signal
lifecycle, timestamp subtraction, or warmup subtraction is permitted.

## U4 denominator

Each U route runs once with no perf or PMU. It parses the traversal phase's
`CLOCK_THREAD_CPUTIME_ID` values from all 7,640 component records:

```text
U4 denominator edges = 20 * 25145756 = 502915120
U4 CPU/edge = summed measured traversal thread CPU ns / 502915120
```

U4 is the paired no-perf denominator for the matching T4 route. D2 U-FIXED,
D2 U-REVERSED, and D4 U3 remain immutable comparison evidence; they are not
substituted for a fresh paired denominator.

## Worker TID reconstruction

The accepted TID set is preregistered from raw perf lifecycle records, not
from sample IPs, timestamps, CPU heuristics, or post-result selection.

The exact executable MMAP2 record identifies one subject process leader. The
subject thread graph must then contain:

```text
one direct libtest child of the subject leader
one unique parent TID with exactly 20 direct child thread FORK records
that parent is the libtest child
twenty distinct worker TIDs in the same subject TGID
no worker has a subject-thread child
one EXIT record for every worker TID
all worker TIDs have at least one sample
each worker TID's samples occur on exactly one CPU
the twenty singleton sample CPUs form exactly {0..19}
```

The process leader, perf/sudo/env/taskset processes, libtest parent TID, and
all other TIDs are excluded from scientific attribution. COMM records are
retained and checked for lifecycle consistency, but worker identity is owned
by the exact FORK graph because unnamed Rust scoped threads may inherit a
truncated COMM without emitting a unique name.

Missing, duplicate, ambiguous, cyclic, incomplete, or multiply valid thread
graphs are `BLOCKED_PROVENANCE`. No fallback to all subject TIDs, all CPUs, or
all D2 IPs is permitted.

## T4 estimator and event

T4 uses exactly one inherited event:

```text
task-clock:u
period                         200000 ns
freq                                0
exclude_kernel                      1
precise_ip                          0
inherit                             1
```

The scientific stream is exactly:

```text
exact D2 Build ID
AND unique normalized D2 mapping
AND TID in preregistered twenty-worker set
AND normalized IP in sealed traversal machine range
```

All non-worker samples and worker samples outside traversal remain raw
evidence. There is no timestamp filter and no attempt to remove warmup.

```text
T4 denominator edges = 21 * 25145756 = 528060876
T4 CPU/edge = accepted traversal samples * 200000 / 528060876
```

## Hard gates and dispatch

U4 dispatch priority:

```text
0 provenance -> BLOCKED_PROVENANCE
1 thermal    -> BLOCKED_THERMAL
2 semantic   -> BLOCKED_SEMANTIC
```

T4 dispatch priority:

```text
0 provenance      -> BLOCKED_PROVENANCE
1 thermal         -> BLOCKED_THERMAL
2 capability      -> BLOCKED_CAPABILITY
3 bucket_map      -> BLOCKED_BUCKET_MAP
4 perturbation    -> BLOCKED_PERTURBATION
5 sample_coverage -> BLOCKED_SAMPLE_COVERAGE
```

Each T4 route requires:

```text
event identity and period                  exact
perf record / mandatory readers             1 / 4
lost                                         0
throttle / unthrottle                        0 / 0
adaptive period                              absent
sample-rate before / after                   8000 / 8000
thermal throttle-counter drift               0
subject semantics and structure              PASS
twenty-worker TID graph                      unique and complete
worker sample CPU closure                    exact {0..19}
Build ID / PIE normalization                 exact / unique
machine-byte mismatches                      0
accepted traversal samples                   >=50000
every worker accepted samples                >0
UNATTRIBUTED accepted traversal samples      <=5%
T4 sampled CPU/edge vs paired U4              <=5%
```

Unknown predicates, incomplete observations, ambiguous dispatch, conflicting
causes at one priority, or controller failure after marker consumption select
`BLOCKED_PROVENANCE`. Every route failure is terminal and permits no rerun.

## Order and claim boundary

```text
D5 paper + structural gate + implementation preflight
  -> markerless bootstrap and UID proof
  -> independent live/scp audit
  -> create four markers once
  -> independent marker audit
  -> U4-FIXED once
  -> T4-FIXED once only after U4-FIXED PASS
  -> U4-REVERSED once only after T4-FIXED PASS
  -> T4-REVERSED once only after U4-REVERSED PASS
  -> independent terminal audit
```

The only positive verdict is:

```text
D5_MULTIWORKER_ATTRIBUTION_PASS
```

The terminal receipt must publish per route bucket counts, percentages and
sampled ns/edge, paired U/T delta, fixed/reversed inflation versus D4, and the
per-bucket sampled ns/edge delta versus D4. Bucket delta sums must reconcile
exactly with each total sampled inflation apart from floating-point rendering.

A PASS establishes primary task-clock attribution of the multiworker
inflation and admits only a separate paper optimization decision. It does not
authorize a rewrite, new build, integration, install, restart, deployment, or
runtime authority change.
