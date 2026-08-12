# L2 Productive V66: Frozen-H and Bounded Recovery Paper

Date: 2026-08-11  
Status: `PAPER_REVIEWED_MICRO_ONLY_AUTHORIZED`  
Runtime authority changed: `false`

## 1. Decision

V65 is not promoted. V64 remains the frozen canonical architecture point.
V66 is permitted only as a shadow-only implementation and micro proof of one
combined mechanism:

```text
frozen V64 hypothesis oracle
-> observed-slot intersection
-> generic syncretic identity birth + learned reverse birth
-> dedup before execution
-> exposed-slot-only exact replay
-> independently calibrated cross-lane comparison
-> deterministic bounded readout
```

The paper does not authorize installation, daemon/IBus changes, live ownership,
or a full package build before the local invariants and `13 x 10 x 2` micro
pass. A micro PASS authorizes the fixed `13 x 100 x 2` proof, not promotion.

## 2. Frozen Evidence

Canonical V64:

```text
package path on build host
  /home/e/projects/lay-productive-v1-build-20260811/out/
  LAY-L2-PRODUCTIVE-PARADIGM-V1-SHADOW-V64.p2m
package bytes        17,309,944
package sha256       9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
proof spool path
  /home/e/projects/lay-productive-v1-build-20260811/work/full-v1-v63-reinduce/
  context-sorted/sorted-events-global.p2s
proof spool bytes    1,154,794,811
proof spool sha256   6e282474b26bf90dc61ee21c93c9dd7dd727c29a2b02650c513ffdd06746e807
LEMMA_HELDOUT cases  1,300
frozen H             1,280
B / S0 / R           1,219 / 1,219 / 1,219
top-16               1,218
raw top-1              267
false singleton          0
integrity errors          0
probe parity          2,600 / 2,600
runtime authority changed false
```

Rejected V65:

```text
base .p2m bytes       17,309,944, byte-identical V64
recovery .p2r bytes    3,514,208
recovery paths            36,915
recovery operations      136,074
reported H                 1,281  <- contaminated oracle
B / S0 / R                 1,277 / 1,277 / 1,277
H -> B                         4
top-16                     1,276
raw top-1                    205
false singleton                0
integrity errors                0
maximum class p99         255.827 ms
proof time              1,951.507 s
```

The four remaining `H -> B` failures are one proof event for one target lemma,
repeated in `layout_projection`, `repeated_fragment`,
`sparse_multi_omission`, and `suffix_truncation`. The target surface has many
syncretic slots. This is evidence for a missing generic identity bridge, not
permission for a word-specific exception.

## 3. Measured Fan-Out

All `159` V65 `.p2r` index records were parsed read-only on the build host.
Posting fan-out per `(POS, source slot)` lookup is:

```text
min      1
p50    211
p75    271
p90    386
p95    418
p99    543
max    641
mean   232.170
```

The ranges are contiguous and sum exactly to `36,915` postings. V65 therefore
starts hundreds of reverse paths per observed source before it knows which
paradigms satisfy the complete observed-slot constraint. For the remaining
failure, `9` observed forms expanded to `3,153` recovery paths and `2,975`
successfully reconstructed anchors before only `54` exact reconstructions and
`6` retained bindings remained.

This profile proves excessive pre-filter work. It does not yet measure V66's
post-intersection or post-dedup counts.

Exact receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/fanout-profile.json`

## 4. Formal Objects

For a proof case `q` and POS domain `d`:

```text
O(q,d) = sorted unique exposed (slot, normalized surface) observations
S(q,d) = slots present in O(q,d)
L(d,s) = all paradigms whose structural slot profiles contain s
A(d,s) = current direct source-anchor compatibility postings
R(d,s) = learned reverse-anchor postings for source slot s
I(d,s) = paradigms whose transferable anchor -> s program is exact identity
E(q,d) = intersection over s in S(q,d) of L(d,s)
```

`L` is an independent structural slot-license index; it is not the current
source-anchor compatibility index `A`. This distinction is mandatory. The
remaining V65 oracle paradigm is absent from `A` and `R`, so using either as
the eligibility universe would reproduce the same failure and prevent `I`
from ever birthing the missing paradigm.

`E` is computed once per `(proof identity, POS)`, not once per source form.
The target slot, target surface, frozen oracle paradigms, damage label, and
proof outcome are forbidden inputs to `E`, `R`, `I`, scoring, or readout.

A cold binding is admissible only when one recovered anchor reproduces every
member of `O(q,d)` exactly under the same paradigm. This is a coverage claim,
not Winner authority.

## 5. Frozen-H Manifest

### 5.1 Purpose

V65 computed `oracle_bindings` through the sidecar-aware runtime in
`src/nanda_wave/l2_field/productive_v1/proof.rs`. That changed `H` from
`1,280` to `1,281`, invalidating the causal comparison. V66 must never compute
its denominator through the experimental lane.

### 5.2 Identity

The immutable manifest header contains at least:

```text
schema_version
V64 .p2m sha256
proof spool sha256 and byte length
L1.1 package sha256
canonical L2 package sha256
axis schema sha256
heldout_per_class = 100
cohorts = [LEMMA_HELDOUT]
damage generator/version identity
entry count = 1,300
H count = 1,280
payload sha256
```

Each sorted entry contains:

```text
proof_identity[32]
damage_class
damage_identity[32]
target_lemma_id
target_pos_domain
sorted unique V64 oracle_paradigm_ids
```

The manifest includes all `1,300` LEMMA_HELDOUT cases. `H` is exactly the
number of entries with at least one V64 oracle paradigm matching the target
lemma and POS: `1,280`.

### 5.3 Construction and Use

The manifest is produced once with an explicit `RecoveryPolicy::Disabled`
V64 loader. The builder fails if a `.p2r` is mapped, the package/spool identity
differs, case identities are duplicated, or the denominator is not exactly
`1,280`.

During V66 proof:

```text
manifest oracle paradigms -> comparator and stage counters only
target-blind observed forms -> V66 runtime only
```

Passing an oracle paradigm ID, target slot, or target surface from the manifest
into candidate birth, recovery, exact replay, scoring, or readout is a hard
integrity error. Missing or extra sampled cases also fail the proof; they do not
silently change the denominator.

## 6. Bounded Recovery Algorithm

### 6.1 Constraint First

For each `(lemma, POS)` group:

1. Normalize and deduplicate observations once.
2. Build `S(q,d)` once.
3. Intersect the sorted `L(d,s)` postings into `E(q,d)` before reverse
   execution.
4. Form target-blind birth sources from `A(d,s)`, `R(d,s)`, and `I(d,s)`.
5. Reject any birth whose paradigm is not in `E(q,d)`.
6. Union the surviving direct, learned-recovery, and identity candidates.

The intersection is exact. `I` is deliberately independent from `A` and `R`;
otherwise it cannot repair their missing-edge mechanism. There is no top-N
paradigm quota and no quality-affecting early truncation.

### 6.2 Dedup Before Execution

Learned reverse programs are executed only after paradigm filtering. Recovered
anchors are inserted into a bounded deterministic map keyed by:

```text
(POS, paradigm_id, recovered_anchor)
```

Multiple observations or recovery programs that produce the same key merge
support/provenance. Exact replay runs once per key. Direct candidates are also
deduplicated by `(POS, paradigm_id, source anchor)` before replay.

Telemetry must record, separately:

```text
posting lookups
raw postings
post-intersection postings
reverse programs executed
unique recovered anchors
exact replay programs executed
retained bindings
```

### 6.3 Exposed-Slot-Only Replay

V65 calls `instantiate_paradigm_surfaces`, walks the complete paradigm trie,
collects every generated slot, and only then checks exposed forms. V66 adds a
compact mmap index:

```text
(paradigm_id, anchor_slot_id, exposed_slot_id)
  -> exact program range
```

Only program ranges for slots in `S(q,d)` execute. A candidate passes iff each
exposed `(slot, surface)` has an exact generated match and no exposed slot is
missing. Programs for hidden/unrequested slots do not execute during binding
validation.

This route applies to both direct V64-compatible and recovered bindings. That
is necessary because V64 itself has measured maximum class p99 `86.001 ms`;
optimizing only `.p2r` cannot satisfy the unchanged `p99 <=5 ms` contract.

The replay index changes access, not morphology semantics. A parity test must
compare exposed-slot results against the old complete-trie oracle for every
program/slot selected by the micro corpus.

## 7. Generic Syncretic Identity Bridge

An identity bridge is licensed only when the packaged transferable program for
paradigm `p`, canonical anchor slot `a`, and observed slot `s` is structurally:

```text
COPY_ALL_SOURCE_SCALARS
TERMINATE(slot=s)
```

It may contain no drop, replace, emitted segment, exact allomorph, or
lemma-local payload. The paradigm must retain its existing independent TRAIN
support gate. The bridge index is keyed only by `(POS, observed slot)` and
paradigm identity; it stores no lexical surface.

At runtime, an observed normalized surface `x` may therefore propose anchor
`x` for every eligible identity paradigm. The candidate still must:

1. belong to the same POS domain;
2. license every observed slot;
3. replay every exposed surface exactly;
4. survive deduplication by `(paradigm_id, x)`;
5. remain non-authoritative when several compatible paradigms generate
   different hidden surfaces.

Equal surfaces across syncretic slots thus create a generic zero-edit route.
No branch may mention `впросинь`, a lemma ID, a proof identity, a fixture name,
or a damage class.

## 8. Rank Preservation

V65 improved binding coverage while raw top-1 regressed `267 -> 205`. The new
lane therefore cannot share one uncalibrated total order with V64 candidates.

V66 keeps two typed origins through readout:

```text
BaseV64       candidate already born without recovery
RecoveredV66 candidate born only through learned recovery or identity bridge
```

The bounded physical representation retains up to `32` BaseV64 identities and
up to `32` RecoveredV66 identities before equal-surface merging. A recovered
candidate cannot consume a base-lane slot. Cross-lane comparison determines
the displayed order, not lane retention.

For two BaseV64 candidates, V64 ordering is byte-for-byte unchanged. A
RecoveredV66 candidate may move ahead of a retained BaseV64 candidate only with
an independent cross-lane comparison certificate fitted outside the fixed
proof set. Its features may use measured geometry, context, independent
support, stability, and contradiction evidence, but may not use target
identity, proof cohort/class, manifest oracle IDs, path count, or a manual
word/suffix rule.

The certificate is valid only when its heldout calibration cell has sufficient
support and its lower confidence bound favors the recovered candidate. The
threshold and coefficients are fitted evidence, not a hand-assigned quota.
Without a valid certificate, V64 order wins and the recovered candidate can
remain in the bounded lattice as `Tied/ABSTAIN` evidence.

Formal monotonic gate for each fixed case `q`:

```text
BaseProjection(ReadoutV66(q)) == ReadoutV64(q)

and

for every BaseV64 candidate b demoted by a RecoveredV66 candidate r:
  valid_independent_certificate(q, r, b) == true
```

Additionally:

```text
V66 raw top-1 >= V64 raw top-1 = 267
V66 must not lose any V64 target retained in the bounded lattice
```

This does not claim that `267/1,300` is good enough for promotion. It only
prevents a coverage experiment from destroying an already measured baseline.

## 9. Proof Scheduler

The current scheduler splits `2,600` cases into contiguous chunks of `137`
for `19` workers. Case cost varies by observed-slot count, posting fan-out,
surviving paradigms, and exact replay work, so contiguous chunks produced an
observed long single-worker tail. Per-worker work telemetry was not recorded;
the mechanism follows directly from the contiguous scheduler and variable case
cost, while the exact imbalance remains unmeasured.

V66 uses a bounded dynamic queue:

1. Assign every case its immutable original ordinal.
2. Compute a target-blind cost estimate from observed-slot count and index
   fan-out only.
3. Sort work ordinals by cost descending, then stable proof identity.
4. Let `19` workers claim one ordinal at a time through an atomic cursor.
5. Store results by original ordinal.
6. Reduce metrics in original ordinal order.

The queue owns only `2,600` ordinals and one result slot per case; it cannot
grow during proof. Execution order may vary, but receipt bytes and aggregate
metrics remain deterministic. Per-worker case count, estimated work, elapsed
time, and final-tail duration become mandatory receipt fields.

The scheduler is proof-only and cannot affect runtime authority or model
scores.

## 10. Numeric Gates

The `13 x 10 x 2` micro and then the fixed `13 x 100 x 2` proof must satisfy
all of the following. There is no aggregate override:

```text
frozen H                                      exactly 1,280
H -> B                                        0
B -> S0                                       0
raw top-1                                     >=267
each LEMMA_HELDOUT class top-16               >95.0%
false singleton                               0
integrity errors                               0
probe parity                                  exact
BaseProjection(V66)                           exact V64 parity
demotions without independent certificate     0
maximum class closed-call p99                 <=5.000 ms
runtime authority changed                     false
```

Package identity and resource fields must also be reported, even though no new
promotion budget is inferred from a micro:

```text
base .p2m bytes/hash
sidecar bytes/hash
total package bytes
steady and peak RSS
cold mmap load
proof wall time
per-worker work and tail
all bounded-recovery counters
```

If strict p99 fails, V66 is rejected even when `H -> B = 0`. If H/B pass but
top-1 regresses, V66 is rejected. If all listed gates pass, the result remains
shadow-only until SLOT_HELDOUT, MULTI_LABEL, UNSUPPORTED, integrated
L1.1/L2/L3/L4/verifier, queue-inclusive, daemon/IBus, and physical product
gates are measured separately.

## 11. Paper Critique

### 11.1 Identity Is Necessary but Not Sufficient

Syncretic identity can admit many paradigms that agree on exposed surfaces but
disagree on a hidden slot. Exact replay cannot make an unidentifiable choice
unique. The design is sound only because such cases remain a lattice and the
bridge grants no Winner authority.

### 11.2 Frozen-H Is a Proof Artifact

The manifest contains target-aware oracle information. Any reuse in runtime
would be label leakage and a hard veto. Code ownership must keep manifest
loading inside the proof module; packaged runtime APIs must never accept it.

### 11.3 Rank Monotonicity Can Preserve a Bad Baseline

V64 top-1 is only `267/1,300`. Preserving it prevents V66 from accidentally
regressing while testing coverage, but it does not solve contextual selection.
Later improvement requires independently trained cross-lane evidence and L3,
not weakening this gate. V66 is a recovery closure, not the final morphology
owner.

### 11.4 The Latency Gate Is Harder Than Removing V65

V64 already reaches maximum class p99 `86.001 ms` under the same closed-call
proof, above the `5 ms` gate. Therefore a V66 implementation restricted to
filtering `.p2r` is guaranteed to fail. Exposed-slot direct execution must
replace complete-trie replay for both base and recovery paths. The paper
authorizes that access-path change only under exact semantic parity.

### 11.5 Current Fan-Out Does Not Predict V66 Cost Yet

The measured `p99=543` and `max=641` are pre-intersection posting counts. They
prove V65 does excess work but do not prove V66 will be fast. Post-intersection,
post-dedup, exact-program, allocation, and per-worker telemetry are mandatory.

### 11.6 Scheduler Improvement Does Not Improve Request Latency

Dynamic scheduling shortens proof wall time and removes idle worker tails. It
does not make one runtime request faster. Request p99 must pass independently;
proof throughput may not be presented as runtime latency.

## 12. Implementation Ownership

The permitted source ownership is:

```text
proof.rs
  frozen manifest build/load, fixed denominator, scheduler, parity counters

