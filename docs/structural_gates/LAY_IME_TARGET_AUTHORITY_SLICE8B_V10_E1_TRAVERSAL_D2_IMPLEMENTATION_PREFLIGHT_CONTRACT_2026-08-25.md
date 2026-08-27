# V10 E1 Traversal D2 Implementation Preflight Contract

Date: 2026-08-25

## Scope

This contract closes implementation-level prerequisites left open by the
reviewed D2 paper contract. It does not implement D2, build an executable,
execute a D2 subject, or admit any D2 sampling route.

Implementation admission is two-phase:

```text
probe-only implementation preflight
  -> P0 static closure
  -> create probe-only controller
  -> ONE benign capability-probe transaction
  -> immutable capability receipt
  -> final D2 implementation preflight
  -> READY_TO_IMPLEMENT or BLOCKED_CAPABILITY
```

The probe-only preflight can authorize only the benign probe. It cannot
authorize a D2 controller, Cargo, Rust compilation, a D2 ELF, V10/V13 loading,
parity, component routes, latency, or scientific attribution.

## P0 Static Closure

Before the probe, verify and pin:

```text
D2 paper contract and structural-review receipt
D1 decision and PMU interpretation correction V2
exact recovered V10 source
39,047-byte production prefix
exact D1 test fragment
package, sidecar, V7 denominator and frozen schedule identities
Cargo.lock, historical toolchain, release profile and target features
target hostname, exact machine-id bytes, kernel and hybrid PMUs
perf wrapper and resolved kernel-specific perf binary
yes, taskset, sudo and readelf identities
absence of D2 build, result, state and capability-probe paths
```

Static version queries are provenance events, not builds. The P0 audit records
that `/usr/bin/cargo -V`, `/usr/bin/rustc -Vv`, login-path `cargo -V` and
login-path `rustc -Vv` were executed. No Cargo build/check/test or rustc
compilation is admitted.

The reported post-paper audit boundary is also retained:

```text
perf executable version query    YES, one externally reported invocation
perf record/stat                 NO before the capability probe
PMU event                        NO before the capability probe
D2 subject                       NO
```

## P1 Fixed Capability Probe

One probe transaction contains exactly three nonadaptive subruns over the
benign `/usr/bin/yes` process. No subrun uses a D2 ELF, V10, V13, a package,
sidecar, schedule, search executor, quality fixture or foreign process.

```text
T-CAP
  CPU                  0
  event                task-clock:u
  period               100,000 ns

I-CORE-CAP
  CPU                  0, cpu_core
  event                cpu_core/event=0xc0/upp
  period               5,000,000 retired instructions
  required precise_ip  2

I-ATOM-CAP
  CPU                  12, cpu_atom
  event                cpu_atom/event=0xc0/upp
  period               5,000,000 retired instructions
  required precise_ip  2
```

The selectors are frozen from sysfs: both hybrid PMUs expose
`instructions = event=0xc0` and `max_precise = 3`; `cpu_core` owns CPUs `0-11`
and `cpu_atom` owns CPUs `12-19`. No event alias, precision level, privilege
selector, period or CPU substitution is allowed after probe admission.

Each subrun attaches `perf record` to one taskset-pinned `yes` PID, records the
pre/during/post `/proc/<pid>/maps` evidence, and retains `perf.data`, exact
command argv, evlist, sample IP/DSO/event rows, raw records and build-ID list.
The probe transaction marker is consumed before the first `perf record`; any
subrun failure is terminal and forbids retry under this contract.

Required probe result:

```text
records and sample IPs                         >0 per subrun
recorded event identity                        exact per subrun
recorded sample_period                         exact per subrun
precise_ip                                     2 for both instruction subruns
exclude_kernel                                 true
lost records / samples                         0
throttle / unthrottle records                  0 / 0
adaptive frequency or period change            NO
sampled DSO Build ID                           exact benign-ELF Build ID
maps and PT_LOAD evidence                      sufficient for IP normalization
foreign-process control                        NO
runtime authority change                       NO
```

The target currently reports `perf_event_max_sample_rate = 8000`, while the
frozen task-clock period requests a nominal `10,000 samples/s`. This is a known
capability risk. The period is not relaxed. Any kernel throttle, period drift,
lost sample, or missing evidence yields `BLOCKED_CAPABILITY`.

## IP Normalization Contract

Every sampled IP is joined by:

```text
sampled DSO Build ID + normalized ELF virtual address
```

