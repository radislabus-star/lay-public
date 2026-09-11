# Poor-input authority repair, 2026-09-11

This note owns the accepted and installed poor-input mechanics repair for candidate
`aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99`. It records what changed, what was measured, what was installed, and what remains outside scope. Runtime authority changed only through the verified installation receipt below; physical keyboard acceptance and general restoration quality promotion remain open.

## Source of truth

Canonical checkout: `/home/ubu/projects/lay-cleanup-20260908` at source head `da638a1380f1822d3bbca5a045b174dd59601915` for the accepted source packet. Full acceptance identity records `699` source files and all local source bytes match those hashes.

Candidate IME/release binary SHA: `aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99`.

Primary private root: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif`.

Final document-graph verification is recorded at `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/final-graph/fetch-receipt.json`; only an existing `PASS` receipt there establishes graph completion.

## Final mechanism result

The accepted repair is the conjunction of three bounded mechanisms.

First, L2 phase competition now preserves the actual signed phase margin. Close positive evidence no longer becomes full repulsion against the local repeated-letter repair. The semantic controls keep the target and rival in the lattice, preserve verifier/projection effects, and prove that explicit contradictory phase evidence still reduces local authority when it is independently measured.

Second, generic L4 phase-bank availability is advisory. Real structural usage can still populate `l4_phase_witness_*` diagnostics, but availability without exact transition attract/repel evidence must not create a `phase_relation` hidden-state certificate, authoritative ambiguity, or selected rival witnessed class. Exact state-specific L4 transition evidence still resolves hidden state and rejects the rival.

Third, within-word Backspace retention and bounded pending-learning feedback now track the Lay target rather than the raw visible suffix alone. Plain Backspace records one bounded deletion against the pending target, modified Backspace/Delete clear pending learning, overflow clears feedback, and unknown typed feedback clears pending learning. This proves buffer/learning target behavior, not hardware origin or a persisted visible-text sink.

## Acceptance evidence before installation

Source proof is composed, not installed authority. `combined-green-composed-v2.json` records `PASS_COMPOSED_SAME_PRODUCTION_SOURCE`, `2052/2052` unique tests, GREEN4 library `1792`, GREEN3 daemon `260`, declared additions `24`, and only a two-line `#[cfg(test)]` Rust delta between those runs: removal of unused `PhaseInterferenceProbe.rank_after` and its initializer. Current production source is identical between the composed proof parts. The first mandatory full attempt stopped at lint because a test-only `PhaseInterferenceProbe.rank_after` field was written but never read; the fix removed only that field and initializer, preserved the actual `evaluations[..].signals.rank_milli` assertions and transition receipts, and did not add a lint-baseline allowance.

Full acceptance attempt 2 on remote `run-JiMwYK` passed. Full identity `poor-input-full-acceptance-completed-identity.json` has status `PASS`, elapsed `1069.518s`, `699` source files matching `file_sha256`, and candidate SHA `aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99`. Both canonical suites selected `2756` tests and passed with zero known semantic failures and zero infrastructure failures: `changed-SUMMARY.json` sha256 `b033b2cda200015b7d6e63e1f7766b5340c3e113e4c2a341793d5d6e53f354ab`; `full-SUMMARY.json` sha256 `b6262b43c8b1a6cabce294f044c04d677336dd143b277efdde914d7f4baec239`.

The final IME release binary is the same bytes as the combined replay candidate: `aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99`. `same-binary-proof-reuse.json` records `PASS_SAME_BINARY_PROOF_REUSE` and binds the full identity, combined build identity, fixed89 comparison, and diagnostic-combined receipt. This permits reuse of actual combined replay receipts without renaming directories or fabricating `fixed-final` outputs.

## Fixed89 no-regression evidence

Fixed89 regression comparison is exact-output parity across the accepted combined candidate. All three safety profiles have zero changed cases, zero regressed classes, and zero new false applies:

| profile | cases | baseline correct | final correct | wrong output | not restored | result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| strict | 89 | 56 / 89 = 62.921% | 56 / 89 = 62.921% | 1 | 32 | exact unchanged |
| normal | 89 | 60 / 89 = 67.416% | 60 / 89 = 67.416% | 4 | 25 | exact unchanged |
| experimental | 89 | 60 / 89 = 67.416% | 60 / 89 = 67.416% | 5 | 24 | exact unchanged |

Across three profiles that is `267/267` exact same outputs/statuses. This is a no-regression proof, not a statement that all 267 outputs are correct.

Compact fixed89 per-error-class denominators, baseline equals final in every row:

