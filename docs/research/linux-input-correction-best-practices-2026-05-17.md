# Linux input correction research

Date: 2026-05-17

Scope: prior art and best practices for `lay`: wrong-layout rescue, safe typing
assist, multi-word correction, Linux desktop compatibility, packaging, and public
distribution.

This document intentionally looks beyond GitHub. It includes older FOSS projects,
Linux desktop input-method architecture, distribution platforms, and spelling
correction research.

## Short conclusion

`lay` should stay a local deterministic keyboard reflex first, not become a
creative text rewriting assistant.

The strongest architecture is layered:

1. Low-level evdev listener and uinput/text backend for fast manual rescue.
2. Conservative word-buffer model for exact local context.
3. Candidate generator: layout flip, typo edit, split/join, protected token.
4. Scorer/arbiter: dictionaries, char n-gram, LEM/noisy-channel style score.
5. Optional desktop-specific atomic edit backend where the platform is reliable.
6. Optional model/LLM only as a late arbiter for ambiguous candidates, not as the
   primary writer.

The product rule should be:

Manual double Shift must always let the user override the system. Automatic
correction must act only when the confidence is very high and the edit is small.

## Project families found

### 1. Old layout switchers

Examples:

- XNeur / gxneur
- Tapper
- old Punto Switcher-like utilities
- small DE-specific scripts around XKB

What they prove:

- There is a real long-running user need: automatic or hotkey-based correction
  of wrong keyboard layout.
- Manual rescue remains valuable even when automatic detection exists.
- X11-era tools often do not map cleanly to GNOME/KDE Wayland.
- Desktop-specific bridges are normal in this domain.

Useful signals:

- Tapper supports modifier-tap layout switching and explicitly lists GNOME, KDE,
  Xfce, i3, LXQt and others. On GNOME Wayland it still needs a Shell extension.
- XNeur historically supports automatic and manual layout switching, but it is
  X Window System-oriented.

Implication for `lay`:

`lay` should not try to pretend that one backend covers every Linux desktop. A
small stable core plus desktop adapters is the right direction.

### 2. Low-level keyboard daemons

Examples:

- keyd
- KMonad
- Keymasq
- interception-tools-style designs

What they prove:

- evdev/uinput is the normal Linux way to build display-server-independent
  keyboard tools.
- Fast daemons should keep the hot path tiny and deterministic.
- System services and explicit permissions are acceptable for this class of
  software.
- Keyboard tools need diagnostics: monitor mode, current backend, active device,
  layout state, and recent actions.

Useful signals:

- keyd describes its design as a system-wide daemon using kernel-level input
  primitives `evdev` and `uinput`.
- keyd values speed, simplicity, consistency, and a small daemon core.
- KMonad shows that multi-tap, tap-hold and layers are established keyboard
  concepts, but it is a keyboard manager, not a text correction engine.

Implication for `lay`:

The current evdev/uinput path is not a hack. It is a legitimate core for fast
keyboard behavior. But text replacement must be careful because uinput replay is
not atomic: delays, modifier state, focus changes, and app behavior can still
matter.

### 3. Input methods: IBus, Fcitx, Wayland text-input

Examples:

- IBus Typing Booster
- Fcitx/Fcitx5
- Wayland `text-input-v3` / `input-method-v2`

What they prove:

- Architecturally, the cleanest text edit is not "press Backspace N times and
  type text", but "delete surrounding text + commit string".
- IME-style tools can provide candidate lists, inline completion, spell checking,
  dictionaries, and user learning.
- The Linux desktop reality is messy: surrounding text behavior differs between
  GTK, Qt, Wayland protocol versions, terminals, non-native widgets and browser
  stacks.

Useful signals:

- Wayland text-input has explicit concepts for surrounding text, commit string
  and delete surrounding text.
- Fcitx developer notes warn that surrounding text is unreliable across APIs,
  encodings and toolkits, and recommends deleting surrounding text only when the
  operation is extremely reliable or making such features optional.
- IBus Typing Booster uses Hunspell dictionaries, learns from user input, and can
  work with multiple languages at the same time.
- IBus Typing Booster documentation says inline completion is hard to see on
  Wayland because preedit styling cannot be changed there in the expected way.

Implication for `lay`:

An IME backend is a serious research direction, but it should not replace the
current fast backend until it is proven in GNOME, KDE, browsers, terminals,
Electron apps and WeChat-like apps. It should be added as an optional backend
with a fallback.

### 4. Text expanders and personal autocorrect

Examples:

- Espanso
- AutoKey
- snippet/package systems

What they prove:

- Users accept local background text tools when they are transparent,
  privacy-first and configurable.
- File-based config and user-owned rules are a good model.
- Packaging must account for X11 vs Wayland differences.
- Cross-app text replacement is hard enough that mature projects document
  different install paths/backends for Linux session types.

