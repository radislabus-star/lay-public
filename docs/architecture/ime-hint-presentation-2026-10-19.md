# Muted IME hints in KDE and GNOME

Requested behavior: a suggested, unaccepted suffix uses muted gray `#888888` or,
where the system controls its color, a translucent presentation. The user
explicitly accepted translucency during implementation. User-entered and
committed text retain their normal styling. This is presentation under C06;
it adds no text, visibility, acceptance, input-source or mutation authority.

## Source and first failed transition

Baseline: public v1.0.91, commit
`513fb674d6b820adb9532c1c4c1ae91193009801`, checkout
`/home/ubu/projects/lay-browser-final-fixes-20261018`, styling branch
`codex/ime-muted-hint-20261019`.

Inline suffixes already carried IBus foreground `0x888888` and a suggestion-only
underline. Legacy auxiliary hints used plain `make_ibus_text`, losing color at
the payload builder. The GTK IBus candidate panel applies auxiliary IBusText
attributes. GNOME Shell's auxiliary callback copies only `text.get_text()` and
discards those attributes, losing color again at the shell renderer.

Verified read-only against installed local GNOME Shell 50.1 resources
`/org/gnome/shell/ui/ibusCandidatePopup.js` and
`/org/gnome/shell/misc/ibusManager.js` in
`/usr/lib/gnome-shell/libshell-18.so`. The popup belongs to IBusManager, not the
keyboard indicator. Primary sources:
[GNOME 50.1 candidate popup](https://github.com/GNOME/gnome-shell/blob/50.1/js/ui/ibusCandidatePopup.js)
and [IBus GTK candidate panel](https://github.com/ibus/ibus/blob/main/ui/gtk3/candidatepanel.vala).
Source inspection is not physical-client acceptance.

## Minimal implementation

- One Rust foreground constant supplies both inline and auxiliary hints.
  Auxiliary ranges use Unicode scalar counts, including non-BMP characters;
  empty hints have no attributes. Plain document and property payloads remain
  unstyled. Output calls, ordering, visibility and delivery routes are unchanged.
- The existing GNOME extension dims only IBusManager's auxiliary label when
  the actual engine is exactly `lay-ime-ru` or `lay-ime-us`. Read-only engine
  signals update opacity to 128/255 (approximately 50%), preserving the system
  theme's color and any already lower opacity. A text notification covers
  asynchronous initial engine discovery. Other engines and bus disconnect
  restore the original opacity. Disable disconnects listeners and restores only
  the opacity still owned by Lay. Missing private display actors leave extension
  initialization usable.
- The extension loader includes the helper in its existing fresh-module
  directory. The extension ZIP includes all JavaScript modules; shipping only
  the former loader/implementation pair omitted its imported helpers.

No polling, theme override, new process, input interception or change to
Tab, Space, Double Shift, focus, candidate generation or learning is added.
KDE uses the existing IBusText display contract. GNOME's private display actor
is version-sensitive and requires compatibility checks when Shell changes.

## Verification and boundaries

Remote source gate completed: 910 PASS / 0 FAIL (892 IME binary tests and 18 tray
tests), 156.1 seconds of recorded remote execution, including auxiliary publication/clear payloads,
unchanged typed-prefix styling and Unicode ranges. Exact receipt:
`/home/ubu/.cache/lay/development/run-t8n_7c4y/RESULT.json`.
GNOME opacity and lifecycle controls: 6/6 PASS; module load in GJS: PASS.
Receipt `/home/ubu/.cache/lay/development/browser-final-fixes-20261018/ime-muted-presentation-source-one/RESULT.json`
retains these successful components and the separate canon failure: the remote
mirror's old Git HEAD incorrectly selected historical decisions. No historical
record or guard was weakened. The final scope used the exact local diff against
the pinned 1.0.91 base and passed the unchanged decision and link validators.

Final receipt:
`/home/ubu/.cache/lay/development/browser-final-fixes-20261018/ime-muted-presentation-source-two/RESULT.json`,
recorded at `2026-10-19T13:48:45.809241+00:00`:

- Architecture canon, its refusal controls and the canonical AST-only graph
  update/guard: PASS. No rule, historical decision or baseline was weakened.
- Extension ZIP: PASS; version `1.0.92`, all nine JavaScript modules present
  exactly once and byte-identical to source. ZIP SHA-256:
  `9cf92adfc6f2dd1a5e8cb17a62e6aa49e6c33859682737ecf7edd4183db1963d`.
- GNOME 45.0, 46.0, 47.0 and 50.1 upstream manager/popup sources expose the
  display actors and engine signals used by the helper. This checks source
  compatibility, not successful rendering in those sessions.

The [portable source receipt](../../tech_debt/evidence/ime-muted-hint-source-20261019.json)
pins the changed runtime/test bytes, result hashes and component verdicts.

1.0.92 publication metadata aligns Cargo, extension and panel versions; it does
not change the tested runtime logic or reopen functional verification.

No local installation or global IBus restart is authorized by this change.
The user will install through the update menu. New presentation bytes have not
yet been physically accepted. The previous accepted native matrix remains
391 PASS / 37 FAIL / 4 BLOCKED over 432 KDE cases, including Firefox and Chrome
150 PASS / 10 FAIL each; it must not be relabeled as testing this styling change.
Fresh GNOME acceptance of the same runtime remains NOT_TESTED. The remote
worker currently has GNOME Shell 42.9, outside the extension's declared
45/46/47/50 support; a mock or older Shell render check cannot certify a full
supported GNOME extension session.
