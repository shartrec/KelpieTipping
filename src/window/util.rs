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
use std::sync::Arc;
use crate::util;
use crate::window::Window;
use adw::gdk::{Key, ModifierType};
use adw::glib;
use adw::glib::{clone, Propagation};
use adw::prelude::{AdwDialogExt, AlertDialogExt, Cast, CastNone, EditableExt, GtkWindowExt, IsA, ListItemExt, WidgetExt};
use adw::AlertDialog;
use gettextrs::gettext;
use gtk::gdk::Texture;
use gtk::glib::Object;
use gtk::prelude::{ActionableExtManual, ButtonExt};
use gtk::{AboutDialog, Button, Entry, Label, ListItem, Root, SignalListItemFactory, Widget};
use crate::model::game::Game;

pub(crate) fn connect_escape<W: IsA<Widget> + adw::glib::clone::Downgrade>(widget: &W, button: &Button) {
    // Create the key controller
    let ev_key = gtk::EventControllerKey::new();
    ev_key.connect_key_pressed(
        clone!(#[weak] button,
            #[upgrade_or] Propagation::Proceed,
            move | _event, key_val, _key_code, modifier | {
                if key_val == Key::Escape && modifier == ModifierType::empty() {
                    button.emit_clicked();
                    Propagation::Stop
                } else {
                    Propagation::Proceed
                }

            }));
    widget.add_controller(ev_key);
}

pub(crate) fn show_error_dialog(root: &Option<Root>, message: &str) {
    // Create a new message dialog
    if let Ok(w) = root
        .as_ref()
        .expect("Can't get the root window")
        .clone()
        .downcast::<gtk::Window>()
    {
        let message = gettext(message);
        let dialog = AlertDialog::new(None, Some(&*message));
        dialog.add_response("OK", "Ok");
        dialog.present(Some(&w));
    };
}

pub(crate) fn build_column_factory<F: Fn(Label, &T) + 'static, T: IsA<Object>>(f: F) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(Some("(none)"));
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&label));
    });

    factory.connect_bind(move |_f, list_item| {
        // Get `StringObject` from `ListItem`
        let obj = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<T>()
            .expect("The item has to be an <T>.");

        // Get `Label` from `ListItem`
        let label = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .child()
            .and_downcast::<Label>()
            .expect("The child has to be a `Label`.");

        // Set "label" to "number"
        f(label, &obj);
    });
    factory
}

pub(crate) fn build_del_column_factory<FGet: Fn(&Button, &T) + 'static, T: IsA<Object>, FDel: Fn(&Button) + 'static>(get_id: FGet, do_delete: FDel) -> SignalListItemFactory {
    let f = {
        let do_delete = Arc::new(do_delete);
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let button = Button::new();
            button.set_icon_name("list-remove-symbolic");
            let do_delete = Arc::clone(&do_delete);
            button.connect_clicked(move |btn| {
                do_delete(&btn);
            });
            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(Some(&button));
        });

        factory.connect_bind(move |_f, list_item| {
            // Get `StringObject` from `ListItem`
            let obj = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .item()
                .and_downcast::<T>()
                .expect("The item has to be an <T>.");

            // Get `Label` from `ListItem`
            let button = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<Button>()
                .expect("The child has to be a `Label`.");

            get_id(&button, &obj);
        });
        factory
    };
    f
}

pub(crate) fn show_help_about(window: &Window) {
    let icon = Texture::from_resource(
        "/com/shartrec/kelpie_tipping/images/kelpiedog_120x120_transparent.png");

    let mut builder = Object::builder::<AboutDialog>();

    builder = builder.property("program-name", util::info::PROGRAM_NAME);
    builder = builder.property("version", util::info::VERSION);
    builder = builder.property("website", util::info::WEBSITE);
    builder = builder.property("license-type", util::info::LICENSE_TYPE);
    builder = builder.property("title", util::info::ABOUT_TITLE);
    builder = builder.property("authors", [util::info::AUTHOR].as_ref());
    builder = builder.property("logo", &icon);

    let about_dialog = builder.build();
    about_dialog.set_transient_for(Some(window));
    about_dialog.set_modal(true);
    about_dialog.set_destroy_with_parent(true);

    about_dialog.set_visible(true);
}

fn validate_numeric(entry: &Entry, name: &str) -> bool {
    match entry.text().as_str().parse::<i32>() {
        Ok(_) => { true }
        Err(_) => {
            show_error_dialog(&entry.root(), format!("{} should be numeric", name).as_str());
            false
        }
    }
}

pub fn validate_not_empty(entry: &Entry, name: &str) -> bool {
    if entry.text().as_str().is_empty() {
        show_error_dialog(&entry.root(), format!("{} is required", name).as_str());
        false
    } else {
        true
    }
}