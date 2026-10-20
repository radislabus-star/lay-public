# TD-160 — Ordinary typed text and a muted suggestion in every client

Status: OPEN — native presentation FAIL. Priority: P1; user requirement applies
to every application and supported system, including Chrome, terminals, GNOME
and KDE. Functional Tab/Space/Double Shift results do not close this task.

2026-10-20 bounded progress: ordinary desktop-launch Kitty on KDE/X11 now
passes independent typed-prefix/suffix visual inspection on installed engine
`c33f9f506a3805598f5838f67c6341c121e78ea038d3c0465e2444eb663775af`.
Receipt: `terminal-final-benchmark-v1/hint.png`; raw PTY contains `пров` before
Tab, only auxiliary `ерка` is gray. This is a Kitty result, not universal PASS.
Its functional benchmark retains 20 PASS / 3 FAIL, including rapid-five;
keep those verdicts separate. Chrome/Konsole and other platform appearance
requirements below remain open.

## Required behavior

Already entered characters use the application's ordinary text appearance.
Only the unaccepted continuation is a muted gray or translucent hint. The
typed prefix must not become selected, inverted or underlined as a whole by
Lay's composition presentation. The hint never becomes document text on blur,
Reset, Escape or an unrelated key. Explicit Tab accepts the current suggestion
exactly once; editing, cancellation, correction and manual conversion retain
their current contracts. Password and sensitive fields retain their protection.

## Established root and evidence

Owner: `LayIbusEngine` through `EngineOutput` and the existing context-admission
and output contracts. The first failed presentation transition is the client's
rendering of the typed owned preedit, not candidate generation or the gray
auxiliary-label attribute.

* Kitty 0.21.2 reverses the complete preedit overlay and does not forward its
  per-span IBus attributes to the renderer.
* Konsole 21.12.3 stores only the preedit string and paints it with its cursor
  appearance, discarding the individual IBus text formats.
* Chrome maps Pango underline NONE to an ordinary solid composition underline;
  it also supplies a whole-composition underline if no spans remain. Foreground
  attributes alone cannot control this rendering.
* A fresh owned Chrome native case on 2026-10-20 passes functional Tab and the
  following manual conversion controls, while its screenshot still shows the
  entire typed prefix `пров` underlined. This is a visual FAIL.

Remote evidence root:
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Chrome: `universal-hint-prefix-v1/result.json` and `hint-text.png`.
Terminal screenshots and pinned primary-source copies are recorded in
`docs/development/remote-x11-input-2026-10-20.md` and local cache
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/remote-x11-input-20261020/`.
These are dated observations, not a proof for future binaries or other clients.

## Options and recommendation

1. **More IBus color/underline attributes — 2/10.** Small, but the measured
   clients discard or reinterpret them. It cannot satisfy the requirement.
   Do not repeat the same attribute experiment against the same renderer.
2. **Patch themes/renderers separately — 3/10.** Could improve one pinned
   application, but cannot provide the required common behavior and introduces
   installation and theme maintenance. No per-client delivery bypass.
3. **Ordinary entered text plus the existing auxiliary hint — 8/10, recommended
   direction; implementation not yet accepted.** Reuse the existing managed
   ordinary-input path and auxiliary publisher. First prove that ownership of
   the entered text can be retained through existing receipts and executors.
   An unconditional early CommitText is not an implementation: it gives text
   to the client and can remove the authority needed for Tab, correction,
   Backspace and Double Shift in clients without SurroundingText. Any change
   to ownership or commit timing needs its own explicit canon decision before
   implementation, and a bounded native RED/GREEN proof.

## Bounded Kitty result — 2026-10-20

KDE/X11 ordinary desktop launch on engine e6d1ea29 is native/visual PASS:
`terminal-kwin-manual-controls-v1` has 3/3 functional rows, and independent
inspection of its before-Tab screenshot confirms ordinary `пров` and gray
auxiliary-only `ерка`. This reuses the existing canonical focus qualification,
live admitted field receipt and native terminal path; it does not change
committed-text ownership or add a client acknowledgement. The two missing
qualification transitions were before the first letter and before a manual
gesture after engine activation. See the owning presentation document and
`2026-10-20-kde-ime-terminal-focus.json`.

This closes only the Kitty mechanism on the exercised platform. The task stays
OPEN for the universal contract; Chrome/Konsole and other platform visual
failures are not relabeled by this result.

## Minimum next implementation boundary

Trace the existing ordinary managed input, owned preedit and committed-tail
append/replacement paths for one snapshot-capable and one opaque client. Write
down the existing input token, focus, field, word, revision and proof available
at each transition. Identify the first authority lost by ordinary CommitText.
Do not invent a widget acknowledgement from a successful signal write or local
tail mirror. UnknownStart may support an already authorized suffix append; it
must not silently become a complete-word deletion capability.

Select the smallest change in those existing owners. Preserve decision and
verifier authority. Do not add an overlay process, parallel text owner, client
name branch, polling loop, delayed replay or a second fallback route. Keep the
atomic protocol unchanged unless its own separate, versioned decision and
tests require an extension. Do not remove owned preedit globally and adjust
the tests to conceal lost behavior.

## RED/GREEN and acceptance

Before the runtime change, add semantic tests for ordinary typed-prefix effects
and separate hint effects in the affected owner, then demonstrate the actual
failure. Cover Unicode scalar ranges, deletion/backspace, middle-word caret,
selection, Escape, focus/Reset/disable, engine handoff and stale candidate work.
Assert exact final text and no accidental hint commit. Keep existing correction
and manual replay proofs; no test may infer visual PASS from payload attributes.

Run native inspections with the exact candidate bytes in GTK Entry/TextView,
Qt line/plain/rich fields, Firefox and Chrome's ten field types and modes,
Kitty, Konsole and WezTerm where available, on GNOME and KDE. Start with one
small diagnostic client per ownership mechanism; run complete affected
matrices only after those pass. A platform or client not exercised is
NOT_TESTED or BLOCKED with a cause, not a universal PASS.

For each visual observation retain a screenshot immediately before Tab, the
typed text, visible hint, actual active field, binary/configuration hashes and
an independent visual verdict. Measure Tab and Double Shift median/p95 using
the existing timing protocol; do not grow deadlines to obtain PASS. Report
functional and visual counts separately. Complete source gates, update the
owning architecture document and graph, then install only on the authorized
remote test runtime. Local user installation remains through the menu updater.

## Done

All exercised native clients satisfy the prefix/hint appearance contract with
no regression in their retained functional proof. The release and updater
point to the tested bytes. Remaining platform gaps are explicit; a source-only
test, a gray suffix beside a reversed word, or functional PASS is insufficient.
