//! wissel is launched by the desktop as the default web browser. The URLs to
//! open arrive as command-line arguments (`Exec=wissel %u`). The picker UI and
//! the hand-off to the chosen browser are not implemented yet.
//!
//! `wissel --list` prints the detected browsers, one per line, as
//! `<id>\t<name>\t<icon>`.

mod browsers;

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
    for url in &args {
        println!("{url}");
    }
    ExitCode::SUCCESS
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
