# TD-133 client-ordering preparation — final review pass 2/2

Reviewed at 2026-10-09 21:50:28 UTC (2026-10-10 locally). **9/10 — STATIC_PREPARATION_ACCEPTED_PENDING_FRESH_LOCAL_GRANT. No remaining preparation blocker found.** All three pass-1 blockers are closed, and the recommended partial-evidence improvement is included. This verdict accepts the bounded preparation, not execution, toolkit compatibility, a causal result, or a runtime repair.

Static review only: no diagnostic/helper import or execution, GUI, physical input, tests, Cargo, install, restart, model operation, or RDP. Only this report was written. Native result, actual GTK/PySide bindings and event coverage, graceful toolkit teardown, pre-key Ready and the first original failed transition remain **NOT_TESTED / UNKNOWN**.

## Exact binding

Checkout `/home/ubu/projects/lay-space-boundary-shift-20261007`, HEAD `24bcea5256661954d5aba97ff037d6a419e1e2cb`. Current project changes are preparation/owner documentation and the public pass-1 report; no production input code change is present.

| Reviewed private file | Verified SHA-256 |
| --- | --- |
| `/home/ubu/.cache/lay/development/td133-client-ordering-diagnostic-20261010.py` | `3d45c368b075dd3b5fffaf040fa66ea655ac32f48b5406fb49806256b5d70cba` |
| `/home/ubu/.cache/lay/development/td133-client-event-fixture-20261010.py` | `e7cb7709323c49b94e59090a38dc1d7bbf9ce51af309c0b642811b7e572935c1` |

The preparation packet binds these hashes and the same HEAD. Private and public pass-1 reports both hash to `e0dfacfed3cd62af104e7654bd84f38dcfcce6dbbc61d29c3f213a64b88133db`. Compared the repaired sources against the pinned immediate parent and original fixture; reused the previously read owner/canon/testing requirements and checked the updated preparation/owning-document entries.

## Closed findings

- **Payload guard:** fixture 46–58 evaluates each callable payload inside the bounded exception guard and avoids payload evaluation after the event cap. Every new event call passes a callable. GTK keeps its original preedit assignment/publication (155–159) and returns `False` from native-key observation (174–179). Qt calls each relevant `super` handler exactly once, independently of successful observation (252–275); default-handler exceptions are not swallowed by the logger.
- **GTK direction:** `event_type=int(event.type)` is recorded (175–178) for both native press/release signals. This provides direction without assuming alternating records. A conversion error is captured as an observer error and prevents a complete-trace receipt. GTK `changed/after-default` still denotes its signal boundary, not completion of the enclosing key/IME operation.
- **Strict boundary oracle:** diagnostic 173–177 requires exact `должен `, a nonempty right token with no whitespace, one final ASCII space and empty preedit. Final end-caret validation remains (302–303). Ordinary right-token correction remains eligible. Double middle/trailing spaces and both original shifted-boundary examples cannot pass this predicate. The packet explicitly records the stricter middle-boundary condition as a difference from the immutable historical oracle; original receipts/denominators are preserved.

## Partial receipts and teardown

`begin_stream` records PID/starttick, phase, wall/monotonic start time and STARTED before typing (223–228, 267–272, 289–306). The four-entry plan is distinct from actual attempts or successful rows; an interrupted stream can remain STARTED with a blocked top-level result.

`client_receipt` (201–221) records the path, PID/starttick, child return code, available payload hash and event/drop/error counts. Missing, malformed and incomplete artifacts retain explicit UNKNOWN states. A captured receipt requires matching schema/PID, nonzero events, no drops/errors and child return code zero. The normal path checks this before moving to another field (317–323). The failure path attempts the receipt after owned child close and retains the last field observation (339–344), through independent cleanup steps. Errors in those attempts do not skip logging restoration or runtime/mode receipts (345–364). An early constructor failure may have no starttick or client file; the receipt makes that absence visible rather than certifying it.

The inherited held-key release, uinput closure, owned child termination with a three-second wait/kill fallback, and best-effort unchanged-byte config restoration remain. New SIGTERM handlers schedule ordinary GTK/Qt close (fixture 204 / 310); graceful return reaches the event flush in `finally` (340–343). Early termination, toolkit hangs, forced kill or flush failure can still lose the RAM capture, and must remain UNKNOWN/BLOCKED. This is an explicit limitation, not a successful native teardown measurement.

## Scope and remaining proof boundary

The fixture rejects unrelated toolkit/field combinations before construction (335–336), and the parent requires the distinct `--client-event-ordering` declaration alongside `--allow-local-physical` (60–63). These flags do not constitute user authorization. Exactly two fresh owned fields and four declared word streams remain; no retries or extra Ready wait are added. Original tap sleeps and assertion deadlines are preserved. New pre-input metadata records actual GTK/display type or Qt/PySide/platform plus only the named IM-module environment variable (fixture 83–86 / 236–237); it does not infer the active IME transport.

Capture is capped at 4096 events/eight observer error names. There is no new per-event file write, render read, product timer, second runtime owner, inverse, shared-learning cleanup, model copy/fit, install/restart or RDP. Existing state publication and short-scope snapshot-size limitations remain. Instrumentation can perturb actual timing; configured sleeps are not measured callback gaps.

Engine serials/clock domains, native direction, widget effects and stream intervals must still be joined. A child PID and platform metadata do not independently bind an engine InputContext to that field or prove admission Ready. Committed-text/preedit/caret PASS is separate from rendered acceptance. A positive warm contrast cannot reproduce the full original predecessor/learning state or close TD-133; original `62 PASS / 2 FAIL / 64` and the GTK layout conflict remain intact.

Preparation review is complete at pass 2/2 with no requested further repair. The next permissible step is presenting this exact four-stream preparation for a fresh explicit local execution grant. The packet currently has `execution_granted=false`, `execution_completed=false`, and `runtime_authority_changed=false`; this review changes none of those states.
