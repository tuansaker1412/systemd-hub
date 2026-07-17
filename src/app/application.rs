//! Adw.Application entry point.

use gtk4::prelude::*;
use gtk4::{self as gtk, gio};
use libadwaita as adw;
use libadwaita::prelude::*;

use super::SystemdHubWindow;

const APP_ID: &str = "dev.systemdhub.SystemdHub";

glib::wrapper! {
    pub struct SystemdHubApplication(ObjectSubclass<imp::SystemdHubApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SystemdHubApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::empty())
            .build()
    }

    fn setup_actions(&self) {
        let quit = gio::SimpleAction::new("quit", None);
        quit.connect_activate(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.quit();
            }
        ));
        self.add_action(&quit);
        self.set_accels_for_action("app.quit", &["<Control>q"]);

        let about = gio::SimpleAction::new("about", None);
        about.connect_activate(glib::clone!(
            #[weak(rename_to = app)]
            self,
            move |_, _| {
                app.show_about();
            }
        ));
        self.add_action(&about);
    }

    fn show_about(&self) {
        let window = self.active_window();
        let dialog = adw::AboutDialog::builder()
            .application_name("Systemd Hub")
            .application_icon("application-x-executable")
            .developer_name("Systemd Hub Contributors")
            .version(env!("CARGO_PKG_VERSION"))
            .comments("Native Linux systemd service manager")
            .license_type(gtk::License::Gpl30)
            .developers(vec!["Systemd Hub Contributors".to_string()])
            .build();
        dialog.present(window.as_ref());
    }
}

impl Default for SystemdHubApplication {
    fn default() -> Self {
        Self::new()
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Default)]
    pub struct SystemdHubApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for SystemdHubApplication {
        const NAME: &'static str = "SystemdHubApplication";
        type Type = super::SystemdHubApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for SystemdHubApplication {}

    impl ApplicationImpl for SystemdHubApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = SystemdHubWindow::new(application.upcast_ref());
                window.upcast()
            };
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.setup_actions();

            // Restore saved theme (System / Light / Dark) before any window is shown.
            crate::services::SettingsService::load_and_apply_theme();
        }
    }

    impl GtkApplicationImpl for SystemdHubApplication {}
    impl AdwApplicationImpl for SystemdHubApplication {}
}
