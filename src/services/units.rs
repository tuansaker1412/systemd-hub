//! Unit listing, detail, and lifecycle actions via the D-Bus layer.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::dbus::SystemdClient;
use crate::models::{ServiceAction, UnitDetail, UnitSummary};

/// Service-layer façade for systemd unit operations.
///
/// Holds a shared D-Bus client. Methods are async and must not be awaited
/// on the GTK main thread without offloading.
#[derive(Clone)]
pub struct UnitService {
    client: Arc<Mutex<Option<SystemdClient>>>,
}

impl UnitService {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
        }
    }

    async fn client(&self) -> Result<tokio::sync::MutexGuard<'_, Option<SystemdClient>>> {
        let mut guard = self.client.lock().await;
        if guard.is_none() {
            let c = SystemdClient::connect().await?;
            *guard = Some(c);
        }
        Ok(guard)
    }

    pub async fn list_services(&self) -> Result<Vec<UnitSummary>> {
        let guard = self.client().await?;
        let client = guard.as_ref().expect("client just connected");
        client.list_services().await
    }

    pub async fn get_detail(&self, name: &str) -> Result<UnitDetail> {
        let guard = self.client().await?;
        let client = guard.as_ref().expect("client just connected");
        client.get_unit_detail(name).await
    }

    pub async fn perform_action(&self, name: &str, action: ServiceAction) -> Result<()> {
        let guard = self.client().await?;
        let client = guard.as_ref().expect("client just connected");
        client.perform_action(name, action).await
    }
}

impl Default for UnitService {
    fn default() -> Self {
        Self::new()
    }
}
