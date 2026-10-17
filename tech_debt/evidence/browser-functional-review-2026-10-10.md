# TD-140 — independent implementation review, 2026-10-10

Reviewer: fresh-context agent `/root/browser_functional_code_review_20261010`.
Read-only scope; no test/build/native input/browser automation/edits by reviewer.
Parent: `92965082038074096d82f693997e107e75fe51b7`.
Two passes maximum; no third pass.

## Pass 1 — 7/10

1. Pending Space/Tab retained the trigger while allowing later editing. A failed
   `ghbdtn ` could be selected and replaced with `привет `, producing false
   operation PASS and timing. Require exclusive expected actions until target.
2. Focus-return accepted toolbar focusout/return without visiting another
   editor. Require trusted focusin of a different registered field.
3. Missing expected completion used generic Escape FAIL. An unavailable
   prerequisite must be BLOCKED, distinct from a performed Tab action failure.

Root repaired the three mechanisms and added three negative regressions.
The original 25 regressions are retained. Foreign focus observations use the
same finite observation budget and text bound; foreign typing blocks the case.

## Pass 2 — 9/10, ACCEPT_SOURCE_SCOPE

Reviewer conclusion: all three material findings closed; no material remaining
source defects within the reviewed scope. Verified the actual final code and
five relevant file hashes against the final49 receipt. Pending actions reject
manual repair/duplicate triggers before a target, focus requires another
registered editor, and the completion condition distinguishes BLOCKED from FAIL.
The new negative regressions protect each boundary.

Final source receipt:
`/home/ubu/.cache/lay/development/browser-functional-source-final49-20261010/RESULT.json`.
49/49 Node, 4/4 fixture Python, 2/2 existing compat and 15/15 canon PASS.
Native execution of new profiles remains NOT_TESTED. Prior Firefox native
54 FAIL / 4 BLOCKED remain open and were not reclassified by source review.
