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

#![allow(unused)]
#![forbid(unsafe_code)]

use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
use std::sync::LazyLock;
use log::error;
use preferences::{AppInfo, Preferences, PreferencesMap};

use crate::event;
use crate::event::{Event, EventManager};

const PREFS_PATH: &str = "tipping";
pub const APP_INFO: AppInfo = AppInfo {
    name: "kelpie-tipping",
    author: "shartrec.com",
};

// Preference constants
pub const DATABASE_URL: &str = "DB_URL";

static MANAGER: LazyLock<PreferenceManager> = LazyLock::new(|| PreferenceManager {
    preferences: {
        match PreferencesMap::<String>::load(&APP_INFO, PREFS_PATH) {
            Ok(map) => Arc::new(RwLock::new(map)),
            Err(e) => {
                error!("Error opening preferences {}", e);
                Arc::new(RwLock::new(PreferencesMap::new()))
            }
        }
    },
    path: PREFS_PATH,
});


pub struct PreferenceManager {
    preferences: Arc<RwLock<PreferencesMap>>,
    path: &'static str,
}

impl PreferenceManager {
    pub fn get<T: FromStr>(&self, key: &str) -> Option<T> {
        match self.preferences.read().unwrap().get(key) {
            Some(s) => match s.parse::<T>() {
                Ok(i) => Some(i),
                Err(_e) => None,
            },
            None => None,
        }
    }
    pub fn put<T: ToString>(&self, key: &str, value: T) {
        {
            let mut prefs = self.preferences.write().unwrap();
            prefs.insert(key.to_string(), value.to_string());
        }
        self.store();
        event::manager().notify_listeners(Event::PreferencesChanged);
    }

    pub fn remove(&self, key: &str) {
        {
            let mut prefs = self.preferences.write().unwrap();
            let _e = prefs.remove(key);
        }
        self.store();
    }

    pub fn clear(&self) {
        {
            let mut prefs = self.preferences.write().unwrap();
            prefs.clear();
        }
        self.store();
    }

    fn store(&self) {
        let prefs = self.preferences.read().unwrap();
        let _ = prefs.save(&APP_INFO, self.path);
    }
}

pub fn manager() -> &'static PreferenceManager {
    &MANAGER
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use preferences::PreferencesMap;

    use crate::preference;

    #[test]
    fn test_save_restore() {
        let manager = preference::PreferenceManager {
            preferences: Arc::new(RwLock::new(PreferencesMap::new())),
            path: "kelpie-unit-test",
        };

        manager.put("Test_KEY 1", "First");
        manager.put("Test_KEY 2", 1);
        manager.put("Test_KEY 3", 24.66);

        assert_eq!(
            manager.get::<String>("Test_KEY 1"),
            Some("First".to_string())
        );
        assert_eq!(manager.get::<i32>("Test_KEY 2"), Some(1));
        assert_eq!(manager.get::<f64>("Test_KEY 3"), Some(24.66));
    }
}
