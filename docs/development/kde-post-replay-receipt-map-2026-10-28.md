# One mechanism: KDE post-replay pending field receipt

Status: preflight before code. Canonicalbaseb835ca478198d6cb765a2246329d5646966c118c;
source1301files/2ce6b24786bab558599a46345c9ec3583f419847420a1f37ebf5a08e93933c5e.
Prior pacing-only experiment rejected and fully restored; retain that evidence.
Current daemon246741/eb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5,
engine3556137/8b84bf5b6c569550af8e4061ea4cca83e74382d243ae693aeb243a1e22a035d3.
User requests one improvement, map first, explicit shared scope and isolation,
short tests before long. No Harness, agents, local GNOME changes or questions.

## First failed transition and dependency map

The exact-tail replay has already passed both lease checks, controlled layout
handoff, suppression and output. First post-output VisibleTailV4 can be the
existing sanitized `passive:unknown-context` sentinel: target engine path,
advanced epoch and target decoder are present, but field/text are empty and
personalization false. The common lease's field check correctly refuses to
interpret that sentinel as an exact current field. The post-output waiter
mistakes this temporary unavailable observation for immediate final failure.

```text
src/bin/lay_daemon/exact_ime_tail_replay.rs:280
  → layout_controller/ime_bridge/settlement.rs: existing 80 ms read-only wait
      → layout_controller/ime_bridge.rs: current_v3_reply / bound_v4_reply
         currently rejects missing field receipt immediately
  → ExactImeHandoffCleanup::drop: CancelExactManualToggleSuppressionV2
      → IME window_interaction/authority.rs:567
      → IME tail_memory.rs:2284: exact replay scope revoked
      → IME window_interaction/observation.rs:4350–4389
         delayed source prefix loses replay provenance, reset lineage discarded
      → later exact final text cannot restore the discarded predecessor
      → IME window_interaction/authority.rs: context_authority refusal
```

The rejected native trace is CACHE/kde-native-cadence-short-v1-row3.jsonl and
CROOT/kde-native-cadence-short-v1/native/firefox/row-3-native-trace.jsonl.
Event600 cancellation precedes event602 mismatch/replay_scope_present=false;
event603 discards lineage; event610 supplies final six-letter SSI. Four later
gesture calls refuse manual authority while bridge token and input stay live.
The IME's existing `exact_replay_contains_prior_snapshot` already retains only
observed source deletion/replacement progress while its exact scope survives.
No change to that IME mechanism or manual-toggle authority is proposed.

## Options and selected minimum

| Option | Score / consequence |
| --- | --- |
| Keep fixed pauses / add longer pauses | 4/10: can mask receipt ordering; no proof of final field or stable latency. Original pacing stays for this task. |
| Weaken manual-toggle authority or accept a missing field | 1/10: violates C03/C05/C07. Rejected. |
| Retry text output, reconstruct the word or choose another route | 1/10: risks a second mutation and changes frozen delivery. Rejected. |
| Preserve read-only pending state in the existing post-output deadline, KDE only | 9/10: retains existing exact scope until a real field receipt or existing timeout. Selected; native proof still required. |

## Exact source scope before editing

Runtime: new `src/bin/lay_daemon/kde/exact_replay_settlement.rs` owns KDE-only
observation classification. `layout_controller/ime_bridge.rs` only wires that
private module. `layout_controller/ime_bridge/settlement.rs` keeps the existing
single waiter, with a small testable read seam and a pending/current distinction.
Tests in `kde/exact_replay_settlement/tests.rs`; canonical manifest and its
known-failure manifest hash updated only by actual discovery, no exemptions.
Scope guard uses the existing daemon KDE directory category `kde-focus`, not
a claim that the focus provider changed. The two shared files are declared.

This is shared KDE post-output integration, not Firefox-only runtime code.
GNOME and non-KDE calls retain the original current_v3_reply rejection. Queued
pre-output waits without an explicitly pinned target path also retain it.
All IME source, emitters/pacing, exact executor, input driver, focus provider,
layout controller and its transaction, candidate ranking/learning and verifier
are protected, byte-identical to baseline.

## Consequences, required invariants and rollback

- Only the existing sanitized unknown-context shape may remain pending: KDE,
  explicit nonempty target path equal to the observed engine, exact target
  decoder, strictly advanced epoch, empty text and field receipt, false
  personalization. Unknown tags, old/missing/different paths, stale epochs,
  other decoders and malformed payloads do not get this treatment.
- Pending is not a field receipt or word/selection authority. It cannot pass
  the existing settled predicate. A later current observation still must pass
  unchanged personalization, source, exact field, path, epoch, decoder and text
  checks. Source/focus/permission contradictions remain failures.
- Verify original focus before every read and again at success. Keep the one
  original80ms deadline, yield and timeout; do not renew it for pending reads.
  No new polling owner, queue, timer, cache, transport, protocol or mutation.
- Missing field is not an observed change in personalization; the pending
  sentinel is sanitized. Do not retain any payload or log text from it. A
  complete changed permission remains an error before success.
- Candidate/lattice, ranker, false authority, other clients' field behavior and
  actual text delivery do not change. The common current lease read remains
  strict for every pre-edit validation. This adapter is post-output only.
- A permanently unavailable field may now consume up to the already specified
  80ms post-output budget on KDE. If the scope cannot settle before native
  resource retirement, this approach must be rejected; no wider timeout or
  premature success may compensate.