anchor_recovery_package.rs
  identity/recovery index format and mmap validation

packaged_runtime.rs
  observed constraint intersection, pre-execution dedup,
  exposed-slot exact replay, typed candidate origin

score/readout owner
  independently trained cross-lane certificate and monotonic comparison
```

The runtime hot path may not depend on proof fixtures or the frozen-H manifest.
The scheduler may not enter runtime code. No literal word, phrase, suffix,
lemma ID, source ID, or proof identity is allowed as authority.

## 13. Implementation Snapshot Before Remote Proof

```text
frozen denominator design            CLOSED_ON_PAPER
V65 fan-out diagnosis                MEASURED
bounded recovery ordering            CLOSED_ON_PAPER
syncretic identity mechanism         CLOSED_ON_PAPER
rank regression prevention           CLOSED_ON_PAPER
proof scheduler ownership            CLOSED_ON_PAPER
runtime implementation               IMPLEMENTED_LOCALLY
frozen-H ownership                    PROOF_ONLY
base/recovery rank lanes              IMPLEMENTED_LOCALLY
exposed-slot direct execution         IMPLEMENTED_LOCALLY
complete-trie parity oracle           RETAINED_FOR_PROOF
V66 package                          BUILT_SHADOW
V66 quality/latency                   FAIL_MICRO
installation/live authority          FORBIDDEN
V64 canonical status                 PRESERVED
```

The local implementation keeps the frozen V64 target-aware manifest entirely
inside `proof.rs`. Runtime candidate birth and recovery receive only universal
observed-form, POS, posting, slot-compatibility, and exact-reconstruction stage
sets. `PackagedProductiveRuntimeV1::load_without_anchor_recovery()` provides the
base V64 oracle used by proof without changing the productive package.

Direct execution resolves the terminal for each selected program through a
compact `u32` index and executes only requested exposed slots. For the frozen
V64 package's `77,854` programs, this index occupies exactly `311,416` bytes;
the package remains mmap-backed. The complete-trie implementation is retained
as a read-only semantic oracle and focused tests require exact direct/oracle
parity.

Local verification completed before any remote package build:

```text
scripts/cargo-guard.sh check --lib
  PASS

scripts/cargo-guard.sh test --lib \
  'nanda_wave::l2_field::productive_v1::' -- --nocapture
  74 passed; 0 failed; 1,250 filtered out
```

These checks cover package determinism, mmap reopen, direct/full-trie parity,
base-lane rank preservation, calibration, composite readout, L3 handoff,
format validation, and integrity failures. They do not measure the fixed
quality denominator, per-class latency, full process RSS, daemon/IBus behavior,
or physical product behavior. Runtime authority remains unchanged.

The design and local implementation authorize only the next `13 x 10 x 2`
shadow micro. A larger proof is authorized only after every micro gate passes;
installation and live promotion remain forbidden until all later gates pass.

## 14. Evidence

V64 corrected proof:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/hbs0-pos-diagnostic-13x100-receipt.json`

V65 full proof and build:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V65_ANCHOR_RECOVERY_2026-08-11/`

V66 paper measurement:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/fanout-profile.json`

V66 local implementation verification:

```text
scripts/cargo-guard.sh check --lib
scripts/cargo-guard.sh test --lib \
  'nanda_wave::l2_field::productive_v1::' -- --nocapture
```

Owning predecessor paper:

`/home/ubu/projects/lay/docs/l2-productive-post-v64-anchor-recovery-paper.md`

## 15. First Remote V66 Micro

The first implementation experiment used the frozen V64 package and spool on
the 20-CPU remote builder. The release binary compiled successfully with Rust
`1.97.0`; the package resume reused all six completed corpus/induction stages.

```text
release build wall                    163.59 s
release build peak RSS              2,261,104 KiB
resume wall                             67.61 s
resume peak RSS                        621,432 KiB
base .p2m bytes                     17,309,944
base .p2m sha256                    9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
recovery .p2r bytes                  3,514,208
recovery .p2r sha256                744a11a8c88921bc3b63899840982ecadc3260485808dd60ea4858bcd54a8436
resident constant cache               311,540 B
runtime authority changed                 false
```

The `13 x 10 x 2` proof measured `260` cases and rejected the implementation:

```text
frozen manifest entries / H          1,300 / 1,280
sampled heldout H / B / S0             127 / 127 / 127
H -> B                                         0
B -> S0                                        0
raw top-1 / frozen-base top-1              31 / 31
BaseProjection failures                       0
uncertified demotions                          0
probe parity failures                          0
false singleton                                0
integrity errors                                0
maximum class p99                         464.644 ms
proof peak RSS                            305,008 KiB
verdict                     FAIL_measured_shadow_gates
```

The micro denominator is too small for the strict per-class percentage gate:
one outside-H case yields `90%` in a ten-case class. This does not authorize
changing the gate or sample. The full frozen denominator remains required, but
it is not run while the independent latency gate fails.

The new telemetry identifies the first shared runtime mechanism:

```text
heldout cases measured for recovery               130
sources                                           6,158
structurally eligible paradigms                  62,916
recovery paths read                           1,546,928
post-intersection recovery paths              1,276,120
reverse programs executed                     1,276,120
unique recovered anchors                        365,986
identity bridge candidates                       53,057
exact replay programs executed                14,025,181
retained bindings                                 3,631
```

Exposed-slot direct execution is semantically correct but is reached after an
unbounded multiplicative recovery frontier. The next permitted implementation
change is therefore one mechanism only: deduplicate and bound recovery work per
eligible paradigm before reverse execution, then perform one exact exposed-slot
validation per recovered anchor identity. No score, coefficient, proof case,
authority threshold, SafetyGate, or verifier behavior may change.

Measured receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/resume-build-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/resume-build.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-13x10.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 27. Post-Memo Symbolized Profile

The authorized profile repeated the identical micro after the per-readout
geometry memo. The diagnostic receipt retained exact sampled coverage, rank,
probe parity, BaseProjection parity, safety, package bytes, and unchanged
runtime authority.

The profile separated cold grounding, which is outside the closed-call latency
timer, from the timed readout. `evaluate_checked` accounted for `31.38%` of
whole-command cycles and `traverse_binding` for `29.63%`. Inside that timed
route the remaining shared geometry mechanism was the coexistence of two
executors:

```text
BaseV64 complete-trie terminal atom finalization       3.11%
BaseV64 incremental geometry-state clone               2.59%
BaseV64 scalar emit                                     2.39%
direct batch simhash                                    3.96%
direct OSA                                              1.60%
feature input                                           1.54%
direct atom-family construction                         1.08%
```

The complete-trie route remains necessary as the frozen V64 oracle, but it is
not required as the experimental execution algorithm. The next permitted
change is therefore one exact ownership split:

```text
frozen oracle runtime, no recovery sidecar
  -> BaseV64 complete trie

experimental V66 runtime, validated recovery sidecar
  -> BaseV64 and RecoveredV66 prepared direct replay
  -> one shared per-readout geometry memo
```

The base package, terminal identities, candidate order, scores, frontiers, and
authority remain frozen. A focused test must compare the complete readout from
the sidecar-backed direct executor with the sidecar-disabled complete-trie
oracle. The fixed proof must continue to compare every experimental BaseV64
projection against the independent oracle; any difference rejects the change.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-geometry-memo-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-geometry-memo-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-geometry-memo-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-geometry-memo-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-geometry-memo-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-geometry-memo.time.txt
```

Not tested by this profile: fixed `13 x 100 x 2`, slot-heldout, multi-label,
unsupported, integrated L1.1/L3/L4/verifier transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 31. Deferred-Simhash Remote Micro

The tenth V66 micro used the unchanged V66 package bytes and the immutable
`13 x 10 x 2` sample. The first remote build failed before execution because
five dependent `productive_v1` modules in the proof workspace predated the
locally verified API. After synchronizing the complete scoped
`productive_v1` tree and proving it byte-identical, the release build passed.
This was a workspace synchronization correction, not a runtime or package
change.

Measured results for `260` cases and `20` workers:

```text
metric                                  unified direct   deferred simhash
H / B / S0                                127/127/127       127/127/127
H -> B losses                                       0                 0
B -> S0 losses                                      0                 0
raw top-1 / base top-1                          31/31             31/31
BaseProjection comparisons / failures          260/0             260/0
uncertified demotions                               0                 0
probe parity failures                               0                 0
false singleton                                     0                 0
integrity errors                                     0                 0
maximum class p99                             8.480 ms         11.409 ms
proof user CPU                                  16.69 s           16.55 s
proof peak RSS                            308,716 KiB      306,768 KiB
productive package                         17,309,944 B     17,309,944 B
runtime authority changed                         false             false
```

The deferred fields preserved every measured candidate, score, rank, frontier,
BaseProjection comparison, probe comparison, safety result, and package byte.
User CPU decreased by `0.84%`, but the maximum-class tail regressed by
`34.54%`. The ten-case class denominator makes p99 equal to the class maximum,
so the result does not establish whether the regression is deterministic or a
scheduling outlier. It nevertheless fails the unchanged hard gate and cannot
authorize the fixed proof or promotion.

The verdict remains `FAIL_measured_shadow_gates`. The next permitted action is
a symbolized profile of this exact runtime. No repeat may be used to select a
friendlier p99, and no candidate, coefficient, denominator, package, or
authority change is authorized.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-deferred-simhash.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-deferred-simhash.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-deferred-simhash-v2.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-deferred-simhash-v2.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-deferred-simhash-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-deferred-simhash-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-deferred-simhash-13x10.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, or physical product behavior.
Runtime authority remained unchanged.

## 30. Deferred-Simhash Strict-Parity Closure

The deferred-simhash implementation initially passed `76/77` focused tests.
The failing compiler reopen test compared the prepared direct executor with the
independent complete-trie oracle immediately after `traverse_binding`. At that
internal boundary, the prepared candidates intentionally contained complete
ranking geometry but not the two deferred simhash fields; normal
`evaluate_checked` execution completed those fields after bounded selection.

The parity helper now completes the exact character and keyboard simhash for
every selected direct candidate before comparing complete candidate vectors.
It does not relax the comparison or alter either executor. Local verification
then completed:

```text
cargo check --lib                                      PASS
focused productive_v1 tests                           77/77 PASS
direct versus complete-trie cold candidate parity      exact
package bytes                                           unchanged
candidate birth, score, rank, frontier                  unchanged
runtime authority changed                                  false
```

This closes only the local strict-parity defect. It does not prove the `5 ms`
latency gate, fixed `13 x 100 x 2` quality, integrated transfer, daemon/IBus
behavior, or physical product behavior. The V66 verdict remains
`FAIL_measured_shadow_gates` until the immutable remote micro measures maximum
class `p99 <= 5 ms`.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-deferred-simhash-focused-tests.txt
```

## 25. Post-Preparation Symbolized Profile

The authorized profile repeated the identical `13 x 10 x 2` micro with the
prepared packaged runtime. The symbolized binary retained `127/127/127`,
`31/31`, zero H-to-B and B-to-S0 losses, exact probe parity, zero
BaseProjection failures, zero uncertified demotions, zero false singletons,
zero integrity errors, byte-identical packages, and unchanged runtime
authority. Its latency is diagnostic and cannot promote V66.

The `cpu_atom/cycles` self-cost ranking is now:

```text
batch_simhash                                    12.45%
repeated u32 unstable sorting                    12.43%
traverse_binding                                  9.08%
OsaLaneV1::emit                                   7.78%
derive_cold_lemma_bindings_with_diagnostics       5.92%
append_atom_family_reused                         5.70%
slot-profile package record                       3.52%
paradigm_reconstructs_exposed_forms               3.50%
execute_packaged_program_into                     2.42%
```

Prepared morph-program and morph-operation record decoding no longer appears
as a leading owner. The first shared remaining mechanism is repeated complete
geometry evaluation for equal materialized normalized surfaces across direct
recovery bindings: atom construction, u32 sorting, simhash, and OSA are all
repeated before the later surface-basin deduplication.

