# WezTerm: native lifecycle and absent visible continuation

Status: OPEN_NATIVE. Priority: P1. Owner: existing WindowInteraction and KDE
presentation adapter; no new input owner or delivery route. Follow-up from
the original shared-native release group, not closed by publication100.

## Measured facts

Source100 closure9e1486e38397f45e824ffc4471649ce8e90f1b8fbc09abc0bf45f2347f1a426e;
loaded engine2662296 SHA443576475a38806da3c7e3e433f91869a57ffe3855374a20fc80f27b844c3811.
First fresh terminal sequence Kitty5PASS,Konsole5PASS,WezTerm5FAIL. Ordinary
WezTerm text reaches the raw PTY, Double Shift leaves it unchanged, Tab inserts
literal tab. Raw CROOT/release100b2-terminal-controls/wezterm-{basic,five}/result.json.

One bounded hint-only diagnostic with logging later passes functional Tab;
this does not replace the five original failures. Its actual IME update emits
only4characters, typed prefix, cursor4, and the saved hint frame shows a green
typed prefix without continuation. Tab acceptance is not visual acceptance.
Raw CROOT/release100b2-wezterm-hint-diagnostic/{engine-trace.jsonl,terminal/result.json,terminal/hint.png}.

WezTerm20240203-110809-5046fc22 runs on X11 with XMODIFIERS=@im=ibus. No user
WezTerm config exists in the examined home/private XDG config paths. The actual
protected IBus has --xim; the existing ibus-x11 helper is on display71 and the
same session bus. Permanent missing-XIM and disabled-user-config explanations
are not supported. Official protocol description:
[WezTerm use_ime](https://wezterm.org/config/lua/config/use_ime.html).

## Next bounded work

Separate intermittent lifecycle/input readiness from deterministic presentation.
For the former retain the first failing received-callback/admission transition;
do not attribute the entire five-case final symptom to a guessed cause. For
the latter locate the existing preview policy that emitted only the typed
prefix despite an available completion. Reuse neighbouring KDE terminal
adapters and the common owned-word contract; preserve GNOME and Kitty/Konsole.

Acceptance requires fresh real client input, repeat/negative schedules, visible
continuation with ordinary typed letters, exact Tab result, loaded bytes and
latency coverage. Do not force a test-only IM module, add sleeps or weaken
source/owner/verifier guards to make the existing row PASS. Keep original
raw verdicts and distinguish whole grid, short controls and diagnostics.
