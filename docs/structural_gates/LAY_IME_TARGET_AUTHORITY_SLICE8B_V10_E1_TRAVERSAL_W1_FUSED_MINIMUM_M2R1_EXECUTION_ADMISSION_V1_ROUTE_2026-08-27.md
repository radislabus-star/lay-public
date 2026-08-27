# NANDA Triad Worksheet

task_id: w1-fused-minimum-m2r1-execution-admission-v1
domain: general
query: Can independent live preflight admit the sealed M2R1 controller without creating M2R1 scientific state?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M2R1 implementation receipt | authorizes | live preflight producer | exact next action and source identities | 1.0 | authorization owner | observation owner | authority | predecessor |
| s2 | live preflight producer | observes_twice | host inputs namespaces processes and perf descriptors | fixed 15 second interval | 1.0 | observation owner | live target state | observation | live closure |
| s3 | foreign performance process | vetoes | M2R1 execution admission | no overlapping scored experiment contract | 1.0 | conflict owner | execution authority | veto | conflict exclusion |
| s4 | disposable UID e probe | proves | cache ancestor write and cleanup capability | actual fsync rename read unlink transaction | 1.0 | capability producer | UID boundary | capability | UID proof |
| s5 | sealed M2R1 bootstrap | owns | future exact path capability proof | root parent and pre marker audit | 1.0 | namespace owner | future path proof | bootstrap | path boundary |
| s6 | live preflight producer | publishes | unadmitted evidence tree | fixed admission symbol absent from producer | 1.0 | evidence producer | evidence artifact | observation | producer boundary |
| s7 | independent auditor | publishes_only_after | producer PASS and third clean snapshot | atomic exclusive 0444 publication | 1.0 | authorization owner | execution admission | audit | publication |
| s8 | failed or unknown predicate | leaves_absent | M2R1 admission | fail closed provenance dispatch | 1.0 | failure owner | execution authority | failure | failure closure |
| s9 | M2R1 admission | authorizes_once | sealed M2R1 controller run | exact task transaction and seven hashes | 1.0 | authorization owner | execution orchestrator | authority | execution boundary |
| s10 | admission transaction | preserves | runtime and scientific ledger | zero Cargo perf subject namespace markers | 1.0 | preservation owner | protected state | preservation | scientific boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M2R1 implementation receipt | authorizes | live preflight producer | exact next action and source identities | 1.0 | authorization owner | observation owner | authority | predecessor |
| c2 | live preflight producer | observes_twice | host inputs namespaces processes and perf descriptors | fixed 15 second interval | 1.0 | observation owner | live target state | observation | live closure |
| c3 | foreign performance process | vetoes | M2R1 execution admission | no overlapping scored experiment contract | 1.0 | conflict owner | execution authority | veto | conflict exclusion |
| c4 | disposable UID e probe | proves | cache ancestor write and cleanup capability | actual fsync rename read unlink transaction | 1.0 | capability producer | UID boundary | capability | UID proof |
| c5 | sealed M2R1 bootstrap | owns | future exact path capability proof | root parent and pre marker audit | 1.0 | namespace owner | future path proof | bootstrap | path boundary |
| c6 | live preflight producer | publishes | unadmitted evidence tree | fixed admission symbol absent from producer | 1.0 | evidence producer | evidence artifact | observation | producer boundary |
| c7 | independent auditor | publishes_only_after | producer PASS and third clean snapshot | atomic exclusive 0444 publication | 1.0 | authorization owner | execution admission | audit | publication |
| c8 | failed or unknown predicate | leaves_absent | M2R1 admission | fail closed provenance dispatch | 1.0 | failure owner | execution authority | failure | failure closure |
| c9 | M2R1 admission | authorizes_once | sealed M2R1 controller run | exact task transaction and seven hashes | 1.0 | authorization owner | execution orchestrator | authority | execution boundary |
| c10 | admission transaction | preserves | runtime and scientific ledger | zero Cargo perf subject namespace markers | 1.0 | preservation owner | protected state | preservation | scientific boundary |

## notes

- A dirty observation is repeatable only because no scientific state or admission is created.
- Exact future-path access remains a bootstrap obligation before marker creation.
- Structural PASS cannot publish admission; implementation and live audits remain mandatory.