The next permitted code change is a score-preserving, per-readout, bounded
geometry memo keyed only by the complete normalized surface. The observed
geometry is immutable for the memo lifetime, and the cached value is the full
`GeometryTerminalEvidenceV1`; therefore a hit must be exactly equal to a fresh
batch evaluation. The memo may avoid repeated computation only. It may not
change generated surfaces, candidates, identities, scores, rank, frontiers,
authority, proof data, SafetyGate, or verifier behavior. Repeated-hit parity
must be tested before the remote micro.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-execution-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-execution-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-execution-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-prepared-execution-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-prepared-execution-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-prepared-execution.time.txt
```

Not tested by this profile: fixed `13 x 100 x 2`, slot-heldout, multi-label,
unsupported, integrated L1.1/L3/L4/verifier transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 23. Post-Batch Symbolized Profile

The authorized profile was repeated against the exact batch-geometry runtime.
The profile receipt preserved `127/127/127`, `31/31`, exact probe parity, zero
safety failures, and unchanged runtime authority. Its `58.523 ms` diagnostic
p99 is not a promotion measurement.

The `cpu_atom/cycles` self-cost profile identified the direct replay pipeline as
the next shared mechanism:

```text
derive_cold_lemma_bindings_with_diagnostics      10.25%  outside closed timer
repeated unstable u32 sorting                    10.05%
MorphOp mmap record decode/validate               7.97%
MorphProgramHeader mmap record decode             7.94%
batch_simhash                                     7.90%
traverse_binding                                  6.38%
OsaLaneV1::emit                                   5.01%
append_atom_family_reused                         4.22%
execute_packaged_program_into                     3.92%
paradigm_reconstructs_exposed_forms               3.54%  outside closed timer
SlotPhaseProfile mmap record decode               3.25%
```

The previous incremental atom owner is no longer material. Recovered V66
bindings instead repeatedly decode already validated immutable package records,
scan each operation list twice, materialize the same program representation,
and invoke generic sorting for three-unit bag atoms. The next permitted change
is one score-preserving prepared-execution mechanism:

1. decode and validate immutable program, operation, terminal, and slot-profile
   records once at mmap load;
2. precompute each program's suffix-drop execution metadata once;
3. make direct replay consume the prepared records without a second package
   decode or validation pass;
4. replace only three-unit bag sorting with an exact fixed sorting network.

This changes no package bytes, candidate birth, generated surface, geometry
definition, feature, coefficient, score, rank, frontier, proof case,
denominator, authority, SafetyGate, or verifier behavior. Exact prepared-versus-
mmap execution parity and unchanged `77/77` geometry tests are mandatory before
the next micro.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-batch-geometry-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-batch-geometry-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-batch-geometry.time.txt
```

Not tested by this profile: fixed quality percentages, queue-inclusive product
latency, daemon/IBus, or physical behavior. Runtime authority remained
unchanged.

## 24. Prepared Packaged-Execution Micro

The seventh V66 micro changed only package execution preparation. Immutable
paradigm, morph-program, morph-operation, terminal, and slot-profile records are
decoded once when the mmap-backed package is opened. Direct recovery replay
then consumes prepared operations in one pass; the package bytes, generated
surfaces, candidates, scores, ranks, frontiers, and readout authority remain
unchanged. Local focused verification completed `77/77` tests.

Against the same immutable `13 x 10 x 2`, `260` cases:

```text
metric                                  batch geometry   prepared execution
H / B / S0                                127/127/127          127/127/127
H -> B losses                                       0                    0
B -> S0 losses                                      0                    0
raw top-1 / base top-1                          31/31                31/31
BaseProjection failures                             0                    0
uncertified demotions                                0                    0
probe parity failures                                0                    0
false singleton                                      0                    0
integrity errors                                      0                    0
maximum class p99                            59.409 ms            57.442 ms
proof user CPU                                  23.09 s              20.66 s
proof wall                                      13.27 s              13.13 s
constant cache                               311,540 B         10,766,704 B
proof peak RSS                            287,252 KiB          308,792 KiB
productive package                         17,309,944 B         17,309,944 B
runtime authority changed                         false                false
```

Prepared execution reduced user CPU by `10.52%`, but maximum class p99 by only
`3.31%`, while adding `10,455,164 B` of constant decoded state and `21,540 KiB`
to the measured peak RSS. The verdict remains
`FAIL_measured_shadow_gates`: `57.442 ms` is `11.49x` above the unchanged
`5 ms` gate. The fixed `13 x 100 x 2` proof, integration, installation, and
runtime authority transfer remain forbidden.

The retained implementation removes repeated package-record decoding from
direct recovery execution. The next permitted action is a symbolized profile
of this exact prepared runtime. No further code change is authorized until that
profile identifies one shared remaining hot mechanism. Candidate birth,
surfaces, scores, coefficients, limits, proof denominators, SafetyGate, and
verifier behavior remain frozen.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-execution-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-execution-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-execution-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-prepared-execution.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 16. Bounded-Frontier Micro

The next V66 experiment applied the already specified `32`-identity productive
lane before exact exposed-form replay. The candidate order used only independent
observed-source count, learned recovery support/stability, paradigm
support/stability, canonical preference, grounded support, and stable IDs.

Against the same `260` immutable cases:

```text
metric                               first micro   bounded frontier
H / B / S0                          127/127/127       127/127/127
raw top-1 / base top-1                    31/31             31/31
BaseProjection failures                      0                 0
uncertified demotions                         0                 0
probe parity failures                         0                 0
maximum class p99                    464.644 ms        193.299 ms
proof wall                              28.51 s           14.31 s
exact replay programs                14,025,181         2,106,966
retained bindings                         3,631             2,263
runtime authority changed                 false             false
```

Verdict remains `FAIL_measured_shadow_gates`. The experiment proves that the
physical recovery frontier is required and rank/safety preserving, but also
proves it is not the remaining latency owner. Reverse execution still processed
`1,276,120` post-intersection paths and materialized them through a large
temporary map. Exact validation also rebuilt the same exposed-form sets and
operation vectors for every candidate program.

The next permitted change preserves the same candidate set and order while:

1. constructing exposed constraints once per POS basin;
2. reusing decoded source scalars and output buffers;
3. removing the million-entry intermediate recovery-work map;
4. executing package operations directly from mmap records.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-bounded-frontier-13x10-receipt.json`

No larger proof or installation was authorized. Runtime authority remained
unchanged.

## 17. Allocation-Cut Micro

The third V66 micro preserved the bounded-frontier candidate set and order but
removed the million-entry recovery-work map. It now constructs exposed
constraints once per POS basin, executes packaged operations directly from mmap
records, and reuses recovery source/output buffers.

Against the same `260` immutable cases:

```text
metric                               bounded frontier   allocation cut
H / B / S0                              127/127/127       127/127/127
raw top-1 / base top-1                        31/31             31/31
BaseProjection failures                          0                 0
uncertified demotions                             0                 0
probe parity failures                             0                 0
false singleton                                   0                 0
integrity errors                                   0                 0
maximum class p99                        193.299 ms        194.462 ms
proof wall                                  14.31 s           14.33 s
proof peak RSS                         305,008 KiB       287,188 KiB
runtime authority changed                     false             false
```

The measured verdict remains `FAIL_measured_shadow_gates`. The change reduced
RSS without changing coverage, rank, safety, or closed-call latency. It is
therefore retained as a structural memory improvement but does not authorize a
larger proof or installation.

The closed-call timer starts after cold binding derivation and after the probed
readout. Recovery birth, exact binding validation, and the removed recovery-work
map are outside this timer. The remaining measured route is:

```text
evaluate_shadow_with_cold_bindings
-> traverse_binding
-> execute_packaged_program
-> GeometryTraversalStateV1 per generated program
-> emit_normalized_str
-> OSA + keyboard + atom/simhash geometry
-> scoring/readout
```

The next permitted implementation change is score-preserving reuse inside this
timed geometry route only: rotate preallocated OSA rows, reset one geometry
state per binding, batch keyboard conversion, and reuse source/output scratch.
Exact geometry and complete-trie parity remain mandatory. No score,
coefficient, proof case, denominator, authority threshold, SafetyGate, or
verifier behavior may change.

Receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-allocation-cut-13x10-receipt.json`

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 18. Timed Geometry-Reuse Micro

The fourth V66 micro changed only the timed geometry implementation. It rotates
three preallocated OSA rows, reuses one observed geometry state per direct
binding, batches keyboard normalization into reusable storage, and reuses one
decoded source and output buffer per binding. A new scalar-versus-batch test
proved exact terminal-evidence parity before the remote run.

Against the same `260` immutable cases:

```text
metric                                  allocation cut   geometry reuse
H / B / S0                                127/127/127       127/127/127
raw top-1 / base top-1                          31/31             31/31
BaseProjection failures                            0                 0
uncertified demotions                               0                 0
probe parity failures                               0                 0
false singleton                                     0                 0
integrity errors                                     0                 0
maximum class p99                          194.462 ms        138.003 ms
proof peak RSS                           287,188 KiB       287,184 KiB
runtime authority changed                       false             false
```

The measured verdict remains `FAIL_measured_shadow_gates`. The exact candidate
and readout result is unchanged, and the closed-call p99 decreased by `29.03%`,
but it remains `27.60x` above the unchanged `5 ms` gate. The larger fixed proof
and installation remain forbidden.

The retained implementation removes repeated observed-geometry construction.
The next permitted action is a function-level profile of the identical micro;
another code change is not authorized until that profile identifies the first
shared remaining mechanism. Score, coefficients, candidate limits, proof data,
denominators, authority, SafetyGate, and verifier remain frozen.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-geometry-reuse-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-geometry-reuse-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-geometry-reuse.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 19. Symbolized Geometry Profile

The authorized function-level profile used the identical `13 x 10 x 2`
shadow micro and a separate release binary with unchanged `opt-level=3`, LTO,
and one codegen unit. Debug symbols and frame pointers were added only for
attribution. The profiled receipt retained `127/127/127`, `31/31`, exact probe
parity, zero safety failures, and unchanged runtime authority. Profiled latency
is diagnostic only and is not a promotion measurement.

The `cpu_atom/cycles` self-cost profile identified one shared mechanism:

```text
AtomLaneAccumulatorV1::add_simhash_atom       18.33%
AtomLaneAccumulatorV1::add_typed_atom         11.88%
AtomLaneAccumulatorV1::terminal_evidence       5.45%
BTreeMap<u64, u32>::entry/or_default            5.08%
AtomLaneAccumulatorV1::append_simhash_unit      4.35%
BTreeMap<u64, u32>::remove                      2.60%
typed_atom_key                                  2.12%
hash_atom                                       1.12%
```

The report also measured allocator and movement cost (`memmove 4.11%`,
`free 2.74%`, `malloc 2.34%`) consistent with rebuilding B-tree nodes while
each generated candidate is emitted and then reset. OSA itself was only
`3.28%`. Cold binding derivation appeared separately at `4.59%` and remains
outside the closed-call timer.

This rejects further OSA or recovery-frontier work as the immediate next
experiment. The next permitted code change is one score-preserving mechanism:
replace generated atom/simhash B-tree refcount tables with reusable,
deterministically hashed `u64` tables whose `clear()` retains allocation. The
typed atom identities, weights, deduplication, terminal checkpoint/restore,
simhash support, candidate order, and all authority gates remain unchanged.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-geometry-reuse-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-geometry-reuse-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-profile.time.txt
```

Not tested by this profile: fixed quality percentages, queue-inclusive product
latency, daemon/IBus, or physical behavior. Runtime authority remained
unchanged.

## 22. Reusable Batch-Geometry Micro

The sixth V66 micro kept incremental branch geometry for the frozen V64
complete-trie lane and moved only fully materialized V66 recovery surfaces to a
reusable batch evaluator. The batch lane emits the same typed and simhash atoms,
sorts and deduplicates them, merges them against the immutable observed profile,
and returns exact terminal evidence. A dedicated RU/EN/layout/punctuation/empty
surface cross-product test proved batch-versus-incremental parity before the
remote release build. Local focused verification completed `77/77` tests.

Against the same `260` immutable cases:

```text
metric                                  reusable atom hash   batch geometry
H / B / S0                                  127/127/127        127/127/127
H -> B losses                                         0                  0
B -> S0 losses                                        0                  0
raw top-1 / base top-1                            31/31              31/31
BaseProjection failures                               0                  0
uncertified demotions                                  0                  0
probe parity failures                                  0                  0
false singleton                                        0                  0
integrity errors                                        0                  0
maximum class p99                              102.744 ms          59.409 ms
proof wall                                            n/a            13.27 s
proof peak RSS                                        n/a       285,328 KiB
runtime authority changed                         false              false
```

The exact sampled coverage, rank, parity, and safety result remained unchanged,
while closed-call p99 decreased by `42.18%`. The measured verdict remains
`FAIL_measured_shadow_gates`: `59.409 ms` is still `11.88x` above the unchanged
`5 ms` gate. The fixed `13 x 100 x 2` proof, integrated product gates, and
installation remain forbidden.

The next permitted action is a symbolized profile of this exact batch-geometry
runtime. No candidate, score, coefficient, frontier, proof denominator,
authority, SafetyGate, or verifier change is authorized before that profile
identifies the first shared remaining mechanism.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-batch-geometry-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-batch-geometry-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-batch-geometry.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 20. Reusable Atom-Hash Micro

The fifth V66 micro replaced only generated atom and simhash B-tree refcount
tables with reusable deterministic `u64` hash tables. Atom identities,
deduplication, weights, checkpoint/restore, geometry evidence, scores, and
candidate order were unchanged. Local verification completed `76/76` focused
tests, including materialized-geometry parity and reset/reuse parity.

Against the same `260` immutable cases:

```text
metric                                  geometry reuse   reusable atom hash
H / B / S0                                127/127/127          127/127/127
raw top-1 / base top-1                          31/31                31/31
BaseProjection failures                            0                    0
uncertified demotions                               0                    0
probe parity failures                               0                    0
false singleton                                     0                    0
integrity errors                                     0                    0
maximum class p99                          138.003 ms           102.744 ms
runtime authority changed                       false                false
```

The exact result remained unchanged and closed-call p99 decreased by `25.55%`.
The measured verdict remains `FAIL_measured_shadow_gates`: p99 is still
`20.55x` above the unchanged `5 ms` gate. The implementation is retained, but
the fixed full proof and installation remain forbidden.

The previous symbolized profile is now stale for owner ranking because its
largest B-tree mechanism has been replaced. The next permitted action is a new
symbolized profile of this exact runtime. No further code change is authorized
before that profile identifies the next shared owner.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-atom-hash-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-atom-hash-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-atom-hash.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 21. Post-Hash Symbolized Profile

The authorized profile was repeated after the reusable atom-hash change. It
used the same immutable micro and a separate symbolized release binary. The
profile receipt again preserved all coverage, rank, parity, and safety values;
its latency is diagnostic only.

The `cpu_atom/cycles` self-cost ranking changed to:

```text
AtomLaneAccumulatorV1::add_simhash_atom       21.58%
AtomLaneAccumulatorV1::add_typed_atom         10.64%
MorphProgramHeader package record              6.48%
MorphOp package record                         6.14%
AtomLaneAccumulatorV1::terminal_evidence       4.14%
OsaLaneV1::emit                                 3.95%
execute_packaged_program_into                   3.66%
append_simhash_unit                             3.39%
append_typed_unit                               3.30%
```

No B-tree atom entry/removal owner remains. The dominant mechanism is now the
incremental atom/simhash refcount and undo protocol itself. That protocol is
required for branching complete-trie traversal, but V66 direct replay already
materializes a complete normalized surface before geometry evaluation.

The next permitted change therefore separates two exact execution forms:

```text
BaseV64 complete trie -> incremental branch geometry
RecoveredV66 direct surface -> reusable batch geometry reduce
```

The batch lane must generate the same typed atoms and simhash atoms, sort and
deduplicate them, merge against the immutable observed atom profile, and return
byte-identical terminal evidence. A dedicated cross-product parity test is
mandatory. Candidate birth, surfaces, scores, rank, limits, and authority do
not change.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-atom-hash-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-atom-hash-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-atom-hash.time.txt
```

