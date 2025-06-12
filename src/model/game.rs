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
use crate::model::team::Teams;
use crate::util::db;
use adw::glib::clone;
use adw::subclass::prelude::ObjectSubclassIsExt;
use adw::{gio, glib};
use chrono::NaiveDate;
use log::error;
use sqlx::{PgPool, Row};

// To use the Game in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Game(ObjectSubclass<imp::Game>);
}

impl Game {
    pub fn new(game_id: i32, round_id: i32, home_game_id: i32, away_game_id: i32,
               game_date: NaiveDate, home_team_score: Option<i32>, away_team_score: Option<i32>) -> Game
    {
        let obj: Game = glib::Object::new();
        obj.imp().set_id(game_id);
        obj.imp().set_round_id(round_id);
        obj.imp().set_home_team_id(home_game_id);
        obj.imp().set_away_team_id(away_game_id);
        obj.imp().set_game_date(game_date);
        obj.imp().set_home_team_score(home_team_score);
        obj.imp().set_away_team_score(away_team_score);
        obj
    }

    pub fn id(&self) -> i32 {
        self.imp().game_id.borrow().clone()
    }
    pub fn round_id(&self) -> i32 {
        self.imp().round_id.borrow().clone()
    }
    pub fn home_team_id(&self) -> i32 {
        self.imp().home_team_id.borrow().clone()
    }
    pub fn away_team_id(&self) -> i32 {
        self.imp().away_team_id.borrow().clone()
    }
    pub fn game_date(&self) -> NaiveDate {
        self.imp().game_date.borrow().clone()
    }
    pub fn home_team_score(&self) -> Option<i32> {
        self.imp().home_team_score.borrow().clone()
    }
    pub fn away_team_score(&self) -> Option<i32> {
        self.imp().away_team_score.borrow().clone()
    }

}

glib::wrapper! {
    pub struct Games(ObjectSubclass<imp::Games>) @implements gio::ListModel;
}

impl Games {
    pub fn new(start: NaiveDate, end: NaiveDate, games: &mut Vec<Game>) -> Games {
        let obj: Games = glib::Object::new();
        obj.imp().set_games(games);
        obj
    }

    pub fn for_round(round_id: i32) -> Games {
        let pool = db::manager().pool();
        let obj : Games /* Type */ = glib::Object::new();
        let games = async_std::task::block_on(async move {
            get_for_round(pool, round_id).await
        });
        match games {
            Ok(game_list) => {
                let mut temp: Vec<Game> = vec![];
                for t in game_list.into_iter() {
                    temp.push(t);
                }
                obj.imp().set_games(&mut temp);
            }
            Err(err) => {
                error!("Error getting games for round{}: {}", round_id, err);
            }
        }
        obj
    }

}

