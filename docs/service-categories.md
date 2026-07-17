# Plan: Phân loại Services (System / Applications / Custom / User / Generated)

## Mục tiêu

Thêm phân loại unit systemd theo 5 nhóm UX, kèm **filter chips** trên trang Services. Mục tiêu giúp người dùng tập trung vào service họ quan tâm (Applications) thay vì danh sách hàng trăm unit lõi/generated.

| Chip | Ý nghĩa | Nguồn dữ liệu chính |
|------|---------|---------------------|
| Applications (mặc định) | App cài thêm: nginx, docker, tailscale… | Vendor path + **không** thuộc core OS |
| System | Core OS: systemd-*, dbus, NetworkManager, polkit… | Vendor path + core heuristics / package |
| Custom | User/admin tự thêm | `/etc/systemd/system`, `/usr/local/...` |
| User | Session user units | User D-Bus (`systemctl --user`) — reserved for follow-up |
| Generated | Unit sinh động | `/run/systemd/generator*`, state generated… |

**Mặc định UI:** chip **Applications** bật; **Generated** ẩn (chỉ hiện khi chọn chip Generated hoặc All).

---

## Bối cảnh kỹ thuật

- `SystemdClient::list_services()` gọi `ListUnitFiles` → có đường dẫn unit file.
- `UnitSummary` mang `unit_file_path` + `category`.
- Classification thuộc **utils layer**; UI chỉ filter.
- App hiện kết nối **system bus**; category `User` dành cho user session units (list user bus có thể bổ sung sau).

### Ràng buộc

1. **Applications vs System cùng path** (`/usr/lib/systemd/system/`) → heuristic tên unit + (tuỳ chọn) package owner.
2. **Custom vs enable symlink**: definition path từ ListUnitFiles là path unit thật, không phải symlink trong `*.wants/`.
3. Không dùng `systemctl` cho lifecycle; D-Bus giữ nguyên.

---

## Thuật toán phân loại

```text
if scope == User → User

if path under /run/systemd/generator*, transient, or enabled_state generated/transient
    → Generated

if path under /etc/systemd/system/ or /usr/local/lib/systemd/system/
    → Custom

if path under /usr/lib/systemd/system/ or /lib/systemd/system/
    if is_core_os(name) → System
    else → Application

fallback: core name heuristics → System, else Application
```

### Core OS name heuristics

Prefix/tên: `systemd-`, `dbus`, `polkit`, `NetworkManager`, `ModemManager`, `wpa_supplicant`, `accounts-daemon`, `rsyslog`, `cron`, `anacron`, `getty@`, `user@`, `user-runtime-dir@`, …

### Package owner (follow-up)

Map path → package qua dpkg/rpm; gói core (`systemd`, `dbus`, `network-manager`, …) → System.

---

## Thay đổi theo layer

| Layer | Thay đổi |
|-------|----------|
| `models/unit.rs` | `UnitCategory`, fields trên `UnitSummary` |
| `utils/unit_category.rs` | `classify_unit` + tests |
| `dbus/systemd.rs` | Giữ path, gán category |
| `ui/service_list.rs` | Filter chips + CustomFilter |
| `ui/unit_object.rs` | `category()` |

---

## UI

Single-select chips: `Applications | System | Custom | User | Generated | All`, default **Applications**.

Status bar: filtered count with category label.

---

## Follow-ups

1. Package index (dpkg/rpm) cho System vs App chính xác hơn.
2. List user bus units → chip User có dữ liệu.
3. Cột/badge Origin (tuỳ chọn).
