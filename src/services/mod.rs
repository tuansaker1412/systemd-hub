//! Business logic. UI talks only to this layer — never directly to D-Bus.

mod journal;
mod system_info;
mod units;

pub use journal::{JournalService, LogEntry};
pub use system_info::SystemInfoService;
pub use units::UnitService;
