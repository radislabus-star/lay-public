# NANDA Triad Worksheet

task_id: w1-fused-minimum-m2r1-build-closure-repair-v1
domain: general
query: Can a fresh M2R1 namespace repair only the parity evidence serialization defect while preserving the original scientific experiment?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | original M2 | terminated as | BLOCKED_BUILD | sealed terminal receipt and Cargo log show exit 101 before parity or subjects | 1.0 | historical evidence | terminal verdict | history-terminal | m2-terminal |
| s2 | original M2 marker tree | forbids | retry or marker recreation | BUILD consumed and retry false | 1.0 | one-shot owner | old execution authority | history-marker | m2-marker-authority |
| s3 | build defect | belongs to | test-only parity signature serialization | rustc E0277 names Vec V13TypedPeak in m2_result_signature | 1.0 | defect owner | diagnostic evidence encoder | repair | defect-localization |
| s4 | M2R1 source repair | projects | peak fields into JSON values | repaired fragment maps form_ref and certificate_keys explicitly | 1.0 | repair owner | serializable evidence value | repair | source-change |
| s5 | M2R1 source repair | preserves | D1 fragment and production prefixes | pinned prefix hashes remain exact | 1.0 | repair owner | production and denominator closure | preservation | byte-boundary |
| s6 | disposable compile proof | precedes | M2R1 marker creation | exact repaired assembly compiled with exit zero outside scientific namespace | 1.0 | validation owner | one-shot admission | pre-marker | compile-closure |
| s7 | M2R1 namespace | owns | eight fresh one-shot markers | new task and transaction IDs do not reuse original M2 state | 1.0 | execution owner | marker authority | execution | fresh-namespace |
| s8 | M2R1 experiment | preserves | six-route scientific contract | candidate code route order events denominators and gates are unchanged | 1.0 | scientific owner | mechanism decision | preservation | scientific-boundary |
| s9 | M2R1 terminal audit | alone decides | mechanism PASS REJECT or BLOCKED | intermediate values are not interpreted | 1.0 | decision owner | final verdict | decision | terminal-only |
| s10 | M2R1 result | cannot authorize | production runtime change | repair contract forbids runtime edits install restart and deployment | 1.0 | evidence owner | runtime authority | boundary | no-promotion |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | original M2 | terminated as | BLOCKED_BUILD | immutable receipt and exact compiler diagnostic | 1.0 | historical evidence | terminal verdict | history-terminal | m2-terminal |
| c2 | original M2 marker tree | forbids | retry or marker recreation | one-shot terminal contract | 1.0 | one-shot owner | old execution authority | history-marker | m2-marker-authority |
| c3 | build defect | belongs to | test-only parity signature serialization | compiler diagnostic and source location agree | 1.0 | defect owner | diagnostic evidence encoder | repair | defect-localization |
| c4 | M2R1 source repair | projects | peak fields into JSON values | explicit value projection compiles | 1.0 | repair owner | serializable evidence value | repair | source-change |
| c5 | M2R1 source repair | preserves | D1 fragment and production prefixes | measured exact hashes | 1.0 | repair owner | production and denominator closure | preservation | byte-boundary |
| c6 | disposable compile proof | precedes | M2R1 marker creation | local source-closure build passed without markers | 1.0 | validation owner | one-shot admission | pre-marker | compile-closure |
| c7 | M2R1 namespace | owns | eight fresh one-shot markers | distinct task and transaction identities | 1.0 | execution owner | marker authority | execution | fresh-namespace |
| c8 | M2R1 experiment | preserves | six-route scientific contract | no scientific field changes | 1.0 | scientific owner | mechanism decision | preservation | scientific-boundary |
| c9 | M2R1 terminal audit | alone decides | mechanism PASS REJECT or BLOCKED | terminal-only interpretation rule | 1.0 | decision owner | final verdict | decision | terminal-only |
| c10 | M2R1 result | cannot authorize | production runtime change | explicit no-promotion boundary | 1.0 | evidence owner | runtime authority | boundary | no-promotion |

## notes

- The old M2 namespace remains immutable and terminal.
- The local compile proof is implementation evidence only, not scientific data.
- M2R1 changes no candidate mechanism, denominator, event, threshold, or route order.
