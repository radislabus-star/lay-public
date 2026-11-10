# Letter-fragment report: installed 123 and blocked update to 124

## Preflight and first failed transitions

Read-only inspection on 2026-11-10 established the accepted development checkout
`/home/ubu/projects/lay-dependency-updates-20261027`, branch
`codex/browser-matrix-repair-20261027`, HEAD
`e334f2f45681ca3b8d81fb68af56dee6dee7a12a`. The working tree was clean.

The primary GNOME machine reports `lay 1.0.123`. Exactly one daemon and one IME
are running; both loaded executable hashes equal their installed files:

- Daemon: `127771cddddb2a87fdae79248e8064db7721cda4530347136a804805c23c492a`.
- IME: `2a4c636a3be959ce096754909599a202146b7711157c2563e6980f5c8909d429`.
- Config: fragment mode and automatic layout selection enabled; text backend
  `ime`, layout backend `auto`. No live config was modified.

The user's observed Space-only behavior agrees with the installed source:

```text
v1.0.123
  ibus_interface.rs::process_key_event
    -> WindowInteraction::process_legacy_key
  completed-word correction
    -> typing_rule_graph/rules/fragment.rs::apply_with_mode
      -> layout_fragments::analyze(token, Boundary::Closed)
```

Tag 123 has no `fragment_layout.rs` IME caller and no
`layout_fragments/letter.rs`. This is a missing letter trigger in that version,
not evidence that a loaded 124 caller waits for Space. Tag 124 adds the separate
paired-letter-release caller and restores the original boundary policy.

The retained exact-candidate 124 GNOME native log independently contains
`ghbd -> прив -> приве -> привет` and `рудд -> hell -> hello`, before a Space or
manual toggle. Each has one `ibus_letter_fragment_layout: applied` record.
These are recorded candidate observations, not a fresh test of the user's
currently loaded 123. See the existing v63 receipt under
`/home/ubu/.cache/lay/development/letter-fragment-request-20261102/gnome-gtk-eight-model-v63-20261109/`.

An initial regular installed `lay-update --check` successfully discovered
123 -> 124. The user's subsequent attempt failed. A fresh direct request then
reproduced HTTP 403 with `X-RateLimit-Remaining: 0`. At the same time an HTTPS
HEAD to `https://github.com/radislabus-star/lay-public/releases/latest` succeeded
and resolved to `/radislabus-star/lay-public/releases/tag/v1.0.124`.
The updater currently treats every discovery exception as fatal. This second
failure prevents delivery of the already published letter-trigger correction.

A later independent KDE inspection found another deployment boundary: installed
CLI124, but active IME SHA
`b765fde82d7f49418b0175880a0eb97aed1dde0ba5321524c60307e668c94a06`,
the exact original123 engine, running from the old Cargo release path. The
daemon had newer installed bytes. A regular update explicitly keeps the active
IME session; CLI124 does not establish that the letter caller is loaded. The
existing explicit full-restart action is the operation for reconciling loaded
modules after an update. No restart was performed by this updater-only repair.

Even the new caller is deliberately conservative: alphabetic tokens3..32,
paired letter release, both relevant options enabled, a Known target with
enough unopposed evidence, no protected-word/manual-return suppression, and an
exact current field/frame. An abstention can leave a word for the independent
ordinary Space correction. Space does not call `on_letter`; observing conversion
after Space alone does not identify which optional detector was active.

## Bounded repair

Classification: release discovery only. Baseline: `e334f2f4`.
Runtime source scope: `update.sh` only. Regression scope:
`tests/test_update_menu.py`. Owning contract:
`docs/architecture/stable-auto-update-2026-10-19.md`; new decision:
`docs/architecture/decisions/2026-11-10-stable-update-rate-limit.json`.
Publication metadata for125 is limited to `Cargo.toml`, the root package entry
in `Cargo.lock`, `VERSIONING.md`, GNOME `metadata.json` and the version/date
constants in `tray_support.js`. No adapter behavior changes are authorized by
that metadata synchronization. Generated architecture/graph artifacts must be
refreshed through the existing scripts.

Use the unchanged stable-release API first. Only its official default endpoint
may fall back after HTTP 403/429, to the official latest-release HTTPS redirect
of the same repository. Reject an unresolved latest URL, another host/repository,
prerelease-shaped tags, query/fragment additions and malformed versions. Keep
exact-tag fetch, source-version comparison, lock, source ownership, dirty-work
preservation, confirmation and no-downgrade checks unchanged. Display the actual
discovery source. No tokens, HTML scraping, stale cache or retry loop.

