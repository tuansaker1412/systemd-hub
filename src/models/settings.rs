//! Application preference models.

use libadwaita as adw;

/// Preferred application color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTheme {
    /// Follow the desktop light/dark preference.
    #[default]
    System,
    /// Always use light style.
    Light,
    /// Always use dark style.
    Dark,
}

impl AppTheme {
    pub const ALL: [AppTheme; 3] = [AppTheme::System, AppTheme::Light, AppTheme::Dark];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "system" | "default" | "auto" => Some(Self::System),
            "light" | "force-light" => Some(Self::Light),
            "dark" | "force-dark" => Some(Self::Dark),
            _ => None,
        }
    }

    pub fn from_index(index: u32) -> Option<Self> {
        Self::ALL.get(index as usize).copied()
    }

    pub fn index(self) -> u32 {
        match self {
            Self::System => 0,
            Self::Light => 1,
            Self::Dark => 2,
        }
    }

    pub fn to_color_scheme(self) -> adw::ColorScheme {
        match self {
            Self::System => adw::ColorScheme::Default,
            Self::Light => adw::ColorScheme::ForceLight,
            Self::Dark => adw::ColorScheme::ForceDark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_round_trips_theme_values() {
        assert_eq!(AppTheme::from_str("system"), Some(AppTheme::System));
        assert_eq!(AppTheme::from_str("Light"), Some(AppTheme::Light));
        assert_eq!(AppTheme::from_str("FORCE-DARK"), Some(AppTheme::Dark));
        assert_eq!(AppTheme::from_str("nope"), None);

        for theme in AppTheme::ALL {
            assert_eq!(AppTheme::from_str(theme.as_str()), Some(theme));
            assert_eq!(AppTheme::from_index(theme.index()), Some(theme));
        }
    }
}
