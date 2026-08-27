# NANDA Triad Worksheet

task_id: d7-worker-topology-measurement-v1
domain: general
query: Is the D7 worker and topology measurement route structurally closed?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | D7 measurement controller | preserves | exact 39047-byte V10 production prefix | pinned source and prefix hashes | 1.0 | measurement owner | production bytes | build | d7-measurement |
| s2 | D7 measurement controller | contains_exactly | W1 W6 W12 W14 W20 CPU placements | frozen placement table | 1.0 | measurement owner | worker topology set | registry | d7-measurement |
| s3 | D7 measurement controller | partitions | all 382 queries once per round with ceil chunking | frozen chunk formula | 1.0 | measurement owner | exact query schedule | work | d7-measurement |
| s4 | D7 measurement controller | measures | 20 rounds and 502915120 examined edges | exact D1 structures and record schema | 1.0 | measurement owner | traversal CPU per edge | component | d7-measurement |
| s5 | D7 measurement controller | encloses_only | post-warmup measured rounds | FIFO enable and disable protocol | 1.0 | measurement owner | PMU denominator | pmu | d7-measurement |
| s6 | D7 measurement controller | requires | exact affinity zero migrations zero semantic errors and zero thermal drift | D7 hard gates | 1.0 | measurement owner | route PASS | validation | d7-measurement |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | D7 measurement controller | preserves | exact 39047-byte V10 production prefix | reviewed deterministic assembly rule | 1.0 | measurement owner | production bytes | build | d7-measurement |
| c2 | D7 measurement controller | contains_exactly | W1 W6 W12 W14 W20 CPU placements | reviewed topology intervention | 1.0 | measurement owner | worker topology set | registry | d7-measurement |
| c3 | D7 measurement controller | partitions | all 382 queries once per round with ceil chunking | reviewed denominator formula | 1.0 | measurement owner | exact query schedule | work | d7-measurement |
| c4 | D7 measurement controller | measures | 20 rounds and 502915120 examined edges | reviewed component record contract | 1.0 | measurement owner | traversal CPU per edge | component | d7-measurement |
| c5 | D7 measurement controller | encloses_only | post-warmup measured rounds | reviewed handshake ordering | 1.0 | measurement owner | PMU denominator | pmu | d7-measurement |
| c6 | D7 measurement controller | requires | exact affinity zero migrations zero semantic errors and zero thermal drift | reviewed hard gate dispatch | 1.0 | measurement owner | route PASS | validation | d7-measurement |

## notes

- The route observes a test-only ELF and does not edit production Rust.
- Generic hybrid event names must resolve to the route-specific active PMU set.
