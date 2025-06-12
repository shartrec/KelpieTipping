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

use adw::gio;
use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use adw::gio::{SimpleAction, SimpleActionGroup};
    use super::*;
    use crate::event;
    use crate::event::Event;
    use crate::model::round::{Round, Rounds};
    use crate::util::db;
    use crate::window::edit_round::RoundDialog;
    use crate::window::util::build_column_factory;
    use adw::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::{Button, Label, ListView, ScrolledWindow, SingleSelection};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_tipping/round_view.ui")]
    pub struct RoundView {
        #[template_child]
        pub round_draw: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub round_list: TemplateChild<ListView>,
        #[template_child]
        pub round_draw_box: TemplateChild<gtk::Box>,
    }

    impl RoundView {
        pub fn initialise(&self) {
            if let Some(rx) = event::manager().register_listener() {
                glib::spawn_future_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        if let Event::RoundsChanged{round_id} = ev {
                            view.refresh(Some(round_id));
                        }
                    }
                }));
            }
            self.refresh(None);
        }

        pub(crate) fn new_round(&self) {
            while let Some(old_round) = self.round_draw_box.first_child() {
                self.round_draw_box.remove(&old_round);
            }
            let round_dialog = RoundDialog::new(&self.round_draw_box, None);
        }

        fn refresh(&self, round_id: Option<i32>) {
            let selection_model = SingleSelection::new(Some(Rounds::new()));
            self.round_list.set_model(Some(&selection_model));
            // find the position in the model with the given round_id or 0 if not found
            let position = if let Some(id) = round_id {
                let rounds = selection_model.model().unwrap().downcast::<Rounds>().unwrap();
                let x = rounds.imp().rounds.read().expect("Failed to read rounds");
                x.iter().position(|r| r.id() == id)
                    .unwrap_or(0) as u32
            } else {
                0
            };

            selection_model.select_item(position, true);
            self.activate_round(position);
            self.round_list.queue_draw();
        }

        fn get_model_round(&self, sel_ap: u32) -> Option<Round> {
            let selection = self.round_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                object.downcast::<Round>().ok()
            } else {
                None
            }
        }
        fn activate_round(&self, position: u32) {
            if let Some(round) = self.get_model_round(position) {
                while let Some(old_round) = self.round_draw_box.first_child() {
                    self.round_draw_box.get().remove(&old_round);
                }
                let _round_dialog = RoundDialog::new(&self.round_draw_box, Some(round.clone()));
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RoundView {
        const NAME: &'static str = "RoundView";
        type Type = super::RoundView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RoundView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.round_list.connect_activate(
                clone!(#[weak(rename_to = view)] self, move |_list_view, position| {
                   view.activate_round(position);
                }),
            );

            self.round_list.set_factory(Some(&build_column_factory(|label: Label, round: &Round| {
                label.set_label(format!("{}", round.number()).as_str());
                label.set_xalign(0.0);
            })));

            // let f = {
            //     let factory = SignalListItemFactory::new();
            //     factory.connect_setup(move |_, list_item| {
            //         let button = Button::new();
            //         button.set_icon_name("edit-delete");
            //
            //         button.connect_clicked(delete_round);
            //         list_item
            //             .downcast_ref::<ListItem>()
            //             .expect("Needs to be ListItem")
            //             .set_child(Some(&button));
            //     });
            //
            //     factory.connect_bind(move |r, list_item| {
            //         // Get `StringObject` from `ListItem`
            //         // let obj = list_item
            //         //     .downcast_ref::<ListItem>()
            //         //     .expect("Needs to be ListItem")
            //         //     .item()
            //         //     .and_downcast::<Round>()
            //         //     .expect("The item has to be an <T>.");
            //         //
            //         // // Get `Label` from `ListItem`
            //         // let button = list_item
            //         //     .downcast_ref::<ListItem>()
            //         //     .expect("Needs to be ListItem")
            //         //     .child()
            //         //     .and_downcast::<Button>()
            //         //     .expect("The child has to be a `Label`.");
            //         //
            //         // button.set_action_target(Some(obj.id()));
            //
            //     });
            //     factory
            // };
            // self.col_delete.set_factory(Some(&f));
        }

    }

    impl BoxImpl for RoundView {}

    impl WidgetImpl for RoundView {}

    fn delete_round(button: &Button) {
        if let Some(value) = button.action_target_value() {
            if let Some(id) = value.get::<i32>() {
                let pool = db::manager().pool();
                glib::spawn_future_local(clone!(async move {
                    //TODO
                // let _ = round::delete(pool, id).await;
                // event::manager().notify_listeners(Event::RoundsChanged);
            }));
            }
        }
    }
}

glib::wrapper! {
    pub struct RoundView(ObjectSubclass<imp::RoundView>)
        @extends gtk::Widget, gtk::Box;
}

impl RoundView {
    pub fn new() -> Self {
        glib::Object::new::<RoundView>()
    }
}

impl Default for RoundView {
    fn default() -> Self {
        Self::new()
    }
}
