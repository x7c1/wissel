//! The picker window: lists the installed browsers and lets the user choose
//! one with the mouse or the keyboard.

mod row;
mod window;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk;
use gtk4::prelude::*;
use libadwaita as adw;

use crate::browsers::Browser;

/// The application id. GNOME matches it against the desktop entry's file
/// name (`io.github.x7c1.wissel.desktop`) to give the window its name and
/// icon.
const APP_ID: &str = "io.github.x7c1.wissel";

/// What `pick` runs when the user chooses a browser.
type OnPick<T> = Box<dyn FnOnce(&Browser, &gdk::AppLaunchContext) -> T>;

/// The window's hook for a choice: shared with the window, callable any
/// number of times, and silent after the first call.
pub(super) type OnChoose = Rc<dyn Fn(&Browser, &gdk::AppLaunchContext)>;

/// Shows the picker for `urls` and blocks until the window closes.
///
/// When the user chooses a browser, `on_pick` runs with that browser and the
/// window's launch context while the window is still open, and its result is
/// returned. Returns `None` when the user cancelled.
///
/// Running `on_pick` before the window closes matters: the activation token
/// GTK attaches to the context is only honoured for a focused window that
/// just received input, and that is what lets a launched browser take focus.
pub fn pick<T: 'static>(
    urls: &[String],
    browsers: Vec<Browser>,
    on_pick: impl FnOnce(&Browser, &gdk::AppLaunchContext) -> T + 'static,
) -> Option<T> {
    // Every invocation needs its own window and its own answer, so the
    // application must not hand its work over to a running instance.
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    let result: Rc<RefCell<Option<T>>> = Rc::default();
    let on_pick: Rc<RefCell<Option<OnPick<T>>>> = Rc::new(RefCell::new(Some(Box::new(on_pick))));
    let on_choose: OnChoose = {
        let result = Rc::clone(&result);
        Rc::new(move |browser, context| {
            if let Some(on_pick) = on_pick.borrow_mut().take() {
                result.replace(Some(on_pick(browser, context)));
            }
        })
    };

    let browsers = Rc::new(browsers);
    let first_url = urls.first().cloned().unwrap_or_default();
    let url_count = urls.len();
    app.connect_activate(move |app| {
        window::present(app, &first_url, url_count, &browsers, &on_choose);
    });

    // The URLs are wissel's to interpret, so GTK gets no arguments to parse.
    app.run_with_args::<&str>(&[]);
    result.take()
}
