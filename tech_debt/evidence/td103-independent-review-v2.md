# TD-103 Independent Review V2

Reviewer: fresh-context agent `01a052f8-0f90-7321-ad53-fcde54b88e69`

Verdict: `REVISE`

Score: `5/10`

## Findings

1. HIGH: output no-clobber still had a check/replace race.
2. HIGH: a torn final journal record was ignored but not truncated before the
   next append.
3. MEDIUM: interruption after backup unlink could leave stale `renamed` state.
4. MEDIUM: `rename(source, backup)` could replace a concurrently created
   backup.
5. MEDIUM: source-ancestor symlinks were not rejected before staging.
6. MEDIUM: object sealing did not reject unexpected directories.
7. LOW: catalog backup counts were hardcoded rather than derived.

## Final Correction

- output publication uses atomic no-replace hard-link creation; a concurrent
  differing publisher is blocked and never overwritten;
- the exclusive controller lock repairs and fsyncs a torn journal tail before
  any recovery or append;
- backup removal has durable intent, and symlink recovery clears every pending
  backup state;
- source-to-backup uses no-replace hard-link publication, inode parity, and
  source unlink instead of overwriting rename;
- every source parent rejects symlink components before staging or reading;
- object-tree membership requires the exact file and directory sets before a
  `0555` seal can pass;
- top-level backup counts are derived from a consistent inventory;
- targeted unit/fault coverage is 29/29 PASS, including all seven findings.

Per the queue rule, this is the second and final independent review. There is
no invented third score. Completion relies on the recorded review scores plus
the objective post-correction gates and real-payload verification.
