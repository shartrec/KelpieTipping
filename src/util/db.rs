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
use log::{info, warn};
use sqlx::PgPool;
use std::sync::OnceLock;

static MANAGER: OnceLock<ConnectionManager> = OnceLock::new();

pub(crate) struct ConnectionManager {
    pool: PgPool
}
impl ConnectionManager {
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        async_std::task::block_on(self.pool.close());
    }
}

pub async fn initialize_manager(conn_url: String) {
    let pool = sqlx::PgPool::connect(&conn_url).await.unwrap_or_else(|e| {
        panic!("Error connecting to database: {}", e);
    });

    match MANAGER.set(ConnectionManager { pool }) {
        Ok(_) => {
            info!("ConnectionManager initialized");
        }
        Err(_) => {
            warn!("ConnectionManager already initialized");
        }
    }
}

pub fn manager() -> &'static ConnectionManager {
    MANAGER.get().expect("Manager not initialized")
}