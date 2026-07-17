//! Settings page: appearance and other app preferences.

use gtk4::prelude::*;
use gtk4::{self as gtk, Align, Label, Orientation, ScrolledWindow, StringList};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::models::AppTheme;

pub struct SettingsPage {
    pub widget: adw::ToolbarView,
    theme_row: adw::ComboRow,
}

impl SettingsPage {
    pub fn new() -> Self {
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(
            &Label::builder()
                .label("Settings")
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

        let title = Label::builder()
            .label("Preferences")
            .css_classes(["title-1"])
            .halign(Align::Start)
            .build();
        content.append(&title);

        let subtitle = Label::builder()
            .label("Customize how Systemd Hub looks.")
            .css_classes(["dimmed"])
            .halign(Align::Start)
            .build();
        content.append(&subtitle);

        let group = adw::PreferencesGroup::new();
        group.set_title("Appearance");
        group.set_description(Some("Choose light, dark, or follow the system preference."));

        let theme_row = adw::ComboRow::builder()
            .title("Theme")
            .subtitle("Color scheme for the application")
            .build();

        let labels: Vec<&str> = AppTheme::ALL.iter().map(|t| t.label()).collect();
        let model = StringList::new(&labels);
        theme_row.set_model(Some(&model));
        theme_row.set_expression(Some(
            gtk::PropertyExpression::new(
                gtk::StringObject::static_type(),
                None::<gtk::Expression>,
                "string",
            )
            .upcast(),
        ));

        group.add(&theme_row);
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
            theme_row,
        }
    }

    /// Reflect the currently stored theme in the combo.
    ///
    /// Call this before connecting change handlers so restore does not re-save.
    pub fn set_theme(&self, theme: AppTheme) {
        self.theme_row.set_selected(theme.index());
    }

    pub fn connect_theme_changed<F: Fn(AppTheme) + 'static>(&self, f: F) {
        self.theme_row.connect_selected_notify(move |row| {
            if let Some(theme) = AppTheme::from_index(row.selected()) {
                f(theme);
            }
        });
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}
