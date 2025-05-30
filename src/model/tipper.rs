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

// To use the Tipper in a Gio::ListModel it needs to ba a glib::Object, so we do all this fancy subclassing stuff
// Public part of the Model type.
glib::wrapper! {
    pub struct Tipper(ObjectSubclass<imp::Tipper>);
}

impl Tipper {
    pub fn new(tipper_id: i32, name: String, email: String) -> Tipper
    {
        let obj: Tipper = glib::Object::new();
        obj.imp().set_id(tipper_id);
        obj.imp().set_name(name);
        obj.imp().set_email(email);
        obj
    }

    pub fn id(&self) -> i32 {
        self.imp().tipper_id.borrow().clone()
    }
    pub fn name(&self) -> String {
        self.imp().name.borrow().clone()
    }
    pub fn email(&self) -> String {
        self.imp().email.borrow().clone()
    }
    pub fn set_id(&self, id: i32) {
        self.imp().set_id(id);
    }
    pub fn set_name(&self, name: String) {
        self.imp().set_name(name);
    }
    pub fn set_email(&self, email: String) {
        self.imp().set_email(email);
    }
}

glib::wrapper! {
    pub struct Tippers(ObjectSubclass<imp::Tippers>) @implements gio::ListModel;
}

impl Tippers {
    pub fn new() -> Tippers {
        glib::Object::new()
    }
}

mod imp {
    use crate::model::tipper::get_all;
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
    pub struct Tipper {
        pub(super) tipper_id: RefCell<i32>,
        pub(super) name: RefCell<String>,
        pub(super) email: RefCell<String>,
    }

    impl Tipper {
        pub(super) fn set_id(&self, id: i32) {
            self.tipper_id.replace(id);
        }

        pub(super) fn set_name(&self, name: String) {
            self.name.replace(name);
        }

        pub(super) fn set_email(&self, email: String) {
            self.email.replace(email);
        }
    }

    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Tipper {
        const NAME: &'static str = "Tipper";
        type Type = super::Tipper;
    }

    impl ObjectImpl for Tipper {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    #[derive(Default)]
    pub struct Tippers {
        pub tippers: Arc<RwLock<Vec<crate::model::tipper::Tipper>>>,
    }

    impl Tippers {

        pub fn tipper_at(&self, position: u32) -> Option<crate::model::tipper::Tipper> {
            let map = self
                .tippers
                .read()
                .expect("Unable to get a lock on the aircraft hangar");
            map.iter().nth(position as usize).as_deref().map(|t| t.clone())
        }

    }
    /// Basic declaration of our type for the GObject type system
    #[glib::object_subclass]
    impl ObjectSubclass for Tippers {
        const NAME: &'static str = "Tippers";
        type Type = super::Tippers;
        type Interfaces = (gio::ListModel, );
    }

    impl ObjectImpl for Tippers {
        fn constructed(&self) {
            self.parent_constructed();

            let pool = db::manager().pool();
            async_std::task::block_on(clone!(#[weak(rename_to = list)] self, async move {
                match get_all(pool).await {
                    Ok(tipper_list) => {
                        let mut binding = list.tippers.write().expect("Can't get lock on tippers cache");
                        for t in tipper_list.into_iter() {
                            binding.push(t);
                        }
                    }
                    Err(err) => {
                        error!("Error getting all tippers: {}", err);
                    }
                }
            }));
        }
    }

    impl ListModelImpl for Tippers {
        fn item_type(&self) -> glib::Type {
            crate::model::tipper::Tipper::static_type()
        }

        fn n_items(&self) -> u32 {
            let map = self
                .tippers
                .read()
                .expect("Unable to get a lock on the tippers");
            map.len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.tipper_at(position).map(|tipper| {
                Object::from(tipper.clone())
            })
        }
    }

}

pub async fn insert(pool: &PgPool, name: String, email: String) -> Result<crate::model::tipper::Tipper, String> {
    let result = sqlx::query("INSERT INTO tippers (name, email) VALUES ($1, $2) RETURNING tipper_id")
        .bind(name.clone())
        .bind(email.clone())
        .fetch_one(pool)
        .await;
    match result {
        Ok(row) => {
            let id = row.get::<i32, _>(0);
            Ok(crate::model::tipper::Tipper::new(id, name, email))
        },
        Err(e) => {
            error!("Error inserting tipper: {}", e);
            Err(format!("Error inserting tipper: {}", e))
        },
    }
}

pub async fn update(pool: &PgPool, id: i32, name: String, email: String) -> Result<u64, String> {
    let result = sqlx::query("UPDATE tippers SET name=$1, email=$2 WHERE tipper_id = $3")
        .bind(name.clone())
        .bind(email.clone())
        .bind(id)
        .execute(pool)
        .await;
    match result {
        Ok(result) => {
            Ok(result.rows_affected())
        },
        Err(e) => {
            error!("Error updating tipper: {}", e);
            Err(format!("Error updating tipper: {}", e))
        },
    }
}

pub async fn delete(pool: &PgPool, id: i32) -> Result<u64, String> {
    let result = sqlx::query("DELETE FROM tippers WHERE tipper_id = $1")
        .bind(id)
        .execute(pool)
        .await;
    match result {
        Ok(result) => {
            Ok(result.rows_affected())
        },
        Err(e) => {
            error!("Error deleting tipper: {}", e);
            Err(format!("Error deleting tipper: {}", e))
        },
    }
}

pub async fn get(pool: &PgPool, id: i32) -> Result<Option<crate::model::tipper::Tipper>, String> {
    let result = sqlx::query("SELECT tipper_id, name, email FROM tippers WHERE tipper_id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await;
    match result {
        Ok(row) => {
            match row {
                Some(row) => {
                    let tipper_id = row.get::<i32, _>(0);
                    let name = row.get::<String, _>(1);
                    let email = row.get::<String, _>(2);
                    Ok(Some(Tipper::new(tipper_id, name, email)))
                },
                None => {
                    Ok(None)
                }
            }
        },
        Err(e) => {
            error!("Error getting tipper: {}", e);
            Err(format!("Error getting tipper: {}", e))
        },
    }
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<crate::model::tipper::Tipper>, String> {
    let result =
        sqlx::query("SELECT tipper_id, name, email FROM tippers ORDER BY name")
            .fetch_all(pool)
            .await;
    match result {
        Ok(rows) => {
            let mut tippers = Vec::new();
            for row in rows {
                let tipper_id = row.get::<i32, _>(0);
                let name = row.get::<String, _>(1);
                let email = row.get::<String, _>(2);
                tippers.push(Tipper::new(tipper_id, name, email));
            }
            Ok(tippers)
        },
        Err(e) => {
            error!("Error getting all tippers: {}", e);
            Err(format!("Error getting all tippers: {}", e))
        },
    }
}
