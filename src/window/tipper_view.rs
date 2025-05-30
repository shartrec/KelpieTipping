/*
 * Copyright (c) 2025. Trevor Campbell and others.
 *
 * This file is part of KelpieTipping.
 *
 * KelpieTipping is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License,or
 * (at your option) any later version.
 *
 * KelpieTipping is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with KelpieTipping; if not, write to the Free Software
 * Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA
 *
 * Contributors:
 *      Trevor Campbell
 *
 */
#![forbid(unsafe_code)]

use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use crate::event;
    use crate::event::Event;
    use crate::model::tipper;
    use crate::model::tipper::{Tipper, Tippers};
    use crate::util::db;
    use crate::window::edit_tipper::TipperDialog;
    use crate::window::util::build_column_factory;
    use adw::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::{Button, ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, SingleSelection};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_tipping/tipper_view.ui")]
    pub struct TipperView {
        #[template_child]
        pub tipper_list: TemplateChild<ColumnView>,
        #[template_child]
        pub col_name: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_email: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_delete: TemplateChild<ColumnViewColumn>,
    }

    impl TipperView {
        pub fn initialise(&self) {
            if let Some(rx) = event::manager().register_listener() {
                glib::spawn_future_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        if let Event::TippersChanged = ev {
                            view.refresh();
                        }
                    }
                }));
            }
            self.refresh();
        }

        fn refresh(&self) {
            let selection_model = SingleSelection::new(Some(Tippers::new()));
            self.tipper_list.set_model(Some(&selection_model));
            self.tipper_list.queue_draw();
        }

        fn get_model_tipper(&self, sel_ap: u32) -> Option<Tipper> {
            let selection = self.tipper_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                object.downcast::<Tipper>().ok()
            } else {
                None
            }
        }

    }

    #[glib::object_subclass]
    impl ObjectSubclass for TipperView {
        const NAME: &'static str = "TipperView";
        type Type = super::TipperView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TipperView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.tipper_list.connect_activate(
                clone!(#[weak(rename_to = view)] self, move | _list_view, position | {
                    if let Ok(w) = view.obj().root()
                        .as_ref()
                        .expect("Can't get the root window")
                        .clone()
                        .downcast::<gtk::Window>() {

                        if let Some(tipper) = view.get_model_tipper(position) {
                            let tipper_dialog = TipperDialog::new();
                            tipper_dialog.imp().set_tipper(Some(tipper));
                            tipper_dialog.set_transient_for(Some(&w));
                            tipper_dialog.set_visible(true);
                        }
                    }
                }),
            );

            self.col_name.set_factory(Some(&build_column_factory(|label: Label, tipper: &Tipper| {
                label.set_label(tipper.name().as_str());
                label.set_xalign(0.0);
            })));

            self.col_email.set_factory(Some(&build_column_factory(|label: Label, tipper: &Tipper| {
                label.set_label(tipper.email().as_str());
                label.set_xalign(0.0);
            })));

            let f =  {
                let factory = SignalListItemFactory::new();
                factory.connect_setup(move |_, list_item| {
                    let button = Button::new();
                    button.set_icon_name("edit-delete");

                    button.connect_clicked(delete_tipper);
                    list_item
                        .downcast_ref::<ListItem>()
                        .expect("Needs to be ListItem")
                        .set_child(Some(&button));
                });

                factory.connect_bind(move |_, list_item| {
                    // Get `StringObject` from `ListItem`
                    let obj = list_item
                        .downcast_ref::<ListItem>()
                        .expect("Needs to be ListItem")
                        .item()
                        .and_downcast::<Tipper>()
                        .expect("The item has to be an <T>.");

                    // Get `Label` from `ListItem`
                    let button = list_item
                        .downcast_ref::<ListItem>()
                        .expect("Needs to be ListItem")
                        .child()
                        .and_downcast::<Button>()
                        .expect("The child has to be a `Label`.");

                    button.set_action_target(Some(obj.id()));

                });
                factory
            };
            self.col_delete.set_factory(Some(&f));
        }
    }

    impl BoxImpl for TipperView {}

    impl WidgetImpl for TipperView {}

    fn delete_tipper(button: &Button) {
        if let Some(value) = button.action_target_value() {
            if let Some(id) = value.get::<i32>() {
                let pool = db::manager().pool();
                glib::spawn_future_local(clone!(async move {
                let _ = tipper::delete(pool, id).await;
                event::manager().notify_listeners(Event::TippersChanged);
            }));
            }
        }
    }
}

glib::wrapper! {
    pub struct TipperView(ObjectSubclass<imp::TipperView>)
        @extends gtk::Widget, gtk::Box;
}

impl TipperView {
    pub fn new() -> Self {
        glib::Object::new::<TipperView>()
    }
}

impl Default for TipperView {
    fn default() -> Self {
        Self::new()
    }
}
