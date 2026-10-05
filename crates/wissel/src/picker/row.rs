use gtk4 as gtk;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::browsers::Browser;
use crate::shortcuts;

/// The icon shown when a browser has none, or when the theme cannot find it.
const FALLBACK_ICON: &str = "web-browser";

/// Builds the list row for the browser at `index`.
pub fn build(browser: &Browser, index: usize) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(browser.name.as_str())
        .use_markup(false)
        .activatable(true)
        .build();

    let image = gtk::Image::from_gicon(&icon(browser));
    image.set_icon_size(gtk::IconSize::Large);
    row.add_prefix(&image);

    if let Some(digit) = shortcuts::digit_for(index) {
        let hint = gtk::Label::new(Some(&digit.to_string()));
        hint.add_css_class("dim-label");
        hint.add_css_class("numeric");
        row.add_suffix(&hint);
    }
    row
}

/// Resolves the browser's icon, falling back to a generic browser icon.
fn icon(browser: &Browser) -> gio::Icon {
    browser
        .icon
        .as_deref()
        .and_then(|name| gio::Icon::for_string(name).ok())
        .filter(is_available)
        .unwrap_or_else(|| gio::ThemedIcon::new(FALLBACK_ICON).upcast())
}

/// Whether `icon` can be drawn: a themed icon must exist in the current
/// theme, other icons (files) are taken as they are.
fn is_available(icon: &gio::Icon) -> bool {
    if !icon.is::<gio::ThemedIcon>() {
        return true;
    }
    match gtk::gdk::Display::default() {
        Some(display) => gtk::IconTheme::for_display(&display).has_gicon(icon),
        None => true,
    }
}
