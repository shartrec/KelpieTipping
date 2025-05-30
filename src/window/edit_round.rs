use crate::model::round::{Playday, Playdays, Round};
use crate::model::team::Team;
use crate::window::util::build_column_factory;
use adw::gio::ListModel;
use adw::glib::Object;
use adw::prelude::{Cast, CastNone, IsA};
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::prelude::ListItemExt;
use gtk::{gio, glib, DropDown, Expression, Label, ListItem, SignalListItemFactory};
use std::cell::RefCell;

mod imp {
    use crate::event;
    use crate::event::Event;
    use crate::model::game::{Game, Games};
    use crate::model::round::{Playdays, Round};
    use crate::model::team::{Team, Teams};
    use crate::model::{game, round};
    use crate::util::db;
    use crate::window::edit_round::{build_column_factory_playday, build_column_factory_team};
    use crate::window::util::connect_escape;
    use adw::glib::{GString, TimeZone};
    use adw::prelude::{ButtonExt, Cast, CastNone, EditableExt, GtkWindowExt, ListModelExt, WidgetExt};
    use adw::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt, WidgetClassExt, WindowImpl};
    use chrono::{Datelike, Local, NaiveDate};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::{EntryExt, PopoverExt};
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};
    use gtk::{glib, AlertDialog, Button, Calendar, ColumnView, ColumnViewColumn, CompositeTemplate, DropDown, Entry, Popover, SingleSelection, SpinButton,TemplateChild};
    use log::info;
    use std::cell::RefCell;
    use std::ops::{Add, Deref, Sub};
    use std::str::FromStr;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_tipping/round_dialog.ui")]
    pub struct RoundDialog {
        #[template_child]
        pub round_view: TemplateChild<gtk::Box>,
        #[template_child]
        pub round_number: TemplateChild<SpinButton>,
        #[template_child]
        pub start_date: TemplateChild<Entry>,
        #[template_child]
        pub start_date_popover: TemplateChild<Popover>,
        #[template_child]
        pub start_date_calendar: TemplateChild<Calendar>,
        #[template_child]
        pub end_date: TemplateChild<Entry>,
        #[template_child]
        pub end_date_popover: TemplateChild<Popover>,
        #[template_child]
        pub end_date_calendar: TemplateChild<Calendar>,

        #[template_child]
        pub game_list: TemplateChild<ColumnView>,
        #[template_child]
        pub col_date: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_home: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_home_score: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_away: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_away_score: TemplateChild<ColumnViewColumn>,
        #[template_child]
        pub col_delete: TemplateChild<ColumnViewColumn>,

        #[template_child]
        pub btn_ok: TemplateChild<Button>,
        #[template_child]
        pub btn_cancel: TemplateChild<Button>,

        round_id: RefCell<Option<i32>>,
        team_model: RefCell<Option<Teams>>,
        playday_model: RefCell<Option<Playdays>>,
    }

    impl RoundDialog {
        pub fn initialise(&self) {
            // Set the default values for the entries
            let teams = Teams::new();
            teams.imp().add_team(Team::new(None, "(none)".to_string(), "(none)".to_string()));
            self.team_model.replace(Some(teams));
        }

        fn refresh(&self) {
            let games = Games::new(self.round_id.borrow().unwrap_or(-1));
            let selection_model = SingleSelection::new(Some(games));
            self.game_list.set_model(Some(&selection_model));
            self.game_list.queue_draw();
        }


        pub fn set_round(&self, round: Option<Round>) {
            match round {
                Some(round) => {
                    self.round_id.replace(Some(round.id()));
                    self.round_number.set_text(&round.number().to_string());
                    self.start_date.set_text(&round.start_date().to_string());
                    self.end_date.set_text(&round.end_date().to_string());

                    self.playday_model.replace(Some(Playdays::new(round.start_date(), round.end_date())));
                }
                None => {
                    self.round_id.replace(None);
                    self.round_number.set_text("");
                    let start = Local::now().date_naive();
                    let end = start.add(chrono::Duration::days(1));
                    self.start_date.set_text(start.to_string().as_str());
                    self.end_date.set_text(end.to_string().as_str());

                    self.playday_model.replace(Some(Playdays::new(start, end)));
                }
            }


            self.col_date.set_factory(Some(&build_column_factory_playday(self.playday_model.clone(), clone!(#[weak(rename_to = window)] self, move |drop: DropDown, game: &Game | {
                if let Ok(start_date) = NaiveDate::from_str(window.start_date.text().as_str()) {
                    let day_offset = game.game_date().sub(start_date);
                    if day_offset.num_days() >= 0 {
                        drop.set_selected(day_offset.num_days() as u32);
                    } else {
                        drop.set_selected(0);
                    }
                } else {
                    drop.set_selected(0);
                }
            }))));

            self.col_home.set_factory(Some(&build_column_factory_team(self.team_model.clone(), |drop: DropDown, game: &Game| {
                let model = drop.model().expect("Dropdown has no model");
                let n_items = model.n_items();

                for i in 0..n_items {
                    if let Some(t) = model.item(i) {
                        let team = t.downcast::<Team>().expect("Item is not a Team");
                        if let Some(id) = team.id() {
                            if id == game.home_team_id() {
                                drop.set_selected(i);
                                break;
                            }
                        }
                    }
                }
            })));

            self.col_away.set_factory(Some(&build_column_factory_team(self.team_model.clone(), |drop: DropDown, game: &Game| {
                let model = drop.model().expect("Dropdown has no model");
                let n_items = model.n_items();

                for i in 0..n_items {
                    if let Some(t) = model.item(i) {
                        let t = t.downcast::<Team>().expect("Item is not a Team");
                        if let Some(id) = t.id() {
                            if id == game.away_team_id() {
                                drop.set_selected(i);
                                break;
                            }
                        }
                    }
                }
            })));

            self.refresh();
        }

        fn save_round(&self) -> bool {
            async_std::task::block_on(clone!(#[weak (rename_to = window)] self, async move {

                let pool = db::manager().pool();
                if window.round_id.borrow().is_some() {
                    // Update the round
                    let id = window.round_id.borrow().unwrap();
                    let number: i32 = i32::from_str(window.round_number.text().to_string().as_str()).unwrap();
                    let start_date = window.start_date.text().to_string().clone();
                    let end_date = window.end_date.text().to_string().clone();
                        info!("Updating round {}", id);
                        let _ = round::update(pool, id, number, start_date.parse().unwrap(), end_date.parse().unwrap()).await;
                } else {
                    // Create a new round
                    let number = i32::from_str(window.round_number.text().to_string().as_str()).unwrap();
                    let start_date = window.start_date.text().to_string().clone();
                    let end_date = window.end_date.text().to_string().clone();
                        let _ = round::insert(pool, number, start_date.parse().unwrap(), end_date.parse().unwrap()).await;
                }


                if let Some(m) = window.game_list.model().and_downcast_ref::<SingleSelection>() {
                    if let Some(games) = m.model().and_downcast_ref::<Games>() {
                        let gg = games.imp();
                        for game in gg.games.read().unwrap().iter() {
                            let round = game.round_id();
                            let home_team = game.home_team_id();
                            let away_team = game.away_team_id();
                            let date = game.game_date();
                            let _ = game::insert(pool, round, home_team, away_team, date, None, None).await;
                        }
                    }
                }
            }));

            event::manager().notify_listeners(Event::RoundsChanged);
            true
        }

        fn validate(&self) -> bool {
            if !self.validate_not_empty(&self.round_number.text(), "Round Number") {
                return false;
            }
            let start_date = self.validate_date(&self.start_date.text(), "Start Date");
            let end_date = self.validate_date(&self.end_date.text(), "End Date");

            if start_date.is_none() || end_date.is_none() {
                return false;
            }

            if start_date > end_date {
                let buttons = vec!["Ok".to_string()];
                let message = "Start Date is after End Date";
                let alert = AlertDialog::builder()
                    .message(message)
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else {
                true
            }
        }

        fn validate_not_empty(&self, field: &GString, message: &str) -> bool {
            if field.is_empty() {
                let buttons = vec!["Ok".to_string()];
                let message = format!("Please enter a valid {}", message);
                let alert = AlertDialog::builder()
                    .message(message)
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                false
            } else {
                true
            }
        }

        fn validate_date(&self, field: &GString, message: &str) -> Option<NaiveDate> {
            let buttons = vec!["Ok".to_string()];
            if field.is_empty() {
                let message = format!("Please enter a valid {}", message);
                let alert = AlertDialog::builder()
                    .message(message)
                    .buttons(buttons)
                    .build();
                alert.show(Some(&self.obj().clone()));
                None
            } else {
                let message = format!("{} is not a valid date", message);
                let date_str = field.to_string();
                match date_str.parse::<NaiveDate>() {
                    Ok(date) => Some(date),
                    Err(_) => {
                        let buttons = vec!["Ok".to_string()];
                        let alert = AlertDialog::builder()
                            .message(message)
                            .buttons(buttons)
                            .build();
                        alert.show(Some(&self.obj().clone()));
                        None
                    }
                }
            }
        }

        fn setup_date_selector(&self, entry: &Entry, popover: &Popover, calendar: &Calendar) {
            // Show the popover when the button is clicked
            entry.connect_icon_press(clone!(#[weak] entry, #[weak] popover, #[weak] calendar, move |_, icon| {

                // set the Calendar to the date in the entry, if it is a valid date
                if icon == gtk::EntryIconPosition::Secondary {
                    let date_str = entry.text().to_string().clone();
                    if let Ok(date) = date_str.parse::<NaiveDate>() {
                        if let Ok(date) = glib::DateTime::new(&TimeZone::local(),
                            date.year(),
                            date.month() as i32,
                            date.day() as i32,
                            0i32, 0i32, 0f64)
                        {
                            calendar.select_day(&date);
                        }
                    }
                    popover.popup();
                }
            }));

            // Update the entry when a date is selected in the calendar
            calendar.connect_day_selected(glib::clone!(#[weak] entry, #[weak] popover, move |calendar| {
            let date = calendar.date();
            let formatted_date = format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day_of_month());
            entry.set_text(&formatted_date);

            popover.popdown(); // Close the popover after selecting a date
            }));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RoundDialog {
        const NAME: &'static str = "RoundDialog";
        type Type = super::RoundDialog;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RoundDialog {
        fn constructed(&self) {
            self.parent_constructed();
            self.initialise();

            self.setup_date_selector(
                &self.start_date,
                &self.start_date_popover,
                &self.start_date_calendar,
            );

            self.setup_date_selector(
                &self.end_date,
                &self.end_date_popover,
                &self.end_date_calendar,
            );

            self.start_date.connect_changed(clone!(#[weak(rename_to = window)] self, move |entry| {
                if let Ok(date) = NaiveDate::from_str(entry.text().as_str()) {
                    if let Some(playdays) = window.playday_model.borrow().as_ref() {
                        playdays.set_start_date(date);
                    }
                }
            }));

            self.end_date.connect_changed(clone!(#[weak(rename_to = window)] self, move |entry| {
                if let Ok(date) = NaiveDate::from_str(entry.text().as_str()) {
                    if let Some(playdays) = window.playday_model.borrow().as_ref() {
                        playdays.set_end_date(date);
                    }
                }
            }));

            self.btn_cancel.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
               window.obj().close();
            }));

            self.btn_ok.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                if window.validate() && window.save_round() {
                    window.obj().close();
                }
            }));


            connect_escape(self.round_view.deref(), &self.btn_cancel.deref());
        }
    }

    impl WidgetImpl for RoundDialog {}

    impl WindowImpl for RoundDialog {}
}

