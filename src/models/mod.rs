//! Domain models shared across service and UI layers.

mod settings;
mod system_info;
mod unit;

pub use settings::AppTheme;
pub use system_info::SystemInfo;
pub use unit::{ServiceAction, StateTone, UnitCategory, UnitDetail, UnitSummary};
