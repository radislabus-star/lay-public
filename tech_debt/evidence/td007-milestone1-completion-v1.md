# TD-007 Milestone 1 Completion V1

Status: `MILESTONE_1_PASS`

Base commit:

```text
6f14574452d9ab504f23be21b11288179bc6f0f2
```

## Scope

Milestone 1 closes 35 baseline failures owned by `correction_core`, candidate
sources, the L3 layout verifier, and edit safety.

```text
CURRENT_CONTRACT_RUNTIME_REPAIR                  5
SUPERSEDED_EXPECTATION_REPLACED                 22
HIDDEN_INPUT_DEPENDENCY_REMOVED                  8
HISTORICAL_CASE_ARCHIVED_WITH_REPLACEMENT_PROOF  0
total                                            35
```

The mechanism-level runtime repairs are:

1. known English tokens cannot enter generic layout-plus-typo fallback;
2. an unknown Cyrillic missing-letter result cannot auto-apply;
3. layout authority cannot be obtained by changing only a neighbor token;
4. compact boundary repair requires exact lexical ownership;
5. generic verified-boundary metadata cannot bypass edit safety.

No runtime rule contains a fixture surface or test identity. DecisionCore,
mutation-plan validation, SafetyGate, and verifier authority were not weakened.
The 13 former compare-only FullWave expectations are active current invariants.

## Exact Partition

```text
TD-006 baseline failures   116
milestone 1 closed          35
remaining exact failures   81
overlap                      0
```

The 35 ledger rows and 81 exact-known rows are disjoint and their union equals
the TD-006 baseline. The remaining split is 61 correctness and 20 package
failures.

## Evidence

```text
ledger
tech_debt/evidence/td007-milestone1-ledger-v1.json
fb9da024791a5c0a793d3a528e6e2c880daa5d5cabdc6de6b94e45c7715dc606

known-failure observation
tech_debt/evidence/td007-milestone1-known-failures-v5.json
4a3b7dcfef4a48ef27e1435a7ac97fe26aea13b22cf7dd30c727f6da64ea4fc3

correctness verification
tech_debt/evidence/td007-milestone1-correctness-verified-v5.json
6e87aed56ac0c302a45bec266bb62786d13ba57dbc5039e35be0ea3bb7266ee6

package verification
tech_debt/evidence/td007-milestone1-package-verified-v5.json
41c84dcef53fbada6800ba5e193b008b025f965b4f3ae5145634657ef259171a

pinned package proof
tech_debt/evidence/td007-milestone1-package-proof-v12.json
9a1d2abec0cce557ae43dd93c87e166f5c341e1098a1326b83427a96c0d1a8a5

manifest
scripts/test-lanes/manifest.json
cd130f2db52a23133666eeb301ce8199bbe8a7095b7faa9c2d4f71df13095e94

known-failure contract
scripts/test-lanes/known_failures.json
bb0e1061a5b9d0333a2f1580a520b9b1a7b1339c112a759baac2a7f56cf5bbc1
```

All three executable receipts bind source closure
`6bf210117510ab2b2a5833cb4f60d22659105a4dfff9255835f6269dbeec72c9`.
The package controller waits for an actual structured L1.1 lattice response
containing the pinned seed; `health=ready` alone is not treated as readiness.

```text
correctness selected                   2,308
correctness exact-known failures          61
package selected                          35
package exact-known failures              20
infrastructure failures                    0
lane verdicts  PASS_WITH_EXACT_KNOWN_FAILURES
package proof  TD007_PINNED_PACKAGE_PROOF_PASS
```

## Review

The independent reviewer used the two allowed passes and scored the second
review-time state `5/10`. Its four material findings were repaired afterward:
the package proof is non-vacuous, archived observations contain comparator
verdicts and exact ledgers, authority scope is explicit, and every final receipt
binds one unchanged source closure. No third pass was used to inflate the score.

Review record:

```text
tech_debt/evidence/td007-milestone1-independent-review-v2.md
13eca39436144385d9287cf5a5b7e3726ce53cebc6d867dc52f9eb11574e25b9
```

## Authority Boundary

```text
source runtime semantics changed   true
installed/live runtime changed     false
Cargo install                      false
service restart                    false
desktop mutation                   false
release admitted                   false
```

The frozen 13 x 20,000 heldout quality route remains mandatory after all
TD-007 runtime mechanisms are complete and before release admission. Milestone
1 does not claim full TD-007 completion or quality promotion.
