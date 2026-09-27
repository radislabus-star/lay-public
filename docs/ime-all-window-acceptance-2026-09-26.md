# Lay IME acceptance across editable windows — 2026-09-26

This is the acceptance contract after the user's 2026-09-26 correction. It
supersedes the two-gesture Double Shift criterion in the older
`ime-window-type-acceptance-2026-09-23.md` matrix. A source test, running
service, selected engine, or a single successful flip is not a client PASS.
Run this matrix on the exact installed candidate and record the binary hash,
selected GNOME and IBus sources, focused window identity, before/after text,
and bounded IME/daemon trace for each row. No external message is sent.

**Temporary installed candidate, 2026-09-27:** IME
`359d0b16f49038d4b69f7453f736abc1145e057f6cf105bf4eb8cf59050ec10e`,
daemon `aa5fe6c98dbbd750c7266907563193b6afa734f58739d9487e089c7fd2f873ea`,
with a GNOME extension exposing IBus `InputMode`. Full source gate and pair
install passed. The user reports that Double Shift fails in some Tor Browser
fields on this pair; the exact fields, gesture numbers and icon states were
not captured. This is a **user-reported Tor failure**, so all-window acceptance
is blocked. Historical verdicts in
the table refer to their stated older binary pairs. Installation and source
evidence: `docs/architecture/runtime-stabilization-2026-09-26.md`.

## Required checks in each editable field

1. A visible IME suggestion survives the next printable key or updates to a
   valid new suggestion. Tab accepts the visible completion, appends one
   trailing space, and does not unexpectedly move focus. Record the actual
   offered suffix rather than assuming a fixed word.
2. Hold Shift physically through a word in the same field. Check that all
   intended letters retain their case through client Reset callbacks and that
   the IME suggestion remains visible and coherent. Then release Shift.
3. On one word in one field, perform **eight rapid Double Shift gestures**
   (sixteen Shift keydowns, paired as gestures). Every gesture must cause the
   expected layout conversion. The eight resulting surfaces must alternate
   A/B without a missed, duplicate, or delayed conversion; the eighth returns
   to the initial surface. **After every pair, the native GNOME layout icon
   and the actual decoder for the next letter must change with the word**
   before the next pair. One selected Lay source may persist across gestures;
   its IBus `InputMode` property must report the displayed RU/EN mode. Each
   complete flipped word must be visible before starting the next pair.
   Record the time, intermediate surface, icon, next-key result, `InputMode`,
   and GNOME/IBus source after each pair; a correct final word after quiet
   time is insufficient.
4. If an autocorrection actually applies, verify Double Shift cancels that
   applied correction. If no correction applies, report `NOT TESTED` for undo,
   never PASS.
5. Repeat the field checks after leaving the window and returning. A source
   switch or service restart must not be needed to recover IME or Double Shift.

For a window with no editable field, record `NOT APPLICABLE` with the inspected
role. For an unavailable or unready application, record `NOT TESTED` with the
blocker. A failure in any required editable window blocks an all-window PASS.

## Window inventory and required rows

The per-pair GNOME icon requirement was confirmed after the historical
text-only eight-gesture runs below. Their `PASS` labels apply only to the
observed text surfaces; no row receives a complete Double Shift PASS until
the native icon, IBus `InputMode`, next-key decoder and selected source are
recorded after each pair on the exact installed candidate.

Read-only inventory at about 03:10 EEST: AT-SPI reported two Firefox frames,
ten Google Chrome frames, one Thunderbird frame, three WeChat frames, one
GNOME Text Editor frame, one GNOME Terminal frame, five Nautilus frames and
single extension-manager and gjs frames. XWayland additionally reported one
Tor and two Chromium-GOST windows. Process inspection found two Kitty
instances and Telegram Desktop. This is a time-stamped inventory, not proof
that each frame exposes an editable field; refresh it immediately before a
physical run and assign one receipt per actual window/field.

