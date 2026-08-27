# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-perf-audit-correction-v3
domain: code
query: Check second version-only perf invocation correction and execution stop before controller code

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | B audit owner | must_record | two pre-B2 perf executable invocations | session trace timestamps 17:33:52Z and 19:29:33.706Z | 1.0 | audit owner | invocation ledger | audit | audit-boundary-v3 |
| s2 | second perf invocation | is_only | version query without PMU or subject | exact argv perf --version and output perf version 6.8.12 | 1.0 | audit event | non-measurement observation | audit | audit-boundary-v3 |
| s3 | pre-B2 PMU measurement count | remains | zero | no stat record event open or subject in either event | 1.0 | audit invariant | measurement count | audit | audit-boundary-v3 |
| s4 | implementation preflight V2 | must_be_preserved_but_superseded_for_execution_by | corrected audit baseline and V3 preflight | V2 frozen audit input changed after publication | 1.0 | prior admission evidence | corrected admission requirement | admission | audit-boundary-v3 |
| s5 | controller implementation | must_wait_for | V3 READY_TO_IMPLEMENT | fail-closed sequencing after unauthorized version query | 1.0 | future implementation | corrected preflight | sequencing | audit-boundary-v3 |
| s6 | B0a build freezer B0b B1 B2 sequence | remains | unchanged | correction changes invocation provenance only | 1.0 | execution sequence | preserved route | sequencing | audit-boundary-v3 |
| s7 | correction V3 | does_not_admit | B3 parity B5 B6 or V12 | existing STOP boundary remains binding | 1.0 | correction verdict | later gates | admission | audit-boundary-v3 |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | B audit owner | must_record | two pre-B2 perf executable invocations | PERF_AUDIT_CORRECTION_V3 events and totals | 1.0 | audit owner | invocation ledger | audit | audit-boundary-v3 |
| c2 | second perf invocation | is_only | version query without PMU or subject | PERF_AUDIT_CORRECTION_V3 event ordinal 2 | 1.0 | audit event | non-measurement observation | audit | audit-boundary-v3 |
| c3 | pre-B2 PMU measurement count | remains | zero | PERF_AUDIT_CORRECTION_V3 totals | 1.0 | audit invariant | measurement count | audit | audit-boundary-v3 |
| c4 | implementation preflight V2 | must_be_preserved_but_superseded_for_execution_by | corrected audit baseline and V3 preflight | PERF_AUDIT_CORRECTION_V3 provenance decision | 1.0 | prior admission evidence | corrected admission requirement | admission | audit-boundary-v3 |
| c5 | controller implementation | must_wait_for | V3 READY_TO_IMPLEMENT | corrected contract admission section | 1.0 | future implementation | corrected preflight | sequencing | audit-boundary-v3 |
| c6 | B0a build freezer B0b B1 B2 sequence | remains | unchanged | corrected contract sequence | 1.0 | execution sequence | preserved route | sequencing | audit-boundary-v3 |
| c7 | correction V3 | does_not_admit | B3 parity B5 B6 or V12 | corrected contract claim boundary | 1.0 | correction verdict | later gates | admission | audit-boundary-v3 |

## notes

- This correction changes audit provenance only; it does not reinterpret a version query as PMU evidence.
- Structural PASS cannot authorize implementation. A new implementation preflight V3 is required.
