//! Service list with search, sort, and ColumnView.

use gtk4::prelude::*;
use gtk4::{
    self as gtk, gio, ColumnView, ColumnViewColumn, CustomFilter, CustomSorter, FilterChange,
    FilterListModel, Label, MultiSorter, Orientation, ScrolledWindow, SearchEntry,
    SignalListItemFactory, SingleSelection, SortListModel,
};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::models::UnitSummary;
use crate::ui::UnitObject;

pub struct ServiceListPage {
    pub widget: adw::ToolbarView,
    store: gio::ListStore,
    selection: SingleSelection,
    filter: CustomFilter,
    status_label: Label,
}

impl ServiceListPage {
    pub fn new() -> Self {
        let store = gio::ListStore::new::<UnitObject>();

        let search_text = Rc::new(RefCell::new(String::new()));
        let search_text_filter = search_text.clone();

        let filter = CustomFilter::new(move |obj| {
            let Some(unit) = obj.downcast_ref::<UnitObject>() else {
                return false;
            };
            let q = search_text_filter.borrow().to_lowercase();
            if q.is_empty() {
                return true;
            }
            unit.name().to_lowercase().contains(&q)
                || unit.description().to_lowercase().contains(&q)
                || unit.active_state().to_lowercase().contains(&q)
        });

        let filter_model = FilterListModel::new(Some(store.clone()), Some(filter.clone()));

        let name_sorter = CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<UnitObject>().map(|u| u.name()).unwrap_or_default();
            let b = b.downcast_ref::<UnitObject>().map(|u| u.name()).unwrap_or_default();
            a.to_lowercase().cmp(&b.to_lowercase()).into()
        });

        let multi = MultiSorter::new();
        multi.append(name_sorter);

        let sort_model = SortListModel::new(Some(filter_model), Some(multi));
        let selection = SingleSelection::new(Some(sort_model));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);

        let column_view = ColumnView::builder()
            .model(&selection)
            .reorderable(false)
            .show_row_separators(true)
            .show_column_separators(false)
            .hexpand(true)
            .vexpand(true)
            .build();

        column_view.append_column(&Self::text_column(
            "Service",
            220,
            true,
            |u| u.name(),
            |a, b| {
                a.name()
                    .to_lowercase()
                    .cmp(&b.name().to_lowercase())
                    .into()
            },
        ));
        column_view.append_column(&Self::text_column(
            "Description",
            280,
            true,
            |u| u.description(),
            |a, b| {
                a.description()
                    .to_lowercase()
                    .cmp(&b.description().to_lowercase())
                    .into()
            },
        ));
        column_view.append_column(&Self::text_column(
            "Status",
            140,
            true,
            |u| u.status_label(),
            |a, b| a.active_state().cmp(&b.active_state()).into(),
        ));
        column_view.append_column(&Self::text_column(
            "Enabled",
            100,
            true,
            |u| u.enabled_state(),
            |a, b| a.enabled_state().cmp(&b.enabled_state()).into(),
        ));
        column_view.append_column(&Self::text_column(
            "Running",
            90,
            false,
            |u| {
                if u.active_state() == "active" {
                    "yes".into()
                } else {
                    "no".into()
                }
            },
            |a, b| a.active_state().cmp(&b.active_state()).into(),
        ));

        let scrolled = ScrolledWindow::builder()
            .child(&column_view)
            .hexpand(true)
            .vexpand(true)
            .build();

        let search_entry = SearchEntry::builder()
            .placeholder_text("Search services…")
            .hexpand(true)
            .build();

        let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Refresh"));
        refresh_btn.set_action_name(Some("win.refresh-services"));

        let status_label = Label::builder()
            .label("No services loaded")
            .css_classes(["dimmed"])
            .xalign(0.0)
            .hexpand(true)
            .build();

        let search_bar = gtk::Box::new(Orientation::Horizontal, 8);
        search_bar.set_margin_start(8);
        search_bar.set_margin_end(8);
        search_bar.set_margin_top(6);
        search_bar.set_margin_bottom(6);
        search_bar.append(&search_entry);
        search_bar.append(&refresh_btn);

        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(
            &Label::builder().label("Services").css_classes(["title"]).build(),
        ));

        let bottom = gtk::Box::new(Orientation::Horizontal, 8);
        bottom.set_margin_start(12);
        bottom.set_margin_end(12);
        bottom.set_margin_top(4);
        bottom.set_margin_bottom(4);
        bottom.append(&status_label);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.add_top_bar(&search_bar);
        toolbar.set_content(Some(&scrolled));
        toolbar.add_bottom_bar(&bottom);

        let filter_for_search = filter.clone();
        let search_text_for_entry = search_text.clone();
        search_entry.connect_search_changed(move |entry| {
            *search_text_for_entry.borrow_mut() = entry.text().to_string();
            filter_for_search.changed(FilterChange::Different);
        });

        // Keep search widgets alive via the widget tree; only retain what methods need.
        let _ = search_entry;
        let _ = search_text;

        Self {
            widget: toolbar,
            store,
            selection,
            filter,
            status_label,
        }
    }

    fn text_column(
        title: &str,
        width: i32,
        expand: bool,
        get: impl Fn(&UnitObject) -> String + 'static,
        sort: impl Fn(&UnitObject, &UnitObject) -> gtk::Ordering + 'static,
    ) -> ColumnViewColumn {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, item| {
            let label = Label::builder()
                .xalign(0.0)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            item.downcast_ref::<gtk::ListItem>()
                .expect("ListItem")
                .set_child(Some(&label));
        });

        let get = Rc::new(get);
        let get_bind = get.clone();
        factory.connect_bind(move |_, item| {
            let list_item = item.downcast_ref::<gtk::ListItem>().expect("ListItem");
            let Some(obj) = list_item.item() else { return };
            let Some(unit) = obj.downcast_ref::<UnitObject>() else { return };
            let Some(child) = list_item.child() else { return };
            if let Ok(label) = child.downcast::<Label>() {
                label.set_label(&get_bind(unit));
            }
        });

        let sorter = CustomSorter::new(move |a, b| {
            let a = a.downcast_ref::<UnitObject>();
            let b = b.downcast_ref::<UnitObject>();
            match (a, b) {
                (Some(a), Some(b)) => sort(a, b),
                _ => gtk::Ordering::Equal,
            }
        });

        ColumnViewColumn::builder()
            .title(title)
            .factory(&factory)
            .expand(expand)
            .fixed_width(width)
            .sorter(&sorter)
            .resizable(true)
            .build()
    }

    pub fn set_units(&self, units: Vec<UnitSummary>) {
        self.store.remove_all();
        let count = units.len();
        for unit in units {
            self.store.append(&UnitObject::new(unit));
        }
        self.status_label
            .set_label(&format!("{count} service{}", if count == 1 { "" } else { "s" }));
        self.filter.changed(FilterChange::Different);
    }

    pub fn set_status(&self, text: &str) {
        self.status_label.set_label(text);
    }

    pub fn connect_selection_changed<F: Fn(Option<UnitSummary>) + 'static>(&self, f: F) {
        let f = Rc::new(f);
        let f_sel = f.clone();
        self.selection.connect_selection_changed(move |sel, _, _| {
            let unit = sel
                .selected_item()
                .and_then(|o| o.downcast::<UnitObject>().ok())
                .map(|u| u.summary());
            f_sel(unit);
        });
        self.selection
            .connect_notify_local(Some("selected-item"), move |sel, _| {
                let unit = sel
                    .selected_item()
                    .and_then(|o| o.downcast::<UnitObject>().ok())
                    .map(|u| u.summary());
                f(unit);
            });
    }

    pub fn selected_unit(&self) -> Option<UnitSummary> {
        self.selection
            .selected_item()
            .and_then(|o| o.downcast::<UnitObject>().ok())
            .map(|u| u.summary())
    }
}

impl Default for ServiceListPage {
    fn default() -> Self {
        Self::new()
    }
}
