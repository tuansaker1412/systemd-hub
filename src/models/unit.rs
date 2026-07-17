//! Systemd unit models.

/// Summary row for the service list.
#[derive(Debug, Clone)]
pub struct UnitSummary {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub unit_path: String,
    pub enabled_state: String,
}

impl UnitSummary {
    pub fn is_running(&self) -> bool {
        self.active_state == "active"
    }

    pub fn status_label(&self) -> String {
        format!("{} / {}", self.active_state, self.sub_state)
    }
}

/// Full detail for the selected service.
#[derive(Debug, Clone, Default)]
pub struct UnitDetail {
    pub name: String,
    pub description: String,
    pub active_state: String,
    pub sub_state: String,
    pub load_state: String,
    pub main_pid: u32,
    pub memory_bytes: Option<u64>,
    pub exec_start: String,
    pub fragment_path: String,
    pub enabled_state: String,
    pub unit_path: String,
}

/// Actions that can be performed on a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
}

impl ServiceAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Restart => "restart",
            Self::Reload => "reload",
            Self::Enable => "enable",
            Self::Disable => "disable",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::Stop => "Stop",
            Self::Restart => "Restart",
            Self::Reload => "Reload",
            Self::Enable => "Enable",
            Self::Disable => "Disable",
        }
    }
}
