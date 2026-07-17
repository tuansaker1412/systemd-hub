//! About page: app logo, version, and basic project information.

use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{self as gtk, Align, Image, Label, Orientation, ScrolledWindow};
use libadwaita as adw;
use libadwaita::prelude::*;

/// Embedded app logo (same asset as Dashboard branding).
const LOGO_JPEG: &[u8] = include_bytes!("../../data/icons/systemd-hub-logo.jpg");

const APP_NAME: &str = "Systemd Hub";
const APP_ID: &str = "dev.systemdhub.SystemdHub";
const APP_TAGLINE: &str = "Native Linux systemd service manager";
const APP_DESCRIPTION: &str =
    "A lightweight desktop app for managing systemd services with a native GTK 4 \
     and libadwaita interface. Communicates with systemd over D-Bus instead of \
     shelling out to systemctl for lifecycle actions.";
const APP_LICENSE: &str = "GPL-3.0-or-later";
const APP_DEVELOPER: &str = "Ngọc Tuấn";
const DEVELOPER_URL: &str = "https://github.com/tuansaker1412";
const REPO_URL: &str = "https://github.com/tuansaker1412/systemd-hub";
const ISSUES_URL: &str = "https://github.com/tuansaker1412/systemd-hub/issues";

pub struct AboutPage {
    pub widget: adw::ToolbarView,
}

impl AboutPage {
    pub fn new() -> Self {
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(
            &Label::builder()
                .label("About")
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

        content.append(&Self::brand_header());

        let about_title = Label::builder()
            .label("About this application")
            .css_classes(["title-1"])
            .halign(Align::Start)
            .build();
        content.append(&about_title);

        let description = Label::builder()
            .label(APP_DESCRIPTION)
            .wrap(true)
            .xalign(0.0)
            .css_classes(["dimmed"])
            .halign(Align::Start)
            .build();
        content.append(&description);

        let details = adw::PreferencesGroup::new();
        details.set_title("Details");
        details.add(&Self::info_row("Application", APP_NAME));
        details.add(&Self::info_row("Version", env!("CARGO_PKG_VERSION")));
        details.add(&Self::info_row("Application ID", APP_ID));
        details.add(&Self::info_row("License", APP_LICENSE));
        details.add(&Self::info_row(
            "Toolkit",
            "GTK 4 · libadwaita · Rust · zbus",
        ));
        content.append(&details);

        let people = adw::PreferencesGroup::new();
        people.set_title("Developer");
        people.set_description(Some("Author and maintainer of Systemd Hub."));
        people.add(&Self::link_row(
            "Developer",
            APP_DEVELOPER,
            "Open developer profile on GitHub",
            DEVELOPER_URL,
        ));
        people.add(&Self::link_row(
            "GitHub profile",
            DEVELOPER_URL,
            "Visit github.com/tuansaker1412",
            DEVELOPER_URL,
        ));
        content.append(&people);

        let project = adw::PreferencesGroup::new();
        project.set_title("Project");
        project.set_description(Some("Source code, releases, and issue tracker on GitHub."));
        project.add(&Self::link_row(
            "Repository",
            "tuansaker1412/systemd-hub",
            "Browse source code and documentation",
            REPO_URL,
        ));
        project.add(&Self::link_row(
            "Report an issue",
            "GitHub Issues",
            "Bug reports, feature requests, and feedback",
            ISSUES_URL,
        ));
        content.append(&project);

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

        Self { widget: toolbar }
    }

    fn brand_header() -> gtk::Box {
        let column = gtk::Box::new(Orientation::Vertical, 12);
        column.set_halign(Align::Center);
        column.set_valign(Align::Center);
        column.set_hexpand(true);
        column.set_margin_bottom(6);

        column.append(&Self::logo_image());

        let app_name = Label::builder()
            .label(APP_NAME)
            .css_classes(["title-1"])
            .halign(Align::Center)
            .justify(gtk::Justification::Center)
            .build();
        column.append(&app_name);

        let version = Label::builder()
            .label(format!("Version {}", env!("CARGO_PKG_VERSION")))
            .css_classes(["title-3"])
            .halign(Align::Center)
            .justify(gtk::Justification::Center)
            .build();
        column.append(&version);

        let tagline = Label::builder()
            .label(APP_TAGLINE)
            .css_classes(["dimmed"])
            .halign(Align::Center)
            .justify(gtk::Justification::Center)
            .build();
        column.append(&tagline);

        let developer = Label::builder()
            .label(format!("by {APP_DEVELOPER}"))
            .css_classes(["dimmed"])
            .halign(Align::Center)
            .justify(gtk::Justification::Center)
            .build();
        column.append(&developer);

        column
    }

    fn logo_image() -> Image {
        let image = Image::builder()
            .pixel_size(128)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        match gdk::Texture::from_bytes(&glib::Bytes::from_static(LOGO_JPEG)) {
            Ok(texture) => {
                image.set_paintable(Some(&texture));
            }
            Err(err) => {
                tracing::warn!("failed to load about logo: {err}");
                image.set_icon_name(Some("application-x-executable"));
            }
        }

        image
    }

    fn info_row(title: &str, value: &str) -> adw::ActionRow {
        let value_label = Label::builder()
            .label(value)
            .xalign(1.0)
            .hexpand(true)
            .selectable(true)
            .wrap(true)
            .build();

        let row = adw::ActionRow::builder().title(title).build();
        row.add_suffix(&value_label);
        row.set_activatable(false);
        row
    }

    /// Clickable row that opens an external URL in the default browser.
    fn link_row(title: &str, value: &str, subtitle: &str, url: &'static str) -> adw::ActionRow {
        let value_label = Label::builder()
            .label(value)
            .xalign(1.0)
            .hexpand(true)
            .selectable(false)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .css_classes(["dimmed"])
            .build();

        let external = Image::from_icon_name("adw-external-link-symbolic");
        external.add_css_class("dimmed");

        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .activatable(true)
            .build();
        // Hand/pointer cursor so links feel clickable on hover.
        row.set_cursor_from_name(Some("pointer"));
        row.add_suffix(&value_label);
        row.add_suffix(&external);
        row.connect_activated(move |_| {
            if let Err(err) =
                gtk::gio::AppInfo::launch_default_for_uri(url, None::<&gtk::gio::AppLaunchContext>)
            {
                tracing::warn!(url, error = %err, "failed to open URL");
            }
        });
        row
    }
}

impl Default for AboutPage {
    fn default() -> Self {
        Self::new()
    }
}