- First verify this mechanism with original paced output. Do not reapply the
  rejected fast cadence unless new independent source and native settlement
  evidence first establish this repair. Keep each experiment separately.
- Rollback only the new adapter and two wiring/read-loop seams, preserving
  Chrome's accepted observation fix. Record a rejected native result, restore
  known bytes, and do not rerun unchanged code for green.

## Proof order

1. RED: scripted unknown-field then exact-field observation fails at the old
   bound-field read. Negative field/epoch/path/mode/privacy/focus cases retained.
2. GREEN: same sequence settles; pending never grants a current field; other
   backends and pre-output calls reject the sentinel; permanently pending reads
   hit the original deadline. No global environment or live client in unit tests.
3. Canonical shared-replay component and ordinary physical-owner checks; remote
   architecture graph/canon and warning-free daemon release build.
4. Same native three-case prefix and exact delegated-route/receipt observations.
   No extra delay or assertion change. Long Firefox only after the short gate
   succeeds; keep client verdict, backend settlement and timing separate.

CROOT `/home/e/projects/lay-development-runner/kde-install-control-20261020`;
CACHE `/home/ubu/.cache/lay/development/browser-matrix-repair-20261027`.

## Source RED — 2026-10-28

CROOT/kde-post-replay-red-v1/result.json:7selected,5PASS,2expectedFAIL;
3.295s discovery and execution, no installation/input. Both delayed-to-current
and focus-loss-after-pending regressions fail at the unchanged strict V4 read:
`exact IME tail field identity is unavailable`. Five safety controls pass.
Source37411845c3331c982094f72e80ecd5ac23cf2ca2b0e5c34adbe78083bdb1d400.
The read seam and tests are present, but the pending classifier still uses the
old rejection. This is a controlled old-classifier RED, not native acceptance.

## Source GREEN and gates — 2026-10-28

Same seven tests:7PASS/0FAIL in3.268s at
CROOT/kde-post-replay-green-v1/result.json. Discovery source
2e5f6b1af226320870cdbbea102d634f9fed3f6e55926a2cfa89583e347f6d76.
Remote rustfmt changed only the owned test file; full canonical discovery added
exactly these seven identities and retained every previous identity. Known
failures remain zero; no waivers. Preparation receipt
CROOT/kde-post-replay-prepare-v1/result.json.

Official daemon development check:336selected/336PASS/0FAIL,169.422s transport
and check. Local CACHE parent/run-l5f6ygc2/RESULT.json; remote
/home/e/projects/lay-development-runner/run-QUe2zY/tests/SUMMARY.json. Source
f111238dc7f3d0440f77c888eccb6d34ee24842ef177561e5b320949e84bbd1c.
This is source evidence only, not native/client acceptance.

Canonical shared-replay component:23selected/23PASS/0FAIL,0.796s. Fmt and
remote graph/architecture PASS (graph gate52.981s). Receipt
CROOT/kde-post-replay-component-v1/result.json; six changed tracked graph
metadata files synchronized after checking their old/new hashes. No runtime
Rust changed during refresh. Source including generated graph receipt:
1e21aabdc5638504d57032e5520912b4b81aad966ae1c0109f1d5f96eddcfdef,
1303files. Candidate build and native acceptance still pending at this entry.

## Native rejection and user stop — 2026-10-28

Candidate daemon9dd15093b632cdfcbbb9ca49cb335f61b812f5a1861d86eda9c902ac8e22f265,
PID364720, warning-free release66.794s. Unchanged three-case native input:
2PASS/1FAIL in15.447s; roundtrip158.15/199.09ms. Rapid-five has all five
ManualToggleV3 status3 calls but wrong final text. Per-row trace anchor is
incomplete, so do not claim a complete native causal trace. Passive complete
session-bus wire captured124V4 replies; all six post-output groups remained
sanitized unknown-context for74.3–79.6ms until the original deadline. No final
exact receipt was obtained. Waiting-only approach REJECTED; no long matrix.
Receipts: CROOT/kde-post-replay-short-v1/result.json and session-wire.log;
CACHE/kde-post-replay-short-v1-v4.json retains matched calls/returns.

Running daemon restored to baselineSHAeb6098b445c48ae5a681fb8b1d33051e28a5da3646d90baa4cd47b09a7228de5,
PID374525; engine/IBus/tray/config unchanged and native input ready. Receipt
CROOT/kde-post-replay-rollback-v1/result.json. All12 owned source/manifest/graph
files restored/deleted after validating hashes; local and remote source1301files,
SHA2ce6b24786bab558599a46345c9ec3583f419847420a1f37ebf5a08e93933c5e.
Rejected source archived CACHE/kde-post-replay-rejected-source-v1.tar.gz;
rollback payload CACHE/kde-post-replay-source-rollback-v1.json. Cached target
release daemon is rejected; never install it. No functional runtime diff remains.

User explicitly stopped work for manual diagnosis. No new implementation or
native input until continuation. Need actual manual failure description before
selecting another mechanism. Existing native-document refresh is Chrome-only;
do not assume it is a reusable Firefox adapter. Why final V4 identity is absent
remains unresolved; neither resource retirement nor IME callback starvation has
been established as the cause. Full matrix remains306PASS55FAIL79BLOCKED/440.