Before changing production discovery, run the actual updater in isolated homes
against controlled HTTP responses and preserve the RED result. Then verify
403/429 discovery, exact-tag apply, current/newer versions, both menu launchers,
untrusted redirects and unavailable fallback. Run existing updater/installer
regressions and shell syntax checks. A local bootstrap may replace only the
installed updater script with reviewed identical bytes; it does not install or
restart IME. User continues receiving Lay through the updater.

Protected: all IME/daemon Rust source, delivery, ownership, verifier, desktop
adapters/settings, live config, learned/model data and active user text. No
field-integration or native-matrix change is part of this repair. If a new
release is prepared, its required release checks must finish before publication;
old native results remain explicitly dated and scoped.

## Verification state

Preflight complete. Original updater RED:3 tests,5 failed subcases; the failures
are the reproduced403/429 discovery, exact-tag apply and both menu launchers.
Reviewed updater GREEN:37 updater/installer tests PASS. Shell syntax, diff
whitespace and architecture canon PASS. At08:46 UTC the installed script was
atomically replaced under its existing update lock after confirming the exact
stock123/124 hash. Backup and before/after hashes are recorded at
`/home/ubu/.cache/lay/development/letter-fragment-request-20261102/updater403-bootstrap.json`.
The actual installed entrypoint then resolved123->124 via the official website
while the API still returned403; check-only elapsed1.08s. No binary/config
installation or service restart occurred. A125 patch is prepared to keep this
repair after an ordinary update would replace the123/124 updater script.
At09:12 UTC the exact same reviewed updater script was bootstrapped on KDE under
its update lock; only the updater changed. Its real entrypoint reports installed
124 equals stable124. No input processes/configuration were modified.

The remote development gate passes3,650 selected Rust correctness/package
tests. Canonical release gates also pass:3,650 correctness/package,11 performance,
475 headless Python with12 environment skips, allten installable125 binaries,
installer checks, architecture/lint/syntax/CLI/ngram, and actual Rust1.88.0 both
locked all-target modes. Source closure1,338 files SHA
`f79d39f81734e4e873923fa552d9c11e014fc8725d23626b18191ab0b7df2f63`
matches local/remote. The complete frozen2317-file ledger separately includes
root `update.sh` and extension metadata and was verified before/after the run.
Public CI4/4 passed on code8695d9c9. Stable125 was published at10:09:02 UTC;
annotated tag1c00f4ea points to that code in both repositories. The latest API
and official website independently select125. Actual installed check-only
entrypoints report primary123->125 in0.592s and KDE124->125 in0.682s, exit0.
No input-runtime installation or service restart accompanied these checks.
Publication receipt:
[release-1.0.125-publication.json](evidence/release-1.0.125-publication.json).
Complete source/build evidence:
[release-1.0.125-verification.json](evidence/release-1.0.125-verification.json).

The pre-release mechanical scope check caught the still-active completed124
scope, whose46 shared paths did not include `update.sh`. The written preflight
already named that script; before snapshot execution the machine-readable
declaration is replaced with the new task, immutable baseline e334f2f4 and only
the updater, generated architecture receipt and the tray's version/date constants
as exact shared files. The date constant requires an explicit shared declaration
under the existing classifier; the whole GNOME component is not enabled. No
guard implementation changes. The old46 integration seams are no longer
editable under this task.

The mandatory AST graph update was attempted and rejected: regeneration removed
capability edges for three unchanged mutation sinks (`call_replace_text`,
`try_ime_replace_tail`, `apply_text_replacement_pipeline`), yielding WATCH. Raw
report and rejected generated graph are retained under C/updater125-architecture-
failure.json and C/updater125-rejected-ast-graph.tar.gz. All Rust source was
identical to124. Only that newly generated navigation output was restored to
the accepted graph; no runtime or guard was edited to accommodate the extractor.
Refresh the PASS-only architecture receipt for125 metadata with the unchanged
accepted graph and verify it using the existing gate. A rejected graph rebuild
is not described as a successful refresh.

Latest complete ordinary KDE matrix remains Firefox143/51/26,
Chrome186/8/26 PASS/FAIL/BLOCKED, 440 total. This analysis does not change it.
