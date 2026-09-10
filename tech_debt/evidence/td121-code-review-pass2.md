# TD-121 independent final code review — pass 2

**REPAIR_REQUIRED — 4/10. High: 4. Medium: 0.**

Reviewed frozen snapshot `/home/ubu/.cache/lay/td121-review-pass2.9zOOK0` against its `baseline/` at `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`. Read SOURCE_FROZEN, REVIEW_TASK, copied AGENTS, the owning TD-121 task, accepted analysis, pass-1 findings, promotion ledger, and relevant source/test changes. Graphify provided navigation only. No tests, builds, network, services, installation, graph refresh, Git writes, or prohibited skill commands were run. This report is the only created file.

The findings below are production-code schedules derived by static inspection, **not executed reproductions**. They are actionable correctness defects in TD-121, independent of explicitly pending subsequent release gates. The required threshold of at least 8/10 with H0/M0 is not met.

## Findings

### 1. High — A bridge fence still certifies an owner with earlier unresolved ingress

Primary: [context_admission.rs:1330](src/bin/lay_ibus_engine/context_admission.rs:1330). Related: [adapter.rs:1987](src/bin/lay_ibus_engine/context_admission/adapter.rs:1987), [bridge_actions.rs:383](src/bin/lay_ibus_engine/bridge_actions.rs:383).

`admission_token()` and `revalidate()` test identity but exclude neither `unsettled` keys nor an unfinished focus handoff. Receiving FocusOut opens a ticket without changing owner, revocation or lineage; receiving a key registers it without changing that token. The bridge marker simply reads `admission_token()`. Its consumer compares the same still-valid engine token and local KnownStart.

Failing schedule: establish a KnownStart owner A with committed text; receive FocusOut(A), but delay its handler before it takes the engine lock; process the later bridge Ping and marker; let VisibleTailV3/ManualToggleV3 acquire the free engine lock first. Both token checks pass and the old field's text remains replay/mutation-capable after the invalidating focus event was already received. An earlier received, unhandled key produces the corresponding ordering hole. FIFO observation now exists, but bridge authority does not consume its unresolved-state information.

The existing `receive_order_key_blocks_focus_out_seal_until_handler_settlement` test at [adapter/tests.rs:1168](src/bin/lay_ibus_engine/context_admission/adapter/tests.rs:1168) asserts source-seal refusal, not bridge refusal. Close this through bridge-specific checks of the existing unsettled/ticket state at fence and consumer validation; globally rejecting an engine token during its own current key would introduce another defect.

### 2. High — Native re-focus is mistaken for delayed native enrichment

Primary: [context_runtime.rs:180](src/bin/lay_ibus_engine/context_runtime.rs:180). Related: [adapter.rs:1233](src/bin/lay_ibus_engine/context_admission/adapter.rs:1233), [context_admission.rs:1378](src/bin/lay_ibus_engine/context_admission.rs:1378), [ibus_interface.rs:181](src/bin/lay_ibus_engine/ibus_interface.rs:181).

Whenever an engine still has `context_owner`, native FocusInId takes the enrichment branch and returns. That branch does not distinguish a repeated receipt for the current activation from a new focus activation after FocusOut/Disable. Those lifecycle cleanup paths retain `context_owner`.

Failing schedule: a native owner E in context A receives and completes FocusOut(A), Disable, then FocusInId(A). The last callback succeeds as enrichment, never issues the new marker and never consumes the reflexive ticket or rotates its owner/activation. The ticket stays Pending; a later factory request sees that unresolved ticket and can terminate admission via the observer's Denied path. With FocusInId(B) instead, context equality fails and `fail_context_activation()` returns immediately rather than starting source-free acquisition. The object loses its local owner/scope, so subsequent keys, including Space, cannot establish next-word completeness until another activation occurs.

The two repaired delayed-native tests cover the initial compatibility activation; they do not exercise this real same-object re-focus envelope. Enrichment must be bound to the existing activation's unrevoked lifecycle, with actual re-focus routed through the existing transfer/source-free acquisition contract.

### 3. High — A revoked ready transfer can reinstall KnownStart from the old grant

Primary: [adapter.rs:1529](src/bin/lay_ibus_engine/context_admission/adapter.rs:1529). Related: [adapter.rs:1190](src/bin/lay_ibus_engine/context_admission/adapter.rs:1190), [context_runtime.rs:403](src/bin/lay_ibus_engine/context_runtime.rs:403), [context_runtime.rs:133](src/bin/lay_ibus_engine/context_runtime.rs:133).

Ready-result consumption and `activation_outcome_is_current()` compare only the owner. A normal reducer revocation preserves that owner while changing revocation and lineage to UnknownStart. The ready slot is not cleared by that transition: `refresh_acquisition_fence_ready()` clears a consumed request's fence, but leaves its ready outcome.

