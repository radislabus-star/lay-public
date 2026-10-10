# TD-133 client-ordering preparation — independent review pass 1

Reviewed at 2026-10-09 21:46:39 UTC (2026-10-10 locally). **Overall 7/10: preparation needs three focused repairs before execution review. Safety 8/10; causal value 6/10 in the submitted form.** The bounded widget-observation route is appropriate; it does not establish the cause or authorize execution. Fresh explicit local authorization remains required.

This was a static, read-only review. No diagnostic/helper was run or imported; no GUI, physical input, Cargo, install, restart, model operation or RDP was used. Only this report was written. Toolkit bindings, graceful GUI teardown in the actual environment, native event coverage, and the causal result are **NOT_TESTED / UNKNOWN**.

## Exact review binding

Checkout `/home/ubu/projects/lay-space-boundary-shift-20261007`, HEAD `24bcea5256661954d5aba97ff037d6a419e1e2cb`, initially clean. Root subsequently added preparation documentation; this review makes no production change.

| File | Verified SHA-256 |
| --- | --- |
| `td133-client-ordering-diagnostic-20261010.py` | `af54279948826afc8bb558e11ee982ed3c76e11c822ae07760bb62d36a5059f6` |
| `td133-client-event-fixture-20261010.py` | `859b69aaa670166a24deb99cc03f4d5163a7e335dbdc010c7197c9105a1c410d` |
| Immediate parent `td133-original-activation-diagnostic-20261009.py` | `ff3d3a19fb62d193e51ea301a00c99d4e0cdc01214011a80c327dd6d59f25f7b` |
| Original `windows/native_field_fixture.py` | `60894fae2853530851cd66e7ed50f382cdad01ca1abbfcc4883613be0fbb577d` |
| Inherited `windows/window_matrix.py` | `970fd4218860bf6b6478f39311d0127a3a882848fd9bc93d79935abbba89d531` |

The first three files are in `/home/ubu/.cache/lay/development/`; the original fixture/helper are in its `space-boundary-client-commit-physical-20261008/windows/` directory. Line references below address these reviewed bytes.

Read `AGENTS.md`, `DEVELOPMENT.md` (remote-only rule and native/private/diagnostic clauses), `ARCHITECTURE.md` C03/C05/C07/C08/C10, the TD-133 owner and its new preparation entry, and the preparation packet. Used the existing graph for owner navigation. The existing reducer/WindowInteraction/output remain the owners; no new runtime route is proposed.

## Blocking repairs

1. **Observer failures can change the observed operation.** Fixture lines 149–175 and 242–265 construct toolkit-derived payloads before entering `StateWriter.event`'s exception guard (45–57). An exception in a getter/conversion can therefore skip GTK's original preedit bookkeeping/publication or Qt's `super().keyPressEvent`, `super().keyReleaseEvent`, or `super().inputMethodEvent`. In PySide an exception in an override can suppress the native default operation; the logger could manufacture the apparent ordering failure. **Minimum fix:** evaluate the complete observation payload inside the bounded logger guard, for example through a lazy callback. Preserve original/default processing exactly once and independently of observation success. Record observer errors; do not swallow exceptions from the original/default operation as logger errors. Toolkit behavior remains untested after a static repair.

2. **GTK press and release are indistinguishable.** Fixture lines 168–179 connect both signals to the same callback and store `event="native-key"` without `event.type` or a direction. Keyval/keycode/state/time do not independently identify which record is the press that must join the engine callback. **Minimum fix:** add explicit press/release direction or distinct event names. Keep the callback's ordinary `False` return. Do not infer direction solely from alternating records. The `changed` observation (174–180) is after that signal's default handler; it is not proof that the enclosing key/IME operation has completed.

3. **The boundary oracle permits an extra middle space.** Diagnostic lines 173–176 accept `должен  ыбыть `: `startswith`, `split()` and the trailing-space checks all succeed. This falsely accepts a boundary placement defect. **Minimum fix:** require exactly the prefix `должен `, exactly one final ASCII space, and a nonempty right-token substring containing no whitespace; retain empty-preedit and final-caret checks. Keep ordinary right-token correction eligible rather than requiring the literal `ыбыть`. This is a stricter oracle for this distinct diagnostic, not the immutable original-matrix oracle; record the difference explicitly. It proves boundary/committed-text behavior only, not the semantic correctness of every possible right-token correction or rendered/pixel acceptance.

