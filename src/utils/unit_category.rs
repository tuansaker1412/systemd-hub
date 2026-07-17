//! Classify systemd units into UX categories for list filtering.

use crate::models::UnitCategory;

/// Whether the unit lives on the system manager or the user manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitScope {
    System,
    User,
}

/// Classify a unit from file path, name, scope, and unit-file enable state.
///
/// Path rules run first; vendor paths (`/usr/lib`, `/lib`) are split into
/// System vs Application via core-OS name heuristics.
pub fn classify_unit(
    unit_file_path: &str,
    name: &str,
    scope: UnitScope,
    enabled_state: &str,
) -> UnitCategory {
    if scope == UnitScope::User {
        return UnitCategory::User;
    }

    let path = unit_file_path.trim();
    let enabled = enabled_state.trim();

    if is_generated_state(enabled) || is_generated_path(path) {
        return UnitCategory::Generated;
    }

    if is_custom_path(path) {
        return UnitCategory::Custom;
    }

    if is_vendor_path(path) {
        return if is_core_unit_name(name) {
            UnitCategory::System
        } else {
            UnitCategory::Application
        };
    }

    // Unknown / empty path: prefer name heuristics, else Application.
    if is_core_unit_name(name) {
        UnitCategory::System
    } else if path.is_empty() {
        UnitCategory::Application
    } else {
        // Odd locations still count as local/admin installs.
        UnitCategory::Custom
    }
}

fn is_generated_state(enabled_state: &str) -> bool {
    enabled_state == "generated" || enabled_state == "transient"
}

fn is_generated_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    path.starts_with("/run/systemd/generator")
        || path.starts_with("/run/systemd/transient/")
        || path.contains("/systemd/generator/")
        || path.contains("/systemd/generator.early/")
        || path.contains("/systemd/generator.late/")
        || path.starts_with("/run/systemd/system/")
}

fn is_custom_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    path.starts_with("/etc/systemd/system/")
        || path.starts_with("/usr/local/lib/systemd/system/")
        || path.starts_with("/usr/local/lib/systemd/user/")
        || path.starts_with("/etc/systemd/user/")
}

fn is_vendor_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    path.starts_with("/usr/lib/systemd/system/")
        || path.starts_with("/lib/systemd/system/")
        || path.starts_with("/usr/lib/systemd/user/")
        || path.starts_with("/lib/systemd/user/")
}

