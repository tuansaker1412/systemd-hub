//! Service detail panel with action buttons.

use gtk4::prelude::*;
use gtk4::{self as gtk, Align, Label, Orientation};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::models::{ServiceAction, UnitDetail};
use crate::utils::format_bytes;

pub struct ServiceDetailPage {
    pub widget: gtk::Box,
    title: Label,
    description: Label,
    active_state: Label,
    sub_state: Label,
    main_pid: Label,
    memory: Label,
    exec_start: Label,
    fragment_path: Label,
    enabled_state: Label,
    action_box: gtk::Box,
    stack: gtk::Stack,
}

impl ServiceDetailPage {
    pub fn new() -> Self {
        let outer = gtk::Box::new(Orientation::Vertical, 0);
        outer.set_hexpand(true);

        let stack = gtk::Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);

        let empty = adw::StatusPage::builder()
            .icon_name("application-x-executable-symbolic")
            .title("No service selected")
            .description("Select a service from the list to view details and logs.")
            .build();

        let content = gtk::Box::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(8);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let title = Label::builder()
            .label("—")
            .css_classes(["title-2"])
            .halign(Align::Start)
            .selectable(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        content.append(&title);

        let description = Label::builder()
            .label("")
            .css_classes(["dimmed"])
            .halign(Align::Start)
            .wrap(true)
            .xalign(0.0)
            .selectable(true)
            .build();
        content.append(&description);

        let action_box = gtk::Box::new(Orientation::Horizontal, 6);
        action_box.set_margin_top(4);
        action_box.set_margin_bottom(4);
        for action in [
            ServiceAction::Start,
            ServiceAction::Stop,
            ServiceAction::Restart,
            ServiceAction::Reload,
            ServiceAction::Enable,
            ServiceAction::Disable,
        ] {
            let btn = gtk::Button::with_label(action.label());
            btn.add_css_class("pill");
            match action {
                ServiceAction::Start => btn.add_css_class("suggested-action"),
                ServiceAction::Stop => btn.add_css_class("destructive-action"),
                _ => {}
            }
            btn.set_action_name(Some("win.service-action"));
            btn.set_action_target_value(Some(&action.as_str().to_variant()));
            action_box.append(&btn);
        }
        content.append(&action_box);

        let group = adw::PreferencesGroup::new();
        group.set_title("Details");

        let active_state = Self::value_label();
        let sub_state = Self::value_label();
        let main_pid = Self::value_label();
        let memory = Self::value_label();
        let enabled_state = Self::value_label();
        let exec_start = Self::value_label();
        let fragment_path = Self::value_label();

        group.add(&Self::row("Active State", &active_state));
        group.add(&Self::row("Sub State", &sub_state));
        group.add(&Self::row("Main PID", &main_pid));
        group.add(&Self::row("Memory", &memory));
        group.add(&Self::row("Enabled", &enabled_state));
        group.add(&Self::row("ExecStart", &exec_start));
        group.add(&Self::row("Unit File", &fragment_path));

        content.append(&group);

        let scrolled = gtk::ScrolledWindow::builder()
            .child(&content)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();

        stack.add_named(&empty, Some("empty"));
        stack.add_named(&scrolled, Some("detail"));
        stack.set_visible_child_name("empty");

        outer.append(&stack);

        Self {
            widget: outer,
            title,
            description,
            active_state,
            sub_state,
            main_pid,
            memory,
            exec_start,
            fragment_path,
            enabled_state,
            action_box,
            stack,
        }
    }

    fn value_label() -> Label {
        Label::builder()
            .label("—")
            .xalign(1.0)
            .hexpand(true)
            .selectable(true)
            .wrap(true)
            .max_width_chars(40)
            .build()
    }

    fn row(title: &str, value: &Label) -> adw::ActionRow {
        let row = adw::ActionRow::builder().title(title).build();
        row.add_suffix(value);
        row.set_activatable(false);
        row
    }

    pub fn clear(&self) {
        self.stack.set_visible_child_name("empty");
    }

    pub fn set_detail(&self, detail: &UnitDetail) {
        self.stack.set_visible_child_name("detail");
        self.title.set_label(&detail.name);
        self.description.set_label(&detail.description);
        self.active_state.set_label(&detail.active_state);
        self.sub_state.set_label(&detail.sub_state);
        self.main_pid.set_label(&if detail.main_pid == 0 {
            "—".into()
        } else {
            detail.main_pid.to_string()
        });
        self.memory.set_label(&match detail.memory_bytes {
            Some(b) => format_bytes(b),
            None => "—".into(),
        });
        self.enabled_state.set_label(&detail.enabled_state);
        self.exec_start.set_label(if detail.exec_start.is_empty() {
            "—"
        } else {
            &detail.exec_start
        });
        self.fragment_path
            .set_label(if detail.fragment_path.is_empty() {
                "—"
            } else {
                &detail.fragment_path
            });

        // Enable/disable action sensitivity based on state.
        let running = detail.active_state == "active";
        let mut child = self.action_box.first_child();
        while let Some(widget) = child {
            if let Ok(btn) = widget.clone().downcast::<gtk::Button>() {
                if let Some(target) = btn.action_target_value() {
                    let action = target.str().unwrap_or_default();
                    let sensitive = match action {
                        "start" => !running,
                        "stop" | "restart" | "reload" => running,
                        _ => true,
                    };
                    btn.set_sensitive(sensitive);
                }
            }
            child = widget.next_sibling();
        }
    }

    pub fn set_loading_name(&self, name: &str) {
        self.stack.set_visible_child_name("detail");
        self.title.set_label(name);
        self.description.set_label("Loading…");
    }
}

impl Default for ServiceDetailPage {
    fn default() -> Self {
        Self::new()
    }
}
