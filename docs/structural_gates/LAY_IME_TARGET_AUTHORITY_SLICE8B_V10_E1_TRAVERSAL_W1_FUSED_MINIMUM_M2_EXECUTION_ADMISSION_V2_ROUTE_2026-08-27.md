# NANDA Triad Worksheet

task_id: w1-fused-minimum-m2-execution-admission-v2
domain: general
query: Can independent live preflight admit the sealed M2 V3 controller without creating M2 execution state?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M2 V3 predecessor authority | authorizes | independent live preflight producer | sealed next_action_admitted exact field | 1.0 | authorization owner | observation owner | authority | predecessor-chain |
| s2 | live preflight producer | observes_twice | host inputs namespaces processes and perf-event descriptors | fixed 15-second quiet window | 1.0 | observation owner | live target state | observation | live-closure |
| s3 | foreign performance process | vetoes | M2 execution admission | no overlapping scored experiment contract | 1.0 | conflict owner | execution authority | veto | conflict-exclusion |
| s4 | disposable UID e probe | proves | neutral ancestor traverse write fsync rename read unlink capability | actual operations and absent cleanup | 1.0 | capability producer | UID boundary | capability | uid-proof |
| s5 | sealed M2 bootstrap | owns | future exact parent-chain capability proof | root 0755 parent and pre-marker bootstrap audit | 1.0 | namespace owner | future path proof | bootstrap | path-boundary |
| s6 | live preflight evidence producer | publishes | unadmitted evidence receipt | admission path belongs only to independent auditor | 1.0 | evidence producer | evidence artifact | observation | producer-boundary |
| s7 | independent admission auditor | publishes_only_after | producer PASS and fresh clean snapshot | atomic 0444 admission publication | 1.0 | authorization owner | M2 execution admission | audit | auditor-publication |
| s8 | failed or unknown live predicate | leaves_absent | M2 execution admission | fail-closed BLOCKED_PROVENANCE dispatch | 1.0 | failure owner | execution authority | failure | failure-closure |
| s9 | execution admission | authorizes_once | sealed V3 controller run entry | exact task transaction and source hashes | 1.0 | authorization owner | execution orchestrator | authority | execution-boundary |
| s10 | admission transaction | preserves | production runtime and scientific marker ledger | zero Cargo perf subject namespace and marker effects | 1.0 | preservation owner | runtime and one-shot state | preservation | scientific-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M2 V3 predecessor authority | authorizes | independent live preflight producer | sealed next_action_admitted exact field | 1.0 | authorization owner | observation owner | authority | predecessor-chain |
| c2 | live preflight producer | observes_twice | host inputs namespaces processes and perf-event descriptors | fixed 15-second quiet window | 1.0 | observation owner | live target state | observation | live-closure |
| c3 | foreign performance process | vetoes | M2 execution admission | no overlapping scored experiment contract | 1.0 | conflict owner | execution authority | veto | conflict-exclusion |
| c4 | disposable UID e probe | proves | neutral ancestor traverse write fsync rename read unlink capability | actual operations and absent cleanup | 1.0 | capability producer | UID boundary | capability | uid-proof |
| c5 | sealed M2 bootstrap | owns | future exact parent-chain capability proof | root 0755 parent and pre-marker bootstrap audit | 1.0 | namespace owner | future path proof | bootstrap | path-boundary |
| c6 | live preflight evidence producer | publishes | unadmitted evidence receipt | admission path belongs only to independent auditor | 1.0 | evidence producer | evidence artifact | observation | producer-boundary |
| c7 | independent admission auditor | publishes_only_after | producer PASS and fresh clean snapshot | atomic 0444 admission publication | 1.0 | authorization owner | M2 execution admission | audit | auditor-publication |
| c8 | failed or unknown live predicate | leaves_absent | M2 execution admission | fail-closed BLOCKED_PROVENANCE dispatch | 1.0 | failure owner | execution authority | failure | failure-closure |
| c9 | execution admission | authorizes_once | sealed V3 controller run entry | exact task transaction and source hashes | 1.0 | authorization owner | execution orchestrator | authority | execution-boundary |
| c10 | admission transaction | preserves | production runtime and scientific marker ledger | zero Cargo perf subject namespace and marker effects | 1.0 | preservation owner | runtime and one-shot state | preservation | scientific-boundary |

## notes

- A dirty observation is repeatable because it creates no M2 scientific state.
- Exact future-path capability remains a bootstrap obligation before markers.
- Structural PASS alone cannot publish admission; implementation preflight and live audit remain mandatory.
