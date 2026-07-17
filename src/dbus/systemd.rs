//! zbus client for org.freedesktop.systemd1.

use anyhow::{Context, Result};
use zbus::proxy;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

use crate::models::{ServiceAction, UnitDetail, UnitSummary};

/// Tuple returned by Manager.ListUnits.
type UnitTuple = (
    String,           // name
    String,           // description
    String,           // load_state
    String,           // active_state
    String,           // sub_state
    String,           // following
    OwnedObjectPath,  // unit_path
    u32,              // job_id
    String,           // job_type
    OwnedObjectPath,  // job_path
);

/// systemd ExecStart property entry: (path, argv, ignore_failure, start_ts, exit_ts, pid, code, status)
type ExecStartEntry = (
    String,
    Vec<String>,
    bool,
    u64,
    u64,
    u32,
    i32,
    i32,
);

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
trait Manager {
    async fn list_units(&self) -> zbus::Result<Vec<UnitTuple>>;

    async fn get_unit(&self, name: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(name = "StartUnit")]
    async fn start_unit(&self, name: &str, mode: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(name = "StopUnit")]
    async fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(name = "RestartUnit")]
    async fn restart_unit(&self, name: &str, mode: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(name = "ReloadUnit")]
    async fn reload_unit(&self, name: &str, mode: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(name = "EnableUnitFiles")]
    async fn enable_unit_files(
        &self,
        files: &[&str],
        runtime: bool,
        force: bool,
    ) -> zbus::Result<(bool, Vec<(String, String, String)>)>;

    #[zbus(name = "DisableUnitFiles")]
    async fn disable_unit_files(
        &self,
        files: &[&str],
        runtime: bool,
    ) -> zbus::Result<Vec<(String, String, String)>>;

    #[zbus(name = "GetUnitFileState")]
    async fn get_unit_file_state(&self, file: &str) -> zbus::Result<String>;

    /// Returns (unit_file_path, enable_state) pairs.
    #[zbus(name = "ListUnitFiles")]
    async fn list_unit_files(&self) -> zbus::Result<Vec<(String, String)>>;

    async fn reload(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1"
)]
trait Unit {
    #[zbus(property)]
    fn id(&self) -> zbus::fdo::Result<String>;

    #[zbus(property)]
    fn description(&self) -> zbus::fdo::Result<String>;

    #[zbus(property, name = "ActiveState")]
    fn active_state(&self) -> zbus::fdo::Result<String>;

    #[zbus(property, name = "SubState")]
    fn sub_state(&self) -> zbus::fdo::Result<String>;

    #[zbus(property, name = "LoadState")]
    fn load_state(&self) -> zbus::fdo::Result<String>;

    #[zbus(property, name = "FragmentPath")]
    fn fragment_path(&self) -> zbus::fdo::Result<String>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Service",
    default_service = "org.freedesktop.systemd1"
)]
trait Service {
    #[zbus(property, name = "MainPID")]
    fn main_pid(&self) -> zbus::fdo::Result<u32>;

    #[zbus(property, name = "MemoryCurrent")]
    fn memory_current(&self) -> zbus::fdo::Result<u64>;

    #[zbus(property, name = "ExecStart")]
    fn exec_start(&self) -> zbus::fdo::Result<Vec<ExecStartEntry>>;
}

/// Thin async client around the systemd manager on the system bus.
pub struct SystemdClient {
    connection: Connection,
}

impl SystemdClient {
    pub async fn connect() -> Result<Self> {
        let connection = Connection::system()
            .await
            .context("failed to connect to the system D-Bus")?;
        Ok(Self { connection })
    }

