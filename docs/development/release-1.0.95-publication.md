# Publication receipt — Lay 1.0.95

Publication requested explicitly by the user on 2026-10-24. This supersedes
the earlier restriction to a private KDE 1.0.94.1 build. Publication scope is
the current source, metadata, notes, commit, tag and release; no manual install
or native-input acceptance was added.

* Public source commit: `036f25e52999ce3543ae0c0937e7f27777945c98`.
* Public stable tag: `v1.0.95`, published `2026-10-24T11:21:53Z`.
* [Release](https://github.com/radislabus-star/lay-public/releases/tag/v1.0.95).
* Public `main` and annotated tag were pushed atomically, without force.
* Package, lockfile, GNOME menu and extension metadata agree on 1.0.95.
* All 801 Rust source hashes match the retained tested-source manifest.
  The only later product changes are version metadata, the explicit KDE tray
  context-menu attachment, and the derived architecture receipt.
* Guarded remote graph update and architecture gate passed; graph binding
  contains 761 `src/` Rust files with zero local-source mismatches.

## Installed updater discovery

The actual menu entrypoints in both environments call `~/.local/bin/lay-update`,
resolving to the installed `~/.local/lib/lay/update.sh`. Before publication,
both installed checks reported installed 1.0.94 and stable 1.0.94, exit 0.
After publication, both reported installed 1.0.94 and stable 1.0.95, exit 0,
using `https://api.github.com/repos/radislabus-star/lay-public/releases/latest`.
The public API returned draft=false and prerelease=false; its tag resolves to
the exact source commit above. This proves discovery, not installation.

Installed CLI remains 1.0.94 on both machines. Loaded evidence at
2026-10-24T11:24:34Z is separate:

| Environment | IME PID | Loaded SHA-256 |
|---|---|---|
| Local GNOME, DISPLAY :0 | 2711382 | `fa5e8950cc4c8c39e108464c9d06da0c00a32d26782f6dc2c030007f70e49a44` |
| Remote KDE, DISPLAY :71 | 3349193 | `2100cb595447fc2c7e034ed10fc6d6f8580b34a6191a9ba9c55eeba15703126a` |

No services, desktop sessions, model data or input routes were restarted or
changed by the publication commands. Future user or automatic updates are
separate operations.

## Incomplete gates and acceptance

[CI](https://github.com/radislabus-star/lay-public/actions/runs/40089564155):
MSRV and public-installer jobs passed. The main test targets compiled, then
`scripts/check-lay-tests.sh all` stopped because the recorded test inventory
differs from the current source. The complete correctness lane did not pass.

[Security](https://github.com/radislabus-star/lay-public/actions/runs/40089564154):
locked-dependency audit failed. The output reports RUSTSEC-2026-0382
(rustls-webpki), RUSTSEC-2026-0381 (webpki-roots), RUSTSEC-2026-0386 (tokio),
and maintenance warnings for zvariant/zvariant_derive. No dependency upgrade
or audit suppression was made for this publication.

The last full native Chrome result remains historical: 172 PASS / 9 FAIL /
39 BLOCKED out of 220 on retained private bytes. No new whole-browser or
GNOME acceptance is claimed. The KDE tray's DBus menu export was verified;
the user still reported no response to the actual icon click. The latest
user clarification narrowed the visible symptom to the tray and did not
establish whether the other Lay functions work in the ordinary profile.

Open PRs checked before publication: #64, #65, #66, #67, #68. No PR was merged,
closed or used to upgrade dependencies as part of the release.