The sampled DSO Build ID must equal the sealed subject Build ID. A pathname
match is not identity.

For `ET_EXEC`:

```text
load_bias     = 0
normalized_ip = sampled_runtime_ip
```

For `ET_DYN`/PIE, select the executable mapping containing the sampled IP and
the corresponding `PT_LOAD` segment by page-aligned file offset:

```text
load_bias = mapping_runtime_start
            - align_down(PT_LOAD.p_vaddr, page_size)

normalized_ip = sampled_runtime_ip - load_bias
```

The controller obtains mapping start/end/offset from perf MMAP/MMAP2 evidence
or a PID-bound `/proc/<pid>/maps` snapshot taken while the sampled mapping is
live. It obtains ELF type, Build ID, PT_LOAD offsets/vaddrs and machine bytes
from the sealed ELF. It never guesses a base from a pathname or the minimum
sampled IP.

The benign probe must prove this route with `/usr/bin/yes`, which is currently
`ET_DYN`, Build ID `8c99ebc2c856857219acc612c2d9be3172b74be5`.

For the future D2 ELF, the ELF-type-specific rule is finalized after the one
symbolized build and before `D2_BUCKET_MAP.json` publication. Any missing,
ambiguous, overlapping or Build-ID-mismatched normalization evidence yields
`BLOCKED_BUCKET_MAP`; no attribution is published.

## Final D2 Implementation Admission

The final implementation preflight consumes the immutable capability receipt.
It reaches `READY_TO_IMPLEMENT` only when P0 and P1 pass. It may authorize
writing the D2 controller and source-preserving build route, but no build may
start before D2-A closure succeeds.

```text
D2-A immutable closure
  -> consume one-build marker
  -> ONE symbolized build
  -> ELF type / Build ID / .text / symbols / DWARF / PT_LOAD audit
  -> finalize actual-ELF normalization rule
  -> disassemble
  -> seal D2_BUCKET_MAP.json
  -> parity
  -> U-SINGLE -> U-FIXED -> U-REVERSED
  -> all-U perturbation gate
  -> only then consume T-* and I-* sampling markers
```

Sampling admission requires all three unsampled routes:

```text
semantic / structural mismatch                   0
U traversal CPU/edge versus sealed D1          <=5% per mapping
U fixed/reversed instructions/request          <=1%
```

Parity alone cannot consume a sampled-route marker.

## Predicted Denominator Feasibility

Sealed D1 gives one 20-round route:

```text
examined edges              502,915,120
single traversal CPU          13.061 s
fixed traversal CPU           22.500 s
predicted task-clock samples 130,607 / 225,004
predicted instruction samples 36,573
```

These exceed the frozen minima of `50,000` task-clock and `20,000` instruction
samples per route. This is a preregistered feasibility estimate only. It cannot
justify changing a period, adding a rerun or weakening a denominator after
execution.

## Bucket-Map Integrity

The generator must classify instruction ranges before any D2 sample is visible.
Ranges carry ELF SHA-256, Build ID, normalized start/end, source/inlined frame,
bucket, reason and machine-byte SHA-256.

Instruction ranges assigned to distinct buckets may not overlap. Ambiguous
DWARF, inlining, compiler-generated or shared-helper ownership is
`UNATTRIBUTED`, not manually forced into a mechanism. A map overlap, byte-hash
mismatch, normalization ambiguity or traversal-unattributed share above `5%`
blocks attribution.

## Static Source Vetoes

The exact D1 fragment SHA-256 is authoritative. The assembled D2 Rust source
must use that fragment byte-for-byte and the exact `39,047`-byte V10 production
prefix. Source scans are secondary tripwires and reject any new:

```text
clock_gettime or Instant::now in the edge loop
Atomic type or fetch_add hot-loop counter
per-edge observer, counter, branch or hook
SWAR or replacement transition
state/edge layout change
rank, stack, prune or budget change
```

## Decision Outputs

For every bucket, valid D2 attribution publishes:

```text
single_ns_per_edge
fixed_ns_per_edge
reversed_ns_per_edge
fixed_inflation_ns_per_edge = fixed - single
inflation_share             = fixed_inflation / 18.77 ns
zero_cost_capacity          = fixed_ns_per_edge / 44.74 ns
```

`inflation_share` and `zero_cost_capacity` answer different questions. A large
single-route transition bucket does not prove that transition creates the
concurrency inflation. No D2 result directly admits SWAR, layout work, V12,
full B, runtime integration or deployment.