mod imp {
    use adw::gio;
    use adw::glib::Object;
    use adw::prelude::StaticType;
    use adw::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};
    use chrono::NaiveDate;
    use gtk::glib;
    use std::cell::RefCell;
    use std::sync::{Arc, RwLock};

    #[derive(Debug)]
    #[derive(Default)]
    pub struct Game {
        pub game_id: RefCell<i32>,
        pub round_id: RefCell<i32>,
        pub home_team_id: RefCell<i32>,
        pub away_team_id: RefCell<i32>,
        pub game_date: RefCell<NaiveDate>,
        pub home_team_score: RefCell<Option<i32>>,
        pub away_team_score: RefCell<Option<i32>>,
    }
    impl Game {
        pub(crate) fn set_id(&self, id: i32) {
            self.game_id.replace(id);
        }
        pub(crate) fn set_round_id(&self, id: i32) {
            self.round_id.replace(id);
        }
        pub(crate) fn set_home_team_id(&self, id: i32) {
            self.home_team_id.replace(id);
        }
        pub(crate) fn set_away_team_id(&self, id: i32) {
            self.away_team_id.replace(id);
        }
        pub(crate) fn set_game_date(&self, date: NaiveDate) {
            println!("!!!!   Setting game date to: {:?}", date);

            self.game_date.replace(date);
        }
        pub(crate) fn set_home_team_score(&self, score: Option<i32>) {
            self.home_team_score.replace(score);
        }
        pub(crate) fn set_away_team_score(&self, score: Option<i32>) {
            self.away_team_score.replace(score);
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Game {
        const NAME: &'static str = "Game";
        type Type = super::Game;
    }

    impl ObjectImpl for Game {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    #[derive(Default)]
    pub struct Games {
        pub games: Arc<RwLock<Vec<crate::model::game::Game>>>,
    }

    impl Games {
        pub(super) fn game_at(&self, position: u32) -> Option<crate::model::game::Game> {
            let map = self.games.read().expect("Unable to get a lock on games");
            map.iter().nth(position as usize).as_deref().map(|t| t.clone())
        }

        pub(crate) fn game_by_id(&self, id: i32) -> Option<crate::model::game::Game> {
            let map = self.games.read().expect("Unable to get a lock on games");
            for game in map.iter() {
                if game.id() == id as i32 {
                    return Some(game.clone());
                }
            }
            None
        }

        pub fn set_games(&self, games: &mut Vec<crate::model::game::Game>) {
            let mut map = self.games.write().expect("Unable to get a lock on games");
            map.clear();
            map.append(games);
        }
    }
    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Games {
        const NAME: &'static str = "Games";
        type Type = super::Games;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for Games {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl ListModelImpl for Games {
        fn item_type(&self) -> glib::Type {
            crate::model::game::Game::static_type()
        }

        fn n_items(&self) -> u32 {
            let map = self.games.read().expect("Unable to get a lock on games");
            map.len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            match self.game_at(position) {
                Some(game) => Some(Object::from(game)),
                None => None,
            }
        }
    }
}

pub async fn insert(
    pool: &PgPool,
    round_id: i32,
    home_team_id: i32,
    away_team_id: i32,
    game_date: NaiveDate,
    home_team_score: Option<i32>,
    away_team_score: Option<i32>,
) -> Result<i32, String> {
    let result = sqlx::query(
        "INSERT INTO games (round_id, home_team_id, away_team_id, game_date, home_team_score, away_team_score) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING game_id",
    )
        .bind(round_id)
        .bind(home_team_id)
        .bind(away_team_id)
        .bind(game_date)
        .bind(home_team_score)
        .bind(away_team_score)
        .fetch_one(pool)
        .await;

    match result {
        Ok(row) => {
            let game_id = row.get::<i32, _>(0);
            Ok(game_id)
        }
        Err(e) => {
            error!("Error inserting game: {}", e);
            Err(format!("Error inserting game: {}", e))
        }
    }
}


pub async fn update(
    pool: &PgPool,
    game_id: i32,
    round_id: i32,
    home_team_id: i32,
    away_team_id: i32,
    game_date: NaiveDate,
    home_team_score: Option<i32>,
    away_team_score: Option<i32>,
) -> Result<u64, String> {
    let result = sqlx::query(
        "UPDATE games SET round_id=$1, home_team_id=$2, away_team_id=$3, game_date=$4, \
         home_team_score=$5, away_team_score=$6 WHERE game_id=$7",
    )
        .bind(round_id)
        .bind(home_team_id)
        .bind(away_team_id)
        .bind(game_date)
        .bind(home_team_score)
        .bind(away_team_score)
        .bind(game_id)
        .execute(pool)
        .await;

    match result {
        Ok(result) => Ok(result.rows_affected()),
        Err(e) => {
            error!("Error updating game: {}", e);
            Err(format!("Error updating game: {}", e))
        }
    }
}

pub async fn delete(pool: &PgPool, game_id: i32) -> Result<u64, String> {
    let result = sqlx::query("DELETE FROM games WHERE game_id=$1")
        .bind(game_id)
        .execute(pool)
        .await;

    match result {
        Ok(result) => Ok(result.rows_affected()),
        Err(e) => {
            error!("Error deleting game: {}", e);
            Err(format!("Error deleting game: {}", e))
        }
    }
}

pub async fn get(pool: &PgPool, game_id: i32) -> Result<Option<Game>, String> {
    let result = sqlx::query(
        "SELECT game_id, round_id, home_team_id, away_team_id, game_date, home_team_score, away_team_score \
         FROM games WHERE game_id=$1",
    )
        .bind(game_id)
        .fetch_optional(pool)
        .await;

    match result {
        Ok(row) => match row {
            Some(row) => {
                let game = Game::new(
                    row.get::<i32, _>(0),
                    row.get::<i32, _>(1),
                    row.get::<i32, _>(2),
                    row.get::<i32, _>(3),
                    row.get::<NaiveDate, _>(4),
                    row.get::<Option<i32>, _>(5),
                    row.get::<Option<i32>, _>(6),
                );
                Ok(Some(game))
            }
            None => Ok(None),
        },
        Err(e) => {
            error!("Error getting game: {}", e);
            Err(format!("Error getting game: {}", e))
        }
    }
}

pub async fn get_for_round(pool: &PgPool, round_id: i32) -> Result<Vec<Game>, String> {
    let result = sqlx::query(
        "SELECT game_id, round_id, home_team_id, away_team_id, game_date, home_team_score, away_team_score \
         FROM games WHERE round_id = $1 ORDER BY game_date",
    )
        .bind(round_id)
        .fetch_all(pool)
        .await;

    match result {
        Ok(rows) => {
            let games = rows
                .into_iter()
                .map(|row| {
                    Game::new(
                        row.get::<i32, _>(0),
                        row.get::<i32, _>(1),
                        row.get::<i32, _>(2),
                        row.get::<i32, _>(3),
                        row.get::<NaiveDate, _>(4),
                        row.get::<Option<i32>, _>(5),
                        row.get::<Option<i32>, _>(6),
                    )
                })
                .collect();
            Ok(games)
        }
        Err(e) => {
            error!("Error getting all games: {}", e);
            Err(format!("Error getting all games: {}", e))
        }
    }
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<Game>, String> {
    let result = sqlx::query(
        "SELECT game_id, round_id, home_team_id, away_team_id, game_date, home_team_score, away_team_score \
         FROM games ORDER BY game_date",
    )
        .fetch_all(pool)
        .await;

    match result {
        Ok(rows) => {
            let games = rows
                .into_iter()
                .map(|row| {
                    Game::new(
                        row.get::<i32, _>(0),
                        row.get::<i32, _>(1),
                        row.get::<i32, _>(2),
                        row.get::<i32, _>(3),
                        row.get::<NaiveDate, _>(4),
                        row.get::<Option<i32>, _>(5),
                        row.get::<Option<i32>, _>(6),
                    )
                })
                .collect();
            Ok(games)
        }
        Err(e) => {
            error!("Error getting all games: {}", e);
            Err(format!("Error getting all games: {}", e))
        }
    }
}