# NANDA Triad Worksheet

task_id: lay-ime-single-candidate-route-2026-08-12
domain: code
query: Replace duplicate ASCII and RU live candidate queries with one typed IME word-candidate route while preserving phrase forecast, display, Tab authorization, and rollback.

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| t1 | IBus ProcessKeyEvent | calls | process_pressed_key | src/bin/lay_ibus_engine/ibus_interface.rs:12 | 1.0 | interface | router | input | input-event | interface | LayIbusEngine | ProcessKeyEvent | handled key | src/bin/lay_ibus_engine/ibus_interface.rs:12 | production |
| t2 | visible character commit | calls | refresh_precognition_after_visible_input | src/bin/lay_ibus_engine/composition_commit.rs:146 | 1.0 | router | orchestrator | display | preedit-refresh | application | LayIbusEngine | committed character | refresh request | src/bin/lay_ibus_engine/composition_commit.rs:146 | production |
| t3 | refresh_precognition_candidates | calls | precognition_candidates | src/bin/lay_ibus_engine/preedit.rs:362 | 1.0 | display-state | orchestrator | display | preedit-refresh | application | LayIbusEngine | active token | candidate proposals | src/bin/lay_ibus_engine/preedit.rs:362 | production |
| t4 | precognition_candidates | calls | semantic_phrase_candidates | src/bin/lay_ibus_engine/preedit.rs:426 | 1.0 | orchestrator | phrase-producer | phrase | phrase-forecast | adapter | LayIbusEngine | phrase tail | suffix proposals | src/bin/lay_ibus_engine/preedit.rs:426 | production |
| t5 | precognition_candidates | calls | ru_l2_word_attractor_candidates | src/bin/lay_ibus_engine/preedit.rs:426 | 1.0 | orchestrator | word-producer | word | word-field | adapter | LayIbusEngine | active token | typed proposals | src/bin/lay_ibus_engine/preedit.rs:426 | production |
| t6 | precognition_candidates | calls | ascii_candidates | src/bin/lay_ibus_engine/preedit.rs:426 | 1.0 | orchestrator | word-producer | word | duplicate-word-field | adapter | PreeditFastState | ASCII active token | typed proposals | src/bin/lay_ibus_engine/preedit.rs:426 | production |
| t7 | ru_l2_word_attractor_candidates | calls | TypingCpu live_completion_candidates | src/bin/lay_ibus_engine/preedit_readout.rs:84 | 1.0 | adapter | field-owner | word | word-field | adapter | LayIbusEngine | RU or ASCII token with context | admitted field readout | src/bin/lay_ibus_engine/preedit_readout.rs:84 | production |
| t8 | ascii_candidates | calls | TypingCpu live_completion_candidates | src/bin/lay_ibus_engine/preedit_readout.rs:10 | 1.0 | adapter | field-owner | word | duplicate-word-field | adapter | PreeditFastState | ASCII token without context | admitted field readout | src/bin/lay_ibus_engine/preedit_readout.rs:10 | production |
| t9 | TypingCpu live_completion_candidates | delegates_to | candidate_gate live_completion_candidates | src/typing_cpu/runtime.rs:39 | 1.0 | facade | field-owner | word | word-field | application | TypingCpu | LiveCompletionRequest | LiveCompletionCandidate | src/typing_cpu/runtime.rs:39 | production |
| t10 | candidate_gate live_completion_candidates | collects | L2 lexical canonical layout boundary and L3 context material | src/nanda_wave/candidate_gate.rs:126 | 1.0 | field-owner | evidence-producers | word | word-field | domain | candidate_gate | normalized token and context | typed field material | src/nanda_wave/candidate_gate.rs:126 | production |
| t11 | candidate_gate live_completion_candidates | calls | TransitionDecisionCore select_live_completions | src/nanda_wave/candidate_gate.rs:411 | 1.0 | field-owner | authority-owner | word | word-field | domain | TransitionDecisionCore | typed field material | admitted ordered candidates | src/nanda_wave/candidate_gate.rs:411 | production |
| t12 | precognition_candidates | calls | TransitionDecisionCore select_ime_readout | src/bin/lay_ibus_engine/preedit.rs:475 | 1.0 | orchestrator | display-owner | display | display-readout | application | TransitionDecisionCore | phrase and word proposals | ordered display proposals | src/bin/lay_ibus_engine/preedit.rs:475 | production |
| t13 | refresh_precognition_candidates | renders | suffix or arrow replacement | src/bin/lay_ibus_engine/preedit.rs:362 | 1.0 | display-owner | view | display | display-readout | interface | LayIbusEngine | ordered display proposals | bracket suffix or arrow replacement | src/bin/lay_ibus_engine/preedit.rs:362 | production |
| t14 | Tab | calls | accept_completion | src/bin/lay_ibus_engine/managed.rs:41 | 1.0 | interface | apply-router | apply | explicit-accept | interface | LayIbusEngine | selected visible proposal | planned edit | src/bin/lay_ibus_engine/managed.rs:41 | production |
| t15 | accept_completion | authorizes_with | IME backend verifier | src/bin/lay_ibus_engine/composition_commit.rs:62 | 1.0 | apply-router | verifier | apply | explicit-accept | application | text_edit | selected proposal | AuthorizedEdit or block | src/bin/lay_ibus_engine/composition_commit.rs:62 | production |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group | layer | owner | entrypoint | output | evidence_path | scope |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|-------|-------|------------|--------|---------------|-------|
| c1 | precognition_candidates | calls_once | word_candidate_proposals | proposed rewrite | 0.95 | orchestrator | word-producer | word | word-field | application | LayIbusEngine | active token and context | typed proposals | proposed rewrite | production |
| c2 | word_candidate_proposals | calls_once | TypingCpu live_completion_candidates | proposed rewrite | 0.95 | adapter | field-owner | word | word-field | adapter | LayIbusEngine | one LiveCompletionRequest | admitted field readout | proposed rewrite | production |
| c3 | semantic_phrase_candidates | remains_separate_from | word_candidate_proposals | proposed rewrite | 0.95 | phrase-producer | word-producer | phrase | phrase-forecast | adapter | LayIbusEngine | phrase tail | suffix proposals | proposed rewrite | production |
| c4 | TypingCpu live_completion_candidates | remains_owner_of | L1 L2 L3 L4 word field | proposed rewrite | 0.95 | field-owner | evidence-producers | word | word-field | domain | TypingCpu | one LiveCompletionRequest | admitted ordered candidates | proposed rewrite | production |
| c5 | TransitionDecisionCore select_ime_readout | merges | phrase and word display proposals | proposed rewrite | 0.95 | display-owner | proposals | display | display-readout | application | TransitionDecisionCore | typed proposals | ordered display proposals | proposed rewrite | production |
| c6 | IBus renderer | displays | bracket suffix or arrow replacement | proposed rewrite | 0.95 | view | stable-output | display | display-readout | interface | LayIbusEngine | ordered display proposals | unchanged visual contract | proposed rewrite | production |
| c7 | Tab | calls | accept_completion | proposed rewrite | 0.95 | interface | apply-router | apply | explicit-accept | interface | LayIbusEngine | selected visible proposal | planned edit | proposed rewrite | production |
| c8 | accept_completion | authorizes_with | IME backend verifier | proposed rewrite | 0.95 | apply-router | verifier | apply | explicit-accept | application | text_edit | selected proposal | AuthorizedEdit or block | proposed rewrite | production |

## notes

- The current duplicate is structural: t7 and t8 call the same field owner for the same ASCII token with different context and limits.
- The rewrite removes t6 and t8. It does not merge phrase forecasting into the word field.
- Candidate birth, display admission, and text mutation remain three different authorities.
- A failed proof or build forbids installation. The global ibus-daemon must not be restarted.
