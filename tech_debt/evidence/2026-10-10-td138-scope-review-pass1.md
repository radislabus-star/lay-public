# TD-138 benchmark-scope review — pass 1

Review score: **9/10 for the controlled characterization plan**. Resolution verdict: **REPLAN_REQUIRED; fixture-only migration cannot resolve this gate**. This is the first of at most two reviews; the final implementation and complete source proof belong to pass 2.

Read-only review of `/home/ubu/projects/lay-space-boundary-shift-20261007`, branch `codex/space-boundary-shift-20261007`, independently verified HEAD `24bcea5256661954d5aba97ff037d6a419e1e2cb`. No tests, builds, graph commands, CLI probes, network, native input, installation or restarts were performed by this reviewer. Only this report was written.

## Material finding

The actual controlled contrast arrived during this review and **fails under the production IME policy**. It selects two exact identities: short-prefix PASS, unique-prefix FAIL. The unchanged six-prefix sequence returns 12 candidates for every prefix:

| Prefix | Elapsed us | Candidates |
| --- | ---: | ---: |
| пол | 6812 | 12 |
| цел | 5143 | 12 |
| рас | 6996 | 12 |
| оста | **591846** | 12 |
| дост | 6362 | 12 |
| остан | 4077 | 12 |

The budget remains 50000 us. Gate statistics report zero cache hits and seven misses, comprising existing warmup plus the six unique measurements. This failing result is not a vacuous empty readout. The opt-in trace on the earlier candidate localized 298096 us to lexical material; the new contrast reports L2 material max 591034 us but does not identify its internal expensive subcall.

Therefore the policy mismatch is a true setup difference, but **policy selection alone is not a sufficient explanation or fix for the latency failure**. One run does not establish a statistical performance regression between the policies. TD-138 explicitly specifies the applicable outcome at lines 31–32: if proper IME scope also fails, no benchmark migration; investigate the first expensive lexical subcall. Follow that branch. A fixture-only change scored as a completed performance remedy would be **1/10** on the current evidence.

## Scope and authority

Binding these fixtures to `HotFieldPolicy::ime()` is a legitimate measurement-scope correction in principle, provided it is explicitly described as an IME hot-readout contract and keeps the other authority denominator visible. It is not intrinsically a forbidden candidate exception:

- `src/bin/lay_ibus_engine.rs:79–84` publishes `Ime/FieldSnapshotOnly` before factory/startup work. `HotFieldPolicy::ime()` defines precisely that existing tuple at `src/hot_field.rs:74–78`.
- Current production candidate-gate readout calls reach it from `src/bin/lay_ibus_engine/preedit_readout.rs:116` through `TypingCpu::live_completion_readout_for_scene`; the cached route is also IME preedit. The module itself describes live IME completion admission.
- The daemon does warm the shared L2/candidate path in `src/bin/lay_daemon/startup_runtime/warmup.rs`, but its ongoing correction path measures completed-tail decoding in `typing_assist_runtime/candidate.rs`. No production daemon call of the live completion readout was found. Startup reuse does not make these prefix fixtures proof of daemon/uinput correction latency.
- Daemon/uinput genuinely retains `FullReferenceAllowed`: `daemon_for_text_backend` selects it when `should_try_ime()` is false (`src/hot_field.rs:62–70`, `src/text_backend.rs:79–87`), and daemon config publishes that policy (`config_runtime.rs:72,195–196`). A future IME-bound fixture must not be presented as coverage or PASS for that route.
- Authority changes available reference evidence. For example, the dictionary API returns an empty reference set under field-only policy (`src/russian_lexicon.rs:87–96`). Equality of candidate surfaces across the two authorities is consequently not an automatic semantic requirement. Neither timing result proves heldout quality, restoration retention or per-class quality. Existing reference correctness coverage must remain independently intact.

Thus a source-binding correction could eventually be admitted without weakening the budget, but it cannot be used now to erase the demonstrated production-scope FAIL or close TD-133.

## Test setup and controlled-proof validity

`src/hot_field.rs:35–40,98–119` uses a thread-local override in library tests and process atomics in production. Selecting/asserting the tuple **before existing synchronous warmup** keeps the test's warmup and measured calls on the same test worker. This matches the production policy values; it does not prove production asynchronous startup, native latency, allocator/RSS behavior or physical-client acceptance. A spawned helper without explicit test-policy propagation would not provide the same test scope.

