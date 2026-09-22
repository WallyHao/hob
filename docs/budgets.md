# Budgets

Lightweight and efficient are not feelings; they are numbers with a gate
attached. This page is the register of those numbers. Rationale lives next to
each gate; this page is the index.

## Current gates

| Gate | Where | Budget | Measured | Kind |
| --- | --- | --- | --- | --- |
| Release binary size | `scripts/check_binary_size.sh` | 1 MiB | 871 KiB | absolute |
| Peak heap, `hob --version` | `scripts/check_heap.sh` | 16 KiB | 1736 B | absolute |
| Peak heap, `hob run` (one-line flow) | `scripts/check_heap.sh` | 128 KiB | 40 KiB | absolute |
| Startup allocations | `tests/allocations.rs` | 4 KiB, 32 allocations | 502 B, 3 allocations | absolute |
| Normal dependencies | `scripts/check_deps.sh` | 150 crates, denylist | 100 crates | absolute |
| Startup instructions | `benches/startup.rs` | against a saved baseline | 1526 / 1761 / 3125 | relative |

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
grows. `HOB_BINARY_BUDGET_KIB`, `HOB_HEAP_BUDGET_KIB` and `HOB_DEPS_BUDGET`
override the scripts for local experiments, never for CI.

## Not covered yet

These are designed but cannot exist before the feature they measure:

- Scaling ratio tests (10/100/1000 steps) for parse, plan and context assembly.
- Cancellation latency and orphan-process checks for running effects.
- LLM request, retry and byte budgets against a local mock server.
- Instruction counts for flow startup (Lua VM plus preload), not just CLI
  dispatch.

Each lands in the same pull request as the feature, not after it.
