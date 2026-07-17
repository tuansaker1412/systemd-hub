//! Domain models shared across service and UI layers.

mod system_info;
mod unit;

pub use system_info::SystemInfo;
pub use unit::{ServiceAction, StateTone, UnitDetail, UnitSummary};
