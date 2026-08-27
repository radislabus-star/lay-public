# M3 Typed View End-To-End V8

Date: 2026-08-27

Status: `DESIGN_FROZEN_BEFORE_IMPLEMENTATION`

## Scope

This paper consumes the sealed test-only actual-owner parity result. It admits
one test-only generation-owner implementation and one physical end-to-end proof
on the established mini-PC. It does not install a sidecar, edit the production
bridge or cache, reload an installed package, start a daemon, change runtime
authority or promote the typed view to production.

Immutable predecessors:

```text
M3 source/lifetime decision SHA-256
  e7b0f66170776677c2b153254aa01a303fdf8538aec273678196dec723715b24
V7 execution manifest SHA-256
  639e2ba15660bbdccb2b098e3419f7045aaf0fe328a6d0e0283fba1eba5307a1
V7 execution admission SHA-256
  0236395f870d984b4141b4163fd48966b391783d2d1d3dba7400781cb9f9fd4d
V7 owner receipt SHA-256
  aa2cfc21f7f97d8502a459aeb311722836ee62c4e4724e4ce104fbadd19ad829
V7 terminal receipt SHA-256
  8a703dd385e7787816afa016e034284834540b53ed794d2544f638dcbd421e83
V7 terminal verdict
  M3_ACTUAL_OWNER_PARITY_PASS
```

## Frozen Inputs And Host

The scientific execution host is:

```text
host               e-MEGA-MINI-M1-13th
kernel             6.8.0-124-generic
logical CPUs       20
measurement CPU    0
cargo              1.97.1 c980f4866
rustc              1.97.1 8bab26f4f68e0e26f0bb7960be334d5b520ea452
LLVM               22.1.6
target             x86_64-unknown-linux-gnu
```

The build also occurs on this host because its glibc is older than the local
development host. A local test binary is not a physical substitute.

Scientific inputs remain:

```text
V13 package         140,556,462 B / cce259fe0ce5...
V7 fixed proof        1,606,189 B / 33fded73e13f...
Productive V90       17,309,944 B / 40fb6a9f0d92...
L1.1 V9              77,962,328 B / bf5a1619a890...
fixed cases                     382
damage classes                   13
```

The Productive and L1.1 files may be copied into the isolated experiment
namespace before marker creation. Their bytes are inputs, not installed
runtime changes. A deterministic test-only L1.1 receipt may project those exact
bytes to experiment-local paths; its artifact and proof hashes must remain
exact and it cannot replace the active receipt.

## Selected Generation Owner

V8 may add a private `#[cfg(test)]` owner in `v13_typed_peak.rs` only:

```text
validated current LAYV13D3 sidecar
        -> safe TypedMaterialization
        -> immutable Arc<ExactGeneration>
        -> generation slot
        -> request borrows Arc
```

Each immutable generation binds:

```text
monotonic owner generation
V13 package SHA and bytes
current full sidecar SHA
sidecar payload SHA
Phase 7D semantics SHA
state / edge / root / symbol identities
typed payload bytes
```

Exactly one typed materialization is allowed per published sidecar generation.
There is no query-local materialization, target-conditioned identity, fallback
byte/typed race, unsafe reinterpretation or independently reloadable authority.

## Request Denominator

The measured closed call is:

```text
Phase7dCertificateOracle construction
  -> typed target-blind exact search
  -> exhaustive result and certificate construction
  -> ExactPeakBirthEnumerationV1
  -> prepare_live_productive_v1_field_with_exact_peaks
  -> PreparedCanonicalTokenField
  -> materialize_live_productive_v1_field
  -> exact candidate/gate observation
```

One full 382-case warmup occurs before measurement. Four measured rounds use
the fixed order `FORWARD, REVERSED, FORWARD, REVERSED`, producing `1,528`
single-request samples. No request includes sidecar construction, typed
materialization, package loading, queue wait, IPC, daemon work or PSS-helper
work.

Timers are frozen as:

```text
search time
  oracle construction through exhaustive typed-search closure

total material time
  oracle construction through materialized candidate/gate observation

owner prepare and final materialization
  reported separately inside total material time
```

The proof reports pooled and per-round p50/p99/max. Hard latency gates use the
maximum per-round p99, not only the pooled value:

```text
single-request search p99          <= 3,000 us
single-request total material p99  <= 5,000 us
maximum query scratch              <= 512 KiB
unresolved / overflow              0 / 0
```

