//! Persist and apply user preferences (theme, …).

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use libadwaita as adw;

use crate::models::AppTheme;

/// Loads/saves app settings under XDG config and applies them to libadwaita.
pub struct SettingsService;

impl SettingsService {
    /// Load the preferred theme, falling back to system when unset or invalid.
    pub fn load_theme() -> AppTheme {
        match Self::read_theme_file() {
            Ok(Some(theme)) => theme,
            Ok(None) => AppTheme::System,
            Err(err) => {
                tracing::warn!(error = %err, "failed to load theme preference; using system");
                AppTheme::System
            }
        }
    }

    /// Persist the theme preference.
    pub fn save_theme(theme: AppTheme) -> Result<()> {
        let path = Self::settings_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create config directory {}", parent.display()))?;
        }

        // Minimal key=value file so we can grow without a heavy config crate.
        let content = format!(
            "# systemd-hub user settings\ncolor-scheme={}\n",
            theme.as_str()
        );
        fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
        tracing::info!(theme = theme.as_str(), "saved theme preference");
        Ok(())
    }

    /// Apply a theme through `AdwStyleManager` (must run on the GTK main thread).
    pub fn apply_theme(theme: AppTheme) {
        let style = adw::StyleManager::default();
        style.set_color_scheme(theme.to_color_scheme());
        tracing::debug!(theme = theme.as_str(), "applied color scheme");
    }

    /// Load from disk and apply immediately.
    pub fn load_and_apply_theme() -> AppTheme {
        let theme = Self::load_theme();
        Self::apply_theme(theme);
        theme
    }

    /// Apply, then persist. Returns an error only if persistence fails.
    pub fn set_theme(theme: AppTheme) -> Result<()> {
        Self::apply_theme(theme);
        Self::save_theme(theme)
    }

    fn settings_path() -> Result<PathBuf> {
        let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else {
            let home = std::env::var("HOME").context("HOME is not set")?;
            PathBuf::from(home).join(".config")
        };
        Ok(base.join("systemd-hub").join("settings"))
    }

    fn read_theme_file() -> Result<Option<AppTheme>> {
        let path = Self::settings_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let content =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(value) = line.strip_prefix("color-scheme=") {
                return Ok(AppTheme::from_str(value));
            }
            // Single-token fallback for very old drafts.
            if let Some(theme) = AppTheme::from_str(line) {
                return Ok(Some(theme));
            }
        }

        Ok(None)
    }
}