Not tested by this profile: fixed quality percentages, queue-inclusive product
latency, daemon/IBus, or physical behavior. Runtime authority remained
unchanged.

## 26. Per-Readout Geometry-Memo Micro

The eighth V66 micro added one score-preserving memo to the direct recovery
geometry evaluator. Its lifetime is one readout, its key is the complete
normalized generated surface, and its value is the complete
`GeometryTerminalEvidenceV1` for the immutable observed surface. A repeated
cache-hit test proved exact equality with an independently executed incremental
geometry evaluator. Local verification completed `77/77` focused tests.

Against the same immutable `13 x 10 x 2`, `260` cases:

```text
metric                              prepared execution   geometry memo
H / B / S0                               127/127/127       127/127/127
H -> B losses                                      0                 0
B -> S0 losses                                     0                 0
raw top-1 / base top-1                         31/31             31/31
BaseProjection failures                            0                 0
uncertified demotions                               0                 0
probe parity failures                               0                 0
false singleton                                     0                 0
integrity errors                                     0                 0
maximum class p99                           57.442 ms         12.624 ms
proof user CPU                                 20.66 s           16.97 s
proof peak RSS                           308,792 KiB       308,724 KiB
productive package                        17,309,944 B      17,309,944 B
runtime authority changed                        false             false
```

Maximum class p99 decreased by `78.02%` and user CPU by `17.86%`; package
bytes, cache package state, candidate result, rank, parity, and authority did
not change. The verdict remains `FAIL_measured_shadow_gates`: `12.624 ms` is
still `2.52x` above the unchanged `5 ms` gate. The fixed proof, integration,
installation, and authority transfer remain forbidden.

The next permitted action is a symbolized profile of this exact memoized
runtime. No second cache, frontier change, coefficient change, or candidate
change is authorized until the profile identifies the first shared residual
mechanism.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-geometry-memo-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-geometry-memo-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-geometry-memo-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-geometry-memo.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 28. Unified Prepared-Direct Execution Micro

The ninth V66 micro kept the sidecar-disabled runtime on the complete V64 trie
as the independent oracle and routed sidecar-backed V66 BaseV64 and RecoveredV66
bindings through the same prepared direct executor and per-readout geometry
memo. A focused package test compared the complete readout from both executors
before the remote run. Local verification completed `77/77` focused tests.

Against the same immutable `13 x 10 x 2`, `260` cases:

```text
metric                                  geometry memo   unified direct
H / B / S0                                127/127/127     127/127/127
H -> B losses                                       0               0
B -> S0 losses                                      0               0
raw top-1 / base top-1                          31/31           31/31
BaseProjection comparisons / failures          260/0           260/0
uncertified demotions                               0               0
probe parity failures                               0               0
false singleton                                     0               0
integrity errors                                     0               0
maximum class p99                            12.624 ms        8.480 ms
proof user CPU                                  16.97 s          16.69 s
proof peak RSS                            308,724 KiB      308,716 KiB
productive package                         17,309,944 B     17,309,944 B
runtime authority changed                         false           false
```

Maximum class p99 decreased by `32.83%`. The independent complete-trie oracle
and experimental prepared-direct projection remained exactly equal in all
`260` comparisons. Package bytes, candidates, scores, ranks, probe parity,
safety, and authority did not change.

The verdict remains `FAIL_measured_shadow_gates`: `8.480 ms` is `1.70x` above
the unchanged `5 ms` gate. The fixed proof, integration, installation, and
authority transfer remain forbidden. The next permitted action is a symbolized
profile of this exact unified-direct runtime; no candidate or scoring change is
authorized.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-unified-direct-replay-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-unified-direct-replay-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-unified-direct-replay-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-unified-direct-replay.time.txt
```

Not tested in this experiment: fixed `13 x 100 x 2`, slot-heldout,
multi-label, unsupported, integrated L1.1/L3/L4/verifier transfer,
queue-inclusive service latency, daemon/IBus, and physical product behavior.
Runtime authority remained unchanged.

## 29. Post-Unification Symbolized Profile

The authorized profile repeated the identical micro after unified prepared
direct execution. It retained exact coverage, rank, BaseProjection, probe
parity, safety, package bytes, and unchanged runtime authority.

Within the closed-call owner, `evaluate_checked` accounted for `29.97%` of
whole-command cycles and `traverse_binding` for `28.00%`. Its first shared
residual mechanism is now terminal simhash materialization before bounded
selection:

```text
batch_simhash                                    5.53%
batch u32 sorting                                2.23%
OSA                                              2.94%
typed atom-family construction                   1.76%
feature input                                    1.63%
binding frontier selection                       1.58%
feature extraction                               1.28%
```

The two simhash fields are preserved in `GeometryTerminalEvidenceV1` and in
the final candidate equality contract, but repository-wide ownership confirms
that neither field enters feature extraction, scoring, ranking, frontier
retention, calibration, or authority. Computing them for every pre-frontier
candidate therefore buys no decision evidence.

The next permitted change is exact deferred materialization:

```text
all generated candidates
  -> distance + similarity + typed-atom evidence
  -> unchanged scoring and bounded selection

selected returned candidates only
  -> exact character and keyboard simhash
  -> complete GeometryTerminalEvidenceV1
  -> unchanged readout/probe/BaseProjection equality
```

No field may be removed or approximated. A focused test must compare the
deferred completed evidence with the independent incremental evaluator and
must prove that ranking evidence differs only by the two not-yet-materialized
simhash fields. Any final-candidate, score, rank, frontier, parity, or authority
difference rejects the change.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-unified-direct-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-unified-direct-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-unified-direct-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-unified-direct-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-unified-direct-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-unified-direct.time.txt
```

Not tested by this profile: fixed `13 x 100 x 2`, slot-heldout, multi-label,
unsupported, integrated L1.1/L3/L4/verifier transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 32. Post-Deferral Profile And Prepared Sidecar Design

The authorized symbolized profile repeated the immutable micro after deferred
simhash. It preserved `127/127/127` H/B/S0, `31/31` raw/base top-1, exact
`260/260` BaseProjection and probe parity, zero false singleton, zero integrity
errors, unchanged package bytes, and unchanged runtime authority. Profiling
overhead raised maximum class p99 to `12.815 ms`; that value is diagnostic and
does not replace the normal release measurement.

The self-cost ranking moved away from simhash and exposed one shared recovery
preparation defect:

```text
derive_cold_lemma_bindings_with_diagnostics            11.88%
repeated SlotPhaseProfile package decode                8.25%
paradigm_reconstructs_exposed_forms                     6.76%
String extension during packaged replay                 5.58%
execute_packaged_program_into                           4.47%
BTreeMap insertion                                      4.23%
allocator free / malloc                            3.86% / 3.67%
```

The recovery sidecar was mmap-backed but decoded each program's operation
records into a new `Vec` for every recovered path execution. The immutable
micro executed `1,276,120` such paths. This repeated decoding and allocation is
not evidence, scoring, ranking, or authority work.

The authorized V66 change prepares sidecar indexes, postings, programs,
suffix-drop metadata, and operations once during checked mmap load. Replay then
uses exact immutable operation slices. Package bytes and path order remain
unchanged; the additional resident cache is measured explicitly rather than
hidden. The first local implementation passed `77/77` focused tests after the
cache-size assertion was changed from an obsolete main-package-only formula to
the complete prepared-record formula.

This is not yet a latency or quality result. The verdict remains
`FAIL_measured_shadow_gates` until the normal stripped remote micro passes the
same parity, safety, package, RSS, and `5 ms` gates.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-deferred-simhash-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-deferred-simhash-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-deferred-simhash-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-deferred-simhash-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-deferred-simhash-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-deferred-simhash.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-prepared-sidecar-focused-tests.txt
```

Not tested by the local prepared-sidecar implementation: normal remote
latency, fixed `13 x 100 x 2`, integrated transfer, daemon/IBus, or physical
product behavior. Runtime authority remained unchanged.

## 33. Prepared-Sidecar Remote Micro

The eleventh V66 micro evaluated the prepared sidecar with the same stripped
release profile, package hashes, immutable `13 x 10 x 2` cases, and `20`
workers:

```text
metric                                deferred simhash   prepared sidecar
H / B / S0                                127/127/127       127/127/127
H -> B losses                                       0                 0
B -> S0 losses                                      0                 0
raw top-1 / base top-1                          31/31             31/31
BaseProjection comparisons / failures          260/0             260/0
probe parity failures                               0                 0
false singleton / integrity errors                0/0               0/0
maximum class p99                            11.409 ms          9.617 ms
proof user CPU                                  16.55 s           16.39 s
proof peak RSS                            306,768 KiB      310,608 KiB
constant prepared cache                   10,766,704 B      14,423,032 B
productive package                         17,309,944 B      17,309,944 B
runtime authority changed                         false             false
```

Maximum class p99 improved by `15.71%` from the immediately preceding normal
run, while user CPU improved by `0.97%`. The prepared cache increased by
`3,656,328 B` and measured peak RSS by `3,840 KiB`. This is an exact but weak
trade: it preserves the complete contract, yet remains `1.92x` above the
unchanged `5 ms` gate.

The verdict remains `FAIL_measured_shadow_gates`; fixed proof, integration,
installation, and authority transfer remain forbidden. Because the p99 sample
is a class maximum and the CPU effect is small, the change is neither promoted
nor rejected before profiling this exact implementation.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-prepared-sidecar.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-prepared-sidecar.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-sidecar-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-sidecar-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-sidecar-13x10.time.txt
```

Not tested: fixed `13 x 100 x 2`, integrated transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 48. Proof-Worker Concurrency Matrix And Gate Separation

The unchanged dense-flags ELF was stripped once and executed over the same
immutable `13 x 10 x 2` cases with only the proof-worker count varied:

```text
workers   maximum class p99   maximum class p50   closed proof wall
1                   3.783 ms               1.183 ms          1.692 s
2                   3.781 ms               1.186 ms          0.859 s
4                   3.790 ms               1.173 ms          0.454 s
8                   4.906 ms               2.042 ms          0.331 s
20                 11.565 ms               2.222 ms          0.231 s
```

Every run preserved `127/127/127` H/B/S0, zero H-to-B and B-to-S0 losses,
`31/31` raw/base top-1, exact BaseProjection and probe parity, zero false
singleton, zero integrity errors, identical package bytes, and unchanged
runtime authority. Request work did not change; only concurrent proof requests
did.

This proves two independent quantities that the previous gate accidentally
combined:

```text
single-request closed-call latency      measure with one proof worker
proof throughput                        measure with all 20 proof workers
20-client queue-inclusive latency       later product/service gate
```

Increasing proof concurrency reduced wall time by `86.34%` from one to twenty
workers while increasing the measured request tail by `205.71%`. A twenty
worker closed-call sample is therefore a contention test, not the standalone
request p99. Section 11.6 remains valid: scheduler throughput does not make one
request faster. The correction is that scheduler contention also must not be
misreported as intrinsic single-request latency.

The micro's strict per-class `>95%` gate is also non-identifiable at denominator
`10`: one miss maps directly from `100%` to `90%`. Four classes measured `90%`
because each contained one miss, while the aggregate H/B/S0 and parity contract
remained unchanged. The micro remains an exact structural/regression gate; the
fixed `13 x 100 x 2` denominator owns the per-class percentage decision.

