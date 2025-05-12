/*
 * Copyright (c) 2003-2003-2023. Trevor Campbell and others.
 *
 * This file is part of Kelpie Tipping.
 *
 * Kelpie Tipping is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * Kelpie Tipping is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Kelpie Tipping; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */
#![forbid(unsafe_code)]

use adw::{TabPage, TabView};
use adw::subclass::prelude::AdwApplicationWindowImpl;
use async_std::task;
use glib::Propagation;
use glib::subclass::InitializingObject;
use gtk::{AlertDialog, CompositeTemplate, FileDialog, glib, Label, Notebook, Paned};
use gtk::gio::{Cancellable, File};
use gtk::glib::{clone, MainContext};
use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;

enum SaveType {
    Native,
    FgRouteManager,
}


// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/shartrec/kelpie_tipping/window.ui")]
pub struct Window {

}

impl Window {


}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "KelpieTippingWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        let obj = self.obj();
        obj.setup_actions();
        obj.load_window_size();


    }

}

// Trait to allow us to add menubars
impl BuildableImpl for Window {}

// Trait shared by all widgets
impl WidgetImpl for Window {
}

// Trait shared by all windows
impl WindowImpl for Window {
    // Save window state right before the window will be closed
    fn close_request(&self) -> Propagation {
        // Save window size
        self.obj()
            .save_window_size()
            .expect("Failed to save window state");

        Propagation::Proceed
    }
}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}

impl AdwApplicationWindowImpl for Window {}
