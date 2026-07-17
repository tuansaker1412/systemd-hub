//! Services page composition: list + collapsible inspector.

mod inspector;
mod view;

pub use inspector::InspectorPage;
pub use view::ServicesView;

// Available for direct use by callers that need the inspector type.
#[allow(unused_imports)]
pub use inspector::ServiceInspector;
