# Restore update action in GNOME and KDE menus

Date: 2026-10-18. Status: accepted by explicit user request.

The user requested “добавь его”, “и обнови меня”, “переработай меню” and
“учти что меню еще в KDE было”. This supersedes the old menu inventory decision
that excluded Check updates. It does not amend C01–C10 or runtime input authority.

Both trays expose the same daily grouping and the installed `lay-update --menu`
entrypoint. `update.sh` remains the updater owner, `install.sh` the installation
owner and `tray_support.js` the JS adapter. Check and confirmation precede apply;
there is no tray-local update pipeline. Existing UI prohibition is replaced by
required-action, owner and order contracts, with isolated executable updater tests.

Rejected alternatives: a link only to GitHub does not update the application;
calling hardcoded ~/projects/lay may install an unrelated checkout; a second
release downloader would create another update owner and is unnecessary here.

Scope: new menu and installation identities require current evidence. Retained
Firefox/Chrome input receipts are historical and do not certify new bytes.
