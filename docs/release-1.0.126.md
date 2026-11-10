# Lay 1.0.126: letter-fragment continuation in native GNOME terminals

With letter-fragment autoflip enabled, a native GNOME terminal could change
`ghbd` to `прив` before Space but type following keys in the old layout.
The new fragment caller now retains the existing writer's layout completion.
It also uses the existing scoped output accounting after its key callback has
finished, so the target engine receives the whole word. Manual Double Shift
therefore converts `привет` to `ghbdtn` instead of just changing its suffix.

The production change is confined to the fragment caller in committed_tail.rs.
It reuses the existing verified writer, output scope and layout worker. No new
text-delivery route, reducer exception, timer or owner is added. The detector
remains letter-triggered; ordinary Space keeps its existing functions. Managed
GNOME and KDE branches preserve their current behavior.

Actual Kitty checks: two-word conversion in both directions, subsequent
physical letters, eight complete manual returns, fast typing, and fragmentsOFF
with ordinary Space. All four scenarios/19 ordered targets pass. Configuration,
learning data and installed125 runtime are restored after each temporary probe.
Focused IME source gate:1182/1182 PASS. These are scoped results, not universal
application acceptance. Detailed evidence and rejected variants are in
[the owning record](development/kitty-fragment-mode-2026-11-10.md).

The complete release gate passed 3,652 correctness/package tests and 11
performance tests. Headless Python: 475 passed and 12 environment-dependent
skips. All ten installable release binaries were built and retained from Cargo's
actual executable records. Locked checks with Rust 1.88.0 passed in both default
and research-tools modes; installer, architecture and lint checks also passed.
Independent read-only review: 9/10, no required fixes.
[Verification receipt](development/evidence/release-1.0.126-verification.json).

Three terminal-state observations measured 10.426–11.252 ms from the complete
wrong-layout prefix to its replacement. This excludes full hardware-to-pixel
latency; it is not a claim about all windows or a comparative speedup.

The earlier installed/loaded version mismatch was corrected with the existing
full-restart action. Updating from the menu installs new files while preserving
active input; use the menu's full restart to load the new IME when appropriate.
No candidate binaries are permanently installed on the primary desktop by
this release task.

Historical complete KDE browser matrix remains Firefox143PASS/51FAIL/26BLOCKED,
Chrome186PASS/8FAIL/26BLOCKED, total329/59/52 of440. This change does not claim to
repair or rerun that matrix.
