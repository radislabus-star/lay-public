# NANDA Triad Worksheet

task_id: td007-semantic-reconciliation
domain: code
query: Reconcile 116 stale semantic tests without weakening candidate retention, DecisionCore, verifier, or quality gates

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | L1.1 exact field | births | grounded candidate lattice | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:7 | 1.0 | producer | lattice | lexical | lexical-l11 | L1.1 | L1.1 | bounded lattice | candidates | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:7 | production |
| t2 | Productive V90 | retains | grounded candidate lattice | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:52 | 1.0 | producer | lattice | lexical | lexical-productive | L2 | Productive V90 | canonical field | retained candidates | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:52 | production |
| t3 | L3 and L4 | add_rank_evidence_to | retained candidate lattice | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:88 | 1.0 | context | lattice | lexical | lexical-context | L3-L4 | L3 and L4 | retained lattice | rank evidence | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:88 | production |
| t4 | TransitionDecisionCore | settles | one correction outcome | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:66 | 1.0 | decision_owner | outcome | decision | decision-core | DecisionCore | TransitionDecisionCore | candidate lattice | typed outcome | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:66 | production |
| t5 | verifier | authorizes | physical edit plan | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:67 | 1.0 | verifier | edit_plan | mutation | edit-verifier | verifier | verifier | typed outcome | verified edit | docs/ime-target-authority-slice8-lexical-readout-2026-08-23.md:67 | production |
| t6 | missing lexical field | blocks | automatic lexical authority | docs/text-correction-gate-architecture.md:1 | 1.0 | unavailable_input | authority | fail-closed | unavailable-field | L2 | canonical field | unavailable field | suggestion or abstention | docs/text-correction-gate-architecture.md:1 | production |
| t7 | TD-007 ledger | classifies | every baseline failure | tech_debt/007-reconcile-semantic-contract-tests.md:22 | 1.0 | evidence_owner | failure | reconciliation | failure-ledger | proof | TD-007 | 116-row baseline | one disposition | tech_debt/007-reconcile-semantic-contract-tests.md:22 | tests |
| t8 | fixed heldout proof | constrains | runtime semantic repair | tech_debt/evidence/td007-heldout-baseline-v1.md:1 | 1.0 | proof | runtime_change | quality | heldout-quality | proof | Gate C V2 | runtime diff | conjunctive quality | tech_debt/evidence/td007-heldout-baseline-v1.md:1 | tests |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | L1.1 exact field | births | grounded candidate lattice | proposed TD-007 route | 1.0 | producer | lattice | lexical | lexical-l11 | L1.1 | L1.1 | bounded lattice | candidates | tech_debt/evidence/td007-structural-route-v1.md | production |
| c2 | Productive V90 | retains | grounded candidate lattice | proposed TD-007 route | 1.0 | producer | lattice | lexical | lexical-productive | L2 | Productive V90 | canonical field | retained candidates | tech_debt/evidence/td007-structural-route-v1.md | production |
| c3 | L3 and L4 | add_rank_evidence_to | retained candidate lattice | proposed TD-007 route | 1.0 | context | lattice | lexical | lexical-context | L3-L4 | L3 and L4 | retained lattice | rank evidence | tech_debt/evidence/td007-structural-route-v1.md | production |
| c4 | TransitionDecisionCore | settles | one correction outcome | proposed TD-007 route | 1.0 | decision_owner | outcome | decision | decision-core | DecisionCore | TransitionDecisionCore | candidate lattice | typed outcome | tech_debt/evidence/td007-structural-route-v1.md | production |
| c5 | verifier | authorizes | physical edit plan | proposed TD-007 route | 1.0 | verifier | edit_plan | mutation | edit-verifier | verifier | verifier | typed outcome | verified edit | tech_debt/evidence/td007-structural-route-v1.md | production |
| c6 | missing lexical field | blocks | automatic lexical authority | proposed TD-007 route | 1.0 | unavailable_input | authority | fail-closed | unavailable-field | L2 | canonical field | unavailable field | suggestion or abstention | tech_debt/evidence/td007-structural-route-v1.md | production |
| c7 | TD-007 ledger | classifies | every baseline failure | proposed TD-007 route | 1.0 | evidence_owner | failure | reconciliation | failure-ledger | proof | TD-007 | 116-row baseline | one disposition | tech_debt/evidence/td007-structural-route-v1.md | tests |
| c8 | fixed heldout proof | constrains | runtime semantic repair | proposed TD-007 route | 1.0 | proof | runtime_change | quality | heldout-quality | proof | Gate C V2 | runtime diff | conjunctive quality | tech_debt/evidence/td007-structural-route-v1.md | tests |

## notes

- Candidate retention and automatic apply authority are separate.
- Current runtime output is observation, not authority to rewrite assertions.
- A missing field must remain fail-closed.
- Historical expectations need a ledger row and a current replacement proof.
