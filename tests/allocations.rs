//! Heap budget for a cold start, measured with a counting allocator.
//!
//! Exactly one test on purpose: the counting allocator wraps the whole
//! process, so parallel tests in the same binary would pollute each other's
//! regions.
//!
//! The measured paths are CLI-only (`--version`, `--help`, an unknown name);
//! none of them loads the Lua runtime. See docs/budgets.md.

use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// Provisional: the measured peak is 2497 bytes, and the headroom above it is
/// deliberate. Re-derive (do not just raise) when startup changes shape.
const MAX_BYTES: usize = 4 * 1024;
const MAX_ALLOCATIONS: usize = 32;

fn measure(args: &[&str], expected_code: u8) -> Stats {
    let mut out = Vec::with_capacity(1024);
    let mut err = Vec::with_capacity(1024);
    let region = Region::new(GLOBAL);
    let code = hob::run(args.iter().copied(), &mut out, &mut err);
    let stats = region.change();
    assert_eq!(code, expected_code, "{args:?} exited with {code}");
    stats
}

#[test]
fn startup_stays_within_the_heap_budget() {
    let cases: [(&[&str], u8); 3] = [
        (&["--version"], 0),
        (&["--help"], 0),
        (&["--definitely-not-a-command"], hob::USAGE_EXIT),
    ];
    for (args, expected) in cases {
        let stats = measure(args, expected);
        assert!(
            stats.bytes_allocated <= MAX_BYTES && stats.allocations <= MAX_ALLOCATIONS,
            "{args:?} measured {stats:?}; budget is {MAX_BYTES} bytes, {MAX_ALLOCATIONS} allocations"
        );
    }
}
