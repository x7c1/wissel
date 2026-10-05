use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use super::row;
use crate::browsers::Browser;
use crate::shortcuts;

const WIDTH: i32 = 420;
const MAX_LIST_HEIGHT: i32 = 480;

/// Builds and shows the picker window.
///
/// Choosing a browser stores it in `chosen` and closes the window; cancelling
/// closes the window and leaves `chosen` empty.
pub fn present(
    app: &adw::Application,
    first_url: &str,
    url_count: usize,
    browsers: &Rc<Vec<Browser>>,
    chosen: &Rc<RefCell<Option<Browser>>>,
) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("wissel")
        .default_width(WIDTH)
        .resizable(false)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(6)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();
    content.append(&url_header(first_url, url_count));

    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .build();
    list.add_css_class("boxed-list");
    for (index, browser) in browsers.iter().enumerate() {
        list.append(&row::build(browser, index));
    }

    let choose = {
        let window = window.downgrade();
        let browsers = Rc::clone(browsers);
        let chosen = Rc::clone(chosen);
        move |index: usize| {
            let Some(browser) = browsers.get(index) else {
                return;
            };
            chosen.replace(Some(browser.clone()));
            if let Some(window) = window.upgrade() {
                window.close();
            }
        }
    };

    {
        let choose = choose.clone();
        list.connect_row_activated(move |_, row| {
            if let Ok(index) = usize::try_from(row.index()) {
                choose(index);
            }
        });
    }

    if browsers.is_empty() {
        let empty = gtk::Label::new(Some("No browsers found"));
        empty.add_css_class("dim-label");
        content.append(&empty);
    } else {
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(MAX_LIST_HEIGHT)
            .child(&list)
            .build();
        content.append(&scrolled);
    }

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&content));
    window.set_content(Some(&toolbar));

    window.add_controller(key_controller(&window, browsers.len(), choose));

    if let Some(first) = list.row_at_index(0) {
        GtkWindowExt::set_focus(&window, Some(&first));
    }
    window.present();
}

/// Shows the URL being opened, and how many more go to the same browser.
fn url_header(first_url: &str, url_count: usize) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Vertical, 2);

    let url = gtk::Label::builder()
        .label(first_url)
        .ellipsize(gtk::pango::EllipsizeMode::Middle)
        .xalign(0.0)
        .tooltip_text(first_url)
        .build();
    url.add_css_class("heading");
    header.append(&url);

    if url_count > 1 {
        let more = url_count - 1;
        let text = if more == 1 {
            "and 1 more link".to_owned()
        } else {
            format!("and {more} more links")
        };
        let count = gtk::Label::builder().label(text).xalign(0.0).build();
        count.add_css_class("dim-label");
        header.append(&count);
    }
    header
}

/// Handles the window-wide keys: digits choose a row directly, Escape
/// cancels. Arrow keys and Enter are left to the list.
fn key_controller(
    window: &adw::ApplicationWindow,
    row_count: usize,
    choose: impl Fn(usize) + 'static,
) -> gtk::EventControllerKey {
    let controller = gtk::EventControllerKey::new();
    // Capture, so that the keys work whichever widget has focus.
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    let window = window.downgrade();
    controller.connect_key_pressed(move |_, key, _, state| {
        let modifiers = gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK;
        if state.intersects(modifiers) {
            return glib::Propagation::Proceed;
        }
        if key == gdk::Key::Escape {
            if let Some(window) = window.upgrade() {
                window.close();
            }
            return glib::Propagation::Stop;
        }
        match key
            .to_unicode()
            .and_then(|c| shortcuts::index_for(c, row_count))
        {
            Some(index) => {
                choose(index);
                glib::Propagation::Stop
            }
            None => glib::Propagation::Proceed,
        }
    });
    controller
}
