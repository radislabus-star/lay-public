# Lay 1.0.92

- Unaccepted IME hints use muted gray `#888888` in inline composition and the
  IBus auxiliary panel used on KDE.
- GNOME's auxiliary panel keeps its system color and displays Lay hints at
  approximately 50% opacity. Other IMEs keep their original appearance; switching
  away from Lay or disabling the extension restores the previous opacity.
- Typed and committed text keep normal styling. Input delivery, Tab acceptance,
  Space correction, Double Shift, models and learning data are unchanged.
- GNOME extension archives include all imported JavaScript modules. Package,
  extension and panel version metadata agree on 1.0.92.

Remote source verification: 910/910 IME and tray tests, including auxiliary
publication/clear attributes and Unicode ranges; 6/6 GNOME opacity and lifecycle
controls; helper module loaded in GJS. Final architecture and package evidence
is recorded in the [presentation owner](architecture/ime-hint-presentation-2026-10-19.md).
The version bump changes metadata only and does not reopen functional testing.

This release is source-verified. Fresh physical rendering in KDE and supported
GNOME sessions remains NOT_TESTED. The previous 1.0.91 native matrix is retained
as historical evidence, not rerun or regraded for this presentation change.
No local installation, input-source change or global IBus restart was performed.
Update through the existing **Проверить обновления** menu.
