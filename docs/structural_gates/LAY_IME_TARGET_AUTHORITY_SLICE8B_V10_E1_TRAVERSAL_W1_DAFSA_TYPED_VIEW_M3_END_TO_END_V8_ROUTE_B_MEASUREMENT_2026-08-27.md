# NANDA Triad Worksheet

task_id: m3-end-to-end-v8-route-b-measurement
domain: general
query: Does V8 bind latency, semantic, scratch and PSS claims to exact measured denominators?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | fixed 382 cases x four rounds | produces | 1528 request samples | end-to-end-v8.md:110-114 | 1.0 | fixed request denominator | latency observations | latency | latency |
| s2 | maximum per-round search p99 | must_not_exceed | 3000 us | end-to-end-v8.md:129-134 | 1.0 | search metric owner | fixed threshold | search-gate | search-gate |
| s3 | maximum per-round total p99 | must_not_exceed | 5000 us | end-to-end-v8.md:134-135 | 1.0 | total metric owner | fixed threshold | total-gate | total-gate |
| s4 | measured exact requests | require | zero semantic and authority mismatch | end-to-end-v8.md:139-152 | 1.0 | owner-path observations | fixed parity contract | semantics | semantics |
| s5 | maximum query scratch | must_not_exceed | 512 KiB | end-to-end-v8.md:135-136 | 1.0 | scratch observation | fixed threshold | scratch | scratch |
| s6 | two helper processes | measure | sidecar plus typed PSS delta | end-to-end-v8.md:160-168 | 1.0 | physical residency observers | generation residency | pss | pss |
| s7 | aggregate helper PSS delta | must_not_exceed | 40 MiB | end-to-end-v8.md:172-177 | 1.0 | aggregate PSS metric | fixed threshold | pss-gate | pss-gate |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | fixed 382 cases x four rounds | produces | 1528 request samples | receipt denominator | 1.0 | fixed request denominator | latency observations | latency | latency |
| c2 | maximum per-round search p99 | must_not_exceed | 3000 us | receipt gate | 1.0 | search metric owner | fixed threshold | search-gate | search-gate |
| c3 | maximum per-round total p99 | must_not_exceed | 5000 us | receipt gate | 1.0 | total metric owner | fixed threshold | total-gate | total-gate |
| c4 | measured exact requests | require | zero semantic and authority mismatch | full owner audit | 1.0 | owner-path observations | fixed parity contract | semantics | semantics |
| c5 | maximum query scratch | must_not_exceed | 512 KiB | search observation | 1.0 | scratch observation | fixed threshold | scratch | scratch |
| c6 | two helper processes | measure | sidecar plus typed PSS delta | smaps_rollup before and after | 1.0 | physical residency observers | generation residency | pss | pss |
| c7 | aggregate helper PSS delta | must_not_exceed | 40 MiB | receipt gate | 1.0 | aggregate PSS metric | fixed threshold | pss-gate | pss-gate |

## notes

- Package loading, generation construction and helper work are outside request timers.
- Pooled p99 cannot hide a failing per-round p99.
