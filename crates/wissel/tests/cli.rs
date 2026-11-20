//! Integration tests for the `wissel` binary.
//!
//! GIO reads the XDG environment once per process, so each test runs the
//! binary as a child process with the XDG variables pointing at the fixture
//! and at empty temporary directories. This keeps the host's desktop entries
//! and `mimeapps.list` out of the result.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// An empty directory under the system temp dir, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "wissel-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        if path.exists() {
            std::fs::remove_dir_all(&path).expect("remove stale temp dir");
        }
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs wissel with `args` against the committed fixture data directory.
fn run(args: &[&str]) -> Output {
    run_with_env(args, &[])
}

/// Like [`run`], with extra environment variables for wissel and the
/// processes it launches.
fn run_with_env(args: &[&str], env: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_wissel"));
    command.args(args);
    output_of(command, env)
}

/// Runs wissel with `args` from a working directory that has been deleted.
///
/// A child process cannot be spawned in a directory that no longer exists, so
/// a shell creates a directory, enters it, removes it and then becomes wissel.
fn run_from_deleted_dir(args: &[&str], env: &[(&str, &Path)]) -> Output {
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg(r#"dir=$(mktemp -d) && cd "$dir" && rmdir "$dir" && exec "$0" "$@""#)
        .arg(env!("CARGO_BIN_EXE_wissel"))
        .args(args);
    output_of(command, env)
}

/// Runs `command` against the committed fixture data directory, with extra
/// environment variables for wissel and the processes it launches.
fn output_of(mut command: Command, env: &[(&str, &Path)]) -> Output {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let data_home = TempDir::new();
    let config_home = TempDir::new();
    let config_dirs = TempDir::new();
    command
        .env("XDG_DATA_DIRS", &fixtures)
        .env("XDG_DATA_HOME", data_home.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("XDG_CONFIG_DIRS", config_dirs.path())
        .envs(env.iter().copied())
        .output()
        .expect("run wissel")
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).expect("stdout is UTF-8")
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).expect("stderr is UTF-8")
}

/// Waits for a detached process to write `path`, and returns its contents.
fn wait_for_file(path: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Ok(contents) = std::fs::read_to_string(path) {
            return contents;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn list_prints_visible_browsers_sorted_by_name() {
    let output = run(&["--list"]);
    assert!(output.status.success(), "{output:?}");
    // "alpha web" sorts before "Beta Browser" only when case is ignored.
    assert_eq!(
        stdout(&output),
        "alpha.desktop\talpha web\t\n\
         beta.desktop\tBeta Browser\tbeta-browser\n\
         recorder.desktop\tRecorder\t\n"
    );
}

#[test]
fn no_arguments_fails_with_usage() {
    let output = run(&[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
}

#[test]
fn open_launches_the_entry_with_all_urls_in_order() {
    let out_dir = TempDir::new();
    let out = out_dir.path().join("urls");
    let output = run_with_env(
        &[
            "--open",
            "recorder.desktop",
            "https://example.com/first",
            "https://example.org/second?q=1",
        ],
        &[("WISSEL_TEST_OUT", &out)],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        wait_for_file(&out),
        "https://example.com/first\nhttps://example.org/second?q=1\n"
    );
}

#[test]
fn open_with_an_unknown_id_fails_naming_the_id() {
    let output = run(&["--open", "missing.desktop", "https://example.com"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(
        stderr(&output).contains("could not open https://example.com with missing.desktop"),
        "{output:?}"
    );
}

#[test]
fn open_without_urls_fails_with_usage() {
    let output = run(&["--open", "recorder.desktop"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
}

#[test]
fn unknown_option_fails_with_usage_instead_of_showing_the_picker() {
    let output = run(&["--help"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(stderr(&output).starts_with("usage:"), "{output:?}");
}

/// Reads the working directory `cwd-recorder.desktop` was launched in.
fn launched_dir(out: &Path) -> PathBuf {
    PathBuf::from(wait_for_file(out).trim_end_matches('\n'))
}

#[test]
fn open_from_a_deleted_working_directory_launches_from_home() {
    let home = TempDir::new();
    let out_dir = TempDir::new();
    let out = out_dir.path().join("cwd");
    let output = run_from_deleted_dir(
        &["--open", "cwd-recorder.desktop", "https://example.com"],
        &[("HOME", home.path()), ("WISSEL_TEST_OUT", &out)],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(
        launched_dir(&out),
        home.path().canonicalize().expect("canonicalize HOME")
    );
}

#[test]
fn open_from_a_deleted_working_directory_without_home_launches_from_root() {
    let missing = TempDir::new();
    let home = missing.path().join("missing");
    let out_dir = TempDir::new();
    let out = out_dir.path().join("cwd");
    let output = run_from_deleted_dir(
        &["--open", "cwd-recorder.desktop", "https://example.com"],
        &[("HOME", &home), ("WISSEL_TEST_OUT", &out)],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(launched_dir(&out), Path::new("/"));
}

#[test]
fn open_keeps_a_working_directory_that_exists() {
    let home = TempDir::new();
    let cwd = TempDir::new();
    let out_dir = TempDir::new();
    let out = out_dir.path().join("cwd");
    let mut command = Command::new(env!("CARGO_BIN_EXE_wissel"));
    command
        .args(["--open", "cwd-recorder.desktop", "https://example.com"])
        .current_dir(cwd.path());
    let output = output_of(command, &[("HOME", home.path()), ("WISSEL_TEST_OUT", &out)]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        launched_dir(&out),
        cwd.path()
            .canonicalize()
            .expect("canonicalize working directory")
    );
}
