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
use adw::subclass::prelude::ObjectSubclassIsExt;
use adw::{gio, glib};
use chrono::NaiveDate;
use log::error;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;

// To use the Round in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Round(ObjectSubclass<imp::Round>);
}

impl Round {
    pub fn new(round_id: i32, number: i32, start_date: NaiveDate, end_date: NaiveDate) -> Round
    {
        let obj: Round = glib::Object::new();
        obj.imp().set_id(round_id);
        obj.imp().set_number(number);
        obj.imp().set_start_date(start_date);
        obj.imp().set_end_date(end_date);
        obj
    }

    pub fn id(&self) -> i32 {
        self.imp().round_id.borrow().clone()
    }
    pub fn number(&self) -> i32 {
        self.imp().round_number.borrow().clone()
    }
    pub fn start_date(&self) -> NaiveDate {
        self.imp().start_date.borrow().clone()
    }
    pub fn end_date(&self) -> NaiveDate {
        self.imp().end_date.borrow().clone()
    }

    pub fn set_id(&self, id: i32) {
        self.imp().set_id(id);
    }
    pub fn set_number(&self, number: i32) {
        self.imp().set_number(number);
    }
    pub fn set_start_date(&self, start_date: NaiveDate) {
        self.imp().set_start_date(start_date);
    }
    pub fn set_end_date(&self, end_date: NaiveDate) {
        self.imp().set_end_date(end_date);
    }
}

glib::wrapper! {
    pub struct Rounds(ObjectSubclass<imp::Rounds>) @implements gio::ListModel;
}

impl Rounds {
    pub fn new() -> Rounds
    {
        glib::Object::new()
    }
}

glib::wrapper! {
    pub struct Playday(ObjectSubclass<imp::Playday>);
}

impl Playday {
    pub fn new(date: NaiveDate) -> Playday
    {
        let obj: Playday = glib::Object::new();
        obj.imp().set_date(date);
        obj
    }
}

glib::wrapper! {
    pub struct Playdays(ObjectSubclass<imp::Playdays>) @implements gio::ListModel;
}

impl Playdays {
    pub fn new(start_date: NaiveDate, end_date: NaiveDate) -> Playdays {
        let obj: Playdays = glib::Object::new();
        obj.imp().init(start_date, end_date);
        obj
    }
    pub fn set_start_date(&self, start_date: NaiveDate) {
        self.imp().set_start_date(start_date);
    }
    pub fn set_end_date(&self, end_date: NaiveDate) {
        self.imp().set_end_date(end_date);
    }
}

