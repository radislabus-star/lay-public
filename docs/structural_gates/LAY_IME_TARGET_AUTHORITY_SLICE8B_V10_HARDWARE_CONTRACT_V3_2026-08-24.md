# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-contract-v3
domain: code
query: Check B0-B7 hardware characterization evidence contract route separation and claim boundary

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | historical exact ELF | executes_as | combined A B C workload | exact source lines 1514..1615 | 1.0 | historical executable | combined workload | historical | historical-envelope |
| s2 | historical exact ELF | cannot_attribute | executor phases | stripped single-entrypoint evidence | 1.0 | historical executable | phase attribution | historical | phase-boundary |
| s3 | B3 observer | measures_only | combined historical envelope | user requires exact route and proof boundary | 1.0 | aggregate observer | combined workload | historical | historical-envelope-observer |
| s4 | diagnostic proxy | is_distinct_from | historical exact ELF | P0 full source closure WATCH | 1.0 | diagnostic candidate | historical executable | proxy | proxy-identity |
| s5 | diagnostic proxy | must_preserve | exact 39047-byte production prefix | exact cfg test boundary and no instrumentation rule | 1.0 | diagnostic candidate | source invariant | proxy | proxy-source |
| s6 | diagnostic proxy | must_pass_before_hardware | exact 382-case production and full-row parity | systemic proof requirement | 1.0 | diagnostic candidate | parity prerequisite | proxy | proxy-parity |
| s7 | B1 environment gate | must_freeze | topology governor affinity load and thermal state | user_request:2026-08-24#hardware-environment | 1.0 | environment gate | environment state | environment | environment |
| s8 | B2 PMU gate | must_reject | unsupported substituted or multiplexed events | user_request:2026-08-24#perf-counters | 1.0 | PMU gate | invalid event evidence | PMU | PMU |
| s9 | B5 one-client schedule | must_use | 382 fixed queries on one fixed P-core CPU | requested one-client schedule plus fixed denominator | 1.0 | diagnostic scheduler | one-client denominator | proxy | one-client |
| s10 | B6 twenty-client schedule | must_preserve | 20 fixed chunks and singleton worker affinities | exact source lines 2136..2173 plus user affinity requirement | 1.0 | diagnostic scheduler | twenty-client denominator | proxy | twenty-client |
| s11 | B hardware characterization | must_remain_separate_from | pristine C latency | P0 frozen measurement contract | 1.0 | diagnostic evidence | acceptance latency | proof | denominator-boundary |
| s12 | B completion status | does_not_admit | V12 implementation | user boundary and P0 correction | 1.0 | scoped verdict | future implementation | admission | V12-admission |
| s13 | current B paper stage | forbids_until_preflight | perf V10 Cargo and diagnostic build | explicit current user boundary | 1.0 | current gate | executable actions | admission | current-admission |
| s14 | B outputs | must_not_overwrite | installed inputs P0 archives or repository receipts | P0 correction and isolated evidence requirement | 1.0 | experiment owner | protected artifacts | control | side-effects |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | historical exact ELF | executes_as | combined A B C workload | B contract lines 10..46 | 1.0 | historical executable | combined workload | historical | historical-envelope |
| c2 | historical exact ELF | cannot_attribute | executor phases | B contract lines 309..360 | 1.0 | historical executable | phase attribution | historical | phase-boundary |
| c3 | B3 observer | measures_only | combined historical envelope | B contract lines 272..308 | 1.0 | aggregate observer | combined workload | historical | historical-envelope-observer |
| c4 | diagnostic proxy | is_distinct_from | historical exact ELF | B contract lines 329..360 | 1.0 | diagnostic candidate | historical executable | proxy | proxy-identity |
| c5 | diagnostic proxy | must_preserve | exact 39047-byte production prefix | B contract lines 333..359 | 1.0 | diagnostic candidate | source invariant | proxy | proxy-source |
| c6 | diagnostic proxy | must_pass_before_hardware | exact 382-case production and full-row parity | B contract lines 350..359 | 1.0 | diagnostic candidate | parity prerequisite | proxy | proxy-parity |
| c7 | B1 environment gate | must_freeze | topology governor affinity load and thermal state | B contract lines 142..215 | 1.0 | environment gate | environment state | environment | environment |
| c8 | B2 PMU gate | must_reject | unsupported substituted or multiplexed events | B contract lines 216..271 | 1.0 | PMU gate | invalid event evidence | PMU | PMU |
| c9 | B5 one-client schedule | must_use | 382 fixed queries on one fixed P-core CPU | B contract lines 361..397 | 1.0 | diagnostic scheduler | one-client denominator | proxy | one-client |
| c10 | B6 twenty-client schedule | must_preserve | 20 fixed chunks and singleton worker affinities | B contract lines 398..427 | 1.0 | diagnostic scheduler | twenty-client denominator | proxy | twenty-client |
| c11 | B hardware characterization | must_remain_separate_from | pristine C latency | B contract lines 456..544 | 1.0 | diagnostic evidence | acceptance latency | proof | denominator-boundary |
| c12 | B completion status | does_not_admit | V12 implementation | B contract lines 527..544 | 1.0 | scoped verdict | future implementation | admission | V12-admission |
| c13 | current B paper stage | forbids_until_preflight | perf V10 Cargo and diagnostic build | B contract lines 43..45 | 1.0 | current gate | executable actions | admission | current-admission |
| c14 | B outputs | must_not_overwrite | installed inputs P0 archives or repository receipts | B contract lines 47..141 and 545..572 | 1.0 | experiment owner | protected artifacts | control | side-effects |

## notes

- Source triads are requirements or sealed facts. Candidate triads are exact
  assertions extracted from the proposed B contract.
- The packet checks route and role preservation only. It has no trusted proof
  manifest and cannot authorize perf, V10, Cargo, proxy construction or V12.
