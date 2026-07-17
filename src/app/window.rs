//! Main application window: sidebar navigation + content pages.

use glib::{clone, ControlFlow, SourceId};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{self as gtk, gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::{Cell, RefCell};
use std::time::Duration;

use crate::models::{ServiceAction, UnitSummary};
use crate::services::{JournalService, SystemInfoService, UnitService};
use crate::ui::{DashboardPage, InspectorPage, ServicesView, Sidebar, SidebarPage};
use crate::RUNTIME;

const FOLLOW_INTERVAL_MS: u64 = 2_000;

glib::wrapper! {
    pub struct SystemdHubWindow(ObjectSubclass<imp::SystemdHubWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SystemdHubWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .property("title", "Systemd Hub")
            .property("default-width", 1280)
            .property("default-height", 800)
            .build()
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Default)]
    pub struct SystemdHubWindow {
        pub toast_overlay: RefCell<Option<adw::ToastOverlay>>,
        pub content_stack: RefCell<Option<gtk::Stack>>,
        pub dashboard: RefCell<Option<DashboardPage>>,
        pub services_view: RefCell<Option<ServicesView>>,
        pub unit_service: RefCell<Option<UnitService>>,
        pub selected_unit: RefCell<Option<String>>,
        pub follow_source: RefCell<Option<SourceId>>,
        pub built: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemdHubWindow {
        const NAME: &'static str = "SystemdHubWindow";
        type Type = super::SystemdHubWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for SystemdHubWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.build_ui();
            obj.setup_actions();
            obj.load_dashboard();
            obj.refresh_services();
        }

        fn dispose(&self) {
            if let Some(id) = self.follow_source.borrow_mut().take() {
                id.remove();
            }
        }
    }

    impl WidgetImpl for SystemdHubWindow {}
    impl WindowImpl for SystemdHubWindow {}
    impl ApplicationWindowImpl for SystemdHubWindow {}
    impl AdwApplicationWindowImpl for SystemdHubWindow {}
}

