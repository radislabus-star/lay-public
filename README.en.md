<div align="center">

# Lay

### Two Shift taps. The right layout.

A local Russian / English typing assistant for Linux.<br>
Fix the last word typed in the wrong keyboard layout, then keep writing.

**[Install](#install)** · [Features](#features) · [Compatibility](#compatibility) · [Русский](README.md)

[![Latest release](https://img.shields.io/github/v/release/radislabus-star/lay-public?label=release&color=2563eb)](https://github.com/radislabus-star/lay-public/releases/latest)
[![Status: alpha](https://img.shields.io/badge/status-alpha-d97706)](#compatibility)
[![License: non-commercial](https://img.shields.io/badge/license-non--commercial-64748b)](LICENSE)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/publicity/readme-hero-dark.svg">
  <img src="docs/publicity/readme-hero.svg" alt="Illustration: ghbdtn → two successive left Shift taps → привет. Also works in reverse: руддщ → hello." width="100%">
</picture>

<sub>Illustration of manual layout correction.</sub>

</div>

Typed `ghbdtn` instead of `привет`? **Double-tap the left Shift** to fix the last word
in the current text field. It works both ways: `руддщ` → `hello`.

Manual correction is the default. Typing assistance and automatic replacement are
separate opt-in controls. Normal use needs no cloud model, API key or subscription.

<a id="install"></a>

## Install and try it

**Ubuntu / GNOME Wayland** is the primary environment. Lay currently supports
**Russian and English** and is **alpha** software. Check [compatibility](#compatibility) first.

### 1. Install Lay

```bash
curl -fsSL https://raw.githubusercontent.com/radislabus-star/lay-public/main/scripts/install-remote.sh | bash
```

This **builds from source** and needs internet access, `sudo` privileges and build time.
The [installer](scripts/install-remote.sh) installs system dependencies, installs Rust
if missing and downloads Lay's language data. An existing Rust toolchain must be **1.88 or newer**.

On KDE, the installer adds the dependencies for Lay's system tray icon.
Ubuntu 22.04 uses Qt5; Qt6 is preferred when available in the package repository.

<details>
<summary>Other distributions and the first desktop session</summary>

The installer recognizes `apt`, `pacman`, `dnf`, `yum` and `rpm-ostree`.
Installer support does not mean every application on a distribution has been tested.

On **Bazzite / Fedora Atomic**, reboot after `rpm-ostree` layers the dependencies,
then run the command above again.

Input-device access is granted to the active local session through `udev`/`logind`.
You do not need to add yourself to the `input` group. Initial desktop integration
may still require logging out and back in.

The default source directory is `~/projects/lay`.

</details>

### 2. Fix your first word

Enable Russian and English layouts. With the English layout active, type `ghbdtn`
in an ordinary text field and double-tap the **left Shift**. Expect `привет`.

### 3. Choose your assistance

Use Lay's panel menu. Manual correction is enabled by default; typing assistance
and autocorrection are separate switches. Inline suggestions and **Tab** acceptance
require active input through Lay/IBus.

<a id="features"></a>

## What Lay does

- **Fix the layout.** Double-tap left **Shift** to convert the last word RU ↔ EN. Enabled by default.

- **Get help while typing.** Typing assistance checks the word at a boundary and can fix its layout or a confirmed typo. Typing assistance and automatic replacement are separate opt-in controls.

- **Complete a word.** Lay shows a continuation; **Tab** accepts the current suggestion and adds a space. Requires active input through Lay/IBus.

- **Keep mixed-language text.** **Smart** mode considers both layouts: `good ntrcn` → `good текст`. Select it in the menu.

The menu also offers direct RU/EN selection and configurable shortcuts. Suggestions
can use confirmed choices and available context. In a recognized terminal, suitable
command completions can receive priority. **Accepting a suggestion does not run the command.**

<details>
<summary>Convert text from the command line</summary>

```bash
lay "ghbdtn"
# привет

lay "руддщ цщкдв"
# hello world

printf '%s\n' 'ghbdtn' | lay
# привет
```

The CLI converts an argument or text from standard input.

</details>

<a id="compatibility"></a>

## Compatibility

| Environment | Current state |
| --- | --- |
| **Ubuntu / GNOME Wayland** | Primary development environment; most thoroughly checked |
| **KDE / Plasma Wayland** | Integration available; fewer verified scenarios |
| **Niri Wayland** | `niri-ipc` integration available; further testing needed |
| **X11** | Experimental XKB backend |
| **Other environments** | Support not claimed yet |

The GNOME extension declares Shell **45, 46, 47 and 50**. Only **RU/EN** is supported today.

Compatibility also depends on the text field. Browsers, editors and terminals handle
input differently. Lay primarily acts on recently typed text; arbitrary selections,
a word under a moved caret and a whole document are not universally supported targets.

Full Russian grammar checking and identical behaviour in every application are not
promised. See [releases](https://github.com/radislabus-star/lay-public/releases) for changes and limitations.

<a id="privacy"></a>

## Your text and data

- **Local processing.** Normal typing runs on your computer. The cloud LLM backend is disabled by default.
- **Local learning.** Enabled learning features can store confirmed corrections and usage history on the device.
- **Opt-in diagnostics.** The normal path does not keep a complete keystroke log. Separately enabled diagnostic logs may contain text.

The daemon reads keyboard events to recognize the gesture and the last word.
Settings are stored in `~/.config/lay/config.json`.

In a supported X11 remote session, Lay can receive keyboard events through
XInput2 without a keyboard connected to the host. Check readiness with
`lay-daemon --input-status`. The **Keyboard access** menu action explains
unavailable input and offers the applicable recovery action.
See [Lay 1.0.94](docs/release-1.0.94.md) for the tested scope.

## Update or uninstall

<details>
<summary><strong>Update Lay</strong></summary>

Select the update action in Lay's panel menu, or run:

```bash
lay-update --menu
```

The updater opens a terminal and asks before building and installing. It fetches
the latest stable public release into `~/.local/share/lay/update-source` and
preserves user settings. A developer branch does not determine update availability.
GNOME and KDE menus include an **Auto-update** switch. Network failures are
reported as check errors.

</details>

<details>
<summary><strong>Uninstall while keeping settings</strong></summary>

For the default installation directory:

```bash
cd ~/projects/lay
bash uninstall.sh
```

Settings, learned data and sources remain. Use your chosen directory for a custom installation.

</details>

<details>
<summary><strong>Remove Lay and its data</strong></summary>

```bash
cd ~/projects/lay
bash uninstall.sh --purge
```

This also removes settings, learned data, caches, logs and a clean Lay source checkout.
A locally modified checkout is not removed automatically.

</details>

## Feedback and development

**[Report a bug](https://github.com/radislabus-star/lay-public/issues/new)**
or [browse issues](https://github.com/radislabus-star/lay-public/issues).
Include the Lay version, distribution, desktop, application and a short
“typed → expected → actual” example. Keep private text and sensitive logs out of reports.

Most technical documentation is in Russian:

- [How it works](HOW_IT_WORKS.md)
- [Development and checks](DEVELOPMENT.md)
- [Architecture](ARCHITECTURE.md)
- [Documentation index](docs/README.md)
- [Version history](VERSIONING.md)

## License

**Free for non-commercial use**: personal use, education, research and testing.
Commercial use requires separate written permission from the copyright holders.
See the [Lay Non-Commercial License](LICENSE).
