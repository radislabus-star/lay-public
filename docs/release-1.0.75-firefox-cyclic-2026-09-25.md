# Lay 1.0.75 — Firefox repeated Double Shift

This release publishes the user-accepted Firefox repair from source checkpoint
`51420a4a18e3b2445a5b7a1d9e591382df640019`. A word can be flipped with
Double Shift repeatedly in both directions after Space, including the first
word. The accepted IME suggestion/Tab and autocorrection-undo routes are kept.

The user's installed and loaded IME at acceptance had SHA-256
`4bbe07233808d1d14ecd072b87c052d760c22fe17bcac5f63d8adf9e8d1c7328`.
The pre-acceptance guarded development check selected 2,897 tests and passed.
The exact user-accepted runtime was not rebuilt, retested, or reinstalled for
this source-version publication.

The Tor Browser route without SurroundingText remains unresolved: Tab and
Double Shift failed in the user's Tor field. A later narrow Tor source
experiment was not accepted and is excluded from 1.0.75. This release makes
no claim of complete window-type coverage. The source and physical evidence
are recorded in [TD-121](../tech_debt/121-preserve-word-across-ime-layout-handoff.md)
and the [window-type matrix](ime-window-type-acceptance-2026-09-23.md).
