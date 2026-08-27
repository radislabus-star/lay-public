# V10 E1 Traversal W1 Fused-Minimum M2 V1

Date: 2026-08-26

## Purpose

This paper isolates one source-to-machine mechanism inside the exact W1
single-P-core traversal:

> Can the row minimum be accumulated while the seven exact radius-3 cells are
> produced, eliminating the separately lowered minimum reduction without
> changing the candidate language, frontier, state, rank or traversal order?

It is not a new sampling route and does not continue D7 concurrency research.
It proposes one test-only diagnostic build and a balanced single-worker
comparison on the pinned target. It grants no production edit or runtime
authority.

## Admission

The predecessor verdict is:

```text
W1_MACHINE_COST_DECOMPOSITION_COMPLETE
next action admitted              FUSED_MINIMUM_MECHANISM_PAPER_ONLY
receipt SHA-256                   12e806f11d921047b1437568af6aa77defaa66aba17f86560e87f0167d8d9194
```

The valid D4 stream places `44,922 / 66,543` traversal samples in
`FUSED_SCALAR_U64_ADVANCE`. The distinct post-recurrence minimum machine block
owns `20,839` samples, or `31.3166%` of traversal and `46.3893%` of the fused
bucket. This is mechanism-ordering evidence, not measured cycles and not a
promise of a 31 percent gain.

## Historical Correction

M1 already contained a guarded scalar minimum chain:

```text
minimum = c0
if len > 1: minimum = min(minimum, c1)
...
if len > 6: minimum = min(minimum, c6)
```

M1 passed exact transition parity and reduced its replay from `596.362` to
`477.598 instructions/transition`. That result cannot be assigned to the
minimum chain: M1 changed equality handling, packed state, fixed-cell
unrolling and minimum lowering together.

The later E1 integration, D1 and D7 use:

```rust
let cells = [c0, c1, c2, c3, c4, c5, c6];
let minimum = cells[..len].iter().copied().min().unwrap_or(outside);
```

Therefore M2 is not a claim that the idea is novel. It is the first full-W1
experiment that isolates minimum lowering while holding the already integrated
packed transition fixed.

## Pinned Evidence

```text
W1 decomposition receipt
  SHA-256                         12e806f11d921047b1437568af6aa77defaa66aba17f86560e87f0167d8d9194

W1 decomposition manifest
  SHA-256                         7038f33c4c3a5f042607ac1cd1d5997648e3b8d158b7cf04acf4cab862da7c1b

D7 terminal audit
  SHA-256                         db8f8fbb2ab0bbf6ba45ca9b4d2ce7c394c3de826d82961ce938adea79024f3e

D7 diagnostic fragment
  SHA-256                         8c9ff3aaf43942aff6090b1350cef1828e24ea5664d312bde2ebdf29be6687ce

M1 decision
  SHA-256                         f75bdc6995bcdc8553b267ae43e511321bb34fe9d4d9acb14a610104356573a1

M1 result manifest
  SHA-256                         775ed5125eb541b54f2e8f9a911c688258d13089ff8bbb629938395f1dbe2f94

M1 source fragment
  SHA-256                         bdef992d1f9bec095b3f683b384f1e7d23323823625cf3547dc44480511f0d76

recovered V10 source
  SHA-256                         f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c

production prefix
  bytes                           39,047
  SHA-256                         ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26

package / sidecar / denominator / schedule
  SHA-256                         cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b
                                  a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd
                                  33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4
                                  2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78
```

The active worktree source is not an input. Source assembly starts from the
sealed V10 prefix and sealed D7 fragment only.

## Exact Compared Forms

All three forms compute the same `c0..c6`, packed state, depth, start and len.
Only the minimum expression differs.

```text
B  ITERATOR_BASELINE
   exact D7 expression:
   [c0..c6][..len].iter().copied().min().unwrap_or(outside)

G  M1_GUARDED_CHAIN
   exact already parity-proven M1 guarded scalar chain

I  INTERLEAVED_RUNNING_MIN
   after each cN is produced:
   mN = min(mN-1, cN)
   all cN with N >= len are exact outside = radius + 1
   therefore their unconditional inclusion cannot lower the valid minimum
```

The interleaved form is frozen as immutable scalar assignments, not an array,
iterator, SIMD intrinsic, SWAR reduction or architecture-specific assembly.
The compiler remains free to lower it, but the post-build audit must publish
the exact machine ranges and bytes for B, G and I before any physical route.

No fourth form may be added after build or result inspection.

## Consequence Analysis

The candidate changes no package bytes, DAFSA decode, equality windows,
candidate language, edge order, stack order, rank, terminal handling,
certificate, budget, cache key, SafetyGate, verifier, daemon, installed binary
or runtime process. The only candidate effect is how the already required row
minimum is computed inside a test-only copy of `d1_u1_advance`.

The diagnostic ELF contains all three forms so every physical route shares one
compiler, link and layout context. The exact D7 baseline remains present as B.
Adding G and I may perturb layout, so B must independently remain within the
frozen D7 W1 validity envelope before candidate comparison is admitted.

## Closed Route Graph

```text
BOOTSTRAP
BUILD
READ-ONLY BUILD AUDIT
PARITY
B0-ITERATOR
G0-M1-GUARDED
I0-INTERLEAVED
I1-INTERLEAVED
G1-M1-GUARDED
B1-ITERATOR
TERMINAL AUDIT
```

The six physical routes form a palindrome. This balances monotonic loaded-host
drift without adaptive ordering. No route is repeated, substituted or added.