| Application/window family | Field to verify | Current candidate verdict |
| --- | --- | --- |
| Firefox, user's WhatsApp window | Existing unsent message composer; `п→р` and `п→Tab` are incident reproductions | One earlier installed pair IME `c2ced61c...` / daemon `7838d2f...` at 14:04–14:05 EEST passed `п→р`, Tab one space and synthetic held Shift, but eight Double Shift failed. Its Reset witness defect has source regression RED/GREEN and full gate PASS. Gated release pair `d6d7423c...` / `7dfd78ac...` remains NOT TESTED in WhatsApp: at later revisit, the tab displayed QR login with no message composer. Earlier exact empty composer was restored without sending. Hardware Shift, applied undo and focus return NOT TESTED. Overall row incomplete; current blocker is WhatsApp login. Receipts in stabilization plan. |
| Firefox, GitHub Issue #46 | Focused unsent `Add a comment` editor | Earlier gated `d6d7423c...` / `7dfd78ac...`: `п→р`, Tab, synthetic held Shift PASS; fixed 80/160 ms eight-gesture sets FAIL with partial AT-SPI surfaces, while next-pair-after-complete-surface PASS in 2,956 ms. New gated and temporarily installed `9ada21b9...` / `dd94599f...` repeated the same split: `п→р`, Tab one U+0020, held Shift PASS; fixed-fast eight FAIL (`[просто, "", g, про, g, п, g, п, g]`, settled `про`), daemon logged eight recognitions and eight replays; adaptive eight PASS in 2,819 ms. Painted frames, hardware Shift, applied undo, focus return and other Firefox windows NOT TESTED. Exact empty draft restored, no comment sent, owned window closed, pair rolled back. Overall row incomplete; exact receipts in stabilization plan. |
| Firefox, other open window(s) | Textarea, contenteditable, GitHub composer and address bar where present | NOT TESTED |
| Google Chrome, every open editable window | Textarea, contenteditable and browser chrome where present | Earlier gated `d6d7423c...` / `7dfd78ac...` passed owned `input` and `contenteditable` but **failed owned textarea Tab** after an `input → textarea` switch: duplicate `FocusInId` revoked native activation. On new gated and temporarily installed IME `9ada21b9...` / daemon `dd94599f...`, the owned local `input → textarea → input` cycle passed `п→р` completion update, Tab with U+0020 and retained focus, synthetic held Shift, and eight complete sequential AT-SPI flips in each field. Additional returns from an owned GTK window passed in both Chrome fields; the harness could refocus the field. Chrome contenteditable on this newest pair, other Chrome windows, physical Shift, painted frames, applied undo and natural field focus retention remain NOT TESTED. Exact empty fields restored, isolated browser closed, candidate rolled back; receipts in stabilization plan. |
| Tor Browser | Editable web field and address bar; track its separate surrounding-text behavior | **USER-REPORTED FAIL on current installed pair:** Double Shift does not work in some Tor fields. Exact field, gesture number, text/icon state and trace are not yet recorded; other Tor fields remain NOT TESTED. |
| Chromium-GOST | Editable web field in each open window | On gated temporarily installed IME `a9b46f75...` / daemon `6a156289...`, a fresh owned input passed `п→Tab` with one space, `п→р` suggestion update and Tab, and eight exact visible word transitions. That route never published a dynamic IBus `InputMode`; GNOME/IBus was RU after the eighth pair. The native icon and next-key decoding were not recorded per pair, so these runs cannot satisfy the user-confirmed icon/input criterion. Fresh held Shift also failed: `ПРОсто` remained mixed-case. The accepted pair was restored after each run. Earlier pair `0a443521...` / `d4f963b7...` passed fresh Tab but failed the second Double Shift at `context_authority`; historical details and exact receipts are in the stabilization plan. Focus return, applied undo, painted frames, built-in hardware keys, other GOST fields and other windows remain NOT TESTED. Overall row **FAIL**. |
| Kitty, including the Codex window | Safe terminal input, before and after Firefox focus transfer | Owned safe `read` window on gated `d6d7423c...` / `7dfd78ac...`: IBus trace updates visible preedit `очему`→`осто` on `п→р`; Tab plus control key shows `почему ч` (one space); synthetic held Shift `ПРО`; eight full sequential Kitty screen surfaces alternate `ghj/про` in 2,459 ms. Scoped PASS. The first failed synthetic attempt was an invalid fixture missing `KEY_A`, so daemon did not subscribe; corrected attempt proved daemon subscribed and recognized eight gestures. User's Codex Kitty window, Firefox focus transfer, painted frames, physical Shift, applied undo and focus return NOT TESTED. Owned `read` captured only newline; no command ran. Candidate rolled back. Receipts in stabilization plan. |
| GNOME Terminal (VTE) | Bounded `read` input so typed text is not executed | Current gated pair NOT TESTED: owned `read` window failed before creation (`Failed to open PTY peer: Too many open files`). No candidate install or input; runtime authority unchanged. Earlier different-pair two-toggle result is in route map. Preflight receipt in stabilization plan. |
| GNOME Text Editor | Owned disposable empty file | On gated `d6d7423c...` / `7dfd78ac...`, Tab committed `почему ` with U+0020, synthetic held Shift gave `ПРО`, and eight full sequential AT-SPI flips alternated `ghj/про` in 2,185 ms: scoped PASS. AT-SPI text after `п→р` was `пр`, but the suggestion popup was not exposed, so suggestion state UNKNOWN. Hardware Shift, painted frames, applied undo and focus return NOT TESTED. Owned file restored byte-empty; candidate rolled back. Receipts in stabilization plan. |
| GTK entry and multiline widget | Owned local entry/TextView controls | Latest gated `d6d7423c...` / `7dfd78ac...`: both fields passed Tab (`почему ` with U+0020), synthetic held Shift (`ПРО`) and eight complete sequential AT-SPI flips (Entry 2,148 ms; TextView 2,264 ms). TextView IBus trace published `очему` after `п` and updated visible preedit to `осто` after `п→р`; painted popup not measured. Entry suggestion after second key UNKNOWN. Physical Shift, painted frames, undo and focus return NOT TESTED. Both fields restored empty; candidate rolled back. Receipts in stabilization plan. |
| Qt multiline widget | Owned local QTextEdit control | Latest gated `d6d7423c...` / `7dfd78ac...`: Tab gave `почему ` (U+0020), synthetic held Shift `ПРО`, and eight complete alternating AT-SPI flips in 2,193 ms, scoped PASS. `п→р` AT-SPI text stayed `пр`; the separate suggestion popup was not inspected, so suggestion state UNKNOWN. Physical Shift, painted frames, undo and focus return NOT TESTED; full row remains incomplete. Earlier bridge candidate SHA `f644a3f9...` also passed Tab/held Shift/eight synthetic flips; prior SHA `2b21860c...` failed its first eight-gesture set (7 flips). Latest receipts in stabilization plan; older receipts in route map. |
| Thunderbird | Unsent draft field | NOT TESTED |
| Telegram Desktop | Unsent draft field | NOT TESTED |
| WeChat windows | Unsent draft field in each editable window | NOT TESTED |
| LibreOffice Writer | Disposable local document | NOT TESTED |
| VS Code | Disposable local file | NOT TESTED |
| Nautilus, extension manager and other open utility windows | Search/rename or other editable control if present; otherwise NOT APPLICABLE | NOT TESTED |

