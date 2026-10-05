//! wissel is launched by the desktop as the default web browser. The URLs to
//! open arrive as command-line arguments (`Exec=wissel %u`).
//!
//! `wissel <url>...` shows a window listing the installed browsers. Choosing
//! one prints its desktop id on stdout and exits 0; closing the window or
//! pressing Escape exits 1 with nothing on stdout. Launching the chosen
//! browser is not implemented yet.
//!
//! `wissel --list` prints the detected browsers, one per line, as
//! `<id>\t<name>\t<icon>`.

mod browsers;
mod picker;
mod shortcuts;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: wissel <url>...\n       wissel --list");
        return ExitCode::FAILURE;
    }
    if args == ["--list"] {
        list();
        return ExitCode::SUCCESS;
    }
    match picker::pick(&args, browsers::installed()) {
        Some(browser) => {
            println!("{}", browser.id);
            ExitCode::SUCCESS
        }
        None => ExitCode::FAILURE,
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