Markers exist only for `BUILD`, `PARITY` and the six physical routes. They are
created only after a bootstrap audit proves target identity, namespace absence,
exact inputs and UID/path capability. Every marker is atomically renamed from
`available` to `consumed-before-exec` before Cargo, subject or perf execution.
Failure keeps the marker consumed and evidence retained. No automatic retry is
permitted.

## One Diagnostic Build

The build is one fresh isolated Cargo invocation with the exact D7 toolchain,
release profile, target features, package closure and offline guard. The
production prefix remains byte-identical. Only test-only bytes may differ.

Before parity, an independent read-only build audit must publish:

```text
ELF SHA / size / mode / Build ID / ET_DYN
.text SHA and PT_LOAD geometry
symbols and DWARF presence
assembled source SHA
B/G/I symbol and machine-range identities
exact Cargo argv and environment
ELF executed                         false
```

Unexpected code folding between B, G and I is `BLOCKED_BUILD`; the forms must
have independently auditable machine ownership.

## Semantic Parity

Parity runs before every physical route and compares B, G and I in lockstep.
It includes:

```text
full frozen 382-query forward schedule
full frozen reverse schedule
all 25,145,756 examined transitions per schedule round
query lengths 23..96
radii 0..3
boundary len 0..7
all reachable equality windows
```

Hard equality includes:

```text
c0..c6
packed next state
depth / start / len
minimum and terminal distance
survive / prune decision
edge and stack order
rank and terminal refs
errors / unresolved
structural counters
candidate results and certificates
```

Any mismatch is terminal `BLOCKED_PARITY`. Historical M1 parity is predecessor
evidence only and does not replace parity in the new ELF.

## Physical Envelope

Every B/G/I route is the exact D7 one-worker envelope:

```text
worker                            1
CPU                               0
warmup rounds                     1
measured rounds                   20
queries per round                 382
examined edges per round          25,145,756
measured examined edges           502,915,120
component records                 7,640 x 118 bytes
thread migration delta            0
```

One inherited process-scoped `perf stat` wraps each route. FIFO control enables
events after warmup and disables them after the twentieth measured round:

```text
perf stat --json-output --no-big-num --delay=-1
events                            instructions,cycles,branches,branch-misses,task-clock
control                           subject-ready -> controller-enabled
                                  subject-done -> controller-disabled
```

Only complete unscaled `cpu_core` hardware rows and one complete software
`task-clock` row are accepted. There is no `perf record`, sampling, attach,
SIGINT lifecycle, E-core route or multiworker route.

## Validity Gates

Every physical route requires:

```text
queries / rounds / records         382 / 20 / 7,640
errors / unresolved                0 / 0
structural mismatch                0
affinity                           [0]
migration delta                    0
thermal throttle drift             0
PMU rows                           exact, complete, unscaled
```

Pair validity is computed separately for traversal CPU/edge, cycles/edge and
instructions/edge:

```text
pair spread = abs(route0 - route1) / pair mean
pair spread <= 2%                  required for B, G and I
```

Build/context validity requires:

```text
abs(B mean traversal - 25.923669775527927) / D7 W1 <= 5%
abs(B mean instructions - 361.20658023962375) / D7 W1 <= 1%
```

If baseline validity or any pair spread fails, no candidate verdict is issued.

## Decision Rule

For each candidate `X in {G, I}`:

```text
CPU gain       = (B mean traversal - X mean traversal) / B mean traversal
cycle gain     = (B mean cycles - X mean cycles) / B mean cycles
instruction delta = (X mean instructions - B mean instructions) / B mean instructions
frequency delta   = abs(X mean frequency - B mean frequency) / B mean frequency
```

Candidate X passes only when:

```text
CPU gain                            >= 5%
cycle gain                          >= 5%
instruction delta                   <= 1%
frequency delta                     <= 3%
all semantic and validity gates     PASS
```

If both candidates pass, select the larger CPU gain. If their CPU means differ
by no more than one percent of the faster mean, select lower instructions/edge.
If still tied within one percent, select G because its exact transition form
already has sealed M1 parity evidence.

No statistical claim beyond these two frozen repetitions is allowed. The
result is a deterministic engineering gate for this pinned host and workload,
not a population estimate.

## Verdicts

```text
W1_FUSED_MINIMUM_MECHANISM_PASS
    one frozen candidate passes and is selected

W1_FUSED_MINIMUM_MECHANISM_REJECTED
    parity and validity pass but neither candidate reaches both 5% gates

BLOCKED_PROVENANCE
BLOCKED_BUILD
BLOCKED_PARITY
BLOCKED_CAPABILITY
BLOCKED_MEASUREMENT
BLOCKED_THERMAL
BLOCKED_PERTURBATION
```

Failure dispatch priority is provenance, build/parity as applicable, thermal,
capability, measurement completeness, then perturbation. An incomplete
observation or ambiguous failure dispatch is `BLOCKED_PROVENANCE`.

## Authority Boundary

Positive M2 permits only a separate test-only source decision paper for the
selected exact candidate. It does not permit a production source edit, V11
change, Cargo install, daemon restart, deployment, package/sidecar rewrite,
new executor, DAFSA rewrite, SWAR variant, runtime affinity policy or claim that
production Lay had this W1 cost.

Rejected M2 closes this minimum-lowering mechanism. The next decomposition
bucket, if research continues under a new paper, is DAFSA decode. It does not
authorize modifying the minimum again.
