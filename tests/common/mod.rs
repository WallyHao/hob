//! Sandboxes shared by the integration tests: a project tree, a configuration
//! tree, and the binary wired to both through the environment.
//!
//! `dead_code` and `unused_imports` are allowed because every integration test
//! crate compiles the whole module and uses only the part it needs.

#![allow(dead_code, unused_imports)]

mod flow;
mod sandbox;

pub(crate) mod mock;

pub(crate) use flow::Flow;
pub(crate) use sandbox::Sandbox;

use std::process::Output;

pub(crate) const BIN: &str = env!("CARGO_BIN_EXE_hob");

pub(crate) fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub(crate) fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
