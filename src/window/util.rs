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

use std::ffi::CStr;
use adw::AlertDialog;
use gtk::{AboutDialog, glib, Label, ListItem, Root, ScrolledWindow, SignalListItemFactory, TreeExpander, TreeListRow, TreeListModel};
use gtk::gdk::Texture;
use gtk::glib::Object;
use adw::prelude::{AdwDialogExt, AlertDialogExt, Cast, CastNone, EditableExt, EditableExtManual, GtkWindowExt, IsA, ListItemExt, WidgetExt};
use gettextrs::gettext;
use crate::util;
use crate::window::Window;

pub fn show_error_dialog(root: &Option<Root>, message: &str) {
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
        let label = Label::new(None);
        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&label));
    });

    factory.connect_bind(move |_, list_item| {
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

pub(crate) fn build_tree_column_factory(f: fn(Label, &TreeListRow)) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        let label = Label::new(None);
        let expander = TreeExpander::new();
        expander.set_child(Some(&label));
        expander.set_indent_for_icon(true);
        expander.set_indent_for_depth(true);
        let list_item = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem");
        list_item.set_child(Some(&expander));
        list_item.set_focusable(false);
    });

    factory.connect_bind(move |_factory, list_item| {
        // Get `StringObject` from `ListItem`
        let obj = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<TreeListRow>()
            .expect("The item has to be an <T>.");

        // Get `Label` from `ListItem`
        if let Some(widget) = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .child() {
            let expander = widget.downcast::<TreeExpander>()
                .expect("The child has to be a `Expander`.");

            let widget = expander.child()
                .expect("The child has to be a `Widget`.");
            let label = widget.downcast::<Label>()
                .expect("The child has to be a `Label`.");
            // Set "label" to "value"
            f(label, &obj);

            expander.set_list_row(Some(&obj));
        }
    });
    factory
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



