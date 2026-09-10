**REPAIR_REQUIRED — 3/10. High: 6. Medium: 2.**

Pass 1 reviewed snapshot `/home/ubu/.cache/lay/td121-review-pass1.hOPQUB` against baseline `ad4bf0860cc2a79b3004f03b8018cc8d8cbca005`. Applicable instructions, the owning task, and the complete context-admission analysis were read. Graphify supplied navigation; findings below use frozen source bytes.

This was static inspection only. Reproduction schedules below are derived from code, **not executed results**.

### Findings

**1. High — Ordered receipt does not establish ordered admission authority.**

[adapter.rs:1062](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission/adapter.rs:1062) publishes method stamps without applying their lifecycle transitions or registering received keys as unsettled. Key registration instead happens when `begin_key_callback` executes at lines 477–499. The bridge marker obtains the reducer’s current token at [adapter.rs:1192](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission/adapter.rs:1192).

Receive an input/focus transition, delay its handler, then process the bridge Ping and marker. The fence can certify the old owner although an earlier invalidating event has already arrived. Similarly, FocusOut can seal while an earlier received key has not entered its callback. `admission_token`/`revalidate` also do not reject an unfinished handoff ([context_admission.rs:1141](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission.rs:1141)).

**Minimal repair:** apply bounded lifecycle metadata and register outstanding key observations in receive order. Bind callbacks to the owner/generation at receipt; acknowledge settlement separately. Bridge authority must exclude unresolved earlier transitions. Wire the existing pre-admission-input poisoning mechanism into production—it currently has only a test caller.

**2. High — The ordinary handoff invalidates its own sealed tail.**

[context_runtime.rs:32](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_runtime.rs:32) seals the source epoch during FocusOut. Disable then calls `reset_for_ibus_soft_reset` ([ibus_interface.rs:249](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/ibus_interface.rs:249)), which ordinarily republishes the tail ([state.rs:392](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/state.rs:392)). Publication increments its epoch ([tail_memory.rs:614](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/tail_memory.rs:614)).

For the normal non-exact `CreateEngine → FocusOut → Disable → FocusIn` envelope, seal epoch `N` becomes shared epoch `N+1`; [context_runtime.rs:179](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_runtime.rs:179) rejects the transfer.

Adverse ordering has another failure: if the marker arrives before final sealing, `process_marker` leaves the fence unready. Later sealing does not publish fence readiness or notify its waiter; Disable itself does not call `try_ready`.

**Minimal repair:** preserve the sealed source through lifecycle cleanup without changing its text revision. Propagate readiness when the final prerequisite settles, using the existing notification mechanism. Prove the complete two-engine callback envelope, including marker-before-final-settlement; do not manufacture grants.

**3. High — Rejected and obsolete callbacks still write newer shared ownership.**

[context_runtime.rs:268](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_runtime.rs:268) unconditionally sets `shared.active_path` to the failed engine and clears shared tail and suppression. It does so even when revoking that engine’s owner was refused.

Separately, old FocusOut/Reset/Disable cleanup reaches the unconditional shared publisher at [tail_memory.rs:614](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/tail_memory.rs:614).

Install owner B, then complete a failed activation or delayed cleanup on A. A can steal B’s active path, erase its tail/guard, or invalidate exact-handoff metadata. The reducer’s old-owner rejection does not protect these subsequent writes.

**Minimal repair:** require the matching owner generation and transition identity at every shared publication/cleanup. Obsolete callbacks must finish without shared writes. Failed activation must not nominate itself as active. Include assertions that B’s complete shared state remains unchanged.

**4. High — Completeness follows key labels rather than actual text transitions.**

[context_runtime.rs:365](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_runtime.rs:365) has three related authority defects:

- Backspace passes the **post-delete** epoch to `backspace_crossed_boundary`. Deletion already increments that epoch through [composition_edit.rs:44](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/composition_edit.rs:44), so it cannot match the boundary revision retained by [context_admission.rs:182](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission.rs:182).
- Ctrl+Space can mark UnknownStart as KnownStart although the command-modifier route returns unhandled without inserting a boundary ([managed.rs:47](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/managed.rs:47)).
- Passive navigation clears the observed tail but leaves a previously KnownStart scope known; the non-boundary branch merely preserves it.

