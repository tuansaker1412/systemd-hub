//! Journal log reader for a unit.
//!
//! Uses `journalctl` for log reading (allowed by the MVP spec when the
//! native journal API is not practical).

use anyhow::{bail, Context, Result};
use tokio::process::Command;

/// A single log line from the journal.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub line: String,
}

pub struct JournalService;

impl JournalService {
    /// Fetch the most recent `lines` log entries for `unit`.
    pub async fn fetch_logs(unit: &str, lines: u32) -> Result<Vec<LogEntry>> {
        if unit.is_empty() {
            bail!("unit name is empty");
        }

        let output = Command::new("journalctl")
            .args([
                "-u",
                unit,
                "--no-pager",
                "-n",
                &lines.to_string(),
                "--output=short-iso",
            ])
            .output()
            .await
            .context("failed to spawn journalctl")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // journalctl returns non-zero when no entries; treat as empty.
            if stderr.contains("No entries") || output.stdout.is_empty() {
                return Ok(Vec::new());
            }
            bail!("journalctl failed: {}", stderr.trim());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| LogEntry {
                line: l.to_string(),
            })
            .collect())
    }

    /// Search filter: case-insensitive substring match.
    pub fn filter_entries(entries: &[LogEntry], query: &str) -> Vec<LogEntry> {
        if query.is_empty() {
            return entries.to_vec();
        }
        let q = query.to_lowercase();
        entries
            .iter()
            .filter(|e| e.line.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }
}