Useful signals:

- Espanso positions itself as privacy-first, local, cross-platform and
  system-wide.
- Espanso documents that Linux users must first identify X11 vs Wayland and then
  choose the matching install method.
- Text expander ecosystems use packages/config files, but they usually do exact
  expansion, not probabilistic correction.

Implication for `lay`:

`lay` should keep user rules in files and UI switches. It should not bury
personal corrections as production code. Public documentation must explain what
is local, what is logged, and how to disable learning/correction layers.

### 5. Spelling correction mathematics

Relevant ideas:

- noisy-channel model
- edit distance / Damerau-Levenshtein distance
- word frequency priors
- character n-grams
- Viterbi / dynamic programming over token states
- candidate ranking with a margin between winner and runner-up

What they prove:

- The best classical spelling correction model is not "replace with closest
  dictionary word". It is "choose the most probable intended text given observed
  noisy input".
- Candidate generation and candidate scoring should be separate.
- The model needs both a language prior and an error model.
- Short words and real-word errors are dangerous because the wrong candidate can
  also be a valid word.

Good scoring shape for `lay`:

```text
score(candidate) =
    language_score(candidate)
  + dictionary_score(candidate)
  + layout_error_score(original, candidate)
  + user_history_score(candidate)
  - edit_distance_penalty(original, candidate)
  - intervention_penalty
  - protected_token_penalty
```

For 2-3-N words, the best mental model is Viterbi over token states:

```text
token state:
  KEEP_ORIGINAL
  FLIP_LAYOUT
  TYPO_FIX
  JOIN_WITH_PREVIOUS
  SPLIT_TOKEN
  PROTECT_TOKEN
```

Then pick the best full path only if it wins with a strong margin. If not,
manual double Shift can still act, but automatic correction must stay quiet.

Implication for `lay`:

The current LEM/ngram direction is aligned with old, proven spelling-correction
math. The next important step is not another list of special cases. It is a
unified candidate lattice and scorer, with the user's examples as regression
tests.

## Best practices for `lay`

### Correction behavior

- Keep manual double Shift as the trusted escape hatch.
- Never block manual reversal because the scorer thinks the text is already
  "good".
- Automatic correction should require a strong margin, especially for:
  - short words;
  - one-letter words;
  - mixed RU/EN text;
  - technical tokens;
  - acronyms;
  - commands;
  - prices and punctuation;
  - hyphenated words.
- Protect technical and code-like spans by default:
  - URLs;
  - emails;
  - file paths;
  - CLI flags;
  - uppercase acronyms;
  - package names;
  - `VPN`, `KDE`, `GNOME`, `NTFS`, `TRX`, etc. as a class, not hardcoded one by
    one.
- Treat examples from chat/logs as tests, not runtime branches.

### Architecture

- Keep `src/bin/lay_daemon.rs` thin.
- Put correction decisions into modules:
  - candidate generation;
  - token classification;
  - scoring;
  - edit plan;
  - backend execution.
- Keep desktop backends separate:
  - GNOME Wayland;
  - KDE Wayland;
  - X11;
  - experimental IME.
- Add backend capability flags:
  - can switch layout;
  - can type text;
  - can delete surrounding text;
  - can read selection;
  - can update tray/menu;
  - can run atomic replace.
- Keep hot dictionaries/scorers warmed when enabled.

### Testing

The minimum serious test set should include:

- one-word wrong-layout rescue;
- repeated manual toggling four times;
- 2-word mixed RU/EN;
- 3-word mixed RU/EN;
- hyphenated words;
- commands and CLI flags;
- punctuation-heavy text;
- prices: `15р-16р`, `4000;`, `$`, `%`;
- acronyms and uppercase spans;
- browser text fields;
- terminal text fields;
- KDE Wayland VM;
- GNOME Wayland host;
- selected-text conversion;
- auto-correction after Space;
- Enter autocorrect when enabled;
- undo of last correction.

Every user bug should become one regression test in the right layer. The test
name should describe the general failure, not just the literal word.

### UX

Recommended tray modes:

- Manual rescue only.
- Typing assist after Space.
- Auto-switch layout.
- Enter autocorrect.
- Learning / remember corrections.
- Experimental scorers:
  - n-gram;
  - LEM;
  - model arbiter.
- Backend:
  - Auto;
  - GNOME;
  - KDE;
  - X11;
  - experimental IME.

Recommended public wording:

- "Fast local keyboard helper for RU/EN layout mistakes."
- "Manual double Shift is always available."
- "Automatic correction is conservative and configurable."
- "Everything runs locally."
- "GNOME is the most mature path; KDE/X11 are newer backends."

Avoid public wording that implies:

- full Grammarly replacement;
- perfect automatic correction;
- full support for every Linux desktop;
- LLM-first behavior;
- all languages.

