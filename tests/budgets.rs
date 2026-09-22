//! Process-level budgets for a cold start.
//!
//! A startup that writes files, touches the network or leaks processes is not
//! lightweight no matter how fast it is, so these tests run the real binary in
//! a sandbox and hold it to what a `--help` is allowed to do: print, exit, and
//! leave nothing behind.
//!
//! Thresholds are provisional until the runtime lands; see docs/budgets.md.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The binary under test, built by cargo for this integration test.
const BIN: &str = env!("CARGO_BIN_EXE_hob");

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("hob-budget-{}-{tag}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Run the binary in a sandbox: a fresh working directory and every location
/// the XDG spec lets a CLI write to redirected into a temp tree.
fn run_sandboxed(tag: &str, args: &[&str]) -> (Output, TempDir, TempDir) {
    let cwd = TempDir::new(&format!("{tag}-cwd"));
    let home = TempDir::new(&format!("{tag}-home"));
    let output = Command::new(BIN)
        .args(args)
        .current_dir(cwd.path())
        .env("HOME", home.path())
        .env("XDG_CACHE_HOME", home.path().join("cache"))
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env("XDG_STATE_HOME", home.path().join("state"))
        .output()
        .expect("spawn hob");
    (output, cwd, home)
}

fn entries(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .expect("read temp dir")
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    paths.sort();
    paths
}

#[test]
fn startup_is_side_effect_free() {
    for args in [&["--version"][..], &["--help"][..]] {
        let (output, cwd, home) = run_sandboxed("side-effect", args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(!output.stdout.is_empty(), "{args:?} printed nothing");
        assert!(entries(cwd.path()).is_empty(), "{args:?} wrote to cwd");
        assert!(entries(home.path()).is_empty(), "{args:?} wrote to home");
    }
}

#[test]
fn version_prints_name_and_version() {
    let (output, _cwd, _home) = run_sandboxed("version", &["--version"]);
    let expected = format!("hob {}\n", env!("CARGO_PKG_VERSION"));
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn help_prints_usage() {
    for args in [&[][..], &["--help"][..], &["-h"][..]] {
        let (output, _cwd, _home) = run_sandboxed("help", args);
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: hob"));
    }
}

#[test]
fn unknown_arguments_fail_with_usage_exit() {
    let (output, _cwd, _home) = run_sandboxed("unknown", &["--definitely-not-a-command"]);
    assert_eq!(output.status.code(), Some(i32::from(hob::USAGE_EXIT)));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown argument"));
}
