# NANDA Triad Worksheet

task_id: w1-fused-minimum-m2-failure-publication-correction-v3
domain: general
query: Does the M2 V3 correction close controller exceptions without changing the scientific route?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V2 local controller | can strand | consumed remote route without local terminal receipt | execute_once has try finally but no exception publication | 1.0 | defect owner | lost local observation | evidence | failure-publication |
| s2 | V3 execution journal | precedes | every external action | correction requires fsynced immutable intent before SSH SCP auditor or producer | 1.0 | observation owner | external action | journal | intent-before-action |
| s3 | uncompleted intent | blocks | all retry | correction fixes failure default as BLOCKED_PROVENANCE and retry false | 1.0 | provenance owner | one-shot authority | failure | terminal-provenance |
| s4 | V3 exception handler | publishes | immutable controller-failure receipt | correction preserves exact known values and UNKNOWN fields | 1.0 | publication owner | terminal observation | failure | exception-receipt |
| s5 | V3 repair | preserves | M2 candidate build parity route and decision contracts | correction changes only local exception publication and source identity | 1.0 | repair owner | scientific contract | preservation | scientific-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V2 local controller | can strand | consumed remote route without local terminal receipt | observed source exception topology | 1.0 | defect owner | lost local observation | evidence | failure-publication |
| c2 | V3 execution journal | precedes | every external action | preregistered intent sequence and fsync boundary | 1.0 | observation owner | external action | journal | intent-before-action |
| c3 | uncompleted intent | blocks | all retry | immutable default verdict and no-resume rule | 1.0 | provenance owner | one-shot authority | failure | terminal-provenance |
| c4 | V3 exception handler | publishes | immutable controller-failure receipt | exact known or UNKNOWN projection fields | 1.0 | publication owner | terminal observation | failure | exception-receipt |
| c5 | V3 repair | preserves | M2 candidate build parity route and decision contracts | no scientific contract field changes | 1.0 | repair owner | scientific contract | preservation | scientific-boundary |

## notes

- V2 implementation bytes and receipt remain immutable historical evidence.
- No remote or scientific action is admitted by this worksheet.
- A missing completion never authorizes resume or retry.
