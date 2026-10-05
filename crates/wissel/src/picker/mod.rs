//! The picker window: lists the installed browsers and lets the user choose
//! one with the mouse or the keyboard.

mod row;
mod window;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use libadwaita as adw;

use crate::browsers::Browser;

/// The application id. GNOME matches it against the desktop entry's file
/// name (`io.github.x7c1.wissel.desktop`) to give the window its name and
/// icon.
const APP_ID: &str = "io.github.x7c1.wissel";

/// Shows the picker for `urls` and blocks until the window closes.
///
/// Returns the chosen browser, or `None` when the user cancelled.
pub fn pick(urls: &[String], browsers: Vec<Browser>) -> Option<Browser> {
    // Every invocation needs its own window and its own answer, so the
    // application must not hand its work over to a running instance.
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    let chosen: Rc<RefCell<Option<Browser>>> = Rc::default();
    let browsers = Rc::new(browsers);
    let first_url = urls.first().cloned().unwrap_or_default();
    let url_count = urls.len();
    {
        let chosen = Rc::clone(&chosen);
        app.connect_activate(move |app| {
            window::present(app, &first_url, url_count, &browsers, &chosen);
        });
    }

    // The URLs are wissel's to interpret, so GTK gets no arguments to parse.
    app.run_with_args::<&str>(&[]);
    chosen.take()
}
