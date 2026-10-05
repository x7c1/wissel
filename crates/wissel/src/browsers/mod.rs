//! Detection of the web browsers installed on the system, and launching them.
//!
//! A browser is any application that registers itself as a handler for
//! `x-scheme-handler/https`. The lookup is delegated to GIO, which walks
//! `XDG_DATA_HOME` and `XDG_DATA_DIRS`, reads each directory's
//! `mimeinfo.cache`, and applies desktop-entry precedence.

mod browser;
pub use browser::Browser;

mod installed;
pub use installed::installed;

mod launch;
pub use launch::launch;

mod launch_error;
pub use launch_error::LaunchError;
