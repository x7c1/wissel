//! wissel is launched by the desktop as the default web browser. The URLs to
//! open arrive as command-line arguments (`Exec=wissel %u`). The picker UI and
//! the hand-off to the chosen browser are not implemented yet.

use std::process::ExitCode;

fn main() -> ExitCode {
    let urls: Vec<String> = std::env::args().skip(1).collect();
    if urls.is_empty() {
        eprintln!("usage: wissel <url>...");
        return ExitCode::FAILURE;
    }
    for url in &urls {
        println!("{url}");
    }
    ExitCode::SUCCESS
}
