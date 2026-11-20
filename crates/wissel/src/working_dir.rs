//! Making sure wissel runs in a working directory that exists.
//!
//! The browser wissel launches inherits wissel's working directory, which
//! wissel itself inherits from whatever opened the link. When that directory
//! has been deleted, some browsers (Google Chrome among them) fail to start or
//! to hand the URL to their running instance, so the link silently does not
//! open.

use std::path::Path;

/// Changes to `$HOME`, or to `/` when that fails, if the current working
/// directory no longer exists. A working directory that exists is left
/// unchanged.
///
/// When even `/` cannot be entered, the working directory is left as it is:
/// the launch may still work for browsers that do not depend on it.
pub fn ensure_working_dir() {
    if std::env::current_dir().is_ok() {
        return;
    }
    let home = std::env::var_os("HOME").filter(|home| !home.is_empty());
    if home.is_some_and(|home| std::env::set_current_dir(home).is_ok()) {
        return;
    }
    let _ = std::env::set_current_dir(Path::new("/"));
}
