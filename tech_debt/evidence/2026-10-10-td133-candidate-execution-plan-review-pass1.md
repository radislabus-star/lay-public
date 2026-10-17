# TD133 candidate execution-plan review — pass 1 — 2026-10-10

Verdict: **8/10; one material recovery-protocol blocker before presenting the plan for a NEW install-plus-four-stream grant.** This is a read-only review of a new execution plan and its diagnostic successor. It is not another TD133/TD138 product-code review, and it grants no execution. One review pass was used; no tests, builds, graph commands, network, UI, diagnostics or runtime operations were executed.

Reviewed checkout: `/home/ubu/projects/lay-space-boundary-shift-20261007`. Read AGENTS.md, ARCHITECTURE.md C01/C03/C05/C07/C08/C09/C10 and DEVELOPMENT.md local/remote, private-client and diagnostic boundaries. Reviewed PLAN.md SHA `393028e0e049402e661b85a2e048870e2f8746e234deb8230a028de209cddf35`; PREPARATION.json SHA `6b90fa4f650acdc320137a6139469f21656fe632322daa9bf766ba6174a32873`. Preparation remains NOT EXECUTED.

## Material finding

**P1 — The fixed three-window transaction loses its safe rollback field before recovery.** PLAN.md:58–62 closes the original empty install witness before starting the two diagnostic fields. The successor closes its own fields on both ordinary completion and failure (diagnostic:323–327, 334–350). PLAN.md:71–76 nevertheless requires replacement/restart on installation or probe failure, without another owned, fresh, empty-field observation immediately before rollback. At that point focus can have returned to a user window. The empty-field protection explicitly required for forward installation (PLAN.md:49–55) therefore does not protect the reverse mutation/control operation.

Minimal plan amendment: keep the **same** original empty GTK witness alive throughout the transaction; close it only in transaction finally. Before any rollback replacement/control, require its still-live PID/starttick, currently focused title/PID, fresh <=1s empty committed text/caret0 and available empty preedit, then repeat the binary-writer check. If that witness is gone, nonempty or no longer owns focus, record recovery BLOCKED and preserve the partial receipt rather than perform an unguarded restart. The window budget stays three, no further typed stream is needed, and no new framework or installer is required. This is a plan-ordering correction, not a runtime patch.

With that amendment the plan is concrete enough for the root operator to present the exact scope for a NEW explicit grant and manually use existing helper functions. The grant must still be obtained. A second review is warranted only if the repair materially expands the transaction or introduces different executable code.

## Verified preparation and retained boundaries

Local file reads/hash checks matched these declared pins:

| Artifact | SHA-256 | Bytes |
| --- | --- | ---: |
| Candidate IME 1.0.82 | `5fe100db732bcc945d9c7d584a2c22bd06eb93c19bee09cc39788c85470dee7a` | 8185232 |
| Prepared rollback IME, mode0755 | `2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559` | 8185744 |
| New diagnostic | `6c0585778c0d72b178fb18c63c6bc7b891e37b0555618a9fabd83498d8af24a1` | 22354 |
| Reviewed parent diagnostic | `3d45c368b075dd3b5fffaf040fa66ea655ac32f48b5406fb49806256b5d70cba` | 21783 |
| Existing generic installer declarations | `6d3da8b658d6f17e37ebe2db7a060e5326b26c5f74acd56c39b3bcfcc50dca82` | 8425 |
| Existing runtime-control shell | `8f4069eb4b2900fcb148eaa331e6094c4790b57355395d9fb59b1003ac3659ab` | 7542 |
| Full source gate | `248d9ecbd967fd39b1985a205041b514f798798f8640c0d6527999cf9e208212` | 5003 |
| Private six-control RESULT | `2cb79e01cc08d5ff9226d5801438c4a4746c87c7560c7e27303bff88ff64f4c4` | 27690 |

The new diagnostic diff changes candidate/full-gate/private-proof pins and provenance metadata only. Stream bodies, observers, key timing, assertion deadlines, signal handling and cleanup remain those of the reviewed parent. Four text streams remain gtk.clean_opener, gtk.warm_invalid_left, qt.clean_opener, qt.warm_invalid_left, with18/12ms keys and1.5s assertions. Existing owned-field clearing/setup keys are not new text streams; no extra Ready wait, inverse or retry is added. Existing post-verdict logger drain is not a latency allowance.

