//! A throwaway project and configuration under the system temp directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::BIN;

/// A throwaway project and configuration under the system temp directory.
pub(crate) struct Sandbox {
    root: PathBuf,
    config: PathBuf,
}

impl Sandbox {
    pub(crate) fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("hob-store-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let config = root.join("config");
        fs::create_dir_all(root.join("project").join(".git")).expect("project marker");
        fs::create_dir_all(root.join("outside")).expect("outside dir");
        fs::create_dir_all(config.join("commands")).expect("config dir");
        Self { root, config }
    }

    pub(crate) fn project(&self) -> PathBuf {
        self.root.join("project")
    }

    /// A directory with no project above it, to exercise the user layer.
    pub(crate) fn outside(&self) -> PathBuf {
        self.root.join("outside")
    }

    pub(crate) fn commands(&self) -> PathBuf {
        self.project().join(".hob").join("commands")
    }

    pub(crate) fn user_commands(&self) -> PathBuf {
        self.config.join("commands")
    }

    pub(crate) fn write_project(&self, name: &str, source: &str) {
        write(&self.commands(), name, source);
    }

    pub(crate) fn write_user(&self, name: &str, source: &str) {
        write(&self.user_commands(), name, source);
    }

    pub(crate) fn write_project_lib(&self, name: &str, source: &str) {
        write(&self.project().join(".hob").join("lib"), name, source);
    }

    pub(crate) fn write_user_lib(&self, name: &str, source: &str) {
        write(&self.config.join("lib"), name, source);
    }

    pub(crate) fn run(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(BIN)
            .env("HOB_CONFIG_DIR", &self.config)
            .current_dir(cwd)
            .args(args)
            .output()
            .expect("spawn hob")
    }

    pub(crate) fn run_project(&self, args: &[&str]) -> Output {
        self.run(&self.project(), args)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Write a command file, creating the namespace directories a nested name needs.
fn write(commands: &Path, name: &str, source: &str) {
    let path = commands.join(format!("{name}.lua"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("commands dir");
    }
    fs::write(path, source).expect("command file");
}
