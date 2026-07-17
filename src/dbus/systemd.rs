//! zbus client for org.freedesktop.systemd1.

use anyhow::{Context, Result};
use zbus::proxy;
use zbus::proxy::MethodFlags;
use zbus::zvariant::{DynamicType, OwnedObjectPath};
use zbus::Connection;

use crate::models::{ServiceAction, UnitDetail, UnitSummary};
use crate::utils::{classify_unit, UnitScope};

/// Tuple returned by Manager.ListUnits.
type UnitTuple = (
    String,          // name
    String,          // description
    String,          // load_state
    String,          // active_state
    String,          // sub_state
    String,          // following
    OwnedObjectPath, // unit_path
    u32,             // job_id
    String,          // job_type
    OwnedObjectPath, // job_path
);

/// systemd ExecStart property entry: (path, argv, ignore_failure, start_ts, exit_ts, pid, code, status)
type ExecStartEntry = (String, Vec<String>, bool, u64, u64, u32, i32, i32);

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
/// Read-only Manager methods.
///
/// Lifecycle / enable / disable / daemon-reload **must not** be declared here.
/// They go through [`SystemdClient::perform_action`] → [`call_interactive`] so
/// Polkit can show an authentication dialog (`AllowInteractiveAuth`).
trait Manager {
    async fn list_units(&self) -> zbus::Result<Vec<UnitTuple>>;

    async fn get_unit(&self, name: &str) -> zbus::Result<OwnedObjectPath>;

    /// Load a unit into memory (even if inactive/disabled) and return its object path.
    #[zbus(name = "LoadUnit")]
    async fn load_unit(&self, name: &str) -> zbus::Result<OwnedObjectPath>;

    #[zbus(name = "GetUnitFileState")]
    async fn get_unit_file_state(&self, file: &str) -> zbus::Result<String>;

    /// Returns (unit_file_path, enable_state) pairs.
    #[zbus(name = "ListUnitFiles")]
    async fn list_unit_files(&self) -> zbus::Result<Vec<(String, String)>>;
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

    /// List installed `.service` units (loaded + unit files).
    ///
    /// `ListUnits` only returns units currently in memory. A disabled inactive
    /// service is often unloaded and would disappear from the UI after disable.
    /// Merging with `ListUnitFiles` keeps those services visible (inactive/dead).
    pub async fn list_services(&self) -> Result<Vec<UnitSummary>> {
        let manager = ManagerProxy::new(&self.connection)
            .await
            .context("failed to create Manager proxy")?;

        let units = manager.list_units().await.context("ListUnits failed")?;

        // path → enable state for every installed unit file.
        let unit_files = match manager.list_unit_files().await {
            Ok(files) => files,
            Err(e) => {
                tracing::warn!(error = %e, "ListUnitFiles failed; showing loaded units only");
                Vec::new()
            }
        };

        let mut by_name: std::collections::HashMap<String, UnitSummary> =
            std::collections::HashMap::new();

        // 1) All installed .service unit files (includes disabled / not loaded).
        for (path, enabled_state) in unit_files {
            let Some(name) = path.rsplit('/').next().map(str::to_string) else {
                continue;
            };
            if !name.ends_with(".service") {
                continue;
            }
            // Template definitions (foo@.service), not concrete instances.
            if name.ends_with("@.service") {
                continue;
            }
            let category = classify_unit(&path, &name, UnitScope::System, enabled_state.as_str());
            by_name.insert(
                name.clone(),
                UnitSummary {
                    name,
                    description: String::new(),
                    load_state: "not-found".into(),
                    active_state: "inactive".into(),
                    sub_state: "dead".into(),
                    unit_path: String::new(),
                    enabled_state,
                    unit_file_path: path,
                    category,
                },
            );
        }

        // 2) Overlay live state from loaded units (running, failed, recently used, …).
        for (name, description, load_state, active_state, sub_state, _following, unit_path, ..) in
            units
        {
            if !name.ends_with(".service") {
                continue;
            }

            let existing = by_name.get(&name);
            let enabled_state = existing
                .map(|s| s.enabled_state.clone())
                .unwrap_or_else(|| "unknown".into());
            let unit_file_path = existing
                .map(|s| s.unit_file_path.clone())
                .unwrap_or_default();
            let category = classify_unit(
                &unit_file_path,
                &name,
                UnitScope::System,
                enabled_state.as_str(),
            );

            by_name.insert(
                name.clone(),
                UnitSummary {
                    name,
                    description,
                    load_state,
                    active_state,
                    sub_state,
                    unit_path: unit_path.to_string(),
                    enabled_state,
                    unit_file_path,
                    category,
                },
            );
        }

        // Prefer a clearer load_state label for unit-file-only rows; reclassify if needed.
        for summary in by_name.values_mut() {
            if summary.unit_path.is_empty() && summary.load_state == "not-found" {
                summary.load_state = "unloaded".into();
            }
            summary.category = classify_unit(
                &summary.unit_file_path,
                &summary.name,
                UnitScope::System,
                summary.enabled_state.as_str(),
            );
        }

        let mut services: Vec<UnitSummary> = by_name.into_values().collect();
        services.sort_by_key(|a| a.name.to_lowercase());
        Ok(services)
    }

