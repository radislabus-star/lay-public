# NANDA Triad Worksheet

task_id: m3-end-to-end-v8r2-direct-exec-correction-v1
domain: general
query: Does V8R2 reuse the exact V8R1 ELF while changing only the loader lifecycle and preserving one-shot history?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | V8R1 terminal evidence | remains | immutable BLOCKED_PROVENANCE history | direct-exec-v1.md:7-19 | 1.0 | historical experiment | terminal predecessor | history | history |
| s2 | V8R2 bootstrap | copies_bytes_from | audited V8R1 ELF | direct-exec-v1.md:63-82 | 1.0 | execution-envelope producer | sealed machine bytes | bootstrap | bootstrap |
| s3 | mode 0555 executable copy | preserves | V8R1 ELF SHA and size | direct-exec-v1.md:71-82 | 1.0 | executable projection | machine-byte identity | identity | identity |
| s4 | V8R2 subject | executes_directly | executable copy | direct-exec-v1.md:86-109 | 1.0 | scientific parent | exact executable path | execution | execution |
| s5 | V8R2 route | excludes | BUILD Cargo rustc loader perf and PMU | direct-exec-v1.md:48-58,102-109 | 1.0 | closed command graph | forbidden producers | veto | veto |
| s6 | V8R2 PASS | requires | complete fresh V8 scientific receipt | direct-exec-v1.md:111-130 | 1.0 | scoped verdict | conjunctive proof | science | science |
| s7 | V8R2 PASS | cannot_grant | production activation or runtime mutation | direct-exec-v1.md:172-180 | 1.0 | test-owner evidence | production authority | boundary | boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | V8R1 terminal evidence | remains | immutable BLOCKED_PROVENANCE history | exact predecessor hashes | 1.0 | historical experiment | terminal predecessor | history | history |
| c2 | V8R2 bootstrap | copies_bytes_from | audited V8R1 ELF | closed remote producer | 1.0 | execution-envelope producer | sealed machine bytes | bootstrap | bootstrap |
| c3 | mode 0555 executable copy | preserves | V8R1 ELF SHA and size | independent byte audit | 1.0 | executable projection | machine-byte identity | identity | identity |
| c4 | V8R2 subject | executes_directly | executable copy | exact argv registry | 1.0 | scientific parent | exact executable path | execution | execution |
| c5 | V8R2 route | excludes | BUILD Cargo rustc loader perf and PMU | reachable command graph audit | 1.0 | closed command graph | forbidden producers | veto | veto |
| c6 | V8R2 PASS | requires | complete fresh V8 scientific receipt | independent terminal dispatch | 1.0 | scoped verdict | conjunctive proof | science | science |
| c7 | V8R2 PASS | cannot_grant | production activation or runtime mutation | explicit terminal claim boundary | 1.0 | test-owner evidence | production authority | boundary | boundary |

## notes

- Structural PASS is coherence only; implementation requires a separate READY preflight.
- V8R2 has one route and one marker. It has no build route.
