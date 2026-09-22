//! One flow file in a throwaway working directory, with its own configuration.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use super::BIN;

/// A flow named `flow.lua` plus the directories it resolves against.
pub(crate) struct Flow {
    dir: PathBuf,
    config: PathBuf,
}

impl Flow {
    pub(crate) fn new(tag: &str, source: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hob-flow-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let config = dir.join("config");
        fs::create_dir_all(config.join("commands")).expect("config dir");
        fs::write(dir.join("flow.lua"), source).expect("flow file");
        Self { dir, config }
    }

    pub(crate) fn dir(&self) -> &Path {
        &self.dir
    }

    /// Write a file inside the sandbox, creating parents.
    pub(crate) fn write(&self, relative: &str, contents: &str) {
        let path = self.dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent dirs");
        }
        fs::write(path, contents).expect("write file");
    }

    pub(crate) fn run(&self, args: &[&str]) -> Output {
        self.run_with(args, &[], None)
    }

    /// Run the flow with extra environment and optional standard input.
    pub(crate) fn run_with(
        &self,
        args: &[&str],
        env: &[(&str, &str)],
        stdin: Option<&str>,
    ) -> Output {
        let mut command = Command::new(BIN);
        command
            .env("HOB_CONFIG_DIR", &self.config)
            .current_dir(&self.dir)
            .arg("run")
            .arg("flow.lua")
            .args(args);
        for (name, value) in env {
            command.env(name, value);
        }
        let Some(input) = stdin else {
            return command.output().expect("spawn hob");
        };
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().expect("spawn hob");
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(input.as_bytes())
            .expect("write stdin");
        child.wait_with_output().expect("wait for hob")
    }
}

impl Drop for Flow {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}
