# Tổng Quan Codebase

## Mục Tiêu Dự Án

Systemd Hub là ứng dụng desktop Linux native để quản lý `systemd` services. Dự án dùng Rust, GTK 4, libadwaita, zbus, Tokio, anyhow và tracing. Ứng dụng không dùng Electron, Tauri hoặc web frontend.

Mục tiêu chính là tạo một công cụ hệ thống nhẹ, nhanh, có giao diện native và giao tiếp với `systemd` qua D-Bus thay vì gọi `systemctl`.

## Kiến Trúc Tổng Thể

Codebase đi theo kiến trúc phân lớp:

```text
GTK/libadwaita UI
        |
Application Window
        |
Service Layer
        |
D-Bus / journal / procfs
        |
systemd, journalctl, Linux system files
```

Quy tắc quan trọng: UI không được gọi D-Bus trực tiếp. Mọi thao tác với `systemd` phải đi qua `src/services/`, sau đó service layer mới gọi `src/dbus/`.

## Cấu Trúc Thư Mục

```text
src/
  main.rs              # entrypoint, tracing, Tokio runtime dùng chung
  app/                 # Adw.Application và cửa sổ chính
  ui/                  # GTK widgets, list, detail, logs, dashboard, sidebar
  services/            # business logic và async orchestration
  dbus/                # zbus client cho org.freedesktop.systemd1
  models/              # kiểu dữ liệu domain
  utils/               # helper định dạng
data/                  # desktop integration
docs/                  # tài liệu thiết kế và MVP
```

## Runtime Và Luồng Async

`src/main.rs` tạo một Tokio runtime toàn tiến trình bằng `once_cell::sync::Lazy`. Các tác vụ D-Bus và đọc journal chạy trên runtime này để tránh khóa GTK main loop.

Trong `src/app/window.rs`, cửa sổ chính spawn task vào `RUNTIME`, nhận kết quả qua `async_channel`, rồi cập nhật UI bằng `glib::spawn_future_local`. Cách này giữ UI responsive và đảm bảo cập nhật widget diễn ra trên GLib main loop.

## Application Layer

`src/app/application.rs` định nghĩa `SystemdHubApplication`. `src/app/window.rs` là trung tâm điều phối UI:

- dựng layout `Sidebar | Service List | Detail + Logs`;
- xử lý shortcut như refresh services và refresh logs;
- lưu service đang được chọn;
- gọi `UnitService`, `SystemInfoService`, `JournalService`;
- hiển thị lỗi và trạng thái qua `ToastOverlay`.

## UI Layer

`src/ui/` chỉ chịu trách nhiệm hiển thị và phát signal:

- `dashboard.rs`: hiển thị hostname, OS, kernel, uptime.
- `service_list.rs`: danh sách service, search, sort, selection.
- `service_detail.rs`: thông tin chi tiết và các nút action.
- `log_viewer.rs`: journal logs, filter, follow mode, copy.
- `sidebar.rs`: điều hướng giữa Dashboard và Services.
- `unit_object.rs`: object wrapper để bind dữ liệu service vào GTK model/view.

UI layer không biết chi tiết D-Bus, Polkit hoặc systemd proxy.

## Service Layer

`src/services/units.rs` cung cấp `UnitService`, facade cho list, detail và lifecycle action. Service này cache `SystemdClient` trong `Arc<Mutex<Option<_>>>` và khởi tạo D-Bus connection khi cần.

`src/services/system_info.rs` đọc thông tin host từ `/etc/hostname`, `/etc/os-release`, `/proc/version` và `/proc/uptime`.

`src/services/journal.rs` đọc log bằng `journalctl -u <unit> --no-pager -n <lines> --output=short-iso`. Đây là ngoại lệ được MVP cho phép vì chỉ dùng cho log reading.

## D-Bus Layer

`src/dbus/systemd.rs` định nghĩa zbus proxy cho:

- `org.freedesktop.systemd1.Manager`;
- `org.freedesktop.systemd1.Unit`;
- `org.freedesktop.systemd1.Service`.

`SystemdClient` hỗ trợ:

- `list_services()`: gọi `ListUnits` và `ListUnitFiles`, lọc `.service`;
- `get_unit_detail()`: đọc ActiveState, SubState, MainPID, memory, ExecStart, unit file;
- `perform_action()`: Start, Stop, Restart, Reload, Enable, Disable.

Enable/Disable gọi `EnableUnitFiles` hoặc `DisableUnitFiles`, sau đó reload systemd manager.

## Domain Models

`src/models/unit.rs` chứa:

- `UnitSummary`: dữ liệu cho từng dòng trong service list.
- `UnitDetail`: dữ liệu đầy đủ cho service đang chọn.
- `ServiceAction`: enum cho Start, Stop, Restart, Reload, Enable, Disable.

Các model nên giữ logic domain nhỏ, ví dụ `status_label()` hoặc `is_running()`, không chứa logic UI hoặc D-Bus.

## Tính Năng MVP Hiện Có

- Dashboard hệ thống.
- Danh sách `.service` unit.
- Chi tiết service.
- Start, Stop, Restart, Reload, Enable, Disable qua D-Bus.
- Log viewer qua `journalctl`.
- Search, refresh, follow logs.
- Giao diện libadwaita hỗ trợ light/dark mode theo hệ thống.

## Lệnh Phát Triển

```bash
cargo build
cargo run
RUST_LOG=debug cargo run
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features
```

Yêu cầu hệ thống: Linux có systemd, Rust stable, GTK 4.12+, libadwaita 1.5+ và các package native như `pkg-config`, `libgtk-4-dev`, `libadwaita-1-dev`.

## Quy Tắc Khi Mở Rộng

- Không gọi `systemctl` cho lifecycle action; dùng D-Bus qua zbus.
- Không để UI truy cập trực tiếp `src/dbus/`.
- Không chạy tác vụ blocking trên GTK main loop.
- Luôn trả lỗi bằng `anyhow::Result` ở service/dbus layer.
- Log lỗi quan trọng bằng `tracing`.
- Giữ module nhỏ, tập trung đúng trách nhiệm.
- Với tính năng liên quan quyền hệ thống, kiểm tra tác động Polkit và system bus.

## Điểm Cần Lưu Ý

Service actions có thể kích hoạt Polkit authentication. Journal và unit metadata có thể chứa thông tin nhạy cảm, vì vậy cần cẩn thận khi log, chụp màn hình hoặc tạo fixture test.
