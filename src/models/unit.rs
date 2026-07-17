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

/// Visual tone for unit state labels in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTone {
    /// Healthy / running / enabled.
    Success,
    /// In transition or caution.
    Warning,
    /// Failed / masked / error.
    Error,
    /// Inactive / disabled / idle.
    Muted,
    /// Default foreground.
    Normal,
}

impl UnitSummary {
    pub fn is_running(&self) -> bool {
        self.active_state == "active"
    }

    pub fn status_label(&self) -> String {
        format!("{} / {}", self.active_state, self.sub_state)
    }

    /// Tone for ActiveState (active, failed, inactive, …).
    pub fn active_state_tone(&self) -> StateTone {
        match self.active_state.as_str() {
            "active" => StateTone::Success,
            "failed" => StateTone::Error,
            "activating" | "deactivating" | "reloading" => StateTone::Warning,
            "inactive" => StateTone::Muted,
            _ => StateTone::Normal,
        }
    }

    /// Tone for SubState (running, dead, failed, …).
    pub fn sub_state_tone(&self) -> StateTone {
        match self.sub_state.as_str() {
            "running" | "listening" | "plugged" | "mounted" | "waiting" => StateTone::Success,
            "failed" | "auto-restart" => StateTone::Error,
            "dead" | "exited" => StateTone::Muted,
            "start-pre" | "start" | "start-post" | "reload" | "stop" | "stop-watchdog"
            | "stop-sigterm" | "stop-sigkill" | "stop-post" | "final-sigterm"
            | "final-sigkill" | "auto-restart-queued" | "cleaning" => StateTone::Warning,
            _ => StateTone::Normal,
        }
    }

    /// Tone for unit-file enablement state.
    pub fn enabled_state_tone(&self) -> StateTone {
        match self.enabled_state.as_str() {
            "enabled" | "enabled-runtime" => StateTone::Success,
            "disabled" => StateTone::Muted,
            "masked" | "masked-runtime" => StateTone::Error,
            "static" | "indirect" | "generated" | "transient" | "alias" => StateTone::Normal,
            _ => StateTone::Normal,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(active: &str, sub: &str, enabled: &str) -> UnitSummary {
        UnitSummary {
            name: "test.service".into(),
            description: "Test".into(),
            load_state: "loaded".into(),
            active_state: active.into(),
            sub_state: sub.into(),
            unit_path: "/".into(),
            enabled_state: enabled.into(),
        }
    }

    #[test]
    fn active_state_tones_map_known_values() {
        assert_eq!(summary("active", "running", "enabled").active_state_tone(), StateTone::Success);
        assert_eq!(summary("failed", "failed", "enabled").active_state_tone(), StateTone::Error);
        assert_eq!(
            summary("activating", "start", "enabled").active_state_tone(),
            StateTone::Warning
        );
        assert_eq!(
            summary("inactive", "dead", "disabled").active_state_tone(),
            StateTone::Muted
        );
    }

    #[test]
    fn sub_and_enabled_state_tones_map_known_values() {
        assert_eq!(summary("active", "running", "enabled").sub_state_tone(), StateTone::Success);
        assert_eq!(summary("inactive", "dead", "disabled").sub_state_tone(), StateTone::Muted);
        assert_eq!(summary("failed", "failed", "enabled").sub_state_tone(), StateTone::Error);
        assert_eq!(
            summary("active", "running", "enabled").enabled_state_tone(),
            StateTone::Success
        );
        assert_eq!(
            summary("inactive", "dead", "disabled").enabled_state_tone(),
            StateTone::Muted
        );
        assert_eq!(
            summary("inactive", "dead", "masked").enabled_state_tone(),
            StateTone::Error
        );
    }

    #[test]
    fn status_label_joins_active_and_sub_state() {
        assert_eq!(
            summary("active", "running", "enabled").status_label(),
            "active / running"
        );
    }
}