    /// Fetch detailed properties for a single unit.
    ///
    /// Uses `GetUnit` when loaded; otherwise `LoadUnit` so disabled/unloaded
    /// services still open in the detail panel.
    pub async fn get_unit_detail(&self, name: &str) -> Result<UnitDetail> {
        let manager = ManagerProxy::new(&self.connection)
            .await
            .context("failed to create Manager proxy")?;

        let path = match manager.get_unit(name).await {
            Ok(path) => path,
            Err(_) => manager
                .load_unit(name)
                .await
                .with_context(|| format!("LoadUnit({name}) failed (unit not loaded)"))?,
        };

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
    ///
    /// # Polkit / interactive auth
    ///
    /// Every privileged systemd method below is invoked via [`call_interactive`]
    /// (`AllowInteractiveAuth`) so the session Polkit agent can show a password
    /// dialog when needed:
    ///
    /// | UI control              | `ServiceAction` | D-Bus method(s)                          |
    /// |-------------------------|-----------------|------------------------------------------|
    /// | Start button            | `Start`         | `StartUnit`                              |
    /// | Stop button             | `Stop`          | `StopUnit`                               |
    /// | Restart button          | `Restart`       | `RestartUnit`                            |
    /// | Reload button           | `Reload`        | `ReloadUnit`                             |
    /// | Enabled switch on       | `Enable`        | `EnableUnitFiles` + `Reload`             |
    /// | Enabled switch off      | `Disable`       | `DisableUnitFiles` + `Reload`            |
    ///
    /// Read-only UI (list, detail, logs/journal) does not need this flag.
    pub async fn perform_action(&self, name: &str, action: ServiceAction) -> Result<()> {
        let manager = ManagerProxy::new(&self.connection)
            .await
            .context("failed to create Manager proxy")?;

        const MODE: &str = "replace";

        match action {
            ServiceAction::Start => {
                call_interactive::<OwnedObjectPath>(&manager, "StartUnit", &(name, MODE))
                    .await
                    .with_context(|| format!("StartUnit({name}) failed"))?;
            }
            ServiceAction::Stop => {
                call_interactive::<OwnedObjectPath>(&manager, "StopUnit", &(name, MODE))
                    .await
                    .with_context(|| format!("StopUnit({name}) failed"))?;
            }
            ServiceAction::Restart => {
                call_interactive::<OwnedObjectPath>(&manager, "RestartUnit", &(name, MODE))
                    .await
                    .with_context(|| format!("RestartUnit({name}) failed"))?;
            }
            ServiceAction::Reload => {
                call_interactive::<OwnedObjectPath>(&manager, "ReloadUnit", &(name, MODE))
                    .await
                    .with_context(|| format!("ReloadUnit({name}) failed"))?;
            }
            ServiceAction::Enable => {
                // Permanent enable (survives reboot). force=true replaces conflicting symlinks.
                let files: &[&str] = &[name];
                call_interactive::<(bool, Vec<(String, String, String)>)>(
                    &manager,
                    "EnableUnitFiles",
                    &(files, false, true),
                )
                .await
                .with_context(|| format!("EnableUnitFiles({name}) failed"))?;
                // daemon-reload also requires auth on many systems.
                call_interactive::<()>(&manager, "Reload", &())
                    .await
                    .context("daemon-reload after enable failed")?;
            }
            ServiceAction::Disable => {
                // Runtime-enabled units live under /run and must be disabled with runtime=true.
                let state = manager.get_unit_file_state(name).await.unwrap_or_default();
                let runtime = matches!(
                    state.as_str(),
                    "enabled-runtime" | "linked-runtime" | "masked-runtime"
                );
                let files: &[&str] = &[name];
                call_interactive::<Vec<(String, String, String)>>(
                    &manager,
                    "DisableUnitFiles",
                    &(files, runtime),
                )
                .await
                .with_context(|| format!("DisableUnitFiles({name}, runtime={runtime}) failed"))?;
                call_interactive::<()>(&manager, "Reload", &())
                    .await
                    .context("daemon-reload after disable failed")?;
            }
        }

        Ok(())
    }
}

/// Call a Manager method with interactive Polkit authorization allowed.
///
/// Without `AllowInteractiveAuth`, privileged calls fail immediately with
/// `org.freedesktop.DBus.Error.InteractiveAuthorizationRequired` and no
/// password dialog is shown.
async fn call_interactive<R>(
    manager: &ManagerProxy<'_>,
    method: &str,
    body: &(impl serde::Serialize + DynamicType),
) -> Result<R>
where
    R: for<'d> zbus::zvariant::DynamicDeserialize<'d>,
{
    manager
        .inner()
        .call_with_flags::<_, _, R>(method, MethodFlags::AllowInteractiveAuth.into(), body)
        .await
        .with_context(|| format!("D-Bus {method} call failed"))?
        .with_context(|| format!("D-Bus {method} returned no reply"))
}
