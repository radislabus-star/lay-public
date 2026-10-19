# Lay 1.0.91

This release publishes the input fixes accepted by the user on 2026-10-19.
The accepted source was built as 1.0.90; the publication changes its version
metadata and records the existing evidence. No new build, functional test,
installation or service restart was performed for publication.

- Adds KDE foreground-window and layout synchronization support to the existing
  daemon and IBus ownership contracts.
- Preserves fresh client observations across focus, Reset and composition
  transitions without granting cached text edit authority.
- Corrects active-composition commit ordering: commit the authorized text before
  retiring its presentation, and retire committed state even if hiding the
  presentation subsequently fails. This recovers the measured Firefox
  two-word Space correction without replaying already committed text.
- Preserves the preceding word during two-word boundary correction in native
  editors, including the measured KWrite case.
- Includes a narrowly matched KDE IBus candidate-panel focus rule, its preserving
  installer/uninstaller, and documented IBus/KWin compatibility patch experiments.
- Expands native client observation and regression controls. The Double Shift
  detector, typed output routes, caret/selection guards and final edit verifier
  retain their existing authority. Models and learning data are unchanged.

## Accepted native results

The complete, independently closed remote KDE matrix uses one pinned candidate
and dependency bundle. These are original grades, including deferred failures.

| Application | PASS | FAIL | BLOCKED | Total |
| --- | ---: | ---: | ---: | ---: |
| Firefox | 150 | 10 | 0 | 160 |
| Chrome | 150 | 10 | 0 | 160 |
| **Browsers subtotal** | **300** | **20** | **0** | **320** |
| GTK3 | 29 | 3 | 0 | 32 |
| Qt5 | 45 | 3 | 0 | 48 |
| KWrite | 14 | 1 | 1 | 16 |
| Konsole | 3 | 10 | 3 | 16 |
| **Total** | **391** | **37** | **4** | **432** |

NOT_TESTED within this matrix: 0. Fresh GNOME acceptance of the same bytes:
NOT_TESTED. Relative to the immediately preceding closed matrix, Firefox
field 6 / moved_boundary changes from FAIL to PASS; all other grades remain.

The user deferred repair of 26 selected-text FAIL cases. The other failures
are one GTK3 focus-return setup with an observed selection and ten Konsole
cases. Four cases remain BLOCKED. These cases are accepted limitations of this
publication, not claims of successful input handling.

## Measured latency

Successful native Double Shift operations, including observer time:

| Application | Samples | Median, ms | p95, ms | Maximum, ms |
| --- | ---: | ---: | ---: | ---: |
| Firefox | 200 | 29.916 | 175.830 | 184.136 |
| Chrome | 200 | 30.488 | 180.291 | 225.163 |
| GTK3 | 40 | 28.000 | 164.628 | 167.059 |
| Qt5 | 60 | 28.335 | 139.841 | 169.122 |
| KWrite | 20 | 29.391 | 142.072 | 145.628 |

Konsole has no successful Double Shift latency sample. Failed and blocked
operations remain in the acceptance counts; the latency table does not cover
them. The recovered Firefox two-word Space correction took 31.587 ms. These
measurements establish this run's latency, not a controlled speedup claim.

## Dependency and update scope

The KDE results above require the measured private IBus and KWin builds.
They do not qualify stock KDE or Plasma 6. The compatibility patches and their
limitations are documented in [scripts/compat/README.md](../scripts/compat/README.md);
the ordinary Lay updater does not replace system IBus or KWin with those builds.

The user will update through **“Проверить обновления”**. Publication does not
change the currently installed version.

Portable counts, source/binary/dependency pins, closure hashes and original
receipt locations are recorded in
[the accepted evidence index](../tech_debt/evidence/td159-accepted-native-432-20261019.json).
The owning investigation remains
[Firefox input stability](architecture/firefox-input-stability-2026-10-05.md).
