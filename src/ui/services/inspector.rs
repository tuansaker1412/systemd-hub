//! Collapsible inspector: service details and logs as separate pages.

use gtk4::prelude::*;
use gtk4::{self as gtk};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::{LogViewer, ServiceDetailPage};

/// Which page is visible in the inspector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorPage {
    Details,
    Logs,
}

/// Right-hand panel shown only when a service is selected.
///
/// Contains a view switcher for Details vs Logs and a collapse control.
pub struct ServiceInspector {
    pub widget: adw::ToolbarView,
    detail: ServiceDetailPage,
    logs: LogViewer,
    stack: adw::ViewStack,
    page_handlers: Rc<RefCell<Vec<Box<dyn Fn(InspectorPage)>>>>,
    collapse_handlers: Rc<RefCell<Vec<Box<dyn Fn()>>>>,
}

impl ServiceInspector {
    pub fn new() -> Self {
        let detail = ServiceDetailPage::new();
        let logs = LogViewer::new();

        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);
        stack.set_hexpand(true);

        let details_page = stack.add_titled(&detail.widget, Some("details"), "Details");
        details_page.set_icon_name(Some("dialog-information-symbolic"));

        let logs_page = stack.add_titled(&logs.widget, Some("logs"), "Logs");
        logs_page.set_icon_name(Some("utilities-system-monitor-symbolic"));

        let switcher = adw::ViewSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_policy(adw::ViewSwitcherPolicy::Wide);

        let collapse_btn = gtk::Button::from_icon_name("go-next-symbolic");
        collapse_btn.set_tooltip_text(Some("Collapse panel"));
        collapse_btn.add_css_class("flat");

        let header = adw::HeaderBar::new();
        header.set_show_start_title_buttons(false);
        header.set_show_end_title_buttons(false);
        header.set_title_widget(Some(&switcher));
        header.pack_end(&collapse_btn);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&stack));
        toolbar.add_css_class("view");

        let page_handlers: Rc<RefCell<Vec<Box<dyn Fn(InspectorPage)>>>> =
            Rc::new(RefCell::new(Vec::new()));
        let collapse_handlers: Rc<RefCell<Vec<Box<dyn Fn()>>>> = Rc::new(RefCell::new(Vec::new()));

        {
            let handlers = page_handlers.clone();
            stack.connect_notify_local(Some("visible-child-name"), move |stack, _| {
                let page = match stack.visible_child_name().as_deref() {
                    Some("logs") => InspectorPage::Logs,
                    _ => InspectorPage::Details,
                };
                for handler in handlers.borrow().iter() {
                    handler(page);
                }
            });
        }

        {
            let handlers = collapse_handlers.clone();
            collapse_btn.connect_clicked(move |_| {
                for handler in handlers.borrow().iter() {
                    handler();
                }
            });
        }

        Self {
            widget: toolbar,
            detail,
            logs,
            stack,
            page_handlers,
            collapse_handlers,
        }
    }

    pub fn detail(&self) -> &ServiceDetailPage {
        &self.detail
    }

    pub fn logs(&self) -> &LogViewer {
        &self.logs
    }

    pub fn show_details(&self) {
        self.stack.set_visible_child_name("details");
    }

    pub fn show_logs(&self) {
        self.stack.set_visible_child_name("logs");
    }

    pub fn visible_page(&self) -> InspectorPage {
        match self.stack.visible_child_name().as_deref() {
            Some("logs") => InspectorPage::Logs,
            _ => InspectorPage::Details,
        }
    }

    pub fn is_logs_visible(&self) -> bool {
        self.visible_page() == InspectorPage::Logs
    }

    pub fn connect_page_changed<F: Fn(InspectorPage) + 'static>(&self, f: F) {
        self.page_handlers.borrow_mut().push(Box::new(f));
    }

    pub fn connect_collapse_clicked<F: Fn() + 'static>(&self, f: F) {
        self.collapse_handlers.borrow_mut().push(Box::new(f));
    }

    pub fn clear(&self) {
        self.detail.clear();
        self.logs.clear();
        self.show_details();
    }
}

impl Default for ServiceInspector {
    fn default() -> Self {
        Self::new()
    }
}
