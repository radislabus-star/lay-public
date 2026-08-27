# NANDA Triad Worksheet

task_id: m3-test-source-integration-format-correction-v2
domain: general
query: Does the corrected integration bind the current V11 validator to one deterministic in-memory sidecar and preserve M3 history without creating a second format authority?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| f1 | sealed M3 sidecar | has_format | LAYV13D2 12-byte records | exact header and M3 source | 1.0 | historical evidence | old encoding | provenance | provenance |
| f2 | current V13DafsaView | validates_only | LAYV13D3 8-8-4 records | current source d9edfe73 | 1.0 | byte validator | current encoding | provenance | provenance |
| f3 | format mismatch | supersedes | implementation preflight V2 READY | discovered before source edit | 1.0 | blocking fact | old admission | admission | admission |
| f4 | current compile_sidecar | produces | one in-memory V11 sidecar | current source equals archived active V11 source | 1.0 | deterministic producer | validated bytes | reconstruction | reconstruction |
| f5 | reconstructed V11 sidecar | must_match | 2460144 bytes and 5ebffb81 SHA | sealed V11 Gate A PASS | 1.0 | candidate bytes | fixed identity | reconstruction | reconstruction |
| f6 | current byte validator | authorizes_input_to | typed materializer | correction V2 ownership chain | 1.0 | format validator | safe decoder consumer | validation | validation |
| f7 | typed materializer | constructs_once | 3689628-byte immutable view | exact counts and 12-byte typed records | 1.0 | generation producer | proof generation | lifetime | lifetime |
| f8 | fixed proof | compares | 382 forward and reverse byte and typed results | correction V2 proof contract | 1.0 | proof observer | parity evidence | parity | parity |
| f9 | old V10 sidecar | remains | immutable historical evidence only | correction V2 preserved scope | 1.0 | historical evidence | non-input artifact | preservation | preservation |
| f10 | positive integration verdict | permits_only | actual-owner consequence paper | correction V2 claim boundary | 1.0 | scoped proof | next design gate | boundary | boundary |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | sealed M3 sidecar | has_format | LAYV13D2 12-byte records | candidate does not feed it to current loader | 1.0 | historical evidence | old encoding | provenance | provenance |
| c2 | current V13DafsaView | validates_only | LAYV13D3 8-8-4 records | candidate preserves current format owner | 1.0 | byte validator | current encoding | provenance | provenance |
| c3 | format mismatch | supersedes | implementation preflight V2 READY | candidate requires fresh admission | 1.0 | blocking fact | old admission | admission | admission |
| c4 | current compile_sidecar | produces | one in-memory V11 sidecar | candidate performs no file publication | 1.0 | deterministic producer | validated bytes | reconstruction | reconstruction |
| c5 | reconstructed V11 sidecar | must_match | 2460144 bytes and 5ebffb81 SHA | candidate fails closed on drift | 1.0 | candidate bytes | fixed identity | reconstruction | reconstruction |
| c6 | current byte validator | authorizes_input_to | typed materializer | candidate adds no second parser | 1.0 | format validator | safe decoder consumer | validation | validation |
| c7 | typed materializer | constructs_once | 3689628-byte immutable view | candidate reuses one owner for every query | 1.0 | generation producer | proof generation | lifetime | lifetime |
| c8 | fixed proof | compares | 382 forward and reverse byte and typed results | candidate requires zero mismatch | 1.0 | proof observer | parity evidence | parity | parity |
| c9 | old V10 sidecar | remains | immutable historical evidence only | candidate never mutates old artifact | 1.0 | historical evidence | non-input artifact | preservation | preservation |
| c10 | positive integration verdict | permits_only | actual-owner consequence paper | candidate denies production authority | 1.0 | scoped proof | next design gate | boundary | boundary |

## notes

- Gate PASS is coherence only and grants no implementation authority.
- V11 overall FAIL is not promoted; only independently passing Gate A format identity is consumed.