The reviewed worker script imports test discovery/execution from the extracted frozen tree. Their `ROOT` is resolved from those imported source files, and discovery builds `lib:lay` through the existing Cargo guard; it is not merely selecting an old cached executable. Each exact performance identity runs through the existing clean filesystem/network sandbox in its own process. The script preserves all inputs, requests, budgets, original output assertions and synchronous warmup, while adding only policy selection/assertion to the two fixtures. Its source stability check admits only that one changed path. There is no warmup of the failing prefix, candidate removal, altered cache key, threshold change or allowlist exception.

The original full gate and frozen baseline FAIL receipts form the retained negative controls. Independently verified packet hashes match their local receipts. The new IME-scope FAIL is stronger evidence against the proposed sole-cause hypothesis than another hypothetical negative control. No retry to obtain a favorable timing result is warranted.

Both existing fixtures do have a latent non-vacuity limitation: short-prefix `all()` accepts an empty vector, and unique-prefix asserts timing without a nonempty-result guard. This is **not the cause of the observed FAIL**, since all six unique requests returned 12 candidates. If a later admitted change revises these fixtures, the minimum useful hardening is to assert successful/ready synchronous warmup and actual nonempty measured readout, preserving the original timing interval and assertions. No new word-specific runtime branch or exact candidate-count requirement is needed. This hardening is not a prerequisite to record the current rejected hypothesis.

## Minimum next action

1. Leave actual benchmark fixtures and production policy/runtime unchanged. Record the contrast receipt, exact FAIL and rejection of the policy-only remedy in the owning TD-138 document/decision. Keep original full gate FAIL and frozen baseline FAIL immutable; TD-133 remains performance-blocked.
2. Explicitly replan the scope before expanding a fixture-only task into runtime optimization. Localize the first expensive lexical/L2 subcall using the existing opt-in trace and unchanged proof inputs on the guarded worker. Do not select a cache, warming strategy, corpus trimming or candidate-generation change before that evidence.
3. If a systemic source correction becomes justified, root writes the required consequence analysis and decision, implements the smallest existing-owner correction, and executes the unchanged full source/performance gate. Any authority/ranking/retention change additionally requires its owning semantic/per-class proof; timing is not a substitute. Pass 2 reviews the final code and exact proof. Native/release admission remains separate.

No material flaw was found in the bounded diagnostic itself. The blocker is its measured result, not a missing speculative refactor. Correct handling is to reject closure and pursue the unresolved mechanism.

## Evidence binding

- New contrast receipt: `/home/ubu/.cache/lay/development/tab-full-04obkqdi/TD138_AUTHORITY_CONTRAST.json`, SHA-256 `401bd73114171df6a73e9dc87af6b4caf0fa53d14e36dda4de8fe1a852c502ff`.
- Reviewed worker script: `/home/ubu/.cache/lay/development/td138-ime-authority-contrast-20261010.py`, SHA-256 `7c6048467583bce12333793bd7527d1e31d5d4d2583e7f8abc0687f0fa45aef4`.
- Frozen baseline archive SHA-256: `51bd32fecb0d135840d904cebd2307555f254c78b13ef7a56d2ca507fa29a105`.
- Baseline candidate-gate source SHA-256: `3c76238a0f1fe1dca374b48e96968cf5bc67f6710c5b052cc357d9cc4c8ed44c`; controlled source SHA-256: `c46851d4993aaef68965ea2ebd2f8524410b5dd955cfdeccc0df467226c0d29b`.
- Contrast unique-prefix log SHA-256: `405e2cbcd6a0829b6261b1273327af5b3508e4b9f5c79ea8d76e60aa234a25f0`; short-prefix log SHA-256: `a260982cbd28ef5255d7e4e31c2ac642425f94d6da05159d8609d830cb8d547c`.
- Original proof packet: `tech_debt/evidence/2026-10-10-td133-managed-space-source-proof.json`. Independently checked local receipt hashes match its `second_full_gate_functional_green_performance_fail`, `frozen_performance_baseline` and `frozen_performance_candidate-trace` entries.
- Independently compared the current `hot_field.rs`, IME main, candidate gate, NANDA warmup module and daemon config policy source with the frozen baseline archive: all five byte-identical at review time. Graph source binding was read only for navigation; no graph output was treated as behavior proof.

Verdict scope: characterization reviewed; candidate policy-only remediation **rejected**; full source gate **FAIL**; installed/native behavior **NOT TESTED**; runtime authority unchanged by this diagnostic and by this review.
