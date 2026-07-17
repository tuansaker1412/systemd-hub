//! GTK widgets and pages. No direct D-Bus access.

mod dashboard;
mod log_viewer;
mod service_detail;
mod service_list;
mod services;
mod sidebar;
mod unit_object;

pub use dashboard::DashboardPage;
pub use log_viewer::LogViewer;
pub use service_detail::ServiceDetailPage;
pub use service_list::ServiceListPage;
pub use services::{InspectorPage, ServicesView};
pub use sidebar::{Sidebar, SidebarPage};
pub use unit_object::UnitObject;
