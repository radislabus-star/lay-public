# Native application IME acceptance

Run on the authorized remote machine, in newly created owned test windows and profiles. Freeze exact IME/daemon hashes, source archive, configuration and app versions before input. Preserve normal client transport selection; never change IME routes or add app-specific production bypasses to obtain PASS.

## Scope

Cover GTK3 Entry/TextView, Qt LineEdit/plain/rich editors, an actual text editor, terminal clients including WezTerm, and an actual Telegram message editor after user authentication. Qt fixtures do not certify Telegram; GTK fixtures do not certify WezTerm. Unsupported or unauthenticated real clients are BLOCKED, not substituted PASS.

## Cases

Use real evdev input and the same physical Double Shift cadence as the browser standard. Retain all intermediate results for both directions and five consecutive gestures; probe the next letter and restore only the owned test field. Include short, long, mixed-case and English-first words and a preserved prefix. Include closing Space, clean preservation, moved-word boundary, one-character Backspace, selection, interior caret, focus return and completion where the actual client provides that capability.

Record the actual caret and selection before each action. Focus-return assertions must use the client observed caret; do not assume that focusing restores an end caret or move it silently. A completion requires an observed genuine current candidate, never an invented accepted text. Unsupported selection/completion/observation is BLOCKED.

For WezTerm, check actual inline preedit and candidate visibility/position at the caret, before/after Space and after focus changes. Capture an actual image as visual evidence. A PTY receipt proves committed input only; an IME output record is not client rendering proof. Use a benign owned terminal input fixture, no shell commands as test text.

For Telegram, use a new test profile and an authenticated test account. Keep test input in a draft; never press the send control/Enter or send messages. Do not overwrite an existing draft. Authentication belongs to the user. Login/search fields are reported separately and do not certify the message editor.

## Evidence

Retain PASS/FAIL/BLOCKED/NOT_TESTED per stable app/case ID, full denominators, exact native input identities and actual visible/consumer receipts. Measure write-return to exact visible target latency with required/measured/missing coverage, median, p95 and maximum. Kernel release timing remains NOT_MEASURED unless directly captured. Screenshots without a clock do not prove latency.

Keep previous Firefox/Chrome complete receipts separately. A result from other apps, a source test or a partial native run never replaces the mandatory ten-field browser matrix. Preserve all failures and stop an unchanged failed scenario rather than retrying until it happens to pass.

Use the [same fast localization procedure](browser_ime_standard.md#быстрая-локализация-отказов): check actual target APIs/setup without keys, group by the first failed transition, and run one mechanism witness with necessary regression controls before the full fixed matrix. Save the failed condition and bounded payload from existing reads; add no diagnostic RPC to measured input. Keep setup, observer and product failures and their timing separate.

## Plasma coverage

Record Plasma/KWin, Wayland/X11, renderer, IBus daemon and actual client-library versions separately. A VKMS/QPainter Wayland run proves that declared native setup only. Freeze ordinary US/RU input and Space-commit baselines before attributing a transport/setup failure to Lay.

Qualify device hotplug in the actual test isolation. On systemd249 an unprivileged
bwrap user namespace maps the host udev sender to UID65534 and libudev discards
post-startup keyboard notifications. A daemon emission log or kernel event
alone cannot establish KWin opened the new keyboard. Preserve filesystem/data
isolation while retaining the host user namespace; if mount setup needs a
privileged helper, drop the private session to the ordinary test UID/GID with
zero permitted/effective/inheritable/ambient capabilities before DBus, KWin or
clients. Assert the actual identity/namespace and temporary directory/socket
access. Temporary ancestor directories must be searchable and private `/tmp`
must have standard1777 permissions; never chmod mounted user data. Qualify a
same-device zero-key control and an original native roundtrip before expanding
the matrix. The TD1592026-10-18 owning-document entry preserves this evidence.

In a private direct-IBus test session, leave `IBUS_USE_PORTAL` absent: setting it to `0` still selects the portal in IBus. Do not alter the user's global environment or replace the host IBus to repair a test. Capture unexpected engine exits; distinguish them from deliberate shutdown after actors finish.

Preserve a failed empty-field or caret prerequisite as its raw verdict and report that the intended operation was not exercised. Missing client accessibility/focus prevents functional coverage; a single setup BLOCKED does not fill a client grid. Record tray registration and visibly observed production-menu labels separately.

The 2026-10-18 baseline found that the daemon's exact focused-window lease requires GNOME. Do not bypass that guard on KDE or use the test observer as runtime authority. In particular, a Wayland `windowId` of zero and an application PID cannot distinguish two windows. KDE Double Shift acceptance requires a production KWin identity provider under the existing focus contract, followed by new native evidence.

Check the installed editor's supported launch arguments before opening a test window. A PID can belong to an exited zombie: verify a non-zombie state, UID, start tick, executable identity and owned cgroup before native input. On the tested KWrite 21.12.3, `--new` is unsupported. Select the unique SHOWING editor Text or Terminal accessibility node, never the first hidden editable node in the application tree. A focus request's return value alone is insufficient: require the actual FOCUSED and SHOWING states and the owned foreground window before keys. KWrite may already be focused while `grab_focus()` returns false.

Keep the first failed setup receipt immutable. A later finite editor follow-up may fill its NOT_TESTED cells only in an explicitly cumulative report with verified matching candidate bytes and separate receipts. The 432-cell KDE functional grid does not include terminal visual preedit, X11, hardware rendering or successful production-menu interaction; record those limits separately. Diagnose IME context-authority refusals separately from the daemon's later GNOME-only exact-window lease.
