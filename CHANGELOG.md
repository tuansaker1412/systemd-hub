# Changelog

All notable changes to **Systemd Hub** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-07-17

First packaged release after the initial MVP. Includes settings, About, branding, packaging, and CI.

### Added

- **Settings** sidebar page with application theme selection:
  - System (follow desktop preference)
  - Light
  - Dark
  - Preference persisted under `~/.config/systemd-hub/settings` and applied via `AdwStyleManager` on startup
- **About** sidebar page with app logo, version, description, and details
- Developer credit **Ngọc Tuấn** with profile link <https://github.com/tuansaker1412>
- Project links on About (and About dialog):
  - Repository: <https://github.com/tuansaker1412/systemd-hub>
  - Issues: <https://github.com/tuansaker1412/systemd-hub/issues>
- Pointer cursor on clickable About link rows
- App logo branding on the Dashboard
- GitHub Actions **CI** (`fmt`, `clippy`, `test`, `build`) and **Release** workflows
- Multi-distro packaging scripts and artifacts:
  - AppImage
  - `.deb` (Ubuntu / Debian)
  - `.rpm` (Fedora)
  - portable `.tar.gz`
- Collapsible services inspector (Details / Logs) layout improvements

### Fixed

- Clippy warnings that blocked CI
- Rustfmt formatting for CI `fmt` check

### Changed

- Application version bumped to **0.1.2**
- About dialog metadata aligned with the About page (developer, website, issue URL, links)

## [0.1.0] - 2026-07-16

Initial MVP of Systemd Hub.

### Added

- Native GTK 4 + libadwaita desktop shell
- System dashboard (hostname, OS, kernel, uptime)
- Service list for loaded `.service` units with search, sort, and refresh
- Service detail view (ActiveState, SubState, PID, memory, ExecStart, unit file)
- Lifecycle actions over D-Bus: Start, Stop, Restart, Reload, Enable, Disable
- Journal log viewer via `journalctl` (filter, copy, refresh, follow)
- Shared Tokio runtime for async D-Bus / journal work off the GTK main loop
- Toast feedback for actions and errors
- Keyboard shortcuts: `Ctrl+Q`, `Ctrl+R`, `Ctrl+Shift+R`

[Unreleased]: https://github.com/tuansaker1412/systemd-hub/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/tuansaker1412/systemd-hub/releases/tag/v0.1.2
[0.1.0]: https://github.com/tuansaker1412/systemd-hub/releases/tag/v0.1.0