The next authorized measurements therefore use the same binary and fixed full
cases twice: one worker owns request p99, and twenty workers own proof
throughput and independently repeated quality totals. Promotion requires exact
quality equality between the two receipts, strict per-class `>95%` on the
fixed denominator, one-worker p99 `<=5 ms`, and later success of the distinct
queue-inclusive 20-client product gate.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-hot-scheduler-workers1-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-hot-scheduler-workers2-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-hot-scheduler-workers4-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-hot-scheduler-workers8-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-hot-scheduler-workers20-13x10.stdout.json
```

Not tested by the matrix: fixed quality percentages, queue-inclusive product
latency, integrated transfer, daemon/IBus, or physical product behavior.
Runtime authority remained unchanged.

## 46. Streaming Exact Replay Local Gate

Target-blind exact replay now streams copy and segment emissions into a
byte-exact matcher over the immutable exposed surfaces. The same generic
program executor owns source-range resolution, operation ordering, termination,
non-empty output, and scalar bounds for both streaming comparison and final
candidate materialization. No runtime word, suffix, coefficient, or fixture
condition was added.

```text
cargo check --lib                                      PASS
focused productive_v1 tests                           80/80 PASS
focused failures                                           0
runtime authority changed                              false
```

The focused gate includes ordered/deduplicated exposed offsets, Unicode source
and segment streaming, exact no-match behavior, complete-trie replay parity,
dense diagnostic memberships, and the existing compiler/runtime proof suite.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-streaming-exact-replay-focused-tests.txt
```

Not tested locally: normal stripped latency, fixed `13 x 100 x 2`, integrated
transfer, queue-inclusive service latency, daemon/IBus, or physical product
behavior. The verdict remains `FAIL_measured_shadow_gates`; runtime authority
remains unchanged.

## 34. Post-Sidecar Profile And Prepared Slot Search

The symbolized prepared-sidecar profile preserved the same coverage, rank,
parity, safety, package, and authority values. Its self-cost ranking showed that
the sidecar preparation did not remove the first shared cold-binding owner:

```text
derive_cold_lemma_bindings_with_diagnostics            11.63%
repeated SlotPhaseProfile package decode                8.08%
paradigm_reconstructs_exposed_forms                     6.53%
String extension during packaged replay                 6.16%
execute_packaged_program_into                           5.73%
BTreeMap insertion                                      5.41%
allocator malloc / free                            4.04% / 2.95%
```

`SlotPhaseProfileRecordV1` is already decoded into the runtime's immutable
prepared array during checked load. Nevertheless, structural eligibility read
the same rows from mmap into a new vector, sorted and deduplicated that vector,
and discarded it once per paradigm and readout. The package compiler emits each
paradigm's target slots in strict `slot_id` order from a `BTreeSet`.

The next authorized exact change validates that strict order once at runtime
load and performs observed-slot binary search directly over the existing
prepared profile slice. It adds no package bytes and no resident cache. A
malformed or non-canonical package fails closed at load. Candidate birth,
surface replay, scores, ranks, frontiers, diagnostics, and authority do not
change.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-sidecar-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-sidecar-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-sidecar-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-prepared-sidecar-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-prepared-sidecar-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-prepared-sidecar.time.txt
```

Not tested by this profile: fixed proof, integration, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 47. Streaming Replay Rejection And Timer Ownership Correction

The streaming exact-replay experiment preserved the complete structural and
quality contract but failed both its optimization hypothesis and the latency
gate:

```text
metric                                  dense flags   streaming replay
H / B / S0                             127/127/127        127/127/127
H -> B / B -> S0 losses                        0/0                0/0
raw top-1 / base top-1                       31/31              31/31
BaseProjection comparisons / failures       260/0              260/0
probe parity failures                            0                  0
false singleton / integrity errors             0/0                0/0
maximum class p99                          11.779 ms          11.675 ms
closed proof wall                          248.076 ms         261.064 ms
proof user CPU                               15.41 s            15.51 s
external peak RSS                       308,368 KiB        308,528 KiB
runtime authority changed                      false              false
```

The DWARF profile attributed `17.02%` self cost to
`paradigm_reconstructs_exposed_forms` and `9.99%` to `memcmp`: replacing one
linear materialization with repeated prefix comparisons made the cold replay
owner more expensive. The implementation is rejected and the owner file is
restored byte-for-byte to source SHA-256
`e0567223ea395eb4514b4169e2865cddf5584c246605baf9131562e20ffe158c`.

More importantly, source-level timer inspection proves that exact replay is
outside the `latency_us` interval. The measured interval begins only after cold
groundings, the frozen base readout, and the independently probed readout are
complete:

```text
cold binding derivation and exact exposed replay       outside timer
base readout                                            outside timer
probed readout                                          outside timer
Instant::now()
  -> evaluate_shadow_with_cold_bindings                 measured
     -> evaluate_checked
        -> traverse_binding
        -> batch geometry / frontier / surface basins
elapsed                                                 class latency sample
clean preservation readout                              outside timer
```

Therefore the whole-command profile cannot authorize another exact-replay
change for the `5 ms` gate. The next experiment uses the unchanged dense-flags
ELF and immutable cases across `1/2/4/8/20` proof workers. It tests whether the
single-sample micro p99 tail scales with actual hot-readout work or with
concurrent scheduler contention. No package, source, candidate, coefficient,
frontier, denominator, or runtime authority changes.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-dwarf-streaming-exact-replay.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-streaming-exact-replay-dwarf-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-streaming-exact-replay-dwarf-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-streaming-exact-replay-dwarf-self-report-no-inline.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-streaming-exact-replay-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-streaming-exact-replay-13x10.time.txt
```

Not tested: fixed `13 x 100 x 2`, integrated transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 35. Prepared Slot-Search Remote Micro

The twelfth V66 micro used direct binary search over the already prepared,
load-validated slot-profile ranges. Against the same immutable `260` cases:

```text
metric                                prepared sidecar   prepared slot search
H / B / S0                                127/127/127          127/127/127
H -> B / B -> S0 losses                           0/0                  0/0
raw top-1 / base top-1                          31/31                31/31
BaseProjection comparisons / failures          260/0                260/0
probe parity failures                               0                    0
false singleton / integrity errors                0/0                  0/0
maximum class p99                             9.617 ms             7.230 ms
proof user CPU                                  16.39 s              15.92 s
proof peak RSS                            310,608 KiB         309,168 KiB
constant prepared cache                   14,423,032 B         14,423,032 B
productive package                         17,309,944 B         17,309,944 B
runtime authority changed                         false                false
```

Maximum class p99 improved by `24.82%`, user CPU by `2.87%`, and peak RSS by
`1,440 KiB` without adding cache or package bytes. Exact candidate and probe
parity prove that the previous per-paradigm mmap decode, vector construction,
sort, and dedup were redundant execution only.

The result is retained, but the verdict remains `FAIL_measured_shadow_gates`:
`7.230 ms` is `1.45x` above the unchanged `5 ms` gate. Fixed proof,
integration, installation, and authority transfer remain forbidden. The next
permitted action is a symbolized profile of this exact runtime.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-prepared-slot-search-focused-tests.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-prepared-slot-search.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-prepared-slot-search.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-slot-search-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-slot-search-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-prepared-slot-search-13x10.time.txt
```

Not tested: fixed proof, integrated transfer, queue-inclusive service latency,
daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 56. V69 Preflight Rejection And V70 Transition-Equivalence Paper

Date: 2026-08-12. V69 preserved the byte-identical V68 package and added the
previously authorized shared-hypothesis join. The release binary was built from
source files whose hashes exactly match the local checkout. Before any full
`13 x 100 x 2` proof, the immutable twenty-worker `13 x 10 x 2` micro measured
the following workload:

```text
metric                                             V69 micro
evaluated cases                                           260
LEMMA_HELDOUT cases                                       130
H / B                                                   127/127
H -> B losses                                               0
BaseProjection comparisons / failures                   260/0
false singleton / integrity errors                        0/0
productive package bytes                           17,309,944
recovery sidecar bytes                              2,123,112
combined package bytes                              19,433,056
shared hypothesis observations                         439,521
unique shared hypotheses                                91,070
shared join attempts                                 23,499,187
exact shared joins                                       12,071
shared exact-replay program executions              457,965,261
proof wall, workers=20                                   17.45 s
external peak RSS                                    341,968 KiB
runtime authority changed                                  false
```

The package budget passes, and the sampled micro has no `H -> B` loss. The
execution budget fails before full proof. A linear extrapolation from `130` to
`1,300` LEMMA_HELDOUT cases predicts approximately `234,991,870` paradigm join
attempts and `4,579,652,610` exact-replay program executions. This extrapolation
is a workload forecast, not a quality result; the full fixed proof was not run.

The first shared mechanism is the Cartesian join in
`PackagedProductiveRuntimeV1::derive_cold_lemma_bindings_inner`:

```text
each unique (anchor slot, recovered surface) hypothesis
  x every structurally eligible paradigm with that anchor slot
  x every relevant exposed-form program in the paradigm
```

The V69 micro executes an average of `19.49` programs per join attempt. The
shared join therefore repeats byte-identical morphology programs for different
paradigm owners. V69 is `REJECTED_BY_PREFLIGHT`; full proof, integration,
installation, and authority transfer are forbidden.

### 56.1 Transition-equivalence theorem

For one POS basin, recovered anchor slot `a`, and ordered exposed constraint set
`E`, define the executable transition signature of paradigm `P` as the complete
ordered bytes of every packaged program in `P` whose source slot is `a` and
whose target slot occurs in `E`, grouped by target slot. The signature includes
the full program header and every operation byte, including decoder references;
duplicate programs are canonicalized without changing multiplicity-visible
outputs.

If paradigms `P` and `Q` have byte-equal executable transition signatures, then
for every recovered anchor surface `s` they produce the same set of surfaces at
every slot in `E`. Consequently:

```text
P reconstructs all exposed constraints from (a, s)
iff
Q reconstructs all exposed constraints from (a, s)
```

The proof follows directly from deterministic packaged-program execution: both
paradigms execute the same operations over the same source scalars and compare
the resulting surfaces with the same byte-exact constraints. Paradigm evidence,
rank, and ownership are not merged; only the redundant reconstruction boolean
is shared.

A hash may select a candidate bucket, but hash equality is never sufficient.
Full signature bytes must be compared before two paradigms share a replay.

### 56.2 Authorized V70 contour

V70 may change only shared-hypothesis exact replay:

1. Build transition-equivalence classes for the current `(POS, anchor slot,
   exposed slot set)` from already independently eligible paradigms.
2. Keep the ordered owner paradigm IDs for every class.
3. Replay one representative per `(shared hypothesis, exact equivalence class)`.
4. On exact success, create the same independently owned candidate for every
   paradigm in that class; do not merge support, stability, rank, or authority.
5. Preserve BaseV64, the V68 fine-owner lane, frozen `H=1280`, byte-exact exposed
   replay, package bytes, `PRODUCTIVE_PHYSICAL_TOP_K=32`, calibration, and all
   safety gates.
6. Do not use a word, lemma, suffix, proof identity, target identity, corpus row,
   arbitrary class cap, wider frontier, or hash-only equality.

V70 diagnostics must report class count, owner count, maximum class size,
representative replay count, owner fan-out after exact success, and full-byte
signature collision checks.

### 56.3 Mandatory preflight estimator

No later productive version may start a full proof without a preflight receipt
containing:

```text
exact package-byte formula and measured file parity
unique shared-hypothesis count
join-unit count after exact structural deduplication
exact-replay execution count
maximum and p99 fan-out
projected full-proof work from the fixed micro denominator
```

The V70 micro is rejected before full proof if projected full exact-replay work
exceeds `50,000,000` program executions, package bytes change unexpectedly,
`H -> B != 0` on the immutable micro, BaseProjection or probe parity changes,
or any false singleton/integrity error appears. The `50,000,000` limit is a
work budget, not a candidate-quality threshold and not a truncation rule.

Only after that preflight passes may the fixed one-worker and twenty-worker
`13 x 100 x 2` proofs run. Promotion gates remain conjunctive: `H -> B = 0`,
`B -> S0 = 0`, strict per-class top-16 `>95%`, one-worker p99 `<=5 ms`, zero
false singleton, zero integrity errors, exact BaseProjection/probe parity, and
unchanged frozen denominator.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V69_EXACT_REPLAY_2026-08-12/release-build.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V69_EXACT_REPLAY_2026-08-12/package-parity.sha256
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V69_EXACT_REPLAY_2026-08-12/micro-workers20-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V69_EXACT_REPLAY_2026-08-12/micro-workers20-13x10.time.txt
```

Measured by this gate: package parity and bytes, sampled stage retention,
BaseProjection, probe parity, false certainty, integrity, shared-join fan-out,
exact-replay work, proof wall, and process RSS. Not tested: full fixed quality,
standalone one-worker latency, integrated `L1.1 -> L2 -> L3 -> L4 -> verifier`,
queue-inclusive service latency, daemon/IBus, or physical input. Runtime
authority remained unchanged.

## 57. V70 Transition-Equivalence Preflight Verdict

Date: 2026-08-12. V70 implemented the section 56 transition-equivalence
theorem without changing the productive package, recovery sidecar, logical
owner set, recovered candidates, ranking, frontier, or authority. Local
verification passed `81/81` focused tests. The immutable twenty-worker
`13 x 10 x 2` micro then measured:

```text
metric                                             V69             V70
H / B / S0                                  127/127/127     127/127/127
raw top-1 / base top-1                            71/71           71/71
BaseProjection failures                               0               0
probe parity failures                                  0               0
false singleton / integrity errors                   0/0             0/0
logical shared join attempts                  23,499,187      23,499,187
transition-equivalence classes                      n/a          44,413
transition-equivalence owners                       n/a         151,115
maximum owners in one class                         n/a             471
representative replay calls                         n/a      13,214,397
shared exact-replay program executions      457,965,261     331,566,153
exact shared owner candidates                    12,071          12,071
proof wall, workers=20                          17.45 s         16.01 s
external peak RSS                              341,968 KiB     359,728 KiB
productive package bytes                     17,309,944      17,309,944
recovery sidecar bytes                         2,123,112       2,123,112
runtime authority changed                          false           false
```

Full-byte transition equivalence removed `27.60%` of shared replay program
executions and `8.25%` of measured wall time while preserving the sampled
quality counters. It is not sufficient. The fixed-denominator linear workload
forecast is approximately `3,315,661,530` shared replay program executions for
the full `13 x 100` LEMMA_HELDOUT proof, `66.31` times the declared `50,000,000`
preflight budget.

