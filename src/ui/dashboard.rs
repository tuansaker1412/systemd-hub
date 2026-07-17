//! Dashboard page: hostname, OS, kernel, uptime.

use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{self as gtk, Align, Image, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::models::SystemInfo;

/// Embedded app logo shown above the system overview section.
const LOGO_JPEG: &[u8] = include_bytes!("../../data/icons/systemd-hub-logo.jpg");

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
            &Label::builder()
                .label("Dashboard")
                .css_classes(["title"])
                .build(),
        ));

        let content = gtk::Box::new(Orientation::Vertical, 18);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_halign(Align::Fill);
        content.set_valign(Align::Start);

        content.append(&Self::logo_header());

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

    /// Logo + app name, centered above "System Overview".
    fn logo_header() -> gtk::Box {
        let column = gtk::Box::new(Orientation::Vertical, 12);
        column.set_halign(Align::Center);
        column.set_valign(Align::Center);
        column.set_hexpand(true);

        let logo = Self::logo_image();
        column.append(&logo);

        let app_name = Label::builder()
            .label("Systemd Hub")
            .css_classes(["title-2"])
            .halign(Align::Center)
            .justify(gtk::Justification::Center)
            .build();
        let tagline = Label::builder()
            .label("Native systemd service manager")
            .css_classes(["dimmed"])
            .halign(Align::Center)
            .justify(gtk::Justification::Center)
            .build();

        column.append(&app_name);
        column.append(&tagline);

        column
    }

    fn logo_image() -> Image {
        let image = Image::builder()
            .pixel_size(96)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        match gdk::Texture::from_bytes(&glib::Bytes::from_static(LOGO_JPEG)) {
            Ok(texture) => {
                image.set_paintable(Some(&texture));
            }
            Err(err) => {
                tracing::warn!("failed to load dashboard logo: {err}");
                // Fallback to the existing symbolic-friendly app icon asset name if available.
                image.set_icon_name(Some("application-x-executable"));
            }
        }

        image
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
