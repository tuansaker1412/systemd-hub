//! Services area: full-width list + optional collapsible inspector.

use gtk4::prelude::*;
use gtk4::{self as gtk};
use libadwaita as adw;
use std::cell::Cell;
use std::rc::Rc;

use crate::models::UnitSummary;
use crate::ui::services::inspector::{InspectorPage, ServiceInspector};
use crate::ui::{LogViewer, ServiceDetailPage, ServiceListPage};

/// Composed Services page used by the main window.
///
/// Layout:
/// - Service list fills the content area by default.
/// - **Click** (activate) on a unit reveals the end-side inspector (Details first).
/// - The open unit stays highlighted until the inspector is closed.
/// - Hover alone does not open the inspector.
/// - Inspector can be collapsed so the list returns to full width.
pub struct ServicesView {
    pub widget: adw::OverlaySplitView,
    list: ServiceListPage,
    inspector: ServiceInspector,
    /// True after the user has opened a unit in the inspector (may still be collapsed).
    has_selection: Rc<Cell<bool>>,
}

impl ServicesView {
    pub fn new() -> Self {
        let list = ServiceListPage::new();
        let inspector = ServiceInspector::new();
        let has_selection = Rc::new(Cell::new(false));

        let split = adw::OverlaySplitView::new();
        split.set_sidebar_position(gtk::PackType::End);
        split.set_content(Some(&list.widget));
        split.set_sidebar(Some(&inspector.widget));
        split.set_show_sidebar(false);
        split.set_collapsed(false);
        split.set_enable_hide_gesture(true);
        // No edge-swipe peek of an empty inspector — open only on row click.
        split.set_enable_show_gesture(false);
        split.set_min_sidebar_width(340.0);
        split.set_max_sidebar_width(640.0);
        split.set_sidebar_width_fraction(0.36);
        split.set_hexpand(true);
        split.set_vexpand(true);

        // Collapse control on the inspector header (window may attach more handlers).
        {
            let split_for_collapse = split.clone();
            inspector.connect_collapse_clicked(move || {
                split_for_collapse.set_show_sidebar(false);
            });
        }

        Self {
            widget: split,
            list,
            inspector,
            has_selection,
        }
    }

    pub fn list(&self) -> &ServiceListPage {
        &self.list
    }

    pub fn detail(&self) -> &ServiceDetailPage {
        self.inspector.detail()
    }

    pub fn logs(&self) -> &LogViewer {
        self.inspector.logs()
    }

    pub fn inspector(&self) -> &ServiceInspector {
        &self.inspector
    }

    pub fn set_units(&self, units: Vec<UnitSummary>, select_name: Option<&str>) {
        self.list.set_units(units, select_name);
    }

    pub fn set_status(&self, text: &str) {
        self.list.set_status(text);
    }

    pub fn selected_unit(&self) -> Option<UnitSummary> {
        self.list.selected_unit()
    }

    /// Highlight `name` in the list (open detail / restore after refresh).
    pub fn select_unit(&self, name: &str) -> bool {
        self.list.select_by_name(name)
    }

    /// Reveal inspector on the Details tab and keep `name` highlighted.
    pub fn open_details(&self, name: &str) {
        self.has_selection.set(true);
        self.inspector.show_details();
        self.widget.set_show_sidebar(true);
        let _ = self.list.select_by_name(name);
    }

    /// Hide inspector; list becomes full width.
    pub fn collapse_inspector(&self) {
        self.widget.set_show_sidebar(false);
    }

    /// Hide inspector, clear detail/logs, and remove list highlight.
    pub fn clear_selection_ui(&self) {
        self.has_selection.set(false);
        self.inspector.clear();
        self.widget.set_show_sidebar(false);
        self.list.unselect();
    }

    pub fn is_inspector_visible(&self) -> bool {
        self.widget.shows_sidebar()
    }

    pub fn is_logs_visible(&self) -> bool {
        self.is_inspector_visible() && self.inspector.is_logs_visible()
    }

    pub fn connect_selection_changed<F: Fn(Option<UnitSummary>) + 'static>(&self, f: F) {
        self.list.connect_selection_changed(f);
    }

    /// Explicit row activate (click / Enter) — opens the detail inspector.
    pub fn connect_unit_activated<F: Fn(UnitSummary) + 'static>(&self, f: F) {
        self.list.connect_unit_activated(f);
    }

    pub fn connect_follow_toggled<F: Fn(bool) + 'static>(&self, f: F) {
        self.inspector.logs().connect_follow_toggled(f);
    }

    pub fn connect_inspector_page_changed<F: Fn(InspectorPage) + 'static>(&self, f: F) {
        self.inspector.connect_page_changed(f);
    }

    pub fn connect_collapse_clicked<F: Fn() + 'static>(&self, f: F) {
        self.inspector.connect_collapse_clicked(f);
    }
}

impl Default for ServicesView {
    fn default() -> Self {
        Self::new()
    }
}
