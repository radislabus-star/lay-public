# TD-121 promotion coverage ledger

Status: `OPEN / installed1.0.66, release client5/5 and full gate PASS; physical NOT_TESTED`,
2026-09-07.
This is a source-to-proof worklist, not an execution receipt. It complements
the fixed C01–C30 and H01–H16 contracts in
[the accepted analysis](td121-context-admission-analysis.md) and
[the owning task](../121-preserve-word-across-ime-layout-handoff.md).

Current final candidate `dfeb50e88c8a9170137563ddaf09ae3a44eb8a5441899e3d399baf60fd8e5185`
passed the same five V2 post-ready cases on optimized release bytes. Receipt
`release-1066-final-IFSMv9/client-release-post-ready/receipt.json` SHA256
`80978ba5382d14a29a896eaff0b5d069aa5a17eeadb0815a84e3f08a6a26a9dc`.
Both final changed/full gates passed2646/2646, including IME411/protected7;
lint and architecture passed. Fourteen Cargo binary targets plus sidecar and
receipt transferred with16/16 digest parity. This does not promote mixed
correction, physical GTK/keyboard, installation or loaded-process verification.
The source-to-C/H coverage mapping below is unchanged. Earlier checkpoint:

Diagnostic actual-client checkpoint: candidate
`d9920add9dd7253c327e0debd24384b912ab5dea033e9f7bc98edd6dd61cf88b`
completed the fixed V2 `post-exact-ready` schedule **5/5**, rc 0, with owned
process cleanup clean. Service runtime was 2.722 s. Receipt:
`/home/e/projects/lay-development-runner/release-1066-final-IFSMv9/client-stale-arm-post-ready/receipt.json`,
SHA-256 `c2f86ff3e69ad8ae5c6a2b1435a2584fff67dcc72d449d1be14e09236b0a56b8`;
focused SUMMARY SHA-256
`2184ab00e4577153fee2cb4a76a892b5ca40a5bc0d8e7f7febc32e4eeef059ea`.
The source/controlled checkpoint is **418/418 PASS**: IME 411 plus the seven
protected tests. It includes the exact delayed-native Transfer handler route,
cross-field SourceFree fallback, ownerless SourceFree factory supersession,
and rejection of a superseded acquisition attempting to arm after observer
refresh. The stale-arm regression also rejects wrong target and nonce, leaves
Bridge arming unchanged, and completes the successor once through its marker
to a SourceFree grant. Bounded repair review: **9/10, High 0, Medium 0**. That
score covers only the native-enrichment, ownerless-factory-supersession, and
stale-arm publication repairs.

The five cases are an actual IBus/Lay **protocol-client** result. They are not
physical key injection, a GTK widget replay, desktop-bus evidence, or an
installed final-release proof. Mixed `lом` and reverse `дjv` retention are
measured, but neither case invokes or evaluates a mixed-token correction;
their correction-quality verdict remains **UNKNOWN**. Physical keyboard/GTK
acceptance is **NOT_TESTED**. Final release gates and artifact transfer now
PASS as recorded above; install and loaded-image verification remain
**NOT_TESTED** at that earlier checkpoint. Following the user's direct
installation instruction,1.0.66 is now installed/loaded with independent
process-image parity: [installation receipt](release-1.0.66-installation.md).
Physical/GTK and mixed-token correction coverage are unchanged and not
promoted. The matrix below still states the actual measured scope.

