use gtk::{gio, glib};

mod imp {
    use crate::event;
    use crate::event::Event;
    use crate::model::tipper;
    use crate::model::tipper::Tipper;
    use crate::util::db;
    use crate::window::util::{connect_escape, validate_not_empty};
    use adw::prelude::{ButtonExt, GtkWindowExt};
    use adw::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, WidgetClassExt, WindowImpl};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};
    use gtk::{glib, AlertDialog, Button, CompositeTemplate, Entry, TemplateChild};
    use log::info;
    use std::cell::RefCell;
    use std::ops::Deref;
    use gtk::prelude::EditableExt;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_tipping/tipper_dialog.ui")]
    pub struct TipperDialog {
        #[template_child]
        pub tipper_view: TemplateChild<gtk::Box>,
        #[template_child]
        pub tipper_name: TemplateChild<Entry>,
        #[template_child]
        pub tipper_email: TemplateChild<Entry>,
        #[template_child]
        pub btn_ok: TemplateChild<Button>,
        #[template_child]
        pub btn_cancel: TemplateChild<Button>,

        tipper_id: RefCell<Option<i32>>,
    }

    impl TipperDialog {
        pub fn set_tipper(&self, tipper: Option<Tipper>) {
            match tipper {
                Some(tipper) => {
                    self.tipper_id.replace(Some(tipper.id()));
                    self.tipper_name.set_text(&tipper.name());
                    self.tipper_email.set_text(&tipper.email());
                }
                None => {
                    self.tipper_id.replace(None);
                    self.tipper_name.set_text("");
                    self.tipper_email.set_text("");
                }
            }
        }

        fn validate(&self) -> bool {
            validate_not_empty(&self.tipper_name, "Name") &&
                validate_not_empty(&self.tipper_email, "Email")
        }

        fn save_tipper(&self) -> bool {
            // Check we have a name
            if self.tipper_name.text().is_empty() {
                let buttons = vec!["Ok".to_string()];
                let alert = AlertDialog::builder()
                    .message("Please enter a name")
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else if self.tipper_email.text().is_empty() {
                let buttons = vec!["Ok".to_string()];
                let alert = AlertDialog::builder()
                    .message("Please enter a nick name")
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else {
                let pool = db::manager().pool();
                if self.tipper_id.borrow().is_some() {
                    // Update the tipper
                    let id = self.tipper_id.borrow().unwrap();
                    let name = self.tipper_name.text().to_string().clone();
                    let email = self.tipper_email.text().to_string().clone();
                    glib::spawn_future_local(clone!( async move {
                        info!("Updating tipper {}", id);
                        let _ = tipper::update(pool, id, name, email).await;
                        event::manager().notify_listeners(Event::TippersChanged);
                    }));
                } else {
                    // Create a new tipper
                    let name = self.tipper_name.text().to_string().clone();
                    let email = self.tipper_email.text().to_string().clone();
                    glib::spawn_future_local(clone!( async move {
                        let _ = tipper::insert(pool, name, email).await;
                        event::manager().notify_listeners(Event::TippersChanged);
                    }));
                }
                true
            }
        }

    }

    #[glib::object_subclass]
    impl ObjectSubclass for TipperDialog {
        const NAME: &'static str = "TipperDialog";
        type Type = super::TipperDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TipperDialog {
        fn constructed(&self) {
            self.parent_constructed();

            self.btn_cancel.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
               window.obj().close();
            }));

            self.btn_ok.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                if window.validate() && window.save_tipper() {
                    window.obj().close();
                }
            }));

            connect_escape(self.tipper_view.deref(), self.btn_cancel.deref());
        }
    }

    impl WidgetImpl for TipperDialog {}

    impl WindowImpl for TipperDialog {}
}

glib::wrapper! {
    pub struct TipperDialog(ObjectSubclass<imp::TipperDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl TipperDialog {
    pub fn new() -> Self {
        glib::Object::new::<TipperDialog>()
    }
}

impl Default for TipperDialog {
    fn default() -> Self {
        Self::new()
    }
}