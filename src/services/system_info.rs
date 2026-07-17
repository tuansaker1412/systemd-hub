//! Collect host system information for the dashboard.

use anyhow::{Context, Result};
use std::fs;
use std::io::Read;

use crate::models::SystemInfo;

pub struct SystemInfoService;

impl SystemInfoService {
    /// Gather hostname, OS pretty name, kernel, and uptime.
    /// Prefer filesystem / uname over shelling out.
    pub fn collect() -> Result<SystemInfo> {
        Ok(SystemInfo {
            hostname: read_hostname(),
            operating_system: read_os_pretty_name(),
            kernel_version: read_kernel_version(),
            uptime_seconds: read_uptime_seconds()?,
        })
    }
}

fn read_hostname() -> String {
    if let Ok(s) = fs::read_to_string("/etc/hostname") {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    hostname_from_uname().unwrap_or_else(|| "unknown".into())
}

fn hostname_from_uname() -> Option<String> {
    // Fallback via /proc/sys/kernel/hostname
    fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .map(|s| s.trim().to_string())
}

fn read_os_pretty_name() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                return value.trim_matches('"').to_string();
            }
        }
    }
    "Linux".into()
}

fn read_kernel_version() -> String {
    fs::read_to_string("/proc/version")
        .ok()
        .and_then(|v| {
            // "Linux version X.Y.Z-..."
            let mut parts = v.split_whitespace();
            parts.next()?; // Linux
            parts.next()?; // version
            parts.next().map(|s| s.to_string())
        })
        .or_else(|| {
            fs::read_to_string("/proc/sys/kernel/osrelease")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn read_uptime_seconds() -> Result<u64> {
    let mut file = fs::File::open("/proc/uptime").context("open /proc/uptime")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .context("read /proc/uptime")?;
    let first = buf
        .split_whitespace()
        .next()
        .context("empty /proc/uptime")?;
    let seconds: f64 = first.parse().context("parse uptime")?;
    Ok(seconds as u64)
}
