# TD-161 — Retain the authorized manual suffix across a layout handoff

Status: OPEN. Priority: P1. This is a functional follow-up to the accepted
Kitty appearance correction; it does not reopen that visual result or TD-160's
separate universal-rendering requirement. No runtime change belongs to this
recording step.

## Reproduction and established first failure

Remote KDE/X11, RFB input, ordinary installed Kitty desktop launcher, owned
raw PTY without a shell. Installed engine SHA-256
`c33f9f506a3805598f5838f67c6341c121e78ea038d3c0465e2444eb663775af`;
daemon `1802bb5e816ebb5e8eac3d8de7baa26ed6702f00b4dec7145b7f2ac47ba5b6dd`.
Type `привет` without Space, then five complete Double Shift gestures at
measured 162–164 ms cadence. Required visible transitions alternate
`ghbdtn`, `привет`, `ghbdtn`, `привет`, `ghbdtn`.

Only the first conversion is observed. The isolated diagnostic confirms all
five manual RPCs arrive: one `complete` and four `not_handled` with
`reason=context_authority`, `bridge_token_live=true`,
`manual_toggle_allowed=false`, `manual_refresh_allowed=false`. The first edit
uses existing `observed_terminal_suffix` / `terminal_erase_commit` authority.
The gesture recognizer is therefore not the first failed owner. Double Shift
does not use the Caps/single-trigger debounce, so changing it is unjustified.

Receipt root:
`/home/e/projects/lay-development-runner/kde-install-control-20261020`.

- `terminal-final-benchmark-v1/result.json`: 20 PASS / 3 FAIL; only one of five
  active-word transitions observed. Preserve the two following failed rows.
- `terminal-rapid-diagnostic-v1/result.json`: independent one-case FAIL,
  temporary `debug_action_log` restored, same visible failure.
- `terminal-rapid-diagnostic-v1/engine-trace.jsonl`: initial edit at records
  1970–1979; four authority refusals at records 2046/2082/2118/2154 (zero-based).
  The trace also contains earlier buffered fixture records; use this segment,
  not whole-file counts, for the five-gesture conclusion.
- `all-native-modes-final-v2/kitty/result.json`: fresh ordinary controls 3/3
  PASS, including the moved-boundary pair. Failures after the benchmark's
  failed burst do not independently establish a broken fresh pair.

## Existing owners to inspect

`src/bin/lay_ibus_engine/window_interaction/authority.rs` checks the live bridge
token and manual authority before dispatch. `context_allows_manual_toggle()`
in `window_interaction/observation.rs` permits the terminal's observed suffix
only with a current matching scope/token and sufficient observed characters.
Follow the first edit's layout activation, factory transfer and admission
reducer; locate exactly which already established suffix capability is lost.
The trace establishes refusal at this boundary, not yet a specific defective
transfer statement. Do not manufacture a complete-word acknowledgement from
a local string mirror or a successful CommitText signal write.

## Options

1. **8/10, investigate first:** retain the existing, exact authorized suffix
   capability through the existing authenticated layout handoff, only when
   its field, focus, owner, revision, source text and output result still
   match. Reuse current reducers and transfer metadata. No second owner,
   polling, delayed retry, fallback path or widened UnknownStart authority.
2. **3/10:** require Space or an additional user key before another toggle.
   It avoids this state but fails the requested consecutive-toggle contract;
   acceptable only as a documented interim limit.
3. **1/10, reject:** bypass `context_authority`, increase a lease/debounce,
   use client-name exceptions or replay from the daemon after IME refusal.
   None establishes permission to delete the current text.

## Minimum implementation and acceptance

First add one regression through the existing production factory/handoff and
manual bridge: establish the actual observed terminal suffix, perform the
first conversion and real layout transfer, then request the second conversion.
Obtain semantic RED at the lost authority. Assert text, disposition, scope,
token and deletion proof, not only a producer identifier or helper boolean.

Retain stale field/focus/revision, reset, selection, sensitive input, mismatched
source, failed/partial output, expired handoff and intervening-key refusals.
Cover active-word and after-Space directions, five consecutive gestures and
the first ordinary key afterward. Do not relax the verifier, resource limits
or existing native expectations. An explicit canon decision is required for
any changed capability propagation.

Run the focused owner and affected IME/daemon contracts remotely under the
existing guard, then one exact-byte owned native case per independent mode.
Assert every intermediate transition exactly once, not just odd/even final
text. Separate DOM/preedit/PTY visibility where relevant. Preserve all failures
and measure successful transitions separately from missing transitions. Run
the ten-field browser protocol if the shared manual authority changes; do
not repeat unrelated broad gates without a changed dependency or concern.
