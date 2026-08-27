# NANDA Triad Worksheet

task_id: d4-estimator-recovery-v1
domain: general
query: Is D4 structurally closed from pre-marker UID proof through single-worker terminal evidence?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | history owner | preserves_terminal | D2 and D3 BLOCKED_PROVENANCE with consumed markers unchanged | sealed terminal receipts | 1.0 | history owner | immutable history | history | d4-history |
| s2 | reuse owner | admits_only | exact audited D2 ELF map parity and U V evidence | D4 predecessor closure | 1.0 | reuse owner | predecessor closure | reuse | d4-reuse |
| s3 | bootstrap owner | creates_before_markers | traversable namespace and one-shot UID e proof | D4 pre-marker contract | 1.0 | bootstrap owner | execution admission | bootstrap | d4-bootstrap |
| s4 | audit owner | proves_by | live read-only projection and real scp as e before markers | D4 independent audit contract | 1.0 | audit owner | evidence mirror | audit | d4-audit |
| s5 | marker owner | creates_after | exact bootstrap audit PASS and before U3 | D4 marker state machine | 1.0 | marker owner | scientific authority | markers | d4-markers |
| s6 | envelope owner | stages_then_pins | CPU6 input loading then CPU0 traversal | D4 launch contract | 1.0 | envelope owner | process affinity | envelope | d4-envelope |
| s7 | U3 denominator owner | measures | twenty rounds times 25145756 edges | D4 U3 contract | 1.0 | U3 denominator owner | measured denominator | u3 | d4-u3-denominator |
| s8 | T3 denominator owner | samples | twenty-one rounds times 25145756 edges | D4 T3 contract | 1.0 | T3 denominator owner | sampled denominator | t3 | d4-t3-denominator |
| s9 | sampling owner | executes_only | task-clock user period 200000 on T3-SINGLE | D4 command graph | 1.0 | sampling owner | perf evidence | t3 | d4-sampling |
| s10 | one-shot owner | orders | marker audit then U3 PASS before T3 consumption | D4 state machine | 1.0 | one-shot owner | route authority | control | d4-one-shot |
| s11 | decision owner | limits | D4 PASS to separate paper decision | D4 claim boundary | 1.0 | decision owner | next authority | decision | d4-decision |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | history owner | preserves_terminal | D2 and D3 BLOCKED_PROVENANCE with consumed markers unchanged | reviewed D4 history route | 1.0 | history owner | immutable history | history | d4-history |
| c2 | reuse owner | admits_only | exact audited D2 ELF map parity and U V evidence | reviewed D4 reuse route | 1.0 | reuse owner | predecessor closure | reuse | d4-reuse |
| c3 | bootstrap owner | creates_before_markers | traversable namespace and one-shot UID e proof | reviewed D4 bootstrap route | 1.0 | bootstrap owner | execution admission | bootstrap | d4-bootstrap |
| c4 | audit owner | proves_by | live read-only projection and real scp as e before markers | reviewed D4 audit route | 1.0 | audit owner | evidence mirror | audit | d4-audit |
| c5 | marker owner | creates_after | exact bootstrap audit PASS and before U3 | reviewed D4 marker route | 1.0 | marker owner | scientific authority | markers | d4-markers |
| c6 | envelope owner | stages_then_pins | CPU6 input loading then CPU0 traversal | reviewed D4 launch route | 1.0 | envelope owner | process affinity | envelope | d4-envelope |
| c7 | U3 denominator owner | measures | twenty rounds times 25145756 edges | reviewed D4 U3 denominator | 1.0 | U3 denominator owner | measured denominator | u3 | d4-u3-denominator |
| c8 | T3 denominator owner | samples | twenty-one rounds times 25145756 edges | reviewed D4 T3 denominator | 1.0 | T3 denominator owner | sampled denominator | t3 | d4-t3-denominator |
| c9 | sampling owner | executes_only | task-clock user period 200000 on T3-SINGLE | reviewed D4 sampling route | 1.0 | sampling owner | perf evidence | t3 | d4-sampling |
| c10 | one-shot owner | orders | marker audit then U3 PASS before T3 consumption | reviewed D4 control route | 1.0 | one-shot owner | route authority | control | d4-one-shot |
| c11 | decision owner | limits | D4 PASS to separate paper decision | reviewed D4 decision route | 1.0 | decision owner | next authority | decision | d4-decision |

## notes

- Each linked group is independently required to PASS.
- Structural coherence grants no code, marker, subject, perf or scientific authority.
