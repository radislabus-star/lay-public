# KDE Firefox hint desktop-observation deadline —30October2026

Read-only diagnostic preflight after accepted1.0.112 publication. Baseline1ea1d44e; actual remote IME2201796/05d62373, daemon1722873/35527e63,tray2201829/c40bd104,IBus3168326 and originalFirefox988548. No production edits/build/install/restart or timeout changes authorized in this diagnostic. Do not reinterpret accepted-publication work as regression testing.

Confirmed original case: first word Textarea, typed при retained, caret3,no selection,field focused; completion engine found вет in9us and applied it. Complete row-3 trace records provider_unavailable: KWin foreground read deadline exceeded at80186us, then Firefox geometry native_deadline80204us,samples0. Local CLIENT try_lock Busy did NOT occur. No font/coordinate query began in the failed publication.

Map:
1. src/bin/lay_ibus_engine/kde/firefox/hint_geometry.rs::kde_firefox_owned_hint_presentation —80ms total; initial +final authenticated desktop reads.
2. src/bin/lay_ibus_engine/kde/desktop.rs:20::observe_focused_window_until — wraps actual provider error; no text output.
3. src/bin/lay_daemon/kde_focus.rs:213::read_with_deadline ->Client::read/read_in: fresh request nonce and authenticated KWin owner; connect, prior cleanup, GetNameOwner, loadScript, run, mailbox Observe, cleanup/unloadScript,final GetNameOwner. This source is compiled by both daemon and IME, so any future edit here is SHARED KDE integration, not Firefox-only.
4. src/bin/lay_ibus_engine/hint_view.rs::publish_inline_hint — unavailable actual geometry clears outgoing suffix.
5. src/bin/lay_ibus_engine/preedit.rs:992::finish_inline_hint_publication — error clears visible completion/selectability. Existing safety is correct: an unobserved field cannot accept stale text.
6. General candidate choice,ownership,verifier,executor,GNOME and terminal presentation are protected.

The generic provider deadline error does not identify which RPC/cleanup/response stage exhausted80ms. Do not claim KWin focus theft, lock contention, pre-existing cause or font regression without paired evidence. Native initial batch3PASS1FAIL and isolated recheck1PASS remain intact.

Next one bounded diagnostic: reuse existing rapid-dbus-profile-v4 collector with hint-sequence mode and unchanged accepted hint-look actor8d227025. Three canonical selectors text:first-tab-pri:active,text:second-tab-pri:active,textarea:first-tab-pri:active reproduce preceding sequence only. Passively record GUI KWin load/run/callback/unload RPCs and existing IBus/debug packet. No manual output-route workaround, repeated-to-green loop, timeout enlargement or new producer/owner. A passing packet measures stage timings but does not prove intermittent failure fixed. Preserve identity/config/mode/focus restoration and raw failures.

Options after stage proof: remove only demonstrably redundant fresh-query work inside existing contract; keep guards. Longer timeout alone trades latency for fewer refusals and is not a root fix. A new cache/queue/provider is a separate architectural task, not the first choice.

## Measured follow-up, unchanged runtime

CROOT/kde-firefox-focus-deadline-profile-v1/result.json records one fixed
three-case hint sequence (text first/second word and textarea first word):
3 PASS,0 FAIL,10.306s. This did not reproduce the original failure and does not
close it. Paired GUI-bus traffic in the selected textarea row contains12 fresh
queries across setup/clear/typing, not12 queries for one hint: loadScript total
27.073ms/max2.697ms; run18.563ms/max2.356ms; Observe2.465ms/max0.366ms;
unloadScript1.673ms/max0.188ms. Complete call pairing is preserved in the
profile folder. No individual failed RPC stage has yet been identified.
Runtime, configuration and connection identities were restored/unchanged.
Do not increase the deadline, cache authority or edit the shared provider from
this successful sample. Next useful evidence must distinguish the original
deadline stage rather than repeat the same three inputs for green.