## Teardown and partial-failure evidence

The inherited safety path is retained: focus/title/PID are checked before presses; held keys are tracked under signal masking; constructor cancellation is deferred until the child handle exists. Cleanup independently attempts this device's key releases, device closure, partial engine capture, owned child closure, logging restoration and runtime/mode receipts (diagnostic 298–332). It does not terminate unrelated windows or restart IBus. Config restoration is explicitly a best-effort byte check, not atomic CAS; this known inherited limitation is not a new observer change.

The child's new SIGTERM handlers schedule ordinary GUI close (fixture 197 / 300). On a normal return, `main`'s `finally` flushes events (328–331). The inherited `Field.close` terminates only its child, waits three seconds, then kills it (helper 91–95). A SIGTERM during early startup before handler registration, a hung event loop, forced kill, or flush failure can leave no or incomplete client event file. These cases must remain UNKNOWN/BLOCKED; the RAM-only logger cannot promise crash preservation. This static analysis does not prove GTK/Qt signal integration.

**Nonblocking receipt improvement worth including in the consolidated repair:** client events are parsed/linked only on the successful close path (diagnostic 282–290). An earlier opener/setup/focus/cancellation failure gets a partial engine capture and child-close attempt, but no client-artifact metadata in `RESULT.json`. After the final child-close attempt, add an independent best-effort receipt for path, PID/starttick, last started phase, return code, and file hash/count/drop/error fields, or explicit missing/truncated UNKNOWN. Record a phase as started before typing so partially emitted streams are distinguishable from the four-stream plan. Receipt failure must not interrupt config/runtime cleanup. Existing raw artifacts should remain private and untouched.

## Bounds, scope and causal limits

The new fixture caps events at 4096 and observer error names at eight, with no new event-file I/O in widget callbacks; flush is at exit. The pre-existing 40ms state publisher and maximum five-minute child lifetime remain. Event count is bounded, though snapshot bytes are not separately capped. For the fixed short owned-field input this is reasonable; it is not a general arbitrary-text recorder budget. New text/caret/focus getter calls and dictionary allocations can perturb timing even without per-event disk I/O. The configured 18/12ms sleeps are retained; actual key/callback gaps still require measurement.

The parent hard-codes exactly two fresh owned fields, GTK3 Entry and Qt rich, each with one `должен ` opener and one `должен ыбыть ` target after raw clear and ordinary RU activation: exactly four declared word streams, no retry. The existing initial raw-clear/activation setup is also retained. Pinned loaded/installed IME bytes, coherent RU checks, private output directory, original trace anchoring and no-key stop condition remain. No inverse, shared-learning cleanup, installation/restart, model copy/fit or RDP is added. Ordinary runtime correction/feedback remains ordinary runtime behavior; this is not a new isolated learning proof.

The widget PID witnesses the owned client event log, but does not itself bind each engine InputContext/callback to that PID. Engine serials, clock domains, physical direction, stream intervals and widget effects still need an explicit event join. Empty text and coherent RU do not prove pre-key admission Ready. Qt before/after `super` observations can show an input-method commit's effect; GTK `changed`/preedit/key signals offer a different, narrower boundary.

A bound failure can identify a first observed divergence and inform a controlled production-reducer RED. A successful warm contrast cannot close TD-133: full original predecessor/learning state is not recreated, observer timing differs, and the original `62 PASS / 2 FAIL / 64` plus GTK layout conflict remain historical evidence. There is no independent functional verifier result here: native causal result and quality are UNKNOWN.

The documented preparation state is `PREPARED_AWAITING_REVIEW_AND_FRESH_LOCAL_GRANT`; earlier local grants are consumed. After the three repairs, rebind script/fixture hashes and finish pass 2 before requesting the concrete four-stream grant. This report provides no execution authorization and selects no runtime fix.