| profile | class/group | correct | wrong | not restored | correct % |
| --- | --- | ---: | ---: | ---: | ---: |
| strict | dirty | 15/47 | 0 | 32 | 31.915% |
| strict | missing_letter | 3/8 | 0 | 5 | 37.500% |
| strict | repeated_letter | 0/4 | 0 | 4 | 0.000% |
| strict | restoration_regressions | 7/24 | 0 | 17 | 29.167% |
| strict | transposition | 5/5 | 0 | 0 | 100.000% |
| strict | context_fixture | 0/6 | 0 | 6 | 0.000% |
| strict | clean | 41/42 | 1 | 0 | 97.619% |
| normal | dirty | 19/47 | 3 | 25 | 40.426% |
| normal | missing_letter | 5/8 | 0 | 3 | 62.500% |
| normal | repeated_letter | 0/4 | 1 | 3 | 0.000% |
| normal | restoration_regressions | 9/24 | 2 | 13 | 37.500% |
| normal | transposition | 5/5 | 0 | 0 | 100.000% |
| normal | context_fixture | 0/6 | 0 | 6 | 0.000% |
| normal | clean | 41/42 | 1 | 0 | 97.619% |
| experimental | dirty | 20/47 | 3 | 24 | 42.553% |
| experimental | missing_letter | 5/8 | 0 | 3 | 62.500% |
| experimental | repeated_letter | 0/4 | 1 | 3 | 0.000% |
| experimental | restoration_regressions | 10/24 | 2 | 12 | 41.667% |
| experimental | transposition | 5/5 | 0 | 0 | 100.000% |
| experimental | context_fixture | 0/6 | 0 | 6 | 0.000% |
| experimental | clean | 40/42 | 2 | 0 | 95.238% |

## Diagnostic18 and native client evidence

Diagnostic18 improved from `10/18` to `14/18` correct. Dirty restored improved from `3/11` to `7/11`; clean controls remained `7/7` preserved.

| group | baseline correct | final correct | baseline wrong | final wrong | baseline not restored | final not restored |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| all | 10/18 = 55.556% | 14/18 = 77.778% | 3/18 = 16.667% | 1/18 = 5.556% | 5/18 = 27.778% | 3/18 = 16.667% |
| clean | 7/7 = 100.000% | 7/7 = 100.000% | 0 | 0 | 0 | 0 |
| dirty | 3/11 = 27.273% | 7/11 = 63.636% | 3/11 = 27.273% | 1/11 = 9.091% | 5/11 = 45.455% | 3/11 = 27.273% |
| poor-input-clean-control | 6/6 = 100.000% | 6/6 = 100.000% | 0 | 0 | 0 | 0 |
| poor-input-punctuation-control | 1/1 = 100.000% | 1/1 = 100.000% | 0 | 0 | 0 | 0 |
| poor-input-repeated-vs-composite | 1/4 = 25.000% | 4/4 = 100.000% | 2/4 = 50.000% | 0 | 1/4 = 25.000% | 0 |
| poor-input-short-layout-lexical | 1/3 = 33.333% | 2/3 = 66.667% | 1/3 = 33.333% | 1/3 = 33.333% | 1/3 = 33.333% | 0 |
| poor-input-substitution | 1/1 = 100.000% | 1/1 = 100.000% | 0 | 0 | 0 | 0 |
| poor-input-ending | 0/2 = 0.000% | 0/2 = 0.000% | 0 | 0 | 2/2 = 100.000% | 2/2 = 100.000% |
| poor-input-ending-punctuation | 0/1 = 0.000% | 0/1 = 0.000% | 0 | 0 | 1/1 = 100.000% | 1/1 = 100.000% |

The exact `скоолько` local/full cases are restored. The diagnostic case `poor_repeat_independent_vвоод_hypothesis` is newly restored; its observed text is `мой ввоод ` with Cyrillic `ввоод`, while the case id preserves the mixed-script `vвоод` spelling. Standalone `nfr` is also newly restored. `nfr b` still goes through the remaining wrong route, and three endings cases remain not restored.

Native client acceptance passed `17/17` in separate lanes: terminal delivery `4`, restoration `5`, lifecycle `3`, manual toggle `3`, first-word US `1`, first-word RU `1`. Receipt and run-metadata startup schedules both match the lane schedules. Manual-toggle and first-word controls are native harness evidence, not automatic physical-keyboard acceptance.

## Installation and runtime identity