The plan uses only one executable replacement and one existing `channel ime` invocation per install/recovery, preserving other roles and global IBus. `install_verified_release.py` is reusable through its declarations/functions; its historical `main`, `reload_managed`, `reload_extension` and historical CONTROL must not be invoked. `snapshot` itself does not return startticks or InputMode: the plan correctly requires those additional observations, and PREPARATION_RUNTIME.private.json records them. That preparation snapshot reports exact2bd installed/loaded, extension1.0.81 and coherentRU; it remains a dated preparation observation, not proof of the identities at eventual execution.

Forward/reverse mutation must retain the plan's immediately preceding target-hash checks and refusal of unexpected writers. Neither helper atomic_copy nor the diagnostic config restoration is an atomic compare-and-swap. The diagnostic explicitly records its best-effort unchanged-byte restoration contract and refuses an observed concurrent config change. Root must preserve that qualification, inspect logging_config_restored/cleanup_errors/runtime_and_mode_preserved and preserve unrelated changes; a child process exit alone is insufficient.

The diagnostic reports DIAGNOSTIC_CAPTURED_NOT_CAUSAL_VERDICT even when a target row is FAIL_VISIBLE_CONTROL. Root must require exactly four successful declared stream results, both complete zero-drop/error widget traces, candidate/runtime/mode identity and restored logging configuration before leaving candidate installed. Its target predicate proves separator/prefix/final-Space/caret/preedit integrity, not exact lexical correctness or causal attribution of the historical failures. The manual plan already states that result distinction.

Resource route is bounded human-safe-run with768MiB for witness and probe; the probe refuses absence of finite inherited CPU/memory caps. Widget lifetime, waits and key schedule are bounded. Runtime settle has the existing15s bound. Heavy/source checks remain remote and are reused, not executed by this transaction. Ordinary positive learning can append; no native inverse or shared-learning deletion is authorized.

Prior input/install grants are expressly consumed. Publication does not authorize this experiment. Successful execution would leave exact5fe100db as the newly granted **experimental** runtime only. No tag/release, TD133 DONE, universal field acceptance or native64 completion follows.

## Proof denominators

| Boundary | Current evidence | Meaning |
| --- | --- | --- |
| Canonical source | 3048/3048 required functional +11/11 performance, lint/release PASS; source archive4c53a2e… stable | Source scope; production review already completed2/2, not reopened here |
| Actual private consumer | Terminal4/4 plus inverse1/1 +1/1 PASS on candidate | Six controls, private IBus/readline; not native acceptance |
| Unchanged lifecycle | Candidate0/3 and baseline0/3 completed, both FAIL at owned-composition projection | Preserved failures; cannot be promoted using the six PASS controls |
| Restoration | Five cases not reached | NOT TESTED |
| Proposed changed-byte native controls | Four declared warm controls, not executed | New experiment; not original fixed64 chronology or compatibility matrix |
| Original/native acceptance | Historical62/64; changed-byte native64 pending safe isolated successor | TD133 remains REPLAN_REQUIRED_NATIVE_ACCEPTANCE |

Public source packet SHA `7057e73d2c641cd340eedbb2e267cf2c4d9b1f80f225f376ad1fae28ca54d40c`; public actual-client packet SHA `f939dba5e0bf9631b7ccdc12eac1f304c3b2b6f0b50049d81a44e3b1d13af929`. Their PASS/FAIL/NOT TESTED boundaries are consistent with PLAN.md. TD138 DONE_SOURCE_SCOPE does not change the other verdicts.

## TD139 staged proposal

**9/10 as an evidence-driven stage2 discussion card; no implementation or execution is selected.** Reviewed card SHA `7361aeea19e49583b7e935b265a7a11f6bcce893b4f99a5c0eae5c41dfc9373a`.

It identifies the demonstrated shared consumer mismatch rather than declaring baseline/product acceptance, compares three designs, preserves immutableV3 failures and separate lane denominators, keeps preedit out of SurroundingText and deletion authority, and requires actual consumer RED tests, versioned proof migration, remote guards and its own review. It is correctly deferred and is not a new prerequisite for the proposed four-stream experiment.

One angle to make explicit during stage2 design: define preedit cursor/selection units and invalidation for Reset/disable/destroyed or superseded input contexts, with a late update/show after such revocation as a RED case. The card's general owner/context and focus-loss requirements are sound; this sharper lifecycle oracle would prevent a render-only observer from resurrecting stale owned composition. It does not block this discussion card or justify a runtime change.

## Review limits

All conclusions above come from current plan/code/evidence file reads, comparison and hashes. No installed-runtime liveness, physical-client behavior or executable safety was newly exercised. No source product refactor, new proof framework or extra acceptance batch is requested. Execution remains pending a NEW explicit user grant after the rollback-field plan correction.
