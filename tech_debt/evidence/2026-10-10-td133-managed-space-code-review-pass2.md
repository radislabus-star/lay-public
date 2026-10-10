# TD133 implementation review — final pass 2 of 2

**9/10 — PASS for implementation review; release remains BLOCKED and native successor NOT_TESTED.** R1 and R2 from pass 1 are resolved in the reviewed source. No further material code finding requires a third review or a TD133 production replan. This does not close TD133, establish the original native cause, or authorize installation.

Read-only review of `/home/ubu/projects/lay-space-boundary-shift-20261007`, HEAD `24bcea5256661954d5aba97ff037d6a419e1e2cb`. No repository tool execution, tests, builds, Graphify, runtime/native/GUI actions, installation, restarts or sub-agents. Only this report was written. Exact source hashes were checked against the final full-gate binding and rechecked at review cutoff:

| File | SHA-256 |
| --- | --- |
| `managed.rs` | `718bb6ba3d046e99c93f7697d38a2993b1eec2010c0d862aefe5189ae08af913` |
| `window_interaction/observation.rs` | `2dacbac0a87bb7b10767bc0490e8d2e2fa98bff8a29fb8b44aa6852f982c5697` |
| `adapter/tests/terminal_delivery.rs` | `cd0bf1d14999a1b638d43cb5ed910a499055263b0c5cd5fb8d642e89d4a3869f` |
| `adapter/tests/residuals.rs` | `473eae29b61164538e8f15dfb787bdcd060452b1e9f33b46bd8260dce461f498` |
| `adapter/tests/word_scope.rs` | `f2ceb70a63670a90168e531116b0b143e4664e32f92ed35352da925b76671871` |

## R1: resolved without restoring edit authority

The real Reset-before-Space-release regression first failed before lifecycle repair: `/home/ubu/.cache/lay/development/td133-candidate-green-___tdko_/REGRESSION.json`, 2 selected / 1 PASS / 1 FAIL at the accepted-release assertion. The final test now uses actual Reset, FocusOut and Disable callbacks, tests the 701ms expired case, rejects a duplicate release and verifies exact next-glyph delivery without a new receipt. It asserts no extra text or deletion from lifecycle callbacks, no managed-word witness and no restored Space correction frame after Reset.

At `window_interaction/observation.rs:3851`, the repair snapshots the existing pending handled-press set only for non-Atomic, surrounding-capable, nonterminal input whose tail ends in literal Space and whose last successful commit is within the existing 700ms recency bound. The snapshot is taken after the existing Reset revocation route. Stale/mismatching accepted-admission callbacks return before this restoration path; a verified focus change already clears the existing set. Opaque/terminal capabilities and expired commits fail the new predicate. Disable and FocusOut retain their separate cleanup paths.

Soft Reset still revokes/clears its existing word, composition, snapshot and prepared-work state. Restoring pending keycodes does not restore a correction token, verifier permission, managed-word-start witness, snapshot floor or model/ranker result. The paired release consumes that existing keycode exactly once through `consume_handled_release`; it emits no text. Multiple same-field Resets cannot manufacture a receipt already consumed or previously cleared by focus cleanup. A clear-preedit error still propagates after normal cleanup; preserving an already accepted transport press in that case does not trigger a text retry or grant a correction.

The recency predicate is a bounded inference from engine state, not a client ACK proving why Reset occurred. It carries all already-pending handled keys in that same engine rather than recognizing a second physical gesture. That distinction is explicit and acceptable: no new ticket, state owner, timer, queue or correction authority was introduced. Physical client confirmation remains necessary.

## R2: resolved by strict transport-oracle migration

The diff changes ordinary surrounding-capable Space press/release expectations and adds exact one-Space CommitText assertions. The migrated predecessor, owner, epoch, revision, delayed exact receipt, contradiction/input-gap refusal, Tab refusal and manual-tail replay assertions remain intact. `expect_legacy_managed_space` checks the exact effect list/order, path, interface, cleared preedit payload and literal Space payload; it cannot silently accept deletion, duplicate commit or fallback.

Shared fixture callers explicitly supply `space_committed=true` for the affected ordinary route and `false` for opaque fixtures. Expectations are not inferred from the production predicate. The original strict native helper remains unchanged, and opaque/terminal ManagedCommit coverage still requires unhandled Space with no text mutation. The known-failure ledger remains empty; its change binds the updated manifest only. Added tests are independently admitted in the manifest. Cargo metadata changes only the candidate version to 1.0.82.

Both no-apply and manual-suppression variants retain their independent security/provenance assertions. Owned-preedit and successful verified replacements return through their existing routes; Atomic remains unchanged. Snapshot-floor clearing and last-commit updates come from the existing managed commit helper. The test covers repeated literal Spaces and the next glyph. Leading Space is also within the actual predicate scope because Space initializes an absent word mode; no separate native leading-Space acceptance was inferred. Include that scope in any subsequently authorized native acceptance.

## Evidence boundaries and remaining gates

`/home/ubu/.cache/lay/development/tab-full-04obkqdi/FULL_GATE.json` has SHA-256 `94f3c55472cdbf7ba8a9c73a0c0ea789c041311ae4c06059954a08d1cf73b249`; its source binding matches this review and the archive is stable. Functional checks are **3047 selected / 3047 PASS / 0 FAIL**: 3011 correctness plus 36 package, including all 710 IME tests. Manifest, architecture and format checks are recorded PASS. The earlier 18- and 3-failure runs remain historical rather than being relabeled.

The full command is still **FAIL**: all 11 performance tests ran, 10 PASS / 1 FAIL. The unchanged library unique-prefix test measured 298466us against its 50000us budget. Lint, remaining desktop compatibility/syntax and release build were not reached. No complete source release PASS exists.

The frozen baseline contrast also fails, 303677us / 50000us; receipt hash `baa4d8633db7e6992f0d30f52dfc9ef01c97076625ce453e8bb75427cc866208`. The candidate diagnostic contrast fails at 299092us, with 298096us in lexical material and canonical/layout/boundary stages 0/70/2us; receipt hash `929ac4c29545e459b94452b29c7846e6767cae4b41fbcd932c37090926f97dd7`. Their hashes, failure excerpts and stable-archive flags were inspected. This establishes a preexisting blocker in the measured baseline/environment, not native latency equivalence or permission to waive the budget. No library/model/warmup/threshold/allowlist change belongs to the reviewed TD133 repair.

Signal FIFO and source tests do not prove CommitText survival/order within a Wayland `done` batch. Changed native text, caret, held/paired keys, focus return, inverses, decoder/icon and performance remain unverified for candidate bytes. Original native **62 PASS / 2 FAIL / 64**, the four passing old-byte diagnostic controls, their consumed grant, installed 1.0.81/2bd and unresolved original attribution remain separate. The release/performance blocker and unreached gates must be resolved through explicit scope control; any exact-byte native successor requires its own authorization. Do not mark DONE or install from this code-review PASS.