Installation receipt: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/installation-poor-input.json`, sha256 `706022c30e9f9bd3423db36340be6dcfe17ffdb41f1a4f87241ae88ae6af4551`, status `INSTALLED_VERIFIED_PHYSICAL_PENDING`, `runtime_authority_changed=true`, completed `2026-09-11T01:31:13Z` after `started_at=2026-09-11T01:31:08.239707Z`.

Installed process identities after receipt:

| process | PID | SHA |
| --- | ---: | --- |
| global `ibus-daemon` | 4715 | preserved |
| `lay-ibus-engine` | 1652608 | `aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99` |
| `lay-daemon` | 1652602 | `be231d78fd01729dd8671409081fe3df20e75e22404d6b632bf2abe6753de01a` |
| `lay-nanda-wave-train` / L3 online | 1652603 | `d2df74855c83e6132fe267147441aaa6bc284fed088b6266304afa51f0ab2742` |
| L1.1 service | 2729596 | preserved |

The installer preserved source, config, extension, input sources, immutable dependencies, global IBus, and the L1.1 service. Backup: `/home/ubu/.local/state/lay/release-backups/poor-input-20260911-bgoa6qf_`. Installation review: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/installation-review.json`, sha256 `7624931c4c997771b724e2c0211c2635078be70e7c833857889bc88a3268c08b`, score `9/10`, H0/M0/L0.

Post-install runtime receipt: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/post-install-runtime.json`, sha256 `858931b908302163a85a92372ad3fb70b96df821870941383e2d412b87be55bb`, status `PASS_INSTALLED_RUNTIME_IDENTITY_AND_L3_WARMUP`. It verifies all five process identities, active services, IME memory warm state, `refreshfail0`, and actual V90 `p2m/p2r` mapping.

Physical keyboard acceptance is `NOT_TESTED`.

## Historical experiments and consequence record

The current installed repair was preceded by bounded experiments that did not change runtime authority. They remain part of the owning evidence trail, but they are not final acceptance by themselves.

Initial frozen-log diagnosis used the installed clean-surface candidate `e5720ec292620c0554696834d2fa4381775e1c5eed013c8c37cfbe24fff4ffbd`. The frozen snapshot was captured at `2026-09-10T22:55:21Z` from IME PID `795644`, daemon PID `795638`, and global IBus PID `4715`. The originally verified L1.1 result for `скоолько` was ABSTAIN outside the calibrated basin with the `сколько` seed retained; the first wrong rank was at L2, and phase-only repair later exposed the L4 generic certificate block. This was not a grounded L1.1 Winner downgrade. Compact score facts showed the correct target with phase competition `-1000`, rank `964/965`, field `0`, and the wrong rival with phase competition `1000`, rank `1134/1136`, field `280`; L4 scene evidence was downstream of the already-mutated rank, not the first cause.

Remote diagnostic baseline receipt `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/diagnostic-baseline-0/receipt.json` executed under the real guard with `runtime_authority_changed=false`, status `COMPLETED_TD123_DIAGNOSTIC`, elapsed `21.29865167895332s`. It measured `18` diagnostic cases: restored `3`, clean preserved `7`, wrong output `3`, not restored `5`. Persisted L3 topology asserted the frozen `compact-base-a + []`; the live observer's `compact-base-b` plus `delta_count=1` was retained only as semantic-count evidence, not frozen input authority.

Fixed89 baseline summary `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/fixed-baseline-summary.json`, sha256 `75e428039f052c0048e25eda3530b5cfb0d6d7c036594eb9c86c5364f8754d1b`, covered the same frozen input and three safety profiles without promotion. Baseline facts were: strict `56/89` non-failing, `15` restored, `41/42` clean preserved, `1` wrong, `32` not restored; normal `60/89`, `19` restored, `41/42` clean preserved, `4` wrong, `25` not restored; experimental `60/89`, `20` restored, `40/42` clean preserved, `5` wrong, `24` not restored.

The first accepted causal hypothesis was the L2 phase amplifier. RED receipts `/home/ubu/.cache/lay/development/run-bzcjnx7j/RESULT.json` sha256 `ab0376011a05cd1f4f55b5462c96559695ac23b55e66af1620037b8b6a03687c` and `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/phase-red-tests/SUMMARY.json` sha256 `b8191eb24a90448005878fcc2eba91f73673063e0569049986f479c29c8aaa72` compiled unchanged production source on remote `run-wnhUV2`; `fmt` passed, `1788` focused `lib:lay` tests ran, and only the new close-positive TD123 mechanism test failed. The RED log shape was synthetic margins `50000/51000`, forced non-field rank `800/790`, after-settlement rank `800/1690`, phase competition `-1000/+1000`, field `0/900`, and verifier checks true. The selected alternative was absolute calibrated lexical evidence over the previous extreme min/max scaling: preserve measured evidence, avoid manufacturing full authority from narrow candidate lattices, keep explicit contradictory evidence competing, and do not add word/source-specific conditions or verifier/SafetyGate bypasses.

The Backspace/`можешь` issue was a separate mechanism. RED evidence came from `/home/ubu/.cache/lay/development/run-v_j0krkx/RESULT.json`, remote `run-TycOOk`, and `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/backspace-red-tests/SUMMARY.json`: `2034` tests executed, `2032` passed, and the two failures were the expected within-known-word current-tail loss and interleaved learning deletion ownership. Consequence analysis kept `WordBuffer` as the raw typed owner and limited the repair to same-owner plain-Backspace reduction plus bounded pending-learning deletion accounting. It explicitly excluded new raw owners, bridge RPC, second event loop, timers, queues, persisted cache/schema, learning package writes, and runtime authority routes.

A phase-only prototype was measured but not accepted as final. GREEN receipt `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/phase-green-tests/SUMMARY.json` reported `1788/1788`; build identity `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/phase-build-identity.json` from remote build `run-j7v8B0` produced candidate `de8d738ad3528db7df2213654f9700c52f21fcbf7c2ad3eb40385ce42f251942`; `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/phase-fixed-comparison.json` verified `267/267` exact same outputs/statuses as baseline across fixed89; and `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/diagnostic-candidate-0/receipt.json` measured Diagnostic18. Diagnostic18 improved only `10/18 -> 12/18`: both `скоолько` rows moved from wrong output to not restored, `мой ввоод ` and standalone `nfr` restored, and `7` clean controls stayed preserved. The remaining visible no-apply was caused by a generic L4 `phase_relation` certificate selecting the rival witnessed class, which led to the accepted L4 producer-boundary controls. L4 producer RED receipt `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/l4-producer-red-tests/SUMMARY.json` ran before the accepted producer repair with `1792` executed, `1789` passed, `3` expected new failures, and `runtime_authority_changed=false`. Those controls kept phase-bank availability as diagnostics unless exact transition evidence exists, while preserving exact state-specific L4 transition authority.

## Source-contract and reducer controls

The old statement that source-string checks were removed is stale. The existing busy-key-queue source contract remains, but it was rebound to the current event-loop text after the let-hoist changed the surrounding source shape. That rebind is `#[cfg(test)]` only and production bytes before the test module are identical. Reducer/runtime controls are semantic: unknown mappings, modified Backspace, manual stale state, ignored Ctrl/Alt/Meta typing commands, Shift+Insert, pending-ready pre-poll cancellation, and Backspace learning behavior assert observable state and safety effects rather than relying on old source-name RED strings.

