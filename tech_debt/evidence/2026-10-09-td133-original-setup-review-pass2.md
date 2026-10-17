# TD-133 original activation diagnostic — independent final review, pass 2/2

**Score: 9/10. ACCEPT_DIAGNOSTIC_CHECKPOINT for two committed-text/preedit/caret controls and the bounded trace evidence. No remaining checkpoint blockers after the rendering-scope correction. This does not close TD-133, certify rendered/native acceptance, or prove an original failure cause.**

Reviewed HEAD: `89e075e2932bee020731fc5e90a4c65648d38329`. The current work concerns the TD-133 card, owning architecture entry, preparation packet and result packet; no production source change was present. The earlier preparation and pass-1 review retain their historical NOT_AUTHORIZED cutoff. The parent supplied the later explicit user grant, “Разрешаю один такой локальный прогон”; its exact two-stream scope is now consumed. No additional local stream is authorized.

## Pinned evidence

Raw receipt: `/home/ubu/.cache/lay/development/td133-original-activation-once-9pkdjzdi/RESULT.json`, SHA-256 `1d30fe73234f93f3cd8f70250b133d87a28c438a52c152746f4768b4ee5ee4c2`.

Executed probe SHA-256 equals the reviewed script: `ff3d3a19fb62d193e51ea301a00c99d4e0cdc01214011a80c327dd6d59f25f7b`. The corrected result packet, `tech_debt/evidence/2026-10-09-td133-original-activation-result.json`, had SHA-256 `d6441f7c3267ecc23979b97eaeeb98d51d1708ba4818a7bd91d57ec1365c107f` at this review cutoff, before its pending review metadata is finalized.

Both private trace hashes, byte/record counts, exact assertion snapshots and later-state hashes/snapshots/deltas were independently checked against the files.

| Field | Trace records | Typed presses | Callback serials | Typed owner |
| --- | ---: | ---: | --- | ---: |
| GTK3 Entry | 525 | 13/13 | 28069–28112 | 634 |
| Qt rich | 587 | 13/13 | 28160–28204 | 639 |

Each decoded sequence is exactly `должен ыбыть `; each physical code sequence is `[38,36,37,39,20,21,57,31,51,31,49,50,57]`. For every typed event, the preceding ProcessKeyEvent enter/stamp/admission and callback-binding metadata agree on serial and owner; admission is accepted and enter disposition is press. Binding process IDs match the receipt's managed IME PID. Packet sequence entries and surrounding metadata match the raw traces without discrepancies.

All 13 callback clocks per field strictly increase and fall between that field's raw-clear-after-activation and final assertion observation. First callbacks occur 17.968600 ms / 27.563059 ms after those activation observations; last callbacks precede final observations by 2.989109 ms / 5.288985 ms respectively. Configured key timing remains 18/12 ms and the assertion remains 1.5 s; these settings are not a claim of identical measured callback cadence.

In both traces the first Space is `space_managed_native_passthrough`, handled=false. A real `(text_chars,cursor_pos,anchor_pos)=(7,7,7)` receipt follows at GTK line 354 / Qt line 404 before the first right-token `ы`. Their Space-to-right callback intervals are 40.142856 ms / 36.135904 ms. This proves the recorded order in these successful controls, not the cause of the historical failures.

## Material scope correction retained

Both assertion snapshots have exact committed/text `должен ыбыть `, empty available preedit and caret 13. The raw script label `PASS_VISIBLE_CONTROL` is preserved as historical metadata, while the successor correctly uses `PASS_COMMITTED_TEXT_PREEDIT_CARET`.

GTK assertion sequence 52 also has available layout text `должен ыбыть ыбыть`. Retained sequence 82 has matching layout text 1.275739099 s later. The packet/card/owning document now retain both, instead of promoting the committed-text check to an unqualified rendered PASS. Qt layout text is unavailable in assertion sequence 34 and later sequence 66. No pixels or continuous frames were observed; the first layout settlement time remains UNKNOWN. The fixture's synchronous signal publication and existing 40 ms timer are source facts, but a sampling-order explanation remains an inference. Neither a renderer defect nor a harmless stale observation has been established.

## Restoration, limitations and blockers

The raw receipt's before/after tuples match exactly: managed PID 3139840, starttick 316158626, installed/loaded accepted `2bd88bfcbc53e9916d56b3560ca8d7cf7cde17c8fdeb4e391d8f2c42f7310559`, and coherent RU source/engine/decoder. Its logging-state check/restoration is true and cleanup_errors is empty. This field also covers an unchanged logging configuration; it is not independent proof that a config rewrite occurred or that restore is atomic CAS. The unchanged probe retains its owned-device/key/child cleanup and no retry/fallback. No inverse, shared-feedback cleanup, install/restart or source-authority change is introduced.

Both last native context replies before typing use `InputContext_7`, despite distinct engine/owner generations. Serial/time correlation and publication records do not uniquely bind a native field PID to that relay or prove that field Ready before input. Those limitations remain explicit. Current source has `spawn=false` and the native-Space/IME-printable paths; source ordering does not prove cross-client consumption causality or installed-source parity.

The original **62 PASS / 2 FAIL / 0 BLOCKED out of 64** remains unchanged. The preceding standalone two controls and these two original-activation controls are separate diagnostics. Fresh fields, logging and uncreated full-matrix predecessor/warm/learning state limit the contrast. The original committed-surface failures were not reproduced; the GTK layout conflict remains a separate unresolved observation. Cause, first original failed transition, causal RED/repair and native acceptance remain OPEN/UNKNOWN. No runtime repair is selected or authorized by this review.

No required code or further execution changes. The evidence-scope omission was corrected within this final pass; no third pass or rerun is requested. No reviewer tests/builds, diagnostic imports/execution, native/GUI/input/service operations, graph refresh, repository edits or agents were used. This report is the only pass-2 reviewer write.
