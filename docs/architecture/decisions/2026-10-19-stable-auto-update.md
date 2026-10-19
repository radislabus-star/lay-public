# Stable public updates and one auto-update switch

Accepted by explicit user request on 2026-10-19. This extends the menu action
decision of 2026-10-18 without changing C01–C10 or input authority. The owning
preflight and consequences are in [stable-auto-update-2026-10-19.md](../stable-auto-update-2026-10-19.md).

`update.sh` owns discovery, scheduling controls and apply; `install.sh --update`
owns replacement of an existing installation. GNOME and KDE only adapt the same
commands. The mandatory finish rule checks the actual installed menu entrypoint
against the public stable release, not a developer branch or source test alone.

Installation exposed an already-cached GNOME loader from before 1.0.92 that
stages only the original four modules. The existing support module stages the
missing hint dependency into that loader's per-enable directory before import.
This retains one hint implementation and fresh module identities without a
desktop restart. Runtime verification waits for the asynchronous load and fails
if the loaded version is absent or stale; matching installed files alone is
insufficient.