## Receipt bundle

- acceptance summary: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/acceptance-summary.json`, sha256 `a87650df7e27964baff8474f32ed64dd94f126cdc83d06ed6bbbc1821cb9d86f`.
- full identity: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/full-acceptance/poor-input-full-acceptance-completed-identity.json`, sha256 `2ab8968f570edf46977111b61f805a8bc328f6c5ca27746924f8ac36c95d37fd`, elapsed `1069.518s`.
- same-binary reuse proof: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/same-binary-proof-reuse.json`, sha256 `953807831f71033b79b8332e4cd7e7108f2faecb2e1e50b41bb18731b22e0f4f`.
- fixed89 combined comparison: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/fixed89-combined-comparison.json`, sha256 `a2faee2b5deb1c073690f64c7e81a4350484ab23c24db2c0c0f4b6322368600e`.
- diagnostic baseline receipt: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/diagnostic-baseline-0/receipt.json`, sha256 `e3c7a07596c75bc5087544064e2e392b9d0373b7ce8028820e4302dfe82a1079`.
- diagnostic final receipt: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/diagnostic-combined-0/receipt.json`, sha256 `cc43ab70abe9aba63192e1d1bb6e0903a776715c75476b96a44d0ab06e0bf28d`.
- native summary: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/native-controls/SUMMARY.json`, sha256 `f0c99716ae8595f991d7e999e3e460b60c0d728e286280267e5452dfa30c862c`.
- composed source proof: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/combined-green-composed-v2.json`, sha256 `d7da731288f461c27ce801ab5173a69ddd55baa76220f4f708a66574e2ce1fa3`.
- changed suite summary: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/full-acceptance/changed-SUMMARY.json`, sha256 `b033b2cda200015b7d6e63e1f7766b5340c3e113e4c2a341793d5d6e53f354ab`.
- full suite summary: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/full-acceptance/full-SUMMARY.json`, sha256 `b6262b43c8b1a6cabce294f044c04d677336dd143b277efdde914d7f4baec239`.
- installation receipt: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/installation-poor-input.json`, sha256 `706022c30e9f9bd3423db36340be6dcfe17ffdb41f1a4f87241ae88ae6af4551`.
- post-install runtime receipt: `/home/ubu/.cache/lay/development/poor-input-20260911-xoi17mif/post-install-runtime.json`, sha256 `858931b908302163a85a92372ad3fb70b96df821870941383e2d412b87be55bb`.

## Scope limits

This is a bounded poor-input mechanics repair. It does not prove universal grammar correction, universal mixed-layout correction, general heldout improvement, latency improvement, RSS budget improvement, or physical keyboard acceptance. Quality for surfaces such as `видешь`, `выровнить`, mixed `nfr` contexts, and Russian endings remains only as covered by the listed diagnostic/fixed/native denominators. General quality promotion remains open.
