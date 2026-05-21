# Development Notes

## Project

`lay` is a Caramba/Punto-style layout switcher for GNOME Wayland.

The daemon listens to physical keyboard events through `evdev`, stores recent
keycodes, and replays them through `uinput` after a manual trigger.

## Runtime Architecture

```text
physical keyboard
  -> lay-daemon
      -> WordBuffer
      -> trigger FSM
      -> backspace + layout switch + keycode replay
  -> GNOME Shell extension
      -> DBus bridge for layout switching and text insertion
```

## Important Constraints

- Do not use clipboard for daemon corrections.
- Keep simple mode deterministic and fast.
- LLM mode must be explicitly enabled.
- Production daemon must not write typed text logs by default.
- Learning logs are opt-in through config/menu.

## Useful Commands

```bash
cargo fmt --check
cargo test
cargo build --release
sudo systemctl restart lay-daemon
```

For diagnostics:

```bash
lay-daemon --debug-log
LAY_DEBUG_LOG=1 lay-daemon
sudo journalctl -u lay-daemon -f
```