The measured owner compression is highly skewed. A maximum class can own `471`
paradigms, but weighted representative replay is reduced only from `23,499,187`
owner pairs to `13,214,397` classes, or `1.78x`. Full operation-byte identity is
too fine because semantically equivalent transitions may carry different
package-local decoder and variant references. Weakening equality to a hash or
dropping those references would be unsound: equal numeric operation shapes do
not prove equal emitted bytes.

V70 is `REJECTED_BY_PREFLIGHT`. No full proof, integration, installation, or
authority transfer was run. A later runtime version is not authorized until a
paper proves canonical symbolic source-to-output semantics while retaining
byte-exact output equivalence. The paper must also forecast its unique semantic
program count from the fixed micro before any full proof.

The critical review and corrected target architecture are now owned by:

```text
/home/ubu/projects/lay/docs/l2-productive-v80-semantic-transducer-paper.md
```

It corrects one overbroad V70 interpretation: `SEGMENT_POOL` references are
already globally canonical inside one package. V80 therefore canonicalizes the
complete symbolic source-to-output transduction, separates non-output terminal
variant identity from execution semantics, and replaces owner iteration with
exact transient output matching plus owner-set intersection.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V70_TRANSITION_EQUIVALENCE_2026-08-12/local-focused-tests.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V70_TRANSITION_EQUIVALENCE_2026-08-12/release-build.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V70_TRANSITION_EQUIVALENCE_2026-08-12/package-parity.sha256
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V70_TRANSITION_EQUIVALENCE_2026-08-12/micro-workers20-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V70_TRANSITION_EQUIVALENCE_2026-08-12/micro-workers20-13x10.time.txt
```

Measured: package parity, sampled retention and quality, structural program
classes, owner fan-out, representative calls, exact-replay work, proof wall,
RSS, BaseProjection, probe parity, and false certainty. Not tested: fixed full
quality, standalone one-worker p99, integrated transfer, queue-inclusive
service latency, daemon/IBus, physical input, or product behavior. Runtime
authority remained unchanged.

## 55. V68 Full Verdict And V69 Shared-Hypothesis Join Paper

Date: 2026-08-12. V68 retained the byte-identical V64 base package and compiled
the V67 fine-owner recovery field as one executable program per shared key.
The fixed `13 x 100 x 2` proof was run with one worker for standalone latency
and with twenty workers for proof throughput. Both receipts have identical
quality counters:

```text
metric                                      workers=1       workers=20
evaluated cases                                  2,600            2,600
H / B / S0                              1,280/1,276/1,276 1,280/1,276/1,276
H -> B / B -> S0 losses                           4/0              4/0
raw top-1 / base top-1                         277/267          277/267
BaseProjection failures                             0                0
probe parity failures                                0                0
false singleton / integrity errors                 0/0              0/0
one-worker maximum class p99                  4.623 ms                n/a
twenty-worker proof wall                            n/a          1.947 s
productive package                          17,309,944 B     17,309,944 B
recovery sidecar                             2,123,112 B      2,123,112 B
runtime authority changed                         false            false
```

V68 reduced recovery program executions from `19,507,413` fine-owner paths to
`3,550,871` shared executions on the full proof. It did not change the first
quality loss. All four `H -> B` failures are the same proof event and target
lemma. The oracle paradigm is independently slot-compatible and occurs in a
recovery posting, but no recovered anchor owned by that fine posting reproduces
all exposed forms. The predeclared V67 branch therefore remains active:

```text
shared reverse program exists
-> no exact fine-owner anchor for the oracle paradigm
-> join recovered shared hypothesis with independently eligible paradigms
-> retain only a complete byte-exact exposed-form reconstruction
```

Exact replay is outside the closed-call latency timer. The one-worker V68 p99
already passes `<=5 ms`; exact-replay counts are not a reason for a latency
optimization. V69 is authorized only as the following target-blind recovery
birth change:

1. Execute each admitted shared reverse program at most once per observed
   source, as in V68.
2. Deduplicate generated shared hypotheses by `(POS, canonical anchor slot,
   normalized anchor surface)` before any paradigm join.
3. Build a prepared map from anchor slot to the already independently
   slot-compatible paradigms. Do not scan all package paradigms per hypothesis.
4. Join a shared hypothesis only to paradigms in that map and require exact
   reproduction of every exposed `(slot, surface)` before it can enter the
   existing recovery frontier.
5. An exact shared certificate may order a recovered candidate ahead of an
   unverified recovered candidate. It does not enter BaseV64 ordering, copy
   shared support into fine support, or grant Winner authority.
6. Keep `PRODUCTIVE_PHYSICAL_TOP_K = 32`. Do not add a second frontier, widen a
   bound, use proof labels, or add word, lemma, suffix, target-slot, or source-ID
   conditions.

V69 must report shared hypotheses, structural join attempts, exact shared
joins, and exact replay executions. Focused tests must preserve package
validation and deterministic ordering. The immutable `13 x 10 x 2` micro must
preserve BaseProjection, probe parity, raw/base top-1, false singleton,
denominator, and all package bytes except when a format change is explicitly
measured. Only then may the fixed `13 x 100 x 2` proof run. Promotion still
requires `H -> B = 0`, `B -> S0 = 0`, strict heldout top-16 `>95%` for every
class, one-worker p99 `<=5 ms`, zero false singleton, and zero integrity error.

Measured receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V68_SHARED_PROGRAM_2026-08-12/full-workers1-13x100-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V68_SHARED_PROGRAM_2026-08-12/full-workers1-13x100.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V68_SHARED_PROGRAM_2026-08-12/full-workers20-13x100-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V68_SHARED_PROGRAM_2026-08-12/full-workers20-13x100.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V69_EXACT_REPLAY_2026-08-12/control-v67-current-binary-13x10-receipt.json
```

Not tested by V68: integrated `L1.1 -> L2 -> L3 -> L4 -> verifier` transfer,
queue-inclusive daemon latency, IBus, physical input, installation, or live
authority. Runtime authority remained unchanged.

## 53. V67 Shared-Support Micro Verdict

The V67 frozen resume reused all six completed corpus and induction stages and
produced a base package byte-identical to V64/V66. The recovery sidecar carried
every measured shared-certified fine owner without copying shared support into
fine rank:

```text
base .p2m bytes / sha256                     17,309,944 / 9fd8c950...3e438
base V64/V66/V67 byte parity                                      exact
recovery .p2r bytes / sha256                  5,898,104 / 95e976c7...e92ec
recovery indexes / postings                              161 / 62,115
shared-certified support-1 postings                            25,200
recovery programs / operations                         62,115 / 228,351
resume wall / peak RSS                         90.59 s / 649,548 KiB
runtime authority changed                                        false
```

The immutable `13 x 10 x 2` micro preserved all sampled structural and safety
dimensions but failed the conjunctive latency and retained-class gates:

```text
sampled H / B / S0                                      127 / 127 / 127
sampled H -> B / B -> S0                                        0 / 0
raw top-1 / frozen-base top-1                                 31 / 31
BaseProjection comparisons / failures                         260 / 0
probe parity failures                                               0
false singleton / integrity errors                                0 / 0
maximum class p99                                           12.973 ms
prefix truncation top-16                                         9/10
repeated fragment top-16                                         9/10
suffix truncation top-16                                         9/10
verdict                                      FAIL_measured_shadow_gates
```

V67 increased recovery paths from `1,546,928` to `2,581,338` and reverse
program executions from `1,276,120` to `2,103,986`. Readout and base rank did
not change; the added fine owners therefore exposed execution multiplicity,
not a score or authority defect. The larger `13 x 100 x 2` proof remains
forbidden because the micro did not pass.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/resume-build-stage-root-fix-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/resume-build-stage-root-fix.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/micro-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/micro-13x10.time.txt
```

Not tested: fixed `13 x 100 x 2`, integrated transfer, queue-inclusive
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 54. V68 Shared-Program Execution Paper

The V67 support audit measured `16,074` unique target-blind shared keys:

```text
G = (POS, observed_slot, canonical_anchor_slot, reverse_program)
```

The sidecar compiler nevertheless emitted one full program for every fine
posting, producing `62,115` program records. During readout, every eligible
fine posting executed that program independently before its recovered surface
was merged. This is exact duplicate work because program execution depends on
`G` and the observed source only; fine owner evidence is consumed afterwards.

V68 is authorized to change only this ownership split:

1. Compile one program identity for each exact `G` and retain all fine postings
   with their original paradigm, support, stability, flag, and provenance.
2. Sort postings inside each `(POS, observed_slot)` index by program identity
   and then by a deterministic full posting order.
3. Mark posting evidence for every owner and intersect every owner with the
   independently computed structural-eligibility set.
4. Execute a shared program once when its group has at least one eligible fine
   owner, then fan out the exact recovered surface to each eligible owner.
5. Preserve candidate dedup, evidence order, `PRODUCTIVE_PHYSICAL_TOP_K = 32`,
   BaseV64 ordering, exact exposed-form replay, calibration, denominator, and
   runtime authority.

The expected package shape is `62,115` postings and exactly `16,074` programs;
the exact count is a compilation measurement, not a runtime constant. V68 is
rejected if a posting is removed, shared support changes fine rank, a program
group crosses POS/slot/program identity, or any readout differs from V67 apart
from execution counters and latency.

The next gate is again only the immutable `13 x 10 x 2` micro. It must retain
V67 H/B/S0, raw/base top-1, BaseProjection parity, probe parity, zero false
singleton, zero integrity errors, and zero uncertified demotions. It must also
reduce reverse program executions and satisfy maximum class p99 `<=5.000 ms`.
Only a complete micro PASS authorizes the fixed `13 x 100 x 2` proof. Runtime
authority remains unchanged.

## 52. V67 Resume Stage-Root Ownership Correction

The final V67 source passed `80/80` focused `productive_v1` tests. Its remote
release build completed in `2:47.23`, peaked at `2,265,988 KiB` RSS, and the
post-build support audit again proved frozen V66 byte and evidence parity:

```text
frozen definition bytes                         3,880,478 identical
frozen definition count                            36,915 identical
fine shared-certified support-1 definitions         25,200
proposed definition count                           62,115
audit wall / peak RSS                       41.99 s / 72,972 KiB
runtime authority changed                             false
```

The first frozen-resume attempt then stopped during
`resume_induce_shared_support_anchor_recovery` after `52.10 s`, at
`275,732 KiB` peak RSS, before package compilation or proof. The measured error
was `ENOENT`: the reducer wrote through `config.root`, but only its historical
`induction/` caller happened to provide a pre-existing directory. The new
isolated `anchor-recovery-shared-support-v1/` root correctly exposed that
unowned filesystem precondition.

This is an execution defect, not a V67 quality verdict and not an authorization
for V68. The systemic correction makes the anchor-recovery reducer create its
own bounded stage root before any output is written. It does not alter frozen
induction, support, postings, ordering, ranking, denominator, calibration,
proof inputs, or runtime authority.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/release-build.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/anchor-support-audit-final.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/anchor-support-audit-final.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/resume-build.stderr.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/resume-build.time.txt
```

Not tested by the failed resume: compiled V67 package size or parity, fixed
proof quality, runtime latency, integration, daemon/IBus, or physical input.
Runtime authority remained unchanged.

## 51. V67 Shared-Support Recovery-Birth Paper

Date: 2026-08-12. The retained V66 diagnosis is
`ORACLE_EXACT_RECOVERY_ANCHOR_NOT_GENERATED`. The frontier remains fixed at
`PRODUCTIVE_PHYSICAL_TOP_K = 32` per POS basin. This section authorizes one
V67 shadow micro; it does not authorize installation or runtime authority.

### 51.1 Full Support Audit

The audit replayed the complete `443,980,523`-byte classified transition spool
through the unchanged V66 reverse-program derivation. In parallel with the
existing fine-grained key

```text
F = (paradigm_id, observed_slot, canonical_anchor_slot, reverse_program)
```

it counted distinct train lemmas for the target-blind shared key

```text
G = (POS, observed_slot, canonical_anchor_slot, reverse_program).
```

The audit rebuilt the frozen V66 definition spool into a separate scratch
directory and required byte identity, evidence-hash identity, definition-count
identity, and maximum-operation identity before accepting any measurement.

```text
frozen V66 definition spool parity                     byte identical
frozen V66 evidence parity                         evidence identical
fine aggregates                                             67,396
fine definitions with support >=2                          36,915
fine definitions filtered at support=1                     30,481
shared aggregates                                           16,074
shared aggregates with support >=2                          10,793
support=1 fine definitions certified by shared support      25,200
uncertified shared support=1 aggregates                      5,281

metric                              V66 current   shared-certified fine lift
definition postings                     36,915                       62,115
definition spool bytes                3,880,478                    6,513,110
lookup buckets                              159                          161
fan-out p50                                 211                          339
fan-out p95                                 417                          632
fan-out p99                                 499                          976
fan-out max                                 641                        1,054
```

The repeated audit used one dependency-ordered streaming read, completed in
approximately `41 s`, consumed about one CPU, and peaked below `75 MiB` RSS.
The scan is measurement-only; it did not compile a sidecar, execute runtime
readout, run fixed proof, or change authority.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/anchor-support-audit.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V67_SHARED_SUPPORT_2026-08-12/anchor-support-audit.time.txt
/home/e/projects/lay-productive-v1-build-20260811/receipts/v67/anchor-support-audit.json
/home/e/projects/lay-productive-v1-build-20260811/receipts/v67/anchor-support-audit.time.txt
```

### 51.2 Consequence

The support threshold is a real systemic birth obstruction: `25,200` fine
paths have one paradigm-local observation but at least two independent train
lemmas support the same target-blind reverse program across paradigms. Removing
the threshold globally is still invalid. It would erase the distinction between
evidence for a transformation and evidence that a transformation applies to a
specific paradigm, and it would raise maximum lookup fan-out by `64.43%`.

V67 therefore uses two separate facts:

```text
shared support >=2
  -> certifies only that the target-blind reverse program may give birth

