//! Systemd Hub — native GTK4/libadwaita systemd service manager.

mod app;
mod dbus;
mod models;
mod services;
mod ui;
mod utils;

use gtk4::prelude::*;
use once_cell::sync::Lazy;
use tracing_subscriber::EnvFilter;

/// Process-wide Tokio runtime for D-Bus and journal work.
/// UI updates always return to the GLib main loop.
pub static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("systemd-hub-async")
        .build()
        .expect("failed to create tokio runtime")
});

fn main() -> glib::ExitCode {
    // Initialize runtime before any spawn.
    Lazy::force(&RUNTIME);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let app = app::SystemdHubApplication::new();
    app.run()
}
