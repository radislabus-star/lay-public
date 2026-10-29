## Latest user clarification: preserve terminal presentation

User says BOTH terminal hints look correct, but Double Shift stopped. No all-KDE presentation installation is authorized by that clarification. The glyph-only draft passed3projection controls and4native Qt mask shapes with no focus/runtime change, but was NEVER installed. Exact original rendererc3043b69 restored locally; uninstalled draft remains only in remote workspace5450833c until next overlay. Current visible terminal rendering remains accepted. Firefox separate issue: hint appears one space away, smaller and above the text; root map points to KDE renderer system-font fallback/height*0.78 and client caret coordinates. No claim that removing a border fixes font/baseline. Next: short isolated terminal Double Shift reproduction; no Firefox runtime change and no global Inline/Auxiliary switch.

## User-requested style change after functional acceptance

User confirms current Firefox works and asks to remove the hint rectangle, also seen in terminal. Current remote daemon1722873/SHA35527e63 and IME1852753/SHA6688be16 remain accepted for this stated scope. Konsole Double Shift is separately reported FAIL, not covered by Firefox acceptance. Hold uninstalled pacing candidate: source25/25+340/340+architecture PASS, no build or activation.

Display map: existing IME InlineHintV1 suffix/rect -> scripts/lay-kde-tray.py::InlineHintView.receive -> QLabel CSS + widget mask. Only that KDE renderer changes. KWin compositor independently reports false; grey glyph-only XShape mask avoids changing desktop compositing. Freeze all other integration code. Preserve exact native caret and window input/focus flags. New ADR2026-10-29-kde-hint-glyph-only-presentation. No test assertion or functional path changes.

# KDE Firefox: distinguish only the unaccepted hint suffix

Baseline b835ca47; source d8a90c49; installed IME698174/fbf3808d, daemon515781/0a71.
No Harness/agents/GNOME operations/publication. Tab phase native16P0F0B(21.598s)
and ordinary22=21P0F1B(74.449s). Email native/DOM disagreement remains BLOCKED.
Tab median17.16ms,p9519.72ms,n16,RFB release to observed target. FullFirefox
126/55/39 and Chrome earlier179/0/41 unchanged. Source1140 is separate proof.

## Actual cause and evidence

CROOT/kde-firefox-queued-owned-append-hint-look-native-v1/firefox/result.json
and hint-look-{text,textarea}-first-tab-pri.png. DOM raw_text=pri/caret3,
projection text=vet, screenshot shows merged black-on-white privet. No typed
letter disappeared in these frames. Current running renderer3f936370 has opaque
white background, black foreground, border:none. Artifact episode absent, not
proved fixed. Existing renderer owns only suffix and coordinates already read
by the KDE Firefox leaf; those coordinates are not mutation authority.

## Exact map and allowed behavior change

```text
kde/firefox/hint_geometry.rs::kde_firefox_owned_hint_rect (READ, UNCHANGED)
 -> hint_view.rs::publish_inline_hint existing desktop/client dispatch
     Some(Ok Firefox rect) adds outlined=true to existing read-only frame
     other client/desktop/default remains outlined=false
 -> existing HintViewBus V1 signal/snapshot (no new RPC)
 -> scripts/lay-kde-tray.py::InlineHintProjection validates optional boolean
 -> InlineHintView.receive applies thin border only to this suffix label
```

Minimal option9/10: one optional flag plus1px outline, existing white/black
style and exact text/position/font preserved. Reject all-KDE CSS change because
it would alter accepted Kitty/Chrome; reject GNOME/common text-attribute edits
because this is the KDE overlay. Do not create a second detector or new state.
Common frame and KDE script are explicit shared seams in the new ADR before
source edits. Typed document text, composition, candidate selection, Tab/Space/
Backspace/DoubleShift/layout/owner/verifier/executor stay unchanged. A missing
flag preserves legacy frames. Reset style for every frame to prevent leakage.

Tests: malformed optional flag must not consume a revision; legacy payloads
remain accepted, both presentation values accepted by KDE and unchanged GNOME
reader; bounded text/owner/revision constraints retained. Actual after snapshot
must still have DOM prefix and separate suffix only. Renderer border fits its
existing2px width allowance; no coordinate/geometry/budget change.

Native title-case diagnostic V1 raw0P16F is INVALID INPUT PRECONDITION, not a
Tab product verdict: actor sent uppercase keysyms without holding Shift and
DOM was already lowercase before Tab; all-uppercase case failed before Tab.
Keep raw results. A corrected actor must hold ordinary Shift and verify the
actual uppercase prefix before testing Tab. No runtime Shift/case edit justified.

Queued phase causal trace rows1,12,15,20 incomplete: native counts valid but
those rows are refused for causal claims. A partial row20 was inspected after
completion; its event counters are not accepted proof. No claim that the new
queued-echo branch was physically observed. Genuine source sequence and actual
native behavioral acceptance are recorded separately.


Outline source RED: two unchanged projection controls PASS, malformed optional
flag test fails as expected (old parser accepted and consumed its revision).
SHORT-v1=28/28Rust PASS +3/3Python projection tests +unchanged GNOME Node tests
with added valid optional-flag frames. One new Rust identity covers style-only
publication change, old-owner refusal and reset on next owner/hidden frame.
Source97671c03. No new IME/tray installed. Full affected source gate next.


Corrected held-Shift V2 with independent actual prefix control=16P0F0B/24.023s,
text/tel/textarea/plain x P/Pri/Prive/PRI, all actual Cyrillic case preserved and
Tab complete. Runtime/config unchanged. Raw wrong-input V1 preserved. No
production case/Shift change needed. Total32 focused Tab cases passed across
lowercase and explicit ordinary Shift; still separate from full matrix.


Full GREEN-v1:1141/1141IME+28/28short PASS,Python projection PASS,unchanged
GNOME Node projection PASS,graph53.445s,registry exact1newcorrectness/0removed/
0changed. Tested e0ccccd6, post-registry source1308 SHA
703848cb813f677ebe5027ac418bb939c4c5e0e4a75006ead533c110408ce3a2.
Next zero-warning IME-only build with frozen KDE script bundle, idle activation
of those two changed components, visual after frames, native controls, full220.
Current loaded IME still698174/fbf3808d and tray3228081/old3f936370; no claim
that outline is installed or physically visible yet.


Installed exact IME795642/689fa38c70359430e0ebf612b5e3c039a2d7f8a5b78855c6805ea9cc89372dfe,
KDEtray795755/scriptc3043b6954687f450d9776a1379d64143ebf5b8df43c88eda8b41d4cba7bf332.
0warningbuild74.544s; idle activation of IME+KDEtray only. Daemon515781/0a71,
IBus3168326,userFirefox988548/config unchanged. Full source703848cb.
Actual afterframes look2/2PASS: DOMpri,projectionvet outlinedtrue, screenshot
shows outline ONLY on suffix. Local CACHE/hint-outline-textarea-after.png.
Current native lower16/16+heldShift16/16, ordinary22=21P0F1B/74.399s; same Email
native/DOM observer disagreement. Protected runtime/config restored. Full220
not rerun. Next rapid-three diagnostic on unchanged cadence before long full.
