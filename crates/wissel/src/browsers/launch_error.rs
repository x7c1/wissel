use std::fmt;

/// A failure to open URLs with a browser.
#[derive(Debug)]
pub enum LaunchError {
    /// No desktop entry has the given desktop file id.
    UnknownId {
        /// The desktop file id that did not resolve.
        id: String,
        /// The URLs that could not be opened.
        urls: Vec<String>,
    },
    /// GIO found the entry but could not start the browser.
    Gio {
        /// The browser's display name.
        browser: String,
        /// The URLs that could not be opened.
        urls: Vec<String>,
        /// The error GIO reported.
        cause: gio::glib::Error,
    },
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownId { id, urls } => write!(
                f,
                "could not open {} with {id}: no such desktop entry",
                urls.join(" ")
            ),
            Self::Gio {
                browser,
                urls,
                cause,
            } => write!(
                f,
                "could not open {} with {browser}: {cause}",
                urls.join(" ")
            ),
        }
    }
}

impl std::error::Error for LaunchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownId { .. } => None,
            Self::Gio { cause, .. } => Some(cause),
        }
    }
}
