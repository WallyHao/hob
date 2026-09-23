# Budgets

Lightweight and efficient are not feelings; they are numbers with a gate
attached. This page is the register of those numbers. Rationale lives next to
each gate; this page is the index.

## Current gates

| Gate | Where | Budget | Measured | Kind |
| --- | --- | --- | --- | --- |
| Release binary size | `scripts/check_binary_size.sh` | 8 MiB | 5957 KiB | absolute |
| Peak heap, `hob --version` | `scripts/check_heap.sh` | 16 KiB | 1736 B | absolute |
| Peak heap, `hob run` (one-line flow) | `scripts/check_heap.sh` | 128 KiB | 69 KiB | absolute |
| Startup allocations | `tests/allocations.rs` | 4 KiB, 32 allocations | 983 B, 2 allocations | absolute |
| Normal dependencies | `scripts/check_deps.sh` | 150 crates, denylist | 108 crates | absolute |
| Startup instructions | `benches/startup.rs` | against a saved baseline | 1534 / 810 / 3521 | relative |
| Flow startup instructions | `benches/startup.rs` | against a saved baseline | 2472672 | relative |

Interrupt cleanup -- exit 130 with no surviving process group -- is asserted by
`tests/interrupt.rs` rather than held to a number.

All measurements are from the pinned toolchain in `rust-toolchain.toml`.
Massif and the counting allocator are deterministic; do not replace them with
wall-clock or RSS comparisons, which are not. The benchmarks additionally need
`iai-callgrind-runner`, which `just env` installs at the version locked in
`Cargo.lock`.

Size and heap only move when a feature becomes reachable from the binary: code
that nothing calls is stripped by LTO. The dependency count moves as soon as
the crates are linked in, reachable or not.

## The ratchet rule

Budgets move deliberately or not at all:

- Lower a budget as soon as the measurement improves; that is free.
- Raise one only in the pull request that adds the feature which needs it, and
  put the new measured number in the commit message.
- Never raise a budget to silence a regression. If a regression is acceptable,
  write down why in the same diff.

## Calibrating

Derive each number from a real measurement with roughly 2x headroom, then
tighten as the area stabilizes. A budget that is never approached is either a
guardrail doing its job or a dead number; re-read it when the layer it covers
grows. `HOB_BINARY_BUDGET_KIB`, `HOB_CLI_HEAP_BUDGET_KIB`,
`HOB_FLOW_HEAP_BUDGET_KIB` and `HOB_DEPS_BUDGET`
override the scripts for local experiments, never for CI.

## Not covered yet

These are designed but cannot exist before the feature they measure:

- Scaling ratio tests (10/100/1000 steps) for parse, plan and context assembly.
- Cost budgets: a model price table has to exist before a dollar cap can mean
  anything. Request and token caps (`--max-calls`, `--max-tokens`) landed with
  `tests/budget.rs` and need no number here.

Each lands in the same pull request as the feature, not after it.