## Semantic And Authority Gates

Every measured request must retain the V7 owner contract:

```text
candidate mismatches                    0
certificate mismatches                  0
structured certificate mismatches       0
lattice marker mismatches                0
emitted surface mismatches               0
gate mismatches                          0
certificate collisions                  0
capacity failures                        0
```

Exact-only candidates remain `SuggestOnly` unless the unchanged existing owner
independently admits the same surface. V8 cannot grant winner, mutation,
DecisionCore, verifier or feedback authority.

## PSS And Residency Gate

The authoritative proof creates one temporary current sidecar file in the
experiment namespace and loads it through the existing validated mmap loader.
Two helper processes are measured before and after loading the mmap sidecar and
building one typed view each. They hold both generations while the controller
reads `/proc/<pid>/smaps_rollup`.

This isolates sidecar plus typed-view residency from package parsing and test
binary startup. The proof reports per-process and aggregate PSS before/after,
mapped sidecar bytes, private typed bytes and process RSS/HWM.

Hard gates:

```text
sidecar bytes                         <= 32 MiB
typed owned payload per process       <= 4 MiB
aggregate two-process PSS delta       <= 40 MiB
typed payload exact                    3,689,628 B
PSS helper failures                             0
```

The temporary sidecar is evidence only. It is removed after the proof or
retained solely inside the sealed experiment evidence on failure; it is never
published under an installed model path.

## Reload, Readers And Rollback

The test owner exercises three state transitions outside the latency region:

```text
publish generation A
  -> concurrent readers borrow the same Arc A

validate and publish generation B
  -> new readers borrow B
  -> an A result cannot commit after B publication
  -> the held A Arc remains memory-safe until its reader exits

injected generation-C construction failure
  -> no publication
  -> B remains current
  -> generation counter and identity remain unchanged
```

Required observations:

```text
reader identity mismatches             0
mixed generation observations          0
stale A commits                         0
stale A cancellations                   1
failed-build publications               0
rollback identity mismatches            0
typed materializations                  2 across two published generations
per-request typed materializations      0
```

This proves the selected test-owner mechanism only. It does not execute or
modify the installed Productive reload function.

## Build And Execution State Machine

The remote namespace starts absent. After read-only bootstrap and source/input
closure, exactly two one-shot markers may be created:

```text
build.available
e2e.available
```

Sequence:

```text
implementation preflight PASS
  -> test-only source implementation
  -> local compile/static checks only
  -> independent remote execution admission
  -> isolated source/input bootstrap
  -> bootstrap audit
  -> create two markers
  -> consume build.available before one Cargo --no-run
  -> seal and audit one release test ELF
  -> quiet-host execution preflight
  -> consume e2e.available before one exact test action
  -> V8 terminal receipt
  -> STOP
```

Cargo environment and argv must be frozen before marker creation. Scientific
execution invokes the sealed test ELF directly; it does not invoke Cargo.

Any build, transport or controller failure after marker consumption is
terminal. No marker is recreated and no scientific action is repeated.

## Failure Taxonomy

```text
identity/source/tool/input drift        BLOCKED_PROVENANCE
Cargo or ELF failure                    BLOCKED_BUILD
request or owner semantic mismatch      BLOCKED_SEMANTIC
capacity/scratch/unresolved              BLOCKED_CAPACITY
search or total p99 over gate           BLOCKED_LATENCY
PSS/payload budget failure              BLOCKED_RSS
generation/readers/stale mismatch       BLOCKED_RELOAD_IDENTITY
host load or thermal precondition       BLOCKED_ENVIRONMENT
```

Failure priority is provenance, build, semantic, capacity, reload identity,
RSS, latency, environment. An incomplete observation or unknown predicate is
`BLOCKED_PROVENANCE`.

## Positive Verdict And Claim Boundary

The only positive verdict is:

```text
M3_END_TO_END_TEST_OWNER_PASS
```

It proves the test-only generation shape, fixed-owner semantics, target-host
closed-call latency, bounded scratch and measured residency. It admits a
separate production authority decision paper only.

Even after PASS:

```text
production bridge edit             not yet admitted
installed sidecar                  absent
runtime reload edit                not yet admitted
daemon / IBus execution            not tested
queue-inclusive product p99        not tested
deployment / install / restart     forbidden
runtime authority changed          false
```
