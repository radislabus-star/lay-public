# Issue #81 — explicit selection conversion

Baseline `c1f098936a16c98867068137d689474fece25714` (Lay 1.0.127).
An actual GTK3 Entry in private Xvfb copied the selected `[jxtim ddtcnb`.
The CLI exited successfully with a status message. The clipboard read-back
after process exit returned no text. The session had no clipboard manager.

The CLI now reads X11 PRIMARY only on the explicit `--selection` invocation.
The pre-existing dictionary projection converts the selected text. The
standard packaged `xclip` keeps the resulting CLIPBOARD selection available
after the command exits. The user initiates paste. Failure to find `xclip`
occurs before a write. Both options remain separate from keyboard observation
and the IME verifier.

The scoped build passed the following checks in a real GTK3 Entry on X11.

| Case | Observed result |
| --- | --- |
| English layout to Russian over two selected words | `до хочешь ввести после` |
| Russian layout to English over two selected words | `left Hello  world right`, with the two spaces retained |
| Copy and use existing `--clipboard` | Converted text available after the CLI exited, with exact selected-range paste |
| PRIMARY unavailable | Error and existing clipboard preserved |
| Conflicting input options | Usage error and existing clipboard preserved |
| `xclip` unavailable | Actionable error and existing clipboard preserved |

The actual-client program used the native selection, clipboard read-back, and
the GTK paste action. It verified that the field still held the original text
before paste. This evidence covers GTK3/X11. Other clients depend on whether
their X11 session exposes the selected range as PRIMARY. The reported sporadic
layout change still lacks an input sequence and the reporter's configuration.

[Preflight and rollback](../architecture/decisions/2026-11-21-explicit-selection-clipboard.json)
