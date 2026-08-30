# TD-103 Independent Review V1

Reviewer: fresh-context agent `01a052ec-918e-7863-883f-38fc7c561443`

Verdict: `REVISE`

Score: `4/10`

## Findings

1. HIGH: unrestricted output paths could replace unrelated writable files.
2. HIGH: store ancestor symlinks were not rejected before writes.
3. HIGH: first journal creation did not fsync the transaction directory.
4. HIGH: the task still presented the same-filesystem owner decision as open.
5. MEDIUM: predictable temporary and backup names lacked provenance checks.
6. MEDIUM: object staging could follow a source symlink before rejecting it.
7. MEDIUM: a missing source and backup did not reconstruct from the verified
   object.

## Correction

- output is confined to direct `tech_debt/evidence/td103-*-vN.json` paths and
  an existing differing file is never replaced;
- store, lock, journal, object-directory, and object-file symlink indirection is
  rejected before write;
- every journal append fsyncs both the file and transaction directory;
- the same-filesystem ownership decision and zero independent backups are now
  explicit, while the object tree is sealed `0555` against accidental removal;
- source rename has durable intent, backup cleanup requires a pending journal
  state, and temporary links use unique names;
- staging refuses source symlinks when the object is absent;
- a missing historical projection is reconstructed from the verified object;
- the correction suite now covers 20 unit and fault-injection tests.
