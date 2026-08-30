# TD-007 Milestone 4 Consequence Analysis V1

Status: `MILESTONE_4_PASS`

Base commit:

```text
86eef2f794130cefa00d8ff74b75341a1c1ceddf
```

## Scope

Milestone 4 reconciles the final 42 historical semantic failures across typing
assist, phrase boundaries, edit safety, CLI integration, and architecture
source contracts. It also repairs the shared mechanisms behind the observed
false mutations; no fixture surface or test identity may enter runtime logic.

## First-Loss Groups

The initial failure observation separates four real mechanism defects from
superseded or non-hermetic expectations:

1. a protected dotted ASCII token could be converted before the protected-token
   guard ran;
2. a punctuation-shaped physical layout key at the start of the last token was
   stripped before contextual projection;
3. the deterministic missing-letter route could rewrite an already known
   Russian surface;
4. an ambiguous initial consonant insertion could be treated as a unique typo
   repair.

The two independent review passes found four additional shared safety gaps:
pending auto-undo checked backend availability before physical-grab ownership;
a shape-only multi-token split could gain apply authority; backed noun
derivation admitted orthographically impossible `-гы/-кы/-хы/-жы/-чы/-шы/-щы`
surfaces; and a 99% boundary-retention shortcut substituted mass for positive
transition proof. Those routes are now fail-closed. Safe independent one-word
repairs remain admissible when a broader multi-token proposal is not.

The other rows name retired producer IDs, require automatic authority from a
candidate that is now `SuggestOnly`, assume single-surface feedback promotion,
or depend on an ambient package/configuration. Those proofs will be replaced by
current semantic postconditions, not updated snapshots.

## Alternatives

1. Restore every historical automatic correction. Rejected: it would reopen
   known-surface drift, unsafe multiword edits, and ambiguous short-layout
   conversion.
2. Change expected output strings only. Rejected: broad corpus tests would no
   longer prove candidate geometry, boundaries, or safety effects.
3. Selected: repair the four shared mechanisms; retain ambiguous candidates as
   suggestions or explicit/manual work where supported; rewrite stale tests to
   assert current owners, typed proof identities, exact projection, boundary
   preservation, and no false automatic authority.

## Consequences

- Candidate birth: protected technical ASCII is no longer born as an automatic
  layout replacement. Exact punctuation-key layout projection remains available
  for a contextual token.
- Candidate retention: known-surface and ambiguous repairs may remain visible or
  diagnosable, but do not gain automatic mutation authority.
- Boundary authority: `SplitPreviousGluedAndRepairTail` is shape-only and
  unverified; moved-prefix proposals require independent transition proof, not
  a retention percentage.
- Ranking: exact reference-backed missing-letter candidates may outrank
  generated-only forms; ties without adequate independent separation abstain.
- Edit safety: SafetyGate, transition verification, backend authorization, and
  verifier ownership are unchanged or made stricter.
- Output ownership: pending auto-undo preserves daemon physical-grab ownership
  before backend availability can redirect the route.
- Morphology: backed-form derivation shares one orthographic `ы` guard across
  both clean-reference and center-backed entrypoints.
- Latency: guards are bounded token checks. No package traversal, network I/O,
  thread, cache, or retained allocation is added.
- Package/RSS: package bytes, source inputs, cache identity, and generation
  invalidation are unchanged.
- Learning: a one-surface feedback event remains censored from general L3
  authority and raw text remains absent from the compiled packet.
- Concurrency: no owner, queue, worker, timer, or stale-result protocol changes.
- IME/Double Shift: manual Double Shift remains a physical key projection and is
  not routed through typing-assist heuristics. Existing protected post-completion
  behavior remains unchanged.
- Rollback: each mechanism change is isolated in the owning classifier/helper;
  replacement tests bind its positive and negative boundary.

## Proof Plan

1. focused mechanism tests for protected ASCII, punctuation-key projection,
   known-surface retention, initial-insertion ambiguity, and reference ranking;
2. all 42 replacement contracts;
3. full local correctness and package lanes with zero unignored failures;
4. the frozen 13 x 20,000 evidence remains the accepted immutable baseline for
   assertion-only changes; any candidate birth/ranking/admission change must be
   accompanied by a fresh scoped quality comparison before release admission.

## Measured Outcome

The final source closure passed both non-timing lanes with an empty
known-failure ledger:

```text
manifest                         2,383 tests
correctness                      2,323 / 2,323 PASS
package                              36 / 36 PASS
semantic failures                            0
infrastructure failures                      0
source closure SHA-256             7543609f4be5f321...
```

The first full-quality attempt used the frozen V8 package from the TD-007
baseline. It stopped before quality workers started because this local
eight-CPU host exceeded the legacy implicit-forward *latency* prerequisite in
two maximum observations:

```text
combined max                  5,630 us > 5,000 us
implicit max                  3,573 us > 2,500 us
quality matrix tested                         false
verdict                       REJECT_C_PREREQUISITE
```

That receipt is retained as a local timing limitation, not interpreted as a
quality result. After the final review repairs, a fresh release proof binary was
built from source closure `7543609f4be5f321...` and executed against the sealed
direct V9 exact-support representation. This is not a format-parity inference:
the bound binary evaluated all 260,000 damaged cases and all 852,582 clean
centers. V9 checksum validity and stored-support equality to the corpus rebuild
were both checked by the same receipt. A separate binding receipt seals exact
argv, executable SHA, source closure before/after, input hashes, timestamps,
exit status, and raw receipt SHA.

```text
verdict                                      PASS_C_QUALITY
damaged denominator                          260,000 / 260,000
clean denominator                            852,582 / 852,582
target retention                             100.0%
aggregate unique top-1                       98.9406%
minimum class unique top-1                   97.0531%
minimum class lattice coverage               99.2150%
clean preservation                           100.0%
authoritative false authority / singleton    0 / 0
grounded legacy candidate loss               0
```

The receipt also reports `4` false-authority observations inside a separately
labelled compatibility observer. That observer is diagnostic and is not the
authoritative exact-field denominator; it is retained rather than folded into
the authoritative zero. No release or deployment is admitted by this proof.

Evidence:

- `tech_debt/evidence/td007-milestone4-correctness-verified-v1.json`
- `tech_debt/evidence/td007-milestone4-package-verified-v1.json`
- `tech_debt/evidence/td007-milestone4-quality-v8-local-prerequisite-blocked-v1.json`
- `tech_debt/evidence/td007-milestone4-quality-13x20000-v9-v1.json`
- `tech_debt/evidence/td007-milestone4-correctness-post-review-v2.json`
- `tech_debt/evidence/td007-milestone4-package-post-review-v1.json`
- `tech_debt/evidence/td007-milestone4-quality-13x20000-final-v1.json`
- `tech_debt/evidence/td007-milestone4-quality-binding-final-v1.json`
- `tech_debt/evidence/td007-milestone4-independent-review-v2.md`
- `tech_debt/evidence/td007-milestone4-ledger-v1.json`

## Authority Boundary

This milestone changes source runtime semantics only for the listed systemic
guards and projection repair. It does not install binaries, restart services,
change live desktop state, or admit a release.