## Distribution platforms

### GitHub

Best as the main project home now:

- existing users are already there;
- issues and PRs are active;
- releases and install scripts are enough for early adoption;
- easy for Habr/LOR users to link.

Need:

- clean README;
- issue templates;
- release notes;
- source archive install path;
- security/privacy note;
- clear supported-platform matrix.

### Codeberg / Forgejo

Good as a mirror for FOSS trust:

- non-commercial, free-software-oriented audience;
- useful for users who dislike GitHub;
- can host pages, releases, issues, translations.

Recommended:

- mirror repository after GitHub docs are stable;
- do not split issues at first;
- README should say where canonical issues live.

### AUR

Very important for Arch/Manjaro users:

- keyboard/power users are common there;
- AUR is normal for tools that need system-level integration;
- users can inspect PKGBUILD.

Recommended package split:

- `lay-git` first;
- later `lay-bin` or stable `lay`.

### openSUSE OBS / Fedora COPR

Good second wave:

- Tapper already uses COPR/RPM route successfully;
- useful for Fedora/openSUSE users;
- better than asking everyone to run install scripts.

Recommended after:

- install layout is stable;
- KDE backend is less experimental;
- daemon and desktop files are clean.

### Nix / Nixpkgs / flake

Good for reproducible users:

- keyd and IBus Typing Booster appear in distro package ecosystems;
- Nix users like declarative service setup;
- helps test dependency cleanliness.

Recommended:

- keep a flake or packaging expression after paths are stable.

### Flathub

Probably not the primary path for `lay`.

Reason:

- Flathub is optimized for sandboxed desktop apps.
- `lay` needs low-level input access, services, desktop integration and possibly
  extensions.
- Flathub asks for minimal static permissions and portal-first integration.

Possible later use:

- only for a GUI/config companion, not the low-level daemon itself.

### GNOME Extensions

Useful only for the GNOME bridge, not the whole product.

Reason:

- The extension alone cannot replace the daemon.
- GNOME Shell API compatibility changes by version.

Recommended:

- keep extension install under `install.sh` for now;
- publish separately only after the daemon/extension contract is stable.

### SourceForge

Not the primary development home, but still relevant for discovery by older
Linux users and "download utility" audiences.

Recommended:

- create a mirror/download page only after stable releases exist.

## Concrete roadmap from this research

### Phase 1: harden what exists

- Keep uinput backend as default fast path.
- Keep typing assist conservative.
- Keep LEM/ngram hot.
- Expand corpus tests around real mixed-language sentences.
- Add command/report bundle for issues.
- Add release notes and update docs.

### Phase 2: cleaner correction core

- Make candidate generation explicit:
  - original;
  - layout-flipped;
  - typo-fixed;
  - split/join candidates;
  - protected token candidate.
- Score all candidates with one function.
- Use margin against second-best candidate.
- Apply automatic correction only if:
  - score margin is high;
  - edit distance is small;
  - protected-token checks pass;
  - spacing plan is safe.

### Phase 3: backend capability layer

- Formalize backend capabilities in code.
- Keep GNOME/KDE/X11 separate.
- Add experimental IME/atomic replace prototype behind a hidden/advanced switch.
- Never remove uinput fallback until IME works in:
  - GNOME text fields;
  - KDE text fields;
  - terminal;
  - Firefox/Chrome;
  - Electron apps;
  - WeChat-like apps.

### Phase 4: package and publish

- GitHub releases.
- `lay-git` AUR.
- Codeberg mirror.
- Fedora COPR/openSUSE OBS if demand continues.
- Nix expression/flake.
- Flathub only for a GUI companion, not the daemon.

## Sources

- IBus Typing Booster: https://mike-fabian.github.io/ibus-typing-booster/
- IBus Typing Booster user docs: https://mike-fabian.github.io/ibus-typing-booster/docs/user/
- Fcitx surrounding text article: https://www.csslayer.info/wordpress/fcitx-dev/why-surrounding-text-is-the-worst-feature-in-the-linux-input-method-world/
- Wayland text-input-v3 protocol: https://wayland.app/protocols/text-input-unstable-v3
- keyd: https://github.com/rvaiya/keyd
- KMonad: https://github.com/kmonad/kmonad
- Tapper: https://kbd-tapper.sourceforge.io/en.html
- XNeur settings/docs: https://xneur.ru/settings/
- Espanso Linux install docs: https://espanso.org/docs/install/linux/
- Espanso GitHub: https://github.com/espanso/espanso
- Norvig spelling corrector: https://norvig.com/spell-correct.html
- Codeberg documentation: https://docs.codeberg.org/getting-started/what-is-codeberg/
- Flathub requirements: https://docs.flathub.org/docs/for-app-authors/requirements
- SourceForge about: https://sourceforge.net/about
