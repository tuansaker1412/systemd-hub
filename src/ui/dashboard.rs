//! Dashboard page: hostname, OS, kernel, uptime.

use gtk4::prelude::*;
use gtk4::{self as gtk, Align, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::models::SystemInfo;

pub struct DashboardPage {
    pub widget: adw::ToolbarView,
    hostname: Label,
    os: Label,
    kernel: Label,
    uptime: Label,
    status: Label,
}

impl DashboardPage {
    pub fn new() -> Self {
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(
            &Label::builder().label("Dashboard").css_classes(["title"]).build(),
        ));

        let content = gtk::Box::new(Orientation::Vertical, 18);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_halign(Align::Fill);
        content.set_valign(Align::Start);

        let title = Label::builder()
            .label("System Overview")
            .css_classes(["title-1"])
            .halign(Align::Start)
            .build();
        content.append(&title);

        let status = Label::builder()
            .label("Loading system information…")
            .css_classes(["dimmed"])
            .halign(Align::Start)
            .build();
        content.append(&status);

        let group = adw::PreferencesGroup::new();
        group.set_title("Host");

        let hostname = Self::value_label("—");
        let os = Self::value_label("—");
        let kernel = Self::value_label("—");
        let uptime = Self::value_label("—");

        group.add(&Self::row("Hostname", &hostname));
        group.add(&Self::row("Operating System", &os));
        group.add(&Self::row("Kernel", &kernel));
        group.add(&Self::row("Uptime", &uptime));

        content.append(&group);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .child(&content)
            .hexpand(true)
            .vexpand(true)
            .build();

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&scrolled));

        Self {
            widget: toolbar,
            hostname,
            os,
            kernel,
            uptime,
            status,
        }
    }

    fn value_label(text: &str) -> Label {
        Label::builder()
            .label(text)
            .xalign(1.0)
            .hexpand(true)
            .selectable(true)
            .wrap(true)
            .build()
    }

    fn row(title: &str, value: &Label) -> adw::ActionRow {
        let row = adw::ActionRow::builder().title(title).build();
        row.add_suffix(value);
        row.set_activatable(false);
        row
    }

    pub fn set_loading(&self) {
        self.status.set_label("Loading system information…");
    }

    pub fn set_info(&self, info: &SystemInfo) {
        self.hostname.set_label(&info.hostname);
        self.os.set_label(&info.operating_system);
        self.kernel.set_label(&info.kernel_version);
        self.uptime.set_label(&info.uptime_display());
        self.status.set_label("Ready");
    }

    pub fn set_error(&self, message: &str) {
        self.status.set_label(&format!("Error: {message}"));
    }
}

impl Default for DashboardPage {
    fn default() -> Self {
        Self::new()
    }
}