/// Core OS / infrastructure unit names that should appear under System.
pub fn is_core_unit_name(name: &str) -> bool {
    let base = name.strip_suffix(".service").unwrap_or(name);
    let base_lower = base.to_ascii_lowercase();

    // Instantiated templates: match on the template stem (before '@').
    let stem = base_lower.split('@').next().unwrap_or(&base_lower);

    if stem.starts_with("systemd-")
        || stem.starts_with("dbus")
        || stem.starts_with("polkit")
        || stem.starts_with("networkmanager")
        || stem.starts_with("nm-")
        || stem.starts_with("modemmanager")
        || stem.starts_with("wpa_supplicant")
        || stem.starts_with("accounts-daemon")
        || stem.starts_with("rsyslog")
        || stem.starts_with("syslog")
        || stem.starts_with("cron")
        || stem.starts_with("anacron")
        || stem.starts_with("udev")
        || stem.starts_with("udisks")
        || stem.starts_with("upower")
        || stem.starts_with("bluetooth")
        || stem.starts_with("avahi")
        || stem.starts_with("cups")
        || stem.starts_with("gdm")
        || stem.starts_with("sddm")
        || stem.starts_with("lightdm")
        || stem.starts_with("plymouth")
        || stem.starts_with("packagekit")
        || stem.starts_with("fwupd")
        || stem.starts_with("thermald")
        || stem.starts_with("power-profiles")
        || stem.starts_with("switcheroo")
        || stem.starts_with("rtkit")
        || stem.starts_with("colord")
        || stem.starts_with("geoclue")
        || stem.starts_with("iio-sensor")
        || stem.starts_with("bolt")
        || stem.starts_with("low-memory")
        || stem.starts_with("emergency")
        || stem.starts_with("rescue")
        || stem.starts_with("initrd")
        || stem.starts_with("kmod")
        || stem.starts_with("modprobe")
        || stem.starts_with("apparmor")
        || stem.starts_with("auditd")
        || stem.starts_with("irqbalance")
        || stem.starts_with("smartmontools")
        || stem.starts_with("smartd")
        || stem.starts_with("fstrim")
        || stem.starts_with("e2scrub")
        || stem.starts_with("btrfs")
        || stem.starts_with("lvm")
        || stem.starts_with("multipath")
        || stem.starts_with("iscsi")
        || stem.starts_with("nfs-")
        || stem.starts_with("rpcbind")
        || stem.starts_with("rpc-")
        || stem.starts_with("ssh")
        || stem.starts_with("sshd")
        || stem.starts_with("getty")
        || stem.starts_with("console-")
        || stem.starts_with("serial-getty")
        || stem.starts_with("user-runtime-dir")
        || stem.starts_with("display-manager")
        || stem.starts_with("snapd")
        || stem.starts_with("xdg-desktop-portal")
        || stem.starts_with("xdg-document-portal")
        || stem.starts_with("xdg-permission-store")
        || stem.starts_with("pipewire")
        || stem.starts_with("wireplumber")
        || stem.starts_with("pulseaudio")
    {
        return true;
    }

    // Exact stems (avoid matching unrelated names like "username.service").
    matches!(
        stem,
        "dbus"
            | "polkit"
            | "networkmanager"
            | "modemmanager"
            | "wpa_supplicant"
            | "rsyslog"
            | "syslog"
            | "cron"
            | "crond"
            | "anacron"
            | "atd"
            | "systemd"
            | "udev"
            | "getty"
            | "autovt"
            | "debug-shell"
            | "emergency"
            | "rescue"
            | "halt"
            | "poweroff"
            | "reboot"
            | "kexec"
            | "suspend"
            | "hibernate"
            | "hybrid-sleep"
            | "suspend-then-hibernate"
            | "finalrd"
            | "friendly-recovery"
            | "apport"
            | "whoopsie"
            | "kerneloops"
            | "unattended-upgrades"
            | "apt-daily"
            | "apt-daily-upgrade"
            | "dpkg-db-backup"
            | "man-db"
            | "logrotate"
            | "plocate-updatedb"
            | "updatedb"
            | "motd-news"
            | "containerd"
            | "user"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_core_is_system() {
        assert_eq!(
            classify_unit(
                "/usr/lib/systemd/system/systemd-logind.service",
                "systemd-logind.service",
                UnitScope::System,
                "static"
            ),
            UnitCategory::System
        );
        assert_eq!(
            classify_unit(
                "/lib/systemd/system/NetworkManager.service",
                "NetworkManager.service",
                UnitScope::System,
                "enabled"
            ),
            UnitCategory::System
        );
        assert_eq!(
            classify_unit(
                "/usr/lib/systemd/system/dbus.service",
                "dbus.service",
                UnitScope::System,
                "static"
            ),
            UnitCategory::System
        );
    }

    #[test]
    fn vendor_apps_are_application() {
        assert_eq!(
            classify_unit(
                "/usr/lib/systemd/system/docker.service",
                "docker.service",
                UnitScope::System,
                "enabled"
            ),
            UnitCategory::Application
        );
        assert_eq!(
            classify_unit(
                "/usr/lib/systemd/system/tailscaled.service",
                "tailscaled.service",
                UnitScope::System,
                "enabled"
            ),
            UnitCategory::Application
        );
        assert_eq!(
            classify_unit(
                "/usr/lib/systemd/system/nginx.service",
                "nginx.service",
                UnitScope::System,
                "disabled"
            ),
            UnitCategory::Application
        );
    }

    #[test]
    fn etc_and_local_are_custom() {
        assert_eq!(
            classify_unit(
                "/etc/systemd/system/anydesk.service",
                "anydesk.service",
                UnitScope::System,
                "enabled"
            ),
            UnitCategory::Custom
        );
        assert_eq!(
            classify_unit(
                "/usr/local/lib/systemd/system/my-api.service",
                "my-api.service",
                UnitScope::System,
                "disabled"
            ),
            UnitCategory::Custom
        );
    }

    #[test]
    fn generators_are_generated() {
        assert_eq!(
            classify_unit(
                "/run/systemd/generator/boot-efi.mount",
                "boot-efi.mount",
                UnitScope::System,
                "generated"
            ),
            UnitCategory::Generated
        );
        assert_eq!(
            classify_unit(
                "/run/systemd/generator.late/speech-dispatcher.service",
                "speech-dispatcher.service",
                UnitScope::System,
                "generated"
            ),
            UnitCategory::Generated
        );
        assert_eq!(
            classify_unit(
                "",
                "app-foo@autostart.service",
                UnitScope::System,
                "generated"
            ),
            UnitCategory::Generated
        );
    }

    #[test]
    fn user_scope_is_user() {
        assert_eq!(
            classify_unit(
                "/usr/lib/systemd/user/pipewire.service",
                "pipewire.service",
                UnitScope::User,
                "enabled"
            ),
            UnitCategory::User
        );
    }

    #[test]
    fn instantiated_core_templates() {
        assert!(is_core_unit_name("user@1000.service"));
        assert!(is_core_unit_name("getty@tty1.service"));
        assert!(!is_core_unit_name("docker.service"));
        assert!(!is_core_unit_name("username.service"));
    }
}
