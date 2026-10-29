# KDE live input: Tab and Double Shift unavailable in the user's Firefox

Resolved within the measured Firefox scope by
`kde-x11-grab-focus-map-2026-10-29.md`: new daemon2374910/ae77e705..., same
IME/IBus/tray/Firefox,10 native scenarios PASS and62 checked gestures. Base1.0.111,
no release/push. The earlier7PASS/2rich-setupBLOCKED receipt remains; two later
rich checks close those preparation slots without product changes. The90s
passive capture contained no user keys and restored debug config. Earlier
diagnostic text below is historical, not a pending implementation or prompt.

One active diagnostic, no source/runtime implementation change. User reports
both Tab and Double Shift unavailable across remote KDE windows. Earlier owned
fixture results do not certify the currently open Firefox.

Canonical base: b835ca478198d6cb765a2246329d5646966c118c in
/home/ubu/projects/lay-dependency-updates-20261027, version1.0.111 with the retained
unpublished scoped patch. Remote DISPLAY:71, FirefoxPID988548.
Loaded IME2067215 SHA bad527220d1034e51444c567770511794f6b3c43c439599484698b84651d9bac;
daemon1894209 SHA15fcc650f2aad3f3ea30d4152011adcd07971ae500f1737db96b3732ae8b715e.

Read-only facts: XInput2 ready, remote device5/master3. Firefox, daemon, IME and
IBus use the same GUI bus/display and GTK IBus. The original Firefox compatibility
library is mapped. KDE layout index1 is Russian, but IBus selects lay-ime-us.
IME is focused and returns passive:daemon-word-buffer. This mismatch is a
proven runtime condition, not yet the proven cause of both reported failures.

C09 existing owner: KDE/IBus layout stack. First repair experiment uses only
the existing user source control: with OwnsActiveText=false, retain KDE's selected
Russian layout and select lay-ime-ru through IBus. No daemon, engine, browser or
tray restart, no installation, no global desktop settings or GNOME change.

Next bounded acceptance uses an owned window in the SAME Firefox process/profile,
native RFB keys and actual AT-SPI text/caret. Check hint/Tab, then one active word
roundtrip; record exact layout/engine and native focus. Never clear existing user
fields. Distinguish runtime alignment from a confirmed functional repair.

## Measured results

`CROOT/kde-live-input-recovery-v1/alignment.json`: IBus selected lay-ime-ru,
KDE index1 retained. No process restarted, original active window unchanged,
config SHA6e62eab3c278c733fedb47e45ef6306105a3700222cf9f7a4e5bff4bd92264d1
retained. Alignment alone did NOT fix Double Shift.

`native-same-session-v1/result.json`: owned standard-page window67109872 in
the SAME existing FirefoxPID988548, not a WebDriver/private profile. RFB input,
actual AT-SPI text/caret/focus. Four rows:2PASS/2FAIL. Tab passes in input and
textarea (`пров` to `проверка `),18.043/18.364ms release-to-stable-target-first-seen,
polling included. Active first Double Shift fails in BOTH fields: actual text
remains `привет`, actual window/field focused. Each trace is complete. No new
all-client acceptance; held keys empty, config exactly restored.

This failure is NOT the earlier row10 bridge-admission refusal. Here Reset
receipt confirms, ManualToggleV3 succeeds withstatus3 (DelegateExactImeTail),
and the replacement never reaches the field. Next causal boundary is existing
daemon exact-tail capture/execution, not gesture detection or initial admission.

Wire repeatsv2/v3 are PREPARATION BLOCKED: native field focus/caret was not
confirmed before input; no Double Shift was emitted. v2's raw outerFAIL is a
tool classification issue, not another product failure. Both raw receipts
retained. Screen shows textarea focused after those clients disconnect; the
transient cause is not established. Do not fix runtime code from this gap.

User standing instruction says call them if repeated work stops progressing.
One90second passive wire capture is armed in `passive-user-gesture-v1`, no keys,
window switches, source installation or restart. Async user prompt asks for one
Double Shift on the owned textarea's completed `проверка `. Capture restores
the prior debug configuration. Await the actual reply/capture; time is not an
answer. No agents/Harness, no push/release or GNOME mutation.

Historical matrix remains306PASS/55FAIL/79BLOCKED of440.
