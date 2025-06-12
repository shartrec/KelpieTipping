/*
 * Copyright (c) 2025. Trevor Campbell and others.
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

use gtk::{self, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {
    use super::*;
    use crate::event;
    use crate::event::Event;
    use crate::model::team;
    use crate::model::team::{Team, Teams};
    use crate::util::db;
    use crate::window::edit_team::TeamDialog;
    use crate::window::util::{build_column_factory, build_del_column_factory};
    use adw::glib::clone;
    use glib::subclass::InitializingObject;
    use gtk::{Button, ColumnView, ColumnViewColumn, Label, ListItem, SignalListItemFactory, SingleSelection};

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/com/shartrec/kelpie_tipping/team_view.ui")]
    pub struct TeamView {
        #[template_child]
        pub team_list: TemplateChild<ColumnView>,
        #[template_child]
        pub col_name: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_nickname: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_delete: TemplateChild<ColumnViewColumn>,
    }

    impl TeamView {
        pub fn initialise(&self) {
            if let Some(rx) = event::manager().register_listener() {
                glib::spawn_future_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        if let Event::TeamsChanged = ev {
                            view.refresh();
                        }
                    }
                }));
            }
            self.refresh();
        }

        fn refresh(&self) {
            let selection_model = SingleSelection::new(Some(Teams::new()));
            self.team_list.set_model(Some(&selection_model));
            self.team_list.queue_draw();
        }

        fn get_model_team(&self, sel_ap: u32) -> Option<Team> {
            let selection = self.team_list.model().unwrap().item(sel_ap);
            if let Some(object) = selection {
                object.downcast::<Team>().ok()
            } else {
                None
            }
        }

    }

    #[glib::object_subclass]
    impl ObjectSubclass for TeamView {
        const NAME: &'static str = "TeamView";
        type Type = super::TeamView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TeamView {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.team_list.connect_activate(
                clone!(#[weak(rename_to = view)] self, move | _list_view, position | {
                    if let Ok(w) = view.obj().root()
                        .as_ref()
                        .expect("Can't get the root window")
                        .clone()
                        .downcast::<gtk::Window>() {

                        if let Some(team) = view.get_model_team(position) {
                            let team_dialog = TeamDialog::new();
                            team_dialog.imp().set_team(Some(team));
                            team_dialog.set_transient_for(Some(&w));
                            team_dialog.set_visible(true);
                        }
                    }
                }),
            );

            self.col_name.set_factory(Some(&build_column_factory(|label: Label, team: &Team| {
                label.set_label(team.name().as_str());
                label.set_xalign(0.0);
            })));

            self.col_nickname.set_factory(Some(&build_column_factory(|label: Label, team: &Team| {
                label.set_label(team.nickname().as_str());
                label.set_xalign(0.0);
            })));

            // Create a factory for the delete button column
            let f = build_del_column_factory(
                |button: &Button, team: &Team| button.set_action_target(Some(team.id())),
                delete_team);
            self.col_delete.set_factory(Some(&f));
        }
    }

    impl BoxImpl for TeamView {}

    impl WidgetImpl for TeamView {}

    fn delete_team(button: &Button) {
        if let Some(value) = button.action_target_value() {
            if let Some(id) = value.get::<i32>() {
                let pool = db::manager().pool();
                glib::spawn_future_local(clone!(async move {
                    let _ = team::delete(pool, id).await;
                    event::manager().notify_listeners(Event::TeamsChanged);
            }));
            }
        }
    }
}

glib::wrapper! {
    pub struct TeamView(ObjectSubclass<imp::TeamView>)
        @extends gtk::Widget, gtk::Box;
}

impl TeamView {
    pub fn new() -> Self {
        glib::Object::new::<TeamView>()
    }
}

impl Default for TeamView {
    fn default() -> Self {
        Self::new()
    }
}
