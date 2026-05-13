# Optional multi-tap Shift scope

Status: implemented as opt-in runtime mode, disabled by default.

## Goal

Keep the current double Shift path fast by default, but define an optional
mode where repeated Shift taps choose the correction scope:

```text
2 taps -> 1 word
3 taps -> 2 words
4 taps -> 3 words
```

This must not make default double Shift slower.

## Non-goals

- Do not change `replace_words` semantics globally.
- Do not make `2 words` mean LLM.
- Do not wait for extra taps unless the user explicitly enables this mode.
- Do not enable the mode by default.

## Config

```json
{
  "multi_tap_scope": false,
  "multi_tap_max_taps": 4
}
```

`replace_words` remains the manual/default scope from config. In multi-tap
mode the tap count only overrides the scope for that one correction.

## Runtime contract

Default mode:

- double Shift fires immediately after the second valid tap;
- there is no extra wait window;
- existing `replace_words` is used.

Multi-tap mode:

- after the second valid tap, daemon waits up to `shift_window_ms` for another
  valid tap;
- if a third tap arrives, scope becomes 2 words and daemon waits one more
  `shift_window_ms`;
- if a fourth tap arrives, scope becomes 3 words and fires immediately;
- any non-trigger key cancels the multi-tap sequence;
- an invalid long-held Shift cancels the sequence;
- single-trigger modes stay separate from multi-tap mode.

## Scope mapping

```text
tap_count < 2 -> no correction
tap_count = 2 -> replace_words = 1
tap_count = 3 -> replace_words = 2
tap_count >= 4 -> replace_words = 3
```

The mapping is intentionally capped at 3 because `LayConfig::active_replace_words`
also caps scope at 3.

## Test matrix

FSM tests before runtime enablement:

- 2 quick taps fires scope 1.
- 3 quick taps fires scope 2.
- 4 quick taps fires scope 3.
- 5 quick taps still caps at scope 3.
- slow third tap after timeout fires scope 1, then starts a new sequence.
- normal double Shift in default mode does not wait.
- typing another key between taps cancels the sequence.

Correction tests:

- one bad word: `ghbdtn` -> `привет`;
- mixed pair: `good ntrcn` -> `good текст`;
- three-word mixed tail keeps good context and changes only the bad current
  tail;
- shell-like text and CLI flags stay protected.

## UI

Add it under an advanced/hidden tray section, not next to the main trigger:

```text
Триггер -> Double Shift
Ещё -> Multi-tap scope
```

The label should make the latency tradeoff explicit:

```text
Multi-tap scope, waits for 3rd tap
```

## Recommendation

Ship only after runtime FSM tests are in place. This feature is useful, but it
is allowed to add latency only when the user explicitly turns it on.