fine support >=1 for paradigm P
  -> certifies only that P is an observed applicability owner

shared certificate + fine applicability + independent structural eligibility
  -> may produce a recovery candidate for P

byte-exact replay of every exposed form
  -> may retain that candidate
```

Shared support must not be copied into fine support or used to inflate ranking.
A lifted posting retains fine support `1` and carries a typed shared-certificate
flag. Existing support `>=2` postings remain byte-semantically unchanged.

### 51.3 V67 Authorized Scope

V67 may change only the anchor-recovery definition reduce, sidecar posting
validation, and recovery birth telemetry:

1. Admit a fine support-1 definition only when its exact shared key has support
   from at least two distinct train lemmas.
2. Preserve the fine `paradigm_id`; do not apply a program to an unobserved
   paradigm in V67.
3. Preserve fine support `1` for ranking and carry one known
   `SHARED_SUPPORT_CERTIFIED` posting flag.
4. Preserve `PRODUCTIVE_PHYSICAL_TOP_K = 32`, BaseV64 ordering, frozen H,
   BaseProjection parity, calibration, denominator, and authority.
5. Deduplicate equal `(paradigm, anchor slot, recovered surface)` candidates
   exactly as V66 does. No word, lemma, suffix, proof identity, or target slot
   may enter runtime logic.

The first V67 run is the immutable `13 x 10 x 2` micro. It is accepted only if
all numeric gates from section 10 pass. The package receipt must additionally
report lifted posting count, flag count, sidecar bytes, fan-out, recovery paths,
post-intersection paths, recovered anchors, frontier drops, and exact replays.

### 51.4 Predeclared Failure Branches

V67 is deliberately narrower than a full shared-hypothesis join. Its failure
interpretation is fixed before compilation:

```text
H -> B remains 4 and oracle has no lifted fine posting
  -> V68: bounded shared-hypothesis birth joined with independently eligible
           paradigms; no top_k widening

oracle lifted posting exists but exact anchor is dropped before replay
  -> V68: typed shared-certificate retention by structural evidence group;
           no coefficient tuning

H -> B becomes 0 but p99 exceeds 5 ms
  -> V68: execute each unique shared reverse program once per observed source,
           then fan out the recovered surface to certified fine owners

any BaseProjection, top-1, false-singleton, denominator, or class regression
  -> reject V67 and diagnose the first changed stage before another version
```

No V68 implementation is authorized merely because V67 fails. The matching
predeclared mechanism must first be present in the receipt. Runtime authority
remains unchanged.

## 49. Fixed 13 x 100 x 2 Proof And Promotion Verdict

The immutable proof was executed twice against the same stripped dense V66 ELF
and the same V66 packages. One worker owns the standalone closed-call latency
gate; twenty workers measure proof throughput and contention. Both runs
produced byte-for-byte equivalent quality counters.

```text
metric                                  workers=1        workers=20
evaluated cases                              2,600             2,600
H / B / S0                         1,280/1,276/1,276 1,280/1,276/1,276
H -> B / B -> S0 losses                       4/0               4/0
raw top-1 / base V64 top-1                 277/267           277/267
BaseProjection failures                         0                 0
probe parity comparisons/failures          2,600/0           2,600/0
false singleton / integrity errors             0/0               0/0
maximum class p99                           4.021 ms          11.713 ms
closed proof wall                          15.868 s           1.782 s
peak RSS                                  261.4 MiB          326.8 MiB
verdict                        FAIL_measured_shadow_gates
runtime authority changed                       false
```

The per-class fixed LEMMA_HELDOUT result is:

```text
damage class                       top-16     percent    top-1     p99
adjacent transposition                  96      96.0%       25   3.520 ms
double substitution                     98      98.0%       21   3.857 ms
extra letter                            97      97.0%       22   3.515 ms
layout projection                       95      95.0%       25   3.918 ms  FAIL
letter substitution                     97      97.0%       24   3.556 ms
missing letter                          96      96.0%       22   3.446 ms
non-adjacent transposition              98      98.0%       24   3.056 ms
omission + transposition                98      98.0%       16   3.739 ms
prefix truncation                       97      97.0%       25   3.094 ms
punctuation suffix                      97      97.0%       31   3.477 ms
repeated fragment                       95      95.0%       18   3.832 ms  FAIL
sparse multi-omission                   96      96.0%       17   4.021 ms
suffix truncation                       93      93.0%        7   3.058 ms  FAIL
```

The accepted gate is strict `top-16 > 95%` for every class. Equality at 95%
does not pass. The standalone one-worker p99 gate passes, but the conjunctive
quality contract fails because `H -> B != 0` and three classes do not exceed
95%. V66 is therefore not promotion eligible.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/full-hot-gate-workers1-13x100.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/full-hot-gate-workers1-13x100.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/full-hot-gate-workers20-13x100.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/full-hot-gate-workers20-13x100.time.txt
```

Measured by this gate: fixed quality, stage retention, exact probe parity,
standalone closed-call latency, proof contention, process RSS, and package
integrity. Not tested: queue-inclusive service latency, integrated
L1.1 -> L2 -> L3 -> L4 -> verifier transfer, daemon/IBus, or physical product
behavior. Runtime authority remained unchanged.

## 50. Fixed-Proof First-Loss Diagnosis

All four `H -> B` losses are repeated observations of one proof event and one
target lemma. They have one shared mechanism:

```text
ORACLE_PARADIGM_ABSENT_FROM_SOURCE_SLOT_POSTINGS       4/4
oracle exact-reconstructing paradigms                    1
oracle slot-compatible paradigms                       460
tracked oracle-compatible identity anchor present      yes
tracked recovery exact reconstruction present           no
target-blind recovered anchors                        2,975
target-blind recovery paths                           3,153
physical recovery frontier per POS basin                 32
aggregate retained across two POS basins                 64
```

The target paradigm is representable, slot-compatible, and exactly
reconstructs the full oracle observation. It is absent from direct source-slot
and learned-recovery postings. The initial diagnostic observed some recovered
candidate with the same `paradigm_id`, but did not identify its exact anchor.
`RecoveredAnchorCandidateV1::evidence_order` then mixes thousands of learned and
identity candidates into one POS-local list and truncates it to
`PRODUCTIVE_PHYSICAL_TOP_K = 32` before exact exposed-form replay. This proof
event has two POS basins, hence the aggregate diagnostic count of 64.

The current receipt does not prove whether the tracked compatible identity is
outside that frontier or enters the frontier and then fails target-blind exact
replay. The previous stronger attribution to truncation is withdrawn. A
target-blind post-frontier paradigm trace is required before selector changes
are authorized.

The required trace was then executed against the unchanged V66 package on the
fixed `13 x 100` proof with `20` workers:

```text
tracked paradigm with recovered candidate       {1: 4}
tracked paradigm after frontier                  {0: 4}
tracked paradigm after exact                     {0: 4}
classified first loss
  paradigm-level candidate dropped                 4
```

This closed the A/B distinction only at `paradigm_id` granularity: the oracle
paradigm had at least one recovered candidate and no candidate for that paradigm
survived the mixed frontier. It did not prove that the frozen oracle binding's
exact source-anchor was generated. The receipt is:

```text
/home/e/projects/lay-productive-v1-build-20260811/receipts/v66/frontier-trace-13x100-receipt.json
```

The next shadow-only experiment is therefore authorized to partition that
frontier by target-blind evidence provenance. Learned reverse recovery and a
structural identity bridge each retain at most `32` candidates per POS basin;
their survivors are merged back into the existing deterministic evidence order
before exact replay. This does not widen either evidence lane, use proof labels,
change the base V64 lane, or grant authority. It can execute at most `64`
recovery candidates per POS basin instead of `32`, so fixed quality and
single-worker latency must both be re-proved.

That experiment was implemented and rejected. It made a candidate for the
oracle paradigm survive the target-blind frontier in all four repeated cases,
but exact exposed-form replay then rejected that candidate:

```text
metric                                      mixed frontier   typed frontiers
H / B / S0                              1280/1276/1276    1280/1276/1276
H -> B / B -> S0 losses                         4/0                4/0
tracked paradigm with recovered candidate       1                  1
tracked paradigm after frontier                  0                  1
tracked paradigm exact reconstruction            0                  0
classified first loss                DROPPED_BY_FRONTIER  FAILS_EXACT_REPLAY
base projection failures                         0                  0
uncertified demotions                            0                  0
raw top-1 / base top-1                      277/267            277/267
20-worker maximum class p99                11.579 ms          19.455 ms
runtime authority changed                      false              false
```

The typed selector therefore proves that the mixed frontier was the first
paradigm-level obstruction, but it is not the causal repair for `H -> B`: it
only admits another anchor of the same paradigm, which fails target-blind exact
replay, while nearly doubling candidate replay work. The selector change is
removed from the retained source. No package was rebuilt and no runtime
authority changed.

Rejected-experiment receipt:

```text
/home/e/projects/lay-productive-v1-build-20260811/receipts/v66/typed-frontier-full-workers20-13x100-receipt.json
```

An exact source-anchor trace then compared identity-bridge candidates by
`(paradigm_id, canonical_anchor_slot, normalized_anchor_surface)` against the
frozen V64 oracle binding. It does not feed target identity into birth, ranking,
or readout. The fixed `13 x 100` result is:

```text
tracked oracle-binding identity anchor before frontier    {0: 4}
tracked oracle-binding identity anchor after frontier     {0: 4}
tracked oracle-binding identity anchor after exact        {0: 4}
coarse same-paradigm recovered candidate                  {1: 4}
H / B / S0                                      1280/1276/1276
H -> B / B -> S0                                         4/0
base projection failures                                   0
raw top-1 / base top-1                                277/267
20-worker maximum class p99                          11.678 ms
runtime authority changed                                false
```

Receipt:

```text
/home/e/projects/lay-productive-v1-build-20260811/receipts/v66/anchor-key-trace-full-workers20-13x100-receipt.json
```

The root diagnosis is therefore earlier than the mixed frontier: the recovery
field creates candidates for the correct paradigm but does not create the exact
source-anchor used by the frozen compatible binding. A lane split cannot retain
an anchor that was never born. This still left open whether another identity
anchor for the same paradigm could satisfy all exposed equations; the next
trace below closes that distinction.

The next permitted work is a target-blind recovery-birth diagnosis: establish
whether another pre-frontier anchor for that paradigm can already reconstruct
all exposed forms, or whether the sidecar lacks the required reverse program
from every remaining principal part. No further frontier change or top-16 work
is authorized before that distinction is measured.

That diagnosis replayed every pre-frontier identity anchor belonging to the
frozen oracle paradigm against the target-blind exposed forms. It found no exact
anchor:

```text
learned recovery posting for oracle paradigm             {0: 4}
same-paradigm recovered candidate                         {1: 4}
oracle-binding identity anchor before frontier            {0: 4}
any exact identity anchor before frontier                 {0: 4}
H / B / S0                                      1280/1276/1276
H -> B / B -> S0                                         4/0
base projection failures                                   0
raw top-1 / base top-1                                277/267
20-worker maximum class p99                          12.054 ms
runtime authority changed                                false
```

Receipt:

```text
/home/e/projects/lay-productive-v1-build-20260811/receipts/v66/pre-frontier-exact-full-workers20-13x100-receipt.json
```

This resolves the remaining distinction. The frontier is not hiding an exact
identity candidate. The sidecar has no learned reverse posting for the oracle
paradigm, and the generic identity bridge produces no anchor that can satisfy
the exposed-form equations. The retained first-loss label is therefore
`ORACLE_EXACT_RECOVERY_ANCHOR_NOT_GENERATED`.

The next architecture must repair recovery birth, not widen a frontier. The
paper candidate is to factor reverse evidence into a shared target-blind anchor
hypothesis field keyed by `(POS, observed_slot, canonical_anchor_slot,
reverse_program)`, then combine recovered anchor surfaces with independently
eligible paradigms and admit only byte-exact exposed-form reconstructions. This
would remove the current requirement that one exact reverse edit program have
support from at least two train lemmas inside the same fine-grained paradigm.
It remains a hypothesis until support distribution, fan-out, package bytes, and
fixed-proof behavior are measured; no implementation or package rebuild is yet
authorized by this result.

This diagnosis does not authorize a lemma, word, suffix, target-slot, proof-ID,
or hand-weighted exception. The next permitted implementation must remain
target-blind and bounded. It must preserve independently observable structural
groups before exact replay, retain base V64 ordering, and reject itself if it
changes the immutable denominator, probe parity, package bytes, false
singleton count, or any previously passing per-class result.

There are also 20 proof cases outside H:

```text
NO_PARADIGM_RECONSTRUCTS_EXPOSED_FORMS                12
NO_TARGET_POS_PARADIGM_RECONSTRUCTS_EXPOSED_FORMS      8
```

Those cases are hypothesis-coverage failures, not the current bounded-frontier
defect, and are not to be hidden by the H -> B repair. Runtime authority
remained unchanged.

## 43. Post-Clone Profile And Dense Diagnostic Flags

The symbolized profile preserved the complete micro contract. Its maximum class
p99 was `11.548 ms` under profiling and is diagnostic only. Clone-free replay
reduced `paradigm_reconstructs_exposed_forms` self cost from `7.44%` to `3.32%`.
The new leading self-costs were:

```text
derive_cold_lemma_bindings_with_diagnostics            12.11%
String extension during exact packaged replay            7.28%
BTreeSet insertion in cold-binding diagnostics            5.57%
geometry ranking                                          4.95%
execute_packaged_program_into                             4.75%
allocator malloc / free                             3.54% / 3.47%
```

The remaining tree insertion is dominated by repeated diagnostic ID tracking.
The immutable micro traverses `1,546,928` recovery paths and records posting,
recovery-posting, and recovered-anchor paradigm IDs on each path, although the
package contains only `2,099` dense paradigm IDs. These sets are proof evidence,
but repeated logarithmic duplicate insertion is not.

