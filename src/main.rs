/*
 * Copyright (c) 2025-2025. Trevor Campbell and others.
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

#![windows_subsystem = "windows"]
#![forbid(unsafe_code)]

mod window;
pub(crate) mod util;
pub(crate) mod model;
pub(crate) mod preference;
pub(crate) mod event;

use adw::Application;
use gtk::{gio, glib, CssProvider, UriLauncher};
use adw::gdk::Display;
use gtk::gio::{Cancellable, File, SimpleAction};
use gtk::glib::clone;
use adw::prelude::*;
use adw::subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt};
use async_std::task;
use log::{error, warn};
use util::Logger;
// use window::Window;
use gettextrs::{TextDomain, TextDomainError};
use sqlx::{PgPool, Pool, Postgres};
use crate::preference::PreferenceManager;
use crate::util::{db, info};
use crate::window::Window;
use crate::window::edit_team::TeamDialog;
use crate::window::util::show_help_about;

const APP_ID: &str = "com.shartrec.KelpieTipping";

fn main() -> glib::ExitCode {

    // Create the LoggerGuard instance, this will initialize the logger
    // and flush it when the instance goes out of scope
    let _logger = Logger::new();

    let conn_url = "postgres://shartrec:laverda21@localhost:5432/kelpie_tips";
    task::block_on(db::initialize_manager(conn_url.to_string()));
    // init_locale();

    // Register and include resources
    gio::resources_register_include!("kelpie_tipping.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_startup(|_app| {
        load_css();
    });

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn init_locale() {
    let path = match std::env::current_exe() {
        Ok(exe_path) => {
            match exe_path.canonicalize() {
                Ok(canonical_path) => {
                    if let Some(parent) = canonical_path.parent() {
                        Some(parent.display().to_string())
                    } else {
                        warn!("Failed to get executable path: No parent directory");
                        None
                    }
                }
                Err(e) => {
                    warn!("Failed to get executable path: {}", e);
                    None
                },
            }
        }
        Err(e) => {
            warn!("Failed to get executable path: {}", e);
            None
        },
    };

    let mut text_domain = TextDomain::new("kelpie_tips");
    if let Some(path) = path.clone() {
        text_domain = text_domain.push(path);
    }
    match text_domain
        .init() {
        Ok(_) => {}
        Err(err) => {
            match err {
                TextDomainError::InvalidLocale(locale) => {
                    warn!("Failed to find translation for {}, using default", locale);
                }
                TextDomainError::TranslationNotFound(locale) => {
                    warn!("Failed to find translation for {}, using default", locale);
                }
                _ => {}
            }
            let mut text_domain = TextDomain::new("kelpie_tips");
            if let Some(path) = path {
                text_domain = text_domain.push(path);
            }
            match text_domain.locale("").init() {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to initialize text domain: {}", err);
                }
            }
        }
    }
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_resource("/com/shartrec/kelpie_tipping/style.css");

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn connect_actions(app: &Application, window: &Window) {

    let action = SimpleAction::new("new-team", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        let team_dialog = TeamDialog::new();
        team_dialog.set_transient_for(Some(&window));
        team_dialog.set_visible(true);
    }));
    app.add_action(&action);

    let action = SimpleAction::new("quit", None);
    action.connect_activate(clone!(#[weak] app, move |_action, _parameter| {
        app.quit()
    }));
    app.add_action(&action);

    let action = SimpleAction::new("help-about", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        show_help_about(&window);
    }));
    app.add_action(&action);

    let action = SimpleAction::new("help-contents", None);
    action.connect_activate(clone!(#[weak] window, move |_action, _parameter| {
        UriLauncher::builder()
            .uri(info::DOCSITE)
            .build()
            .launch(Some(&window), Some(&Cancellable::default()), |_| {});
    }));
    app.add_action(&action);
}

fn build_ui(app: &Application) {
    let window = Window::new(app);
    connect_actions(app, &window);
    window.present();
}

async fn setup_database() -> Result<Pool<Postgres>, sqlx::Error> {
    let conn_url =
        preference::manager().get::<String>(preference::DATABASE_URL).unwrap_or_else(|| {
            warn!("No database URL found, using default");
            "sqlite://kelpie.db".to_string()
        });
    let pool = sqlx::PgPool::connect(&conn_url).await?;
    Ok(pool)
}