# V10 E1 Traversal D2 T-CAP Salvage Interpretation Correction V3

Date: 2026-08-25

## Scope

This correction preserves the immutable V1 probe and V2 offline-salvage
receipts. V2 ran all four admitted readers successfully but published
`BLOCKED_TCAP_EVIDENCE` because its parser required explicit zero-valued
`freq` and `precise_ip` fields. V3 may reinterpret only the sealed reader
outputs and freeze the exact remote yes ELF bytes needed for PT_LOAD geometry.

```text
new perf reader invocation       NO
perf record/stat                 NO
event open                       NO
yes execution                    NO
D2 subject                       NO
```

Pinned V2 evidence:

```text
local V2 receipt       805fb3316b6a64b507e94b515e533ecbf18852d61d8b20d86cadfd5ea0039ef9
remote V2 receipt      7b2348e795b5392449871056983b129dd2cfcf02f0584b2bd7cda0eb046abfff
remote manifest        23cbcaeb9021b8693ccbcdbbe1df2efa2b6d6883dfa806656926bc29bd892764
evlist output          8c281ab1b257e02c31ad8b5d42da3127dc34bad85dbc31656f9bd8ecd6c84b43
sample output          997d8c64b482509959b0f728e8c12df16a0e994aff1e6386d0cd3a42b28e7f4a
raw-record output      fef73b5591319fdf137fea05a8839fe1691f1ab4c8ddd4c7886e7fe9ebc647a5
build-ID output        a2c4d26d06f0d008176a2774611cef6ee840b1ecda2f12198bb12db3db16e489
```

## Zero-Field Semantics

The exact record command used `--count 100000`, not frequency mode, and the
sealed task-clock evlist line contains:

```text
type: 1
config: 0x1
{ sample_period, sample_freq }: 100000
exclude_kernel: 1
```

It contains no `freq` flag and no `precise_ip` flag. For this fixed-count
task-clock event, V3 interprets absent optional nonzero flags as their
zero-valued `perf_event_attr` defaults:

```text
freq        = 0
precise_ip  = 0
```

This rule is scoped to the exact sealed task-clock line and exact record
command. It cannot be reused to infer missing `precise_ip=2` for future precise
events; I-CORE and I-ATOM must report and validate precision independently.

## DSO Canonicalization

The sealed `perf script -F cpu,event,ip,dso` rows render DSO values in one pair
of parentheses, for example `(/usr/bin/yes)`. V3 strips exactly one complete
outer pair before path identity comparison. Any unmatched, nested or different
wrapper is rejected. No basename or suffix-only identity is accepted.

## Exact ELF Closure

The V1 record and V2 build-ID output identify remote `/usr/bin/yes` as:

```text
SHA-256  ee431b97fb62f59ee94fa698dbc98971001bbb1cbd9c5e32ce4ab4c5530924d8
Build ID 8c99ebc2c856857219acc612c2d9be3172b74be5
```

V3 may copy the current remote file into a new correction evidence directory
only after its SHA-256 matches exactly. The copied bytes are sealed before
interpretation and must reproduce the same Build ID and ET_DYN executable
PT_LOAD geometry. A mismatch is `BLOCKED_PROVENANCE`; no fallback ELF is used.

## Recovery Conjuncts

V3 publishes `T_CAP_RECOVERED_FROM_SEALED_EVIDENCE` only when all sealed facts
recompute:

```text
reader invocations in V2                         4
reader stderr bytes                              0 for all four
sample count                                     4578
sample CPUs                                      {0}
sample event                                     task-clock:u
type / config / fixed period                     1 / 1 / 100000
freq / exclude_kernel / precise_ip               0 / 1 / 0
lost / throttle / unthrottle                     0 / 0 / 0
yes Build ID                                     exact
maps-before == maps-during for yes               true
canonical yes sample DSO                         /usr/bin/yes
every yes sample IP has one executable map       true
every map has one executable PT_LOAD             true
every normalized IP is inside that PT_LOAD       true
```

Any failed conjunct is terminal and cannot be repaired by relabeling samples.
V1 and V2 verdicts remain unchanged.

## Decision Boundary

V3 PASS admits only creation of the precise-only implementation preflight from
shutdown repair V2. It does not admit a precise record by itself and does not
admit D2 implementation, build, subject execution, bucket mapping, full B,
V12, runtime integration or deployment.
