# Lay 1.0.113 — publication verified

Published 2026-10-30T03:12:09Z:
https://github.com/radislabus-star/lay-public/releases/tag/v1.0.113

Public main and peeled annotated tag:
f7e1ccfc855df4c33e2e82d681ba2573e8337059.
Annotated tag object:3d936469728f5cf56e8ab70c91680141dc74a7fb.
The public latest-release endpoint independently returns v1.0.113,
draft=false,prerelease=false,target=f7e1ccfc. Private branch/tag were pushed;
origin/main remains untouched. No tag was moved and no force push was used.

Independent source verification:
- CI:https://github.com/radislabus-star/lay-public/actions/runs/40993117623 — SUCCESS.
- Security:https://github.com/radislabus-star/lay-public/actions/runs/40993117500 — SUCCESS.
- Complete canonical correctness/package wrapper:3611 selected,0 semantic
  failures,0 infrastructure failures,verdict=PASS. This closes the earlier
  observation-binding stop without hiding its original exit1 or isolated logs.
- Both strict lint profiles, MSRV, installer, release binaries and ngram cache
  probe passed. The dedicated remote installed release set has10 programs,
 0 compiler warnings; retained compiler-artifact identities are in the proof.

Scoped native KDE evidence is10/10PASS, separately from the historical matrix.
The first8 cases covered first word, second word, physical punctuation-key
letters and manual Double Shift roundtrip after Space in normal Chrome/Firefox
text inputs. Two additional native cases verified automatic RU→EN:
руддщ +Space ->hello +Space, Chrome66.3ms,Firefox17.1ms.
Inputs use the unchanged r19 RFB keyboard/readiness/observation helpers;
additional explicitly selected fixtures do not alter the original220-case matrix.
Fixtures:ghbdtn ->привет;привет ghbdtn ->привет привет;
,typjdjpf[ ->бензовозах;руддщ ->hello. Config was restored byte-for-byte;
global IBus and the original Firefox session were preserved. No input-route
source was modified. The private no-display client remains a setup failure
before the first case with the flag both ON/OFF and was never counted as PASS.

Model reproduction independently matches model.bin,model.json and heldout
JSONL byte-for-byte. Model SHA256:
28dc04cccc131c0b78167758982402ae519509d98bc9f0abcff22782286cdca9.
Full comparison and limitations remain in fragment-layout-contour-2026-10-30.md:
no original Caramba weights, noun-form RU proof scope,13 clean EN ambiguities,
and3 fewer EN restorations in exchange for fewer wrong changes. Default OFF.

Actual installed updater entrypoints verified after publication:
- Local GNOME:installed1.0.112,stable1.0.113,offers1.0.112 ->1.0.113.
- Dedicated KDE:installed1.0.113,stable1.0.113,reports current stable.
- Both resolve to ~/.local/lib/lay/update.sh SHA256
 aebfb65ecec2003adda575bd1bc3a5881df2b8eb1e1f5e47a768367be0e7abae.
Only --check was executed; no local installation or restart was performed.
Actual KDE menu exports Lay enabled=1 and fragment method=0 after restoration.

Evidence CACHE:release113-{ci,security,public-readback,updater-readback}.json,
release113-ci.log,release113-{local,kde}-updater-check.log;
fragment-kde-native-reverse-v1-result.json.
CROOT:fragment-kde-native-v1,fragment-kde-native-reverse-v1,
fragment-reproduction-v1,fragment-runtime-final-pre-release.json.
Historical full matrix remains338PASS/24FAIL/78BLOCKED out of440.

This completion entry and the matching handoff are documentation only.
Published source/tag/binaries remain the verified f7e1ccfc version; no functional
checks, builds or service restarts are repeated for this completion record.
