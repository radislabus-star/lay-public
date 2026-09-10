# TD-121 — deployed IBus private transport baseline

Date: 2026-09-05. Source baseline cc1e2207; no Lay production edits in this
experiment. `runtime_authority_changed=false`; production-transfer and
correction-quality verdicts remain `NOT_TESTED`.

## What ran

Exact command: `bash /home/ubu/.cache/lay/td121-real-ibus-imX7Og/launch.sh`.
Installed `/usr/bin/ibus-daemon` was launched only inside separate PID/net/IPC/
mount namespaces, read-only host filesystem, private session bus, private XDG
directories and socket. No Lay binary, keyboard/uinput, desktop bus, global
IBus setting, model, learner or package was touched. The private session bus
alone used AppArmor mode disabled, with global AppArmor unchanged.

The actual IBus daemon routed two inert Python IBus.Engine fixtures, a real
input context, real SetGlobalEngine/GlobalEngineChanged, CurrentInputContext
property and no-destination self-signals. Its global-engine mode returned true.
One continuously drained GDBus input filter recorded receive order. This is
transport evidence, not a zbus observer implementation test.

## Measurements and denominators

- 22/22 cases executed and assertions passed: bootstrap context/self-marker1,
  real foreign-engine ABA followed by marker20, distinct foreign sender1.
- All20 ABA cases observed foreign then original GlobalEngineChanged before
  the self-marker; a distinct connection's marker retained a different sender.
- Get + marker21 observations: min537µs, median745µs, nearest-rank p951061µs,
  max1108µs. Small unloaded private sample, not production p99 or load proof.
- Private driver0.362s; service0.521s, CPU0.401s, memory peak37.6MiB, swap0.
  Limits CPU75%, memory768MiB, swap0, tasks48, runtime45s; driver alarm35s.
- Private IBus PID5 reaped with return−15; containing service exited0.
  Live global IBus4715, Lay daemon3453123, IME3453154, L3 learner3453057,
  L1.1 server3452522 all still existed after cleanup.

## Not tested / remaining promotion gates

No deliberately delayed GlobalEngineChanged overtook its property reply in
this run: early-reply cases0/20. Therefore C09/C10 adversarial ordering requires
the explicit stalled-queue/model test as well; this sample is not that proof.
No sender-header spoof attempt was made (only a genuine second sender).
Native/cached capability convergence, zbus OrderedStream merge, bounded ring,
callback settlement, input completeness, hot-upgrade transfer, three safety
profiles, physical key delivery and client-visible Lay restoration remain open.
Do not label C01–C30 or TD-121 PASS from these22 transport cases.

The result supports the selected same-connection self-marker's feasibility
against deployed IBus and the proposed5ms acquisition target in this sample.
It does not authorize equality/TTL-only transfer or an increased Space timeout.

## Artifacts

Private root `/home/ubu/.cache/lay/td121-real-ibus-imX7Og`;
unit `lay-td121-real-ibus-imX7Og.service`, invocation
`910d7c6f646c4a1cb86e455f4b62b751`.

| Artifact | SHA-256 |
|---|---|
| Installed ibus-daemon | `24338c0e7cfb749d2ac9b9254d7d54029e71b5ac9342aa1b8d78b1d55f5ac1b6` |
| receipt.json | `5bc95e40e313d708dc200db3ebe5cc256a485704441c1380cedacbf7fda9e44c` |
| driver.py | `c6a9b40863d87cab6bca8675b13aa9b5dc26a9adcf12d1a079e645f2291630f1` |
| launch.sh | `a4cbc70c45c1d91ba3d3855cf5b029625c639faab87b81ba18b7e0c37b70a885` |
| dbus.conf | `fa33ebaea2f2a489d1d59f0f7e1d0dd046bf52a86bc9e9570086b81f30f01ee4` |

Raw synthetic events remain in the private receipt, not copied into Git.
Rollback is stopping/reaping only the owned private process; no live mutation
was made. Required architecture refresh belongs to the final coherent change,
after the concurrent TD-120 source patch settles.

## Header-contract follow-up (separate immutable run)

Exact command:
`bash /home/ubu/.cache/lay/td121-real-ibus-headers-xJm4MG/launch.sh`.
Same installed IBus hash and isolation/resource limits, fresh private root;
unit `lay-td121-headers-xJm4MG.service`.23/23 cases passed: the previous
bootstrap/20 ABA checks, actual sender-header spoof attempt and callback-header
capture. No key delivery, zbus helper or production Lay transfer claim.

Measured mapping is **three distinct roles**, not one sender comparison:

| Message | Observed authenticated sender |
|---|---|
| Factory CreateEngine and Engine FocusIn/FocusOut/Enable/Disable/SetCapabilities | `org.freedesktop.DBus` |
| GlobalEngineChanged | IBus unique owner, `:1.0` in this private epoch |
| Own self-marker | Own factory connection, `:1.1` in this private epoch |

The second connection explicitly set its marker's Sender header to the own
connection's name; real IBus rewrote it to the actual second sender. Own-sender
validation therefore distinguishes it. No duplicate complete
`(sender, serial, path, member)` keys were observed; serial is still correlation,
never the ordering clock. ProcessKeyEvent headers were not sent in this fixture.
Never hardcode these ephemeral unique-name numbers into production.

Driver0.392s, service0.597s, CPU0.474s, peak35.9MiB, swap0. Owned PID5 reaped,
return−15; service exit0. Live global IBus4715, Lay daemon3453123 and IME3453154
remained unchanged. No global AppArmor/runtime/config mutation.

| Follow-up artifact | SHA-256 |
|---|---|
| receipt.json | `9e036f41af70ec9d8eaac36f0a08a645d29068545cfe92af099ffef15f6202e8` |
| driver.py | `021d6aa2f9b19add16e68af5d8a6b08990a4599f903eefddc9c376b07fec9f0b` |
| launch.sh | `21e05a2eeafddf3791f3a315e8c2610f946c3ede8aec054cff52e6262cb79f17` |

This closes the deployed Factory/lifecycle sender distinction needed by
P121-3; remaining-budget unit proof and actual adapter/key callback integration
are separate gates. The22 and23 cases are separate overlapping diagnostic runs,
not45 distinct implementation cases or a per-language quality denominator.
