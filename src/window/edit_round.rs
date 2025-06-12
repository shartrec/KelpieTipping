use crate::model::round::{Playday, Playdays, Round};
use crate::model::team::Team;
use crate::window::util::build_column_factory;
use adw::gio::ListModel;
use adw::glib::{clone, Object};
use adw::prelude::{Cast, CastNone, IsA};
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::prelude::{ActionableExt, ListItemExt, WidgetExt};
use gtk::{gio, glib, Button, DropDown, Expression, Label, ListItem, SignalListItemFactory};
use std::cell::RefCell;
use crate::event;
use crate::event::Event;
use crate::model::game;
use crate::util::db;

mod imp {
    use crate::event;
    use crate::event::Event;
    use crate::model::game::{Game, Games};
    use crate::model::round::{Playday, Playdays, Round};
    use crate::model::team::{Team, Teams};
    use crate::model::{game, round, team};
    use crate::util::date::to_ymd;
    use crate::util::{db, game_allocator};
    use crate::window::edit_round::{build_column_factory_playday, build_column_factory_team, delete_game};
    use crate::window::util::{build_del_column_factory, connect_escape, show_error_dialog};
    use adw::glib::{closure_local, GString, TimeZone};
    use adw::prelude::{ButtonExt, Cast, CastNone, EditableExt, GtkWindowExt, ListModelExt, ObjectExt, WidgetExt};
    use adw::subclass::prelude::{CompositeTemplate, ObjectImpl, ObjectImplExt, ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt, WidgetClassExt, WindowImpl};
    use chrono::{Datelike, Local, NaiveDate};
    use gtk::glib::clone;
    use gtk::glib::subclass::InitializingObject;
    use gtk::prelude::{ActionableExtManual, EntryExt, PopoverExt, SelectionModelExt};
    use gtk::subclass::prelude::BoxImpl;
    use gtk::subclass::widget::{CompositeTemplateInitializingExt, WidgetImpl};
    use gtk::{glib, AlertDialog, Button, Calendar, ColumnView, ColumnViewColumn, CompositeTemplate, DropDown, Entry, NoSelection, Popover, SignalListItemFactory, SpinButton, TemplateChild};
    use log::{info, warn};
    use std::cell::RefCell;
    use std::ops::{Add, Deref, Sub};
    use std::str::FromStr;
    use std::sync::Arc;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/shartrec/kelpie_tipping/round_panel.ui")]
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
        pub btn_add_game: TemplateChild<Button>,
        #[template_child]
        pub btn_save: TemplateChild<Button>,
        #[template_child]
        pub btn_revert: TemplateChild<Button>,

