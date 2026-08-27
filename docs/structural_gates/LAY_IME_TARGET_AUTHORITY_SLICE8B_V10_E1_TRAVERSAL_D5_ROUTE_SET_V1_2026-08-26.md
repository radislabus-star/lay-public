# NANDA Triad Worksheet

task_id: d5-multiworker-tid-estimator-v1
domain: general
query: Is D5 structurally closed from terminal D4 through exact multiworker TID attribution and terminal evidence?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | history owner | preserves_terminal | D2 D3 and D4 receipts states and consumed markers unchanged | sealed predecessor receipts | 1.0 | history owner | immutable history | history | d5-history |
| s2 | reuse owner | admits_only | exact audited D2 ELF map parity and D4 single attribution | D5 predecessor closure | 1.0 | reuse owner | predecessor closure | reuse | d5-reuse |
| s3 | bootstrap owner | creates_before_markers | traversable D5 namespace and one-shot UID e proof | D5 markerless contract | 1.0 | bootstrap owner | execution admission | bootstrap | d5-bootstrap |
| s4 | audit owner | proves_by | live read-only projection and real scp as e before markers | D5 independent audit contract | 1.0 | audit owner | evidence mirror | audit | d5-audit |
| s5 | marker owner | creates_after | exact bootstrap audit PASS and before U4-FIXED | D5 marker state machine | 1.0 | marker owner | scientific authority | markers | d5-markers |
| s6 | envelope owner | executes_exactly | component twenty with fixed then reversed CPU mappings | D5 subject contract | 1.0 | envelope owner | subject execution | envelope | d5-envelope |
| s7 | TID owner | reconstructs_from | one libtest parent and exactly twenty direct worker FORK children | D5 raw lifecycle contract | 1.0 | TID owner | scientific role identity | tid | d5-tid |
| s8 | U4 fixed owner | measures | twenty measured fixed rounds times 25145756 edges | D5 U4 fixed contract | 1.0 | denominator owner | paired no-perf denominator | u4-fixed | d5-u4-fixed |
| s9 | T4 fixed owner | samples | twenty-one fixed bursts over exact worker TIDs | D5 T4 fixed contract | 1.0 | sampling owner | fixed attribution | t4-fixed | d5-t4-fixed |
| s10 | U4 reversed owner | measures | twenty measured reversed rounds times 25145756 edges | D5 U4 reversed contract | 1.0 | denominator owner | paired no-perf denominator | u4-reversed | d5-u4-reversed |
| s11 | T4 reversed owner | samples | twenty-one reversed bursts over exact worker TIDs | D5 T4 reversed contract | 1.0 | sampling owner | reversed attribution | t4-reversed | d5-t4-reversed |
| s12 | sampling owner | executes_only | task-clock user period 200000 on T4 routes | D5 command graph | 1.0 | sampling owner | perf evidence | sampling | d5-sampling |
| s13 | one-shot owner | orders | marker audit then paired U PASS before each T consumption | D5 state machine | 1.0 | one-shot owner | route authority | control | d5-one-shot |
| s14 | decision owner | reconciles | per-bucket sampled inflation to total fixed and reversed inflation | D5 terminal contract | 1.0 | decision owner | attribution claim | decision | d5-decision |
| s15 | claim owner | limits | D5 PASS to separate paper optimization decision | D5 claim boundary | 1.0 | claim owner | next authority | boundary | d5-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | history owner | preserves_terminal | D2 D3 and D4 receipts states and consumed markers unchanged | reviewed D5 history route | 1.0 | history owner | immutable history | history | d5-history |
| c2 | reuse owner | admits_only | exact audited D2 ELF map parity and D4 single attribution | reviewed D5 reuse route | 1.0 | reuse owner | predecessor closure | reuse | d5-reuse |
| c3 | bootstrap owner | creates_before_markers | traversable D5 namespace and one-shot UID e proof | reviewed D5 bootstrap route | 1.0 | bootstrap owner | execution admission | bootstrap | d5-bootstrap |
| c4 | audit owner | proves_by | live read-only projection and real scp as e before markers | reviewed D5 audit route | 1.0 | audit owner | evidence mirror | audit | d5-audit |
| c5 | marker owner | creates_after | exact bootstrap audit PASS and before U4-FIXED | reviewed D5 marker route | 1.0 | marker owner | scientific authority | markers | d5-markers |
| c6 | envelope owner | executes_exactly | component twenty with fixed then reversed CPU mappings | reviewed D5 subject route | 1.0 | envelope owner | subject execution | envelope | d5-envelope |
| c7 | TID owner | reconstructs_from | one libtest parent and exactly twenty direct worker FORK children | reviewed D5 lifecycle route | 1.0 | TID owner | scientific role identity | tid | d5-tid |
| c8 | U4 fixed owner | measures | twenty measured fixed rounds times 25145756 edges | reviewed D5 U4 fixed denominator | 1.0 | denominator owner | paired no-perf denominator | u4-fixed | d5-u4-fixed |
| c9 | T4 fixed owner | samples | twenty-one fixed bursts over exact worker TIDs | reviewed D5 T4 fixed estimator | 1.0 | sampling owner | fixed attribution | t4-fixed | d5-t4-fixed |
| c10 | U4 reversed owner | measures | twenty measured reversed rounds times 25145756 edges | reviewed D5 U4 reversed denominator | 1.0 | denominator owner | paired no-perf denominator | u4-reversed | d5-u4-reversed |
| c11 | T4 reversed owner | samples | twenty-one reversed bursts over exact worker TIDs | reviewed D5 T4 reversed estimator | 1.0 | sampling owner | reversed attribution | t4-reversed | d5-t4-reversed |
| c12 | sampling owner | executes_only | task-clock user period 200000 on T4 routes | reviewed D5 sampling route | 1.0 | sampling owner | perf evidence | sampling | d5-sampling |
| c13 | one-shot owner | orders | marker audit then paired U PASS before each T consumption | reviewed D5 state route | 1.0 | one-shot owner | route authority | control | d5-one-shot |
| c14 | decision owner | reconciles | per-bucket sampled inflation to total fixed and reversed inflation | reviewed D5 terminal route | 1.0 | decision owner | attribution claim | decision | d5-decision |
| c15 | claim owner | limits | D5 PASS to separate paper optimization decision | reviewed D5 claim boundary | 1.0 | claim owner | next authority | boundary | d5-boundary |

## notes

- Each linked group is independently required to PASS.
- Structural coherence grants no controller, namespace, marker, subject, perf,
  scientific, optimization, build, integration, or runtime authority.
