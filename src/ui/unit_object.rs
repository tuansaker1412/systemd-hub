//! GObject wrapper around [`UnitSummary`] for use with Gio list models.

use glib::Object;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;

use crate::models::UnitSummary;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct UnitObject {
        pub data: RefCell<Option<UnitSummary>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UnitObject {
        const NAME: &'static str = "SystemdHubUnitObject";
        type Type = super::UnitObject;
    }

    impl ObjectImpl for UnitObject {}
}

glib::wrapper! {
    pub struct UnitObject(ObjectSubclass<imp::UnitObject>);
}

impl UnitObject {
    pub fn new(summary: UnitSummary) -> Self {
        let obj: Self = Object::builder().build();
        *obj.imp().data.borrow_mut() = Some(summary);
        obj
    }

    pub fn summary(&self) -> UnitSummary {
        self.imp()
            .data
            .borrow()
            .as_ref()
            .expect("UnitObject data missing")
            .clone()
    }

    pub fn name(&self) -> String {
        self.summary().name
    }

    pub fn description(&self) -> String {
        self.summary().description
    }

    pub fn active_state(&self) -> String {
        self.summary().active_state
    }

    pub fn enabled_state(&self) -> String {
        self.summary().enabled_state
    }

    pub fn sub_state(&self) -> String {
        self.summary().sub_state
    }

    pub fn status_label(&self) -> String {
        self.summary().status_label()
    }
}