        round_id: RefCell<Option<i32>>,
        team_model: RefCell<Option<Teams>>,
        playday_model: RefCell<Option<Playdays>>,
    }

    impl RoundDialog {
        pub fn initialise(&self) {
            // Set the default values for the entries
            let teams = Teams::new();
            self.team_model.replace(Some(teams));

            if let Some(rx) = event::manager().register_listener() {
                glib::spawn_future_local(clone!(#[weak(rename_to = view)] self, async move {
                    while let Ok(ev) = rx.recv().await {
                        if let Event::GamesChanged = ev {
                            let games = Games::for_round(view.round_id.borrow().unwrap_or(-1));
                            let selection_model = NoSelection::new(Some(games));
                            view.game_list.set_model(Some(&selection_model));
                            view.game_list.queue_draw();
                        }
                    }
                }));
            }

        }

        pub fn set_round(&self, round: Option<Round>) {
            match round {
                Some(round) => {
                    self.round_id.replace(Some(round.id()));
                    self.round_number.set_text(&round.number().to_string());
                    self.start_date.set_text(&to_ymd(&round.start_date()));
                    self.end_date.set_text(&to_ymd(&round.end_date()));

                    self.playday_model.replace(Some(Playdays::new(round.start_date(), round.end_date())));

                    let games = Games::for_round(self.round_id.borrow().unwrap_or(-1));
                    let selection_model = NoSelection::new(Some(games));
                    self.game_list.set_model(Some(&selection_model));
                    self.game_list.queue_draw();
                }
                None => {
                    // Get the last defined round and set it as the current round to one week later
                    let lr = async_std::task::block_on( async move {
                        let pool = db::manager().pool();
                        round::get_last_round(pool).await
                    });
                    let (start, end) =
                        if let Ok(Some(last_round)) = lr {
                            self.round_number.set_text((&last_round.number() + 1).to_string().as_str());
                            let start = last_round.start_date().add(chrono::Duration::days(7));
                            let end = last_round.end_date().add(chrono::Duration::days(7));
                            (start, end)
                        } else {
                            // No rounds defined, set default values
                            self.round_number.set_text("");
                            let start = Local::now().date_naive();
                            let end = start.add(chrono::Duration::days(1));
                            (start, end)
                        };
                    self.round_id.replace(None);
                    self.start_date.set_text(&to_ymd(&start));
                    self.end_date.set_text(&to_ymd(&end));
                    self.playday_model.replace(Some(Playdays::new(start, end)));

                    let binding = Teams::new();
                    let teams  = binding.imp().teams.read().unwrap();
                    let mut game_list = game_allocator::allocate_games(
                        -1, // No round ID yet
                        &teams,
                        start,
                        end,
                    );
                    let games = Games::new(start, end, &mut game_list);
                    let selection_model = NoSelection::new(Some(games));
                    self.game_list.set_model(Some(&selection_model));
                    self.game_list.queue_draw();
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
                drop.connect_selected_notify(clone!(#[weak] window, #[weak] game, move |drop| {
                    if let Some(selected) = drop.selected_item() {
                        if let Some(day) = selected.downcast_ref::<Playday>() {
                            game.imp().set_game_date(day.imp().date());
                            println!("Setting game date to: {:?}", day.imp().date());
                        }
                    }
                }));
            }))));

            self.col_home.set_factory(Some(&self.get_team_factory(
                self.team_model.clone(),
                |game| game.home_team_id(),
                |game, id| game.imp().set_home_team_id(id),
            )));

            self.col_away.set_factory(Some(&self.get_team_factory(
                self.team_model.clone(),
                |game| game.away_team_id(),
                |game, id| game.imp().set_away_team_id(id),
            )));

            // Create a factory for the delete button column
            let f = build_del_column_factory(
                |button: &Button, game: &Game| button.set_action_target(Some(game.id())),
                delete_game);
            self.col_delete.set_factory(Some(&f));

        }

        fn get_team_factory<FGet: Fn(&Game) -> i32 + 'static, FSet: Fn(&Game, i32) + 'static>(
            &self,
            team_model: RefCell<Option<Teams>>,
            get_team_id: FGet,
            set_team_id: FSet,
        ) -> SignalListItemFactory {
            let set_team_id = Arc::new(set_team_id);
            let factory = build_column_factory_team(
                team_model,
                move |drop: DropDown, game: &Game| {
                    let set_team_id = Arc::clone(&set_team_id);
                    let model = drop.model().expect("Dropdown has no model");
                    let n_items = model.n_items();

                    for i in 0..n_items {
                        if let Some(t) = model.item(i) {
                            let team = t.downcast::<Team>().expect("Item is not a Team");
                            if team.id() == get_team_id(game) {
                                drop.set_selected(i);
                                break;
                            }
                        }
                    }
                    drop.connect_selected_notify(clone!(#[weak] game, move |drop| {
                        if let Some(selected) = drop.selected_item() {
                            if let Some(team) = selected.downcast_ref::<Team>() {
                                set_team_id(&game, team.id());
                                println!("Setting team to: {:?}", team.name());
                            }
                        }
                    }));
                },
            );
            factory
        }

        fn save_round(&self) -> bool {

            let mut new_round_id = -1;
            let pool = db::manager().pool();
            if self.round_id.borrow().is_some() {
                // Update the round
                let id = self.round_id.borrow().unwrap();
                let number: i32 = i32::from_str(self.round_number.text().to_string().as_str()).unwrap();
                let start_date = self.start_date.text().to_string().clone();
                let end_date = self.end_date.text().to_string().clone();
                async_std::task::block_on(async move {
                    let _ = round::update(pool, id, number, start_date.parse().unwrap(), end_date.parse().unwrap()).await;
                });
                new_round_id = id;
            } else {
                // Create a new round
                let number = i32::from_str(self.round_number.text().to_string().as_str()).unwrap();
                let start_date = self.start_date.text().to_string().clone();
                let end_date = self.end_date.text().to_string().clone();
                new_round_id = async_std::task::block_on(async move {
                    if let Ok(round_id) = round::insert(pool, number, start_date.parse().unwrap(), end_date.parse().unwrap()).await {
                        round_id
                    } else {
                        -1
                    }
                });
            }

            if let Some(m) = self.game_list.model().and_downcast_ref::<NoSelection>() {
                if let Some(games) = m.model().and_downcast_ref::<Games>() {
                    let gg = games.imp();
                    for mut game in gg.games.write().unwrap().iter() {
                        let round = game.round_id();
                        let home_team = game.home_team_id();
                        let away_team = game.away_team_id();
                        let date = game.game_date();

                        let id = game.id();
                        if id > 0 {
                            info!("Updating game {}", id);
                            async_std::task::block_on(async move {
                                let _rows = game::update(pool, id, round, home_team, away_team, date, None, None).await;
                            });
                        } else {
                            let id = async_std::task::block_on(async move {
                                if let Ok(id) = game::insert(pool, new_round_id, home_team, away_team, date, None, None).await {
                                    id
                                } else {
                                    warn!("Failed to insert new game for round {}", new_round_id);
                                    -1
                                }
                            });
                            game.imp().set_id(id);
                        }
                    }
                }
            }

            event::manager().notify_listeners(Event::RoundsChanged{round_id: new_round_id});
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
                return false;
            }

            // Validate the games are set up correctly
            // Check each team is used only once per round
            if let Some(model) = self.game_list.model().and_downcast_ref::<NoSelection>() {
                if let Some(games) = model.model().and_downcast_ref::<Games>() {
                    let game_list = games.imp().games.read().unwrap();
                    let mut game_teams = std::collections::HashSet::new();

                    for game in game_list.iter() {
                        for id in [game.home_team_id(), game.away_team_id()] {
                            if !game_teams.insert(id) {
                                let pool = db::manager().pool();
                                let t = async_std::task::block_on(async { team::get(pool, id).await }).ok().flatten();
                                let team_name = if let Some(t) = t {
                                    t.name()
                                } else {
                                    game.home_team_id().to_string()
                                };
                                show_error_dialog(&self.game_list.root(), format!("Team {} is used more than once in this round", team_name).as_str());
                                return false;
                            }
                        }
                        // also check the game date is between the round start and end dates
                        if game.game_date() < start_date.unwrap() || game.game_date() > end_date.unwrap() {
                            let message = format!("Game date {} is not between the round start date {} and end date {}",
                                                  game.game_date(), start_date.unwrap(), end_date.unwrap());
                            show_error_dialog(&self.game_list.root(), message.as_str());
                            return false;
                        }
                    }
                }
            }

            true
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
        type ParentType = gtk::Box;

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

            self.btn_add_game.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                if let Some(model) = window.game_list.model().and_downcast_ref::<NoSelection>() {
                    if let Some(games) = model.model().and_downcast_ref::<Games>() {
                        let mut game_list = games.imp().games.write().unwrap();
                        let size = game_list.len() as u32;
                        let binding = Teams::new();
                        let teams  = binding.imp().teams.read().unwrap();

                        if let Some(playdays) = window.playday_model.borrow().as_ref() {
                            game_allocator::add_extra_game(
                                window.round_id.borrow().unwrap_or(-1),
                                &teams,
                                &playdays.imp().start_date.borrow(),
                                &playdays.imp().end_date.borrow(),
                                &mut game_list
                            );
                            // need to relase the write lock here
                            drop(game_list);
                            model.items_changed(0, size, size + 1);
                        }
                    }
                }
            }));

            self.btn_save.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                if window.validate() {
                    window.save_round();
                }
            }));

            self.btn_revert.connect_clicked(clone!(#[weak(rename_to = window)] self, move |_button| {
                let round_id = window.round_id.borrow().clone();
                let read = async_std::task::block_on(
                    async move {
                        let pool = db::manager().pool();
                        if let Some(id) = round_id {
                            round::get(pool, id).await
                        } else {
                            warn!("No round ID set, cannot revert");
                            Ok(None)
                        }
                    });
                if let Ok(round) = read {
                    glib::MainContext::default().spawn_local( async move {
                        window.set_round(round);
                    });
                }
            }));
        }

        fn dispose(&self) {
            info!("Disposing RoundDialog");
        }
    }

    impl WidgetImpl for RoundDialog {}

    impl BoxImpl for RoundDialog {}
}

glib::wrapper! {
    pub struct RoundDialog(ObjectSubclass<imp::RoundDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RoundDialog {
    pub fn new(parent: &gtk::Box, round: Option<Round>) -> Self {
        let obj = Object::new::<RoundDialog>();
        obj.set_parent(parent);
        obj.imp().set_round(round);
        obj
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

fn delete_game(button: &Button) {
    if let Some(value) = button.action_target_value() {
        if let Some(id) = value.get::<i32>() {
            let pool = db::manager().pool();
            glib::spawn_future_local(clone!(async move {
                let _ = game::delete(pool, id).await;
                event::manager().notify_listeners(Event::GamesChanged);
            }));
        }
    }
}
