# TD-007 Milestone 2 Completion V1

Status: `MILESTONE_2_PASS`

Base commit:

```text
447e1d24d49110637ff7f1e671098ef3736c2df6
```

## Scope

Milestone 2 closes all 28 baseline `ime_authority` failures. The current
contract distinguishes deterministic projection from automatic edit authority:
frameless lexical candidates abstain, while an exact layout replacement may
apply only through a closed contour certificate.

```text
NONDETERMINISTIC_TARGET_ASSERTION_REPLACED_BY_FRAME_CONSISTENCY  1
SUPERSEDED_FRAMELESS_AUTOMATIC_AUTHORITY_REPLACED               13
PROJECTION_PRESERVED_AUTHORITY_WITHHELD                           4
SUPERSEDED_PRODUCER_ID_REPLACED                                   3
CLOSED_EXACT_AUTHORITY_REPLACEMENT                                6
SAFETY_REGRESSION_REPAIRED                                        1
total                                                             28
```

The runtime repair is a structural boundary guard for an unbacked short
derivational-prefix fragment. It prevents the false split fallback exposed by
`перхвачу` and preserves the verified `да норм`, `то есть`, and
`Елена просит` boundary routes. SafetyGate, edit-plan validation, DecisionCore,
and verifier authority were not weakened.

## Exact Partition

```text
baseline failures before milestone 2   81
milestone 2 closed                      28
remaining exact failures                53
partition overlap                        0
```

The 28 ledger rows and 53 exact-known rows are disjoint. Their union equals the
milestone-1 remainder. The remaining split is 33 correctness and 20 package
failures.

## Evidence

```text
ledger
tech_debt/evidence/td007-milestone2-ledger-v1.json
0637cde490c16000abd85f74adc2edaa5f532000f5a0a3316795283b0984d86c

canonical failure observation
tech_debt/evidence/td007-milestone2-known-failures-v1.json
8f11cdc9543f3c035c954d187df56867334666f6becba51a2ce0b7521237128f

independent contract verification
tech_debt/evidence/td007-milestone2-verified-v3.json
94780ddd330a7f69f302f39530e4be9d92da503c59246519dbbc26f760c81014

manifest
scripts/test-lanes/manifest.json
ff16fd9031dc23fe48fedbe4782e42530cbfde7d29da5929fa2fe5f756eee967

known-failure contract
scripts/test-lanes/known_failures.json
41a7ca18b9d5c6a9f58796f682cadf6b2f23780235f020ea233b711b08e71a28
```

The verification receipt binds source closure
`aa6ecabde1a2c17e29ffdcc15abe2e9ab3ce4f3ce616fff0227da9147a7ce09b`
and the final known-failure contract. It reports:

```text
correctness selected                    2309
correctness exact-known failures          33
package selected                          35
package exact-known failures              20
infrastructure failures                    0
verdict          PASS_WITH_EXACT_KNOWN_FAILURES
```

## Review

The independent reviewer used the two allowed passes. Pass 1 scored `5/10`
and found three material proof weaknesses; pass 2 scored `7/10` and confirmed
the runtime, IME contract, ledger, and receipt repairs. Its one remaining P2
atomic-target weakness was repaired afterward and covered by the focused proof
plus full verification V3. No third pass was used.

Review record:

```text
tech_debt/evidence/td007-milestone2-independent-review-v2.md
a6ea84ec3ff7d10d810e472a856624c0c0d6e28bf60c71e3204c281216a92152
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

The frozen 13 x 20,000 heldout route remains mandatory after milestones 3 and
4. Milestone 2 does not claim full TD-007 completion or quality promotion.
