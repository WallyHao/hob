// --- benches/startup.rs ---
//! Instruction counts for a cold start, measured under valgrind.
//!
//! Wall-clock timing in CI is noise, instruction counts are not, so these are
//! the numbers a regression gate compares against a saved baseline:
//! `cargo bench --bench startup -- --save-baseline main` records one, and
//! `--baseline main` compares against it.
//!
//! The absolute side of the startup budget lives in tests/allocations.rs and
//! `scripts/check_binary_size.sh`; these benchmarks cover the relative side: a
//! change that doubles startup instructions is a regression even while the
//! numbers stay under budget.

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

fn dispatch(args: &[&str]) -> u8 {
    let mut out = Vec::with_capacity(1024);
    let mut err = Vec::with_capacity(1024);
    let code = hob::run(args.iter().copied(), &mut out, &mut err);
    black_box(out);
    black_box(err);
    code
}

#[library_benchmark]
fn version() -> u8 {
    black_box(dispatch(&["--version"]))
}

#[library_benchmark]
fn help() -> u8 {
    black_box(dispatch(&["--help"]))
}

#[library_benchmark]
fn unknown_argument() -> u8 {
    black_box(dispatch(&["--definitely-not-a-command"]))
}

#[library_benchmark]
fn flow() -> u8 {
    black_box(dispatch(&["run", "benches/data/flow.lua"]))
}

library_benchmark_group!(
    name = startup;
    benchmarks = version, help, unknown_argument, flow
);

main!(library_benchmark_groups = startup);
