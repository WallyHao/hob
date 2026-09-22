// --- hob library ---
// Scriptable CLI for the AI chores of a single developer.
//
// The binary and the benchmarks share this entry point, so the startup budgets
// measure the real dispatch code instead of a copy of it.

mod cli;
mod driver;
mod effect;
mod exec;
mod lua;

pub mod provider;

pub use cli::{NAME, USAGE_EXIT, VERSION, run};
