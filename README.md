# Systemd Hub

Native Linux desktop application for managing systemd services.

Built with **Rust**, **GTK 4**, **libadwaita**, and **zbus** (D-Bus). No Electron or web frontend.

## Features (MVP)

- **Dashboard** — hostname, OS, kernel, uptime
- **Service list** — all loaded `.service` units with search, sort, and refresh
- **Service detail** — ActiveState, SubState, PID, memory, ExecStart, unit file
- **Actions** — Start / Stop / Restart / Reload / Enable / Disable via D-Bus
- **Logs** — journal viewer with filter, copy, refresh, and follow mode

## Requirements

- Linux with systemd
- GTK 4.12+ and libadwaita 1.5+
- Rust 1.75+ (stable)
- System packages (Debian/Ubuntu example):

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev
```

## Build & run

```bash
cargo build --release
cargo run --release
```

Debug logging:

```bash
RUST_LOG=debug cargo run
```

## Architecture

```
src/
  main.rs          # entry + shared Tokio runtime
  app/             # Adw.Application + main window
  ui/              # widgets (no D-Bus)
  services/        # business logic façade
  dbus/            # zbus client for org.freedesktop.systemd1
  models/          # domain types
  utils/           # formatting helpers
```

UI never talks to D-Bus directly. All systemd I/O goes through the service layer on a background Tokio runtime; results return to the GLib main loop for UI updates.

## Shortcuts

| Action | Shortcut |
|--------|----------|
| Quit | `Ctrl+Q` |
| Refresh services | `Ctrl+R` |
| Refresh logs | `Ctrl+Shift+R` |

## Notes

- Service lifecycle actions use the system bus (`org.freedesktop.systemd1`). Privileged operations may trigger Polkit authentication.
- Log reading uses `journalctl` (allowed for the MVP journal path).
- Dark/light mode follows the system color scheme via libadwaita.

## License

GPL-3.0-or-later
