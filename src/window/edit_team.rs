use adw::prelude::{EditableExt, WidgetExt};
use gtk::{gio, glib, Entry};

use crate::window::util::show_error_dialog;

mod imp {
    use log::info;
use crate::event;
    use crate::event::Event;
    use crate::model::team;
    use crate::model::team::Team;
    use crate::util::db;
    use crate::window::edit_team::validate_not_empty;
    use adw::prelude::{ButtonExt, EditableExt, GtkWindowExt, WidgetExt};
    use adw::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt, WidgetClassExt, WindowImpl};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};
    use gtk::{glib, AlertDialog, Button, CompositeTemplate, Entry, TemplateChild};
    use std::cell::RefCell;
    use std::ops::Deref;
    use adw::glib::MainContext;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_tipping/team_dialog.ui")]
    pub struct TeamDialog {
        #[template_child]
        pub team_name: TemplateChild<Entry>,
        #[template_child]
        pub team_nickname: TemplateChild<Entry>,
        #[template_child]
        pub btn_ok: TemplateChild<Button>,
        #[template_child]
        pub btn_cancel: TemplateChild<Button>,

        team_id: RefCell<Option<i32>>,
    }

    impl TeamDialog {
        pub fn set_team(&self, team: Option<Team>) {
            match team {
                Some(team) => {
                    self.team_id.replace(Some(team.id()));
                    self.team_name.set_text(&team.name());
                    self.team_nickname.set_text(&team.nickname());
                }
                None => {
                    self.team_id.replace(None);
                    self.team_name.set_text("");
                    self.team_nickname.set_text("");
                }
            }
        }

        fn validate(&self) -> bool {
            validate_not_empty(&self.team_name, "Name") &&
            validate_not_empty(&self.team_nickname, "Nickname")
        }

        fn save_team(&self) -> bool {
            // Check we have a name
            if self.team_name.text().is_empty() {
                let buttons = vec!["Ok".to_string()];
                let alert = AlertDialog::builder()
                    .message("Please enter a name")
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else if self.team_nickname.text().is_empty() {
                let buttons = vec!["Ok".to_string()];
                let alert = AlertDialog::builder()
                    .message("Please enter a nick name")
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else {
                let pool = db::manager().pool();
                if self.team_id.borrow().is_some() {
                    // Update the team
                    let id = self.team_id.borrow().unwrap();
                    let name = self.team_name.text().to_string().clone();
                    let nickname = self.team_nickname.text().to_string().clone();
                    glib::spawn_future_local(clone!( async move {
                        info!("Updating team {}", id);
                        let _ = team::update(pool, id, name, nickname).await;
                        event::manager().notify_listeners(Event::TeamsChanged);
                    }));

                } else {
                    // Create a new team
                    let name = self.team_name.text().to_string().clone();
                    let nickname = self.team_nickname.text().to_string().clone();
                    glib::spawn_future_local(clone!( async move {
                        let _ = team::insert(pool, name, nickname).await;
                        event::manager().notify_listeners(Event::TeamsChanged);
                    }));
                }
                true
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TeamDialog {
        const NAME: &'static str = "TeamDialog";
        type Type = super::TeamDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TeamDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.btn_cancel.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
               window.obj().close();
            }));

            self.btn_ok.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                if window.validate() && window.save_team() {
                    window.obj().close();
                }
            }));
        }
    }

    impl WidgetImpl for TeamDialog {}

    impl WindowImpl for TeamDialog {}
}

glib::wrapper! {
    pub struct TeamDialog(ObjectSubclass<imp::TeamDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl TeamDialog {
    pub fn new() -> Self {
        glib::Object::new::<TeamDialog>()
    }
}

impl Default for TeamDialog {
    fn default() -> Self {
        Self::new()
    }
}

fn number_from(entry: &Entry) -> i32 {
    entry.text().as_str().parse::<i32>().unwrap_or(0)
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

fn validate_not_empty(entry: &Entry, name: &str) -> bool {
    if entry.text().as_str().is_empty() {
        show_error_dialog(&entry.root(), format!("{} is required", name).as_str());
        false
    } else {
        true
    }
}