# Lay 1.0.125: stable updates under GitHub API rate limits

Source/build gates completed on2026-11-10. Public CI and stable publication are
separate promotion gates. No input-runtime installation is performed by this
development task. The user continues obtaining stable versions through the
menu updater.

The updater previously stopped on GitHub API HTTP 403, leaving the user's
running version at1.0.123 even after the letter-triggered1.0.124 was published.
The default official release API now falls back after403/429 to the same
repository's official HTTPS latest-release redirect. The resulting URL must
identify exactly `radislabus-star/lay-public` and a stable `vX.Y.Z` tag.

Exact-tag fetch, source-version matching, confirmation, lock, owned-source
checks, user-work preservation and no downgrade remain unchanged. A custom API
cannot silently select the public repository. Untrusted/unresolved redirects,
prerelease-shaped tags and unavailable fallback remain errors. No credentials,
HTML parsing, stale cache or retry loop is introduced. Discovery prints its
actual source.

The only production behavior change is in `update.sh`. Package/lock and GNOME
tray metadata are synchronized to1.0.125. All Rust input/correction code and
desktop integration behavior are unchanged from1.0.124. Its scoped native
evidence remains historical; it is not relabeled as a new1.0.125 client run.

The reproduced failure, source comparison and preflight are recorded in
[the analysis](development/fragment124-update-rate-limit-2026-11-10.md).
The owning decision is
[stable-update-rate-limit](architecture/decisions/2026-11-10-stable-update-rate-limit.json).

Verification to date:37 updater/installer tests PASS, including controlled
403/429 discovery, exact-tag apply, no downgrade, both desktop menu launchers,
untrusted redirects and unavailable fallback. The original source fails the
three selected positive regressions (five subcases). Shell syntax, diff
whitespace and architecture canon checks PASS. The reviewed installed updater
script independently reports123->124 while the real API is rate-limited;
check-only elapsed1.08s. No daemon/IME restart or binary install accompanied
that script-only bootstrap. The same reviewed script is bootstrapped on KDE;
its actual updater reports installed124 equals stable124. KDE's active IME
still has the exact original123 hash: an ordinary update retains active input.
Use the existing full-restart action after upgrading to load the new IME.

Canonical release verification is PASS:3,650 Rust correctness/package tests,
11 serialized performance tests,475 headless Python passes with12 environment
skips, architecture/lint/syntax/CLI/ngram checks, allten installable release
binaries, installer contracts and both actual Rust1.88.0 locked all-target
checks. Source closure:1,338 files,
`f79d39f81734e4e873923fa552d9c11e014fc8725d23626b18191ab0b7df2f63`.
The full frozen snapshot additionally binds the root updater and extension
metadata, which are outside that Cargo source closure. All snapshot file hashes
and modes were checked before and after execution. The reviewed updater SHA is
`b7c05b3eff7199cadca53d8403482dd6ca4c957e5cc396dc511721e9d6b2d836`.

An AST navigation-graph rebuild was rejected because it lost capability edges
for three unchanged mutation sinks. Its report/output are retained. The accepted
Rust graph was preserved, and the existing PASS-only receipt was refreshed for
Cargo125 metadata. No runtime or architecture guard was changed to fit the
extractor. The new task scope closes the prior46 input-integration seams.

Complete source/build and bootstrap evidence:
[release-1.0.125-verification.json](development/evidence/release-1.0.125-verification.json).
Public CI and stable publication are not inferred from these source results.

Latest complete ordinary KDE matrix remains329 PASS /59 FAIL /52 BLOCKED of440
(Firefox143/51/26, Chrome186/8/26). This updater change does not repair or rerun
that native matrix.
