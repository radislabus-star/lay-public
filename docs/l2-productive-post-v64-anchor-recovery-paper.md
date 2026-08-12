# Post-V64 Canonical-Anchor Recovery Paper

Status: `V65_MEASURED_AND_REJECTED_FOR_INTEGRATION`, runtime authority unchanged.

This document owns the next systemic morphology experiment after V64. It does
not authorize installation, daemon/IBus restart, live ownership, manual weights,
literal-word rules, a wider candidate frontier, or a raw-corpus pass.

## 1. Corrected V64 Evidence

The V64 package is unchanged:

```text
path    /home/e/projects/lay-productive-v1-build-20260811/out/LAY-L2-PRODUCTIVE-PARADIGM-V1-SHADOW-V64.p2m
bytes   17,309,944
sha256  9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
```

The proof identity was corrected from `(lemma, paradigm)` to
`(lemma, POS, paradigm)`. The previous proof allowed an unrelated POS basin of
the same lemma to satisfy `H` or `B`. With exact target-POS ownership, the fixed
`13 classes x 100 x 2 cohorts` proof is:

```text
LEMMA_HELDOUT   1,300
H               1,280
B               1,219
S0              1,219
S1              1,219
S2              1,219
S3              1,219
R               1,219
top-16          1,218
raw top-1         267
```

The corrected first-loss decomposition is:

```text
outside target-POS hypothesis H                         20 cases / 6 lemmas
H retained, target-blind B absent                       61 cases / 18 lemmas
  oracle paradigm absent from remaining slot postings   59 cases
  oracle paradigm fails exposed-form reconstruction      2 cases
B retained, target slot absent at S0                     0 cases
S0 -> S1 -> S2 -> S3 -> R loss                           0 cases
```

All `2,600` probed and unprobed results were structurally identical. False
singleton and integrity errors were zero. Runtime authority did not change.

## 2. First Shared Mechanism

Transition induction currently chooses one canonical anchor per lemma/POS:

```text
forms in one lemma/POS
-> select_canonical_anchor(forms)
-> derive anchor -> every target transition
-> paradigm postings contain only transition source slots
```

For the `59` measured losses, the hidden proof target is that canonical anchor.
After target masking, many other exact forms remain, but the compatible
paradigm has no program whose source slot is one of those remaining forms.
Consequently it is absent from all target-blind postings before exact
compatibility can run.

This is not a score, frontier, phase, or authority defect. It violates the paper
claim that an incomplete lemma can be grounded from every exposed principal
part supported by the learned paradigm.

## 3. Required Operator

Let `a` be the canonical anchor, `s` an observed non-anchor principal part, and
`t` a requested target slot. V64 stores a forward program:

```text
F[a -> x] for learned target slots x
```

The new logical operator is sparse two-hop anchor recovery:

```text
observed s
-> learned reverse recovery R[s -> a]
-> recovered canonical surface a_hat
-> existing forward program F[a -> t]
-> candidate target surface
```

`R` is not created by naively reversing opcodes. It is induced from the frozen
classified transition evidence, where both source and target surfaces are
available, and admitted only when its exact execution is independently
verified. Non-injective recovery retains every compatible recovered anchor.

## 4. Bounded Representation

The accepted micro representation is a separate read-only recovery lane:

```text
AnchorRecoveryIndexV1
  key = (POS, exposed_source_slot)
  value = bounded postings of (paradigm_id, recovery_program_id)

AnchorRecoveryProgramV1
  source_slot_id
  canonical_anchor_slot_id
  typed edit program
  train lemma support
  stability/provenance digest
```

The existing forward trie, calibrated coefficients, `16 / 32` physical
candidate bounds, surface-basin equivalence, and verifier remain unchanged.
Recovery happens before `ColdLemmaBindingV1` admission and only creates an
additional legal source path to the same paradigm identity.

The compiler reuses the frozen reduced/classified transition artifacts. It may
replay transition induction and package reduce, but it must not rescan the
`434,934,248 B` raw corpus.

## 5. Exact Admission

For observations `O` with hidden target `(t, y_t)`, a recovered binding is
admitted only if:

```text
POS(source) == POS(paradigm) == POS(target)
R[s -> a](surface_s) = a_hat
for every exposed (slot_i, surface_i) in O:
    F[a -> slot_i](a_hat) contains surface_i exactly
```

Target bytes and target annotations do not participate. Multiple recovered
anchors or paradigms remain a tied binding lattice. Zero compatible recovery
paths yields `ABSTAIN`. No recovery evidence may directly grant Winner.

## 6. Complexity Budget

Rejected representation:

```text
all source forms x all target forms = quadratic program expansion
```

Accepted representation:

```text
one recovery edge per admitted non-anchor principal part
+ existing canonical forward programs
= linear sparse two-hop closure
```

Measured V64 package counts are `77,854` programs, `270,449` operations,
`72,104` terminals, and `17,309,944 B`. Before implementation, recovery size is
an estimate, not a fact:

```text
micro package ceiling          35 MiB
hard project package ceiling  195 MiB
compile/resume ceiling         10 minutes on the 20-thread remote host
proof peak RSS ceiling         350 MiB
```

Crossing a ceiling rejects the representation; it is not repaired by widening
the budget after the run.

## 7. Causal Gates

The first proof is `13 x 10 x 2`, 19 workers, shadow-only. It must report:

```text
recovery-applicable H->B cases
recovery path born
recovered anchor exact against all exposed forms
oracle binding retained
S0/S1/S2/S3/R retention
probed/unprobed parity
false singleton and integrity errors
package bytes, RSS, elapsed, per-class latency
```

Protected invariants:

```text
all previous 1,219 B/S0/R cases retained
B == S0
S0 == S1 == S2 == S3 == R
probe parity exact
false singleton 0
integrity errors 0
SEEN_EXACT no aggregate or per-class regression
runtime authority unchanged
```

The mechanism gate is conditional rather than manually tuned:

```text
for every case where an exact evidence-supported R[s -> a] exists,
the oracle paradigm must enter B;
cases without such an operator remain outside coverage or ABSTAIN.
```

Only a passing micro permits `13 x 100 x 2`. The normative `20,000` per class,
L1.1/L2/L3 integration, unsupported/multi-label cohorts, daemon/IBus, queue
latency, and live owner remain later independent gates.

## 8. Expected Bound, Not A Claim

If all `59` posting-only losses are recoverable and the `2` reconstruction
losses are unchanged, the maximum measured movement is:

```text
B/R  1,219 -> 1,278 of 1,300 = 98.3077%
H                       1,280 = 98.4615%
```

This is an upper-bound hypothesis, not a promised result. Per-class strict
`>95%` still must be measured separately. The `20` cases outside target-POS
`H` cannot be repaired by anchor recovery; they require a separate hypothesis
coverage paper or remain `ABSTAIN`.

## 9. Critical Review

1. A reverse edit is not generally a mathematical inverse. Exact train-side
   reconstruction and complete exposed-form replay are mandatory.
2. A recovered anchor may be non-identifiable. The lane must retain all valid
   anchors and cannot force a singleton.
3. A second full anchor-to-all program bank would duplicate the trie and mix
   source identity with target identity. The sparse recovery lane avoids that.
4. Broadening compatibility postings without an executable recovery program is
   unsound: it would make `B` true while generation still has no legal source.
5. Runtime caching can reduce repeated latency only after semantic parity; it
   cannot create missing bindings and is not part of the quality mechanism.
6. The two exact-reconstruction losses and twenty outside-H cases are protected
   negative controls. This experiment is rejected if it appears to repair them
   by target leakage or weakened exactness.

Pre-implementation verdict: the sparse reverse-anchor recovery operator was
internally consistent, bounded, target-blind, and addressed the measured
`59/61` first mechanism on paper. That verdict authorized only the V65 micro;
the measured rejection is recorded in section 10. Promotion and installation
remain forbidden.

