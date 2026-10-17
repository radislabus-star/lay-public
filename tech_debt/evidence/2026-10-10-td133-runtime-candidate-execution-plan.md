# Exact-byte experimental install and four-stream native control

Status: PREPARED, NOT EXECUTED. Requires one new explicit user grant for both
the IME-only install/restart and the four declared input streams. Prior grants
are consumed. No remote desktop. This is an experiment, not release acceptance
or completion of TD133's original fixed64.

Candidate `/home/ubu/.cache/lay/development/tab-full-4reofpse/lay-ibus-engine`,
version1.0.82,8185232bytes, SHA256
`5fe100db732bcc945d9c7d584a2c22bd06eb93c19bee09cc39788c85470dee7a`.
Full gate SHA248d9ecb…:3048/3048 required+11/11 performance, lint/release PASS;
source archive4c53a2e… stable, production review9/10, two passes. Actual private
terminal4 and bound inverse1+1 PASS; unchanged lifecycle0/3 fails on both old
and new bytes because general Client ignores owned preedit. Restoration5 not
reached, not omitted from release acceptance. No changed-byte native PASS yet.

Verified rollback is already prepared at
`/home/ubu/.cache/lay/development/td133-runtime-candidate-20261010/rollback-ime-2bd88bfcbc53`,
exact2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559
and original mode. No models are copied. Source publication, if performed,
does not grant execution of this plan.

## Existing reusable tools

- Generic runtime helper `/home/ubu/.cache/lay/development/ranker-release-20261004/install_verified_release.py`,
  SHA6d3da8b658d6f17e37ebe2db7a060e5326b26c5f74acd56c39b3bcfcc50dca82.
  Import declarations only; call its existing `sha`, `atomic_copy`, `snapshot`,
  `shell` and `run`. Never call `main`, `reload_managed` or `reload_extension`.
- Actual checkout `scripts/lay-runtime-control.sh`, SHA
  8f4069eb4b2900fcb148eaa331e6094c4790b57355395d9fb59b1003ac3659ab.
  Only `channel ime`: existing same-language temporary XKB then managed IME
  replacement. It preserves global IBus and other services/input source list.
- Owned empty GTK witness uses pinned generic `window_matrix.Field` with
  ui=None (no keys) and pinned native_field_fixture.py. Pins are in the prepared
  diagnostic. Its app starts in a fresh private output with a unique title.
- Four-stream successor:
  `/home/ubu/.cache/lay/development/td133-candidate-client-ordering-diagnostic-20261010.py`.
  Exact hash is in PREPARATION.json. Its only changes from reviewed parent
  3d45c368… are candidate/source-proof pins and provenance metadata; unchanged
  stream bodies, key schedule, deadline, observers, cleanup and config rule.

## Transaction operated by root

1. Revalidate candidate/gate/probe/helper/control/backup pins, archived runtime
   source and actual managed runtime2bd. Read installed and loaded hashes and
   PID/starttick, active service roles, extension version, GNOME source,
   IBus engine/InputMode, source list and config hash. Require coherent RU.
   Save PRE_INSTALL.json privately. Reject drift before any live mutation.
2. Under human-safe-run768MiB create exactly one owned empty GTK witness, no
   typed input. Bind unique title/PID/starttick, current focused-window identity,
   fresh<=1s empty committed text/caret0 and available empty preedit. Save its
   observation. Never replace the binary while user text/composition is active.
3. Recheck empty ownership and current target2bd. Write STARTED receipt before
   mutation. Atomically copy only candidate to existing installed IME path
   using existing helper, preserving mode; verify SHA before launch. Execute
   existing `channel ime` once. No rebuild, package/config/extension write,
   global IBus/daemon/L1.1/L3 restart or source-list migration.
4. Within existing15s runtime-settle bound require new IME PID/starttick and
   loaded/installed5fe100db. Keep other role PID/startticks, loaded/installed
   hashes, extension version, original GNOME source/IBus mode/source list and
   config hash. Record INSTALL.json. Recheck the empty owned witness and retain
   it through all diagnostics as the original rollback context. Do not close
   it before the transaction is settled.
5. Run successor once in the same bounded scope with
   `--allow-local-physical --client-event-ordering`. It opens two new fields;
   each types `должен `, clears only that field, normally activates RU, then
   types `должен ыбыть `: four streams total,18/12ms keys,1.5s assertions.
   Require exact prefix, one separator, final Space, caret, empty preedit,
   current owner, unchanged loaded candidate/mode and complete observer traces.
   No retry, inverse, shared learning cleanup or extra Ready delay. Ordinary
   positive learning may append; no user learning is deleted or replaced.
6. Save all partial results and post-runtime identity. Before any recovery
   byte replacement or IME control, revalidate the retained original witness:
   current focused title/PID/starttick, live child, fresh<=1s empty committed
   text/caret0 and available empty preedit. Diagnostic child cleanup normally
   returns focus; it is not trusted without this exact check. If the witness
   is lost or safely obtaining its focus is unavailable, record recovery
   BLOCKED and request steering. Do not use a user window, add a fourth window
   or synthesize focus keys. Keep partial receipts and no further input.
   With valid witness, restore prepared2bd bytes and use the same IME-only
   control once, verify recovery. Also prove target still equals this
   transaction's candidate (or already-old bytes); an unexpected writer/hash
   blocks overwrite. Preserve unrelated config/source changes. Independently
   clean owned keys and diagnostic children in finally; close the install
   witness after success/verified recovery or after recording blocked recovery,
   never before its final required empty-context check.
7. On four successful controls leave candidate as the explicitly approved
   experimental version, recording its exact SHA/scope. Do not tag/release,
   mark TD133 DONE or claim native64/universal acceptance. Original62/64, old
   failures and lifecycle baseline failure remain immutable.

Three new windows total: one empty install witness plus two diagnostic fields.
Only four declared text streams. Root owns execution; reviewers read only.
Rollback and result inspection can proceed under this same grant after a failed
attempt; another input batch would need a separate explicit grant.

## Consequences and boundaries

No added runtime owner, API, queue, timer, cache, ranking/lexical protection,
SafetyGate/verifier or model mutation. The reviewed candidate changes scoped
ordinary Space emission and release bookkeeping; existing terminal/opaque and
atomic routes remain. Real Wayland aggregation is the purpose of this trial.
The pause/atomic replacement/restored-source protocol is the existing one.
One changed executable and two ordinary native diagnostic fields are the
minimum useful practical next step. Its four positives cannot establish the
old two failures' exact chronology or the complete compatibility matrix.
Safe actual-engine four-path isolated native64 successor remains separate
work; retired native inverse/cleanup launchers are never used here.
