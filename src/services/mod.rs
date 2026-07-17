//! Business logic. UI talks only to this layer — never directly to D-Bus.

mod journal;
mod settings;
mod system_info;
mod units;

pub use journal::{JournalService, LogEntry};
pub use settings::SettingsService;
pub use system_info::SystemInfoService;
pub use units::UnitService;
