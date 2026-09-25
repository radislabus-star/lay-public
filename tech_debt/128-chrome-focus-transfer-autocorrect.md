# TD-128: Chrome Space correction after a focus transfer

Status: **OPEN / physically reproduced / no accepted repair**.

The installed engine SHA-256
`daaa47bf400b8fb06d124a31c0790422f8a830aa26cecfdfd8152b2687687b88`
passes continuous suggestion and `публекует ` → `публикует ` in separate
fresh-field Chrome runs and in two direct same-word runs in one field. In a
single ordinary Chrome window, typing `пу` in
one textarea, moving by Tab to another, then typing `публекует ` leaves the
second field unchanged. Receipt:
`/home/ubu/.cache/lay/development/browser-autocorrect-20260922/physical-chrome-final-accepted-combined/receipt.json`
(SHA-256 `6f469b2ae96974462c25c3e1436e0d7257dd97336274f6516ee721ec547960fa`).
The second field advertises capabilities 41 before its first printable,
receives managed per-character CommitText, and lacks an exact word-start
witness. Space therefore has no authorized correction frame. An explicit
layout reactivation can admit the field but does not give this client an
exact surrounding-refresh marker.

Two input-ownership experiments are rejected. Treating all unmarked
surrounding-capable clients as owned preedit caused 10 unexpected failures
in the fixed project gate (`run-_6qt7fnj/RESULT.json`). Restricting preedit
to an admitted empty UnknownStart with no external snapshot still failed
11 of 595 engine tests in the TD-121 transfer and Reset/replay family
(`chrome-narrow-focused-bin.log`). Both changes were reverted before the
accepted engine was built. No SafetyGate, edit-plan or verifier relaxation is
authorized by these observations.

Investigate the first authority loss across
`FocusOut → FocusIn → source-free/transfer activation → word scope → first
printable → Space frame`. Keep source-free words, transferred tails and
Reset/replay inheritance distinct. A repair must show why the new field's
first word is locally complete without treating a stale client snapshot or
another field's tail as authority. Group the existing 11 failing contracts
by their shared transfer mechanism before making another runtime change.

Acceptance requires a no-manual-reactivation same-window Chrome result:
continuous suggestion in the first field and exact correction in the second,
on one installed archive-built engine. The complete fixed gate must pass with
zero new failures, including the TD-121 transfer/Reset negatives and TD-125
capability-transition contracts. Firefox and fresh-field Chrome must retain
their accepted physical behavior. Other client classes, performance, package
size, RSS and latency remain unmeasured by this report.

Owning architecture record: `docs/ime-daemon-route-map-2026-06-20.md`,
“Accepted browser display source and installed artifact” and its follow-up
experiment.
