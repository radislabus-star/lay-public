# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-tcap-salvage-interpretation-v3
domain: code
query: Reinterpret only sealed T-CAP reader outputs with exact zero-field and DSO-wrapper semantics

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | history owner | retains_unchanged | V1 and V2 terminal receipts | correction V3 scope | 1.0 | history owner | evidence | preservation | tcap-v3-preservation |
| s2 | attr interpretation owner | maps_exactly | absent optional task-clock flags to zero | correction V3 zero-field semantics | 1.0 | parser owner | bounded value | interpretation | tcap-v3-attr |
| s3 | DSO identity owner | canonicalizes_exactly | one outer parenthesis pair | correction V3 DSO canonicalization | 1.0 | identity owner | path | identity | tcap-v3-dso |
| s4 | ELF closure owner | accepts_only | SHA and Build-ID exact remote yes bytes | correction V3 exact ELF closure | 1.0 | provenance owner | artifact | closure | tcap-v3-elf |
| s5 | decision owner | emits_only | recovered T-CAP or named terminal blocker | correction V3 recovery conjuncts | 1.0 | authority owner | bounded result | decision | tcap-v3-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | history owner | retains_unchanged | V1 and V2 terminal receipts | pinned immutable receipt hashes | 1.0 | history owner | evidence | preservation | tcap-v3-preservation |
| c2 | attr interpretation owner | maps_exactly | absent optional task-clock flags to zero | fixed-count command and sealed task-clock attr line | 1.0 | parser owner | bounded value | interpretation | tcap-v3-attr |
| c3 | DSO identity owner | canonicalizes_exactly | one outer parenthesis pair | sealed perf script DSO format | 1.0 | identity owner | path | identity | tcap-v3-dso |
| c4 | ELF closure owner | accepts_only | SHA and Build-ID exact remote yes bytes | pinned V1 yes identity | 1.0 | provenance owner | artifact | closure | tcap-v3-elf |
| c5 | decision owner | emits_only | recovered T-CAP or named terminal blocker | all recomputed conjuncts required | 1.0 | authority owner | bounded result | decision | tcap-v3-decision |

## notes

- No perf reader, record, event or subject execution is admitted.
- Missing precision on future precise events cannot use the task-clock zero-default rule.
- Structural PASS remains coherence-only and cannot admit D2 or runtime mutation.
