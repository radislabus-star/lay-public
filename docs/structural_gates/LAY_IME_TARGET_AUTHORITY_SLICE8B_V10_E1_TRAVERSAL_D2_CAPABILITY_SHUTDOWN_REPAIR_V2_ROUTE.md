# NANDA Triad Worksheet

task_id: lay-v10-e1-traversal-d2-capability-shutdown-repair-v2
domain: code
query: Preserve terminal V1, recover sealed T-CAP offline, then test only missing precise channels

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V1 preservation owner | retains_unchanged | terminal receipt marker and sealed evidence | repair V2 immutable V1 boundary | 1.0 | history owner | evidence | preservation | d2-repair-preservation |
| s2 | scope owner | classifies_as | controller shutdown protocol with T-CAP unknown | repair V2 scope and taxonomy | 1.0 | decision owner | bounded verdict | interpretation | d2-repair-interpretation |
| s3 | T-CAP salvage owner | reads_only | sealed V1 perf data and maps | repair V2 R1 offline salvage | 1.0 | reader owner | evidence | salvage | d2-repair-salvage |
| s4 | shutdown owner | validates_before | accepted zero or negative SIGINT return | repair V2 R2 controlled shutdown | 1.0 | control owner | lifecycle result | control | d2-repair-shutdown |
| s5 | precise sequence owner | executes_once_after | recovered T-CAP then core then atom | repair V2 R3 one-shot sequence | 1.0 | execution owner | evidence | precise | d2-repair-precise |
| s6 | final decision owner | emits_only | capability ready or named blocker | repair V2 R4 final decision | 1.0 | authority owner | bounded result | decision | d2-repair-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V1 preservation owner | retains_unchanged | terminal receipt marker and sealed evidence | pinned V1 hashes and no-retry boundary | 1.0 | history owner | evidence | preservation | d2-repair-preservation |
| c2 | scope owner | classifies_as | controller shutdown protocol with T-CAP unknown | readers never ran and precise routes never started | 1.0 | decision owner | bounded verdict | interpretation | d2-repair-interpretation |
| c3 | T-CAP salvage owner | reads_only | sealed V1 perf data and maps | four preregistered reader commands and no subject | 1.0 | reader owner | evidence | salvage | d2-repair-salvage |
| c4 | shutdown owner | validates_before | accepted zero or negative SIGINT return | controller-requested single SIGINT protocol | 1.0 | control owner | lifecycle result | control | d2-repair-shutdown |
| c5 | precise sequence owner | executes_once_after | recovered T-CAP then core then atom | separate markers and fail-closed core-before-atom state | 1.0 | execution owner | evidence | precise | d2-repair-precise |
| c6 | final decision owner | emits_only | capability ready or named blocker | three conjunctive channel receipts required | 1.0 | authority owner | bounded result | decision | d2-repair-decision |

## notes

- V1 BLOCKED_CAPABILITY remains immutable and is not renamed.
- R1 runs readers only; it cannot record, execute yes or open an event.
- R3 cannot exist before R1 PASS and a separate implementation preflight.
- Structural PASS is coherence-only and cannot admit D2, full B, V12 or runtime mutation.
