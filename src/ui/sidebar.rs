//! Application sidebar navigation.

use gtk4::prelude::*;
use gtk4::{self as gtk, Align, Label, ListBox, ListBoxRow, Orientation, SelectionMode};
use libadwaita as adw;

/// Pages reachable from the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPage {
    Dashboard,
    Services,
    Settings,
    About,
}

pub struct Sidebar {
    pub widget: adw::ToolbarView,
    list: ListBox,
}

impl Sidebar {
    pub fn new() -> Self {
        let list = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .css_classes(["navigation-sidebar"])
            .build();

        list.append(&Self::row("Dashboard", "view-grid-symbolic", "dashboard"));
        list.append(&Self::row(
            "Services",
            "application-x-executable-symbolic",
            "services",
        ));
        list.append(&Self::row(
            "Settings",
            "preferences-system-symbolic",
            "settings",
        ));
        list.append(&Self::row("About", "help-about-symbolic", "about"));

        // Select dashboard by default.
        if let Some(row) = list.row_at_index(0) {
            list.select_row(Some(&row));
        }

        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(
            &Label::builder()
                .label("Systemd Hub")
                .css_classes(["title"])
                .build(),
        ));
        header.set_show_end_title_buttons(false);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&list));

        Self {
            widget: toolbar,
            list,
        }
    }

    fn row(title: &str, icon: &str, id: &str) -> ListBoxRow {
        let box_ = gtk::Box::new(Orientation::Horizontal, 12);
        box_.set_margin_start(12);
        box_.set_margin_end(12);
        box_.set_margin_top(8);
        box_.set_margin_bottom(8);

        let image = gtk::Image::from_icon_name(icon);
        image.set_valign(Align::Center);

        let label = Label::builder()
            .label(title)
            .xalign(0.0)
            .hexpand(true)
            .build();

        box_.append(&image);
        box_.append(&label);

        let row = ListBoxRow::new();
        row.set_child(Some(&box_));
        row.set_widget_name(id);
        row
    }

    pub fn connect_page_selected<F: Fn(SidebarPage) + 'static>(&self, f: F) {
        self.list.connect_row_selected(move |_, row| {
            let Some(row) = row else { return };
            let page = match row.widget_name().as_str() {
                "services" => SidebarPage::Services,
                "settings" => SidebarPage::Settings,
                "about" => SidebarPage::About,
                _ => SidebarPage::Dashboard,
            };
            f(page);
        });
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}