Historical pre-successor checkpoint: 7/7 final-pass regressions and 399/399
full IME tests PASS, 0 ignored/filtered in the full run, 15.49s. All 7 exact
residual test names were
discovered. Full log SHA256
`8e882054a95b4c4146b7ac9dab5cd74db91271303388e917a480b553bbbad185`;
focused log SHA256
`00fe9da1a54de4cab2041f0aa9583bba93105e6d46d15c0a3d0eaefa2ed06678`.
Paths: `/home/e/.cache/lay/td121-remote-sidecar/td121-residuals-full-20260907.log`
and `td121-residuals-green-controlled-config-20260907.log` in that directory.
The real client separately proved full-prefix handoff, then failed correction
on both cold debug and optimized candidates (`prefetch_not_ready`):0/5 complete
cases in each run. Optimized candidate6c480f39 built in3m14s. Warmup completion
and inline exact-certificate refusal reason are not yet independently observed.
Independent pass2 remains the actual pre-repair4/10,H4/M0, not a self-issued
passing score. Four static schedules now have controlled RED/GREEN evidence.
An additional narrow independent closure check requires the user's explicit
exception to the exhausted review limit; request sent, not presumed granted.
At that historical checkpoint, successor binding and all release/physical
acceptance remained pending.

Historical checkpoint before the legacy/final-pass work: adapter22/22 and full IME390/390 PASS are
source-bound unit/controlled-harness results, not the five-case client proof.
Actual private run `td121-private-five.cVZ7BN` completed0/5 cases with candidate
`b7e783753d03950e81ff31aab40f17257ada78e668cdceaff101d3f28124e81c`.
The first legacy Space remained UnknownStart; marker-before-key was not
established by setup. The new word_scope tests use the atomic callback, so
the actual legacy entrypoint and both causal readiness schedules are the
next discriminating proof. See the current checkpoints in
[client evidence](td121-private-client-proof.md#latest-five-case-run-checkpoint--2026-09-07)
and [analysis](td121-context-admission-analysis.md#legacy-client-first-boundary-checkpoint--2026-09-07).
No C/H row is promoted by390 alone; no install/restart/commit/push occurred.

## Denominators and source identity

- Frozen helper/adapter 30/30 and the earlier IME binary 379/379 are historical
  checkpoints. Neither proves the consolidated receive-order, lifecycle,
  acquisition, atomic, or layout changes now being edited.
- The later 9-test run was a focused checkpoint, not 379 plus 9 distinct tests.
- Consolidated discovery found that the runtime adapter's unqualified
  `mod tests;` loaded `context_admission/tests.rs` a second time, not
  `context_admission/adapter/tests.rs`. The earlier runtime harness therefore
  cannot substantiate execution of those adapter tests. The explicit path is
  being corrected and the actual discovered list must be retained before the
  next run. This finding does not rewrite the independently staged helper
  crate's historical 30-test evidence or count duplicated core tests as new
  adapter coverage. The explicit `#[path = "adapter/tests.rs"]` inclusion is
  now installed and compiled with test-only zbus `p2p`/`bus-impl` support.
  The subsequent 383-test harness is a discovery checkpoint, not the final
  accepted denominator.
- The two private-client diagnostics failed before completing any declared
  case. Policy-off observation cannot prove correction or UnknownStart safety.
- Review pass1:3/10,H6/M2; pass2:4/10,H4/M0, both preserved as historical
  pre-correction verdicts. Neither is relabelled as final acceptance.
- Every row below remains `OPEN` for final promotion until its assertion,
  actual entrypoint, exact source identity, command, result, and receipt agree.
  A test name is only a navigation pointer; it is not a coverage verdict.
- C/H rows overlap. Do not add them together or add them to the Cargo harness
  denominator. Report actual expanded scenarios and harness tests separately.

Short source labels below are relative to `src/bin/lay_ibus_engine/`:
`R` = `context_admission/tests.rs`; `A` = `context_admission/adapter/tests.rs`;
`E` = `context_runtime/tests.rs`; `L` = `layout_sync/tests.rs`.

## Current actual/source mapping, 2026-09-07

This mapping records only what the current 5/5 receipt and the bound 418-test
source checkpoint establish. `PASS_ACTUAL_PROTOCOL` is narrower than physical
or final-release PASS. `PARTIAL` leaves the unexecuted clauses in the C/H row
open.

| Executed case or repair | C/H mapping | Current scoped verdict | Measured fact and untested remainder |
|---|---|---|---|
| `same_context_us_to_us_authority_restoration` | C01; H01 US→US; H03 partial | `PASS_ACTUAL_PROTOCOL` | Same canonical context, different real engine path, retained `l`, full `ljv`, one deletion of `ljv`, visible `дом `, and no duplicate prefix. No physical/GTK input claim |
| `same_context_us_to_ru_mixed` | C02; H01 US→RU; H13 partial | `PARTIAL` | Exact retained `lом`, different engine path, and zero deletion in the tracking interval. Mixed-token correction was not requested or scored, so correction quality is `UNKNOWN` |
| `same_context_ru_to_us_mixed_retention` | C03; H01 RU→US; H13 partial | `PASS_ACTUAL_PROTOCOL` for retention | Exact retained `дjv`, same canonical context, different engine path, and zero deletion. This is retention, not correction-quality or physical-input evidence |
| `different_field_unknown_negative_authority_on` | C07; H04 partial | `PASS_ACTUAL_PROTOCOL` | Different canonical context receives no inherited tail, stays `passive:unknown-context`, emits literal `ljv `, and performs no deletion. Identical real GUI/window geometry was not exercised |
| `unknown_start_negative_authority_on` | C19 and C21 partial | `PARTIAL` | With the same-run C01 authority-positive control, source-free `ljv ` remains literal with zero deletion and the boundary closes the unknown word. Tab/manual effects, learning, and next-word rearm were not exercised |
| Delayed native Transfer enrichment | C05 | `PASS_SOURCE_AND_ACTUAL_ROUTE` | R covers same request/nonce, ordered reply/marker, contradictions, and cross-field fallback. A `native_transfer` drives the actual `focus_in_id` handler, preserves and installs the Transfer once, and proves no second Get/rearm. The final 5/5 candidate traverses the repaired lifecycle route |
| Ownerless SourceFree factory supersession and stale-arm rejection | C17 and C18 support only | `PASS_SOURCE`; actual setup restored | Only a current ownerless/activationless SourceFree acquisition may be superseded by a later factory. Old generation reply/failure and late arm cannot revoke or occupy the successor slot. The complete back-to-back C17 and missing/slow/disconnected-bus C18 expansions remain open |
| Protected-source regression group | No additional C/H promotion | `PASS_SOURCE / PHYSICAL_NOT_TESTED` | Seven protected-source tests pass in the 418 denominator. They do not supply C30, real GTK capture/grab/replay, or physical keyboard evidence; those remain `NOT_TESTED` |

The earlier source audit still leaves C12, C17, C24, C25, C28, and C29
without their complete declared release proof. H07 and H09–H15 still require
their declared actual/physical expansions; H09 protocol delivery and physical
delivery must remain separate. No current receipt supplies physical GTK or
keyboard evidence, measured RSS, or latency distributions. These gaps do not
invalidate the scoped 5/5 protocol-client repair result, but they prevent a
final TD-121/release promotion claim.

## C01–C30 closure worklist

| ID | Existing source pointer / proof route | Remaining assertion before promotion |
|---|---|---|
| C01 | E `td121_same_context_transfer_preserves_full_and_mixed_partial_words`; private actual-client US→US | Seeded grant is not a real factory/focus handoff. Prove observed `l`, admitted full `ljv`, one correction, no duplicate prefix in the actual client |
| C02 | Same E test; separate private US→RU case | Preserve full mixed `lом`; record its correction verdict separately from `ljv` |
| C03 | Symmetric private/state route | Explicit RU→US full-prefix retention, not inferred symmetry |
| C04 | A `controlled_p2p_native_marker_has_zero_get_and_ping_echo_returns_token` | Actual native lifecycle and target installation; no compatibility Get |
| C05 | Native-enrichment repair | Delayed native receipt enriches the same completed or pending activation without tail loss, rearm, or a second authority route |
| C06 | R `reflexive_ticket_is_single_use_and_rotates_owner_generation` | Complete same-engine FocusOut/Disable/FocusIn envelope and inserted B/fake negative; exact lease remains separate |
| C07 | R `different_field_transfer_falls_back_to_the_same_receipt_and_marker_contract`; E transfer negatives | Actual different-field client proves no inherited text, guard, or feedback |
| C08 | Reducer/transport foreign-field schedules | A→B→A while Lay remains global irreversibly rejects the original ticket |
| C09 | R `p121_1_d1_held_fifo_rejects_foreign_aba_and_get_alone`; deployed IBus queue evidence | Re-execute actual reducer against held FIFO on final bytes; do not claim real daemon scheduling was forced |
| C10 | Same D1; R `p121_2_d2_actual_zbus_join_holds_ready_marker_for_earlier_stream` | Early Get cannot substitute for the later ordered marker; preserve no-foreign positive control |
| C11 | Factory/foreign cancellation schedule | Factory-created target cancelled by foreign engine cannot consume after return |
| C12 | Bootstrap mode contract | Unverified/false global mode denies transfer while actual original input still works |
| C13 | R wrong-nonce test; A observer cancellation | Missing/spoofed/wrong marker refuses; expired late marker cannot kill a later valid acquisition |
| C14 | Consolidated readiness/seal repair | Real source final-key settlement after target FocusIn; marker-before-seal wakes readiness and transfers latest tail once |
| C15 | A `receive_order_key_blocks_focus_out_seal_until_handler_settlement` | Actual source callback effects and final seal, not only helper counters; unresolved received input blocks bridge authority too |
| C16 | A stale FocusOutId/Reset tests; E late failure/stale grant tests | Late old-owner cleanup/publication leaves all new-owner shared tail, guard, exact metadata, and active path unchanged |
| C17 | Back-to-back lifecycle schedule | E0→E1→E2 with delayed handlers installs only current grant and rejects obsolete frames |
| C18 | R timeout test; detached acquisition repair | Failed Get expires once; late reply cannot revive. Timely completed acquisition must survive idle time before next callback, without prolonging the Get deadline |
| C19 | E UnknownStart Tab/Space tests; authority-on private negative | Observed suffix after missing prefix is delivered once; zero correction, Tab/manual effects, and learning; subsequent boundary rearms only next word |
| C20 | R `unknown_start_is_sticky_until_next_observed_boundary` | Actual Backspace-to-empty followed by letters is still incomplete |
| C21 | E `td121_unknown_start_actual_space_delivers_literal_without_correction_or_feedback` | Actual callback settlement closes old incomplete word before next-word rearm |
| C22 | E `td121_unknown_space_then_boundary_backspace_refuses_tab_completion` | Cross-boundary Backspace uses real pre/post text effects; command-modified Space/navigation cannot manufacture completeness |
| C23 | R rendezvous/eviction/duplicate-key tests; A burst/cancellation tests | Exact final callback ring/queue bounds, no waiter leak or unsafe transfer; counts alone are not measured RSS |
| C24 | A slow compatibility Get regression under construction | Actual ProcessKeyEvent produces literal output before held Get is released; local stamp wait is not a Get wait; total Space allowance unchanged |
| C25 | R owner-replacement waiter test; A cancellation route | Bus epoch/owner loss rejects old receipt and late completion; original input remains usable |
| C26 | A Ping+marker route and actual bridge | Pending earlier IBus invalidation forbids session-bus bridge authority; no cross-connection Sequence comparison |
| C27 | E `td121_atomic_known_commit_and_stale_token_refusal`; TD-120 bridge/atomic regressions | Actual AtomicV1 consecutive transactions across boundary, native-unhandled settlement, stale clone no-write, and no global layout work during discarded preview |
| C28 | Existing material/config/frame regression routes | Reload while acquisition/frame pending does not promote stale frame or incomplete word; identity and lexical authority remain distinct |
| C29 | Actual ManualToggleV3 and daemon disposition | UnknownStart returns NotHandled→Complete(None), with no daemon-buffer or exact-tail fallback |
| C30 | Existing protected exact GTK/terminal tests and smoke | Preserve independent GTK capture/grab/replay and terminal single-commit plans, one detector, one mutation, no duplicate layout/suppression |

## H01–H16 expansion worklist

| ID | Required expansion; final result remains OPEN |
|---|---|
| H01 | US→RU, RU→US, US→US new-object handoff; exact outputs, not text-only fixture |
| H02 | Same-object quick handoff; compare the actual lifecycle route |
| H03 | Same canonical native context/new path; old candidate acceptance invalid |
| H04 | Different context with identical window/app/text; no shared-state transfer |
| H05 | A→B→A and delayed native FocusOutId; new owner unaffected |
| H06 | Lease before/after bounds, delayed callback, and timely acquisition consumed after idle; use controlled time/order |
| H07 | Both modifier orders; manual/modifier intent supersedes pending automatic intent |
| H08 | Popped-before-dispatch versus already-dispatched switch; separate verdicts, no compensation |
| H09 | Letter before/during/after factory transition; protocol delivery and physical delivery reported separately |
| H10 | ManagedCommit width 11 and narrow TerminalPassthrough width 2, independent of app names |
| H11 | SurroundingText on/off, cold native discovery, cached false hot upgrade; global IBus unchanged |
| H12 | Missing/slow/stale context, disconnected bus; input remains responsive without unsafe fallback |
| H13 | Full opposite-layout and mixed tokens, clean English, protected text; no lexical exception |
| H14 | Physical Double Shift ownership, four taps, accepted Tab, auto-undo, exact round trips |
| H15 | Config/material change between enqueue/apply; old certificates denied |
| H16 | Atomic prepare/abort/commit; no live worker/generation side effect before accepted settlement |

## Final acceptance packet

1. Source freeze and final focused/combined IME run; bind logs and test list.
2. Private actual-client corrected-binary run, fixed positive and negative cases;
   log drain after verdict is excluded from measured behavior deadlines.
3. C/H row closure with explicit `PASS`, `FAIL`, or `NOT_TESTED` and exact scope.
4. Fresh-context review pass 2, ≥8/10, High=0, Medium=0. A low verdict is not
   repaired by re-labelling the review or by starting unlimited review rounds.
5. Accepted TD-121 composition successor bound to reviewed source and genuine
   named functional evidence. Preserve immutable TD-113→TD-120 predecessor.
6. Remote canonical manifest/regression and architecture/release gates, then
   verified artifact transfer, rollback-safe install, loaded-image verification.
7. Physical keyboard evidence remains a separate gate; protocol traffic alone
   cannot be labelled a real-keyboard confirmation.

This ledger changes no runtime authority or production process. Architecture
refresh is deferred until the consolidated source/document freeze and will run
remotely under the existing resource guard, not as a local parallel check.

## Recovery checkpoint, 2026-09-06 21:19 UTC

Parent read the actual production epoch-bind path and these two completed
guarded remote logs, retained under
`/home/ubu/.cache/lay/td121-remote-sidecar/`:

| Focused regression | Result | Log and SHA-256 |
|---|---|---|
| `td121_marker_ready_owner_is_installed_before_focus_out_and_disable` | 1 PASS, 382 filtered, 0.01 s | `td121-c17-monotonic-epoch-pass2-20260907.raw.log`; `b03439f550822a9ed4bf4e77a07dd89ebce8111ad2ec0d9dbfc331173857f831` |
| `td121_marker_before_final_source_settlement_promotes_after_deadline` | 1 PASS, 382 filtered, 0.04 s | `td121-marker-before-settlement-pass3-20260907.raw.log`; `f431f70687d8aa701a5403e34b33046089b13bcce4b31c09fe6edf578a97b417` |

The first fixture installs real marker-ready metadata before FocusOut/Disable,
starts shared tail epoch at 41, asserts a strictly newer installed revision,
then rejects that old epoch and accepts the current epoch at the actual exact
suppression-arm predicate. It is not a physical GTK replay or the complete
back-to-back C17 schedule. Production now binds the source-free empty reducer
snapshot once to `shared.handoff_tail_epoch.checked_add(1)`; it no longer
rewinds the externally exposed shared epoch to zero.

The second fixture covers marker-before-final-settlement promotion and the
consumed request no longer blocking the next transfer. These two checkpoints
do not close C01-C30, the final review, or release acceptance. Full command and
final source bindings still belong in the consolidated execution packet.

The first-activation foreign-profile ordering defect remains under repair:
`CreateEngine(Lay)` with no existing owner currently loses the requested
profile, so a FocusIn received before GlobalEngineChanged(Lay) can be rejected.
This is a source-confirmed defect, not a proven explanation of either earlier
failed private-client run. Required proof includes both relative signal/focus
orders without retries, per-key RPC, or a global-profile guess. C05 native
enrichment and C27 consecutive actual AtomicV1 settlement also remain open.

## Compiled candidate checkpoint, 2026-09-07 local time

The repaired adapter namespace passed 22/22 in 0.34 s, including both late
native cases, initial foreign ordering, marker-ready lifecycle, actual native
Backspace and consecutive atomic receipts. It is a subset of the subsequent
full IME suite: **390/390 PASS**, no failed/ignored/filtered tests, 15.61 s.
Parent read the result and verified the full log SHA-256:
`0b5a8f5882efc728f104e3861b0171d80a6f950e515ec168243bfc3ad2464f29`.
Log: `/home/ubu/.cache/lay/td121-remote-sidecar/td121-lay-ibus-engine-full-pass2-native-preimage-20260907.raw.log`.

The standalone debug candidate is
`/home/e/projects/lay-td119-gate-v1/target/debug/lay-ibus-engine`, 160773008
bytes, SHA-256
`b7e783753d03950e81ff31aab40f17257ada78e668cdceaff101d3f28124e81c`.
Parent independently rehashed that remote file and compared Cargo.toml and
the seven changed core/adapter/atomic test paths with local sources. The
local lockfile lacked the direct dev-dependency `futures-lite` list entry;
the narrow diff was applied locally to match the actually compiled remote
lockfile exactly, SHA-256
`45752d3e0c74d712e6ebf6d50923fc3eb38492e34205cfa872e9a171e973b2d1`.
No dependency version changed in this reconciliation. This is selected source
parity, not a claim that the remote staging Git HEAD is the source commit.

Both exact inner commands ran from
`/home/e/projects/lay-td120-121-SUdh2I` under the dedicated resource profile:

```sh
env LAY_RESOURCE_PROFILE=dedicated-20cpu CARGO_BUILD_JOBS=20 RUST_TEST_THREADS=1 CARGO_TARGET_DIR=/home/e/projects/lay-td119-gate-v1/target scripts/lay-resource-guard.sh -- scripts/cargo-guard.sh test --bin lay-ibus-engine -- --test-threads=1 --nocapture
env LAY_RESOURCE_PROFILE=dedicated-20cpu CARGO_BUILD_JOBS=20 CARGO_TARGET_DIR=/home/e/projects/lay-td119-gate-v1/target scripts/lay-resource-guard.sh -- scripts/cargo-guard.sh build --bin lay-ibus-engine
```

Both exited 0. The build log is
`/home/ubu/.cache/lay/td121-remote-sidecar/td121-standalone-debug-candidate-native-preimage-20260907.raw.log`,
SHA-256 `fdfb51fc9c2f9c056be16da87ff819f44c6d7cfa96525db6758afd8ba9a8f9f3`.
The command supplied no release/feature/target/RUSTFLAGS override. Toolchain
reported by the executor: rustc 1.97.1, Cargo 1.97.1, default stable
x86_64-unknown-linux-gnu. The frozen five-case private-client harness received
one serial execution lease, `td121-private-five-20260907T0051-candidate-b7e78375`.
Its result is pending. Source edits are frozen during that run. This checkpoint
does not close final C/H promotion, independent review, canonical release
checks, physical keyboard acceptance, installation or push.
