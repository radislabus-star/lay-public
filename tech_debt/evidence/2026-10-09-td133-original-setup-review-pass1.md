# TD-133 original activation setup diagnostic — independent review, pass 1/2

**Score: 9/10. ACCEPT_PREPARATION for the declared narrow setup probe. Execution remains NOT_AUTHORIZED; runtime behavior and causal correctness remain NOT_TESTED/UNKNOWN. No required preparation changes.**

Reviewed checkout HEAD: `60e0d095f4468d744762831d56336fe428583fb4`. Read AGENTS.md, ARCHITECTURE.md, DEVELOPMENT.md and the TD-133 owning card/diagnostic summary. A bounded read-only graph query supplied navigation only; its seven nodes did not establish this cache probe's behavior.

## Exact identities and scope

- Candidate: `/home/ubu/.cache/lay/development/td133-original-activation-diagnostic-20261009.py`, SHA-256 `ff3d3a19fb62d193e51ea301a00c99d4e0cdc01214011a80c327dd6d59f25f7b`.
- Unchanged parent: `/home/ubu/.cache/lay/development/td133-native-diagnostic-20261009.py`, SHA-256 `b023e68924a20ba8caf19b619bf9a44d6d8e5bd53af392d04b282592ca69354e`.
- Both historical helper hashes match candidate PINS: window matrix `970fd4218860bf6b6478f39311d0127a3a882848fd9bc93d79935abbba89d531`; native fixture `60894fae2853530851cd66e7ed50f382cdad01ca1abbfcc4883613be0fbb577d`.

The exact diff adds the second explicit CLI flag, distinct output/provenance metadata, and `Field.select('ru')` plus post-activation mode/empty-field observation. It retains exactly one `должен ыбыть ` stream in each of a new GTK3 Entry and Qt rich field, 18/12 ms key timing and the 1.5 s visible assertion (candidate lines 160–171, 203–258).

The original first-pair wrapper and original matrix source confirm that the historical invalid-left case used the same prefix-free surface and `Field.clear('ru')`. That clear path performs Ctrl+A/Backspace followed by `Field.select`; the latter issues one normal ActivateLayout and then its existing bounded coherence readbacks (window_matrix.py:77–90). This is the selected setup hypothesis, not replay of the original preceding cases or full matrix. Its existing coherence polling is not an added IME admission-readiness wait or another activation attempt.

## Safety and failure paths

The inherited checks require one managed IME with equal installed/loaded accepted `2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559` bytes, bounded human-safe-run scope, unchanged RU tuple and pinned helpers. The candidate imports helper definitions; it does not call their historical main, service restarts, inverse or shared cleanup routes.

Child ownership is retained before construction/focus failure; every key press checks the owned title/PID and current observation. Held keys, deferred cancellation, owned device/child teardown, partial capture, logging restore and final runtime/mode observation retain the parent's failure handling. Setup, focus, mode or trace ambiguity stops further input. A visible FAIL is recorded once; the other declared field may still execute as its separate control. There is no same-field retry, second stream, fallback transport or corrective global mode reconciliation.

Config restore remains explicitly best-effort with unchanged-byte checks, not atomic CAS. Cleanup failures cannot be promoted to success. These inherited limitations are honestly recorded; this review does not certify crash/SIGKILL cleanup.

## Evidence boundaries and execution blocker

`PASS_VISIBLE_CONTROL` proves preserved `должен `, two tokens, one trailing space, empty observed preedit and final caret. It allows ordinary right-token correction, as the original control did; it is not sentence-restoration or product acceptance.

The prior `/td133-native-once-jrdlshgi/RESULT.json` records two visible PASS controls, preserved runtime/mode and no cleanup errors. Its traces both expose the same `InputContext_7` relay. Field PID/focus and engine event timing do not uniquely bind callbacks to that native field or prove Ready before the first key. The new capture still labels causal binding as unresolved (candidate lines 173–186). An empty field/preedit and coherent layout cannot fill that gap.

The original fixed receipt still records **62 PASS / 2 FAIL / 0 BLOCKED out of 64**, including GTK3 `долженыбыть  ` and Qt rich `долженыбы ть `. This prospective diagnostic has its own denominator and cannot overwrite those failures, prove their cause or authorize runtime repair.

**Execution blocker:** the previous one-shot grant was consumed. Obtain a fresh explicit grant for this exact two-stream original-layout setup, then use both `--allow-local-physical` and `--original-layout-setup`; the flags do not themselves establish user authorization. No automatic rerun is authorized by this review. Minimum code changes: none.

No tests/builds, diagnostic imports or executions, native/GUI/input operations, services, graph refreshes, installations, runtime measurements or repository edits were performed. This review file is the only reviewer write. The historical helper was read only and never executed.
