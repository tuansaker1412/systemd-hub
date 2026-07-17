//! Host system information displayed on the dashboard.

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub hostname: String,
    pub operating_system: String,
    pub kernel_version: String,
    pub uptime_seconds: u64,
}

impl SystemInfo {
    pub fn uptime_display(&self) -> String {
        crate::utils::format_duration(self.uptime_seconds)
    }
}