    /// List all loaded `.service` units with enable state when available.
    pub async fn list_services(&self) -> Result<Vec<UnitSummary>> {
        let manager = ManagerProxy::new(&self.connection)
            .await
            .context("failed to create Manager proxy")?;

        let units = manager
            .list_units()
            .await
            .context("ListUnits failed")?;

        // One bulk call for enable states instead of N GetUnitFileState round-trips.
        let enable_map = match manager.list_unit_files().await {
            Ok(files) => files
                .into_iter()
                .filter_map(|(path, state)| {
                    let name = path.rsplit('/').next()?.to_string();
                    if name.ends_with(".service") {
                        Some((name, state))
                    } else {
                        None
                    }
                })
                .collect::<std::collections::HashMap<_, _>>(),
            Err(e) => {
                tracing::warn!(error = %e, "ListUnitFiles failed; enable states unknown");
                std::collections::HashMap::new()
            }
        };

        let mut services = Vec::new();
        for (name, description, load_state, active_state, sub_state, _following, unit_path, ..) in
            units
        {
            if !name.ends_with(".service") {
                continue;
            }

            let enabled_state = enable_map
                .get(&name)
                .cloned()
                .unwrap_or_else(|| "unknown".into());

            services.push(UnitSummary {
                name,
                description,
                load_state,
                active_state,
                sub_state,
                unit_path: unit_path.to_string(),
                enabled_state,
            });
        }

        services.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(services)
    }

    /// Fetch detailed properties for a single unit.
    pub async fn get_unit_detail(&self, name: &str) -> Result<UnitDetail> {
        let manager = ManagerProxy::new(&self.connection)
            .await
            .context("failed to create Manager proxy")?;

        let path = manager
            .get_unit(name)
            .await
            .with_context(|| format!("GetUnit({name}) failed"))?;

        let unit = UnitProxy::builder(&self.connection)
            .path(&path)?
            .build()
            .await
            .context("failed to create Unit proxy")?;

        let service = ServiceProxy::builder(&self.connection)
            .path(&path)?
            .build()
            .await
            .context("failed to create Service proxy")?;

        let description = unit.description().await.unwrap_or_default();
        let active_state = unit
            .active_state()
            .await
            .unwrap_or_else(|_| "unknown".into());
        let sub_state = unit.sub_state().await.unwrap_or_else(|_| "unknown".into());
        let load_state = unit.load_state().await.unwrap_or_else(|_| "unknown".into());
        let fragment_path = unit.fragment_path().await.unwrap_or_default();
        let main_pid = service.main_pid().await.unwrap_or(0);

        let memory_bytes = match service.memory_current().await {
            Ok(v) if v != u64::MAX => Some(v),
            _ => None,
        };

        let exec_start = match service.exec_start().await {
            Ok(entries) if !entries.is_empty() => {
                let args = &entries[0].1;
                if args.is_empty() {
                    entries[0].0.clone()
                } else {
                    args.join(" ")
                }
            }
            _ => String::new(),
        };

        let enabled_state = manager
            .get_unit_file_state(name)
            .await
            .unwrap_or_else(|_| "unknown".into());

        Ok(UnitDetail {
            name: name.to_string(),
            description,
            active_state,
            sub_state,
            load_state,
            main_pid,
            memory_bytes,
            exec_start,
            fragment_path,
            enabled_state,
            unit_path: path.to_string(),
        })
    }

    /// Perform a lifecycle or enable/disable action.
    pub async fn perform_action(&self, name: &str, action: ServiceAction) -> Result<()> {
        let manager = ManagerProxy::new(&self.connection)
            .await
            .context("failed to create Manager proxy")?;

        const MODE: &str = "replace";

        match action {
            ServiceAction::Start => {
                manager
                    .start_unit(name, MODE)
                    .await
                    .with_context(|| format!("StartUnit({name}) failed"))?;
            }
            ServiceAction::Stop => {
                manager
                    .stop_unit(name, MODE)
                    .await
                    .with_context(|| format!("StopUnit({name}) failed"))?;
            }
            ServiceAction::Restart => {
                manager
                    .restart_unit(name, MODE)
                    .await
                    .with_context(|| format!("RestartUnit({name}) failed"))?;
            }
            ServiceAction::Reload => {
                manager
                    .reload_unit(name, MODE)
                    .await
                    .with_context(|| format!("ReloadUnit({name}) failed"))?;
            }
            ServiceAction::Enable => {
                manager
                    .enable_unit_files(&[name], false, true)
                    .await
                    .with_context(|| format!("EnableUnitFiles({name}) failed"))?;
                manager
                    .reload()
                    .await
                    .context("daemon-reload after enable failed")?;
            }
            ServiceAction::Disable => {
                manager
                    .disable_unit_files(&[name], false)
                    .await
                    .with_context(|| format!("DisableUnitFiles({name}) failed"))?;
                manager
                    .reload()
                    .await
                    .context("daemon-reload after disable failed")?;
            }
        }

        Ok(())
    }
}
