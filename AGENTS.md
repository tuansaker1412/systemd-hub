# Repository Guidelines

## Project Structure & Module Organization

- `src/main.rs` starts the app and shared Tokio runtime.
- `src/app/` contains the `Adw.Application` and main window wiring.
- `src/ui/` contains GTK widgets; keep D-Bus calls out of this layer.
- `src/services/` contains business logic and async orchestration.
- `src/dbus/` contains zbus clients for `org.freedesktop.systemd1`.
- `src/models/` and `src/utils/` hold domain types and formatting helpers.
- `data/` stores desktop integration files; `docs/` stores project notes.

## Build, Test, and Development Commands

- `cargo build` builds the debug binary.
- `cargo run` runs the app locally.
- `RUST_LOG=debug cargo run` runs with debug logging through `tracing-subscriber`.
- `cargo build --release` creates an optimized release binary.
- `cargo test` runs unit and integration tests when present.
- `cargo fmt --check` verifies Rust formatting.
- `cargo clippy --all-targets --all-features` runs lint checks across targets.

Development requires systemd, GTK 4.12+, libadwaita 1.5+, Rust stable, and native build dependencies such as `pkg-config`, `libgtk-4-dev`, and `libadwaita-1-dev`.

## Coding Style & Naming Conventions

Use Rust 2021 conventions and standard `rustfmt` formatting. UI code builds widgets and sends requests; service and D-Bus modules perform system interaction. Use `snake_case` for files, modules, functions, and variables; `PascalCase` for structs, enums, traits, and GTK object types. Keep async system work off the GLib UI thread.

## Testing Guidelines

Place unit tests near the implementation with `#[cfg(test)]` modules. Put broader integration tests under `tests/` if added. Name tests after expected behavior, for example `formats_inactive_unit_state`. Prefer testing service, model, and utility logic; gate or mock tests that mutate real systemd units.

## Commit & Pull Request Guidelines

Recent history uses short Conventional Commit-style subjects such as `feat: implement Systemd Hub MVP (GTK4 + zbus)`. Follow that pattern with prefixes like `feat:`, `fix:`, `docs:`, `test:`, and `refactor:`.

Pull requests should include a concise summary, testing performed, linked issues when applicable, and screenshots or screen recordings for UI changes. Note any systemd, Polkit, or journal access implications, especially when service lifecycle behavior changes.

## Security & Configuration Tips

Service actions use the system bus and may trigger Polkit authentication. Do not bypass authorization checks or run privileged commands from the UI layer. Treat journal output and unit metadata as potentially sensitive in logs, screenshots, and test fixtures.
