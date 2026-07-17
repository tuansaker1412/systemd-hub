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
/// - Selecting a unit reveals an end-side inspector (Details first).
/// - Logs are a separate inspector tab and are not shown until chosen.
/// - Inspector can be collapsed so the list returns to full width.
pub struct ServicesView {
    pub widget: adw::OverlaySplitView,
    list: ServiceListPage,
    inspector: ServiceInspector,
    /// True when a unit is selected (inspector may still be collapsed).
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
        split.set_enable_show_gesture(true);
        split.set_min_sidebar_width(340.0);
        split.set_max_sidebar_width(640.0);
        split.set_sidebar_width_fraction(0.36);
        split.set_hexpand(true);
        split.set_vexpand(true);

        // Collapse control on the inspector header.
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

    pub fn set_units(&self, units: Vec<UnitSummary>) {
        self.list.set_units(units);
    }

    pub fn set_status(&self, text: &str) {
        self.list.set_status(text);
    }

    pub fn selected_unit(&self) -> Option<UnitSummary> {
        self.list.selected_unit()
    }

    /// Reveal inspector on the Details tab (used when a unit is selected).
    pub fn open_details(&self) {
        self.has_selection.set(true);
        self.inspector.show_details();
        self.widget.set_show_sidebar(true);
    }

    /// Hide inspector; list becomes full width. Selection may still be kept by the window.
    pub fn collapse_inspector(&self) {
        self.widget.set_show_sidebar(false);
    }

    /// Hide inspector and clear selection-related UI state in child panels.
    pub fn clear_selection_ui(&self) {
        self.has_selection.set(false);
        self.inspector.clear();
        self.widget.set_show_sidebar(false);
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

    pub fn connect_follow_toggled<F: Fn(bool) + 'static>(&self, f: F) {
        self.inspector.logs().connect_follow_toggled(f);
    }

    pub fn connect_inspector_page_changed<F: Fn(InspectorPage) + 'static>(&self, f: F) {
        self.inspector.connect_page_changed(f);
    }
}

impl Default for ServicesView {
    fn default() -> Self {
        Self::new()
    }
}
