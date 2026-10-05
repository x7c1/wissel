use gio::prelude::*;

use super::Browser;

/// The MIME type a desktop entry must handle to count as a browser.
const HTTPS_SCHEME: &str = "x-scheme-handler/https";

/// The desktop file id of wissel itself, which must never be offered.
const SELF_ID: &str = "io.github.x7c1.wissel.desktop";

/// Lists the installed browsers, sorted by name case-insensitively and then
/// by id.
///
/// Entries hidden with `NoDisplay=true` and wissel's own entry are excluded.
pub fn installed() -> Vec<Browser> {
    let mut browsers: Vec<Browser> = gio::AppInfo::all_for_type(HTTPS_SCHEME)
        .into_iter()
        .filter(|app| app.should_show())
        .filter_map(|app| {
            let id = app.id()?.to_string();
            if id == SELF_ID {
                return None;
            }
            Some(Browser {
                id,
                name: app.display_name().to_string(),
                icon: app
                    .icon()
                    .and_then(|icon| IconExt::to_string(&icon))
                    .map(|s| s.to_string()),
            })
        })
        .collect();
    sort(&mut browsers);
    browsers
}

fn sort(browsers: &mut [Browser]) {
    browsers.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser(id: &str, name: &str) -> Browser {
        Browser {
            id: id.to_owned(),
            name: name.to_owned(),
            icon: None,
        }
    }

    #[test]
    fn sorts_by_name_case_insensitively_then_by_id() {
        let mut browsers = vec![
            browser("b.desktop", "Zeta"),
            browser("c.desktop", "alpha"),
            browser("a.desktop", "Alpha"),
        ];
        sort(&mut browsers);
        let ids: Vec<&str> = browsers.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, ["a.desktop", "c.desktop", "b.desktop"]);
    }
}
