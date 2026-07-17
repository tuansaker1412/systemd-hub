# Project: Native Linux Systemd Manager

Build a native desktop application for Linux to manage systemd services.

## Goal

Create a lightweight, fast, production-quality application that feels like a native Linux system utility.

The application must **NOT** use Electron, Tauri or any web frontend.

## Tech Stack

* Rust (stable)
* GTK4
* libadwaita
* zbus (communicate with systemd over D-Bus)
* tokio
* serde
* anyhow
* tracing

Do not execute `systemctl` commands by spawning shell processes unless absolutely necessary. Prefer communicating directly with `org.freedesktop.systemd1` via D-Bus using zbus.

For viewing logs, use the systemd journal API if practical. If necessary, use `journalctl` only for log reading.

---

## Architecture

Use a clean architecture.

```
src/

main.rs

app/

ui/

services/

dbus/

models/

utils/
```

Separate UI from business logic.

The UI must never directly access D-Bus.

Only the service layer communicates with systemd.

---

## First Version Features

### 1. Dashboard

Display:

* hostname
* operating system
* kernel version
* uptime

---

### 2. Service List

Display all systemd services.

Columns:

* Service name
* Description
* Status
* Enabled
* Running state

Support:

* search
* refresh
* sorting

---

### 3. Service Detail

Selecting a service displays:

* Description
* ActiveState
* SubState
* Main PID
* Memory usage (if available)
* ExecStart
* Unit file path

Buttons:

* Start
* Stop
* Restart
* Reload
* Enable
* Disable

These actions should communicate with systemd through D-Bus.

---

### 4. Log Viewer

Display logs for the selected service.

Support:

* latest logs
* auto refresh
* follow mode
* search
* copy text

Initially load the most recent entries.

---

## UI

Use libadwaita widgets.

Layout:

```
+-------------------------------------------+
| Sidebar | Service List | Detail + Logs    |
+-------------------------------------------+
```

Use:

* NavigationSplitView
* ColumnView
* HeaderBar
* ToolbarView
* ToastOverlay

Support:

* Dark mode
* Light mode
* HiDPI
* Wayland
* X11

---

## Code Quality

Follow Rust best practices.

Requirements:

* modular
* documented
* strongly typed
* minimal unsafe code
* proper error handling using anyhow
* logging using tracing
* async where appropriate

Avoid large files.

Keep each module focused.

---

## Project Roadmap

Implement incrementally.

Step 1:

* initialize project
* dependency setup
* application window
* sidebar
* empty pages

Step 2:

* D-Bus connection
* list services

Step 3:

* service detail

Step 4:

* service actions

Step 5:

* journal viewer

Step 6:

* polish UI

Do not implement everything at once.

After each step, ensure the project compiles successfully before proceeding.

Always keep the project in a buildable state.