An unknown suffix followed by Space, Backspace, and more letters can consequently regain whole-word correction/manual/feedback authority while joined to an unknown prefix.

**Minimal repair:** advance completeness from the settled text effect, retaining the boundary identity needed for deletion. Revoke on navigation and unproven external-edit/input gaps. Preserve the closed word’s authority separately from the next word’s rearm.

**5. High — Atomic proposals retain the previous transaction’s admission identity.**

[atomic.rs:92](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/atomic.rs:92) captures the token **before** settling the prior receipt. The captured token is stored in the next proposal at lines 130–142. The D-Bus wrapper likewise captures `tail_before` before prior settlement ([ibus_interface.rs:70](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/ibus_interface.rs:70)).

Submit a Space proposal, then submit a printable proposal while acknowledging Space. Space settlement advances lineage, but the printable proposal retains the old token. Its later valid receipt fails revalidation at [atomic.rs:235](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/atomic.rs:235), losing observation of already-submitted input.

Additionally, the wrapper settles every native-unhandled result “without input,” so native text boundaries do not receive the ordinary completeness advancement.

**Minimal repair:** settle the prior transaction before capturing the current event’s token and tail. Carry the actual observation outcome through native and submitted settlement. Test consecutive transactions across a boundary through `ProcessKeyEventAtomicV1`, including a subsequent receipt.

**6. High — Layout work uses word lineage as its request identity.**

[layout_sync.rs:46](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/layout_sync.rs:46) changes the local decoder and queues a full admission token. Successful Space settlement then advances word lineage. If the worker runs afterward, [layout_sync.rs:219](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/layout_sync.rs:219) rejects the legitimate layout request, leaving decoder and external layout inconsistent.

The request also lacks a separate request generation. After external activation, lines 229–230 only log completion; conflicting in-flight completion does not revoke text authority. Blocking switch paths remain outside the new request checks.

**Minimal repair:** bind layout work to context, owner, and layout-request generation independently of ordinary word-boundary advancement. Check before dispatch and classify conflicting completion as Unknown without a compensating switch. Retain GNOME’s single activation owner.

**7. Medium — Acquisition waiting and timeout cleanup violate the bounded lifecycle contract.**

[context_runtime.rs:137](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_runtime.rs:137) awaits Get and marker inside the mutable FocusIn callback. The zbus interface lock therefore holds subsequent keys behind the complete acquisition budget, rather than only the permitted local stamp rendezvous.

On marker timeout, [adapter.rs:961](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission/adapter.rs:961) returns without clearing the pending slot. Future fences encounter `Busy`. A late matching marker instead returns `Timeout` from `process_marker`, terminating the observer and disabling admission connection-wide.

**Minimal repair:** keep context acquisition independent of the engine callback lock; let pending keys follow the literal UnknownStart route. Expire and clear only the matching request/fence on every exit. A late expired marker must not terminate the observer.

**8. Medium — Native capability and delayed native enrichment are not integrated.**

[ibus_interface.rs:405](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/ibus_interface.rs:405) still advertises `FocusId=false`, so cold native discovery cannot select the implemented native route.

If delayed FocusInId is delivered, [ibus_interface.rs:124](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/ibus_interface.rs:124) starts another activation. Without a pending transfer, [adapter.rs:848](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission/adapter.rs:848) selects source-free activation, revoking the current lineage and clearing its tail instead of enriching the existing activation. FocusOutId also discards its canonical-context argument.

**Minimal repair:** implement generation-bound native enrichment and context-aware stale FocusOutId refusal, then advertise native capability. Preserve cached-false compatibility without restarting IBus or changing engine names.

### Proof and promotion boundaries

The frozen tests do not close these integration gaps:

- [context_runtime/tests.rs:167](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_runtime/tests.rs:167) constructs transfer grants directly.
- Its key helper at line 65 bypasses the D-Bus callback and completeness settlement.
- [context_admission/tests.rs:308](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission/tests.rs:308) supplies the same boundary revision directly, missing the actual Backspace epoch change.
- [adapter/tests.rs:262](/home/ubu/.cache/lay/td121-review-pass1.hOPQUB/src/bin/lay_ibus_engine/context_admission/adapter/tests.rs:262) proves factory stamp correlation, not the full engine handoff.