mod imp {
    use crate::model::round::get_all;
    use crate::util::db;
    use adw::glib::{clone, MainContext, Object};
    use adw::prelude::{ListModelExt, StaticType};
    use adw::subclass::prelude::ObjectSubclassExt;
    use adw::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};
    use adw::{gio, glib};
    use chrono::{Local, NaiveDate};
    use log::{error, info};
    use std::cell::RefCell;
    use std::ops::{Deref, Sub};
    use std::sync::{Arc, RwLock};
    use crate::event;
    use crate::event::Event;

    #[derive(Debug)]
    #[derive(Default)]
    pub struct Round {
        pub round_id: RefCell<i32>,
        pub round_number: RefCell<i32>,
        pub start_date: RefCell<NaiveDate>,
        pub end_date: RefCell<NaiveDate>,
    }

    impl Round {
        pub fn set_id(&self, id: i32) {
            self.round_id.replace(id);
        }
        pub fn set_number(&self, number: i32) {
            self.round_number.replace(number);
        }
        pub fn set_start_date(&self, start_date: NaiveDate) {
            self.start_date.replace(start_date);
        }
        pub fn set_end_date(&self, end_date: NaiveDate) {
            self.end_date.replace(end_date);
        }
    }
    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Round {
        const NAME: &'static str = "Round";
        type Type = super::Round;
    }

    impl ObjectImpl for Round {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    #[derive(Default)]
    pub struct Rounds {
        pub rounds: Arc<RwLock<Vec<crate::model::round::Round>>>,
    }

    impl Rounds {
        pub fn round_at(&self, position: u32) -> Option<crate::model::round::Round> {
            let map = self
                .rounds
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            map.iter().nth(position as usize).as_deref().map(|t| t.clone())
        }
    }
    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Rounds {
        const NAME: &'static str = "Rounds";
        type Type = super::Rounds;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for Rounds {
        fn constructed(&self) {
            self.parent_constructed();

            let pool = db::manager().pool();
            let rounds = async_std::task::block_on( async move {
                get_all(pool).await
            });
            match rounds {
                Ok(round_list) => {
                    let mut binding = self.rounds.write().expect("Can't get lock on rounds cache");
                    for r in round_list.into_iter() {
                        binding.push(r);
                    }
                }
                Err(err) => {
                    error!("Error getting all rounds: {}", err);
                }
            }
        }
    }

    impl ListModelImpl for Rounds {
        fn item_type(&self) -> glib::Type {
            crate::model::round::Round::static_type()
        }

        fn n_items(&self) -> u32 {
            let map = self
                .rounds
                .read()
                .expect("Unable to get a lock on the rounds");
            map.len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.round_at(position).map(|round| {
                Object::from(round.clone())
            })
        }
    }
    #[derive(Debug)]
    #[derive(Default)]
    pub struct Playday {
        pub date: RefCell<NaiveDate>,
    }

    impl Playday {
        pub fn set_date(&self, date: NaiveDate) {
            self.date.replace(date);
        }

        pub fn date(&self) -> NaiveDate {
            self.date.borrow().clone()
        }

        pub fn to_string(&self) -> String {
            self.date.borrow().format("%a, %-d %b").to_string()
        }
    }
    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Playday {
        const NAME: &'static str = "Playday";
        type Type = super::Playday;
    }

    impl ObjectImpl for Playday {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    pub struct Playdays {
        pub(crate) start_date: RefCell<NaiveDate>,
        pub(crate) end_date: RefCell<NaiveDate>,
    }


    impl Playdays {
        pub fn init(&self, start_date: NaiveDate, end_date: NaiveDate) {
            self.start_date.replace(start_date);
            self.end_date.replace(end_date);
        }

        pub fn set_start_date(&self, start_date: NaiveDate) {

            let old_size = self.n_items() as i32;
            let old_date = self.start_date.replace(start_date);
            if start_date > *self.end_date.borrow().deref() {
                self.end_date.replace(start_date);
            };
            if old_date != start_date {
                let new_size = self.n_items() as i32;

                let days_changed = new_size - old_size;
                let (added, removed) = if days_changed >= 0 {
                    (days_changed, 0)
                } else {
                    (0, 0 - days_changed)
                };

                println!("Playdays changed: old_size={}, new_size={}, added={}, removed={}", old_size, new_size, added, removed);
                self.obj().items_changed(0, removed as u32, added as u32);
                event::manager().notify_listeners(Event::PlaydaysChanged);
            }
        }

        pub fn set_end_date(&self, end_date: NaiveDate) {
            let old_size = self.n_items() as i32;
            let old_date = self.end_date.replace(end_date);
            if end_date < *self.start_date.borrow().deref() {
                self.start_date.replace(end_date);
            };

            if old_date != end_date {
                let new_size = self.n_items() as i32;

                let days_changed = new_size - old_size;
                let (added, removed) = if days_changed >= 0 {
                    (days_changed, 0)
                } else {
                    (0, 0 - days_changed)
                };
                println!("Playdays changed: old_size={}, new_size={}, added={}, removed={}", old_size, new_size, added, removed);

                self.obj().items_changed((old_size - 1 - removed) as u32, removed as u32, added as u32);
                event::manager().notify_listeners(Event::PlaydaysChanged);
            }
        }

        pub fn start_date(&self) -> NaiveDate {
            self.start_date.borrow().clone()
        }

        pub fn end_date(&self) -> NaiveDate {
            self.end_date.borrow().clone()
        }
    }

    impl Default for Playdays {
        fn default() -> Self {

            let s = Local::now().date_naive();
            let e = Local::now().date_naive();
            Playdays{ start_date: RefCell::new(s), end_date:RefCell::new(e) }
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Playdays {
        const NAME: &'static str = "Playdays";
        type Type = super::Playdays;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for Playdays {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for Playdays {
        fn item_type(&self) -> glib::Type {
            crate::model::round::Playday::static_type()
        }

        fn n_items(&self) -> u32 {
            let days = self.end_date().sub(self.start_date()).num_days() + 1;
            days.max(1) as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            let day = self.start_date() + chrono::Duration::days(position as i64);
            let pd = super::Playday::new(day);
            Some(Object::from(pd))
        }
    }
}

pub async fn insert(
    pool: &PgPool,
    round_number: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<i32, String> {
    let result = sqlx::query(
        "INSERT INTO rounds (round_number, start_date, end_date) VALUES ($1, $2, $3) RETURNING round_id",
    )
        .bind(round_number)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await;

    match result {
        Ok(row) => {
            let id = row.get::<i32, _>(0);
            Ok(id)
        }
        Err(e) => {
            error!("Error inserting round: {}", e);
            Err(format!("Error inserting round: {}", e))
        }
    }
}

pub async fn update(
    pool: &PgPool,
    id: i32,
    round_number: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<u64, String> {
    let result = sqlx::query(
        "UPDATE rounds SET round_number=$1, start_date=$2, end_date=$3 WHERE round_id=$4",
    )
        .bind(round_number)
        .bind(start_date)
        .bind(end_date)
        .bind(id)
        .execute(pool)
        .await;

    match result {
        Ok(result) => Ok(result.rows_affected()),
        Err(e) => {
            error!("Error updating round: {}", e);
            Err(format!("Error updating round: {}", e))
        }
    }
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, String> {
    let result = sqlx::query("DELETE FROM rounds WHERE round_id=$1")
        .bind(id)
        .execute(pool)
        .await;

    match result {
        Ok(result) => Ok(result.rows_affected()),
        Err(e) => {
            error!("Error deleting round: {}", e);
            Err(format!("Error deleting round: {}", e))
        }
    }
}

fn build_round(row: PgRow) -> Round {
    let round_id = row.get::<i32, _>(0);
    let round_number = row.get::<i32, _>(1);
    let start_date = row.get::<NaiveDate, _>(2);
    let end_date = row.get::<NaiveDate, _>(3);
    Round::new(round_id, round_number, start_date, end_date)
}

pub async fn get(pool: &PgPool, id: i32) -> Result<Option<Round>, String> {
    let result = sqlx::query("SELECT round_id, round_number, start_date, end_date FROM rounds WHERE round_id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(row) => match row {
            Some(row) => {
                Ok(Some(build_round(row)))
            }
            None => Ok(None),
        },
        Err(e) => {
            error!("Error getting round: {}", e);
            Err(format!("Error getting round: {}", e))
        }
    }
}

pub async fn get_last_round (pool: &PgPool) -> Result<Option<Round>, String> {
    let result = sqlx::query("SELECT round_id, round_number, start_date, end_date FROM rounds ORDER BY round_number DESC LIMIT 1")
        .fetch_optional(pool)
        .await;

    match result {
        Ok(row) => match row {
            Some(row) => {
                Ok(Some(build_round(row)))
            }
            None => Ok(None),
        },
        Err(e) => {
            error!("Error getting round: {}", e);
            Err(format!("Error getting round: {}", e))
        }
    }
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<Round>, String> {
    let result = sqlx::query("SELECT round_id, round_number, start_date, end_date FROM rounds ORDER BY round_number")
        .fetch_all(pool)
        .await;

    match result {
        Ok(rows) => {
            let mut rounds = Vec::new();
            for row in rows {
                rounds.push(build_round(row));
            }
            Ok(rounds)
        }
        Err(e) => {
            error!("Error getting all rounds: {}", e);
            Err(format!("Error getting all rounds: {}", e))
        }
    }
}