glib::wrapper! {
    pub struct RoundDialog(ObjectSubclass<imp::RoundDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RoundDialog {
    pub fn new(round: Option<Round>) -> Self {
        let obj = Object::new::<RoundDialog>();
        obj.imp().set_round(round);
        obj
    }
}

impl Default for RoundDialog {
    fn default() -> Self {
        Self::new(None)
    }
}

pub(super) fn build_column_factory_playday<F: Fn(DropDown, &T) + 'static, T: IsA<Object>>(model: RefCell<Option<Playdays>>, f: F) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    connect_drop_playday(model, &factory);
    connect_bind(f, &factory);
    factory
}

pub(super) fn build_column_factory_team<M: IsA<ListModel>, F: Fn(DropDown, &T) + 'static, T: IsA<Object>>(model: RefCell<Option<M>>, f: F) -> SignalListItemFactory {
    let factory = SignalListItemFactory::new();
    connect_drop_team(model, &factory);
    connect_bind(f, &factory);
    factory
}

fn connect_bind<F: Fn(DropDown, &T) + 'static, T: IsA<Object>>(f: F, factory: &SignalListItemFactory) {
    factory.connect_bind(move |_, list_item| {
        // Get `StringObject` from `ListItem`
        let obj = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .item()
            .and_downcast::<T>()
            .expect("The item has to be an <T>.");

        // Get `Label` from `ListItem`
        let drop = list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .child()
            .and_downcast::<DropDown>()
            .expect("The child has to be a `DropDown`.");

        // Set "label" to "number"
        f(drop, &obj);
    });
}
fn connect_drop_playday(model: RefCell<Option<Playdays>>, factory: &SignalListItemFactory) {
    factory.connect_setup(move |_, list_item| {
        let drop = DropDown::new(None::<ListModel>, None::<Expression>);
        drop.set_model(model.borrow().as_ref());
        drop.set_factory(Some(&build_column_factory(|label: Label, playday: &Playday| {
            label.set_label(playday.imp().to_string().as_str());
            label.set_xalign(0.0);
        })));

        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&drop));
    });
}

//noinspection DuplicatedCode
fn connect_drop_team<M: IsA<ListModel>>(model: RefCell<Option<M>>, factory: &SignalListItemFactory) {
    factory.connect_setup(move |_, list_item| {
        let drop = DropDown::new(None::<ListModel>, None::<Expression>);
        drop.set_model(model.borrow().as_ref());
        drop.set_factory(Some(&build_column_factory(|label: Label, team: &Team| {
            label.set_label(team.name().as_str());
            label.set_xalign(0.0);
        })));

        list_item
            .downcast_ref::<ListItem>()
            .expect("Needs to be ListItem")
            .set_child(Some(&drop));
    });
}