//! wissel is launched by the desktop as the default web browser. The URLs to
//! open arrive as command-line arguments (`Exec=wissel %u`).
//!
//! `wissel <url>...` shows a window listing the installed browsers. Choosing
//! one opens all the URLs in that browser and exits 0; closing the window or
//! pressing Escape exits 1.
//!
//! `wissel --open <desktop-id> <url>...` opens the URLs with the given
//! desktop entry without showing the picker.
//!
//! `wissel --list` prints the detected browsers, one per line, as
//! `<id>\t<name>\t<icon>`.
//!
//! Any other first argument starting with `-` prints the usage and exits 1.
//!
//! When a browser cannot be launched, wissel prints the reason on stderr and
//! exits 1.

mod browsers;
mod picker;
mod shortcuts;

use std::process::ExitCode;

use gtk4::prelude::*;

const USAGE: &str = "usage: wissel <url>...
       wissel --open <desktop-id> <url>...
       wissel --list";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None => usage(),
        Some("--list") if args.len() == 1 => {
            list();
            ExitCode::SUCCESS
        }
        Some("--open") => match &args[1..] {
            [id, urls @ ..] if !urls.is_empty() => launch(id, urls, &gio::AppLaunchContext::new()),
            _ => usage(),
        },
        // An unknown option would otherwise reach the chosen browser as an
        // argument of its own.
        Some(arg) if arg.starts_with('-') => usage(),
        Some(_) => pick(&args),
    }
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::FAILURE
}

/// Shows the picker and opens `urls` in the chosen browser.
///
/// The launch happens inside the picker, while its window is still open, so
/// the window's launch context hands the browser a valid activation token.
fn pick(urls: &[String]) -> ExitCode {
    let launch_urls = urls.to_vec();
    match picker::pick(urls, browsers::installed(), move |browser, context| {
        browsers::launch(&browser.id, &launch_urls, context)
    }) {
        Some(launched) => report(launched),
        None => ExitCode::FAILURE,
    }
}

fn launch(id: &str, urls: &[String], context: &impl IsA<gio::AppLaunchContext>) -> ExitCode {
    report(browsers::launch(id, urls, context))
}

fn report(launched: Result<(), browsers::LaunchError>) -> ExitCode {
    match launched {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("wissel: {error}");
            ExitCode::FAILURE
        }
    }
}

fn list() {
    for browser in browsers::installed() {
        println!(
            "{}\t{}\t{}",
            browser.id,
            browser.name,
            browser.icon.as_deref().unwrap_or("")
        );
    }
}