The next authorized exact change uses one dense byte flag lane indexed by
checked package paradigm ID. Independent bits record posting, slot-compatible,
exact-reconstructing, recovery-posting, recovered-anchor,
recovery-exact-reconstructing, and direct-selected membership. Final proof
diagnostic `BTreeSet`s are materialized once in ascending ID order so external
diagnostics and first-loss intersections remain unchanged. Candidate birth,
recovery, exact replay, evidence order, scores, ranks, frontiers, package bytes,
denominators, and authority must remain identical.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-clone-free-exact-replay.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-clone-free-exact-replay-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-clone-free-exact-replay-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-clone-free-exact-replay-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-clone-free-exact-replay-symbolized-13x10.perf.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-clone-free-exact-replay-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-clone-free-exact-replay-symbolized-self-report.txt
```

Not tested by this profile: fixed proof, integrated transfer, queue-inclusive
service latency, daemon/IBus, or physical product behavior. Runtime authority
remained unchanged.

## 44. Dense Diagnostic Flags Local Gate

Cold-binding paradigm memberships now use one checked dense flag lane during
derivation. Final proof-facing sets are materialized in ascending package-ID
order after candidate and recovery work is complete.

```text
cargo check --lib                                      PASS
focused productive_v1 tests                           79/79 PASS
focused failures                                           0
runtime authority changed                              false
```

The focused gate includes independent membership bits, duplicate marking,
ascending final IDs, and fail-closed zero/out-of-range identities. It does not
measure normal stripped latency, fixed proof, integrated transfer,
queue-inclusive service latency, daemon/IBus, or physical product behavior.
The verdict remains `FAIL_measured_shadow_gates`; runtime authority remains
unchanged.

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-dense-diagnostic-flags-focused-tests.txt
```

## 45. Dense Diagnostic Flags Remote Micro And Profile

The normal stripped remote micro preserved every structural, ranking, parity,
safety, package, and authority value, but did not pass the latency gate:

```text
metric                                      clone-free replay   dense flags
H / B / S0                                      127/127/127     127/127/127
H -> B / B -> S0 losses                                 0/0             0/0
raw top-1 / base top-1                                31/31           31/31
BaseProjection comparisons / failures                260/0           260/0
probe parity failures                                     0               0
false singleton / integrity errors                      0/0             0/0
maximum class p99                                  11.494 ms       11.779 ms
closed proof wall time                              226.929 ms      248.076 ms
proof user CPU                                        15.55 s         15.41 s
external peak RSS                                 308,848 KiB     308,368 KiB
productive package                                17,309,944 B    17,309,944 B
anchor-recovery package                            3,514,208 B     3,514,208 B
runtime authority changed                               false           false
```

The maximum was one `LEMMA_HELDOUT::missing_letter` sample; its class median
was `1.913 ms`. The exact membership rewrite is retained because it removed
repeated tree insertion without changing any evidence surface, but the latency
verdict remains `FAIL_measured_shadow_gates`. Fixed proof, integration,
installation, and authority transfer remain forbidden.

The DWARF profile preserved the same contract. Profiling overhead is diagnostic
only. Its leading self-costs after dense flags were:

```text
String extension during exact replay                    7.56%
derive_cold_lemma_bindings                               7.12%
execute_packaged_program_into                            5.49%
geometry ranking                                         4.69%
OSA emit                                                 4.57%
geometry sorting                                         4.28%
paradigm_reconstructs_exposed_forms                      3.47%
identity_anchor_slot                                     3.12%
```

The previous diagnostic `BTreeSet` insertion cost disappeared. The next
authorized change removes only target-blind exact-replay `String`
materialization. Packaged operations stream into a byte-exact matcher over the
already ordered exposed surfaces while the common executor retains the full
structural validation. Final-candidate materialization in `traverse_binding`
is unchanged. Program execution counts, matched `(slot, surface)` results,
candidates, scores, ranks, frontiers, package bytes, denominators, and authority
must remain identical.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-dwarf-dense-diagnostic-flags.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-dwarf-dense-diagnostic-flags.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-diagnostic-flags-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-diagnostic-flags-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-diagnostic-flags-dwarf-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-diagnostic-flags-dwarf-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-diagnostic-flags-dwarf-13x10.perf.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-dense-diagnostic-flags-dwarf-self-report-no-inline.txt
```

Not tested: fixed `13 x 100 x 2`, integrated transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 36. Post-Slot Profile And Dense Eligibility

The post-slot symbolized profile proved that repeated slot-profile decoding no
longer appears in the self-cost ranking. The remaining cold-binding cost is:

```text
derive_cold_lemma_bindings_with_diagnostics            14.46%
paradigm_reconstructs_exposed_forms                     7.21%
String extension during exact replay                    6.15%
BTreeMap insertion in cold binding                      5.27%
execute_packaged_program_into                           4.51%
geometry ranking                                        4.38%
allocator malloc / free                            3.13% / 3.46%
```

The next first shared mechanism is structural-eligibility ownership. Paradigm
identities are dense immutable package IDs, but each readout materialized
eligible paradigms in a `BTreeMap` and then performed logarithmic membership
queries for every recovery path. The micro observed `1,276,120` recovery
intersections; tree ownership is not evidence and does not affect order.

The authorized exact change keeps eligible paradigms in package-ID order in a
vector and uses a dense per-readout membership lane indexed by paradigm ID.
Iteration order remains ascending package order, lookup becomes constant-time,
and the final ordered maps and evidence sort remain unchanged. No package,
candidate, score, frontier, coefficient, denominator, or authority change is
authorized.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-slot-search-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-slot-search-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-prepared-slot-search-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-prepared-slot-search-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-prepared-slot-search-symbolized-self-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-prepared-slot-search.time.txt
```

Not tested by this profile: fixed proof, integration, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 37. Dense Eligibility Local Structural Gate

Structural eligibility now retains eligible paradigms in ascending immutable
package-ID order and records membership in a dense one-byte lane indexed by
the same ID. The recovery intersection no longer performs logarithmic
`BTreeMap` membership queries. Direct and recovery paths resolve an admitted
paradigm from the existing checked prepared record array; final selected and
recovered maps, evidence order, candidate bounds, scores, ranks, and authority
remain unchanged.

Local verification completed:

```text
cargo check --lib                                      PASS
focused productive_v1 tests                           77/77 PASS
focused failures                                           0
runtime authority changed                              false
```

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-dense-eligibility-focused-tests.txt
```

This local gate does not measure normal stripped latency, fixed `13 x 100 x 2`
quality, integrated L1.1/L3/L4/verifier transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. The verdict remains
`FAIL_measured_shadow_gates` and runtime authority remains unchanged until the
same immutable remote micro satisfies maximum class `p99 <= 5 ms`.

## 38. Dense Eligibility First Remote Micro

The first normal stripped remote micro retained every structural, ranking,
parity, safety, package, and authority value, but did not pass the latency
gate:

```text
metric                              prepared slot search   dense eligibility
H / B / S0                              127/127/127          127/127/127
H -> B / B -> S0 losses                         0/0                  0/0
raw top-1 / base top-1                        31/31                31/31
BaseProjection comparisons / failures        260/0                260/0
probe parity failures                             0                    0
false singleton / integrity errors              0/0                  0/0
maximum class p99                           7.230 ms            16.449 ms
proof user CPU                                15.92 s              15.84 s
external peak RSS                        309,168 KiB         309,168 KiB
constant prepared cache                 14,423,032 B        14,423,032 B
productive package                       17,309,944 B        17,309,944 B
runtime authority changed                       false                false
```

The maximum was one `LEMMA_HELDOUT::suffix_truncation` sample. Median latency
for that class was `1.712 ms`; nine other heldout classes also exceeded `5 ms`
at their single-sample p99. At measurement time an unrelated orphaned
`nando-transition-serving` process consumed approximately `73%` of one CPU,
while the managed serving process remained active separately. This is recorded
as possible scheduler interference, not as an excuse to pass the result.

The verdict remains `FAIL_measured_shadow_gates`. Fixed proof, integration,
installation, and authority transfer remain forbidden. One identical repeat is
permitted to determine whether the regression is reproducible; both results
must remain in the evidence record. A repeat cannot erase this failed sample.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-dense-eligibility.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-dense-eligibility.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-eligibility-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-eligibility-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-eligibility-13x10.time.txt
```

Not tested: fixed `13 x 100 x 2`, integrated transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 39. Dense Eligibility Repeat And Verdict

The one authorized identical repeat reproduced the latency failure while
preserving the complete non-latency contract:

```text
metric                                  first run       repeat
H / B / S0                            127/127/127  127/127/127
H -> B / B -> S0 losses                       0/0          0/0
raw top-1 / base top-1                      31/31        31/31
BaseProjection comparisons / failures      260/0        260/0
probe parity failures                           0            0
false singleton / integrity errors            0/0          0/0
maximum class p99                         16.449 ms    11.596 ms
proof user CPU                              15.84 s      15.73 s
external peak RSS                      309,168 KiB  308,848 KiB
runtime authority changed                     false        false
```

The repeat maximum moved to
`LEMMA_HELDOUT::non_adjacent_transposition`; this confirms a broad heldout-tail
problem rather than one damage-class mechanism. Dense membership is therefore
retained only as an exact structural optimization, not as a latency success.
The next permitted action is a symbolized profile of this exact implementation.
Fixed proof and promotion remain forbidden.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-eligibility-repeat1-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-eligibility-repeat1-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-dense-eligibility-repeat1-13x10.time.txt
```

Not tested: fixed `13 x 100 x 2`, integrated transfer, queue-inclusive service
latency, daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.

## 40. Post-Dense Profile And Clone-Free Exact Replay

The symbolized profile preserved `127/127/127` H/B/S0, `31/31` raw/base top-1,
exact `260/260` BaseProjection and probe parity, zero false singleton, zero
integrity errors, unchanged package bytes, and unchanged runtime authority.
Profiling overhead produced maximum class p99 `11.808 ms`; it is diagnostic and
does not replace normal stripped measurements.

The leading self-costs were:

```text
derive_cold_lemma_bindings_with_diagnostics            11.95%
paradigm_reconstructs_exposed_forms                      7.44%
BTreeSet insertion, principally exact-match dedup         5.65%
execute_packaged_program_into                            5.59%
geometry and recovery sorting                            5.36%
```

Exact exposed-form replay currently clones every matched generated surface into
a fresh `BTreeSet<(slot, surface)>`, even though exposed constraints are already
immutable and unique. This work proves duplicate suppression only; it does not
contribute evidence, scores, ranking, or authority.

The next authorized exact change stores exposed constraints as ordered slot and
surface vectors with stable dense match offsets. Each replay uses a dense
matched lane, sets the corresponding bit after byte-exact equality, and counts
the first match only. It must still execute every previously eligible packaged
program so `exact_replay_program_execution_count` remains identical. Any change
to candidates, diagnostics, scores, ranks, frontiers, parity, package bytes, or
authority rejects the optimization.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-symbolized-dense-eligibility.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-eligibility-symbolized-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-eligibility-symbolized-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-eligibility-symbolized-13x10.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/profile-dense-eligibility-symbolized-13x10.perf.log
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-dense-eligibility-symbolized-report.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/perf-dense-eligibility-symbolized-self-report.txt
```

Not tested by this profile: fixed proof, integrated transfer, queue-inclusive
service latency, daemon/IBus, or physical product behavior. Runtime authority
remained unchanged.

## 41. Clone-Free Exact Replay Local Gate

The implementation replaced per-replay matched `BTreeSet` ownership and
generated-surface clones with ordered immutable constraints and one dense match
lane. Duplicate packaged programs still set one match, every previously
eligible program still executes, and the match result remains exact over every
unique `(slot, surface)` constraint.

```text
cargo check --lib                                      PASS
focused productive_v1 tests                           78/78 PASS
focused failures                                           0
runtime authority changed                              false
```

Receipt:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/local-clone-free-exact-replay-focused-tests.txt
```

This local gate does not measure normal stripped latency, fixed proof,
integrated transfer, queue-inclusive service latency, daemon/IBus, or physical
product behavior. The verdict remains `FAIL_measured_shadow_gates`; runtime
authority remains unchanged.

## 42. Clone-Free Exact Replay Remote Micro

The normal stripped micro preserved the complete contract and reduced closed
proof CPU, but did not pass the maximum-class latency gate:

```text
metric                              dense repeat   clone-free replay
H / B / S0                          127/127/127          127/127/127
H -> B / B -> S0 losses                     0/0                  0/0
raw top-1 / base top-1                    31/31                31/31
BaseProjection comparisons / failures    260/0                260/0
probe parity failures                         0                    0
false singleton / integrity errors          0/0                  0/0
maximum class p99                       11.596 ms            11.494 ms
closed proof wall time                  253.696 ms           226.929 ms
proof user CPU                            15.73 s              15.55 s
external peak RSS                    308,848 KiB         308,848 KiB
runtime authority changed                   false                false
```

Removing matched-surface clones and tree ownership improved closed proof wall
time by `10.55%` and user CPU by `1.14%`. The class maximum remained unstable
and moved to `LEMMA_HELDOUT::extra_letter`; several heldout classes still
exceeded `5 ms`. The optimization is retained as exact redundant-work removal,
but its latency verdict is FAIL. Fixed proof, integration, installation, and
authority transfer remain forbidden. The next permitted action is a symbolized
profile of this exact implementation.

Receipts:

```text
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/release-build-clone-free-exact-replay.time.txt
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-clone-free-exact-replay-13x10-receipt.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-clone-free-exact-replay-13x10.stdout.json
/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V66_BOUNDED_RECOVERY_PAPER_2026-08-11/micro-clone-free-exact-replay-13x10.time.txt
```

Not tested: fixed proof, integrated transfer, queue-inclusive service latency,
daemon/IBus, or physical product behavior. Runtime authority remained
unchanged.
