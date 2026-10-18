# Lay 1.0.90

- GNOME and KDE tray menus share daily grouping: enablement; layout and input
  mode; assistance and autocorrection; settings, updates and diagnostics.
- “Проверить обновления” opens the installed updater in a terminal. Progress
  and errors remain visible; build/install requires confirmation.
- Updates use the owning checkout, exclude concurrent runs, refuse divergent
  history before preserving local edits and retain unrelated Ollama data.
- Includes already reviewed serde 1.0.229 and clap 4.6.7 dependency updates.

IME delivery and correction logic are unchanged. Source verification, actual
installation and GNOME/KDE visual acceptance are recorded separately in the
menu architecture document; new native input acceptance is NOT_TESTED.
