//! Journal log viewer with search, refresh, follow, and copy.

use gtk4::prelude::*;
use gtk4::{
    self as gtk, Align, Label, Orientation, ScrolledWindow, SearchEntry, TextBuffer, TextView,
};
use std::cell::Cell;
use std::rc::Rc;

use crate::services::LogEntry;

pub struct LogViewer {
    pub widget: gtk::Box,
    text_view: TextView,
    buffer: TextBuffer,
    search_entry: SearchEntry,
    follow_toggle: gtk::ToggleButton,
    status: Label,
    entries: Rc<std::cell::RefCell<Vec<LogEntry>>>,
    auto_scroll: Rc<Cell<bool>>,
}

impl LogViewer {
    pub fn new() -> Self {
        let outer = gtk::Box::new(Orientation::Vertical, 0);
        outer.set_hexpand(true);
        outer.set_vexpand(true);

        let header = Label::builder()
            .label("Logs")
            .css_classes(["heading"])
            .halign(Align::Start)
            .margin_start(12)
            .margin_end(12)
            .margin_top(8)
            .build();

        let search_entry = SearchEntry::builder()
            .placeholder_text("Filter logs…")
            .hexpand(true)
            .build();

        let refresh_btn = gtk::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Refresh logs"));
        refresh_btn.set_action_name(Some("win.refresh-logs"));

        let follow_toggle = gtk::ToggleButton::builder()
            .icon_name("media-playlist-repeat-symbolic")
            .tooltip_text("Follow (auto-refresh)")
            .build();

        let copy_btn = gtk::Button::from_icon_name("edit-copy-symbolic");
        copy_btn.set_tooltip_text(Some("Copy logs"));

        let toolbar = gtk::Box::new(Orientation::Horizontal, 6);
        toolbar.set_margin_start(12);
        toolbar.set_margin_end(12);
        toolbar.set_margin_top(6);
        toolbar.set_margin_bottom(6);
        toolbar.append(&search_entry);
        toolbar.append(&refresh_btn);
        toolbar.append(&follow_toggle);
        toolbar.append(&copy_btn);

        let buffer = TextBuffer::new(None);
        let text_view = TextView::builder()
            .buffer(&buffer)
            .editable(false)
            .cursor_visible(false)
            .monospace(true)
            .wrap_mode(gtk::WrapMode::WordChar)
            .left_margin(8)
            .right_margin(8)
            .top_margin(8)
            .bottom_margin(8)
            .hexpand(true)
            .vexpand(true)
            .build();

        let scrolled = ScrolledWindow::builder()
            .child(&text_view)
            .hexpand(true)
            .vexpand(true)
            .min_content_height(160)
            .build();

        let status = Label::builder()
            .label("No logs")
            .css_classes(["dimmed"])
            .halign(Align::Start)
            .margin_start(12)
            .margin_end(12)
            .margin_top(4)
            .margin_bottom(8)
            .build();

        outer.append(&header);
        outer.append(&toolbar);
        outer.append(&scrolled);
        outer.append(&status);

        let entries = Rc::new(std::cell::RefCell::new(Vec::new()));
        let auto_scroll = Rc::new(Cell::new(true));

        let entries_for_search = entries.clone();
        let buffer_for_search = buffer.clone();
        let status_for_search = status.clone();
        search_entry.connect_search_changed(move |entry| {
            let query = entry.text().to_string();
            let all = entries_for_search.borrow();
            let filtered = crate::services::JournalService::filter_entries(&all, &query);
            Self::apply_entries_to_buffer(&buffer_for_search, &filtered);
            status_for_search.set_label(&format!(
                "{} line{}{}",
                filtered.len(),
                if filtered.len() == 1 { "" } else { "s" },
                if query.is_empty() {
                    String::new()
                } else {
                    format!(" (filtered from {})", all.len())
                }
            ));
        });

        let buffer_for_copy = buffer.clone();
        copy_btn.connect_clicked(move |_| {
            let (start, end) = buffer_for_copy.bounds();
            let text = buffer_for_copy.text(&start, &end, false);
            if let Some(display) = gtk::gdk::Display::default() {
                display.clipboard().set_text(&text);
            }
        });

        Self {
            widget: outer,
            text_view,
            buffer,
            search_entry,
            follow_toggle,
            status,
            entries,
            auto_scroll,
        }
    }

    fn apply_entries_to_buffer(buffer: &TextBuffer, entries: &[LogEntry]) {
        let text = entries
            .iter()
            .map(|e| e.line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        buffer.set_text(&text);
    }

    pub fn set_entries(&self, entries: Vec<LogEntry>) {
        let query = self.search_entry.text().to_string();
        *self.entries.borrow_mut() = entries;
        let all = self.entries.borrow();
        let filtered = crate::services::JournalService::filter_entries(&all, &query);
        let count = filtered.len();
        Self::apply_entries_to_buffer(&self.buffer, &filtered);
        self.status.set_label(&format!(
            "{count} line{}",
            if count == 1 { "" } else { "s" }
        ));

        if self.auto_scroll.get() {
            let mark = self.buffer.create_mark(None, &self.buffer.end_iter(), false);
            self.text_view.scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
        }
    }

    pub fn set_status(&self, text: &str) {
        self.status.set_label(text);
    }

    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.buffer.set_text("");
        self.status.set_label("No logs");
    }

    pub fn is_follow_enabled(&self) -> bool {
        self.follow_toggle.is_active()
    }

    pub fn connect_follow_toggled<F: Fn(bool) + 'static>(&self, f: F) {
        self.follow_toggle.connect_toggled(move |btn| {
            f(btn.is_active());
        });
    }
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new()
    }
}
