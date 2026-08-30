# TD-007 Milestone 3 Completion V1

Status: `MILESTONE_3_PASS`

Base commit:

```text
b093a6df2328687ee8f569a5c21db47635b95282
```

## Scope

Milestone 3 closes all eleven historical Nanda L2/L3 contract failures. The
replacement proofs bind candidate origin to the active downstream effect:
retention remains distinct from mutation authority, known surfaces remain
protected, and exact-layout projection does not silently add a second typo
mutation.

Two package-dependent proofs are now hermetic. Sparse omission reaches the
unified candidate gate and retains its error class while authority is demoted.
Participle ambiguity is derived from the actual fuzzy frontier and every
reference-backed member remains `SuggestOnly`.

No runtime authority, candidate birth, ranking, or admission rule changed.

## Exact Partition

```text
baseline failures before milestone 3   53
milestone 3 closed                      11
remaining exact failures                42
partition overlap                        0
```

The remaining split is 22 correctness and 20 package failures.

## Evidence

```text
consequence analysis
tech_debt/evidence/td007-milestone3-consequence-analysis-v1.md
e237d8f350d2817689207eee9a0c804d290c75b2039d9b61ba46734d897a2bd5

ledger
tech_debt/evidence/td007-milestone3-ledger-v1.json
a1c4c01b2eb2e3d04a0ac77114417428161acab7b5c9120348a3131c0e8d3edd

verification
tech_debt/evidence/td007-milestone3-verified-v1.json
5c8fcb3e375fa81ecbc9ea2832687f7388bb52767f63e0250b16bf18e67a88bd

known-failure contract
scripts/test-lanes/known_failures.json
01fe9e3a4021132c52e6cdd895673b73c52670b52a4542d20c46acf141a02617
```

The final verification binds source closure
`f2ac1cb4dc6c2cf7415a4f7fa316ab75ded27d4d04c6e3d838de151d106ddfb7`
and reports:

```text
correctness selected                    2309
correctness exact-known failures          22
package selected                          35
package exact-known failures              20
unexpected failures                        0
infrastructure failures                    0
verdict          PASS_WITH_EXACT_KNOWN_FAILURES
```

## Review

The two allowed review passes were used. Pass 1 scored `5/10` and found five
material proof or provenance weaknesses. All were repaired. Pass 2 scored
`7/10` and found one remaining downstream-effect gap; it was repaired by
requiring the exact-layout sequence to reach the L3 `Suggest` result. No third
scored pass was used.

Review record:

```text
tech_debt/evidence/td007-milestone3-independent-review-v2.md
3afa0579c7a029bbce6d0b6e9fd1d8d34dc81683bb78c51bf1d29111b97e86fb
```

## Authority Boundary

```text
source runtime semantics changed   false
installed/live runtime changed     false
Cargo install                      false
service restart                    false
desktop mutation                   false
release admitted                   false
```

The fixed 13 x 20,000 heldout gate remains mandatory after milestone 4. This
milestone does not claim full TD-007 completion or quality promotion.