The seven-of-eight Qt failure is now attributed to a bridge-fence marker that
arrived 365 microseconds after its 5-ms deadline on the third gesture. The
daemon restored the original source before deleting text; the fourth gesture
then produced the repeated action-log direction. A source candidate gives
that already owned bridge fence 20 ms while preserving the shorter callback
budget and identity checks. Its focused IME suite passed 610/610; the exact
installed version used for the Qt row still failed and was rolled back.
At that source-only stage the new bytes had **no physical-client verdict**.
Causal proof:
`/home/ubu/.cache/lay/development/live-qt-mirror-candidate-20260926/CAUSAL_PROOF.json`.

Subsequent owned Qt input on exact bridge-candidate SHA `f644a3f9...`
passed the three listed synthetic scenarios. The source gate passed 2,875
correctness and 36 package tests, but the Qt verdict is based on the
installed binary and the observed widget text, not that gate. Receipt:
`/home/ubu/.cache/lay/development/live-bridge-candidate-20260926/QT_ACCEPTANCE.json`.
The candidate was rolled back to accepted SHA `a8b9d168...`. This does not
complete the row's hardware, undo or focus-return checks, nor any other row.

The focus/Firefox and narrow `п→Tab` candidate was temporarily installed for
an owned Qt A/B, then rolled back to the accepted loaded/installed IME.
The sentinel Tab scenario has a red/green production-adapter proof and passed
607/607 IME tests; the full correctness/package gate passed 2,908/2,908 on
the candidate snapshot. Qt Tab failed on candidate and baseline; exact local
receipts and the observed profile-transition mechanism are in the owning
route map. The eight synthetic Shift pairs did not reach the daemon because
its watched input devices were enumerated before that virtual keyboard was
created; those earlier gestures remain `NOT TESTED`. The corrected Qt
capability candidate then passed Tab in the owned Qt field, but failed all
eight synthetic Double Shift gestures at IME `context_authority` and was
rolled back. Continue on a corrected exact binary, and stop only when every
applicable row on the exact installed binary passes; preserve separate FAIL,
NOT TESTED and NOT APPLICABLE rows.
