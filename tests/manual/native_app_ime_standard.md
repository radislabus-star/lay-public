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
