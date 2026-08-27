# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-perf-audit-correction-v3-route-repair-v1
domain: code
query: Check corrected owner separation for the second pre-B2 perf invocation and execution stop

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | B audit owner | must_record | two pre-B2 perf executable invocations | session trace timestamps 17:33:52Z and 19:29:33.706Z | 1.0 | audit owner | invocation ledger | audit | audit-ledger-v3 |
| s2 | B audit owner | must_classify | second invocation as version query without PMU or subject | exact argv perf --version and output perf version 6.8.12 | 1.0 | audit owner | non-measurement observation | audit | audit-ledger-v3 |
| s3 | B audit owner | must_record | zero pre-B2 PMU measurements | no stat record event open or subject in either event | 1.0 | audit owner | measurement count | audit | audit-ledger-v3 |
| s4 | B implementation admission owner | must_supersede_for_execution | implementation preflight V2 with corrected audit baseline and V3 preflight | V2 frozen audit input changed after publication | 1.0 | admission owner | corrected admission requirement | admission | preflight-admission-v3 |
| s5 | B execution sequence owner | must_require_before_controller_implementation | V3 READY_TO_IMPLEMENT | fail-closed sequencing after unauthorized version query | 1.0 | sequence owner | corrected preflight | sequencing | execution-sequence-v3 |
| s6 | B execution sequence owner | must_preserve | B0a build freezer B0b B1 B2 sequence | correction changes invocation provenance only | 1.0 | sequence owner | preserved route | sequencing | execution-sequence-v3 |
| s7 | B implementation admission owner | must_not_admit | B3 parity B5 B6 or V12 | existing STOP boundary remains binding | 1.0 | admission owner | later gates | admission | preflight-admission-v3 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | B audit owner | must_record | two pre-B2 perf executable invocations | PERF_AUDIT_CORRECTION_V3 events and totals | 1.0 | audit owner | invocation ledger | audit | audit-ledger-v3 |
| c2 | B audit owner | must_classify | second invocation as version query without PMU or subject | PERF_AUDIT_CORRECTION_V3 event ordinal 2 | 1.0 | audit owner | non-measurement observation | audit | audit-ledger-v3 |
| c3 | B audit owner | must_record | zero pre-B2 PMU measurements | PERF_AUDIT_CORRECTION_V3 totals | 1.0 | audit owner | measurement count | audit | audit-ledger-v3 |
| c4 | B implementation admission owner | must_supersede_for_execution | implementation preflight V2 with corrected audit baseline and V3 preflight | PERF_AUDIT_CORRECTION_V3 provenance decision | 1.0 | admission owner | corrected admission requirement | admission | preflight-admission-v3 |
| c5 | B execution sequence owner | must_require_before_controller_implementation | V3 READY_TO_IMPLEMENT | corrected contract admission section | 1.0 | sequence owner | corrected preflight | sequencing | execution-sequence-v3 |
| c6 | B execution sequence owner | must_preserve | B0a build freezer B0b B1 B2 sequence | corrected contract sequence | 1.0 | sequence owner | preserved route | sequencing | execution-sequence-v3 |
| c7 | B implementation admission owner | must_not_admit | B3 parity B5 B6 or V12 | corrected contract claim boundary | 1.0 | admission owner | later gates | admission | preflight-admission-v3 |

## notes

- The earlier V3 worksheet and its owner-conflict VETO are retained as negative evidence.
- This repair separates audit, admission, and execution-sequence ownership without changing the corrected facts or admission boundary.
- Structural PASS cannot authorize implementation. A new implementation preflight V3 is required.
