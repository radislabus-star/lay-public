# NANDA Triad Worksheet

task_id: lay-v10-b-hardware-contract-v4
domain: code
query: Check corrected B0a build freezer B0b sequence and B0-B7 route boundaries

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
| s15 | B0 diagnostic executable | must_be_built_once_between | B0a input closure and schedule-freezer | parse_v7_cases is test-only and historical ELF lacks freezer entrypoint | 1.0 | build owner | sequencing boundary | execution | schedule-freezer |
| s16 | B0 schedule-freezer | must_run_between | one diagnostic build and B0b schedule closure | user sequencing correction 2026-08-24 | 1.0 | schedule owner | sequencing boundary | execution | schedule-freezer |
| s17 | B design audit | must_distinguish | bare perf invocation from PMU measurement | verified session trace 2026-08-24T17:33:52Z | 1.0 | audit owner | measurement boundary | audit | audit-boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | historical exact ELF | executes_as | combined A B C workload | B contract lines 10..46 | 1.0 | historical executable | combined workload | historical | historical-envelope |
| c2 | historical exact ELF | cannot_attribute | executor phases | B contract lines 374..430 | 1.0 | historical executable | phase attribution | historical | phase-boundary |
| c3 | B3 observer | measures_only | combined historical envelope | B contract lines 337..373 | 1.0 | aggregate observer | combined workload | historical | historical-envelope-observer |
| c4 | diagnostic proxy | is_distinct_from | historical exact ELF | B contract lines 394..430 | 1.0 | diagnostic candidate | historical executable | proxy | proxy-identity |
| c5 | diagnostic proxy | must_preserve | exact 39047-byte production prefix | B contract lines 398..425 | 1.0 | diagnostic candidate | source invariant | proxy | proxy-source |
| c6 | diagnostic proxy | must_pass_before_hardware | exact 382-case production and full-row parity | B contract lines 420..430 | 1.0 | diagnostic candidate | parity prerequisite | proxy | proxy-parity |
| c7 | B1 environment gate | must_freeze | topology governor affinity load and thermal state | B contract lines 207..280 | 1.0 | environment gate | environment state | environment | environment |
| c8 | B2 PMU gate | must_reject | unsupported substituted or multiplexed events | B contract lines 281..336 | 1.0 | PMU gate | invalid event evidence | PMU | PMU |
| c9 | B5 one-client schedule | must_use | 382 fixed queries on one fixed P-core CPU | B contract lines 431..467 | 1.0 | diagnostic scheduler | one-client denominator | proxy | one-client |
| c10 | B6 twenty-client schedule | must_preserve | 20 fixed chunks and singleton worker affinities | B contract lines 468..497 | 1.0 | diagnostic scheduler | twenty-client denominator | proxy | twenty-client |
| c11 | B hardware characterization | must_remain_separate_from | pristine C latency | B contract lines 526..614 | 1.0 | diagnostic evidence | acceptance latency | proof | denominator-boundary |
| c12 | B completion status | does_not_admit | V12 implementation | B contract lines 597..614 | 1.0 | scoped verdict | future implementation | admission | V12-admission |
| c13 | current B paper stage | forbids_until_preflight | perf V10 Cargo and diagnostic build | B contract lines 43..46 | 1.0 | current gate | executable actions | admission | current-admission |
| c14 | B outputs | must_not_overwrite | installed inputs P0 archives or repository receipts | B contract lines 65..206 and 615..645 | 1.0 | experiment owner | protected artifacts | control | side-effects |
| c15 | B0 diagnostic executable | must_be_built_once_between | B0a input closure and schedule-freezer | B contract lines 153..192 | 1.0 | build owner | sequencing boundary | execution | schedule-freezer |
| c16 | B0 schedule-freezer | must_run_between | one diagnostic build and B0b schedule closure | B contract lines 153..206 | 1.0 | schedule owner | sequencing boundary | execution | schedule-freezer |
| c17 | B design audit | must_distinguish | bare perf invocation from PMU measurement | B contract lines 48..64 | 1.0 | audit owner | measurement boundary | audit | audit-boundary |

## notes

- The schedule-freezer is an explicit route with its own build and execution
  order; it is not treated as a file already present in B0a.
- The packet checks structure only. It cannot authorize build, freezer, perf,
  V10, PMU measurement or V12.
