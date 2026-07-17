<div align="center">

![Systemd Hub Icon](data/icons/systemd-hub.svg "App Icon")

# Systemd Hub

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![CI](https://github.com/tuansaker1412/systemd-hub/actions/workflows/ci.yml/badge.svg)](https://github.com/tuansaker1412/systemd-hub/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/tuansaker1412/systemd-hub?include_prereleases&label=release)](https://github.com/tuansaker1412/systemd-hub/releases/latest)

**A native Linux desktop app for managing systemd services**

[Features](#-features) • [Installation](#-installation) • [Build from source](#-build-from-source) • [Architecture](#-architecture) • [Changelog](#-changelog) • [Contributing](#-contributing)

</div>

---

## 📋 Overview

Systemd Hub is a lightweight, native desktop application for managing systemd services. It is built with **Rust**, **GTK 4**, **libadwaita**, and **zbus** — no Electron, Tauri, or web frontend.

The app talks to systemd over D-Bus (`org.freedesktop.systemd1`) instead of spawning `systemctl` for lifecycle actions, and is designed to feel like a proper system utility:

- Browse and search loaded `.service` units
- Inspect status, PID, memory, ExecStart, and unit file path
- Start, stop, restart, reload, enable, and disable services
- Read service journal logs with filter, copy, and follow mode
- View host dashboard info (hostname, OS, kernel, uptime)

**Note:** Systemd Hub targets users who prefer a graphical interface over the command line. Experienced administrators may still prefer `systemctl` and `journalctl` directly for advanced workflows.

---

## ✨ Features

### Core functionality

| Feature | Status |
|---------|--------|
| System dashboard (hostname, OS, kernel, uptime) | ✅ |
| Service list for loaded `.service` units | ✅ |
| Search, sort, and refresh | ✅ |
| Service detail panel (ActiveState, SubState, PID, memory, ExecStart, unit file) | ✅ |
| Start / Stop / Restart / Reload | ✅ |
| Enable / Disable via D-Bus | ✅ |
| Journal log viewer (`journalctl`) | ✅ |
| Log filter, copy, refresh, and follow mode | ✅ |

### User experience

| Feature | Status |
|---------|--------|
| Native GTK 4 + libadwaita UI | ✅ |
| Theme: System / Light / Dark (Settings) | ✅ |
| Sidebar navigation (Dashboard / Services / Settings / About) | ✅ |
| About page (logo, version, developer, repo & issues) | ✅ |
| Toast feedback for actions and errors | ✅ |
| Keyboard shortcuts | ✅ |
| Unit file editor | 🔜 |
| Timers / sockets / paths views | 🔜 |
| Dependency graph | 🔜 |

---

## 🔧 Installation

### Method 1: GitHub Releases (recommended)

Download prebuilt packages from the [latest release](https://github.com/tuansaker1412/systemd-hub/releases/latest).

| Target | Artifacts |
|--------|-----------|
| **AppImage** (portable) | `.AppImage` (GTK 4 + libadwaita bundled) |
| Ubuntu 24.04 | `.tar.gz`, `.deb` |
| Debian Trixie | `.tar.gz`, `.deb` |
| Fedora (latest) | `.tar.gz`, `.rpm` |

Release assets also include `SHA256SUMS`.

> **Note:** Native packages need **GTK 4.12+** and **libadwaita 1.5+** and are linked against the distro they were built on. Prefer the package that matches your system, or use the **AppImage** for a portable build (still requires a sufficiently recent glibc; built on Ubuntu 24.04).

#### AppImage

```bash
chmod +x systemd-hub-*.AppImage
./systemd-hub-*.AppImage
```

If FUSE is unavailable:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./systemd-hub-*.AppImage
```

#### Debian / Ubuntu (`.deb`)

```bash
sudo apt install ./systemd-hub_*.deb
```

#### Fedora (`.rpm`)

```bash
sudo dnf install ./systemd-hub-*.rpm
```

#### Tarball

```bash
tar -xzf systemd-hub-*.tar.gz
cd systemd-hub-*/
./systemd-hub
```

---

### Method 2: Build from source

For developers and users who prefer building locally.

#### System requirements

| Dependency | Debian / Ubuntu | Fedora | Arch |
|------------|-----------------|--------|------|
| Rust toolchain | `cargo rustc` | `cargo rustc` | `rust` |
| GTK 4 development files | `libgtk-4-dev` | `gtk4-devel` | `gtk4` |
| Libadwaita development files | `libadwaita-1-dev` | `libadwaita-devel` | `libadwaita` |
| Build tools / pkg-config | `build-essential pkg-config` | `gcc make pkgconf` | `base-devel` |
| Runtime | systemd, journalctl | systemd | systemd |

Minimum versions: **GTK 4.12+**, **libadwaita 1.5+**, **Rust stable**.

**Debian / Ubuntu example:**

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev
```

**Additional resources:**
- [Rust installation guide](https://www.rust-lang.org/tools/install)
- [GTK 4 setup guide](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_linux.html)
- [Libadwaita setup guide](https://gtk-rs.org/gtk4-rs/stable/latest/book/libadwaita.html)

#### Build steps

1. Clone the repository:

```bash
git clone https://github.com/tuansaker1412/systemd-hub.git
cd systemd-hub
```

2. Build and run:

```bash
cargo build --release
cargo run --release
```

Debug logging:

```bash
RUST_LOG=debug cargo run
```

3. Optional local packaging (after `cargo build --release`):

```bash
VERSION=0.1.3 DISTRO=ubuntu-24.04 scripts/package-tarball.sh
VERSION=0.1.3 DISTRO=ubuntu-24.04 scripts/package-deb.sh
# On Fedora with rpm-build installed:
VERSION=0.1.3 DISTRO=fedora-latest scripts/package-rpm.sh
# AppImage (needs linuxdeploy tools; downloaded automatically):
VERSION=0.1.3 scripts/package-appimage.sh
```

---

## 🏗 Architecture

```text
GTK / libadwaita UI
        |
Application Window
        |
Service Layer
        |
D-Bus / journal / procfs
        |
systemd, journalctl, Linux system files
```

```text
src/
  main.rs          # entrypoint, tracing, shared Tokio runtime
  app/             # Adw.Application and main window wiring
  ui/              # GTK widgets (no D-Bus calls)
  services/        # business logic and async orchestration
  dbus/            # zbus client for org.freedesktop.systemd1
  models/          # domain types
  utils/           # formatting helpers
data/              # desktop integration and icons
docs/              # design notes
scripts/           # packaging helpers
```

Important rules:

- The UI never calls D-Bus directly.
- Service lifecycle actions go through `src/services/` → `src/dbus/` over the system bus.
- Blocking work runs on a shared Tokio runtime; UI updates return to the GLib main loop.
- Log reading uses `journalctl` (MVP exception for journal access).

More detail: [CODEBASE.md](CODEBASE.md).

---

## ⌨️ Shortcuts

| Action | Shortcut |
|--------|----------|
| Quit | `Ctrl+Q` |
| Refresh services | `Ctrl+R` |
| Refresh logs | `Ctrl+Shift+R` |

---

## 🚀 Releases & CI

GitHub Actions workflows live under `.github/workflows/`:

| Workflow | Trigger | What it does |
|----------|---------|--------------|
| **CI** (`ci.yml`) | Push / PR to `main` | `fmt`, `clippy`, `test`, `build` |
| **Release** (`release.yml`) | Push tag `v*` (or manual dispatch) | Multi-distro packages + GitHub Release |

### Create a release

1. Bump `version` in `Cargo.toml` and update [CHANGELOG.md](CHANGELOG.md).
2. Commit and push to `main`.
3. Tag and push:

```bash
git tag v0.1.3
git push origin v0.1.3
```

---

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for release notes. Latest: **v0.1.3**.

---

## ⚠ Notes

- Privileged service actions may trigger **Polkit** authentication on the system bus.
- Journal output and unit metadata can contain sensitive information — be careful with logs, screenshots, and test fixtures.
- Theme preference (System / Light / Dark) is configurable in **Settings** and stored under `~/.config/systemd-hub/settings`.

---

## 🤝 Contributing

Contributions are welcome. Please open an issue or pull request on [GitHub](https://github.com/tuansaker1412/systemd-hub).

Suggested workflow:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
cargo build
```

Project guidelines: [AGENTS.md](AGENTS.md) and [CODEBASE.md](CODEBASE.md).

Pull requests should include a short summary, testing notes, and screenshots for UI changes. Mention any systemd, Polkit, or journal access implications when lifecycle behavior changes.

---

## 📄 License

Systemd Hub is licensed under the **GNU General Public License v3.0 or later**. See [LICENSE](LICENSE) for details.