impl SystemdHubWindow {
    fn build_ui(&self) {
        let imp = self.imp();
        if imp.built.replace(true) {
            return;
        }

        *imp.unit_service.borrow_mut() = Some(UnitService::new());

        let sidebar = Sidebar::new();
        let dashboard = DashboardPage::new();
        let services_view = ServicesView::new();

        let content_stack = gtk::Stack::new();
        content_stack.set_hexpand(true);
        content_stack.set_vexpand(true);
        content_stack.add_named(&dashboard.widget, Some("dashboard"));
        content_stack.add_named(&services_view.widget, Some("services"));
        content_stack.set_visible_child_name("dashboard");

        let sidebar_page = adw::NavigationPage::builder()
            .title("Systemd Hub")
            .child(&sidebar.widget)
            .build();
        let content_page = adw::NavigationPage::builder()
            .title("Content")
            .child(&content_stack)
            .build();

        let split = adw::NavigationSplitView::new();
        split.set_sidebar(Some(&sidebar_page));
        split.set_content(Some(&content_page));
        split.set_min_sidebar_width(200.0);
        split.set_max_sidebar_width(280.0);
        split.set_sidebar_width_fraction(0.18);

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&split));

        self.set_content(Some(&toast_overlay));

        *imp.toast_overlay.borrow_mut() = Some(toast_overlay);
        *imp.content_stack.borrow_mut() = Some(content_stack);
        *imp.dashboard.borrow_mut() = Some(dashboard);
        *imp.services_view.borrow_mut() = Some(services_view);

        sidebar.connect_page_selected(clone!(
            #[weak(rename_to = window)]
            self,
            move |page| {
                {
                    let stack = window.imp().content_stack.borrow();
                    let Some(stack) = stack.as_ref() else {
                        return;
                    };
                    match page {
                        SidebarPage::Dashboard => stack.set_visible_child_name("dashboard"),
                        SidebarPage::Services => stack.set_visible_child_name("services"),
                    }
                }
                if page == SidebarPage::Dashboard {
                    window.load_dashboard();
                }
            }
        ));

        if let Some(view) = imp.services_view.borrow().as_ref() {
            view.connect_selection_changed(clone!(
                #[weak(rename_to = window)]
                self,
                move |unit| {
                    window.on_unit_selected(unit);
                }
            ));

            view.connect_follow_toggled(clone!(
                #[weak(rename_to = window)]
                self,
                move |enabled| {
                    window.set_follow_mode(enabled);
                }
            ));

            view.connect_inspector_page_changed(clone!(
                #[weak(rename_to = window)]
                self,
                move |page| {
                    window.on_inspector_page_changed(page);
                }
            ));
        }
    }

    fn setup_actions(&self) {
        let refresh_services = gio::SimpleAction::new("refresh-services", None);
        refresh_services.connect_activate(clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| {
                window.refresh_services();
            }
        ));
        self.add_action(&refresh_services);

        let refresh_logs = gio::SimpleAction::new("refresh-logs", None);
        refresh_logs.connect_activate(clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| {
                window.refresh_logs();
            }
        ));
        self.add_action(&refresh_logs);

        let service_action =
            gio::SimpleAction::new("service-action", Some(glib::VariantTy::STRING));
        service_action.connect_activate(clone!(
            #[weak(rename_to = window)]
            self,
            move |_, param| {
                let Some(variant) = param else { return };
                let Some(action_str) = variant.str() else { return };
                let action = match action_str {
                    "start" => ServiceAction::Start,
                    "stop" => ServiceAction::Stop,
                    "restart" => ServiceAction::Restart,
                    "reload" => ServiceAction::Reload,
                    "enable" => ServiceAction::Enable,
                    "disable" => ServiceAction::Disable,
                    _ => return,
                };
                window.perform_service_action(action);
            }
        ));
        self.add_action(&service_action);

        if let Some(app) = self.application() {
            app.set_accels_for_action("win.refresh-services", &["<Control>r"]);
            app.set_accels_for_action("win.refresh-logs", &["<Control><Shift>r"]);
        }
    }

    fn toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(3);
        if let Some(overlay) = self.imp().toast_overlay.borrow().as_ref() {
            overlay.add_toast(toast);
        }
    }

    fn load_dashboard(&self) {
        let imp = self.imp();
        if let Some(page) = imp.dashboard.borrow().as_ref() {
            page.set_loading();
        }

        let result = SystemInfoService::collect();
        if let Some(page) = imp.dashboard.borrow().as_ref() {
            match result {
                Ok(info) => page.set_info(&info),
                Err(e) => {
                    tracing::error!(error = %e, "failed to load system info");
                    page.set_error(&e.to_string());
                }
            }
        }
    }

    fn refresh_services(&self) {
        let imp = self.imp();
        if let Some(view) = imp.services_view.borrow().as_ref() {
            view.set_status("Loading services…");
        }

        let Some(service) = imp.unit_service.borrow().clone() else {
            return;
        };

        let (tx, rx) = async_channel::bounded(1);
        RUNTIME.spawn(async move {
            let result = service.list_services().await;
            let _ = tx.send(result).await;
        });

        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                match rx.recv().await {
                    Ok(Ok(units)) => {
                        let count = units.len();
                        if let Some(view) = window.imp().services_view.borrow().as_ref() {
                            view.set_units(units);
                        }
                        tracing::info!(count, "loaded services");
                    }
                    Ok(Err(e)) => {
                        tracing::error!(error = %e, "failed to list services");
                        if let Some(view) = window.imp().services_view.borrow().as_ref() {
                            view.set_status(&format!("Error: {e}"));
                        }
                        window.toast(&format!("Failed to list services: {e}"));
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "channel closed while listing services");
                    }
                }
            }
        ));
    }

    fn on_unit_selected(&self, unit: Option<UnitSummary>) {
        let imp = self.imp();
        match unit {
            None => {
                *imp.selected_unit.borrow_mut() = None;
                self.set_follow_mode(false);
                if let Some(view) = imp.services_view.borrow().as_ref() {
                    view.clear_selection_ui();
                }
            }
            Some(summary) => {
                *imp.selected_unit.borrow_mut() = Some(summary.name.clone());
                if let Some(view) = imp.services_view.borrow().as_ref() {
                    // Always open Details first; logs load only when user opens Logs.
                    view.open_details();
                    view.detail().set_loading_name(&summary.name);
                    view.logs().clear();
                }
                self.set_follow_mode(false);
                self.load_unit_detail(summary.name);
            }
        }
    }

    fn on_inspector_page_changed(&self, page: InspectorPage) {
        match page {
            InspectorPage::Details => {
                // Stop auto-follow when leaving logs.
                if let Some(view) = self.imp().services_view.borrow().as_ref() {
                    view.logs().set_follow_enabled(false);
                }
                self.set_follow_mode(false);
            }
            InspectorPage::Logs => {
                self.refresh_logs();
            }
        }
    }

    fn load_unit_detail(&self, name: String) {
        let imp = self.imp();
        let Some(service) = imp.unit_service.borrow().clone() else {
            return;
        };

        let (tx, rx) = async_channel::bounded(1);
        let name_for_task = name.clone();
        RUNTIME.spawn(async move {
            let result = service.get_detail(&name_for_task).await;
            let _ = tx.send(result).await;
        });

        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                match rx.recv().await {
                    Ok(Ok(detail)) => {
                        let current = window.imp().selected_unit.borrow().clone();
                        if current.as_deref() != Some(detail.name.as_str()) {
                            return;
                        }
                        if let Some(view) = window.imp().services_view.borrow().as_ref() {
                            view.detail().set_detail(&detail);
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!(error = %e, unit = %name, "failed to load unit detail");
                        window.toast(&format!("Failed to load {name}: {e}"));
                    }
                    Err(_) => {}
                }
            }
        ));
    }

    fn perform_service_action(&self, action: ServiceAction) {
        let imp = self.imp();
        let Some(name) = imp.selected_unit.borrow().clone() else {
            self.toast("No service selected");
            return;
        };
        let Some(service) = imp.unit_service.borrow().clone() else {
            return;
        };

        self.toast(&format!("{} {}…", action.label(), name));

        let (tx, rx) = async_channel::bounded(1);
        let name_for_task = name.clone();
        RUNTIME.spawn(async move {
            let result = service.perform_action(&name_for_task, action).await;
            let _ = tx.send(result).await;
        });

        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                match rx.recv().await {
                    Ok(Ok(())) => {
                        window.toast(&format!("{} succeeded for {}", action.label(), name));
                        glib::timeout_add_local_once(
                            Duration::from_millis(400),
                            clone!(
                                #[weak]
                                window,
                                move || {
                                    window.load_unit_detail(name.clone());
                                    window.refresh_services();
                                    let logs_open = window
                                        .imp()
                                        .services_view
                                        .borrow()
                                        .as_ref()
                                        .map(|v| v.is_logs_visible())
                                        .unwrap_or(false);
                                    if logs_open {
                                        window.refresh_logs();
                                    }
                                }
                            ),
                        );
                    }
                    Ok(Err(e)) => {
                        tracing::error!(
                            error = %e,
                            unit = %name,
                            action = action.as_str(),
                            "action failed"
                        );
                        window.toast(&format!("{} failed: {e}", action.label()));
                    }
                    Err(_) => {}
                }
            }
        ));
    }

    fn refresh_logs(&self) {
        let imp = self.imp();
        let Some(name) = imp.selected_unit.borrow().clone() else {
            if let Some(view) = imp.services_view.borrow().as_ref() {
                view.logs().clear();
            }
            return;
        };

        // Only fetch when the Logs tab is actually visible.
        let logs_visible = imp
            .services_view
            .borrow()
            .as_ref()
            .map(|v| v.is_logs_visible())
            .unwrap_or(false);
        if !logs_visible {
            return;
        }

        if let Some(view) = imp.services_view.borrow().as_ref() {
            view.logs().set_status("Loading logs…");
        }

        let (tx, rx) = async_channel::bounded(1);
        let name_for_task = name.clone();
        RUNTIME.spawn(async move {
            let result = JournalService::fetch_logs(&name_for_task, 200).await;
            let _ = tx.send(result).await;
        });

        glib::spawn_future_local(clone!(
            #[weak(rename_to = window)]
            self,
            async move {
                match rx.recv().await {
                    Ok(Ok(entries)) => {
                        let current = window.imp().selected_unit.borrow().clone();
                        if current.as_deref() != Some(name.as_str()) {
                            return;
                        }
                        if let Some(view) = window.imp().services_view.borrow().as_ref() {
                            if !view.is_logs_visible() {
                                return;
                            }
                            view.logs().set_entries(entries);
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!(error = %e, unit = %name, "failed to load logs");
                        if let Some(view) = window.imp().services_view.borrow().as_ref() {
                            view.logs().set_status(&format!("Error: {e}"));
                        }
                    }
                    Err(_) => {}
                }
            }
        ));
    }

    fn set_follow_mode(&self, enabled: bool) {
        let imp = self.imp();
        if let Some(id) = imp.follow_source.borrow_mut().take() {
            id.remove();
        }
        if !enabled {
            return;
        }

        let id = glib::timeout_add_local(
            Duration::from_millis(FOLLOW_INTERVAL_MS),
            clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or]
                ControlFlow::Break,
                move || {
                    let still = window
                        .imp()
                        .services_view
                        .borrow()
                        .as_ref()
                        .map(|v| v.is_logs_visible() && v.logs().is_follow_enabled())
                        .unwrap_or(false);
                    if still {
                        window.refresh_logs();
                        ControlFlow::Continue
                    } else {
                        ControlFlow::Break
                    }
                }
            ),
        );
        *imp.follow_source.borrow_mut() = Some(id);
    }
}