The successor must remain **PROPOSED/PENDING**. Its rejection by the acceptance-only source contract is intentional. There is also a checkpoint identity discrepancy: JSON records composition SHA-256 `bb4a9118…1328bb26`, while the frozen file hashes to `9d94b7e7…a6c6694`. Bind the eventual successor to the actual repaired, reviewed bytes; preserve the TD-120 predecessor.

Cargo changes add explicit stream/timer dependencies and align the package version; no compilation claim is made.

**Untested:** all builds/tests in this review; actual-client restoration and physical key delivery; production transport ordering; latency percentiles, CPU/RSS and backpressure; native/cached convergence; bridge and atomic adverse schedules; exact GTK replay, terminal single-commit round trips, and real-keyboard Double Shift. Earlier **379/379** binary tests are historical scoped evidence, not release acceptance. Client restoration and correction quality remain **UNKNOWN**.

Repairs must retain SafetyGate/verifier, daemon-only physical Double Shift, exclusive atomic gesture ownership, and the distinct exact replay routes.

### SHA-256 identities

Whole-file hashes. IME paths below are relative to snapshot `src/bin/lay_ibus_engine/`.

```text
context_admission.rs
aabfe040707c73c0be442656c645ea1d79a398f3ca7396d4d80af953708aa98d
context_admission/adapter.rs
e5e690f010dbf4db628b5b7736603a34b22b9e7d12d01b83d686a6a737b5f3a5
context_admission/tests.rs
d12f045fca4684bf3708a29593f7c64d8fca6a19c47aa6d5babeed799b511a12
context_admission/adapter/tests.rs
58f5afcc959fd5a1317787c2a50d903e27d733905b14c0e508223de72935ee51
context_runtime.rs
8c4c9e6e6183509a1366fb356c7d90fe4d9c27064a8a73904c74e5b321f63414
context_runtime/tests.rs
f4f835e25e6333a4262463a3587cabb6914e8d0de4d6193fb278d65ba542e175
ibus_interface.rs
cf3f8f007d1005cf3a2b756396eaa5dac61f40e7579867a53b011c2546e9304d
state.rs
2f8454dc921f8e6fef2ef1e141bf8a33e9d9431034eb34265cc3d30799a596f2
tail_memory.rs
591883bea449014fa74fdae7249e85e027c88ca1cb9538c92941d8cfe0af3367
composition_edit.rs
1386808b843245312fd419fc97e5252a1af995f39e0fdb76b8e99a8445ceffb2
managed.rs
ed8abbe020f90cac2c177b53e6b3a747b344b4d826090dc72f959bdf6a04aeeb
atomic.rs
7a059d77bf20d5e91cd9e432acdf43318330cba3871ef7dd43f4cc5487928276
layout_sync.rs
e7cfca6c93f5c7e47aa7d6b3f99ac681982b6989d0cf436418a27f8e6a38846a
bridge_actions.rs
88a33da562e4a4ba002f9f0464202ea9ad2a4596b70ba85abed4e8b9414ff018
composition_commit.rs
9d94b7e75a50573a2c6686e38841e2f639cd87f1655643487d6468998a6c6694
```

Other paths are relative to the snapshot root:

```text
Cargo.toml
d620c9473fa286aacbd14759131bbe5684c25c6d8e7a76a8e1b1df7913a38fb6
Cargo.lock
b5eefdc2cdb5f995e8884ed1afaeb2642ecbe659d273658bf9b59a0fec3e26ff
tests/td113_hybrid_source_contract.rs
0ee78a707cbb465a60aab6f3f04fb75b3199dd08e17f6c2b734766b4275a2792
tech_debt/121-preserve-word-across-ime-layout-handoff.md
e69560dac9f7dbf7f884e5ac38ba4a3c0d1b06db1d7464c15ee338e3c4e6d797
tech_debt/evidence/td121-context-admission-analysis.md
d649e439ba9c0820958e1b1d1d3aa2160b694c8e681786b7bc05436cca90b14f
tech_debt/evidence/td121-composition-mutation-successor.json
63c35636788ba0d90e8d2d0c872a422836cf9aeec39276aa9944c51afd76d6b7
```