Exact owning receipt:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V64_SURFACE_BASIN_2026-08-11/hbs0-pos-diagnostic-13x100-receipt.json`.

## 10. V65 Measured Result

V65 implemented the paper operator as an optional mmap `.p2r` sidecar. The
frozen V64 `.p2m` remained byte-identical:

```text
V64/V65 base .p2m bytes     17,309,944
V64/V65 base .p2m sha256    9fd8c950398fb8ba47a2c9f2236880239d9f4376b191a691b0d01c47ddd3e438
V65 recovery .p2r bytes      3,514,208
V65 recovery .p2r sha256     744a11a8c88921bc3b63899840982ecadc3260485808dd60ea4858bcd54a8436
V65 total bytes             20,824,152
recovery paths                  36,915
recovery operations            136,074
```

The reduce requires at least two independent train lemmas, skips classified
lemma/POS basins that have no paradigm assignment, and caches exact reverse
program replay by `(paradigm, exposed slot, anchor slot)`. The full resume reused
the frozen raw-corpus, sorted-context, and transition-induction artifacts:

```text
total resume             104.416 s
anchor recovery reduce    39.702 s
peak compile RSS         620,900 KiB
runtime authority changed false
```

The `13 x 10 x 2` micro passed the conditional mechanism checks on its sample:

```text
LEMMA_HELDOUT 130 -> H 127 -> B/S0/S1/S2/S3/R 127
H -> B          0
B -> S0         0
probe parity  260 / 260
false singleton 0
integrity errors 0
peak RSS       222,844 KiB
```

That conditional result authorized the fixed `13 x 100 x 2` measurement. The
full result is:

```text
                         V64      V65
LEMMA_HELDOUT cases      1,300    1,300
H                        1,280    1,281
B/S0/S1/S2/S3/R          1,219    1,277
top-16                   1,218    1,276
raw top-1                  267      205
outside H                   20       19
H -> B                      61        4
B -> S0                      0        0
SEEN_EXACT R              1,297    1,297
SEEN_EXACT top-16         1,297    1,297
SEEN_EXACT top-1            455      455
probe parity              2,600    2,600
false singleton               0        0
integrity errors               0        0
```

The operator materially improves target-blind binding and readout retention,
but V65 is rejected for integration for four independent measured reasons:

1. `H -> B` is reduced from `61` to `4`, not closed to zero. The four failures
   are one lemma surface, `впросинь`, observed in four damage classes; its oracle
   paradigm is still absent from target-blind recovery postings.
2. The oracle `H` denominator changed from `1,280` to `1,281`. The sidecar is
   participating in the oracle path, so the frozen V64 hypothesis denominator
   was not preserved. A later proof must compute base `H` independently from
   target-blind recovery `B` before making a causal closure claim.
3. LEMMA_HELDOUT raw top-1 regressed from `267` to `205`. Recovery never grants
   Winner authority, but its added basins still change candidate ordering. The
   conjunctive quality contract rejects this regression.
4. Runtime cost is outside the product budget. Across LEMMA_HELDOUT classes,
   p50 is `19.009-28.269 ms`, maximum p99 is `255.827 ms`, and maximum case
   latency is `483.606 ms`. The full proof took `1,951.507 s`; V64 took
   `76.812 s`. The current `(POS, source slot)` recovery index expands too many
   paths and repeatedly replays complete paradigms.

Per-class readout retention is now `96-100%`, but suffix-truncation top-16 is
exactly `95.0%` and therefore fails the strict `>95%` gate. Overall top-1 also
remains far below the promotion contract. `SLOT_HELDOUT`, `MULTI_LABEL`,
`UNSUPPORTED`, integrated L1.1/L2/L3/L4/verifier, queue-inclusive latency, and
the physical product matrix remain untested.

Verdict scope:

```text
reverse-anchor representation    PROVED_BOUNDED
conditional H -> B mechanism     IMPROVED_NOT_CLOSED
fixed quality proof              FAIL
shadow integration               FORBIDDEN
daemon/IBus installation         NOT_PERFORMED
live ownership                   FORBIDDEN
V64 canonical status             PRESERVED
```

Exact V65 receipts:

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V65_ANCHOR_RECOVERY_2026-08-11/build-receipt.json`

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V65_ANCHOR_RECOVERY_2026-08-11/micro-13x10-receipt.json`

`/home/ubu/projects/lay/docs/structural_gates/receipts/L2_PRODUCTIVE_V65_ANCHOR_RECOVERY_2026-08-11/full-13x100-receipt.json`
