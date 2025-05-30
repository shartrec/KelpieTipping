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
use log::error;
use sqlx::{PgPool, Row};

// To use the Team in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Team(ObjectSubclass<imp::Team>);
}

impl Team {
    pub fn new(team_id: Option<i32>, name: String, nickname: String) -> Team
    {
        let obj: Team = glib::Object::new();
        obj.imp().set_id(team_id);
        obj.imp().set_name(name);
        obj.imp().set_nickname(nickname);
        obj
    }

    pub fn id(&self) -> Option<i32> {
        self.imp().team_id.borrow().clone()
    }
    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }
    pub fn nickname(&self) -> String {
        self.imp().nickname.borrow().clone()
    }
    pub fn set_id(&self, id: Option<i32>) {
        self.imp().set_id(id);
    }
    pub fn set_name(&self, name: String) {
        self.imp().set_name(name);
    }
    pub fn set_nickname(&self, nickname: String) {
        self.imp().set_nickname(nickname);
    }
}

glib::wrapper! {
    pub struct Teams(ObjectSubclass<imp::Teams>) @implements gio::ListModel;
}

impl Teams {
    pub fn new() -> Teams {
        glib::Object::new()
    }
}

mod imp {
    use crate::model::team::get_all;
    use crate::util::db;
    use adw::gio;
    use adw::glib::{clone, Object};
    use adw::prelude::StaticType;
    use adw::subclass::prelude::{ListModelImpl, ObjectImpl, ObjectImplExt, ObjectSubclass};
    use gtk::glib;
    use log::error;
    use std::cell::RefCell;
    use std::sync::{Arc, RwLock};

    #[derive(Default)]
    pub struct Team {
        pub(super) team_id: RefCell<Option<i32>>,
        pub(super) name: RefCell<String>,
        pub(super) nickname: RefCell<String>,
    }

    impl Team {
        pub(super) fn set_id(&self, id: Option<i32>) {
            self.team_id.replace(id);
        }

        pub(super) fn set_name(&self, name: String) {
            self.name.replace(name);
        }

        pub(super) fn set_nickname(&self, nickname: String) {
            self.nickname.replace(nickname);
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Team {
        const NAME: &'static str = "Team";
        type Type = super::Team;
    }

    impl ObjectImpl for Team {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    #[derive(Default)]
    pub struct Teams {
        pub teams: Arc<RwLock<Vec<crate::model::team::Team>>>,
    }

    impl Teams {

        pub fn team_at(&self, position: u32) -> Option<crate::model::team::Team> {
            let map = self
                .teams
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            map.iter().nth(position as usize).as_deref().map(|t| t.clone())
        }

        pub fn add_team(&self, team: crate::model::team::Team) {
            let mut binding = self.teams.write().expect("Can't get lock on teams cache");
            binding.insert(0, team);
        }
    }

    // Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Teams {
        const NAME: &'static str = "Teams";
        type Type = super::Teams;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for Teams {
        fn constructed(&self) {
            self.parent_constructed();

            let pool = db::manager().pool();
            async_std::task::block_on(clone!(#[weak(rename_to = list)] self, async move {
                match get_all(pool).await {
                    Ok(team_list) => {
                        let mut binding = list.teams.write().expect("Can't get lock on teams cache");
                        for t in team_list.into_iter() {
                            binding.push(t);
                        }
                    }
                    Err(err) => {
                        error!("Error getting all teams: {}", err);
                    }
                }
            }));
        }
    }

    impl ListModelImpl for Teams {
        fn item_type(&self) -> glib::Type {
            crate::model::team::Team::static_type()
        }

        fn n_items(&self) -> u32 {
            let map = self
                .teams
                .read()
                .expect("Unable to get a lock on the teams");
            map.len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.team_at(position).map(|team| {
                Object::from(team.clone())
            })
        }
    }

}

pub async fn insert(pool: &PgPool, name: String, nickname: String) -> Result<crate::model::team::Team, String> {
    let result = sqlx::query("INSERT INTO teams (name, nickname) VALUES ($1, $2) RETURNING team_id")
        .bind(name.clone())
        .bind(nickname.clone())
        .fetch_one(pool)
        .await;
    match result {
        Ok(row) => {
            let id = row.get::<i32, _>(0);
            Ok(crate::model::team::Team::new(Some(id), name, nickname))
        },
        Err(e) => {
            error!("Error inserting team: {}", e);
            Err(format!("Error inserting team: {}", e))
        },
    }
}

pub async fn update(pool: &PgPool, id: i32, name: String, nickname: String) -> Result<u64, String> {
    let result = sqlx::query("UPDATE teams SET name=$1, nickname=$2 WHERE team_id = $3")
        .bind(name.clone())
        .bind(nickname.clone())
        .bind(id)
        .execute(pool)
        .await;
    match result {
        Ok(result) => {
            Ok(result.rows_affected())
        },
        Err(e) => {
            error!("Error updating team: {}", e);
            Err(format!("Error updating team: {}", e))
        },
    }
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, String> {
    let result = sqlx::query("DELETE FROM teams WHERE team_id = $1")
        .bind(id)
        .execute(pool)
        .await;
    match result {
        Ok(result) => {
            Ok(result.rows_affected())
        },
        Err(e) => {
            error!("Error deleting team: {}", e);
            Err(format!("Error deleting team: {}", e))
        },
    }
}

pub async fn get(pool: &PgPool, id: i32) -> Result<Option<crate::model::team::Team>, String> {
    let result = sqlx::query("SELECT team_id, name, nickname FROM teams WHERE team_id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    match result {
        Ok(row) => {
            match row {
                Some(row) => {
                    let team_id = row.get::<i32, _>(0);
                    let name = row.get::<String, _>(1);
                    let nickname = row.get::<String, _>(2);
                    Ok(Some(Team::new(Some(team_id), name, nickname)))
                },
                None => {
                    Ok(None)
                }
            }
        },
        Err(e) => {
            error!("Error getting team: {}", e);
            Err(format!("Error getting team: {}", e))
        },
    }
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<crate::model::team::Team>, String> {
    let result =
        sqlx::query("SELECT team_id, name, nickname FROM teams ORDER BY name")
            .fetch_all(pool)
            .await;
    match result {
        Ok(rows) => {
            let mut teams = Vec::new();
            for row in rows {
                let team_id = row.get::<i32, _>(0);
                let name = row.get::<String, _>(1);
                let nickname = row.get::<String, _>(2);
                teams.push(Team::new(Some(team_id), name, nickname));
            }
            Ok(teams)
        },
        Err(e) => {
            error!("Error getting all teams: {}", e);
            Err(format!("Error getting all teams: {}", e))
        },
    }
}