Failing schedule: finish a KnownStart transfer A/E0 -> A/E1 through its marker, leaving the result ready before E1's first key; receive Reset(E1), revoking the reducer lineage; complete that Reset handler while E1 still has no installed local owner; then receive and handle E1's first key. Reset cannot clear E0's guarded shared tail. The ready transfer passes both owner-only checks and installs the old tail and KnownStart scope. Installation then fetches the reducer's **new UnknownStart token**. `context_word_is_known()` checks local KnownStart plus token validity, without requiring the scope to equal that token's lineage, so correction/manual/learning authority becomes available again despite the revocation.

Bind ready-outcome validity to the complete existing activation/revocation/lineage identity, not just owner equality, and do not combine an old grant's scope with a newly fetched unrelated token. Prove same-owner revocation between readiness and installation; a test that replaces the owner does not cover it.

### 4. High — Enter erases the evidence needed to revoke completeness on Backspace

Primary: [context_runtime.rs:553](src/bin/lay_ibus_engine/context_runtime.rs:553). Related: [context_runtime.rs:582](src/bin/lay_ibus_engine/context_runtime.rs:582), [managed.rs:59](src/bin/lay_ibus_engine/managed.rs:59), [tail_memory.rs:896](src/bin/lay_ibus_engine/tail_memory.rs:896).

Boundary-crossing Backspace is recognized only from the last character of `tail_before`. Native Enter closes/empties the committed-tail mirror, then `advance_context_word_scope()` explicitly rearms KnownStart for Enter. The boundary is therefore absent from the mirror when the following Backspace arrives. Although WordScope records a last boundary, this path does not use it.

Failing schedule in a multiline client: observe only an UnknownStart suffix; press Enter, which the client applies as a newline while the engine clears its tail and becomes KnownStart; immediately press Backspace, which removes that newline and rejoins the old unknown prefix. The empty mirror makes `crossed_boundary=false`, so KnownStart survives. New letters followed by Space/Tab/manual toggle can consequently be treated as a complete token although they extend the previous unknown word.

The fixed regression covers a Space physically retained in the mirror, not an Enter boundary discarded by the existing field-close operation. Preserve enough existing boundary evidence, or revoke on an unproven Backspace beyond the observed mirror, so this route cannot invent completeness. This is an authority contract failure; a particular lexical replacement is not needed to demonstrate it.

## Confirmed improvements and evidence limits

The changes do address substantial parts of pass 1: keys are registered on receive; source sealing checks outstanding keys; Get/marker acquisition is detached from the mutable engine callback; completed readiness survives idle time; source-free tail revisions use a checked shared successor; shared tail publication checks owner generation; atomic prior settlement precedes the next token/tail capture; native Backspace uses a narrow live mirror update; layout intent excludes word lineage and carries request generation. The protected composition completion now has an UnknownStart guard. The inspected changes retain daemon-only legacy Double Shift ownership and the distinct exact GTK/terminal routes.

Independently read the retained final IME log: **392 passed, 0 failed/ignored/filtered, 15.62 seconds**, including both named legacy callback tests. SHA-256: `a210817599507c8f7ccd94a5a2e8817104bc1ce1298eda4aa90ee7b0edcba4bd`; local artifact `/home/ubu/.cache/lay/td121-remote-sidecar/td121-full-legacy-trace-parent-20260907.log`. Confirmed explicit adapter and word_scope module inclusion. These are 392 harness tests; expanded scenarios and physical delivery are separate denominators. I did not execute them.

Verified all five IME source hashes listed in SOURCE_FROZEN. Also verified composition source SHA-256 `9d94b7e75a50573a2c6686e38841e2f639cd87f1655643487d6468998a6c6694`, mode `0664`, matching its corrected proposed successor. The immutable TD-120 predecessor remains protected. The acceptance-only source test is intentionally not satisfied by PROPOSED/PENDING; this is a subsequent gate, not a false completion claim.

During review the parent reported that the corrected private client's full-buffer representation confirms same-context/new-object transfer of ` ljv` with KnownStart, owner 1 -> 2. The parent reports that restoration then stopped at `prefetch_not_ready` on a cold debug candidate under the private CPU limit. That update was not independently inspected here and does not establish restoration quality, release latency, all five client cases, or physical keyboard behavior. It also does not remove the four static schedules above.

Composition promotion, the C/H evidence closure, canonical/full release checks, graph refresh, artifacts, installation, and physical-key acceptance remain OPEN. No release acceptance, production authority change, or additional review round is declared by this report. Unchanged baseline limitations, including broader Wave quality and unmeasured GUI/physical delivery, were not counted as TD-121 findings; the four findings above are sufficient for the verdict.
